#!/usr/bin/env bash
# 新加的未跟踪产物 9 MiB：前 8 MiB 是填充行，假第二台主机名只在第 8 MiB 之后的一行里。格文件头写「新文件整份都算新加」，代码只读前 READ_LIMIT（8 MiB）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf "%s\n" "/multi-host.env" > .gitignore
mkdir -p records research/results
printf "# 样本调度\n\n- 实一交回：跑了 5 轮。\n" > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
python3 -c 'import sys; line = "progress 0123456789 abcdefghij klmnopqrstuvwxyz ABCDEFGHIJ\n"; sys.stdout.write(line * (9 * 1024 * 1024 // len(line))); sys.stdout.write("第二台 sample-peer-box-17 跑完了\n")' > research/results/e999-sample-long-run.out
