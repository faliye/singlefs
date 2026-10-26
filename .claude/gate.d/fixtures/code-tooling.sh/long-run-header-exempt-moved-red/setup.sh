#!/usr/bin/env bash
# 与 long-run-header-exempt-moved-green 同一个搬迁，只是没有搬迁登记表：建 git 仓，把 scripts/new-rig 放回搬家之前的 scripts/old-rig（脚本里的路径一起换回去），
# 表登记旧路径与旧文件的 sha256，提交成基准；再 git mv 到 scripts/new-rig、放回工作区的脚本与表。只起 git、sed、sha256sum。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
cp scripts/new-rig/live.sh .live-new
cp .claude/long-run-live-hosts-exempt .table-new
mkdir -p scripts/old-rig
sed 's#scripts/new-rig#scripts/old-rig#g' .live-new > scripts/old-rig/live.sh
rm scripts/new-rig/live.sh
rmdir scripts/new-rig
printf '%s\n' '# 样本：基准那一版（搬家之前）' "scripts/old-rig/live.sh  $(sha256sum scripts/old-rig/live.sh | cut -d' ' -f1)  # 样本：文件头按现算写、不能改" > .claude/long-run-live-hosts-exempt
git add -A .claude scripts
git commit -qm base
git mv scripts/old-rig scripts/new-rig
mv .live-new scripts/new-rig/live.sh
mv .table-new .claude/long-run-live-hosts-exempt
