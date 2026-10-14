#!/usr/bin/env bash
# 攻方样本（X5）：第一轮 V7 那一段历史（从第二台 tar 一个产物目录过来，第二台仓路径整段成了本地路径的一部分），目录里多一份带 NUL 的二进制产物、一份带 ISO 时间戳的日志。
# 假私有值同第一轮样本（PEER_REPOSITORY_DIRECTORY=/srv/sample-private/peer-tree，编的）。private-values 把路径打码；同一道 doc-text 的 prime-marks / clock-times / session-names 三格原样打路径。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/setup.sh' > .gitignore
mkdir -p records research/results
printf '# 样本调度\n\n- 干净的一行。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
d=research/results/from-peer/srv/sample-private/peer-tree/research/results
mkdir -p "$d"
printf 'name=e999 rows=3\n' > "$d/e999.out"
printf 'started 2026-10-13T05:49:46\n' > "$d/e999.log"
printf 'IMG\0\0\0\1' > "$d/e999.img"
