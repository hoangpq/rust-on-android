#include <v8.h>
#include "jsobject.h"
#include "java.h"

namespace node {

    using v8::Local;
    using v8::Value;
    using v8::Number;
    using v8::Handle;
    using v8::String;
    using v8::Isolate;
    using v8::Boolean;
    using v8::Persistent;
    using v8::Undefined;
    using v8::Exception;
    using v8::HandleScope;
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::EscapableHandleScope;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        Persistent<FunctionTemplate> JSObject::_func_wrapper;

        JSObject::JSObject() {}

        JSObject::~JSObject() {}

        void JSObject::Init(Isolate *isolate) {
            Local<FunctionTemplate> function_template = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> instance_template = function_template->InstanceTemplate();
            instance_template->SetInternalFieldCount(1);
            instance_template->SetCallAsFunctionHandler(Call, Handle<Value>());
            _func_wrapper.Reset(isolate, function_template);
        }

        void JSObject::New(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            if (args.IsConstructCall()) {
                args.GetReturnValue().Set(args.This());
            } else {
                isolate->ThrowException(
                        String::NewFromUtf8(isolate, "Function is not constructor."));
            }
        }

        Local<Value> JSObject::NewInstance(Isolate *isolate) {
            Handle<FunctionTemplate> _function_template =
                    Local<FunctionTemplate>::New(isolate, _func_wrapper);

            Local<Object> jsInst = _function_template->GetFunction()->NewInstance();
            JSObject *wrapper = new JSObject();
            wrapper->Wrap(jsInst);
            return Local<Value>::New(isolate, jsInst);
        }

        void JSObject::Call(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            HandleScope scope(isolate);
            JSObject *wrapper = ObjectWrap::Unwrap<JSObject>(args.This());

            args.GetReturnValue().Set(Undefined(isolate));
        }

    }
}
