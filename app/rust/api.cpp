#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <assert.h>
#include <stdio.h>
#include <v8.h>

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
v8_function_callback_info_get(FunctionCallbackInfo<Value> *info, int32_t index) {
  return (*info)[index];
}

extern "C" int32_t
v8_function_callback_length(FunctionCallbackInfo<Value> *info) {
  return info->Length();
}

#ifdef __cplusplus
#endif

#endif
