#include "wrapper.h"

string_t _new_string_t(std::string s) {
  string_t st;
  st.ptr = reinterpret_cast<const uint8_t *>(s.c_str());
  st.len = static_cast<uint32_t>(s.length());
  return st;
}

Persistent<FunctionTemplate> JavaWrapper::constructor_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
  // Java interoperable
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, New);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "java$"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);
  tpl->InstanceTemplate()->SetCallAsFunctionHandler(Call, Handle<Value>());
  tpl->InstanceTemplate()->SetAccessor(
      String::NewFromUtf8(isolate_, "toString", String::kInternalizedString),
      ToStringAccessor);

  tpl->InstanceTemplate()->SetNamedPropertyHandler(Getter, Setter);

  constructor_.Reset(isolate_, tpl);
  exports->Set(String::NewFromUtf8(isolate_, "$java"), tpl);
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

void JavaWrapper::Call(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate = args.GetIsolate();
  HandleScope handleScope(isolate);

  int argc = args.Length();
  jvalue *jargs = (jvalue *)malloc(argc * sizeof(jvalue));

  /*for (int i = 0; i < argc; i++) {
    jvalue val;
    if (args[i]->IsNumber()) {
      val.i = (jint)args[i]->Uint32Value();
    }
    jargs[i] = val;
  }*/

  jargs[0].i = (jint)100;


  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(args.This());
  jvalue val = instance_call(wrapper->value_.l, _new_string_t(wrapper->method_),
                             argc, jargs);

  args.GetReturnValue().Set(Number::New(isolate, (int)val.i));
}
