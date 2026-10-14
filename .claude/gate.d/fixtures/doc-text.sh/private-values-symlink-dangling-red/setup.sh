#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮放一份干净的新日志，外加一个未跟踪的符号链接，链接文字是第二台的假仓路径（本机不存在，坏链接）：
# git 进仓的是链接文字本身，跟着链接走的判法整份跳过它；先判链接、比链接文字才红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/setup.sh' '/.private-values-hostname' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样；setup.sh 里造链接的那一行不进比对
mkdir -p records research/results
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 实一交回：跑了 5 轮。
X
git add -A && git commit -qm base
printf 'name=e999 rows=3 clean\n' > research/results/e999-local.log
ln -s /srv/sample-private/peer-tree/research/results/e999 research/results/peer-latest
