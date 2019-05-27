#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <assert.h>
#include <jni.h>
#include <stdio.h>
#include <v8.h>

using namespace v8;

class V8Runtime {
public:
  JNIEnv *env_;
  jobject holder_;
  Isolate *isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
};

struct isolate;

class Deno {
public:
  Isolate *isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
  isolate *rust_isolate_;

  Deno(Isolate *isolate, Local<Context> context, Local<ObjectTemplate> global)
      : isolate_(isolate) {
    this->context_.Reset(this->isolate_, context);
    this->global_.Reset(this->isolate_, global);
  }

  void *Into() { return reinterpret_cast<void *>(this); }
};

const char *jStringToChar(JNIEnv *env, jstring name) {
  const char *str = env->GetStringUTFChars(name, 0);
  env->ReleaseStringUTFChars(name, str);
  return str;
}

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

extern "C" const char *fetchEvent(JNIEnv **env, jclass c, jmethodID m) {
  jstring s = (jstring)(*env)->CallStaticObjectMethod(c, m);
  const char *result = jStringToChar(*env, s);
  (*env)->DeleteLocalRef(s);
  return result;
}

const char *ToCString(const String::Utf8Value &value) {
  return *value ? *value : "<string conversion failed>";
}

// Rust bridge
extern "C" {
void adb_debug(const char *);
void create_timer(isolate *, const void *);
isolate *create_isolate(const void *);
}

static void WeakCallback(const WeakCallbackInfo<int> &data) {
  adb_debug("weak callback");
}

void Log(const FunctionCallbackInfo<Value> &args) {
  assert(args.Length() > 0);
  String::Utf8Value utf8(args[0]);
  adb_debug(ToCString(utf8));
}

void Timeout(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsFunction());
  Isolate *isolate = args.GetIsolate();
  // void *denoPtr = args.Data().As<External>()->Value();

  auto deno = static_cast<Deno *>(args.Data().As<External>()->Value());
  auto fn = new Persistent<Function>(isolate, Local<Function>::Cast(args[0]));
  fn->SetWeak<int>(new int(10), WeakCallback, WeakCallbackType::kParameter);

  create_timer(deno->rust_isolate_, reinterpret_cast<void *>(fn));
  // fn->Reset();
  // isolate->RequestGarbageCollectionForTesting(Isolate::kFullGarbageCollection);
}

extern "C" void invokeFunction(void *ptr, void *f) {
  auto deno = reinterpret_cast<Deno *>(ptr);
  auto fn = reinterpret_cast<Persistent<Function> *>(f);
  Local<Function> cb = fn->Get(deno->isolate_);
  Local<Context> context_ = deno->context_.Get(deno->isolate_);
  const unsigned argc = 1;
  Local<Value> argv[argc] = {
      String::NewFromUtf8(deno->isolate_, "Invoked", NewStringType::kNormal)
          .ToLocalChecked()};
  cb->Call(context_, Null(deno->isolate_), argc, argv).ToLocalChecked();
}

extern "C" isolate *initDeno(void *ptr) {
  auto deno = reinterpret_cast<Deno *>(ptr);
  Locker locker(deno->isolate_);
  Isolate::Scope isolate_scope(deno->isolate_);

  Local<External> env_ = External::New(deno->isolate_, deno);

  Local<ObjectTemplate> global_ = deno->global_.Get(deno->isolate_);
  global_->Set(String::NewFromUtf8(deno->isolate_, "$timeout"),
               FunctionTemplate::New(deno->isolate_, Timeout, env_));

  global_->Set(String::NewFromUtf8(deno->isolate_, "$log"),
               FunctionTemplate::New(deno->isolate_, Log, env_));

  Local<Context> context_ = Context::New(deno->isolate_, nullptr, global_);
  deno->context_.Reset(deno->isolate_, context_);
  deno->rust_isolate_ = create_isolate(ptr);
  return deno->rust_isolate_;
}

extern "C" const char *evalScript(void *ptr, const char *s) {
  auto deno = reinterpret_cast<Deno *>(ptr);

  Locker locker(deno->isolate_);
  Isolate::Scope isolate_scope(deno->isolate_);
  HandleScope handle_scope(deno->isolate_);

  {
    Local<Context> context_ =
        Local<Context>::New(deno->isolate_, deno->context_);

    Context::Scope scope(context_);
    // Create a string containing the JavaScript source code.
    Local<String> source =
        String::NewFromUtf8(deno->isolate_, s, NewStringType::kNormal)
            .ToLocalChecked();
    // Compile the source code.
    Local<Script> script = Script::Compile(context_, source).ToLocalChecked();
    // Run the script to get the result.
    Local<Value> result = script->Run(context_).ToLocalChecked();
    // Convert the result to an UTF8 string and print it.
    String::Utf8Value utf8(result);

    return ToCString(utf8);
  }
}

#ifdef __cplusplus
#endif

#endif
