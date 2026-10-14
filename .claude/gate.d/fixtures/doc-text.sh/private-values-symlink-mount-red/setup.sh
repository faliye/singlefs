#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮在被 git 忽略的 mnt/ 下建以假第二台主机名命名的挂载点（第二台的目录挂在本机），里面一份干净的产物，
# 再放一个指到它的未跟踪符号链接：目标存在、内容干净，跟着链接走的判法读到的是干净内容；先判链接、比链接文字才红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/setup.sh' '/.private-values-hostname' '/mnt/' > .gitignore   # 挂载点在 git 忽略的 mnt/ 下，不进改动范围
mkdir -p records research/results
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 实一交回：跑了 5 轮。
X
git add -A && git commit -qm base
mkdir -p mnt/sample-peer-box-17/results
printf 'name=e999 rows=3 clean\n' > mnt/sample-peer-box-17/results/e999.out
ln -s ../../mnt/sample-peer-box-17/results/e999.out research/results/peer-e999.out
