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
#include "context.h"

#ifndef NODE_EXTENSION_H_
#define NODE_EXTENSION_H_

extern "C" jlong JNICALL Java_com_node_sample_MainActivity_createPointer(JNIEnv *, jobject);
extern "C" void JNICALL Java_com_node_sample_MainActivity_dropPointer(JNIEnv *, jobject, jlong);
extern "C" jstring JNICALL Java_com_node_sample_MainActivity_getUtf8String(JNIEnv *, jobject);
extern "C" jobject JNICALL Java_com_node_sample_MainActivity_getNativeObject(JNIEnv *, jobject);

namespace node {
    namespace loader {}
}

NodeContext g_ctx;
bool jvmInitialized = false;

#endif  // NODE_EXTENSION_H_
