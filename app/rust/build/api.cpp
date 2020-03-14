#include "api.h"
#include "util/util.h"
#include "v8_jni/wrapper.h"

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

  java_register_callback(d->isolate_, context_);
  d->resolver_.Reset(d->isolate_, resolver_);
}

const char *__unused jStringToChar(JNIEnv *env, jstring name) {
  const char *str = env->GetStringUTFChars(name, 0);
  env->ReleaseStringUTFChars(name, str);
  return str;
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

extern "C" void throw_exception(const uint8_t *data, uint32_t len) {
  Isolate *isolate_ = Isolate::GetCurrent();
  Local<String> message = String::NewFromUtf8(isolate_, (const char *)data,
                                              NewStringType::kNormal, len)
                              .ToLocalChecked();
  isolate_->ThrowException(message);
}

const char *ToCString(const String::Utf8Value &value) {
  return *value ? *value : "<string conversion failed>";
}

void Log(const FunctionCallbackInfo<Value> &args) {
  auto d = Deno::unwrap(args.Data().As<External>()->Value());
  lock_isolate(d->isolate_);

  int length = args.Length();
  for (int i = 0; i < length; i++) {
      String::Utf8Value value(d->isolate_, args[i]->ToString(d->isolate_));
    adb_debug(ToCString(value));
  }
}

// exception
void ExceptionString(TryCatch *try_catch) {
    Isolate *isolate = Isolate::GetCurrent();
    Local<Context> context = isolate->GetCurrentContext();

    String::Utf8Value exception(isolate, try_catch->Exception());
  const char *exception_string = ToCString(exception);
  adb_debug(exception_string);

  Handle<Message> message = try_catch->Message();
  if (!message.IsEmpty()) {
      String::Utf8Value filename(isolate,
                                 message->GetScriptOrigin().ResourceName());
    const char *filename_string = ToCString(filename);
      int line_number = message->GetLineNumber(context).ToChecked();
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

    Local<Context> context = d->isolate_->GetCurrentContext();

    String::Utf8Value url(d->isolate_, args[0]->ToString(d->isolate_));
    uint32_t promise_id = args[1]->Uint32Value(context).ToChecked();

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

void Toast(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsString());

  auto d = reinterpret_cast<Deno *>(args.Data().As<External>()->Value());
  lock_isolate(d->isolate_);

    String::Utf8Value value(d->isolate_, args[0]->ToObject(d->isolate_));
}

void NewTimer(const FunctionCallbackInfo<Value> &args) {
  assert(args[0]->IsUint32()); // promise_id
  assert(args[1]->IsUint32()); // delay

  void *d_ = args.Data().As<External>()->Value();
  auto d = Deno::unwrap(d_);
  lock_isolate(d->isolate_);

    Local<Context> context = d->isolate_->GetCurrentContext();
    d->recv_cb_(d->user_data_, args[0]->Uint32Value(context).ToChecked(),
                args[1]->Uint32Value(context).ToChecked());
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
extern "C" __unused void resolve(void *d_, uint32_t promise_id,
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
}

extern "C" void __unused SendBuffer(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate_ = args.GetIsolate();

  assert(args[0]->IsArrayBuffer());
  assert(args[1]->IsFunction());

  auto ab = Local<ArrayBuffer>::Cast(args[0]);
  auto contents = ab->GetContents();

  char *str = worker_send_bytes(contents.Data(), ab->ByteLength(), args[1]);
  args.GetReturnValue().Set(String::NewFromUtf8(isolate_, str));
}

extern "C" void *__unused deno_init(deno_recv_cb recv_cb, uint32_t uuid) {
  V8::InitializeICU();
  Platform *platform_ = platform::CreateDefaultPlatform();
  V8::InitializePlatform(platform_);
  V8::Initialize();

  Isolate::CreateParams create_params;
  create_params.array_buffer_allocator =
      ArrayBuffer::Allocator::NewDefaultAllocator();
  Isolate *isolate_ = Isolate::New(create_params);

  Isolate::Scope isolate_scope(isolate_);
  HandleScope scope(isolate_);
  Handle<v8::Context> context = Context::New(isolate_);
  Context::Scope context_scope(context);

  auto deno = new Deno(isolate_, uuid);

  Local<External> env_ = External::New(deno->isolate_, deno);
  Local<ObjectTemplate> global_ = ObjectTemplate::New(deno->isolate_);

  global_->Set(String::NewFromUtf8(isolate_, "$sendBuffer"),
               FunctionTemplate::New(isolate_, SendBuffer, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$fetch"),
               FunctionTemplate::New(isolate_, Fetch, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$static"),
               FunctionTemplate::New(isolate_, HeapStatic, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$newTimer"),
               FunctionTemplate::New(isolate_, NewTimer, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$toast"),
               FunctionTemplate::New(isolate_, Toast, env_));

  global_->Set(String::NewFromUtf8(isolate_, "$testFn"),
               FunctionTemplate::New(isolate_, test_fn, env_));

  // console
  Local<ObjectTemplate> console_ = ObjectTemplate::New(deno->isolate_);

  console_->Set(String::NewFromUtf8(isolate_, "log"),
                FunctionTemplate::New(isolate_, Log, env_));

  global_->Set(String::NewFromUtf8(isolate_, "console"), console_);

  JavaWrapper::Init(isolate_, global_);

  Local<Context> context_ = Context::New(isolate_, nullptr, global_);
  JavaWrapper::SetContext(context_);

  deno->ResetContext(context_);
  deno->ResetTemplate(global_);
  deno->recv_cb_ = recv_cb;

  isolate_map_[deno->uuid_] = deno;

  return deno->Into();
}

extern "C" void __unused eval_script(void *deno_, const char *name_s,
                                     const char *script_s) {
  auto deno = Deno::unwrap(deno_);
  lock_isolate(deno->isolate_);

  Local<Context> context_ = Local<Context>::New(deno->isolate_, deno->context_);
  Context::Scope scope(context_);

  TryCatch try_catch(deno->isolate_);

  Local<String> source =
      String::NewFromUtf8(deno->isolate_, script_s, NewStringType::kNormal)
          .ToLocalChecked();

  ScriptOrigin origin(String::NewFromUtf8(deno->isolate_, name_s));
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

Deno *__unused lookup_deno_by_uuid(std::map<uint32_t, Deno *> isolate_map_,
                                   uint32_t uuid) {
  auto it = isolate_map_.find(uuid);
  if (it != isolate_map_.end()) {
    return it->second;
  }
  return nullptr;
}

// Utils for Rust represent
extern "C" void __unused upcast_value(Local<Value> *source,
                                      Local<Value> *dest) {
  // upcast
  *dest = Local<Value>::Cast(*source);
}

extern "C" void __unused new_number(Local<Number> *out, double value) {
  Isolate *isolate_ = Isolate::GetCurrent();
  *out = Number::New(isolate_, value);
}

extern "C" double __unused number_value(Local<Number> *number) {
  assert((*number)->IsNumber());
    Isolate *isolate = Isolate::GetCurrent();
    return (*number)->NumberValue(isolate->GetCurrentContext()).ToChecked();
}

extern "C" void __unused new_array(Local<Array> *out, uint32_t length) {
  Isolate *isolate_ = Isolate::GetCurrent();
  *out = Array::New(isolate_, length);
}

extern "C" bool __unused mem_same_handle(Local<Value> v1, Local<Value> v2) {
  return v1 == v2;
}

extern "C" void __unused
set_return_value(const FunctionCallbackInfo<Value> &args, Local<Value> value) {
  args.GetReturnValue().Set(value);
}

extern "C" void __unused new_array_buffer(Local<ArrayBuffer> *out, void *data,
                                          size_t byte_length) {
  Isolate *isolate_ = Isolate::GetCurrent();
  *out = ArrayBuffer::New(isolate_, data, byte_length,
                          ArrayBufferCreationMode::kInternalized);
}

extern "C" bool __unused function_call(Local<Value> *out, Local<Function> fun,
                                       Local<Value> self, uint32_t argc,
                                       Local<Value> argv[]) {

  Isolate *isolate_ = Isolate::GetCurrent();
  MaybeLocal<Value> maybe_result =
      fun->Call(isolate_->GetCurrentContext(), self, argc, argv);
  return maybe_result.ToLocal(out);
}

extern "C" const char *__unused raw_value(Local<Value> val) {
  Isolate *isolate_ = Isolate::GetCurrent();

  if (val->IsNullOrUndefined()) {
    isolate_->ThrowException(String::NewFromUtf8(isolate_, "<Empty>"));
  }

  if (val->IsString()) {
      String::Utf8Value utf8_val(isolate_, val);
    return *utf8_val;
  }

  Local<String> result =
          JSON::Stringify(isolate_->GetCurrentContext(), val->ToObject(isolate_),
                          String::NewFromUtf8(isolate_, "  "))
          .ToLocalChecked();

    String::Utf8Value utf8(isolate_, result);
  return *utf8;
}

extern "C" void __unused new_utf8_string(Local<String> *out,
                                         const uint8_t *data, uint32_t len) {
  MaybeLocal<String> maybe_local = String::NewFromUtf8(
      Isolate::GetCurrent(), (const char *)data, NewStringType::kNormal, len);
  maybe_local.ToLocal(out);
}

extern "C" void __unused new_object(Local<Object> *out) {
  Isolate *isolate_ = Isolate::GetCurrent();
  *out = Object::New(isolate_);
}

extern "C" bool __unused object_set(bool *out, Local<Object> obj,
                                    Local<Value> key, Local<Value> val) {
  Local<Context> context_ = Isolate::GetCurrent()->GetCurrentContext();
  Maybe<bool> maybe = obj->Set(context_, key, val);
  if (maybe.IsJust()) {
    *out = maybe.FromJust();
    return true;
  }
  return false;
}

extern "C" bool __unused object_index_set(bool *out, Local<Object> obj,
                                          uint32_t index, Local<Value> value) {
  Local<Context> context_ = Isolate::GetCurrent()->GetCurrentContext();
  Maybe<bool> maybe = obj->Set(context_, index, value);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

bool string_get(Local<String> *key, const uint8_t *data, uint32_t len) {
  Isolate *isolate_ = Isolate::GetCurrent();
  MaybeLocal<String> maybe_key = String::NewFromUtf8(
      isolate_, (const char *)data, NewStringType::kNormal, len);
  return maybe_key.ToLocal(key);
}

extern "C" bool __unused object_string_set(bool *out, Local<Object> obj,
                                           const uint8_t *data, uint32_t len,
                                           Local<Value> val) {

  Isolate *isolate_ = Isolate::GetCurrent();
  Local<String> key;
  if (!string_get(&key, data, len)) {
    return false;
  }

  Maybe<bool> maybe = obj->Set(isolate_->GetCurrentContext(), key, val);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

extern "C" bool __unused object_string_get(Local<Value> *out, Local<Object> obj,
                                           const uint8_t *data, uint32_t len) {

  Isolate *isolate_ = Isolate::GetCurrent();
  Local<String> key;
  if (!string_get(&key, data, len)) {
    return false;
  }

  MaybeLocal<Value> maybe_local = obj->Get(isolate_->GetCurrentContext(), key);
  return maybe_local.ToLocal(out);
}

extern "C" void __unused null_value(Local<Primitive> *out) {
  *out = Null(Isolate::GetCurrent());
}

extern "C" void __unused undefined_value(Local<Primitive> *out) {
  *out = Undefined(Isolate::GetCurrent());
}

extern "C" void new_function(Local<Function> *out, FunctionCallback cb) {
  Isolate *isolate_ = Isolate::GetCurrent();
  MaybeLocal<Function> maybe_local =
      Function::New(isolate_->GetCurrentContext(), cb);
  maybe_local.ToLocal(out);
}

extern "C" void __unused promise_then(Local<Promise> *promise,
                                      Local<Function> handler) {
  Isolate *isolate_ = Isolate::GetCurrent();
  Local<Context> context_ = isolate_->GetCurrentContext();
  MaybeLocal<Promise> maybe_local = (*promise)->Then(context_, handler);
  maybe_local.ToLocal(promise);
}

extern "C" void callback_info_get(const FunctionCallbackInfo<Value> &args,
                                  uint32_t index, Local<Value> *out) {
  assert(args.Length() >= index);
  *out = args[index];
}

extern "C" void attach_current_thread(JNIEnv **env) {
  int res = vm->GetEnv(reinterpret_cast<void **>(&(*env)), JNI_VERSION_1_6);
  if (res != JNI_OK) {
    res = vm->AttachCurrentThread(&(*env), nullptr);
    if (JNI_OK != res) {
      return;
    }
  }
}