#!/usr/bin/env bash
# 基准里的旧行就带假第二台主机名（只判新加的行，旧行不报）；新加的行只有 4 个字的假短词、通用词与「第二台」「本机」这类说法。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' > .gitignore   # 真仓里 multi-host.env 被 git 忽略，样本里照样
mkdir -p records
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度

- 旧行：第二台的主机是 sample-peer-box-17，早就写在这里了。
X
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 第二台上起过一趟，跑完了。
- 短词 tiny7 只有四个字，不报。
- 清场用 systemctl 与 is-active 这类通用命令，不报。
- 本机与第二台各跑了一半。
X
