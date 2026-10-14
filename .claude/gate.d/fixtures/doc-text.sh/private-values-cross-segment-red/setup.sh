#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮从第二台把一个目录 tar 过来，tar 去掉开头的 / 之后，第二台的仓路径整段成了本地路径的一部分：
# 假私有值 PEER_REPOSITORY_DIRECTORY=/srv/sample-private/peer-tree 自己带 /、跨了几级目录，逐段比哪一段都不含整串，红行会把它原样打出来；
# 按整串在整条路径里找，命中的那一段才打成「〔文件名含私有值〕」。第二份产物的路径里假第二台主机名与假仓路径相连，两处命中并成一段。内容都是干净的。
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
mkdir -p research/results/from-peer/srv/sample-private/peer-tree/research/results
printf 'name=e999 rows=3\n' > research/results/from-peer/srv/sample-private/peer-tree/research/results/e999.out
mkdir -p research/results/sample-peer-box-17/srv/sample-private/peer-tree
printf 'name=e998 rows=3\n' > research/results/sample-peer-box-17/srv/sample-private/peer-tree/e998.out
