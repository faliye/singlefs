#!/usr/bin/env bash
# 定义轮判决之后没再改：.claude/agents/demo.md 在 demo-r1 那一轮开工快照里有一行（sha256sum -b 的二进制模式，路径前带 *），与此刻的内容对得上 ⇒ 判绿、报 1 个对得上。
# 主 agent 入口同一份判决点名，快照里没有它的行：没比、逐个列出，不判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents research/prompts .claude/gate.d
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# 主 agent\n\n职责写在这里。\n' > .claude/main-agent.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：只放一条碰不到这份样本改动的正则
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '# 主 agent\n\n职责与交回怎么读写在这里。\n' > .claude/main-agent.md
mkdir -p research/prompts/demo-r1-snapshot
sha256sum -b .claude/agents/demo.md > research/prompts/demo-r1-snapshot/judged-sha256.txt
{
  printf '# demo 第一轮：主 agent 核实\n\n开工快照：research/prompts/demo-r1-snapshot/。三条腿都没打中。\n\n'
  printf '## 这一轮被判过的文件\n\n- .claude/agents/demo.md：退出码要贴\n- .claude/main-agent.md：交回怎么读\n'
} > research/prompts/demo-r1-main-verification.md
