#!/usr/bin/env bash
find `pwd` -name "*.rs" | xargs rustfmt --edition 2018 --color always --verbose
