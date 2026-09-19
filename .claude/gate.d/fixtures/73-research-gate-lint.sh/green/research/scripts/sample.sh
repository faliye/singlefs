#!/usr/bin/env bash
source "$(dirname "$0")/lib.sh"
[[ -f input.txt ]] || die "找不到 input.txt" "先跑 make-input.sh 生成它"
