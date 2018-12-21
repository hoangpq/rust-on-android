#include <v8.h>
#include "jobject.h"
#include "java.h"

namespace node {

    using v8::Local;
    using v8::Value;
    using v8::Number;
    using v8::Handle;
    using v8::String;
    using v8::Isolate;
    using v8::Persistent;
    using v8::Exception;
    using v8::HandleScope;
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::EscapableHandleScope;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        Persistent<FunctionTemplate> JavaFunctionWrapper::_func_wrapper;

        JavaFunctionWrapper::JavaFunctionWrapper(
                jobject instance, jmethodID methodId, char *methodName) : _instance(instance),
                                                                          _methodId(methodId),
                                                                          _methodName(methodName) {}

        JavaFunctionWrapper::~JavaFunctionWrapper() {}

        void JavaFunctionWrapper::Init(Isolate *isolate) {
            Local<FunctionTemplate> function_template = FunctionTemplate::New(isolate, New);
            Local<ObjectTemplate> instance_template = function_template->InstanceTemplate();
            instance_template->SetInternalFieldCount(1);
            instance_template->SetCallAsFunctionHandler(Call, Handle<Value>());
            _func_wrapper.Reset(isolate, function_template);
        }

        void JavaFunctionWrapper::New(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            if (args.IsConstructCall()) {
                args.GetReturnValue().Set(args.This());
            } else {
                isolate->ThrowException(
                        String::NewFromUtf8(isolate, "Function is not constructor."));
            }
        }

        Local<Value>
        JavaFunctionWrapper::NewInstance(
                Isolate *isolate, jobject jinst, jmethodID methodId, char *methodName) {
            Handle<FunctionTemplate> _function_template =
                    Local<FunctionTemplate>::New(isolate, _func_wrapper);

            Local<Object> jsinst = _function_template->GetFunction()->NewInstance();
            JavaFunctionWrapper *function_wrapper = new JavaFunctionWrapper(jinst, methodId,
                                                                            methodName);

            function_wrapper->Wrap(jsinst);
            return Local<Value>::New(isolate, jsinst);
        }

        void JavaFunctionWrapper::Call(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            HandleScope scope(isolate);
            JavaFunctionWrapper *wrapper = ObjectWrap::Unwrap<JavaFunctionWrapper>(args.This());

            JNIEnv *env = g_ctx.env;
            jclass cls = env->FindClass("java/lang/Double");
            jmethodID midInit = env->GetMethodID(cls, "<init>", "(D)V");
            if (NULL == midInit) return;

            double num = args[0]->NumberValue(isolate->GetCurrentContext()).FromMaybe(0);
            jobject newObj = env->NewObject(cls, midInit, num);
            env->CallBooleanMethod(wrapper->_instance, wrapper->_methodId, newObj);

            if (env->ExceptionCheck()) {
                env->ExceptionDescribe();
                env->ExceptionClear();
            }

            args.GetReturnValue().Set(Number::New(isolate, 10.0));
        }

    }
}
