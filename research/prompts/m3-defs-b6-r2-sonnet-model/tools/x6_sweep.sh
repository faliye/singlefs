#!/usr/bin/env bash
# X6：仓里现有的、文件头带 # long-run-* 声明的脚本，逐份经 lib_long_run.py script 判（detached-run.sh start / long-run-check 用的同一判法），打退出码、文件头里有 hosts 还是 unshardable。
# 用法：x6_sweep.sh <仓根>
set -u
root=$1
cd "$root" || exit 2
for file in $(grep -rlE '^# *long-run-(hosts|unshardable|shards):' scripts .claude --include='*.sh' --include='*.py' 2>/dev/null | grep -v fixtures | grep -v '^scripts/hosts/detached-run.sh$' | sort); do
  interpreter=bash; [[ $file == *.py ]] && interpreter=python3
  python3 -B .claude/hooks/lib_long_run.py script "$root" -- $interpreter "$root/$file" >/dev/null 2>&1; code=$?
  kinds=$(grep -oE '^# *long-run-(hosts|unshardable)' "$file" | sed -E 's/^# *long-run-//' | sort -u | tr '\n' ',')
  printf '%s\trc=%s\t头里有=%s\n' "$file" "$code" "${kinds%,}"
done
