#include "utils.h"
#include <string>
#include <jni.h>

namespace util {

    string Util::JavaToString(JNIEnv *env, jstring str) {
        jclass objClazz = env->GetObjectClass(str);
        jmethodID methodId = env->GetMethodID(objClazz, "getBytes", "(Ljava/lang/String;)[B");

        jstring charsetName = env->NewStringUTF("UTF-8");
        auto byteArray = (jbyteArray) env->CallObjectMethod(str, methodId,
                                                            charsetName);
        env->DeleteLocalRef(charsetName);

        jbyte *pBytes = env->GetByteArrayElements(byteArray, nullptr);

        const jsize length = env->GetArrayLength(byteArray);
        std::string results((const char *) pBytes, (unsigned long) length);

        env->ReleaseByteArrayElements(byteArray, pBytes, JNI_ABORT);
        env->DeleteLocalRef(byteArray);

        return results;
    }

    Local<String> Util::ConvertToV8String(const string &s) {
        auto isolate = Isolate::GetCurrent();
        return String::NewFromUtf8(isolate, s.c_str());
    }

    string Util::GetPackageName(JNIEnv *env, jclass class_) {
        jclass utilClass = env->FindClass("com/node/util/JNIUtils");
        jmethodID getPackageName = env->GetStaticMethodID(
                utilClass, "getPackageName", "(Ljava/lang/Class;)Ljava/lang/String;");

        auto packageName = (jstring) env->CallStaticObjectMethod(
                utilClass, getPackageName, class_);

        return JavaToString(env, packageName);
    }

}
