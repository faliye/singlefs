#!/usr/bin/env bash
# 索引与产物进基准提交、上游设在那里；新写的实验页停在**未跟踪**状态，它点名的产物树里与历史里都没有。
# git diff 看不见未跟踪的页，只有共用脚本把它整份算新增，第三道才判得到（三方判决 gate-fix-forks-r1 的 T5）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add .claude/kb/experiments.md research
git commit -qm '索引与产物'
git branch 上游 && git branch -q --set-upstream-to=上游 master
