#include <jni.h>

#ifndef NATIVE_LIB_H_
#define NATIVE_LIB_H_

extern "C" void init_module();

extern "C" void JNICALL Java_com_node_sample_MainActivity_asyncComputation
        (JNIEnv *, jobject, jobject);

#endif // NATIVE_LIB_H_
