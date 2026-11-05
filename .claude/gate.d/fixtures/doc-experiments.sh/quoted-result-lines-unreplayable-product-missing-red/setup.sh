#!/usr/bin/env bash
# 产物先进基准提交，整行抄产物行的那一页停在**未跟踪**状态（登记表也未跟踪）：共用脚本把未跟踪文件整份算新增，整行抄的产物行才被判。
# 复跑表（replay.sh 骨架）只登记 e99-sample-2026-11-03.out；分片跑出的产物不在复跑表里，能不能被找到只看登记表。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add research scripts
git commit -qm '产物'
git branch 上游 && git branch -q --set-upstream-to=上游 master
