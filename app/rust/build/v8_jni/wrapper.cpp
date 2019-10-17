#include "wrapper.h"

Persistent<FunctionTemplate> JavaWrapper::constructor_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, New);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "Java"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);
    // tpl->InstanceTemplate()->SetCallAsFunctionHandler(Call, Handle<Value>());
    // tpl->InstanceTemplate()->SetNamedPropertyHandler(Getter, Setter);
  constructor_.Reset(isolate_, tpl);
  exports->Set(String::NewFromUtf8(isolate_, "Java"), tpl);
}

void JavaWrapper::New(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsString());

  if (args.IsConstructCall()) {
    std::string package = v8str(args[0]->ToString());
    auto *wrapper = new JavaWrapper(package);

      wrapper->ptr_ = new_instance(_new_string_t(package));
    wrapper->Wrap(args.This());

    args.GetReturnValue().Set(args.This());
  }
}

void JavaWrapper::Getter(Local<String> property,
                         const PropertyCallbackInfo<Value> &info) {
    info.GetReturnValue().Set(info.This());
}

void JavaWrapper::Setter(Local<String> property, Local<Value> value,
                         const PropertyCallbackInfo<Value> &info) {}

void JavaWrapper::ToStringAccessor(Local<String> property,
                                   const PropertyCallbackInfo<Value> &info) {}

void JavaWrapper::Call(const FunctionCallbackInfo<Value> &info) {
    Isolate *isolate_ = info.GetIsolate();
    info.GetReturnValue().Set(Undefined(isolate_));
}

JavaWrapper::~JavaWrapper() { adb_debug("Destroyed"); }
