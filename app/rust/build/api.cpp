#include <features.h>

#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <cassert>
#include <cstdio>
#include <iostream>
#include <jni.h>
#include <map>
#include <string>
#include <thread>
#include <v8.h>

#define lock_isolate(isolate_)                                                 \
  Locker locker(isolate_);                                                     \
  Isolate::Scope isolate_scope(isolate_);                                      \
  HandleScope handle_scope(isolate_);

using namespace v8;

typedef void (*deno_recv_cb)(void *data, uint32_t promise_id, uint32_t delay);

using ResolverPersistent = Persistent<Promise::Resolver>;

// Rust bridge
extern "C" {
void adb_debug(const char *);
void fetch(void *data, const char *, uint32_t);
void console_time(const FunctionCallbackInfo<Value> &);
void console_time_end(const FunctionCallbackInfo<Value> &);
void dispatch_fn(void *data);
}

Local<Function> get_function(Local<Object> obj, Local<String> key) {
  Local<Value> value = obj->Get(key);
  assert(value->IsFunction());
  return Local<Function>::Cast(value);
}

class Deno {
public:
  Isolate *isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
  Persistent<Function> resolver_;
  Persistent<Function> stack_empty_check_;
  Locker *locker_;

  uint32_t uuid;
  void *user_data_;
  deno_recv_cb recv_cb_;

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

  /* do not remove */
  void __unused SetDenoCallback(deno_recv_cb recv_cb) {
    this->recv_cb_ = recv_cb;
  }

  void *Into() { return reinterpret_cast<void *>(this); }

  static Deno *unwrap(void *d_) { return reinterpret_cast<Deno *>(d_); }
};

extern "C" void __unused deno_lock(void *d_) {
  auto *d = Deno::unwrap(d_);
  d->locker_ = new Locker(d->isolate_);
}

extern "C" void __unused deno_unlock(void *d_) {
  auto *d = Deno::unwrap(d_);
  delete d->locker_;
  d->locker_ = nullptr;
}

extern "C" void __unused set_deno_data(void *d_, void *user_data_) {
  auto d = Deno::unwrap(d_);
  d->user_data_ = user_data_;
}

extern "C" void __unused set_deno_resolver(void *d_) {
  auto d = Deno::unwrap(d_);
  lock_isolate(d->isolate_);
  Local<Context> context_ = d->context_.Get(d->isolate_);
  Context::Scope scope(context_);
  // resolver
  Local<Function> resolver_ = get_function(
      context_->Global(), String::NewFromUtf8(d->isolate_, "resolve"));
  d->resolver_.Reset(d->isolate_, resolver_);
  // stack check
  Local<Function> stack_empty_check_ = get_function(
      context_->Global(), String::NewFromUtf8(d->isolate_, "isStackEmpty"));
  d->stack_empty_check_.Reset(d->isolate_, stack_empty_check_);
}

static std::map<uint32_t, Deno *> isolate_map_;

const char *__unused jStringToChar(JNIEnv *env, jstring name) {
  const char *str = env->GetStringUTFChars(name, 0);
  env->ReleaseStringUTFChars(name, str);
  return str;
}

extern "C" Local<Function> __unused v8_function_cast(Local<Value> v) {
  return Local<Function>::Cast(v);
}

extern "C" void __unused v8_function_call(Local<Function> fn, int32_t argc,
                                          Local<Value> argv[]) {
  Isolate *isolate_ = Isolate::GetCurrent();
  fn->Call(isolate_->GetCurrentContext(), Null(isolate_), argc, argv);
}

extern "C" Local<ArrayBuffer> __unused v8_buffer_new(void *data,
                                                     size_t byte_length) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return ArrayBuffer::New(isolate_, data, byte_length,
                          ArrayBufferCreationMode::kInternalized);
}

extern "C" Local<Value>
    __unused v8_function_callback_info_get(FunctionCallbackInfo<Value> *info,
                                           int32_t index) {
  return (*info)[index];
}

extern "C" int32_t __unused
v8_function_callback_length(FunctionCallbackInfo<Value> *info) {
  return info->Length();
}

