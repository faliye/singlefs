#!/usr/bin/env bash
# 与 quoted-result-lines-history-added-red 同一个基准（历史节里早先有一行错引文 count=2）。
# 这次改动只在正文新抄一行对的（产物里逐字找得到），历史节里没动：早先的错行不判，成功行的「没判的」里列着它（页里第 12 行，正文新抄一行之后）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add -A
git commit -qm '基准：历史节里已有一行旧错引文'
sed -i 's/^E7RESULT name=verdict arm=a count=3$/E7RESULT name=verdict arm=a count=3\nE7RESULT name=done emitted=2/' .claude/kb/experiments/99-样本.md
