#include <iostream>
#include <jni.h>
#include <v8.h>
#include <node.h>
#include <env.h>
#include <env-inl.h>
#include <uv.h>

#include "../utils.h"

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

    ArrayBuffer::Allocator *allocator;
    uint32_t zero_fill_field_ = 1;

    uint32_t *zero_fill_field() { return &zero_fill_field_; }

    class IsolateData {
    public:
        IsolateData(Isolate *isolate, uv_loop_t *event_loop, uint32_t *zero_fill_field) {};
    };

    Isolate *InitV8Isolate() {
        if (g_ctx.isolate_ == NULL) {
            std::cout << "current isolate is null before locker" << std::endl;
            // Create a new Isolate and make it the current one.
            Isolate::CreateParams create_params;
            allocator = ArrayBuffer::Allocator::NewDefaultAllocator();
            create_params.array_buffer_allocator = allocator;
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

        Isolate *isolate_ = runtime->isolate_;
        jsize len = env->GetArrayLength(data);
        jint *body = env->GetIntArrayElements(data, 0);

        Locker locker(isolate_);
        Isolate::Scope isolate_scope(isolate_);
        HandleScope handle_scope(isolate_);

        Local<Context> context = Local<Context>::New(
                runtime->isolate_, runtime->context_);

        Context::Scope context_scope(context);
        Local<Array> array = Array::New(isolate_, 3);

        // Fill out the values
        for (int i = 0; i < len; i++) {
            array->Set(static_cast<uint32_t>(i), Integer::New(isolate_, (int) body[i]));
        }

        std::string _key = Util::JavaToString(env, key);
        context->Global()->Set(String::NewFromUtf8(isolate_, _key.c_str()), array);

    }

    extern "C" jstring JNICALL
    Java_com_node_v8_V8Context_eval(
            JNIEnv *env, jobject instance, jstring script) {

        jclass objClazz = env->GetObjectClass(instance);
        jfieldID field = env->GetFieldID(objClazz, "runtimePtr", "J");
        jlong ptr = env->GetLongField(instance, field);

        V8Runtime *runtime = reinterpret_cast<V8Runtime *>(ptr);

        Isolate *isolate_ = runtime->isolate_;
        std::string _script = Util::JavaToString(env, script);

        // lock isolate
        Locker locker(isolate_);
        Isolate::Scope isolate_scope(isolate_);
        HandleScope handle_scope(isolate_);

        Local<Context> context = Local<Context>::New(
                runtime->isolate_, runtime->context_);
        Context::Scope scope_context(context);

        Local<String> source =
                String::NewFromUtf8(isolate_, _script.c_str(),
                                    NewStringType::kNormal).ToLocalChecked();

        // Run the script to get the result.
        Local<Value> result = Script::Compile(context, source)
                .ToLocalChecked()->Run(context).ToLocalChecked();

        String::Utf8Value utf8(result);
        return env->NewStringUTF(*utf8);
    }
}
