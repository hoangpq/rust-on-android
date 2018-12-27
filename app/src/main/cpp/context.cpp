#include "context.h"
#include <string>
#include <jni.h>

std::string Util::JavaToString(JNIEnv *env, jstring str) {
    jclass objClazz = env->GetObjectClass(str);
    jmethodID methodId = env->GetMethodID(objClazz, "getBytes", "(Ljava/lang/String;)[B");

    jstring charsetName = env->NewStringUTF("UTF-8");
    jbyteArray stringJbytes = (jbyteArray) env->CallObjectMethod(str, methodId,
                                                                 charsetName);
    env->DeleteLocalRef(charsetName);

    jbyte *pBytes = env->GetByteArrayElements(stringJbytes, NULL);

    const jsize length = env->GetArrayLength(stringJbytes);
    std::string results((const char *) pBytes, (unsigned long) length);

    env->ReleaseByteArrayElements(stringJbytes, pBytes, JNI_ABORT);
    env->DeleteLocalRef(stringJbytes);

    return results;
}
