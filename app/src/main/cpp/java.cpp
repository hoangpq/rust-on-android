#include <v8.h>
#include <android/log.h>
#include "java.h"

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
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        Persistent<Function> JavaType::constructor;

        JavaType::JavaType(char *className, NodeContext &ctx)
                : _ctx(ctx), _className(className) {}

        JavaType::~JavaType() {}

        void JavaType::Init(Isolate *isolate) {
            // Prepare constructor template
            Local<FunctionTemplate> function_template = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> instance_template = function_template->InstanceTemplate();

            instance_template->SetInternalFieldCount(1);
            instance_template->SetNamedPropertyHandler(
                    NamedGetter, NamedSetter, NULL, NULL, Enumerator);
            instance_template->SetCallAsFunctionHandler(Call, Handle<Value>());

            instance_template->SetAccessor(String::NewFromUtf8(Isolate::GetCurrent(), "valueOf",
                                                               String::kInternalizedString),
                                           ValueOfAccessor);

            Local<ObjectTemplate> prototype_template = function_template->PrototypeTemplate();
            prototype_template->SetAccessor(String::NewFromUtf8(Isolate::GetCurrent(), "toString",
                                                                String::kInternalizedString),
                                            ToStringAccessor);

            // Prototype
            NODE_SET_PROTOTYPE_METHOD(function_template, "$toast", Toast);
            NODE_SET_PROTOTYPE_METHOD(function_template, "$version", Version);
            constructor.Reset(isolate, function_template->GetFunction());
        }

        void JavaType::Enumerator(const PropertyCallbackInfo<Array> &js_info) {
            HandleScope scope(js_info.GetIsolate());
        }

        void JavaType::ToStringAccessor(Local<String> js_property,
                                        const PropertyCallbackInfo<Value> &js_info) {
            HandleScope scope(js_info.GetIsolate());
            JavaType *t = ObjectWrap::Unwrap<JavaType>(js_info.This());
            js_info.GetReturnValue().Set(
                    String::NewFromUtf8(scope.GetIsolate(), t->getClassName()));
            js_info.GetReturnValue().Set(js_info.This());
        }

        void JavaType::ValueOfAccessor(Local<String> js_property,
                                       const PropertyCallbackInfo<Value> &js_info) {
            HandleScope scope(js_info.GetIsolate());
            JavaType *t = ObjectWrap::Unwrap<JavaType>(js_info.This());
            js_info.GetReturnValue().Set(
                    String::NewFromUtf8(scope.GetIsolate(), t->getClassName()));
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
            JavaType::InitEnvironment(args, &env);
            // From Rust static lib
            args.GetReturnValue().Set(Number::New(isolate, getAndroidVersion(&env)));
        }

        void JavaType::NewInstance(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            Local<Context> context = isolate->GetCurrentContext();

            JNIEnv *env = nullptr;
            JavaType::InitEnvironment(args, &env);

            const unsigned argc = 1;
            Local<Value> argv[argc] = {args[0]};
            Local<Function> cons = Local<Function>::New(isolate, constructor);
            Local<Object> instance =
                    cons->NewInstance(context, argc, argv).ToLocalChecked();

            jint ver = env->GetVersion();
            double jniVersion = (double) ((ver >> 16) & 0x0f) + (ver & 0x0f) * 0.1;
            instance->Set(String::NewFromUtf8(isolate, "jniVersion"),
                          Number::New(isolate, jniVersion));

            args.GetReturnValue().Set(instance);
        }

        void JavaType::InitEnvironment(const FunctionCallbackInfo<Value> &args, JNIEnv **env) {
            Isolate *isolate = args.GetIsolate();
            Local<Context> context = isolate->GetCurrentContext();

            // get $vm from global object
            Local<Object> global = context->Global();
            Local<String> $vmKey = String::NewFromUtf8(isolate, "$vm");
            Local<Object> $vm = Local<Object>::Cast(global->Get(context, $vmKey).ToLocalChecked());
            JavaType *t = ObjectWrap::Unwrap<JavaType>($vm);

            jint res = t->getJavaVM()->GetEnv(reinterpret_cast<void **>(&(*env)), JNI_VERSION_1_6);
            if (res != JNI_OK) {
                res = t->getJavaVM()->AttachCurrentThread(&(*env), NULL);
                if (JNI_OK != res) {
                    args.GetReturnValue()
                            .Set(String::NewFromUtf8(isolate, "Unable to invoke activity!"));
                }
            }
        }

        void JavaType::WrapObject(v8::Local<v8::Object> handle) {
            Wrap(handle);
        }

        void JavaType::Call(const FunctionCallbackInfo<Value> &js_args) {
            HandleScope scope(js_args.GetIsolate());
            js_args.GetReturnValue().Set(
                    String::NewFromUtf8(js_args.GetIsolate(), "Method called"));
        }

        void JavaType::NamedGetter(Local<String> js_key,
                                   const PropertyCallbackInfo<Value> &js_info) {
            HandleScope scope(js_info.GetIsolate());
            String::Utf8Value key(js_key);
        }

        void JavaType::NamedSetter(Local<String> js_key, Local<Value> js_value,
                                   const PropertyCallbackInfo<Value> &js_info) {}

        void CreateJavaType(const FunctionCallbackInfo<Value> &args) {
            jvm::JavaType::NewInstance(args);
        }

        void InitAll(Local<Object> target) {
            JavaType::Init(target->GetIsolate());
            NODE_SET_METHOD(target, "type", CreateJavaType);
        }

        void InitJavaVM(Local<Object> target) {
            InitAll(target);
        }

    }  // anonymous namespace


} // namespace node

NODE_MODULE_CONTEXT_AWARE_BUILTIN(java, node::jvm::InitJavaVM);
