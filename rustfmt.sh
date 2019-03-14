#!/usr/bin/env bash
find `pwd` -name "*.rs" | xargs rustfmt --force --write-mode overwrite
