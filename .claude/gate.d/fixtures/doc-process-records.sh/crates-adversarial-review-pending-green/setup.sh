#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 挂账表 .claude/gate.d/review-section-pending.tsv：基准里有两行（old-r1、demo-r1），工作区只剩 demo-r1（只缩不涨：删了一行不算涨）。
# demo-r1 是这一次新写的判决、没有「## 这一轮被判过的文件」一节，只在正文里提到 lib.rs ⇒ 按挂账表认整份正文，本该判绿，并逐个报出按挂账表认的判决。
# old-r1 是基准里就有的旧判决（不是这一次新写的）：表里的行本来就不指望它，这一行删掉不碍事。
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf '# old 第一轮：主 agent 核实\n\n改动：旧的。\n' > research/prompts/old-r1-main-verification.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '# 挂账表（样本）\nresearch/prompts/old-r1-main-verification.md\t这一节上线之前写的判决没有那一节\nresearch/prompts/demo-r1-main-verification.md\t这一节上线之前写的判决没有那一节\n' > .claude/gate.d/review-section-pending.tsv
git add -A && git commit -qm base
printf '# 挂账表（样本）\nresearch/prompts/demo-r1-main-verification.md\t这一节上线之前写的判决没有那一节\n' > .claude/gate.d/review-section-pending.tsv
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf '# demo 第一轮：主 agent 核实\n\n改动：crates/demo/src/lib.rs 的 publish 返回 2。三条腿都没打中。\n' > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
