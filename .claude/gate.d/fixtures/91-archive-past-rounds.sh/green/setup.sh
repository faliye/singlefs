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
# 上游设在基准提交，再做一次没推的提交写一份同步记录：「这一轮」的起点要取上游的 merge-base（与门禁 68 号同一份取法），
# 这份记录是这一轮的、不许被判成上一轮的；只认「GATE_BASE，否则 HEAD」时它会被判该删。
git branch upstream-sample
git branch -q --set-upstream-to=upstream-sample
printf '<!-- knowledge-sync -->\n这一轮已提交、还没推的同步记录\n' > research/prompts/this-round-sync.md
git add research/prompts/this-round-sync.md
git commit -qm '这一轮：没推的一次提交'
