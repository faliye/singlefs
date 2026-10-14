#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮新放一份未跟踪的串口日志，前 8 KiB 里有 NUL，后面一行带假本机主机名：按二进制整份跳过（在「没判的」里报一份），不报红。
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
printf 'boot\0\nlogin on sample-local-host-77\n' > research/results/e999-serial.out
printf -- '- 第二台与本机各跑了一半。\n' >> records/2026-09-24-样本调度.md
