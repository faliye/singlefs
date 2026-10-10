#!/usr/bin/env bash
# 用法 S=<草稿> run-tests.sh <日志文件(绝对路径)> <cargo test 的参数…>；在 $S/work 里跑，日志末行 exit=<码>
set -u
. "$(dirname "$0")/env.sh"
log=$1; shift
cd "$S/work" && $RUNNER "$@" > "$log" 2>&1
echo "exit=$?" >> "$log"
