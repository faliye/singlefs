#!/usr/bin/env bash
# 产物先进基准提交，抄错了的那一页停在**未跟踪**状态：新写的 kb 页在 git add 之前，
# git diff 看不见它，只有共用脚本把未跟踪文件整份算新增才判得到（三方判决 gate-fix-forks-r1 的 T5）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add research
git commit -qm '产物'
# 上游设在基准提交：旧的取法（先认 gate-ok、再认上游的 tip）在这里取得到基准，于是只看 git diff、漏掉未跟踪的那一页
git branch 上游 && git branch -q --set-upstream-to=上游 master
