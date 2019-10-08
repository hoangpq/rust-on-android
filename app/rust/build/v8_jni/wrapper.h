#include <iostream>
#include <jni.h>

#include "object_wrap.h"
#include "v8.h"

using namespace v8;

extern "C" {
jobject new_integer(int32_t);
}

class JavaWrapper : public rust::ObjectWrap {

public:
  static void Init(Isolate *isolate_, Local<ObjectTemplate> exports);

private:
  explicit JavaWrapper(std::string packageName) : p_(std::move(packageName)){};
  ~JavaWrapper() override = default;

  static void JNew(const FunctionCallbackInfo<Value> &args);

  static void Getter(Local<String> property,
                     const PropertyCallbackInfo<Value> &info);

  static void Setter(Local<String> property, Local<Value> value,
                     const PropertyCallbackInfo<void> &info);

  std::string p_;
  jobject obj_;

  static Persistent<FunctionTemplate> constructor_;
};
