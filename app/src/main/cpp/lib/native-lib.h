#include <jni.h>

#ifndef _native_lib_h_
#define _native_lib_h_

extern "C" void JNICALL
Java_com_node_sample_MainActivity_asyncComputation(JNIEnv *, jobject, jobject);

#endif
