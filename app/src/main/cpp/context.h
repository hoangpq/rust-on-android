#ifndef _context_h
#define _context_h

#include <jni.h>

typedef struct NodeContext {
    JavaVM *javaVM;
    JNIEnv *env;
    jclass mainActivityClz;
    jobject mainActivityObj;
} NodeContext;

extern NodeContext g_ctx;

static const char *kTAG = "NodeJS Runtime";

#define LOGI(...) \
  ((void)__android_log_print(ANDROID_LOG_INFO, kTAG, __VA_ARGS__))

#define LOGE(...) \
  ((void)__android_log_print(ANDROID_LOG_ERROR, kTAG, __VA_ARGS__))

#endif // _context_h
