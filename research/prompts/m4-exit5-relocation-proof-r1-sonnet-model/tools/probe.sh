#!/bin/bash
# 用法：probe.sh <探针目录> —— 对每个探针文件直接起 verus（crate 根单文件），打 rc、verified/errors 与第一条 error 的原文
V=${VERUS:-$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus}
cd "$1" || exit 2
for f in p*.rs; do
  "$V" --no-cheating --crate-type=lib --edition=2021 --num-threads "${THREADS:-31}" --rlimit 10 "$f" > "$f.out" 2>&1
  rc=$?
  echo "== $f rc=$rc"
  grep -m1 -E '^(error|verification results)' "$f.out" | cut -c1-200
  grep -m1 -A1 '^error' "$f.out" | tail -1 | cut -c1-200
done
