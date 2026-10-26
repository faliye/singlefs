#!/usr/bin/env bash
# 定义轮判决之后又改过：demo-r1 那一轮开工快照 research/prompts/demo-r1-snapshot/sha256sums.txt（定义轮的旧叫法，路径带 ./ 前缀）记的是
# .claude/agents/demo.md 改后第一版，判决写完之后又加了一行 ⇒ 判红只列 demo.md。共用约束记在同一目录的 judged-sha256.txt 里、没再改，对得上不列。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents research/prompts .claude/gate.d
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围。\n' > .claude/agent-common.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：只放一条碰不到这份样本改动的正则
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围与草稿目录。\n' > .claude/agent-common.md
mkdir -p research/prompts/demo-r1-snapshot
sha256sum ./.claude/agents/demo.md > research/prompts/demo-r1-snapshot/sha256sums.txt
sha256sum .claude/agent-common.md > research/prompts/demo-r1-snapshot/judged-sha256.txt
{
  printf '# demo 第一轮：主 agent 核实\n\n开工快照：research/prompts/demo-r1-snapshot/。三条腿都没打中。\n\n'
  printf '## 这一轮被判过的文件\n\n- .claude/agents/demo.md：退出码要贴\n- .claude/agent-common.md：写范围多一句\n'
} > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md research/prompts/demo-r1-snapshot
# 判决之后又改了一处
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n2. 交回前删副本。\n' > .claude/agents/demo.md
