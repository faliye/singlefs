#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮在 research/results/ 下新放一份 9 MiB 的未跟踪日志（每段打一行进度），假第二台主机名只在最后一行，已经在 8 MiB 之后：
# 整份读才看得到，只读前 8 MiB 会漏。点名最后一行的行号与键名。
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
python3 -c 'import sys; line = "progress 0123456789 abcdefghij klmnopqrstuvwxyz ABCDEFGHIJ\n"; count = 9 * 1024 * 1024 // len(line); sys.stdout.write(line * count); sys.stdout.write("第二台 sample-peer-box-17 跑完了\n")' > research/results/e999-sample-long-run.out
