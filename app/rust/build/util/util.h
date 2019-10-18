#ifndef JNI_UTIL_H_
#define JNI_UTIL_H_

#include "v8.h"
#include <iostream>
#include <jni.h>
#include <string>

using namespace v8;
using namespace std;

typedef struct {
    const uint8_t *ptr;
    uint32_t len;
} string_t;

typedef struct {
    jvalue value;
    uint8_t t;
} value_t;

using namespace v8;
using namespace std;

extern "C" {
jlong new_instance(string_t, const value_t *, uint32_t);
void instance_call(jlong, string_t, const value_t *, uint32_t,
                   const FunctionCallbackInfo<Value> &);
void adb_debug(const char *);
}

string_t _new_string_t(const std::string &s);

value_t _new_int_value_(uint32_t val);

std::string v8str(Local<String> input);

string_t v8string_t(Local<Value> input);

#endif // JNI_UTIL_H_
