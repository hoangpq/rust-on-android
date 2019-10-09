#include "wrapper.h"

Persistent<FunctionTemplate> JavaWrapper::constructor_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
  // Java interoperable
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, JNew);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "java$"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);
  tpl->InstanceTemplate()->SetAccessor(String::NewFromUtf8(isolate_, "package"),
                                       Getter, Setter);

  constructor_.Reset(isolate_, tpl);
  exports->Set(String::NewFromUtf8(isolate_, "$java"), tpl);
}

void JavaWrapper::JNew(const FunctionCallbackInfo<Value> &args) {
  Local<String> name = args[0]->ToString();
  String::Utf8Value val(name);
  std::string val_s(*val);

  auto *wrapper = new JavaWrapper(val_s);
  // wrapper->v_ = new_integer(42);

  // jvalue j;
  // j.i = (jint)100;

  // static_call(j);

  wrapper->Wrap(args.This());

  args.GetReturnValue().Set(args.This());
}

void JavaWrapper::Getter(Local<String> property,
                         const PropertyCallbackInfo<Value> &info) {
  Isolate *isolate_ = info.GetIsolate();

  String::Utf8Value val(property);
  std::string val_s(*val);

  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(info.This());

  if (val_s == "package") {
    info.GetReturnValue().Set(
        String::NewFromUtf8(isolate_, wrapper->p_.c_str()));
  } else {
    info.GetReturnValue().Set(Undefined(isolate_));
  }
}

void JavaWrapper::Setter(Local<String> property, Local<Value> value,
                         const PropertyCallbackInfo<void> &info) {}

#pragma clang diagnostic pop