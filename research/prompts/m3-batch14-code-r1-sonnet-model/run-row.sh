#!/usr/bin/env bash
# 一条变异套内存包装与线程上限跑（整条脚本经包装，包装里不再嵌包装）。
# 环境：MUT_W 副本根（里面有 Cargo.toml、crates/），CARGO_TARGET_DIR 编译目录。
# 用法：run-row.sh ROW | run-row.sh --adhoc 源文件 原文文件 替换文件 cargo-test参数…
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-100}"
exec nice -n 19 bash "${REPO:-/home/fy5090/code/singlefs}/research/scripts/run-with-memory-cap.sh" 6G \
  bash "${REPO:-/home/fy5090/code/singlefs}/research/scripts/capped.sh" "${THREADS:-7}" python3 "$here/mutrow.py" "$@"
