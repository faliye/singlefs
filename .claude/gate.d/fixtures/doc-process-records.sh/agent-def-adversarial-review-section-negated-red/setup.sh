#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 改了定义、共用约束与主 agent 入口；同一份新判决的「## 这一轮被判过的文件」一节里三份都出现：
# demo.md 那一行写判了什么；共用约束那一行写「没判」（列表项）；主 agent 入口那一行写「这一轮没攻它」（三级小标题底下）
# ⇒ 写了否定词的两行不算点名，本该判红只列共用约束与主 agent 入口，并提示它们只在写了否定词的行上出现。demo.md 不许列进缺失。
mkdir -p .claude/agents research/prompts .claude/gate.d
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围。\n' > .claude/agent-common.md
printf '# 主 agent\n\n职责写在这里。\n' > .claude/main-agent.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围与草稿目录。\n' > .claude/agent-common.md
printf '# 主 agent\n\n职责与交回怎么读写在这里。\n' > .claude/main-agent.md
{
  printf '# demo 第一轮：主 agent 核实\n\n三条腿只攻了定义的那一句。\n\n'
  printf '## 这一轮被判过的文件\n\n'
  printf -- '- .claude/agents/demo.md：贴原样输出多了退出码，三条腿没打中\n'
  printf -- '- .claude/agent-common.md：没判，留到改它的那一轮\n\n'
  printf '### 入口\n\n- .claude/main-agent.md：这一轮没攻它\n'
} > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
