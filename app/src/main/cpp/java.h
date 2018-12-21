#ifndef _java_h_
#define _java_h_

#include <jni.h>
#include <android/log.h>

#include "v8.h"
#include "node.h"
#include "env.h"
#include "env-inl.h"
#include "node_object_wrap.h"

#include "context.h"

// extern "C" int getAndroidVersion(JNIEnv **);

namespace node {

    using v8::Local;
    using v8::Value;
    using v8::Array;
    using v8::Value;
    using v8::Handle;
    using v8::Object;
    using v8::String;
    using v8::Isolate;
    using v8::Function;
    using v8::Persistent;
    using v8::FunctionTemplate;
    using v8::FunctionCallbackInfo;
    using v8::PropertyCallbackInfo;

    namespace jvm {

        class JavaType : public ObjectWrap {
        public:
            JavaType(jclass, JNIEnv **);
            virtual ~JavaType();
            static Persistent<FunctionTemplate> constructor;
            static void Init(Isolate *);
            static void NewInstance(const FunctionCallbackInfo<Value> &);
            static void InitEnvironment(Isolate *, JNIEnv **);

            JNIEnv* GetCurrentJNIEnv() { return *_env; }
            jclass GetJavaClass() { return _klass; };
            jobject GetJavaInstance() { return _jinstance; };

        private:
            jclass _klass;
            JNIEnv **_env;
            jobject _jinstance;

            static void New(const FunctionCallbackInfo<Value> &);
            static void NamedGetter(Local<String>, const PropertyCallbackInfo<Value>&);
            static void NamedSetter(Local<String> , Local<Value>, const PropertyCallbackInfo<Value>&);
            static void Call(const FunctionCallbackInfo <Value> &);
            static void Enumerator(const PropertyCallbackInfo <Array> &);
            void InitJavaMethod(Isolate *, Local<Object>);
            static void ValueOfAccessor(Local <v8::String>, const v8::PropertyCallbackInfo<Value> &);
            static void ValueOf(const FunctionCallbackInfo <Value> &);
            static void ToString(const FunctionCallbackInfo <Value> &);
            static void ToStringAccessor(Local <String>, const PropertyCallbackInfo <Value> &);

            static Handle<Value> JavaNameGetter(JNIEnv *, const PropertyCallbackInfo<Value> &, const char *);
        };

    }  // anonymous namespace

} // namespace node

#endif // _java_h_
