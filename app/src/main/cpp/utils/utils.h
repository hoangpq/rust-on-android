#ifndef _context_h
#define _context_h

#include <jni.h>
#include <v8.h>
#include <iostream>
#include <string>

using namespace std;
using namespace v8;

typedef struct NodeContext {
    JavaVM *javaVM;
    JNIEnv *env;
    jclass mainActivityClz;
    jobject mainActivityObj;
    Isolate *isolate_;
    Persistent<Context> globalContext_;
    Persistent<ObjectTemplate> globalObject_;
} NodeContext;

typedef struct JFunc {
    std::string methodName;
    std::string sig;
    int argumentCount;
} JFunc;

namespace util {

    class Util {
    public:
        static string JavaToString(JNIEnv *env, jstring str);
        static Local<String> ConvertToV8String(const string &s);
    };
}

extern NodeContext g_ctx;

static const char *kTAG = "NodeJS Runtime";

#define LOGI(...) \
  ((void)__android_log_print(ANDROID_LOG_INFO, kTAG, __VA_ARGS__))

#define LOGE(...) \
  ((void)__android_log_print(ANDROID_LOG_ERROR, kTAG, __VA_ARGS__))

#endif
