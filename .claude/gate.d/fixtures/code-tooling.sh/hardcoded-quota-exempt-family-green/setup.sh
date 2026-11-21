#!/usr/bin/env bash
# 排除表只缩不涨认拆分：建 git 仓，基准里写死上限那一行在入口 scripts/hosts/split_values.py 里、表里那一行写入口的路径，提交成基准；
# 再放回工作区里拆好的样子（入口只剩说明、那一行在部分文件 split_values_cap.py 里、表里那一行改成部分文件的路径）。只起 git、cp、mv、sed。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv scripts/hosts/split_values.py .entry-now
mv .claude/gate.d/hardcoded-quota-exempt.tsv .table-now
cp scripts/hosts/split_values_cap.py scripts/hosts/split_values.py
mv scripts/hosts/split_values_cap.py .part-now
sed 's#split_values_cap\.py#split_values.py#' .table-now > .claude/gate.d/hardcoded-quota-exempt.tsv
git add -A .claude scripts
git commit -qm base
mv .entry-now scripts/hosts/split_values.py
mv .part-now scripts/hosts/split_values_cap.py
mv .table-now .claude/gate.d/hardcoded-quota-exempt.tsv
