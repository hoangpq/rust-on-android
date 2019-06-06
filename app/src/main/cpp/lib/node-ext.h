#ifndef _node_extension_h_
#define _node_extension_h_

#include <android/log.h>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <jni.h>
#include <pthread.h>
#include <string>
#include <unistd.h>

#include "env-inl.h"
#include "env.h"
#include "node.h"
#include "node_buffer.h"
#include "v8.h"

#include "../utils/utils.h"

extern "C" jlong JNICALL
Java_com_node_sample_MainActivity_createPointer(JNIEnv *, jobject);
extern "C" jstring JNICALL
Java_com_node_sample_MainActivity_getUtf8String(JNIEnv *, jobject);
extern "C" void onNodeServerLoaded(JNIEnv **, jobject);

namespace node {

namespace loader {
void AndroidToast(const FunctionCallbackInfo<Value> &args);
void AndroidLog(const FunctionCallbackInfo<Value> &args);
void AndroidError(const FunctionCallbackInfo<Value> &args);
void OnLoad(const FunctionCallbackInfo<Value> &args);
} // namespace loader
} // namespace node

#endif
