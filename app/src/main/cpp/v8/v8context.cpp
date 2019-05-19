#include "v8context.h"
#include <unistd.h>

#define LockV8Context(env, instance)                                           \
  jclass objClazz = (env)->GetObjectClass(instance);                           \
  jfieldID field = (env)->GetFieldID(objClazz, "runtime__", "J");              \
  jlong runtimePtr = (env)->GetLongField(instance, field);

#define LockV8Result(env, instance)                                            \
  jclass objClazz = (env)->GetObjectClass(instance);                           \
  jfieldID runtimePtrField = (env)->GetFieldID(objClazz, "runtime__", "J");    \
  jfieldID resultPtrField = (env)->GetFieldID(objClazz, "result__", "J");      \
  jlong runtimePtr = (env)->GetLongField(instance, runtimePtrField);           \
  jlong resultPtr = (env)->GetLongField(instance, resultPtrField);

#define LockIsolate(ptr)                                                       \
  V8Runtime *runtime = reinterpret_cast<V8Runtime *>(ptr);                     \
  Locker locker(runtime->isolate_);                                            \
  Isolate::Scope isolate_scope(runtime->isolate_);                             \
  HandleScope handle_scope(runtime->isolate_);                                 \
  Local<Context> context =                                                     \
      Local<Context>::New(runtime->isolate_, runtime->context_);               \
  Context::Scope context_scope(context);

