#!/usr/bin/env bash
# 要建 git 仓：不是 git 仓时第 ④ 条（日期）报「未跑」退 77，弄坏开关打开后这份红样本就不是「判绿」而是「没判完」。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm base'
