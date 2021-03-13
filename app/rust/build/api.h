#ifndef lib_api
#define lib_api

#ifdef __cplusplus
#endif

#include <libplatform/libplatform.h>
#include <cassert>
#include <cstdio>
#include <features.h>
#include <iostream>
#include <jni.h>
#include <map>
#include <string>
#include <thread>
#include <v8.h>

#define lock_isolate(isolate_)            \
  Locker locker(isolate_);                \
  Isolate::Scope isolate_scope(isolate_); \
  HandleScope handle_scope(isolate_);

using namespace v8;

using ResolverPersistent = Persistent<Promise::Resolver>;
typedef void (* deno_recv_cb)(void* data, uint32_t promise_id, uint32_t delay);

// NDK vm instance
static JavaVM* vm;

// Rust bridge
extern "C" {
void adb_debug(const char*);
void fetch(void* data, const char*, uint32_t);
void test_fn(const FunctionCallbackInfo<Value>&);
char* worker_send_bytes(void*, size_t, Local<Value> val);
void attach_current_thread(JNIEnv** env);
void register_vm(JavaVM* _vm) { vm = _vm; }
JavaVM* __unused get_java_vm() { return vm; }
}

class Deno {
public:
  Isolate* isolate_;
  Persistent<Context> context_;
  Persistent<ObjectTemplate> global_;
  Persistent<Function> resolver_;
  Locker* locker_;
  JNIEnv* env_;

  uint32_t uuid_;
  void* user_data_;
  deno_recv_cb recv_cb_;

  explicit Deno(Isolate* isolate, uint32_t uuid)
      : isolate_(isolate), uuid_(uuid) {
    attach_current_thread(&this->env_);
  }

  Deno(Isolate* isolate, Local<Context> context, Local<ObjectTemplate> global)
      : isolate_(isolate) {
    this->context_.Reset(this->isolate_, context);
    this->global_.Reset(this->isolate_, global);
  }

  ~Deno() { vm->DetachCurrentThread(); }

  void ResetContext(Local<Context> c) {
    this->context_.Reset(this->isolate_, c);
  }

  void ResetTemplate(Local<ObjectTemplate> t) {
    this->global_.Reset(this->isolate_, t);
  }

  void __unused SetDenoCallback(deno_recv_cb recv_cb) {
    this->recv_cb_ = recv_cb;
  }

  void* Into() { return reinterpret_cast<void*>(this); }

  static Deno* unwrap(void* d_) { return reinterpret_cast<Deno*>(d_); }
};

static std::map<uint32_t, Deno*> isolate_map_;

#ifdef __cplusplus
#endif

#endif
