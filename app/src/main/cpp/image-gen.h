#include <jni.h>
#include <android/bitmap.h>

extern "C" void JNICALL Java_com_node_sample_GenerateImageActivity_blendBitmap
        (JNIEnv *, jobject, jobject bmp, jdouble pixel_size, jdouble x0, jdouble y0,
         jobject callback);
