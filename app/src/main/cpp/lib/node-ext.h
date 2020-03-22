#ifndef _node_extension_h_
#define _node_extension_h_

#include "v8.h"
#include <android/log.h>
#include <android/looper.h>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <jni.h>
#include <pthread.h>
#include <string>
#include <unistd.h>

#include "../utils/utils.h"

using namespace util;

extern "C" void register_vm(JavaVM *vm);
extern "C" void register_native(JNIEnv *env);

NodeContext g_ctx;
static ALooper *mainThreadLooper;
static int messagePipe[2];

void AndroidToast(const FunctionCallbackInfo<v8::Value> &args);

void AndroidLog(const FunctionCallbackInfo<v8::Value> &args);

void AndroidError(const FunctionCallbackInfo<v8::Value> &args);

void OnLoad(const FunctionCallbackInfo<v8::Value> &args);

#endif
