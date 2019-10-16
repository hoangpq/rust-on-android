#ifndef JNI_WRAPPER_H_
#define JNI_WRAPPER_H_

#include <iostream>
#include <jni.h>

#include "../util/util.h"
#include "object_wrap.h"
#include "v8.h"

using namespace v8;

class JavaWrapper : public rust::ObjectWrap {

public:
  static void Init(Isolate *isolate_, Local<ObjectTemplate> exports);

    jobject GetInstance() { return instance_; }

    jlong GetInstancePtr() { return ptr_; }

private:
    explicit JavaWrapper(std::string package) : package_(package) {};

    ~JavaWrapper();

  static void New(const FunctionCallbackInfo<Value> &args);

  static void Getter(Local<String> property,
                     const PropertyCallbackInfo<Value> &info);

  static void Setter(Local<String> property, Local<Value> value,
                     const PropertyCallbackInfo<Value> &info);

  static void ToStringAccessor(Local<String> property,
                               const PropertyCallbackInfo<Value> &info);

  static void Call(const FunctionCallbackInfo<Value> &args);

  std::string package_;
    jobject instance_;
    jlong ptr_;

  static Persistent<FunctionTemplate> constructor_;
};

#endif // JNI_WRAPPER_H_
