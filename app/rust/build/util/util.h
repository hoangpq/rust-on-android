#ifndef JNI_UTIL_H_
#define JNI_UTIL_H_

#include "v8.h"
#include <iostream>
#include <jni.h>
#include <string.h>
#include <string>

using namespace v8;
using namespace std;

typedef void (*JNICallback)(void *, jlong);

typedef struct {
  const uint8_t *ptr;
  uint32_t len;
} string_t;

typedef union data_t {
  int32_t i;
  jlong s;
} data_t;

typedef struct value_t {
  data_t data;
  uint8_t t;
} value_t;

typedef struct {
  JNICallback callback;
  jlong callback_data_;
  bool jni_call_;
  jlong ptr;
  jlong name;
  value_t *args;
  uint32_t argc;
  Isolate *isolate_;
  Persistent<Context> *context_;
  uint32_t uuid;
} message_t;

using namespace v8;
using namespace std;

extern "C" {
jlong _rust_new_string(const char *);
jlong new_instance(string_t, const value_t *, uint32_t);
void instance_call_args(jlong, jlong, const value_t *, uint32_t,
                        const FunctionCallbackInfo<Value> &);
Local<Value> instance_call_callback(jlong, jlong, const value_t *, uint32_t);
void adb_debug(const char *);
}

string_t _new_string_t(const std::string &s);
value_t _new_int_value(uint32_t val);
value_t _new_string_value(char *, int);

std::string v8str(Local<String> input);
string_t v8string_t(Local<Value> input);

Local<Function> get_function(Local<Object> obj, Local<String> key);

#endif // JNI_UTIL_H_
