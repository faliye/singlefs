#!/usr/bin/env bash
# 红样本的现场：基准提交里的登记表没有复跑写 none: 的辛、癸两行（e991-h.out、e991-j.out），工作区照原样，
# 「只缩不涨」比出这两行是 none 行比基准多（壬那一行 e991-i.out 基准里就有，不报）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
units=.claude/gate.d/replay-comparison-units.tsv
current="$(mktemp)"
cp "$units" "$current"
grep -v -e $'\te991-h.out\t' -e $'\te991-j.out\t' "$current" > "$units"
git add -A
git commit -qm '样本基准：登记表少两行复跑写 none: 的行'
cp "$current" "$units"
rm -f "${current:?}"
