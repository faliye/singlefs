#!/bin/bash
# 用法: cite.sh <crates 相对路径> <行号> [最多字符]  -> 打出 「原文」（`路径:行号`）
T=$HOME/.local/share/singlefs/work/m3-batch50-code-r1/tree
f="$1"; n="$2"; m="${3:-110}"
line=$(awk -v n="$n" 'NR==n' "$T/$f" | sed 's/^[ \t]*//; s/[ \t]*$//')
q=$(printf '%s' "$line" | python3 -c 'import sys;s=sys.stdin.read();print(s[:int(sys.argv[1])])' "$m")
if ! grep -nF -- "$q" "$T/$f" | grep -q "^$n:"; then echo "MISS $f:$n" >&2; fi
printf '「%s」（`%s:%s`）' "$q" "$f" "$n"
