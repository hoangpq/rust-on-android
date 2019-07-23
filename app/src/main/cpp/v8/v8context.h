#ifndef _v8context_h_
#define _v8context_h_

#include <android/log.h>
#include <env-inl.h>
#include <env.h>
#include <iostream>
#include <jni.h>
#include <node.h>
#include <stdio.h>

#include <uv.h>
#include <v8.h>

#include "../utils/utils.h"

namespace node {
namespace av8 {

extern "C" {
jobject createTimeoutHandler(JNIEnv **);
void postDelayed(JNIEnv **, jobject, jlong, jlong, jint);
char *workerSendBytes(void *, size_t, Local<Value> val);

void init_event_loop(JNIEnv **);
void setInterval(void *);
};

class V8Runtime {
public:
  JNIEnv *env_;
  jobject holder_;
  Isolate *isolate_;
  Persistent<Context> context_;
};

struct GlobalContext {
  Isolate *isolate_;
  Persistent<Context> globalContext_;
  Persistent<ObjectTemplate> globalObject_;
  JNIEnv *env_; // Main thread
};

static GlobalContext ctx_;

} // namespace av8
} // namespace node

#endif
