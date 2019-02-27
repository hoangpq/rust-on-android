#include <v8.h>
#include "jsobject.h"
#include "../java/java.h"
#include "v8context.h"

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

        Persistent<FunctionTemplate> JSObject::constructor_;

        JSObject::JSObject() = default;

        JSObject::~JSObject() = default;

        void JSObject::Init(Isolate *isolate) {
            Local<FunctionTemplate> ft_ = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> it_ = ft_->InstanceTemplate();
            it_->SetInternalFieldCount(1);
            it_->SetNamedPropertyHandler(NamedGetter);
            it_->SetCallAsFunctionHandler(Call, Handle<Value>());
            constructor_.Reset(isolate, ft_);
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
        void JSObject::NewInstance(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate_ = args.GetIsolate();
            Handle<FunctionTemplate> _function_template =
                    Local<FunctionTemplate>::New(isolate_, constructor_);

            Local<Object> instance_ = _function_template->GetFunction()->NewInstance();
            auto *wrapper = new JSObject();
            wrapper->Wrap(instance_);

            args.GetReturnValue().Set(instance_);
        }

        void JSObject::NamedGetter(Local<String> key, const PropertyCallbackInfo<Value> &info) {
            Isolate *isolate = info.GetIsolate();
            EscapableHandleScope scope(isolate);
            String::Utf8Value m(key->ToString());
            string methodName(*m);
            info.GetReturnValue().Set(info.This());
        }

        void JSObject::Call(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            args.GetReturnValue().Set(String::NewFromUtf8(isolate, "Call"));
        }

    }
}
