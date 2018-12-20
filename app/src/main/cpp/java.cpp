#include <v8.h>
#include "java.h"
#include "jobject.h"
#include <string.h>

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
            instance_template->SetCallAsFunctionHandler(Call, Handle<Value>());

            instance_template->SetNamedPropertyHandler(
                    NamedGetter, NamedSetter, NULL, NULL, Enumerator);
            instance_template->SetCallAsFunctionHandler(Call, Handle<Value>());

            instance_template->SetAccessor(
                    String::NewFromUtf8(isolate, "valueOf", String::kInternalizedString),
                    ValueOfAccessor);

            Local<ObjectTemplate> prototype_template = function_template->PrototypeTemplate();
            prototype_template->SetAccessor(
                    String::NewFromUtf8(isolate, "toString", String::kInternalizedString),
                    ToStringAccessor);

            // Prototype
            NODE_SET_PROTOTYPE_METHOD(function_template, "$toast", Toast);
            NODE_SET_PROTOTYPE_METHOD(function_template, "$version", Version);
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

        void JavaType::Toast(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            Handle<Context> context = isolate->GetCurrentContext();
            Local<String> fnName = String::NewFromUtf8(isolate, "$toast");
            Handle<Object> global = context->Global();
            // Get $toast function from global context
            Local<Function> $toast = Local<Function>::Cast(
                    global->Get(context, fnName).ToLocalChecked());
            Local<Value> funcArgs[1];
            funcArgs[0] = String::NewFromUtf8(
                    isolate, "Invoke $toast function in global context successfully!");
            $toast->Call(global, 1, funcArgs);
        }

        void JavaType::Version(const FunctionCallbackInfo<Value> &args) {
            JNIEnv *env;
            Isolate *isolate = args.GetIsolate();
            JavaType::InitEnvironment(isolate, &env);
            // From Rust static lib
            args.GetReturnValue().Set(Number::New(isolate, getAndroidVersion(&env)));
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

            jint ver = env->GetVersion();
            double jniVersion = (double) ((ver >> 16) & 0x0f) + (ver & 0x0f) * 0.1;
            instance->Set(String::NewFromUtf8(isolate, "$jni_version"),
                          Number::New(isolate, jniVersion));

            JavaType *type = new JavaType(clazz, &env);

            type->InitJavaMethod(isolate, instance);
            type->Wrap(instance);

            args.GetReturnValue().Set(scope.Escape(instance));
        }

        void JavaType::InitJavaMethod(Isolate *isolate, Local<Object> wrapper) {
            JNIEnv *env = GetCurrentJNIEnv();
            jclass clazz = env->FindClass("java/lang/Class");
            jmethodID clazz_getMethods = env
                    ->GetMethodID(clazz, "getMethods", "()[Ljava/lang/reflect/Method;");

            jclass methodClazz = env->FindClass("java/lang/reflect/Method");
            jmethodID method_getName = env->GetMethodID(methodClazz, "getName",
                                                        "()Ljava/lang/String;");

            jobjectArray methodObjects = (jobjectArray)
                    env->CallObjectMethod(_klass, clazz_getMethods);

            jsize methodCount = env->GetArrayLength(methodObjects);
            auto callFn = FunctionTemplate::New(isolate, Call)->GetFunction();

            for (jsize i = 0; i < methodCount; i++) {

                jobject method = env->GetObjectArrayElement(methodObjects, i);
                jobject obj = env->CallObjectMethod(method, method_getName);
                jclass objClazz = env->GetObjectClass(obj);
                jmethodID methodId = env->GetMethodID(objClazz,
                                                      "toString", "()Ljava/lang/String;");

                jstring result = (jstring) env->CallObjectMethod(obj, methodId);
                const char *str = env->GetStringUTFChars(result, NULL);
                env->ReleaseStringUTFChars(result, str);
                // map java class method to javascript object method
                wrapper->Set(String::NewFromUtf8(isolate, str), callFn);
            }

            // init by java constructor
            jmethodID constructor = env->GetMethodID(this->_klass, "<init>", "()V");
            this->_jinstance = env->NewObject(this->_klass, constructor);
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

        void JavaType::NamedGetter(Local<String> key, const PropertyCallbackInfo<Value> &info) {
            Isolate *isolate = info.GetIsolate();
            EscapableHandleScope scope(isolate);
            String::Utf8Value key_(key->ToString());

            if (strcmp(*key_, "add")) {
                JavaType *t = ObjectWrap::Unwrap<JavaType>(info.Holder());
                jmethodID methodId = g_ctx.env->GetMethodID(t->GetJavaClass(), *key_, "()V");
                Local<Object> jObject = JavaObject::NewInstance(t->GetJavaInstance(), methodId,
                                                                isolate);
                info.GetReturnValue().Set(scope.Escape(jObject));
            }
        }

        void JavaType::NamedSetter(Local<String> key, Local<Value> value,
                                   const PropertyCallbackInfo<Value> &info) {}

        void JavaType::Enumerator(const PropertyCallbackInfo<Array> &info) {
            HandleScope scope(info.GetIsolate());
        }

        void JavaType::ToStringAccessor(Local<String> property,
                                        const PropertyCallbackInfo<Value> &info) {
            HandleScope scope(info.GetIsolate());
        }

        void
        JavaType::ValueOfAccessor(Local<String> property, const PropertyCallbackInfo<Value> &info) {
            HandleScope scope(info.GetIsolate());
        }

    }  // anonymous namespace

} // namespace node
