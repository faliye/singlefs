#!/usr/bin/env bash
# 新加的未跟踪产物，假第二台主机名只在文件名里，内容干净。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf "%s\n" "/multi-host.env" > .gitignore
mkdir -p records research/results
printf "# 样本调度\n\n- 实一交回：跑了 5 轮。\n" > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf 'name=e999 rows=3\n' > research/results/e999-sample-peer-box-17.out
