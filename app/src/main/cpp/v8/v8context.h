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

#include "../lib/node-ext.h"
#include "../utils/utils.h"
#include "jsobject.h"

namespace node {
namespace av8 {

extern "C" {
jobject createTimeoutHandler(JNIEnv **);
void postDelayed(JNIEnv **, jobject, jlong, jlong, jint);
char *workerSendBytes(void *, size_t, Local<Value> val);
void Perform(const FunctionCallbackInfo<Value> &);

void init_event_loop(JNIEnv **);
void setInterval(void *);
};

struct isolate;

class Deno {
public:
  Isolate *isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
  isolate *rust_isolate_{};

  explicit Deno(Isolate *isolate) : isolate_(isolate) {}

  Deno(Isolate *isolate, Local<Context> context, Local<ObjectTemplate> global)
      : isolate_(isolate) {
    this->context_.Reset(this->isolate_, context);
    this->global_.Reset(this->isolate_, global);
  }

  void ResetContext(Local<Context> c) {
    this->context_.Reset(this->isolate_, c);
  }
  void ResetTemplate(Local<ObjectTemplate> t) {
    this->global_.Reset(this->isolate_, t);
  }

  void *Into() { return reinterpret_cast<void *>(this); }
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
