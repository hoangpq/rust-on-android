#include "wrapper.h"

string_t _new_string_t(const std::string &s) {
  string_t st;
  st.ptr = reinterpret_cast<const uint8_t *>(s.c_str());
  st.len = static_cast<uint32_t>(s.length());
  return st;
}

value_t _new_value_t(uint32_t val) {
  value_t value;
  value.value.i = (jint)val;
  value.t = 0;
  return value;
}

Persistent<FunctionTemplate> JavaWrapper::constructor_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
  // Java interoperable
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, New);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "Java"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);
  tpl->InstanceTemplate()->SetCallAsFunctionHandler(Call, Handle<Value>());
  tpl->InstanceTemplate()->SetAccessor(
      String::NewFromUtf8(isolate_, "toString", String::kInternalizedString),
      ToStringAccessor);

  tpl->InstanceTemplate()->SetNamedPropertyHandler(Getter, Setter);

  constructor_.Reset(isolate_, tpl);
  exports->Set(String::NewFromUtf8(isolate_, "Java"), tpl);
}

std::string v8str(Local<String> input) {
  String::Utf8Value val(input);
  std::string s(*val);
  return s;
}

void JavaWrapper::New(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsString());

  if (args.IsConstructCall()) {
    std::string package = v8str(args[0]->ToString());
    auto *wrapper = new JavaWrapper(package);

    bool isMethodConstructor =
        args[1]->IsBoolean() ? args[1]->BooleanValue() : false;

    if (!isMethodConstructor) {
      wrapper->value_.l = new_instance(_new_string_t(package));
      wrapper->class_ = new_class(_new_string_t(package));
    }

    wrapper->Wrap(args.This());
    args.GetReturnValue().Set(args.This());
  }
}

void JavaWrapper::Getter(Local<String> property,
                         const PropertyCallbackInfo<Value> &info) {
  Isolate *isolate_ = info.GetIsolate();

  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(info.This());

  Local<FunctionTemplate> constructor_ =
      Local<FunctionTemplate>::New(isolate_, JavaWrapper::constructor_);

  const int argc = 2;
  Local<Value> argv[argc] = {
      String::NewFromUtf8(isolate_, wrapper->package_.c_str()),
      Boolean::New(isolate_, true)};

  Local<Object> obj =
      constructor_->GetFunction()
          ->NewInstance(isolate_->GetCurrentContext(), argc, argv)
          .ToLocalChecked();

  auto *method = new JavaWrapper(wrapper->package_);
  method->package_ = wrapper->package_;
  method->method_ = v8str(property);
  method->value_ = wrapper->value_;
  method->Wrap(obj);

  info.GetReturnValue().Set(obj);
}

void JavaWrapper::Setter(Local<String> property, Local<Value> value,
                         const PropertyCallbackInfo<Value> &info) {}

void JavaWrapper::ToStringAccessor(Local<String> property,
                                   const PropertyCallbackInfo<Value> &info) {}

void JavaWrapper::Call(const FunctionCallbackInfo<Value> &info) {
  Isolate *isolate = info.GetIsolate();

  int argc = info.Length();
  auto *args = new value_t[argc];

  for (int i = 0; i < argc; i++) {
    if (info[i]->IsNumber()) {
      args[i] = _new_value_t(info[i]->Uint32Value());
    }
  }

  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(info.This());
  instance_call(wrapper->value_.l, _new_string_t(wrapper->method_), args,
                static_cast<uint32_t>(argc), info);

  delete args;
}
