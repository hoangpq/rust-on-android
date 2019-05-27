#!/usr/bin/env bash
rust_lib="`pwd`/app/rust/build"
jni_lib="`pwd`/app/src/main"
find ${rust_lib} -name *.h | xargs clang-format -i -style=file
find ${rust_lib} -name *.cpp | xargs clang-format -i -style=file
find ${jni_lib} -name *.h | xargs clang-format -i -style=file
find ${jni_lib} -name *.cpp | xargs clang-format -i -style=file
