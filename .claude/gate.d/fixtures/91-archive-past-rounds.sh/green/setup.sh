#!/usr/bin/env bash
# 绿样本：版本库里没有上一轮的记录，工作区里只有本轮新写、还没提交的那几份。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts research/results
printf '占位\n' > README.md
git add -A
git commit -qm '基准：还没有任何实验记录'
printf '本轮的腿报告\n' > research/prompts/this-round-opus-output.md
printf 'E2 name=done emitted=1\n' > research/results/e2-this-round-2026-09-21.out