namespace node {

using namespace std;
using namespace v8;
using namespace util;

namespace av8 {

using jvm::JSObject;

const char *ToCString(Local<String> str) {
  String::Utf8Value value(str);
  return *value ? *value : "<string conversion failed>";
}

void ForName(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate_ = args.GetIsolate();
  JNIEnv *env_ = static_cast<JNIEnv *>(args.Data().As<External>()->Value());

  jclass utilClass = env_->FindClass("com/node/util/JNIUtils");

  jmethodID getClassMethodList =
      env_->GetStaticMethodID(utilClass, "getClassMethodList",
                              "(Ljava/lang/String;)[Ljava/lang/String;");

  jmethodID getClass = env_->GetStaticMethodID(
      utilClass, "getClass", "(Ljava/lang/String;)Ljava/lang/Class;");

  Local<String> className = args[0]->ToString();
  String::Utf8Value s(className);

  auto classStr = env_->NewStringUTF(*s);

  auto arr = (jobjectArray)env_->CallStaticObjectMethod(
      utilClass, getClassMethodList, classStr);

  jsize arrLength = env_->GetArrayLength(arr);
  int len = int(arrLength);

  Local<Array> array = Array::New(isolate_, len);
  for (int i = 0; i < len; i++) {
    auto methodName =
        (jstring)env_->GetObjectArrayElement(arr, static_cast<jsize>(i));

    array->Set(static_cast<uint32_t>(i),
               Util::ConvertToV8String(Util::JavaToString(env_, methodName)));
  }

  auto class_ =
      (jclass)env_->CallStaticObjectMethod(utilClass, getClass, classStr);

  args.GetReturnValue().Set(JSObject::NewInstance(isolate_, class_));
}

void Log(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate = args.GetIsolate();
  Local<Context> context = isolate->GetCurrentContext();

  EscapableHandleScope handle_scope(isolate);
  Local<String> result = handle_scope.Escape(
      JSON::Stringify(context, args[0]->ToObject()).ToLocalChecked());
  const char *jsonString = ToCString(result);
  LOGD("%s", jsonString);
}

void Toast(const FunctionCallbackInfo<Value> &args) {
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

void Send(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate_ = args.GetIsolate();

  assert(args[0]->IsArrayBuffer());
  assert(args[1]->IsFunction());

  auto ab = Local<ArrayBuffer>::Cast(args[0]);
  auto contents = ab->GetContents();

  char *str = workerSendBytes(contents.Data(), ab->ByteLength(), args[1]);
  args.GetReturnValue().Set(String::NewFromUtf8(isolate_, str));
}

void CreateTimer(const FunctionCallbackInfo<Value> &args, int type) {
  Isolate *isolate_ = args.GetIsolate();
  Local<Context> context = isolate_->GetCurrentContext();

  JNIEnv *env = static_cast<JNIEnv *>(args.Data().As<External>()->Value());
  Local<Function> func = Local<Function>::Cast(args[0]);
  auto *fn = new Persistent<Function>(isolate_, func);

  auto handler_ = createTimeoutHandler(&env);
  double t = args[1]->NumberValue(context).FromMaybe(0);
  postDelayed(&env, handler_, reinterpret_cast<jlong>(fn), (jlong)t, type);
}

void SetTimeOut(const FunctionCallbackInfo<Value> &args) {
  CreateTimer(args, 1);
  args.GetReturnValue().Set(Util::ConvertToV8String("Not implemented yet"));
}

void SetInterval(const FunctionCallbackInfo<Value> &args) {
  // CreateTimer(args, 2);
  Isolate *isolate = args.GetIsolate();
  assert(args[0]->IsFunction());
  assert(args[1]->IsNumber());
  Local<Function> func = Local<Function>::Cast(args[0]);
  // auto *fn = new Persistent<Function>(isolate, func);
  setInterval(ctx_.rt);
  args.GetReturnValue().Set(Util::ConvertToV8String("Interval"));
}

void New(const FunctionCallbackInfo<Value> &args) {
  args.GetReturnValue().Set(args.Holder());
}

void InvokeRef(const FunctionCallbackInfo<Value> &args) {
  Isolate *isolate = args.GetIsolate();

  V8Runtime *runtime =
      static_cast<V8Runtime *>(args.Data().As<External>()->Value());
  if (JNI_OK != runtime->env_) {
    Util::InitEnvironment(runtime->isolate_, &runtime->env_);
  }
  jclass clz = runtime->env_->FindClass("com/node/v8/V8Context");
  jmethodID method = runtime->env_->GetMethodID(clz, "updateUI", "(I)V");

  runtime->env_->CallVoidMethod(runtime->holder_, method,
                                (jint)args[0]->Int32Value());
  return args.GetReturnValue().Set(Number::New(isolate, 1.0));
}

V8Runtime *createRuntime(JNIEnv **env_) {
  if (ctx_.isolate_ == nullptr) {

    // Create a new Isolate and make it the current one.
    Isolate::CreateParams create_params;
    create_params.array_buffer_allocator =
        ArrayBuffer::Allocator::NewDefaultAllocator();

    ctx_.isolate_ = Isolate::New(create_params);

    Locker locker(ctx_.isolate_);
    Isolate::Scope isolate_scope(ctx_.isolate_);
    HandleScope handle_scope(ctx_.isolate_);

    JSObject::Init(ctx_.isolate_);
  }

  auto *runtime = new V8Runtime();
  runtime->isolate_ = ctx_.isolate_;
  runtime->env_ = *env_;

  Locker locker(runtime->isolate_);
  Isolate::Scope isolate_scope(runtime->isolate_);
  HandleScope handle_scope(runtime->isolate_);

  // Init helpers function
  Local<External> envRef_ = External::New(runtime->isolate_, runtime->env_);
  Local<ObjectTemplate> globalObject = ObjectTemplate::New(runtime->isolate_);
  Local<ObjectTemplate> class_ = ObjectTemplate::New(runtime->isolate_);

  class_->Set(Util::ConvertToV8String("forName"),
              FunctionTemplate::New(runtime->isolate_, ForName, envRef_));

  globalObject->Set(Util::ConvertToV8String("Class"), class_);

  globalObject->Set(
      Util::ConvertToV8String("$timeout"),
      FunctionTemplate::New(runtime->isolate_, SetTimeOut, envRef_));

  globalObject->Set(
      Util::ConvertToV8String("$interval"),
      FunctionTemplate::New(runtime->isolate_, SetInterval, envRef_));

  globalObject->Set(Util::ConvertToV8String("$log"),
                    FunctionTemplate::New(runtime->isolate_, Log));

  globalObject->Set(Util::ConvertToV8String("log"),
                    FunctionTemplate::New(runtime->isolate_, Log));

  globalObject->Set(Util::ConvertToV8String("toast"),
                    FunctionTemplate::New(runtime->isolate_, Toast, envRef_));

  globalObject->Set(Util::ConvertToV8String("$send"),
                    FunctionTemplate::New(runtime->isolate_, Send));

  globalObject->Set(Util::ConvertToV8String("$perform"),
                    FunctionTemplate::New(runtime->isolate_, Perform));

  Local<External> ref = External::New(runtime->isolate_, runtime);
  globalObject->Set(Util::ConvertToV8String("$invokeRef"),
                    FunctionTemplate::New(runtime->isolate_, InvokeRef, ref));

  Local<Context> globalContext =
      Context::New(ctx_.isolate_, nullptr, globalObject);

  ctx_.globalContext_.Reset(ctx_.isolate_, globalContext);
  ctx_.globalObject_.Reset(ctx_.isolate_, globalObject);

  return runtime;
}

Handle<Object> RunScript(Isolate *isolate, Local<Context> context,
                         const string &_script) {
  Local<String> source =
      String::NewFromUtf8(isolate, _script.c_str(), NewStringType::kNormal)
          .ToLocalChecked();
  Local<Script> script = Script::Compile(context, source).ToLocalChecked();
  Local<Value> value = script->Run(context).ToLocalChecked();
  return value->ToObject();
}

extern "C" void JNICALL Java_com_node_v8_V8Context_initRuntime(JNIEnv *env,
                                                               jclass klass) {
  ctx_.rt = createRuntime();
  initRuntime(&env, ctx_.rt);
}

extern "C" jobject JNICALL Java_com_node_v8_V8Context_create(JNIEnv *env,
                                                             jclass klass) {

  V8Runtime *runtime = createRuntime(&env);
  Locker locker(runtime->isolate_);
  Isolate::Scope isolate_scope(runtime->isolate_);
  HandleScope handle_scope(runtime->isolate_);

  jmethodID constructor = env->GetMethodID(klass, "<init>", "(J)V");

  auto instance = env->NewGlobalRef(
      env->NewObject(klass, constructor, reinterpret_cast<jlong>(runtime)));

  runtime->holder_ = instance;

  Local<ObjectTemplate> globalObject =
      ctx_.globalObject_.Get(runtime->isolate_);

  Local<Context> context =
      Context::New(runtime->isolate_, nullptr, globalObject);

  runtime->context_.Reset(runtime->isolate_, context);
  Context::Scope contextScope(context);

  return instance;
}

extern "C" JNIEXPORT void JNICALL Java_com_node_v8_V8Context_setKey(
    JNIEnv *env, jobject instance, jstring key, jintArray data) {

  LockV8Context(env, instance);
  LockIsolate(runtimePtr);
  jsize len = env->GetArrayLength(data);
  jint *body = env->GetIntArrayElements(data, nullptr);

  Local<Array> array = Array::New(runtime->isolate_, len);
  for (int i = 0; i < len; i++) {
    array->Set(static_cast<uint32_t>(i),
               Integer::New(runtime->isolate_, (int)body[i]));
  }

  string _key = Util::JavaToString(env, key);
  context->Global()->Set(Util::ConvertToV8String(_key), array);
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_node_v8_V8Context_eval(JNIEnv *env, jobject instance, jstring script) {

  LockV8Context(env, instance);
  LockIsolate(runtimePtr);
  std::string _script = Util::JavaToString(env, script);

  Context::Scope scope_context(context);
  Local<Object> result = RunScript(runtime->isolate_, context, _script);

  jclass resultClass = env->FindClass("com/node/v8/V8Context$V8Result");
  jmethodID constructor = env->GetMethodID(resultClass, "<init>", "(JJ)V");

  auto *container = new Persistent<Object>;
  container->Reset(runtime->isolate_, result);

  auto resultPtr = reinterpret_cast<jlong>(container);
  return env->NewObject(resultClass, constructor, resultPtr, runtimePtr);
}

extern "C" JNIEXPORT void JNICALL Java_com_node_v8_V8Context_callFn(
    JNIEnv *env, jobject instance, jlong fn, jboolean interval, jlong time) {

  LockV8Context(env, instance);
  LockIsolate(runtimePtr);

  Local<Function> func = Local<Function>::New(
      runtime->isolate_, *reinterpret_cast<Persistent<Function> *>(fn));

  // Blocking call
  func->Call(context, Null(runtime->isolate_), 0, nullptr);

  // Interval call
  if (JNI_TRUE == interval) {
    auto handler_ = createTimeoutHandler(&env);
    postDelayed(&env, handler_, fn, time, 2);
  }
}

extern "C" JNIEXPORT jobjectArray JNICALL
Java_com_node_v8_V8Context_00024V8Result_toIntegerArray(JNIEnv *env,
                                                        jobject instance) {

  LockV8Result(env, instance);
  LockIsolate(runtimePtr);

  Handle<Object> result = Local<Object>::New(
      runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

  jclass integerClass = env->FindClass("java/lang/Integer");
  jmethodID constructor = env->GetMethodID(integerClass, "<init>", "(I)V");

  if (!result->IsArray()) {
    jclass Exception = env->FindClass("java/lang/Exception");
    env->ThrowNew(Exception, "Result is not an array!");
    return nullptr;
  }

  Local<Array> jsArray(Handle<Array>::Cast(result));
  jobjectArray array =
      env->NewObjectArray(jsArray->Length(), integerClass, nullptr);
  for (uint32_t i = 0; i < jsArray->Length(); i++) {
    env->SetObjectArrayElement(array, i,
                               env->NewObject(integerClass, constructor,
                                              jsArray->Get(i)->Int32Value()));
  }
  return array;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_node_v8_V8Context_00024V8Result_toInteger(JNIEnv *env,
                                                   jobject instance) {

  LockV8Result(env, instance);
  LockIsolate(runtimePtr);

  Handle<Object> result = Local<Object>::New(
      runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

  jclass integerClass = env->FindClass("java/lang/Integer");
  jmethodID constructor = env->GetMethodID(integerClass, "<init>", "(I)V");
  return env->NewObject(integerClass, constructor, result->Int32Value());
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_node_v8_V8Context_00024V8Result_toNativeString(JNIEnv *env,
                                                        jobject instance) {
  LockV8Result(env, instance);
  LockIsolate(runtimePtr);

  Handle<Object> result = Local<Object>::New(
      runtime->isolate_, *reinterpret_cast<Persistent<Object> *>(resultPtr));

  String::Utf8Value str(result->ToString());
  return env->NewStringUTF(*str);
}

} // namespace av8
} // namespace node
