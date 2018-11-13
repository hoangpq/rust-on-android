#include <jni.h>
#include <android/bitmap.h>

extern "C" void JNICALL Java_com_node_sample_GenerateImageActivity_generateJuliaFractal
        (JNIEnv *, jobject, jstring, jobject);
