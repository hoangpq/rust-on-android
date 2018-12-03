#ifndef SRC_JAVA_VM_H_
#define SRC_JAVA_VM_H_

#include <jni.h>
#include "v8.h"
#include "node.h"
#include "env.h"
#include "env-inl.h"
#include "node_object_wrap.h"

extern "C" int getAndroidVersion(JNIEnv **);

namespace node {

    using v8::Value;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        class JavaType : public node::ObjectWrap {
        public:

            JavaVM *_jvm;
            JNIEnv *_env;

            explicit JavaType(JavaVM **);

            static void Init(v8::Isolate *isolate);

            static void NewInstance(const v8::FunctionCallbackInfo<v8::Value> &args);

            inline void PWrap(v8::Local<v8::Object> handle) {
                Wrap(handle);
            }

            static void Toast(const v8::FunctionCallbackInfo<v8::Value> &args);

            static void InitEnvironment(const FunctionCallbackInfo<Value> &args, JNIEnv **env);

            ~JavaType();

        private:

            static void New(const v8::FunctionCallbackInfo<v8::Value> &args);

            static v8::Persistent<v8::Function> constructor;

            static void AndroidVersion(const FunctionCallbackInfo<Value> &args);
        };

        void CreateJavaType(const FunctionCallbackInfo<Value> &args);

    }  // anonymous namespace

} // namespace node

#endif // SRC_JAVA_VM_H_
