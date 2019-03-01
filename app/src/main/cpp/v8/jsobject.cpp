#include <v8.h>
#include <utility>
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

        using util::Util;

        Persistent<FunctionTemplate> JSObject::constructor_;

        JSObject::JSObject(jclass c) : class_(c) {};

        JSObject::~JSObject() = default;

        void JSObject::Init(Isolate *isolate) {
            Local<FunctionTemplate> ft_ = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> it_ = ft_->InstanceTemplate();
            it_->SetInternalFieldCount(1);
            it_->SetNamedPropertyHandler(NamedGetter);
            it_->SetCallAsFunctionHandler(Call, Handle<Value>());
            it_->Set(Util::ConvertToV8String("typeOf"), FunctionTemplate::New(isolate, TypeOf));
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

        Handle<Object>
        JSObject::NewInstance(Isolate *isolate_, jclass class_) {
            Handle<FunctionTemplate> _function_template =
                    Local<FunctionTemplate>::New(isolate_, constructor_);

            Local<Object> instance_ = _function_template->GetFunction()->NewInstance();

            auto *wrapper = new JSObject(class_);
            JavaType::InitEnvironment(isolate_, &wrapper->env_);

            wrapper->Wrap(instance_);
            return instance_;
        }

        void JSObject::NamedGetter(Local<String> key, const PropertyCallbackInfo<Value> &info) {
            Isolate *isolate_ = info.GetIsolate();
            String::Utf8Value m(key->ToString());
            string method_(*m);

            auto *wrapper = ObjectWrap::Unwrap<JSObject>(info.Holder());
            Handle<Object> instance_ = NewInstance(isolate_, wrapper->class_);

            info.GetReturnValue().Set(instance_);
        }

        void JSObject::Call(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate_ = args.GetIsolate();

            JNIEnv *env = nullptr;
            JavaType::InitEnvironment(isolate_, &env);

            auto *wrapper = ObjectWrap::Unwrap<JSObject>(args.Holder());
            auto type_ = Util::GetPackageName(env, wrapper->class_);
            wrapper->type_ = type_;
            args.GetReturnValue().Set(Util::ConvertToV8String(type_));
        }

        void JSObject::TypeOf(const FunctionCallbackInfo<Value> &args) {
            auto wrapper = ObjectWrap::Unwrap<JSObject>(args.Holder());
            args.GetReturnValue().Set(Util::ConvertToV8String(wrapper->type_));
        }

    }
}
