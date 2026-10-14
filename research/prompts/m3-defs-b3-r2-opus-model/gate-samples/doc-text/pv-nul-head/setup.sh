#!/usr/bin/env bash
# 新加的未跟踪日志，前 8 KiB 里有一个 NUL 字节（串口、原样抓的日志会这样），后面一行带假本机主机名；记录另加一行干净的字，好让这一格有新加的行可判。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf "%s\n" "/multi-host.env" > .gitignore
mkdir -p records research/results
printf "# 样本调度\n\n- 实一交回：跑了 5 轮。\n" > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf 'boot\0\nlogin on sample-local-host-77\n' > research/results/e999-serial.out
printf -- '- 第二台与本机各跑了一半。\n' >> records/2026-09-24-样本调度.md
