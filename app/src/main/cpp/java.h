#ifndef JAVA_H_
#define JAVA_H_

#include <jni.h>
#include <android/log.h>

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
    using v8::FunctionTemplate;
    using v8::FunctionCallbackInfo;
    using v8::PropertyCallbackInfo;

    namespace jvm {

        class JavaType : public node::ObjectWrap {
        public:
            JavaType(jclass, JNIEnv **);
            virtual ~JavaType();
            static Persistent<FunctionTemplate> constructor;
            static void Init(Isolate *isolate);
            static void NewInstance(const FunctionCallbackInfo<Value> &args);
            static void InitEnvironment(Isolate *isolate, JNIEnv **env);
            JNIEnv* GetCurrentJNIEnv() { return *_env; }

        private:
            jclass _klass;
            JNIEnv **_env;
            static void New(const FunctionCallbackInfo<Value> &args);
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
            void InitJavaMethod(Isolate *isolate, Local<Object>);
        };

    }  // anonymous namespace

} // namespace node

#endif // JAVA_H_
