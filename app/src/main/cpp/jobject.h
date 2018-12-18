#ifndef _jobject_h_
#define _jobject_h_

#include <jni.h>
#include <android/log.h>

#include "v8.h"
#include "node.h"
#include "node_object_wrap.h"

#include "context.h"

namespace node {

    using v8::Value;
    using v8::Local;
    using v8::Handle;
    using v8::Object;
    using v8::Isolate;
    using v8::Persistent;
    using v8::FunctionTemplate;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        class JavaObject : public ObjectWrap {
        public:
            static Persistent<FunctionTemplate> constructor;
            JavaObject(jobject, jmethodID);
            virtual ~JavaObject();
            static void Init(Isolate *isolate);
            static Local<Object> NewInstance(jobject ,jmethodID, Isolate *);

        private:
            jobject _instance;
            jmethodID _methodId;
            static void New(const FunctionCallbackInfo<Value> &args);
        };

    }  // anonymous namespace

} // namespace node

#endif // _jobject_h_
