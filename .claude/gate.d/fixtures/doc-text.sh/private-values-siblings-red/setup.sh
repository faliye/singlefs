#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮从第二台把一个产物目录 tar 过来，tar 去掉开头的 / 之后，第二台的仓路径整段成了本地路径的一部分（假私有值
# PEER_REPOSITORY_DIRECTORY=/srv/sample-private/peer-tree 自己带 /、跨了几级目录）。目录里三份产物：干净的 .out，带 ISO 时间戳的 .log（clock-times 红），
# 前 8 KiB 里有 NUL 的 .img（prime-marks 与 session-names 判绿后列「二进制：」）。内容里没有私有值，只有路径里有。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/setup.sh' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样；setup.sh 里造路径的那几行不进扫描
mkdir -p records research/results
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 实一交回：跑了 5 轮。
X
git add -A && git commit -qm base
d=research/results/from-peer/srv/sample-private/peer-tree/research/results
mkdir -p "$d"
printf 'name=e999 rows=3\n' > "$d/e999.out"
printf 'log_opened 2026-09-24T05:49:46\n' > "$d/e999.log"
printf 'IMG\0\0\0\1' > "$d/e999.img"
