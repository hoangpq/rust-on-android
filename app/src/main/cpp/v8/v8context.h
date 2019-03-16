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

#include "../utils/utils.h"
#include "jsobject.h"

struct buf_s {
  void *data;
  size_t len;
};
typedef struct buf_s buf;

extern "C" jobject createTimeoutHandler(JNIEnv **);
extern "C" void postDelayed(JNIEnv **, jobject, jlong, jlong, jint);
extern "C" char* workerSendBytes(void*, size_t, void*);
extern "C" void* createCallback();

namespace node {

namespace av8 {

static JNIEnv *env_ = nullptr;

class V8Runtime {
public:
  Isolate *isolate_;
  Persistent<Context> context_;
};

} // namespace av8
} // namespace node

#endif
