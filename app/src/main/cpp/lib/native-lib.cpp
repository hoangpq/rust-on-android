#include <android/log.h>
#include <cstdio>
#include <cstdlib>
#include <libplatform/libplatform.h>
#include <pthread.h>
#include <stdio.h>
#include <string>
#include <unistd.h>
#include <uv.h>

#include "../utils/utils.h"
#include "native-lib.h"
#include "node.h"
#include "v8.h"

namespace node {

using namespace v8;

// Node's libUV requires all arguments being on contiguous memory.
extern "C" jint JNICALL
Java_com_node_sample_MainActivity_startNodeWithArguments(
    JNIEnv *env, jobject /* this */, jobjectArray arguments) {

  // argc
  jsize argument_count = env->GetArrayLength(arguments);
  // Compute byte size need for all arguments in contiguous memory.
  size_t c_arguments_size = 0;
  for (int i = 0; i < argument_count; i++) {
    c_arguments_size += strlen(env->GetStringUTFChars(
        (jstring)env->GetObjectArrayElement(arguments, i), 0));
    c_arguments_size++; // for '\0'
  }
  // Stores arguments in contiguous memory.
  char *args_buffer = (char *)calloc(c_arguments_size, sizeof(char));
  // argv to pass into node.
  char *argv[argument_count];
  // To iterate through the expected start position of each argument in
  // args_buffer.
  char *current_args_position = args_buffer;
  // Populate the args_buffer and argv.
  for (int i = 0; i < argument_count; i++) {
    const char *current_argument = env->GetStringUTFChars(
        (jstring)env->GetObjectArrayElement(arguments, i), 0);
    // Copy current argument to its expected position in args_buffer
    strncpy(current_args_position, current_argument, strlen(current_argument));
    // Save current argument start position in argv
    argv[i] = current_args_position;
    // Increment to the next argument's expected position.
    current_args_position += strlen(current_args_position) + 1;
  }
  // Start node, with argc and argv.
  return jint(node::Start(argument_count, argv));
}

} // namespace node