extern "C" void __unused v8_utf8_string_new(Local<String> *out,
                                            const uint8_t *data, int32_t len) {
  Isolate *isolate_ = Isolate::GetCurrent();
  String::NewFromUtf8(isolate_, (const char *)data, NewStringType::kNormal, len)
      .ToLocal(out);
}

extern "C" void __unused v8_set_return_value(FunctionCallbackInfo<Value> *info,
                                             Local<Value> *value) {
  info->GetReturnValue().Set(*value);
}

extern "C" Local<String> __unused v8_string_new_from_utf8(const char *data) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return String::NewFromUtf8(isolate_, data);
}

// V8 value to char*
extern "C" const char *__unused v8_value_into_raw(Local<Value> value) {
  assert(value->IsString());
  String::Utf8Value s(value);
  return *s;
}

extern "C" Local<Number> __unused v8_number_from_raw(uint64_t number) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return Number::New(isolate_, number);
}

const char *ToCString(const String::Utf8Value &value) {
  return *value ? *value : "<string conversion failed>";
}

void Log(const FunctionCallbackInfo<Value> &args) {
  assert(args.Length() > 0);
  String::Utf8Value s(args[0]);
  adb_debug(ToCString(s));
}

// exception
void ExceptionString(TryCatch *try_catch) {
  String::Utf8Value exception(try_catch->Exception());
  const char *exception_string = ToCString(exception);
  adb_debug(exception_string);

  Handle<Message> message = try_catch->Message();
  if (!message.IsEmpty()) {
    String::Utf8Value filename(message->GetScriptOrigin().ResourceName());
    const char *filename_string = ToCString(filename);
    int line_number = message->GetLineNumber();
    char s[1024];
    // (filename):(line number)
    sprintf(s, "%s:%i", filename_string, line_number);
    adb_debug(s);
  }
}

void Fetch(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsString());
  assert(args[1]->IsNumber());

  auto d = Deno::unwrap(args.Data().As<External>()->Value());
  lock_isolate(d->isolate_);

  String::Utf8Value url(args[0]->ToString());
  uint32_t promise_id = args[1]->Uint32Value();

  fetch(d->user_data_, *url, promise_id);
}

void HeapStatic(const FunctionCallbackInfo<Value> &args) {
  auto d = reinterpret_cast<Deno *>(args.Data().As<External>()->Value());
  lock_isolate(d->isolate_);

  d->isolate_->RequestGarbageCollectionForTesting(
      Isolate::kFullGarbageCollection);

  HeapStatistics stats;
  d->isolate_->GetHeapStatistics(&stats);
  args.GetReturnValue().Set(
      Number::New(d->isolate_, (double)stats.used_heap_size()));
}

extern "C" bool __unused stack_empty_check(void *d_) {
  auto d = Deno::unwrap(d_);
  lock_isolate(d->isolate_);

  Local<Context> context_ = d->context_.Get(d->isolate_);
  Context::Scope scope(context_);
  Local<Function> stack_empty_check_ = d->stack_empty_check_.Get(d->isolate_);

  MaybeLocal<Value> result =
      stack_empty_check_->Call(context_, Null(d->isolate_), 0, nullptr);

  return !result.IsEmpty() ? result.ToLocalChecked()->BooleanValue() : false;
}

void NewTimer(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsUint32()); // promise_id
  assert(args[1]->IsUint32()); // delay

  void *d_ = args.Data().As<External>()->Value();
  auto d = Deno::unwrap(d_);
  lock_isolate(d->isolate_);

  d->recv_cb_(d->user_data_, args[0]->Uint32Value(), args[1]->Uint32Value());
}

/* do not remove */
extern "C" void __unused fire_callback(void *d_, uint32_t promise_id) {
  auto d = Deno::unwrap(d_);
  lock_isolate(d->isolate_);

  Local<Context> context_ = d->context_.Get(d->isolate_);
  Context::Scope scope(context_);
  Local<Function> resolver_ = d->resolver_.Get(d->isolate_);

  const unsigned argc = 2;
  Local<Value> argv[argc] = {
      Number::New(d->isolate_, promise_id),
      String::NewFromUtf8(d->isolate_, ""),
  };

  resolver_->Call(context_, Null(d->isolate_), argc, argv);
}

