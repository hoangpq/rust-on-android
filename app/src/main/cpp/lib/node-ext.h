#ifndef _node_extension_h_
#define _node_extension_h_

#include <android/log.h>
#include <android/looper.h>
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

using namespace util;

extern "C" jstring JNICALL
Java_com_node_sample_MainActivity_getUtf8String(JNIEnv *, jobject);

extern "C" void init_event_loop();
extern "C" void register_vm(JavaVM *vm);

NodeContext g_ctx;
static ALooper *mainThreadLooper;
static int messagePipe[2];

extern "C" int looperCallback(int fd, int events, void *data);
extern "C" JNIEnv *get_main_thread_env();
extern "C" void write_message(const void *, size_t count);

namespace node {

namespace loader {
void AndroidToast(const FunctionCallbackInfo<Value> &args);
void AndroidLog(const FunctionCallbackInfo<Value> &args);
void AndroidError(const FunctionCallbackInfo<Value> &args);
void OnLoad(const FunctionCallbackInfo<Value> &args);
} // namespace loader
} // namespace node

#endif
