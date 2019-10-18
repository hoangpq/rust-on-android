#include "wrapper.h"

Persistent<FunctionTemplate> JavaWrapper::constructor_;

void JavaWrapper::Init(Isolate *isolate_, Local<ObjectTemplate> exports) {
  Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate_, New);

  tpl->SetClassName(String::NewFromUtf8(isolate_, "Java"));
  tpl->InstanceTemplate()->SetInternalFieldCount(1);
  // tpl->InstanceTemplate()->SetCallAsFunctionHandler(Call, Handle<Value>());
  // tpl->InstanceTemplate()->SetNamedPropertyHandler(Getter, Setter);

  Local<ObjectTemplate> proto = tpl->PrototypeTemplate();
  proto->Set(String::NewFromUtf8(isolate_, "isField"),
             FunctionTemplate::New(isolate_, IsField));
  proto->Set(String::NewFromUtf8(isolate_, "isMethod"),
             FunctionTemplate::New(isolate_, IsMethod));

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
      string_t packageName = _new_string_t(package);

      if (info.Length() == 1) {
        wrapper->ptr_ = new_instance(packageName, nullptr, 0);
      } else {
        Local<Array> array = Local<Array>::Cast(info[1]);
        uint32_t argc = array->Length();

        auto *args = new value_t[argc];
        for (unsigned int i = 0; i < argc + 1; i++) {
          if (array->Has(i)) {
            if (array->Get(i)->IsInt32()) {
              args[i] = _new_int_value_(array->Get(i)->Uint32Value());
            }
          }
        }
        wrapper->ptr_ = new_instance(packageName, args, argc);
      }
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
