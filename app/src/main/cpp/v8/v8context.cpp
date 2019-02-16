#include "v8context.h"
#include <unistd.h>

#define LockV8Context(env, instance) \
    jclass objClazz = env->GetObjectClass(instance);\
    jfieldID field = env->GetFieldID(objClazz, "runtimePtr", "J");\
    jlong runtimePtr = env->GetLongField(instance, field);

#define LockV8Result(env, instance) \
    jclass objClazz = env->GetObjectClass(instance);\
    jfieldID runtimePtrField = env->GetFieldID(objClazz, "runtimePtr", "J");\
    jfieldID resultPtrField = env->GetFieldID(objClazz, "resultPtr", "J");\
    jlong runtimePtr = env->GetLongField(instance, runtimePtrField);\
    jlong resultPtr = env->GetLongField(instance, resultPtrField);

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
    using jvm::JSObject;

    Isolate *InitV8Isolate() {
        if (g_ctx.isolate_ == NULL) {
            // Create a new Isolate and make it the current one.
            Isolate::CreateParams create_params;
            create_params.array_buffer_allocator = ArrayBuffer::Allocator::NewDefaultAllocator();
            g_ctx.isolate_ = Isolate::New(create_params);

            Locker locker(g_ctx.isolate_);
            Isolate::Scope isolate_scope(g_ctx.isolate_);
            HandleScope handle_scope(g_ctx.isolate_);

            JSObject::Init(g_ctx.isolate_);
        }
        return g_ctx.isolate_;
    }

    void Sleep(const FunctionCallbackInfo<Value> &args) {
        // sleep(static_cast<unsigned int>(args[0]->Int32Value()));
        args.GetReturnValue().Set(args[1]);
    }

    jlong CreateRuntime() {
        V8Runtime *runtime = new V8Runtime();
        runtime->isolate_ = InitV8Isolate();

        Locker locker(g_ctx.isolate_);
        Isolate::Scope isolate_scope(g_ctx.isolate_);
        HandleScope handle_scope(g_ctx.isolate_);

        Local<ObjectTemplate> global = ObjectTemplate::New(g_ctx.isolate_);
        global->Set(String::NewFromUtf8(g_ctx.isolate_, "sleep"),
                    FunctionTemplate::New(g_ctx.isolate_, Sleep));

        Local<Context> context = Context::New(runtime->isolate_, NULL, global);
        runtime->context_.Reset(runtime->isolate_, context);
        Context::Scope contextScope(context);
        return reinterpret_cast<jlong>(runtime);
    }

    Handle<Object> RunScript(Isolate *isolate, Local<Context> context, std::string _script) {
        Local<String> source =
                String::NewFromUtf8(isolate, _script.c_str(),
                                    NewStringType::kNormal).ToLocalChecked();
        Local<Object> result = Script::Compile(context, source)
                .ToLocalChecked()->Run(context).ToLocalChecked()->ToObject();
        return result;
    }

    void SetV8Key(Isolate *isolate, Local<Context> context, std::string key, Local<Value> value) {
        context->Global()->Set(String::NewFromUtf8(isolate, key.c_str()), value);
    }

    extern "C" void JNICALL
    Java_com_node_v8_V8Context_init(JNIEnv *env, jclass klass) { InitV8Isolate(); }

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
        jint *body = env->GetIntArrayElements(data, 0);

        Local<Array> array = Array::New(runtime->isolate_, 3);
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

        Persistent<Object> *container = new Persistent<Object>;
        container->Reset(runtime->isolate_, result);

        jlong resultPtr = reinterpret_cast<jlong>(container);
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
            return NULL;
        }

        Local<Array> jsArray(Handle<Array>::Cast(result));
        jobjectArray array = env->NewObjectArray(jsArray->Length(), integerClass, NULL);
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

    extern "C" JNIEXPORT jobject JNICALL
    Java_com_node_v8_V8Context_00024V8Result_toPromise(JNIEnv *env, jobject instance) {

        LockV8Result(env, instance);
        LockIsolate(runtimePtr);

        Handle<Object> result = Local<Object>::New(
                runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

        Local<Promise> promise(Handle<Promise>::Cast(result));
        SetV8Key(runtime->isolate_, context, "__p", promise);

        jclass promiseClass = env->FindClass("com/node/v8/V8Promise");
        jmethodID constructor = env->GetMethodID(promiseClass, "<init>", "(JJ)V");
        return env->NewObject(promiseClass, constructor, resultPtr, runtimePtr);
    }

    extern "C" JNIEXPORT void JNICALL
    Java_com_node_v8_V8Promise_then(JNIEnv *env, jobject instance, jobject observer) {

        LockV8Result(env, instance);
        LockIsolate(runtimePtr);

        Handle<Object> result = Local<Object>::New(
                runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

        Local<Promise> promise(Handle<Promise>::Cast(result));
        Local<Value> pResult = promise->Result();

        auto *container = new Persistent<Object>;
        container->Reset(runtime->isolate_, pResult->ToObject(runtime->isolate_));

        jclass observableClass = env->GetObjectClass(observer);
        jmethodID subscribe = env->GetMethodID(
                observableClass, "subscribe", "(Ljava/lang/Object;)V");

        SetV8Key(runtime->isolate_, context, "__fn", JSObject::NewInstance(
                runtime->isolate_, observer, subscribe, runtimePtr));
        RunScript(runtime->isolate_, context, "__p.then(__fn)");
    }

}
