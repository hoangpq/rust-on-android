#include <jni.h>
#include <android/bitmap.h>

#ifndef IMAGE_GEN_H_
#define IMAGE_GEN_H_

extern "C" void JNICALL Java_com_node_sample_GenerateImageActivity_blendBitmap
        (JNIEnv *, jobject, jobject bmp, jdouble pixel_size, jdouble x0, jdouble y0);

#endif // IMAGE_GEN_H_
