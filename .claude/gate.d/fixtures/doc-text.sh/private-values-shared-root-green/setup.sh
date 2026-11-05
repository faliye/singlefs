#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮往记录加四行：数据根本身（第 4 行）、数据根下面的路径（第 5 行）、另一段共享根（第 6 行），第 7 行没有私有值。
# 共享根两台同一绝对路径，不当私有值：四行都不报；两个假地址照取，报「2 个长词」。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样
mkdir -p records
cat > records/2026-09-24-样本调度.md <<'Y'
# 样本调度

- 实一交回：跑了 5 轮。
Y
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'Y'
- 数据根放在 /srv/sample-shared/data-root 下面，两台同一路径。
- 产物在 /srv/sample-shared/data-root/work/sample-run 里。
- 仓在 /srv/sample-shared/code-tree，两台都挂着。
- 第二台与本机各跑了一半。
Y
