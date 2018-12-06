#ifndef JAVA_H_
#define JAVA_H_

#include <jni.h>
#include <v8.h>
#include "v8.h"
#include "node.h"
#include "env.h"
#include "env-inl.h"
#include "node_object_wrap.h"
#include "context.h"

extern "C" int getAndroidVersion(JNIEnv **);

namespace node {

    using v8::Local;
    using v8::Value;
    using v8::Array;
    using v8::Object;
    using v8::String;
    using v8::Isolate;
    using v8::Function;
    using v8::Persistent;
    using v8::FunctionCallbackInfo;
    using v8::PropertyCallbackInfo;

    namespace jvm {

        class JavaType : public node::ObjectWrap {
        public:
            JavaType(char *className, NodeContext &ctx);
            virtual ~JavaType();
            static void Init(Isolate *isolate);
            static void NewInstance(const FunctionCallbackInfo<Value> &args);
            static void InitEnvironment(const FunctionCallbackInfo<Value> &args, JNIEnv **env);
            void WrapObject(Local<Object> handle);
        public:
            JavaVM* getJavaVM() { return _ctx.javaVM; }
            JNIEnv* getJNIEnv() { return *_env; }
            char* getClassName() { return _className; };

        private:
            char *_className;
            NodeContext _ctx;
            JNIEnv** _env;
            static void New(const FunctionCallbackInfo<Value> &args);
            static Persistent<Function> constructor;
            static void NamedGetter(Local<String> js_key,
                                    const PropertyCallbackInfo<Value>& js_info);
            static void NamedSetter(Local<String> js_key, Local<Value> js_value,
                                    const PropertyCallbackInfo<Value>& js_info);
            static void Call(const FunctionCallbackInfo <Value> &js_args);
            static void Toast(const FunctionCallbackInfo<Value> &args);
            static void Version(const FunctionCallbackInfo<Value> &args);
            static void Enumerator(const PropertyCallbackInfo <Array> &js_info);
            static void ToStringAccessor(Local <String> js_property,
                                         const PropertyCallbackInfo <Value> &js_info);
            static void ValueOfAccessor(Local <String> js_property,
                                        const PropertyCallbackInfo <Value> &js_info);
        };

        void CreateJavaType(const FunctionCallbackInfo<Value> &args);

    }  // anonymous namespace

} // namespace node

#endif // JAVA_H_
