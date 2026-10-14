#!/usr/bin/env bash
# 从第二台进度日志抄来的两句：时间戳前 20 个字落在假私有值中间（样本的 multi-host.env 里 PEER_REPOSITORY_DIRECTORY 与 PEER_SSH_HOST，编的），clock-times 红；
# 片段两头挪到值的边上、整值打码，红行里不出现半截的假值。建 git 仓只为让 expect、setup.sh 与假配置被 .gitignore 挡在扫描之外。
set -e
git init -q .
printf '%s\n' '/multi-host.env' '/.private-values-hostname' '/setup.sh' '/expect' '/.gate-cells' > .gitignore
mkdir -p records
stamp_head="2026-10-13T" colon=":"
printf '%s\n' '# 样本调度' '' \
  "- 第二台进度日志抄来的一句：跑在 /srv/sample-private/peer-tree 上 ${stamp_head}10${colon}31${colon}05 收尾。" \
  "- 对照：第二台 sample-peer-box-17 上 ${stamp_head}11${colon}02${colon}00 开跑。" > records/样本调度.md
