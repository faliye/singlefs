#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮新放一份未跟踪的产物，内容干净，文件名里带假第二台主机名：只比行文看不到，文件名也比才红；路径里那一段打成「〔文件名含私有值〕」。
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
printf 'name=e999 rows=3\n' > research/results/e999-sample-peer-box-17.out
