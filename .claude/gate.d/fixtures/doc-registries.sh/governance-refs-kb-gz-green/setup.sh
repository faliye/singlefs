#!/usr/bin/env bash
# kb 中文页点名 research/results/ 下的 <名>.out，而那份产物只以 <名>.out.gz 存放，两处都不红：
#   research/results/e2-tree.out      树里只有 e2-tree.out.gz（提交着）                ⇒ 判过、指得到
#   research/results/e3-history.out   e3-history.out.gz 提交过、之后删掉（只在历史里） ⇒ research/ 下进过 git，放行、计 1
# 弄坏开关 GOVERNANCE_REFS_BREAK=gz-storage-ignored（只查原样的 .out）下两处都判红「从没进过 git」。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb research/results
printf -- '---\nname: sample\ndescription: governance-refs 那一格认 .out.gz 存放的绿样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '# 样例实验页\n\n这一轮的产物在 `research/results/e2-tree.out`。\n上一轮产物 `research/results/e3-history.out` 已归档。\n' > .claude/kb/results.md
printf 'name=e2 verdict=true\n' > research/results/e2-tree.out.gz
printf 'name=e3 verdict=true\n' > research/results/e3-history.out.gz
git add -A && git commit -qm 基准
git rm -q research/results/e3-history.out.gz && git commit -qm '归档上一轮压缩产物'
