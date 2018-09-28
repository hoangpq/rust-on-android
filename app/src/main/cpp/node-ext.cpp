#include "v8.h"
#include "node.h"
#include "node_ext.h"

#include <stddef.h>
#include <stdint.h>

#include <jni.h>
#include <string>
#include <cstdlib>
#include <pthread.h>
#include <unistd.h>

typedef struct node_context {
    JavaVM *javaVM;
    jclass mainActivityClz;
    jobject mainActivityObj;
} NodeContext;
NodeContext g_ctx;

namespace node {

    using v8::Context;
    using v8::Local;
    using v8::Number;
    using v8::String;
    using v8::Object;
    using v8::Value;
    using v8::Isolate;
    using v8::Exception;
    using v8::HandleScope;
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::FunctionCallbackInfo;

    namespace loader {

        const char *ToCString(Local<String> str) {
            String::Utf8Value value(str);
            return *value ? *value : "<string conversion failed>";
        }

        static void Toast(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            JNIEnv *env;
            jint res = g_ctx.javaVM->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6);
            if (res != JNI_OK) {
                res = g_ctx.javaVM->AttachCurrentThread(&env, NULL);
                if (JNI_OK != res) {
                    args.GetReturnValue()
                            .Set(String::NewFromUtf8(isolate, "Unable to invoke activity!"));
                }
            }
            Local<String> str = args[0]->ToString();
            const char *msg = ToCString(str);

            jmethodID methodId = env->GetMethodID(g_ctx.mainActivityClz, "toast",
                                                  "(Ljava/lang/String;)V");

            jstring javaMsg = env->NewStringUTF(msg);
            env->CallVoidMethod(g_ctx.mainActivityObj, methodId, javaMsg);
            env->DeleteLocalRef(javaMsg);
            args.GetReturnValue().Set(str);
        }

        class ModuleWrap {
        public:
            static void Initialize(v8::Local<v8::Object> target,
                                   v8::Local<v8::Value> unused,
                                   v8::Local<v8::Context> context);
        };

        class AndroidModuleWrap : public ModuleWrap {
        public:
            static void Initialize(Local<Object> target,
                                   Local<Value> unused,
                                   Local<Context> context,
                                   void *priv) {

                ModuleWrap::Initialize(target, unused, context);
                // define function in global context
                v8::Isolate *isolate = target->GetIsolate();
                Local<Object> global = context->Global();
                global->Set(String::NewFromUtf8(isolate, "$toast"),
                            FunctionTemplate::New(isolate, loader::Toast)->GetFunction());
            }

        };
    }
}

JNIEXPORT jint JNI_OnLoad(JavaVM *vm, void *reserved) {
    JNIEnv *env;
    memset(&g_ctx, 0, sizeof(g_ctx));
    if (vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6) != JNI_OK) {
        return JNI_ERR; // JNI version not supported.
    }
    g_ctx.javaVM = vm;
    g_ctx.mainActivityObj = NULL;
    return JNI_VERSION_1_6;
}

JNIEXPORT void JNI_OnUnload(JavaVM *vm, void *reserved) {
    JNIEnv *env;
    if (vm->GetEnv((void **) &env, JNI_VERSION_1_6) != JNI_EDETACHED) {
        vm->DetachCurrentThread();
    }
}

extern "C" void JNICALL
Java_com_node_sample_MainActivity_initVM(
        JNIEnv *env,
        jobject instance) {

    // init objects
    jclass clz = env->GetObjectClass(instance);
    g_ctx.mainActivityClz = (jclass) env->NewGlobalRef(clz);
    g_ctx.mainActivityObj = env->NewGlobalRef(instance);
}

extern "C" void JNICALL
Java_com_node_sample_MainActivity_releaseVM(
        JNIEnv *env,
        jobject /* this */) {

    // release allocated objects
    env->DeleteGlobalRef(g_ctx.mainActivityClz);
    env->DeleteGlobalRef(g_ctx.mainActivityObj);
    g_ctx.mainActivityObj = NULL;
    g_ctx.mainActivityClz = NULL;
}

NODE_MODULE_CONTEXT_AWARE_BUILTIN(module_wrap, node::loader::AndroidModuleWrap::Initialize)
