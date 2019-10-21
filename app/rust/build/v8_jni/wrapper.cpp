#include "wrapper.h"
#include <unistd.h>

extern "C" {
void write_message(const void *, size_t count);
}

Persistent<FunctionTemplate> JavaWrapper::constructor_;
Persistent<Function> JavaWrapper::registerUITask_;
Persistent<Function> JavaWrapper::resolverUITask;
Persistent<Context> JavaWrapper::resolverContext_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
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

  // resolver
  Local<Context> context_ = resolverContext_.Get(isolate_);
  Context::Scope scope(context_);
  Local<Object> global = context_->Global();
  Local<Function> resolver_ =
          get_function(global, String::NewFromUtf8(isolate_, "registerUITask"));
  registerUITask_.Reset(isolate_, resolver_);
}

void JavaWrapper::SetContext(Local<Context> context_) {
  resolverContext_.Reset(Isolate::GetCurrent(), context_);
}

void JavaWrapper::New(const FunctionCallbackInfo<Value> &info) {
  assert(info[0]->IsString());

  if (info.IsConstructCall()) {
    std::string package = v8str(info[0]->ToString());
    auto *wrapper = new JavaWrapper(package);

    if (package == "context") {
      wrapper->ptr_ = get_current_activity();
      wrapper->context_ = true;
    } else {
      uint32_t argc = 0;
      value_t *args = nullptr;
      string_t packageName = _new_string_t(package);

      if (info[1]->IsArray()) {
        Local<Array> array = Local<Array>::Cast(info[1]);
        if (array->Length() > 0) {
          argc = array->Length();
          args = new value_t[argc];

          for (unsigned int i = 0; i < argc + 1; i++) {
            if (array->Has(i)) {
              if (array->Get(i)->IsInt32()) {
                args[i] = _new_int_value(array->Get(i)->Uint32Value());
              }
            }
          }
        }
      }

      wrapper->ptr_ = new_instance(packageName, args, argc);
      delete args;
    }

    wrapper->Wrap(info.This());
    info.GetReturnValue().Set(info.This());
  }
}

void JavaWrapper::IsMethod(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsString());
  Isolate *isolate_ = args.GetIsolate();
  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(args.This());

  args.GetReturnValue().Set(
          Boolean::New(isolate_, is_method(wrapper->ptr_, v8string_t(args[0]))));
}

void JavaWrapper::IsField(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsString());
  Isolate *isolate_ = args.GetIsolate();
  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(args.This());

  args.GetReturnValue().Set(
          Boolean::New(isolate_, is_field(wrapper->ptr_, v8string_t(args[0]))));
}

int looperCallback(int fd, int events, void *data) {
  message_t msg;
  read(fd, &msg, sizeof(message_t));

  int x = 10;
  auto info = reinterpret_cast<FunctionCallbackInfo<Value> *>(&x);
  instance_call(msg.ptr, msg.name, msg.args, msg.argc, *info, true);
  return 1;
}

void JavaWrapper::Call(const FunctionCallbackInfo<Value> &info) {
  Isolate *isolate_ = info.GetIsolate();
  info.GetReturnValue().Set(Undefined(isolate_));
}

void JavaWrapper::InvokeJavaFunction(const FunctionCallbackInfo<Value> &info) {
  assert(info[0]->IsObject());
  assert(info[1]->IsString());
  assert(info[2]->IsArray());

  Isolate *isolate_ = info.GetIsolate();
  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(info[0]->ToObject());
  std::string method(v8str(info[1]->ToString()));
  Local<Array> array = Local<Array>::Cast(info[2]);

  uint32_t argc = array->Length();
  auto *args = new value_t[argc];
  for (unsigned int i = 0; i < argc; i++) {
    if (array->Has(i)) {
      Local<Value> value = array->Get(i);
      if (value->IsInt32()) {
        args[i] = _new_int_value(value->Uint32Value());
      }
      if (value->IsString()) {
        String::Utf8Value val(value->ToString());
        args[i] = _new_string_value(*val, val.length());
      }
    }
  }

  jlong name = _rust_new_string(method.c_str());
  if (!wrapper->context_) {
    instance_call(wrapper->ptr_, name, args, argc, info, false);
  } else {
    message_t msg;
    msg.ptr = wrapper->ptr_;
    msg.name = name;
    msg.argc = argc;
    msg.args = args;
    write_message(&msg, sizeof(message_t));

    /*Local<Context> context_ = isolate_->GetCurrentContext();
    Local<Function> resolver_ = registerUITask_.Get(isolate_);
    Local<Value> argv[0] = {};
    Local<Object> result = Local<Object>::Cast(
        resolver_->Call(context_, Null(isolate_), 0, argv).ToLocalChecked());

    uint32_t uiTaskId =
        result->Get(String::NewFromUtf8(isolate_, "uiTaskId"))->Uint32Value();
    info.GetReturnValue().Set(
        result->Get(String::NewFromUtf8(isolate_, "promise")));*/
  }
}

JavaWrapper::~JavaWrapper() { adb_debug("Destroyed"); }
