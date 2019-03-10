#ifndef _v8context_h_
#define _v8context_h_

#include <iostream>
#include <jni.h>
#include <v8.h>
#include <node.h>
#include <env.h>
#include <env-inl.h>
#include <uv.h>
#include <android/log.h>

#include "jsobject.h"
#include "../utils/utils.h"

extern "C" void postDelayed(JNIEnv **, jobject, jlong, jlong);

namespace node {

    namespace av8 {

        static JNIEnv *env_ = nullptr;

        class V8Runtime {
        public:
            Isolate *isolate_;
            Persistent<Context> context_;
        };

    }
}

#endif
