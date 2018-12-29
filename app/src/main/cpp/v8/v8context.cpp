#include <iostream>
#include <jni.h>
#include <v8.h>
#include "../utils.h"

namespace v8 {

    using namespace std;
    using namespace v8;

    Persistent<Context> context_;

    Isolate *InitV8Isolate() {
        if (g_ctx.isolate_ == NULL) {
            std::cout << "current isolate is null before locker" << std::endl;
            // Create a new Isolate and make it the current one.
            Isolate::CreateParams create_params;
            create_params.array_buffer_allocator = ArrayBuffer::Allocator::NewDefaultAllocator();
            g_ctx.isolate_ = Isolate::New(create_params);
        }
        return g_ctx.isolate_;
    }

    extern "C" void JNICALL
    Java_com_node_v8_V8Context_init(JNIEnv *env, jclass klass) {
        InitV8Isolate();
    }

    extern "C" void JNICALL
    Java_com_node_v8_V8Context_set(
            JNIEnv *env, jclass klass, jstring key, jintArray data) {

        InitV8Isolate();
        Isolate *isolate_ = g_ctx.isolate_;
        jsize len = env->GetArrayLength(data);
        jint *body = env->GetIntArrayElements(data, 0);

        Locker locker(isolate_);
        Isolate::Scope isolate_scope(isolate_);
        HandleScope handle_scope(isolate_);

        Local<Context> context = Context::New(isolate_);
        context_.Reset(isolate_, context);

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
            JNIEnv *env, jclass klass, jstring script) {

        InitV8Isolate();
        Isolate *isolate_ = g_ctx.isolate_;
        std::string _script = Util::JavaToString(env, script);

        // lock isolate
        Locker locker(isolate_);
        Isolate::Scope isolate_scope(isolate_);
        HandleScope handle_scope(isolate_);

        TryCatch tryCatch(isolate_);
        Local<Context> context = Local<Context>::New(isolate_, context_);
        Context::Scope scope_context(context);

        Local<String> source =
                String::NewFromUtf8(isolate_, _script.c_str(),
                                    NewStringType::kNormal).ToLocalChecked();

        // Run the script to get the result.
        Local<Value> result = Script::Compile(context, source)
                .ToLocalChecked()->Run(context).ToLocalChecked();

        if (tryCatch.HasCaught()) {
            Local<String> _error = tryCatch.Exception()->ToString();
            String::Utf8Value error(_error);
            std::cout << *error << std::endl;
        }

        String::Utf8Value utf8(result);
        return env->NewStringUTF(*utf8);
    }
}
