#!/usr/bin/env bash
find `pwd` -name "src/**/*.h" | xargs clang-format -i -style=file
find `pwd` -name "src/**/*.cpp" | xargs clang-format -i -style=file
