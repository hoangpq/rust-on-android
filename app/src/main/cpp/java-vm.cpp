#include "java-vm.h"

namespace node {

    using v8::Context;
    using v8::Function;
    using v8::FunctionCallbackInfo;
    using v8::FunctionTemplate;
    using v8::Isolate;
    using v8::Local;
    using v8::Number;
    using v8::Object;
    using v8::Persistent;
    using v8::String;
    using v8::Value;

    namespace {

        Persistent<Function> JavaType::constructor;

        JavaType::JavaType(double value) : value_(value) {
        }

        JavaType::~JavaType() {}

        void JavaType::Init(Isolate *isolate) {
            // Prepare constructor template
            Local<FunctionTemplate> tpl = FunctionTemplate::New(isolate, New);
            tpl->SetClassName(String::NewFromUtf8(isolate, "MyObject"));
            tpl->InstanceTemplate()->SetInternalFieldCount(1);

            // Prototype
            NODE_SET_PROTOTYPE_METHOD(tpl, "plusOne", PlusOne);

            constructor.Reset(isolate, tpl->GetFunction());
        }

        void JavaType::New(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();

            if (args.IsConstructCall()) {
                // Invoked as constructor: `new MyObject(...)`
                double value = args[0]->IsUndefined() ? 0 : args[0]->NumberValue();
                JavaType *obj = new JavaType(value);
                obj->Wrap(args.This());
                args.GetReturnValue().Set(args.This());
            } else {
                // Invoked as plain function `MyObject(...)`, turn into construct call.
                const int argc = 1;
                Local<Value> argv[argc] = {args[0]};
                Local<Function> cons = Local<Function>::New(isolate, constructor);
                Local<Context> context = isolate->GetCurrentContext();
                Local<Object> instance =
                        cons->NewInstance(context, argc, argv).ToLocalChecked();
                args.GetReturnValue().Set(instance);
            }
        }

        void JavaType::NewInstance(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();

            const unsigned argc = 1;
            Local<Value> argv[argc] = {args[0]};
            Local<Function> cons = Local<Function>::New(isolate, constructor);
            Local<Context> context = isolate->GetCurrentContext();
            Local<Object> instance =
                    cons->NewInstance(context, argc, argv).ToLocalChecked();

            args.GetReturnValue().Set(instance);
        }

        void JavaType::PlusOne(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();

            JavaType *obj = ObjectWrap::Unwrap<JavaType>(args.Holder());
            obj->value_ += 1;

            args.GetReturnValue().Set(Number::New(isolate, obj->value_));
        }
    }  // anonymous namespace

    void CreateObject(const FunctionCallbackInfo<Value> &args) {
        JavaType::NewInstance(args);
    }

    void InitAll(Local<Object> target) {
        JavaType::Init(target->GetIsolate());
        NODE_SET_METHOD(target, "createObject", CreateObject);
    }

    void InitVM(Local<Object> target,
                Local<Value> unused,
                Local<Context> context,
                void *priv) {
        InitAll(target);
    }

} // namespace node


NODE_MODULE_CONTEXT_AWARE_BUILTIN(java, node::InitVM);
