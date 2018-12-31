#include <iostream>
#include <jni.h>
#include <v8.h>
#include <node.h>
#include <env.h>
#include <env-inl.h>
#include <uv.h>

#include "../utils/utils.h"

#ifndef NODE_CONTEXT_EMBEDDER_DATA_INDEX
#define NODE_CONTEXT_EMBEDDER_DATA_INDEX 32
#endif

namespace node {

    using namespace std;
    using namespace v8;

    class V8Runtime {
    public:
        Isolate *isolate_;
        Persistent<Context> context_;
    };

    Isolate *InitV8Isolate() {
        if (g_ctx.isolate_ == NULL) {
            // Create a new Isolate and make it the current one.
            Isolate::CreateParams create_params;
            create_params.array_buffer_allocator = ArrayBuffer::Allocator::NewDefaultAllocator();
            g_ctx.isolate_ = Isolate::New(create_params);
        }
        return g_ctx.isolate_;
    }

    jlong CreateRuntime() {
        V8Runtime *runtime = new V8Runtime();
        runtime->isolate_ = InitV8Isolate();

        Locker locker(runtime->isolate_);
        Isolate::Scope isolate_scope(runtime->isolate_);
        HandleScope handle_scope(runtime->isolate_);

        Local<Context> context = Context::New(runtime->isolate_);
        runtime->context_.Reset(runtime->isolate_, context);
        Context::Scope contextScope(context);
        return reinterpret_cast<jlong>(runtime);
    }

    extern "C" void JNICALL
    Java_com_node_v8_V8Context_init(JNIEnv *env, jclass klass) {
        InitV8Isolate();
    }

    extern "C" jobject JNICALL
    Java_com_node_v8_V8Context_create(JNIEnv *env, jclass klass) {
        jlong ptr = CreateRuntime();
        jmethodID constructor = env->GetMethodID(klass, "<init>", "(J)V");
        return env->NewObject(klass, constructor, ptr);
    }

    extern "C" void JNICALL
    Java_com_node_v8_V8Context_set(
            JNIEnv *env, jobject instance, jstring key, jintArray data) {

        jclass objClazz = env->GetObjectClass(instance);
        jfieldID field = env->GetFieldID(objClazz, "runtimePtr", "J");
        jlong ptr = env->GetLongField(instance, field);

        V8Runtime *runtime = reinterpret_cast<V8Runtime *>(ptr);
        Locker locker(runtime->isolate_);
        Isolate::Scope isolate_scope(runtime->isolate_);
        HandleScope handle_scope(runtime->isolate_);

        jsize len = env->GetArrayLength(data);
        jint *body = env->GetIntArrayElements(data, 0);

        Local<Context> context = Local<Context>::New(runtime->isolate_, runtime->context_);

        Context::Scope context_scope(context);
        Local<Array> array = Array::New(runtime->isolate_, 3);

        for (int i = 0; i < len; i++) {
            array->Set(static_cast<uint32_t>(i), Integer::New(runtime->isolate_, (int) body[i]));
        }
        std::string _key = Util::JavaToString(env, key);
        context->Global()->Set(String::NewFromUtf8(runtime->isolate_, _key.c_str()), array);
    }

    extern "C" jobject JNICALL
    Java_com_node_v8_V8Context_eval(
            JNIEnv *env, jobject instance, jstring script) {

        jclass objClazz = env->GetObjectClass(instance);
        jfieldID field = env->GetFieldID(objClazz, "runtimePtr", "J");
        jlong ptr = env->GetLongField(instance, field);

        V8Runtime *runtime = reinterpret_cast<V8Runtime *>(ptr);
        Locker locker(runtime->isolate_);
        Isolate::Scope isolate_scope(runtime->isolate_);
        HandleScope handle_scope(runtime->isolate_);

        std::string _script = Util::JavaToString(env, script);
        Local<Context> context = Local<Context>::New(
                runtime->isolate_, runtime->context_);

        Context::Scope scope_context(context);
        Local<String> source =
                String::NewFromUtf8(runtime->isolate_, _script.c_str(),
                                    NewStringType::kNormal).ToLocalChecked();

        Local<Object> result = Script::Compile(context, source)
                .ToLocalChecked()->Run(context).ToLocalChecked()->ToObject();

        jclass resultClass = env->FindClass("com/node/v8/V8Context$V8Result");
        jmethodID constructor = env->GetMethodID(resultClass, "<init>", "(JJ)V");

        Persistent<Object> *container = new Persistent<Object>;
        container->Reset(runtime->isolate_, result);

        jlong resultPtr = reinterpret_cast<jlong>(container);
        return env->NewObject(resultClass, constructor, resultPtr, ptr);
    }

    extern "C" jobjectArray
    Java_com_node_v8_V8Context_00024V8Result_toIntegerArray(JNIEnv *env, jobject instance) {

        jclass objClazz = env->GetObjectClass(instance);
        jfieldID runtimePtrField = env->GetFieldID(objClazz, "runtimePtr", "J");
        jfieldID resultPtrField = env->GetFieldID(objClazz, "resultPtr", "J");

        jlong runtimePtr = env->GetLongField(instance, runtimePtrField);
        jlong resultPtr = env->GetLongField(instance, resultPtrField);

        V8Runtime *runtime = reinterpret_cast<V8Runtime *>(runtimePtr);
        Locker locker(runtime->isolate_);
        Isolate::Scope isolate_scope(runtime->isolate_);
        HandleScope handle_scope(runtime->isolate_);

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

    extern "C" jobject
    Java_com_node_v8_V8Context_00024V8Result_toInteger(JNIEnv *env, jobject instance) {

        jclass objClazz = env->GetObjectClass(instance);
        jfieldID runtimePtrField = env->GetFieldID(objClazz, "runtimePtr", "J");
        jfieldID resultPtrField = env->GetFieldID(objClazz, "resultPtr", "J");

        jlong runtimePtr = env->GetLongField(instance, runtimePtrField);
        jlong resultPtr = env->GetLongField(instance, resultPtrField);

        V8Runtime *runtime = reinterpret_cast<V8Runtime *>(runtimePtr);
        Locker locker(runtime->isolate_);
        Isolate::Scope isolate_scope(runtime->isolate_);
        HandleScope handle_scope(runtime->isolate_);

        Handle<Object> result = Local<Object>::New(
                runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

        jclass integerClass = env->FindClass("java/lang/Integer");
        jmethodID constructor = env->GetMethodID(integerClass, "<init>", "(I)V");
        return env->NewObject(integerClass, constructor, result->Int32Value());
    }

}
