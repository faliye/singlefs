#!/usr/bin/env bash
# 改了定义、共用约束、主 agent 说明，另新建一份没跟踪的定义；同一次改动带来的判决按路径点名了这四份 ⇒ 本该判绿。
# 顺带改一份不归这道管的文件（README.md），它不该被算进去。
# 另删掉一份定义（gone.md），用户豁免了这次删除：豁免表里那一行第二段写 deleted ⇒ 按豁免算，不要求点名。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents research/prompts
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围。\n' > .claude/agent-common.md
printf '# 主 agent\n\n职责写在这里。\n' > .claude/main-agent.md
printf '# 样本仓\n' > README.md
printf '# gone\n\n1. 核查腿。\n' > .claude/agents/gone.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
git rm -q .claude/agents/gone.md
printf '.claude/agents/gone.md\tdeleted\t用户说删掉这份、这次不走三方\n' > .claude/agent-def-review-exempt
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围与草稿目录。\n' > .claude/agent-common.md
printf '# 主 agent\n\n职责与交回怎么读写在这里。\n' > .claude/main-agent.md
printf '# fresh\n\n1. 只读。\n' > .claude/agents/fresh.md
printf '# 样本仓（改）\n' > README.md
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

点名：.claude/agents/demo.md、.claude/agents/fresh.md、.claude/agent-common.md、.claude/main-agent.md。三条腿都没打中。
MD
