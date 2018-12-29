#include <v8.h>
#include "java.h"
#include "jobject.h"

NodeContext g_ctx;

namespace node {

    using v8::Handle;
    using v8::Local;
    using v8::Number;
    using v8::Object;
    using v8::String;
    using v8::Value;
    using v8::Context;
    using v8::Isolate;
    using v8::Function;
    using v8::Persistent;
    using v8::HandleScope;
    using v8::Exception;
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::EscapableHandleScope;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        Persistent<FunctionTemplate> JavaType::constructor;

        JavaType::JavaType(jclass klass, JNIEnv **env) : _klass(klass), _env(env) {}

        JavaType::~JavaType() {}

        void JavaType::Init(Isolate *isolate) {
            // Prepare constructor template
            Local<FunctionTemplate> function_template = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> instance_template = function_template->InstanceTemplate();

            instance_template->SetInternalFieldCount(1);
            instance_template->SetNamedPropertyHandler(
                    NamedGetter, NamedSetter, NULL, NULL, Enumerator);

            instance_template->SetAccessor(
                    String::NewFromUtf8(Isolate::GetCurrent(), "valueOf",
                                        String::kInternalizedString), ValueOfAccessor);

            Local<ObjectTemplate> prototype_template = function_template->PrototypeTemplate();
            prototype_template->SetAccessor(
                    String::NewFromUtf8(Isolate::GetCurrent(), "toString",
                                        String::kInternalizedString), ToStringAccessor);

            constructor.Reset(isolate, function_template);
        }

        void JavaType::New(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            if (args.IsConstructCall()) {
                args.GetReturnValue().Set(args.This());
            } else {
                isolate->ThrowException(
                        String::NewFromUtf8(isolate, "Function is not constructor."));
            }
        }

        void JavaType::NewInstance(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            EscapableHandleScope scope(isolate);

            JNIEnv *env = nullptr;
            JavaType::InitEnvironment(isolate, &env);

            if (args.Length() < 1) {
                isolate->ThrowException(Exception::TypeError(
                        String::NewFromUtf8(isolate, "Wrong number of arguments")));
                return;
            }

            String::Utf8Value javaClassName(args[0]->ToString());
            jclass clazz = env->FindClass(*javaClassName);

            Handle<FunctionTemplate> _js_function_template =
                    Local<FunctionTemplate>::New(Isolate::GetCurrent(), JavaType::constructor);
            Local<Object> instance = _js_function_template->GetFunction()->NewInstance();
            JavaType *type = new JavaType(clazz, &env);

            type->InitJavaMethod(isolate, instance);
            type->Wrap(instance);

            args.GetReturnValue().Set(scope.Escape(instance));
        }

        std::string JavaType::GetMethodDescriptor(jobject method) {
            JNIEnv *env = g_ctx.env;
            // method descriptor
            jclass jniUtilClass = env->FindClass("com/node/util/JNIUtils");
            jmethodID getClassDescriptorMethodId = env->GetStaticMethodID(
                    jniUtilClass, "getJNIMethodSignature",
                    "(Ljava/lang/reflect/Method;)Ljava/lang/String;");

            jstring methodSig = (jstring) env->CallStaticObjectMethod(jniUtilClass,
                                                                      getClassDescriptorMethodId,
                                                                      method);
            return Util::JavaToString(env, methodSig);
        }

        int JavaType::GetArgumentCount(jobject method) {
            JNIEnv *env = g_ctx.env;
            // method argument count
            jclass jniUtilClass = env->FindClass("com/node/util/JNIUtils");
            jmethodID getArgumentCountMethodId = env->GetStaticMethodID(
                    jniUtilClass, "getArgumentCount", "(Ljava/lang/reflect/Method;)I");

            jint argumentCount = env->CallStaticIntMethod(
                    jniUtilClass, getArgumentCountMethodId, method);
            return argumentCount;
        }

