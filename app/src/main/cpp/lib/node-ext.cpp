#include "node-ext.h"

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

static jdouble msqrt(JNIEnv *env, jclass clazz, jdouble value) {
  return std::sqrt(value);
}

extern "C" void JNICALL Java_com_node_sample_MainActivity_initVM(
        JNIEnv *env, jobject instance, jobject callback) {

  // init objects
  jclass clz = env->GetObjectClass(callback);
  g_ctx.mainActivityClz = (jclass) env->NewGlobalRef(clz);
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

void write_message(const void *what, size_t count) {
  write(messagePipe[1], what, count);
}

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

  JNIEnv *env = g_ctx.env;
  jclass jniHelperClass = env->FindClass("com/node/util/JNIHelper");

  // Register your class' native methods.
  static const JNINativeMethod methods[] = {
          {"sqrt", "(D)D", reinterpret_cast<void *>(msqrt)},
  };

  env->RegisterNatives(jniHelperClass, methods,
                       sizeof(methods) / sizeof(JNINativeMethod));

  return JNI_VERSION_1_6;
}

JNIEXPORT void JNI_OnUnload(JavaVM *vm, void *) {
  JNIEnv *env;
  if (vm->GetEnv((void **)&env, JNI_VERSION_1_6) != JNI_EDETACHED) {
    vm->DetachCurrentThread();
  }
}
