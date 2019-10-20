#include "util.h"

string_t _new_string_t(const std::string &s) {
  string_t st;
  st.ptr = reinterpret_cast<const uint8_t *>(s.c_str());
  st.len = static_cast<uint32_t>(s.length());
  return st;
}

value_t _new_int_value(uint32_t val) {
  value_t v;
  v.data.i = val;
  v.t = 0;
  return v;
}

value_t _new_string_value(char *bytes, int length) {
  value_t v;
  v.data.s = _rust_new_string(bytes);
  v.t = 3;
  return v;
}

std::string v8str(Local<String> input) {
  String::Utf8Value val(input);
  std::string s(*val);
  return s;
}

string_t v8string_t(Local<Value> input) {
  String::Utf8Value val(input);
  std::string s(*val);
  return _new_string_t(s);
}
