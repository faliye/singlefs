#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮往记录加五行：假第二台主机名（第 4 行）、假路径（第 5 行）、假本机主机名（第 6 行）各一行，
# 第 7 行只有 4 个字的假短词（门槛 8 个字，不报），第 8 行没有私有值。点名前三行的键名。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样
mkdir -p records
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 实一交回：跑了 5 轮。
X
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 第二台上起过一趟，主机是 sample-peer-box-17，跑完了。
- 仓库放在 /srv/sample-private/peer-tree 下面。
- 这一趟在 sample-local-host-77 上跑。
- 短词 tiny7 只有四个字，不报。
- 第二台与本机各跑了一半。
X
