#include "node-ext.h"

namespace node {

using v8::Context;
using v8::EscapableHandleScope;
using v8::Exception;
using v8::Function;
using v8::FunctionCallbackInfo;
using v8::FunctionTemplate;
using v8::HandleScope;
using v8::Isolate;
using v8::Local;
using v8::Number;
using v8::Object;
using v8::ObjectTemplate;
using v8::String;
using v8::Value;

using v8::JSON;
using v8::MaybeLocal;

namespace loader {

const char *ToCString(Local<String> str) {
  String::Utf8Value value(str);
  return *value ? *value : "<string conversion failed>";
}

void AndroidToast(const FunctionCallbackInfo<Value> &args) {
  Local<String> str = args[0]->ToString();
  const char *msg = ToCString(str);

  JNIEnv *env_ = static_cast<JNIEnv *>(args.Data().As<External>()->Value());
  jmethodID methodId = env_->GetMethodID(g_ctx.mainActivityClz, "subscribe",
                                         "(Ljava/lang/String;)V");

  jstring javaMsg = env_->NewStringUTF(msg);
  env_->CallVoidMethod(g_ctx.mainActivityObj, methodId, javaMsg);
  env_->DeleteLocalRef(javaMsg);
  args.GetReturnValue().Set(str);
}

void AndroidLog(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate = args.GetIsolate();
  Local<Context> context = isolate->GetCurrentContext();

  EscapableHandleScope handle_scope(isolate);
  Local<String> result = handle_scope.Escape(
      JSON::Stringify(context, args[0]->ToObject()).ToLocalChecked());
  const char *jsonString = ToCString(result);
  LOGD("%s", jsonString);
}

void AndroidError(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate = args.GetIsolate();
  Local<Context> context = isolate->GetCurrentContext();

  EscapableHandleScope handle_scope(isolate);
  Local<String> result = handle_scope.Escape(
      JSON::Stringify(context, args[0]->ToObject()).ToLocalChecked());
  const char *jsonString = ToCString(result);
  LOGE("%s", jsonString);
}

    void OnLoad(const FunctionCallbackInfo<Value> &args) { init_event_loop(); }

// Override header
class ModuleWrap {
public:
  static void Initialize(v8::Local<v8::Object> target,
                         v8::Local<v8::Value> unused,
                         v8::Local<v8::Context> context){};
};

class AndroidModuleWrap : public ModuleWrap {
public:
  static void Initialize(Local<Object> target, Local<Value> unused,
                         Local<Context> context, void *priv) {

    Isolate *isolate_ = target->GetIsolate();

    ModuleWrap::Initialize(target, unused, context);
    // define function in global context
    Local<Object> global = context->Global();

    JNIEnv *env_;
    Util::InitEnvironment(isolate_, &env_);
    Local<External> jEnvRef = External::New(isolate_, env_);

    global->Set(Util::ConvertToV8String("$toast"),
                FunctionTemplate::New(isolate_, loader::AndroidToast, jEnvRef)
                    ->GetFunction());

    global->Set(
        Util::ConvertToV8String("$log"),
        FunctionTemplate::New(isolate_, loader::AndroidLog)->GetFunction());

    global->Set(
        Util::ConvertToV8String("$error"),
        FunctionTemplate::New(isolate_, loader::AndroidError)->GetFunction());

    global->Set(Util::ConvertToV8String("$load"),
                FunctionTemplate::New(isolate_, loader::OnLoad, jEnvRef)
                    ->GetFunction());
  }
};

} // namespace loader

} // namespace node

JNIEXPORT jint JNI_OnLoad(JavaVM *vm, void *) {
  memset(&g_ctx, 0, sizeof(NodeContext));
  register_vm(vm);
  g_ctx.javaVM = vm;
  g_ctx.mainActivityObj = nullptr;
  Util::AttachCurrentThread(&g_ctx.env);

  mainThreadLooper = ALooper_forThread();
  ALooper_acquire(mainThreadLooper);
  pipe(messagePipe);
  ALooper_addFd(mainThreadLooper, messagePipe[0], 0, ALOOPER_EVENT_INPUT,
                looperCallback, nullptr);

  return JNI_VERSION_1_6;
}

JNIEXPORT void JNI_OnUnload(JavaVM *vm, void *) {
  JNIEnv *env;
  if (vm->GetEnv((void **)&env, JNI_VERSION_1_6) != JNI_EDETACHED) {
    vm->DetachCurrentThread();
  }
}

extern "C" void JNICALL Java_com_node_sample_MainActivity_initVM(
    JNIEnv *env, jobject instance, jobject callback) {

  // init objects
  jclass clz = env->GetObjectClass(callback);
  g_ctx.mainActivityClz = (jclass)env->NewGlobalRef(clz);
  g_ctx.mainActivityObj = env->NewGlobalRef(callback);
  g_ctx.mainActivity = env->NewGlobalRef(instance);
}

extern "C" void JNICALL
Java_com_node_sample_MainActivity_releaseVM(JNIEnv *env, jobject instance) {

  // release allocated objects
  env->DeleteGlobalRef(g_ctx.mainActivityObj);
  env->DeleteGlobalRef(g_ctx.mainActivityClz);
  env->DeleteGlobalRef(g_ctx.mainActivity);

  g_ctx.mainActivityObj = nullptr;
  g_ctx.mainActivityClz = nullptr;
  g_ctx.mainActivity = nullptr;
}

JNIEnv *get_main_thread_env() { return g_ctx.env; }

void write_message(const void *what, size_t count) {
  write(messagePipe[1], what, count);
}

NODE_MODULE_CONTEXT_AWARE_BUILTIN(module_wrap,
                                  node::loader::AndroidModuleWrap::Initialize);
