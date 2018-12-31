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
    using v8::Boolean;
    using v8::Persistent;
    using v8::Undefined;
    using v8::Exception;
    using v8::HandleScope;
    using v8::ObjectTemplate;
    using v8::FunctionTemplate;
    using v8::EscapableHandleScope;
    using v8::FunctionCallbackInfo;

    namespace jvm {

        Persistent<FunctionTemplate> JavaFunctionWrapper::_func_wrapper;

        JavaFunctionWrapper::JavaFunctionWrapper(
                JavaType *type, jobject instance, std::string methodName)
                : _type(type),
                  _instance(instance),
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
        JavaFunctionWrapper::NewInstance(JavaType *type, Isolate *isolate, jobject jinst,
                                         std::string methodName) {
            Handle<FunctionTemplate> _function_template =
                    Local<FunctionTemplate>::New(isolate, _func_wrapper);

            Local<Object> jsinst = _function_template->GetFunction()->NewInstance();
            JavaFunctionWrapper *function_wrapper = new JavaFunctionWrapper(type, jinst,
                                                                            methodName);

            function_wrapper->Wrap(jsinst);
            return Local<Value>::New(isolate, jsinst);
        }

        jobject JavaFunctionWrapper::V8ToJava(Handle<Value> value) {
            JNIEnv *env = g_ctx.env;

            jobject result = NULL;
            if (value->IsNumber()) {
                if (value->IsInt32()) {
                    jclass cls = env->FindClass("java/lang/Integer");
                    jmethodID midInit = env->GetMethodID(cls, "<init>", "(I)V");
                    result = env->NewObject(cls, midInit, value->Int32Value());
                } else {
                    jclass cls = env->FindClass("java/lang/Double");
                    jmethodID midInit = env->GetMethodID(cls, "<init>", "(D)V");
                    result = env->NewObject(cls, midInit, value->NumberValue());
                }
            }
            if (value->IsString()) {
                String::Utf8Value s(value->ToString());
                result = env->NewStringUTF(*s);
            }
            return result;
        }

        void JavaFunctionWrapper::HandleException(Isolate *isolate) {
            JNIEnv *env = g_ctx.env;
            if (env->ExceptionCheck()) {
                env->ExceptionDescribe();
                env->ExceptionClear();
                isolate->ThrowException(Exception::TypeError(
                        String::NewFromUtf8(isolate, "Something went wrong!")));
            }
        }

        void JavaFunctionWrapper::Call(const FunctionCallbackInfo<Value> &args) {
            Isolate *isolate = args.GetIsolate();
            HandleScope scope(isolate);

            JavaFunctionWrapper *wrapper = ObjectWrap::Unwrap<JavaFunctionWrapper>(args.This());
            JNIEnv *env = g_ctx.env;

            int argumentCount = args.Length();
            Local<Value> jsValue = Undefined(isolate);

            for (JFunc &func : wrapper->_type->funcList) {
                if (func.argumentCount == argumentCount &&
                    func.methodName.compare(wrapper->_methodName) == 0) {

                    jmethodID methodId = env->GetMethodID(wrapper->_type->GetJavaClass(),
                                                          func.methodName.c_str(),
                                                          func.sig.c_str());

                    if (func.argumentCount == 0) {
                        jint size = env->CallIntMethod(wrapper->_type->GetJavaInstance(), methodId);
                        jsValue = Number::New(isolate, size);
                    } else {
                        jobject jValue = V8ToJava(args[0]);
                        env->CallBooleanMethod(
                                wrapper->_type->GetJavaInstance(), methodId, jValue);
                        env->DeleteLocalRef(jValue);
                    }

                    HandleException(isolate);
                }
            }
            args.GetReturnValue().Set(jsValue);
        }

    }
}
