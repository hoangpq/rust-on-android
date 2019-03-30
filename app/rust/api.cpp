#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <stdio.h>
#include <v8.h>

using namespace v8;

extern "C" Local<Function> v8_function_cast(Local<Value> *val) {
  return Local<Function>::Cast(*val);
}

extern "C" void v8_function_call(Local<v8::Function> fun, int32_t argc,
                                 Local<Value> argv[]) {
  Isolate *isolate_ = Isolate::GetCurrent();
  fun->Call(isolate_->GetCurrentContext(), Null(isolate_), argc, argv);
}

extern "C" Local<ArrayBuffer> v8_buffer_new(void *data, size_t byte_length) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return ArrayBuffer::New(isolate_, data, byte_length,
                          ArrayBufferCreationMode::kInternalized);
}

#ifdef __cplusplus
#endif

#endif