/* do not remove */
extern "C" const char *__unused resolve_promise(void *d_, uint32_t promise_id,
                                                const char *value) {
  auto d = Deno::unwrap(d_);
  lock_isolate(d->isolate_);

  Handle<Context> context_ = d->context_.Get(d->isolate_);
  Context::Scope scope(context_);
  Local<Function> resolver_ = d->resolver_.Get(d->isolate_);

  const unsigned argc = 2;
  Local<Value> argv[argc] = {
      Number::New(d->isolate_, promise_id),
      String::NewFromUtf8(d->isolate_, value,
                          String::NewStringType::kInternalizedString),
  };

  resolver_->Call(context_, Null(d->isolate_), argc, argv);
  return value;
}

extern "C" void *__unused deno_init(deno_recv_cb recv_cb) {
  // Create a new Isolate and make it the current one.
  Isolate::CreateParams create_params;
  create_params.array_buffer_allocator =
      ArrayBuffer::Allocator::NewDefaultAllocator();
  // create_params.array_buffer_allocator->Allocate(1024);

  Isolate *isolate_ = Isolate::New(create_params);
  lock_isolate(isolate_);

  auto deno = new Deno(isolate_);

  Local<External> env_ = External::New(deno->isolate_, deno);
  Local<ObjectTemplate> global_ = ObjectTemplate::New(deno->isolate_);

  global_->Set(String::NewFromUtf8(isolate_, "$fetch"),
               FunctionTemplate::New(isolate_, Fetch, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$static"),
               FunctionTemplate::New(isolate_, HeapStatic, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$newTimer"),
               FunctionTemplate::New(isolate_, NewTimer, env_));

  // console
  Local<ObjectTemplate> console_ = ObjectTemplate::New(deno->isolate_);

  console_->Set(String::NewFromUtf8(isolate_, "log"),
                FunctionTemplate::New(isolate_, Log, env_));

  console_->Set(String::NewFromUtf8(isolate_, "time"),
                FunctionTemplate::New(isolate_, console_time, env_));

  console_->Set(String::NewFromUtf8(isolate_, "timeEnd"),
                FunctionTemplate::New(isolate_, console_time_end, env_));

  global_->Set(String::NewFromUtf8(isolate_, "console"), console_);

  Local<Context> context_ = Context::New(isolate_, nullptr, global_);
  deno->ResetContext(context_);
  deno->ResetTemplate(global_);
  deno->recv_cb_ = recv_cb;

  isolate_map_[0] = deno;

  return deno->Into();
}

extern "C" void eval_script(void *deno_, const char *script_s) {
  auto deno = Deno::unwrap(deno_);
  lock_isolate(deno->isolate_);

  Local<Context> context_ = Local<Context>::New(deno->isolate_, deno->context_);
  Context::Scope scope(context_);

  TryCatch try_catch(deno->isolate_);

  Local<String> source =
      String::NewFromUtf8(deno->isolate_, script_s, NewStringType::kNormal)
          .ToLocalChecked();

  ScriptOrigin origin(String::NewFromUtf8(deno->isolate_, "script.js"));
  MaybeLocal<Script> script = Script::Compile(context_, source, &origin);

  if (script.IsEmpty()) {
    ExceptionString(&try_catch);
    return;
  }

  MaybeLocal<Value> result = script.ToLocalChecked()->Run(context_);
  if (result.IsEmpty()) {
    ExceptionString(&try_catch);
    return;
  }
}

Deno *lookup_deno_by_uuid(std::map<uint32_t, Deno *> isolate_map_,
                          uint32_t uuid) {
  auto it = isolate_map_.find(uuid);
  if (it != isolate_map_.end()) {
    return it->second;
  }
  return nullptr;
}

extern "C" void __unused lookup_deno_and_eval_script(uint32_t uuid,
                                                     const char *script) {
  Deno *deno;
  if ((deno = lookup_deno_by_uuid(isolate_map_, uuid)) != nullptr) {
    adb_debug("eval_script");
    eval_script(deno, script);
  }
}

#ifdef __cplusplus
#endif

#endif
