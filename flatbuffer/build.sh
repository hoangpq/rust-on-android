#!/usr/bin/env bash
flatc -s --gen-mutable `pwd`/user.fbs
flatc -r --gen-mutable `pwd`/user.fbs
