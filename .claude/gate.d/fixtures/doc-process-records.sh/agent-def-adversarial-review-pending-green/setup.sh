#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 改了一份定义；这一次新写的判决没有「## 这一轮被判过的文件」一节、只在正文里提到它，挂账表（未提交、基准里没有这张表）登记了这份判决
# ⇒ 按挂账表认整份正文，本该判绿，并逐个报出按挂账表认的判决（两格共用同一张表）。
mkdir -p .claude/agents research/prompts .claude/gate.d
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '# demo 第一轮：主 agent 核实\n\n改动：.claude/agents/demo.md 第 1 步多贴退出码。三条腿都没打中。\n' > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
printf 'research/prompts/demo-r1-main-verification.md\t这一节上线之前写的判决没有那一节\n' > .claude/gate.d/review-section-pending.tsv
