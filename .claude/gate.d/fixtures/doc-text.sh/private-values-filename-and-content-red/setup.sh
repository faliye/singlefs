#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮新放一份未跟踪的产物，文件名与第 1 行都带假第二台主机名：行那一类与文件名那一类都红，
# 红行里的路径把含值的那一段打成「〔文件名含私有值〕」，输出里不出现假主机名。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样
mkdir -p records research/results
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 实一交回：跑了 5 轮。
X
git add -A && git commit -qm base
printf 'ssh sample-peer-box-17 跑完了\n' > research/results/e999-sample-peer-box-17.out
