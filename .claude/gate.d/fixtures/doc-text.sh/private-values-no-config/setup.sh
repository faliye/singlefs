#!/usr/bin/env bash
# 没有 multi-host.env 的机器（别人的克隆、第二台）：这一轮加了一行，没有私有值可比，这一格退 77。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样
mkdir -p records
printf '# 样本调度\n\n- 实一交回：跑了 5 轮。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf '%s\n' '- 第二台上起过一趟，跑完了。' >> records/2026-09-24-样本调度.md