        void JavaType::InitJavaMethod(Isolate *isolate, Local<Object> wrapper) {
            JNIEnv *env = g_ctx.env;
            jclass clazz = env->FindClass("java/lang/Class");
            jmethodID clazz_getMethods = env
                    ->GetMethodID(clazz, "getMethods", "()[Ljava/lang/reflect/Method;");

            jclass methodClazz = env->FindClass("java/lang/reflect/Method");
            jmethodID method_getName = env->GetMethodID(
                    methodClazz, "getName", "()Ljava/lang/String;");

            jobjectArray methodObjects = (jobjectArray)
                    env->CallObjectMethod(_klass, clazz_getMethods);

            jsize methodCount = env->GetArrayLength(methodObjects);
            auto callFn = FunctionTemplate::New(isolate, Call)->GetFunction();

            for (jsize i = 0; i < methodCount; i++) {

                jobject method = env->GetObjectArrayElement(methodObjects, i);
                jobject obj = env->CallObjectMethod(method, method_getName);
                jclass objClazz = env->GetObjectClass(obj);

                jmethodID toStringMethodId = env->GetMethodID(
                        objClazz, "toString", "()Ljava/lang/String;");

                jstring jmethodName = (jstring) env->CallObjectMethod(obj, toStringMethodId);
                std::string methodName = Util::JavaToString(env, jmethodName);

                if (methodName.compare("wait") == 0 ||
                    methodName.compare("equals") == 0 ||
                    methodName.compare("notify") == 0 ||
                    methodName.compare("toString") == 0 ||
                    methodName.compare("hashCode") == 0 ||
                    methodName.compare("getClass") == 0 ||
                    methodName.compare("notifyAll") == 0)
                    continue;

                JFunc jfunc;
                jfunc.methodName = methodName;
                jfunc.sig = GetMethodDescriptor(method);
                jfunc.argumentCount = GetArgumentCount(method);

                funcList.push_back(jfunc);
                // map java class method to javascript object method
                wrapper->Set(String::NewFromUtf8(isolate, methodName.c_str()), callFn);
            }

            // init by java constructor
            jmethodID constructor = env->GetMethodID(_klass, "<init>", "()V");
            this->_jinstance = env->NewObject(_klass, constructor);
        }

        void JavaType::InitEnvironment(Isolate *isolate, JNIEnv **env) {
            jint res = g_ctx.javaVM->GetEnv(reinterpret_cast<void **>(&(*env)), JNI_VERSION_1_6);
            if (res != JNI_OK) {
                res = g_ctx.javaVM->AttachCurrentThread(&(*env), NULL);
                if (JNI_OK != res) {
                    isolate->ThrowException(
                            String::NewFromUtf8(isolate, "Unable to invoke activity!"));
                }
            }
        }

        void JavaType::Call(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            HandleScope scope(isolate);
            args.GetReturnValue().Set(args.This());
        }

        Handle<Value> JavaType::JavaNameGetter(
                JNIEnv *env, const PropertyCallbackInfo<Value> &args, std::string methodName) {

            Isolate *isolate = args.GetIsolate();
            JavaType *wrapper = ObjectWrap::Unwrap<JavaType>(args.Holder());
            jobject ref = env->NewGlobalRef(wrapper->GetJavaInstance());
            return JavaFunctionWrapper::NewInstance(wrapper, isolate, ref, methodName);
        }

        void JavaType::NamedGetter(Local<String> key, const PropertyCallbackInfo<Value> &info) {
            Isolate *isolate = info.GetIsolate();
            EscapableHandleScope scope(isolate);
            String::Utf8Value m(key->ToString());
            std::string methodName(*m);

            if (methodName.compare("toString") == 0) {
                info.GetReturnValue().Set(
                        scope.Escape(FunctionTemplate::New(isolate, ToString)->GetFunction()));
            } else {
                info.GetReturnValue().Set(
                        scope.Escape(JavaNameGetter(g_ctx.env, info, methodName)));
            }
        }

        void JavaType::NamedSetter(Local<String> key, Local<Value> value,
                                   const PropertyCallbackInfo<Value> &info) {}

        void JavaType::Enumerator(const PropertyCallbackInfo<Array> &info) {
            HandleScope scope(info.GetIsolate());
        }

        void JavaType::ValueOf(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            HandleScope scope(isolate);
            args.GetReturnValue().Set(Number::New(isolate, 10.0));
        }

        void JavaType::ValueOfAccessor(Local<String> property,
                                       const PropertyCallbackInfo<Value> &info) {
            HandleScope scope(info.GetIsolate());
            Local<FunctionTemplate> js_function = FunctionTemplate::New(info.GetIsolate(), ValueOf);
            info.GetReturnValue().Set(js_function->GetFunction());
        }

        void JavaType::ToString(const FunctionCallbackInfo<Value> &args) {
            HandleScope scope(args.GetIsolate());
            JNIEnv *env = g_ctx.env;
            JavaType *wrapper = ObjectWrap::Unwrap<JavaType>(args.Holder());
            jmethodID valueOf = env->GetMethodID(
                    wrapper->_klass, "toString", "()Ljava/lang/String;");

            jstring valueOfResult = (jstring) env->CallObjectMethod(wrapper->_jinstance, valueOf);

            const char *ch = env->GetStringUTFChars(valueOfResult, 0);
            env->ReleaseStringUTFChars(valueOfResult, ch)
                    ;
            args.GetReturnValue().Set(String::NewFromUtf8(args.GetIsolate(), ch));
        }

        void JavaType::ToStringAccessor(Local<String> property,
                                        const PropertyCallbackInfo<Value> &info) {
            HandleScope scope(info.GetIsolate());
            Local<FunctionTemplate> func = FunctionTemplate::New(info.GetIsolate(), ToString);
            info.GetReturnValue().Set(func->GetFunction());
        }

    }  // anonymous namespace

} // namespace node
