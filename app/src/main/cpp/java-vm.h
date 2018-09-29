#ifndef SRC_JAVA_VM_H_
#define SRC_JAVA_VM_H_

#include "v8.h"
#include "node.h"
#include "env.h"
#include "env-inl.h"
#include "node_object_wrap.h"

namespace node {


    namespace {

        class JavaType : public node::ObjectWrap {
        public:
            static void Init(v8::Isolate *isolate);

            static void NewInstance(const v8::FunctionCallbackInfo<v8::Value> &args);

        private:
            explicit JavaType(double value = 0);

            ~JavaType();

            static void New(const v8::FunctionCallbackInfo<v8::Value> &args);

            static void PlusOne(const v8::FunctionCallbackInfo<v8::Value> &args);

            static v8::Persistent<v8::Function> constructor;
            double value_;
        };


    }  // anonymous namespace

} // namespace node

#endif // SRC_JAVA_VM_H_
