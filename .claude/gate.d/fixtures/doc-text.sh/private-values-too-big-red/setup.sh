#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮新放一份 3000 字节的未跟踪日志（里面没有私有值），样本根的 .private-values-size-limit 把上限设成 1000 字节：
# 超过上限判红、不截断，点名这份文件（路径里没有私有值，原样打）。
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
python3 -c 'import sys; sys.stdout.write("progress 0123456789 abcdefghij\n" * 100)' > research/results/e999-sample-long-run.out
