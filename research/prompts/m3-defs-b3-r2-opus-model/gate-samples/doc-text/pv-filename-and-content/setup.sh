#!/usr/bin/env bash
# 新加的未跟踪产物，文件名与内容里都有假第二台主机名：红行按格文件头只打文件、行号与键名，文件名本身带着值。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf "%s\n" "/multi-host.env" > .gitignore
mkdir -p records research/results
printf "# 样本调度\n\n- 实一交回：跑了 5 轮。\n" > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf 'ssh sample-peer-box-17 跑完了\n' > research/results/e999-sample-peer-box-17.out
