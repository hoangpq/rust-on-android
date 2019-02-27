#include "v8context.h"
#include <unistd.h>

#define LockV8Context(env, instance) \
    jclass objClazz = (env)->GetObjectClass(instance);\
    jfieldID field = (env)->GetFieldID(objClazz, "runtime__", "J");\
    jlong runtimePtr = (env)->GetLongField(instance, field);

#define LockV8Result(env, instance) \
    jclass objClazz = (env)->GetObjectClass(instance);\
    jfieldID runtimePtrField = (env)->GetFieldID(objClazz, "runtime__", "J");\
    jfieldID resultPtrField = (env)->GetFieldID(objClazz, "result__", "J");\
    jlong runtimePtr = (env)->GetLongField(instance, runtimePtrField);\
    jlong resultPtr = (env)->GetLongField(instance, resultPtrField);

#define LockIsolate(ptr) \
    V8Runtime* runtime = reinterpret_cast<V8Runtime*>(ptr);\
    Locker locker(runtime->isolate_);\
    Isolate::Scope isolate_scope(runtime->isolate_);\
    HandleScope handle_scope(runtime->isolate_);\
    Local<Context> context = Local<Context>::New(runtime->isolate_, runtime->context_);\
    Context::Scope context_scope(context);

namespace node {

    using namespace std;
    using namespace v8;
    using namespace util;

    using jvm::JSObject;

    void forName(const FunctionCallbackInfo<Value> &args) {
        Isolate *isolate_ = args.GetIsolate();

        JNIEnv *env = nullptr;
        jint res = g_ctx.javaVM->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6);
        if (res != JNI_OK) {
            res = g_ctx.javaVM->AttachCurrentThread(&env, nullptr);
            if (JNI_OK != res) {
                isolate_->ThrowException(Util::ConvertToV8String("Unable to invoke activity!"));
            }
        }
        jclass utilClass = env->FindClass("com/node/util/JNIUtils");
        jmethodID getClassMethodList = env->GetStaticMethodID(
                utilClass, "getClassMethodList", "(Ljava/lang/String;)[Ljava/lang/String;");

        Local<String> className = args[0]->ToString();
        String::Utf8Value s(className);

        auto arr = (jobjectArray) env->CallStaticObjectMethod(utilClass, getClassMethodList,
                                                              env->NewStringUTF(*s));

        jsize arrLength = env->GetArrayLength(arr);
        int len = int(arrLength);

        Local<Array> array = Array::New(isolate_, len);
        for (int i = 0; i < len; i++) {
            auto methodName = (jstring) env->GetObjectArrayElement(arr, static_cast<jsize>(i));
            array->Set(static_cast<uint32_t>(i),
                       Util::ConvertToV8String(Util::JavaToString(env, methodName)));
        }

