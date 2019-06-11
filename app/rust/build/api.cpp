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
#include <v8.h>

#define lock_isolate(isolate_)                                                 \
  Locker locker(isolate_);                                                     \
  Isolate::Scope isolate_scope(isolate_);                                      \
  HandleScope handle_scope(isolate_);

using namespace v8;

typedef void (*deno_recv_cb)(void *data, void *deno, uint32_t promise_id,
                             uint32_t timer_id, uint32_t delay);

using ResolverPersistent = Persistent<Promise::Resolver>;

// Rust bridge
extern "C" {
void adb_debug(const char *);
void remove_timer(void *data, uint32_t timer_id);
void fetch(void *data, const char *, uint32_t);
void console_time(const FunctionCallbackInfo<Value> &);
void console_time_end(const FunctionCallbackInfo<Value> &);
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
  deno_recv_cb recv_cb_;

  void *user_data_;
  uint32_t uuid;
  bool hasData = false;

  explicit Deno(Isolate *isolate) : isolate_(isolate) {}

  Deno(Isolate *isolate, Local<Context> context, Local<ObjectTemplate> global)
      : isolate_(isolate) {
    this->context_.Reset(this->isolate_, context);
    this->global_.Reset(this->isolate_, global);
    Context::Scope context_scope(context);

    Local<Function> resolveFn = get_function(
        context->Global(), String::NewFromUtf8(this->isolate_, "resolve"));
  }

  void ResetContext(Local<Context> c) {
    this->context_.Reset(this->isolate_, c);
  }

  void ResetTemplate(Local<ObjectTemplate> t) {
    this->global_.Reset(this->isolate_, t);
  }

  /* do not remove */
  void SetDenoCallback(deno_recv_cb recv_cb) { this->recv_cb_ = recv_cb; }

  void *Into() { return reinterpret_cast<void *>(this); }
};

static std::map<uint32_t, Deno *> isolate_map_;

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

// V8 value to char*
extern "C" const char *v8_value_into_raw(Local<Value> value) {
  assert(value->IsString());
  String::Utf8Value s(value);
  return *s;
}

extern "C" Local<Number> v8_number_from_raw(uint64_t number) {
  Isolate *isolate_ = Isolate::GetCurrent();
  return Number::New(isolate_, number);
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

void Log(const FunctionCallbackInfo<Value> &args) {
  assert(args.Length() > 0);
  String::Utf8Value s(args[0]);
  adb_debug(ToCString(s));
}

void ClearTimer(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsNumber());
  auto d = reinterpret_cast<Deno *>(args.Data().As<External>()->Value());
  remove_timer(d->user_data_, args[0]->Uint32Value());

  args.GetReturnValue().Set(args[0]);
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

  auto d = reinterpret_cast<Deno *>(args.Data().As<External>()->Value());
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

void NewTimer(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsUint32()); // promise_id
  assert(args[1]->IsUint32()); // timer_id
  assert(args[2]->IsUint32()); // timer delay

  void *ptr = args.Data().As<External>()->Value();
  auto d = reinterpret_cast<Deno *>(ptr);
  lock_isolate(d->isolate_);

  d->recv_cb_(d->user_data_, ptr, args[0]->Uint32Value(),
              args[1]->Uint32Value(), args[2]->Uint32Value());

  adb_debug("new_timer_created");
}

/* do not remove */
extern "C" void fire_callback(void *raw, uint32_t promise_id,
                              uint32_t timer_id) {
  auto d = reinterpret_cast<Deno *>(raw);
  lock_isolate(d->isolate_);

  Handle<Context> context_ = d->context_.Get(d->isolate_);
  Context::Scope scope(context_);
  Local<Function> resolveFn = get_function(
      context_->Global(), String::NewFromUtf8(d->isolate_, "resolve"));

  const unsigned argc = 2;
  Local<Value> argv[argc] = {
      Number::New(d->isolate_, promise_id),
      String::NewFromUtf8(d->isolate_, ""),
  };

  resolveFn->Call(context_, Null(d->isolate_), argc, argv);
}

/* do not remove */
extern "C" const char *resolve_promise(void *raw, uint32_t promise_id,
                                       const char *value) {
  auto d = reinterpret_cast<Deno *>(raw);
  lock_isolate(d->isolate_);

  Handle<Context> context_ = d->context_.Get(d->isolate_);
  Context::Scope scope(context_);
  Local<Function> resolveFn = get_function(
      context_->Global(), String::NewFromUtf8(d->isolate_, "resolve"));

  const unsigned argc = 2;
  Local<Value> argv[argc] = {
      Number::New(d->isolate_, promise_id),
      String::NewFromUtf8(d->isolate_, value,
                          String::NewStringType::kInternalizedString),
  };

  resolveFn->Call(context_, Null(d->isolate_), argc, argv);

  return value;
}

extern "C" void *deno_init(deno_recv_cb recv_cb) {
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

  global_->Set(String::NewFromUtf8(isolate_, "$clear"),
               FunctionTemplate::New(isolate_, ClearTimer, env_));

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

  return deno->Into();
}

extern "C" void eval_script(void *raw, void *data, const char *script_s) {
  auto deno = reinterpret_cast<Deno *>(raw);
  lock_isolate(deno->isolate_);

  // ref to rust isolate
  if (!deno->hasData) {
    deno->user_data_ = data;
    deno->hasData = true;
  }

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

extern "C" void lookup_deno_and_eval_script(uint32_t uuid, const char *script) {
  Deno *deno;
  if ((deno = lookup_deno_by_uuid(isolate_map_, uuid)) != nullptr) {
    eval_script(deno, deno->user_data_, script);
  }
}

#ifdef __cplusplus
#endif

#endif
