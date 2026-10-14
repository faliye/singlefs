#!/usr/bin/env bash
# 攻方样本（零轮形态 K10 的路径比对）：从第二台把一个目录 tar 过来，tar 去掉开头的 / 之后，第二台的仓路径整段成了本地路径的一部分。
# 假私有值 PEER_REPOSITORY_DIRECTORY=/srv/sample-private/peer-tree 跨了几段目录：逐段打码打不到，红行把路径原样打出来。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' > .gitignore
mkdir -p records research/results
printf '# 样本调度\n\n- 干净的一行。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
mkdir -p research/results/from-peer/srv/sample-private/peer-tree/research/results
printf 'name=e999 rows=3\n' > research/results/from-peer/srv/sample-private/peer-tree/research/results/e999.out
