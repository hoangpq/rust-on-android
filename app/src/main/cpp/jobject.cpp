#include <v8.h>
#include "jobject.h"
#include "java.h"

namespace node {

    using v8::Local;
    using v8::Value;
    using v8::Handle;
    using v8::String;
    using v8::Isolate;
    using v8::Persistent;
    using v8::Exception;
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::EscapableHandleScope;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        Persistent<FunctionTemplate> JavaObject::constructor;

        JavaObject::JavaObject(
                jobject instance, jmethodID methodId) : _instance(instance),
                                                        _methodId(methodId) {}

        JavaObject::~JavaObject() {

        }

        void JavaObject::Init(Isolate *isolate) {
            // Prepare constructor template
            Local<FunctionTemplate> function_template = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> instance_template = function_template->InstanceTemplate();
            instance_template->SetInternalFieldCount(1);
            constructor.Reset(isolate, function_template);
        }

        void JavaObject::New(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            if (args.IsConstructCall()) {
                args.GetReturnValue().Set(args.This());
            } else {
                isolate->ThrowException(
                        String::NewFromUtf8(isolate, "Function is not constructor."));
            }
        }

        Local<Object> JavaObject::NewInstance(jobject jObject, jmethodID methodId, Isolate *isolate) {
            EscapableHandleScope scope(isolate);
            Handle<FunctionTemplate> _js_function_template =
                    Local<FunctionTemplate>::New(Isolate::GetCurrent(), JavaObject::constructor);
            Local<Object> instance = _js_function_template->GetFunction()->NewInstance();
            JavaObject *obj = new JavaObject(jObject, methodId);
            obj->Wrap(instance);
            return instance;
        }
    }
}
