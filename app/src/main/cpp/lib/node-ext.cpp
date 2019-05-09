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

using node::jvm::JavaFunctionWrapper;
using node::jvm::JavaType;

namespace loader {

using namespace util;

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

void OnLoad(const FunctionCallbackInfo<Value> &args) {
  if (g_ctx.mainActivity) {
    JNIEnv *env_ = static_cast<JNIEnv *>(args.Data().As<External>()->Value());
    onNodeServerLoaded(&env_, g_ctx.mainActivity);
  }
}

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

    JavaType::Init(isolate_);
    JavaFunctionWrapper::Init(isolate_);

    ModuleWrap::Initialize(target, unused, context);
    // define function in global context
    Local<Object> global = context->Global();

    JNIEnv *env_;
    Util::InitEnvironment(isolate_, &env_);
    Util::InitEnvironment(isolate_, &g_ctx.env);

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

    Local<ObjectTemplate> vmTemplate = ObjectTemplate::New(isolate_);
    Local<Object> vm = vmTemplate->NewInstance();

    auto javaTypeFn =
        FunctionTemplate::New(isolate_, JavaType::NewInstance)->GetFunction();

    vm->Set(Util::ConvertToV8String("type"), javaTypeFn);
    global->Set(Util::ConvertToV8String("Java"), vm);
  }
};

} // namespace loader

} // namespace node

JNIEXPORT jint JNI_OnLoad(JavaVM *vm, void *) {
  JNIEnv *env_;
  memset(&g_ctx, 0, sizeof(NodeContext));
  if (vm->GetEnv(reinterpret_cast<void **>(&env_), JNI_VERSION_1_6) != JNI_OK) {
    return JNI_ERR; // JNI version not supported.
  }
  g_ctx.javaVM = vm;
  g_ctx.mainActivityObj = nullptr;
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

NODE_MODULE_CONTEXT_AWARE_BUILTIN(module_wrap,
                                  node::loader::AndroidModuleWrap::Initialize);
