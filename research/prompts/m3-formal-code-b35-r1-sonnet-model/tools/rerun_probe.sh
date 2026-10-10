#!/usr/bin/env bash
# 用法 S=<草稿> rerun_probe.sh <日志名> <测试名>   （OPT=1 的探针编译目录，日志落 $S/<日志名>）
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/env.sh"
export OPT=1
. "$HERE/env.sh"
log="$S/$1"; shift
bash "$HERE/run-tests.sh" "$log" -p singlefs-harness --test zz_probe_b35 -- --nocapture --exact "$@"
