#include <android/bitmap.h>
#include <jni.h>

#ifndef _image_gen_h_
#define _image_gen_h_

extern "C" void JNICALL Java_com_node_sample_GenerateImageActivity_blendBitmap(
    JNIEnv *, jobject, jobject bmp, jdouble pixel_size, jdouble x0, jdouble y0);

#endif
