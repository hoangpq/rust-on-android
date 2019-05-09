#!/usr/bin/env bash
dir="`pwd`/app/src/main"
find ${dir} -name *.h | xargs clang-format -i -style=file
find ${dir} -name *.cpp | xargs clang-format -i -style=file
