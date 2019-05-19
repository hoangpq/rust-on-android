#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <assert.h>
#include <stdio.h>
#include <v8.h>
#include <jni.h>

using namespace v8;

extern "C" Local<Function> v8_function_cast(Local<Value> v) {
  return Local<Function>::Cast(v);
}

extern "C" void v8_function_call(Local<Function> fn, int32_t argc,
                                 Local<Value> argv[]) {
  Isolate *isolate_ = Isolate::GetCurrent();
  fn->Call(isolate_->GetCurrentContext(), Null(isolate_), argc, argv);
}

extern "C" Local<ArrayBuffer> v8_buffer_new(void *data, size_t byte_length) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return ArrayBuffer::New(isolate_, data, byte_length,
                          ArrayBufferCreationMode::kInternalized);
}

extern "C" Local<Value>
v8_function_callback_info_get(FunctionCallbackInfo<Value> *info,
                              int32_t index) {
  return (*info)[index];
}

extern "C" int32_t
v8_function_callback_length(FunctionCallbackInfo<Value> *info) {
  return info->Length();
}

extern "C" void v8_utf8_string_new(Local<String> *out, const uint8_t *data,
                                   int32_t len) {
  Isolate *isolate_ = Isolate::GetCurrent();
  String::NewFromUtf8(isolate_, (const char *)data, NewStringType::kNormal, len)
          .ToLocal(out);
}

extern "C" void v8_set_return_value(FunctionCallbackInfo<Value> *info,
                                    Local<Value> *value) {
  info->GetReturnValue().Set(*value);
}

extern "C" Local<String> v8_string_new_from_utf8(const char *data) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return String::NewFromUtf8(isolate_, data);
}

extern "C" void executeFunction(void *f) {
  Isolate *isolate_ = Isolate::GetCurrent();
  Persistent<Function> *fn = reinterpret_cast<Persistent<Function> *>(f);
  Local<Function> func = fn->Get(isolate_);
  func->Call(isolate_->GetCurrentContext(), Null(isolate_), 0, nullptr);
}

extern "C" void CallStaticVoidMethod(JNIEnv **env, jclass c, jmethodID m) {
  (*env)->CallStaticVoidMethod(c, m);
  // (*env)->FindClass("java/lang/String");
}

#ifdef __cplusplus
#endif

#endif
