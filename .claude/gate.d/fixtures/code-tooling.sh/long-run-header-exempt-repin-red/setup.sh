#!/usr/bin/env bash
# 豁免表「只缩不涨按路径 + sha256 成对比」那一条要一个基准：建 git 仓，提交改正文之前的 live.sh（slots=7）和登记它的 sha256 的表，
# 再把工作区的 live.sh（slots=8）与重钉了 sha256 的表放回来（路径与行数都没变，只有 sha256 换了）。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
cp scripts/live.sh .live-new
cp .claude/long-run-live-hosts-exempt .table-new
sed 's/^slots=8$/slots=7/' .live-new > scripts/live.sh
printf '%s\n' '# 样本：基准那一版（改正文之前）' "scripts/live.sh  $(sha256sum scripts/live.sh | cut -d' ' -f1)  # 样本：文件头按现算写、不能改" > .claude/long-run-live-hosts-exempt
git add -A .claude scripts
git commit -qm base
mv .live-new scripts/live.sh
mv .table-new .claude/long-run-live-hosts-exempt
