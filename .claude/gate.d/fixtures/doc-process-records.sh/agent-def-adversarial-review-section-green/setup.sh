#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 改了定义、共用约束与主 agent 入口；新判决的「## 这一轮被判过的文件」一节里列了它们（定义写在列表项里、共用约束写在表格行里、主 agent 入口写在三级小标题底下）⇒ 本该判绿。
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
  printf '# demo 第一轮：主 agent 核实\n\n三条腿都没打中。\n\n'
  printf '## 这一轮被判过的文件\n\n- .claude/agents/demo.md\n\n'
  printf '| 文件 | 判了什么 |\n|---|---|\n| .claude/agent-common.md | 写范围多一句 |\n\n'
  printf '### 入口\n\n- .claude/main-agent.md\n'
} > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
