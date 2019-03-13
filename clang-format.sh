#!/usr/bin/env bash
find `pwd` -name "*." | xargs clang-format -i -style=file
