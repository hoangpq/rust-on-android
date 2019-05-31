#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <cassert>
#include <cstdio>
#include <jni.h>
#include <map>
#include <string>
#include <v8.h>

#define lock_isolate(isolate_)                                                 \
  Locker locker(isolate_);                                                     \
  Isolate::Scope isolate_scope(isolate_);                                      \
  HandleScope handle_scope(isolate_);

using namespace v8;

class V8Runtime {
public:
  JNIEnv *env_;
  jobject holder_;
  Isolate *isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
};

struct rust_isolate;
typedef int32_t (*deno_recv_cb)(void *isolate_, void *d, void *cb, int duration,
                                bool interval);

using ResolverPersistent = Persistent<Promise::Resolver>;

int promise_uuid = 0;
std::map<int, ResolverPersistent *> promise_map;

// Rust bridge
extern "C" {
void adb_debug(const char *);
void remove_timer(rust_isolate *isolate_, int32_t timer_id);
void fetch(rust_isolate *isolate_, const char *, int);
}

class Deno {
public:
  Isolate *isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
  deno_recv_cb recv_cb_;
  rust_isolate *rust_isolate_;

  explicit Deno(Isolate *isolate) : isolate_(isolate) {}

  Deno(Isolate *isolate, Local<Context> context, Local<ObjectTemplate> global)
      : isolate_(isolate) {
    this->context_.Reset(this->isolate_, context);
    this->global_.Reset(this->isolate_, global);
  }

  void ResetContext(Local<Context> c) {
    this->context_.Reset(this->isolate_, c);
  }

  void ResetTemplate(Local<ObjectTemplate> t) {
    this->global_.Reset(this->isolate_, t);
  }

  void SetDenoCallback(deno_recv_cb recv_cb) { this->recv_cb_ = recv_cb; }

  void *Into() { return reinterpret_cast<void *>(this); }
};

extern "C" rust_isolate *set_deno_data(void *raw, rust_isolate *isolate) {
  auto d = reinterpret_cast<Deno *>(raw);
  d->rust_isolate_ = isolate;
  return d->rust_isolate_;
}

extern "C" void set_deno_callback(void *raw, deno_recv_cb recv_cb) {
  auto d = reinterpret_cast<Deno *>(raw);
  d->SetDenoCallback(recv_cb);
}

extern "C" void lock_deno_isolate(void *raw) {
  auto d = reinterpret_cast<Deno *>(raw);
  HandleScope handle_scope(d->isolate_);
}

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

extern "C" const char *fetch_event(JNIEnv **env, jclass c, jmethodID m) {
  auto s = (jstring)(*env)->CallStaticObjectMethod(c, m);
  const char *result = jStringToChar(*env, s);
  (*env)->DeleteLocalRef(s);
  return result;
}

const char *ToCString(const String::Utf8Value &value) {
  return *value ? *value : "<string conversion failed>";
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
  assert(args[1]->IsNumber());

  auto ptr = args.Data().As<External>()->Value();
  auto d = reinterpret_cast<Deno *>(ptr);
  auto cb =
      new Persistent<Function>(d->isolate_, Local<Function>::Cast(args[0]));

  int32_t duration = args[1]->Int32Value();
  int32_t uid = d->recv_cb_(d->rust_isolate_, ptr, reinterpret_cast<void *>(cb),
                            duration, false);

  args.GetReturnValue().Set(Number::New(d->isolate_, uid));
}

void Interval(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsFunction());
  assert(args[1]->IsNumber());

  auto ptr = args.Data().As<External>()->Value();
  auto d = reinterpret_cast<Deno *>(ptr);
  auto cb =
      new Persistent<Function>(d->isolate_, Local<Function>::Cast(args[0]));

  int32_t duration = args[1]->Int32Value();
  int32_t uid = d->recv_cb_(d->rust_isolate_, ptr, reinterpret_cast<void *>(cb),
                            duration, true);

  args.GetReturnValue().Set(Number::New(d->isolate_, uid));
}

void ClearTimer(const FunctionCallbackInfo<Value> &args) {
  assert(args.Length());
  assert(args[0]->IsNumber());

  auto d = reinterpret_cast<Deno *>(args.Data().As<External>()->Value());
  remove_timer(d->rust_isolate_, args[0]->Int32Value());

  args.GetReturnValue().Set(args[0]);
}

void Destroyed(const WeakCallbackInfo<int> &info) {
  char s[20];
  sprintf(s, "Request %d", *info.GetParameter());
  adb_debug(s);
  promise_map[*info.GetParameter()]->Reset();
}

