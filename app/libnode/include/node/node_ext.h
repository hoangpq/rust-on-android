#ifndef SRC_NODE_EXTENSION_H_
#define SRC_NODE_EXTENSION_H_

#include "node_buffer.h"
#include "v8.h"
#include "node.h"
#include "env.h"
#include "env-inl.h"

namespace node {

    using v8::Isolate;
    using v8::Local;
    using v8::Value;
    using v8::Object;
    using v8::FunctionCallbackInfo;

    namespace extension {
        void InitExtension(Local<Object> target);
        static void Log(const FunctionCallbackInfo<Value> &args);
        static void Toast(const FunctionCallbackInfo<Value> &args);
    }  // namespace extension

}  // namespace node

#endif  // SRC_NODE_EXTENSION_H_
