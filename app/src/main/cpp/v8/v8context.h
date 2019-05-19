#ifndef _v8context_h_
#define _v8context_h_

#include <android/log.h>
#include <env-inl.h>
#include <env.h>
#include <iostream>
#include <jni.h>
#include <node.h>
#include <uv.h>
#include <v8.h>

#include "../lib/node-ext.h"
#include "../utils/utils.h"
#include "jsobject.h"
#include <stdio.h>

namespace node {
namespace av8 {

extern "C" {
jobject createTimeoutHandler(JNIEnv **);
void postDelayed(JNIEnv **, jobject, jlong, jlong, jint);
char *workerSendBytes(void *, size_t, Local<Value> val);
void Perform(const FunctionCallbackInfo<Value> &);
void *createRuntime();
void initRuntime(JNIEnv **, void *);
void setInterval(void *);
};

class V8Runtime {
public:
  JNIEnv *env_;
  jobject holder_;
  Isolate *isolate_;
  Persistent<Context> context_;

public:
  static jclass V8_CONTEXT_CLASS;
  static jmethodID V8_CONTEXT_SHOW_ITEM_COUNT_METHOD;
};

struct GlobalContext {
  Isolate *isolate_;
  Persistent<Context> globalContext_;
  Persistent<ObjectTemplate> globalObject_;
  JNIEnv *env_; // Main thread
  void *rt;
};

static GlobalContext ctx_;

} // namespace av8
} // namespace node

#endif
