#include <jni.h>

#ifndef _native_lib_h_
#define _native_lib_h_

extern "C" void init_module();

extern "C" void JNICALL Java_com_node_sample_MainActivity_asyncComputation
        (JNIEnv *, jobject, jobject);

extern "C" jstring JNICALL Java_com_node_sample_MainActivity_executeScript
        (JNIEnv *, jobject, jstring);

#endif // _native_lib_h_
