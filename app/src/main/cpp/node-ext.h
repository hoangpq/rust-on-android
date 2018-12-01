#include <jni.h>
#include <stddef.h>
#include <stdint.h>
#include <string>
#include <cstdlib>
#include <pthread.h>
#include <unistd.h>
#include <android/log.h>

#include "v8.h"
#include "env.h"
#include "env-inl.h"
#include "node_buffer.h"
#include "node.h"

#ifndef NODE_EXTENSION_H_
#define NODE_EXTENSION_H_

static const char *kTAG = "NodeJS Runtime";

#define LOGI(...) \
  ((void)__android_log_print(ANDROID_LOG_INFO, kTAG, __VA_ARGS__))

#define LOGE(...) \
  ((void)__android_log_print(ANDROID_LOG_ERROR, kTAG, __VA_ARGS__))

typedef struct node_context {
    JavaVM *javaVM;
    jclass mainActivityClz;
    jobject mainActivityObj;
} NodeContext;
NodeContext g_ctx;

extern "C" jlong JNICALL Java_com_node_sample_MainActivity_createPointer(JNIEnv *, jobject);

extern "C" void JNICALL Java_com_node_sample_MainActivity_dropPointer(JNIEnv *, jobject, jlong);

extern "C" jstring JNICALL Java_com_node_sample_MainActivity_getUtf8String(JNIEnv *, jobject);

extern "C" jobject JNICALL Java_com_node_sample_MainActivity_getNativeObject(JNIEnv *, jobject);

namespace node {

    using v8::Value;
    using v8::FunctionCallbackInfo;

    namespace loader {
        void InitJavaEnv(JNIEnv **env, const FunctionCallbackInfo<Value> &args);
    }
}

#endif  // NODE_EXTENSION_H_
