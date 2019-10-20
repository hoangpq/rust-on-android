#include "wrapper.h"
#include <unistd.h>

extern "C" {
void write_message(const void *, size_t count);
}

Persistent<FunctionTemplate> JavaWrapper::constructor_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, New);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "Java"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);

  Local<ObjectTemplate> proto = tpl->PrototypeTemplate();
  proto->Set(String::NewFromUtf8(isolate_, "isField"),
             FunctionTemplate::New(isolate_, IsField));
  proto->Set(String::NewFromUtf8(isolate_, "isMethod"),
             FunctionTemplate::New(isolate_, IsMethod));
  proto->Set(String::NewFromUtf8(isolate_, "testMethod"),
             FunctionTemplate::New(isolate_, TestMethod));

  constructor_.Reset(isolate_, tpl);
  exports->Set(String::NewFromUtf8(isolate_, "Java"), tpl);
}

void JavaWrapper::New(const FunctionCallbackInfo<Value> &info) {
  assert(info[0]->IsString());

  if (info.IsConstructCall()) {
    std::string package = v8str(info[0]->ToString());
    auto *wrapper = new JavaWrapper(package);

    if (package == "context") {
      wrapper->ptr_ = get_current_activity();
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
  read(fd, &msg, sizeof(message_t)); // read message from pip

  char ss[100];
  sprintf(ss, "receive: %lld", msg.args[0].data.s);
  adb_debug(ss);

  test_method(msg.ptr, msg.args, msg.argc);
  return 1;
}

void JavaWrapper::TestMethod(const FunctionCallbackInfo<Value> &info) {
  Isolate *isolate_ = info.GetIsolate();
  auto *wrapper = rust::ObjectWrap::Unwrap<JavaWrapper>(info.This());
  // schedule to run on main thread
  message_t msg;
  msg.ptr = wrapper->ptr_;
  msg.argc = 1;

  String::Utf8Value val(info[0]->ToString());
  msg.args = new value_t[1];
  msg.args[0] = _new_string_value(*val, val.length());;

  char ss[100];
  sprintf(ss, "send: %lld", msg.args[0].data.s);
  adb_debug(ss);

  write_message(&msg, sizeof(message_t));
  info.GetReturnValue().Set(Undefined(isolate_));
}

void JavaWrapper::Call(const FunctionCallbackInfo<Value> &info) {
  Isolate *isolate_ = info.GetIsolate();
  info.GetReturnValue().Set(Undefined(isolate_));
}

JavaWrapper::~JavaWrapper() { adb_debug("Destroyed"); }
