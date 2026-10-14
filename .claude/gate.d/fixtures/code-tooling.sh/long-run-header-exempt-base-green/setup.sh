#!/usr/bin/env bash
# 豁免表有基准（建 git 仓，把表与脚本原样提交为基准）而路径与 sha256 都没变：只缩不涨按路径 + sha256 成对比，判绿并写「与基准那一版比没涨」。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add -A .claude scripts
git commit -qm base