void Fetch(const FunctionCallbackInfo<Value> &args) {
  assert(args.Length());
  assert(args[0]->IsString());

  auto d = reinterpret_cast<Deno *>(args.Data().As<External>()->Value());
  lock_isolate(d->isolate_);
  String::Utf8Value url(args[0]->ToString());

  Local<Context> context_ = d->context_.Get(d->isolate_);
  auto resolver = Promise::Resolver::New(context_).ToLocalChecked();
  args.GetReturnValue().Set(resolver->GetPromise());

  int promise_id = ++promise_uuid;
  auto persistent = new ResolverPersistent(d->isolate_, resolver);
  persistent->SetWeak<int>(new int(promise_id), Destroyed,
                           WeakCallbackType::kParameter);

  promise_map[promise_id] = new ResolverPersistent(d->isolate_, resolver);
  fetch(d->rust_isolate_, *url, promise_id);
}

extern "C" void resolve_promise(void *raw, int32_t promise_id,
                                const char *value) {
  auto d = reinterpret_cast<Deno *>(raw);
  lock_isolate(d->isolate_);

  if (promise_map.find(promise_id) != promise_map.end()) {
    ResolverPersistent *persistent = promise_map.at(promise_id);
    Local<Promise::Resolver> resolver = persistent->Get(d->isolate_);
    resolver->Resolve(d->context_.Get(d->isolate_),
                      String::NewFromUtf8(d->isolate_, value));
  }
}

extern "C" void invoke_function(void *raw, void *f) {
  auto deno = reinterpret_cast<Deno *>(raw);
  lock_isolate(deno->isolate_);

  auto fn = reinterpret_cast<Persistent<Function> *>(f);
  Local<Function> cb = fn->Get(deno->isolate_);
  Local<Context> context_ = deno->context_.Get(deno->isolate_);

  const unsigned argc = 1;
  Local<Value> argv[argc] = {
      String::NewFromUtf8(deno->isolate_, "Invoked", NewStringType::kNormal)
          .ToLocalChecked()};
  cb->Call(context_, Null(deno->isolate_), argc, argv).ToLocalChecked();
}

extern "C" void *deno_init(deno_recv_cb recv_cb) {
  // Create a new Isolate and make it the current one.
  Isolate::CreateParams create_params;
  create_params.array_buffer_allocator =
      ArrayBuffer::Allocator::NewDefaultAllocator();

  Isolate *isolate_ = Isolate::New(create_params);
  lock_isolate(isolate_);

  auto deno = new Deno(isolate_);

  Local<External> env_ = External::New(deno->isolate_, deno);
  Local<ObjectTemplate> global_ = ObjectTemplate::New(deno->isolate_);

  global_->Set(String::NewFromUtf8(isolate_, "$timeout"),
               FunctionTemplate::New(isolate_, Timeout, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$interval"),
               FunctionTemplate::New(isolate_, Interval, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$clear"),
               FunctionTemplate::New(isolate_, ClearTimer, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$log"),
               FunctionTemplate::New(isolate_, Log, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$fetch"),
               FunctionTemplate::New(isolate_, Fetch, env_));

  Local<Context> context_ = Context::New(isolate_, nullptr, global_);
  deno->ResetContext(context_);
  deno->ResetTemplate(global_);
  deno->recv_cb_ = recv_cb;

  return deno->Into();
}

extern "C" void eval_script_void(void *raw, const char *s) {
  auto deno = reinterpret_cast<Deno *>(raw);
  lock_isolate(deno->isolate_);

  {
    Local<Context> context_ =
        Local<Context>::New(deno->isolate_, deno->context_);

    Context::Scope scope(context_);
    Local<String> source =
        String::NewFromUtf8(deno->isolate_, s, NewStringType::kNormal)
            .ToLocalChecked();

    Local<Script> script = Script::Compile(context_, source).ToLocalChecked();
    script->Run(context_);
  }
}

extern "C" const char *eval_script(void *raw, const char *s) {
  auto deno = reinterpret_cast<Deno *>(raw);
  lock_isolate(deno->isolate_);

  {
    TryCatch try_catch(deno->isolate_);

    Local<Context> context_ =
        Local<Context>::New(deno->isolate_, deno->context_);

    Context::Scope scope(context_);
    Local<String> source =
        String::NewFromUtf8(deno->isolate_, s, NewStringType::kNormal)
            .ToLocalChecked();

    Local<Script> script = Script::Compile(context_, source).ToLocalChecked();
    Local<Value> result = script->Run(context_).ToLocalChecked();
    String::Utf8Value utf8(result);

    return ToCString(utf8);
  }
}

#ifdef __cplusplus
#endif

#endif
