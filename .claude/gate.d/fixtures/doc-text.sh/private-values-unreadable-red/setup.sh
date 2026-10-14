#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮从第二台把一个产物目录 tar 回来（tar 去掉开头的 / 之后，第二台的假仓路径整段成了本地路径的一部分），
# 里面一份权限 000 的 .out（权限位跟着 tar 过来），另有一份干净的新日志。四格都读到这份文件：读不了不许崩出带原路径的回溯，路径经打码。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/setup.sh' '/.private-values-hostname' > .gitignore   # setup.sh 里造路径的那几行不进扫描
mkdir -p records research/results
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 实一交回：跑了 5 轮。
X
git add -A && git commit -qm base
printf 'name=e999 rows=3 clean\n' > research/results/e999-local.log
d=research/results/from-peer/srv/sample-private/peer-tree/research/results
mkdir -p "$d"
printf 'name=e999 rows=3\n' > "$d/e999.out"
chmod 000 "$d/e999.out"
