#!/usr/bin/env bash
# 改了一份定义与共用约束，新写的判决只按路径点名了定义 ⇒ 共用约束没人点名，本该判红。
# 另改了主 agent 入口，唯一点名它的是一份基准里就有、这次只被改过（补了一句）的旧判决 ⇒ 旧判决不算点名，它也该列进缺失。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents research/prompts
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围。\n' > .claude/agent-common.md
printf '# 主 agent\n\n职责写在这里。\n' > .claude/main-agent.md
printf '# old 第一轮：主 agent 核实\n\n点名：.claude/main-agent.md。三条腿都没打中。\n' > research/prompts/old-r1-main-verification.md
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '# 共用约束\n\n- 只写写范围与草稿目录。\n' > .claude/agent-common.md
printf '# 主 agent\n\n职责与交回怎么读写在这里。\n' > .claude/main-agent.md
printf '\n路径改写：旧路径换成新路径。\n' >> research/prompts/old-r1-main-verification.md
printf '# demo 第一轮：主 agent 核实\n\n改动：.claude/agents/demo.md 第 1 步多贴退出码。三条腿都没打中。\n' > research/prompts/demo-r1-main-verification.md
