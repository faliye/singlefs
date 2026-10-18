#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 挂账表在基准里已经有了（一行：old-r1）；工作区又加了一行 demo-r1（这一次新写的判决，文件在、理由够长、路径对）⇒ 只缩不涨，新加的一行判红。
# lib.rs 只在 demo-r1 的正文里提到，靠这一行才能算被点名：表只缩不涨，所以不许靠新加的行放过。
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf '# old 第一轮：主 agent 核实\n\n改动：旧的。\n' > research/prompts/old-r1-main-verification.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf 'research/prompts/old-r1-main-verification.md\t这一节上线之前写的判决没有那一节\n' > .claude/gate.d/review-section-pending.tsv
git add -A && git commit -qm base
printf 'research/prompts/old-r1-main-verification.md\t这一节上线之前写的判决没有那一节\nresearch/prompts/demo-r1-main-verification.md\t新写的判决想靠挂账表放过去\n' > .claude/gate.d/review-section-pending.tsv
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf '# demo 第一轮：主 agent 核实\n\n改动：crates/demo/src/lib.rs 的 publish 返回 2。三条腿都没打中。\n' > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
