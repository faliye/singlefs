#!/usr/bin/env bash
# 与 hardcoded-quota-exempt-family-green 同一个拆分，只是挪进的文件叫 other_values.py（族外）：建 git 仓，基准里写死上限那一行在入口
# scripts/hosts/split_values.py 里、表里那一行写入口的路径，提交成基准；再放回工作区里拆好的样子。只起 git、cp、mv、sed。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv scripts/hosts/split_values.py .entry-now
mv .claude/gate.d/hardcoded-quota-exempt.tsv .table-now
cp scripts/hosts/other_values.py scripts/hosts/split_values.py
mv scripts/hosts/other_values.py .part-now
sed 's#other_values\.py#split_values.py#' .table-now > .claude/gate.d/hardcoded-quota-exempt.tsv
git add -A .claude scripts
git commit -qm base
mv .entry-now scripts/hosts/split_values.py
mv .part-now scripts/hosts/other_values.py
mv .table-now .claude/gate.d/hardcoded-quota-exempt.tsv
