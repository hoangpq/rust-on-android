#include <stdint.h>

extern "C" const char* as_char_ptr(const uint8_t* data) {
    return (const char*)data;
}