        JSObject::NewInstance(args);
    }

    Isolate *InitV8Isolate() {
        if (g_ctx.isolate_ != nullptr) return g_ctx.isolate_;

        // Create a new Isolate and make it the current one.
        Isolate::CreateParams create_params;
        create_params.array_buffer_allocator = ArrayBuffer::Allocator::NewDefaultAllocator();
        Isolate *isolate_ = Isolate::New(create_params);

        Locker locker(isolate_);
        Isolate::Scope isolate_scope(isolate_);
        HandleScope handle_scope(isolate_);

        Local<ObjectTemplate> globalObject = ObjectTemplate::New(isolate_);

        Local<ObjectTemplate> class_ = ObjectTemplate::New(isolate_);
        class_->Set(Util::ConvertToV8String("forName"),
                    FunctionTemplate::New(isolate_, forName));

        globalObject->Set(Util::ConvertToV8String("Class"), class_);
        Local<Context> globalContext = Context::New(isolate_, nullptr, globalObject);

        g_ctx.isolate_ = isolate_;
        g_ctx.globalContext_.Reset(isolate_, globalContext);
        g_ctx.globalObject_.Reset(isolate_, globalObject);

        JSObject::Init(isolate_);

        return g_ctx.isolate_;
    }

    jlong CreateRuntime() {
        auto *runtime = new V8Runtime();
        runtime->isolate_ = InitV8Isolate();

        Locker locker(runtime->isolate_);
        Isolate::Scope isolate_scope(runtime->isolate_);
        HandleScope handle_scope(runtime->isolate_);

        Local<Context> context = Context::New(
                runtime->isolate_, nullptr, g_ctx.globalObject_.Get(runtime->isolate_));
        runtime->context_.Reset(runtime->isolate_, context);
        Context::Scope contextScope(context);

        return reinterpret_cast<jlong>(runtime);
    }

    Handle<Object> RunScript(Isolate *isolate, Local<Context> context, string _script) {
        Local<String> source =
                String::NewFromUtf8(isolate, _script.c_str(),
                                    NewStringType::kNormal).ToLocalChecked();
        Local<Script> script = Script::Compile(context, source).ToLocalChecked();
        Local<Value> value = script->Run(context).ToLocalChecked();
        return value->ToObject();
    }

    void SetV8Key(Isolate *isolate, Local<Context> context, const string &key, Local<Value> value) {
        context->Global()->Set(String::NewFromUtf8(isolate, key.c_str()), value);
    }

    extern "C" void JNICALL
    Java_com_node_v8_V8Context_init(JNIEnv *, jclass) { InitV8Isolate(); }

    extern "C" jobject JNICALL
    Java_com_node_v8_V8Context_create(JNIEnv *env, jclass klass) {
        jlong ptr = CreateRuntime();
        jmethodID constructor = env->GetMethodID(klass, "<init>", "(J)V");
        return env->NewObject(klass, constructor, ptr);
    }

    extern "C" JNIEXPORT void JNICALL
    Java_com_node_v8_V8Context_set(
            JNIEnv *env, jobject instance, jstring key, jintArray data) {

        LockV8Context(env, instance);
        LockIsolate(runtimePtr);
        jsize len = env->GetArrayLength(data);
        jint *body = env->GetIntArrayElements(data, nullptr);

        Local<Array> array = Array::New(runtime->isolate_, len);
        for (int i = 0; i < len; i++) {
            array->Set(static_cast<uint32_t>(i), Integer::New(runtime->isolate_, (int) body[i]));
        }
        std::string _key = Util::JavaToString(env, key);
        context->Global()->Set(String::NewFromUtf8(runtime->isolate_, _key.c_str()), array);
    }

    extern "C" JNIEXPORT jobject JNICALL
    Java_com_node_v8_V8Context_eval(
            JNIEnv *env, jobject instance, jstring script) {

        LockV8Context(env, instance);
        LockIsolate(runtimePtr);
        std::string _script = Util::JavaToString(env, script);

        Context::Scope scope_context(context);
        Local<Object> result = RunScript(runtime->isolate_, context, _script);

        jclass resultClass = env->FindClass("com/node/v8/V8Context$V8Result");
        jmethodID constructor = env->GetMethodID(resultClass, "<init>", "(JJ)V");

        auto *container = new Persistent<Object>;
        container->Reset(runtime->isolate_, result);

        auto resultPtr = reinterpret_cast<jlong>(container);
        return env->NewObject(resultClass, constructor, resultPtr, runtimePtr);
    }

    extern "C" JNIEXPORT jobjectArray JNICALL
    Java_com_node_v8_V8Context_00024V8Result_toIntegerArray(JNIEnv *env, jobject instance) {

        LockV8Result(env, instance);
        LockIsolate(runtimePtr);

        Handle<Object> result = Local<Object>::New(
                runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

        jclass integerClass = env->FindClass("java/lang/Integer");
        jmethodID constructor = env->GetMethodID(integerClass, "<init>", "(I)V");

        if (!result->IsArray()) {
            jclass Exception = env->FindClass("java/lang/Exception");
            env->ThrowNew(Exception, "Result is not an array!");
            return nullptr;
        }

        Local<Array> jsArray(Handle<Array>::Cast(result));
        jobjectArray array = env->NewObjectArray(jsArray->Length(), integerClass, nullptr);
        for (uint32_t i = 0; i < jsArray->Length(); i++) {
            env->SetObjectArrayElement(array, i, env->NewObject(integerClass, constructor,
                                                                jsArray->Get(i)->Int32Value()));
        }
        return array;
    }

    extern "C" JNIEXPORT jobject JNICALL
    Java_com_node_v8_V8Context_00024V8Result_toInteger(JNIEnv *env, jobject instance) {

        LockV8Result(env, instance);
        LockIsolate(runtimePtr);

        Handle<Object> result = Local<Object>::New(
                runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

        jclass integerClass = env->FindClass("java/lang/Integer");
        jmethodID constructor = env->GetMethodID(integerClass, "<init>", "(I)V");
        return env->NewObject(integerClass, constructor, result->Int32Value());
    }

    extern "C" JNIEXPORT jstring JNICALL
    Java_com_node_v8_V8Context_00024V8Result_toJavaString(JNIEnv *env, jobject instance) {
        LockV8Result(env, instance);
        LockIsolate(runtimePtr);

        Handle<Object> result = Local<Object>::New(
                runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

        String::Utf8Value s(result->ToString());
        return env->NewStringUTF(*s);
    }

}
