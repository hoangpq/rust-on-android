#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <assert.h>
#include <stdio.h>
#include <v8.h>

using namespace v8;

extern "C" Local<Function> v8_function_cast(Local<Value> *val) {
  return Local<Function>::Cast(*val);
}

extern "C" void v8_function_call(Local<Function> *fn, int32_t argc,
                                 Local<Value> argv[]) {
  Isolate *isolate_ = Isolate::GetCurrent();
  (*fn)->Call(isolate_->GetCurrentContext(), Null(isolate_), argc, argv);
}

extern "C" void v8_function_call_no_args(Local<Function> *fn) {
  Isolate *isolate_ = Isolate::GetCurrent();
  Local<Value> argv[] = {};
  (*fn)->Call(isolate_->GetCurrentContext(), Null(isolate_), 0, argv);
}

extern "C" void v8_function_call_info(Local<Function> fun,
                                      const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate_ = args.GetIsolate();
  Local<Value> argv[] = {};
  fun->Call(isolate_->GetCurrentContext(), Null(isolate_), 0, argv);
}

extern "C" Local<ArrayBuffer> v8_buffer_new(void *data, size_t byte_length) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return ArrayBuffer::New(isolate_, data, byte_length,
                          ArrayBufferCreationMode::kInternalized);
}

extern "C" Local<Value>
v8_function_callback_info_get(const FunctionCallbackInfo<Value> *info,
                              int32_t index) {
  return (*info)[index];
}

extern "C" int32_t
v8_function_callback_length(FunctionCallbackInfo<Value> *info) {
  return info->Length();
}

extern "C" void v8_set_undefined_return(FunctionCallbackInfo<Value> *info) {
  Isolate *isolate_ = Isolate::GetCurrent();
  info->GetReturnValue().Set(Undefined(isolate_));
}

extern "C" void v8_perform(Local<Function> *fn, FunctionCallbackInfo<Value> *args) {
  Isolate *isolate_ = Isolate::GetCurrent();
  Local<Value> argv[] = {};
  (*fn)->Call(isolate_->GetCurrentContext(), Null(isolate_), 0, argv);
}

#ifdef __cplusplus
#endif

#endif
