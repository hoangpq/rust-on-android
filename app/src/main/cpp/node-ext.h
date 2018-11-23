#ifndef SRC_NODE_EXTENSION_H_
#define SRC_NODE_EXTENSION_H_

#include <jni.h>
#include "v8.h"
#include "node.h"
#include "env.h"
#include "env-inl.h"
#include "node_buffer.h"
#include "java-vm.h"


extern "C" jlong JNICALL Java_com_node_sample_MainActivity_createPointer(JNIEnv *, jobject);

extern "C" void JNICALL Java_com_node_sample_MainActivity_dropPointer(JNIEnv *, jobject, jlong);

#endif  // SRC_NODE_EXTENSION_H_
