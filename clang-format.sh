#!/usr/bin/env bash
find $PWD -name "*." | xargs clang-format -i -style=file
