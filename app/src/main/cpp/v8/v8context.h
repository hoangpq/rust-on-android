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

#include "../java/jsobject.h"
#include "../utils/utils.h"

namespace node {

    class V8Runtime {
    public:
        Isolate *isolate_;
        Persistent<Context> context_;
    };
}

#endif // _v8context_h_
