#include <jni.h>

extern "C" void init_module();

extern "C" void JNICALL Java_com_node_sample_MainActivity_asyncComputation
        (JNIEnv *, jobject, jobject);
