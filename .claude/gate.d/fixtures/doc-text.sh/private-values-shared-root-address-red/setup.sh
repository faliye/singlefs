#!/usr/bin/env bash
# 基准里一份记录是干净的；这一轮往记录加三行：数据根（第 4 行，共享根，不报）、第二台的假地址（第 5 行，HOSTS 里的 IPv4 地址，照报）、第 6 行没有私有值。
# 只点名第 5 行：「新加的 1 行」。
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
- 第二台的地址是 172.31.250.199，跑完了。
- 第二台与本机各跑了一半。
Y
