#!/usr/bin/env bash
# 已跟踪的实验页：基准提交里「## 历史版本」已有一行错引文（count=2，产物里没有），早先就在，不判。
# 这次改动在历史节末尾又新增一行错引文（count=9，登记着的产物里没有）：新增的行照判，必须红并只点名这一行（页里第 12 行）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-added-lines-skipped 退回整节排除，这份样本判绿，判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add -A
git commit -qm '基准：历史节里已有一行旧错引文'
printf 'E7RESULT name=verdict arm=a count=9\n' >> .claude/kb/experiments/99-样本.md
