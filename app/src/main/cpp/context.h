#ifndef _context_h
#define _context_h

#include <jni.h>
#include <v8.h>
#include <iostream>
#include <string>

using namespace std;

using v8::Isolate;

typedef struct NodeContext {
    JavaVM *javaVM;
    JNIEnv *env;
    jclass mainActivityClz;
    jobject mainActivityObj;
    Isolate *isolate;
} NodeContext;

typedef struct JFunc {
    std::string methodName;
    std::string sig;
    int argumentCount;
} JFunc;

class Util {
public:
    static std::string JavaToString(JNIEnv *env, jstring str);
};

extern NodeContext g_ctx;

static const char *kTAG = "NodeJS Runtime";

#define LOGI(...) \
  ((void)__android_log_print(ANDROID_LOG_INFO, kTAG, __VA_ARGS__))

#define LOGE(...) \
  ((void)__android_log_print(ANDROID_LOG_ERROR, kTAG, __VA_ARGS__))

#endif // _context_h
