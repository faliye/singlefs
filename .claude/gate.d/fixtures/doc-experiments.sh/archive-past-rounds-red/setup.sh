#!/usr/bin/env bash
# 红样本：上一轮的记录已经提交进版本库、还留在工作区——正是这一道要赶走的那一批。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts research/results
printf '上一轮的腿报告\n' > research/prompts/old-round-opus-output.md
printf 'E1 name=done emitted=1\n' > research/results/e1-old-round-2026-09-21.out
git add -A
git commit -qm '上一轮：记录进了版本库'
