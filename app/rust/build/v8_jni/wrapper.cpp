#include "wrapper.h"
#include <unistd.h>

Persistent<FunctionTemplate> JavaWrapper::constructor_;
Persistent<Function> JavaWrapper::registerUITask_;
Persistent<Function> JavaWrapper::resolverUITask_;
Persistent<Context> JavaWrapper::resolverContext_;

void JavaWrapper::Init(Isolate* isolate_, Local<ObjectTemplate> exports) {
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, New);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "Java"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);

  Local<ObjectTemplate> proto = tpl->PrototypeTemplate();
  proto->Set(String::NewFromUtf8(isolate_, "isField"),
             FunctionTemplate::New(isolate_, IsField));
  proto->Set(String::NewFromUtf8(isolate_, "isMethod"),
             FunctionTemplate::New(isolate_, IsMethod));

  constructor_.Reset(isolate_, tpl);
  exports->Set(String::NewFromUtf8(isolate_, "Java"), tpl);

  exports->Set(String::NewFromUtf8(isolate_, "$invokeJavaFn"),
               FunctionTemplate::New(isolate_, InvokeJavaFunction));
}

void JavaWrapper::SetContext(Local<Context> context_) {
  resolverContext_.Reset(Isolate::GetCurrent(), context_);
}

void JavaWrapper::New(const FunctionCallbackInfo<Value>& info) {
  assert(info[0]->IsString());
  Isolate* isolate = Isolate::GetCurrent();

  if (info.IsConstructCall()) {
    std::string package = v8str(info[0]->ToString(isolate));
    auto* wrapper = new JavaWrapper(package);

    if (package == "context") {
      wrapper->ptr_ = get_current_activity();
    } else {
      uint32_t argc = 0;
      value_t* args = nullptr;
      string_t packageName = _new_string_t(package);
      wrapper->ptr_ = new_instance(packageName, args, argc);
      delete args;
    }

    wrapper->Wrap(info.This());
    info.GetReturnValue().Set(info.This());
  }
}

void JavaWrapper::IsMethod(const FunctionCallbackInfo<Value>& args) {
  assert(args[0]->IsString());
  Isolate* isolate_ = args.GetIsolate();
  auto* wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(args.This());

  args.GetReturnValue().Set(
      Boolean::New(isolate_, is_method(wrapper->ptr_, v8string_t(args[0]))));
}

void JavaWrapper::IsField(const FunctionCallbackInfo<Value>& args) {
  assert(args[0]->IsString());
  Isolate* isolate_ = args.GetIsolate();
  auto* wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(args.This());

  args.GetReturnValue().Set(
      Boolean::New(isolate_, is_field(wrapper->ptr_, v8string_t(args[0]))));
}

void JavaWrapper::Call(const FunctionCallbackInfo<Value>& info) {
  Isolate* isolate_ = info.GetIsolate();
  info.GetReturnValue().Set(Undefined(isolate_));
}

void JavaWrapper::InvokeJavaFunction(const FunctionCallbackInfo<Value>& info) {
  assert(info[0]->IsObject());
  assert(info[1]->IsString());
  assert(info[2]->IsArray());

  Isolate* isolate_ = info.GetIsolate();
  auto* wrapper =
      rust::ObjectWrap::Unwrap<JavaWrapper>(info[0]->ToObject(isolate_));

  std::string method(v8str(info[1]->ToString(isolate_)));
  Local<Array> array = Local<Array>::Cast(info[2]);

  Local<Context> context = isolate_->GetCurrentContext();
  uint32_t argc = array->Length();
  auto* args = new value_t[argc];
  for (unsigned int i = 0; i < argc; i++) {
    if (array->Has(context, i).ToChecked()) {
      Local<Value> value = array->Get(i);
      if (value->IsInt32()) {
        args[i] = _new_int_value(value->Uint32Value(context).ToChecked());
      }
      if (value->IsString()) {
        String::Utf8Value val(isolate_, value->ToString(isolate_));
        args[i] = _new_string_value(*val, val.length());
      }
    }
  }

  Local<String> mainActivity = String::NewFromUtf8(isolate_, "activity");

  Local<Object> javaContext = info[0]->ToObject(isolate_);
  Local<String> contextName =
      javaContext->Get(context, String::NewFromUtf8(isolate_, "name"))
          .ToLocalChecked()
          ->ToString(isolate_);

  jlong name = _rust_new_string(method.c_str());

  instance_call_args(wrapper->ptr_, name, args, argc, info);
}

void JavaWrapper::CallbackRegister(Isolate* isolate_, Local<Context> context) {
  Local<Object> global = context->Global();

  resolverContext_.Reset(isolate_, context);

  Local<Function> register_ =
      get_function(global, String::NewFromUtf8(isolate_, "registerUITask"));
  registerUITask_.Reset(isolate_, register_);

  Local<Function> resolver_ =
      get_function(global, String::NewFromUtf8(isolate_, "resolverUITask"));
  resolverUITask_.Reset(isolate_, resolver_);
}

JavaWrapper::~JavaWrapper() { adb_debug("Destroyed"); }

void java_register_callback(Isolate* isolate_, Local<Context> context) {
  JavaWrapper::CallbackRegister(isolate_, context);
}