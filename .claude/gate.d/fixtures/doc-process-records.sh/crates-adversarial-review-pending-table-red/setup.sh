#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 挂账表自己有四处不合规（表是新的、基准里没有，只缩不涨不比）：第 1 行指向的判决不存在、第 2 行理由不到 8 个字、第 3 行的路径不是判决文件、
# 第 5 行重复登记了第 4 行的判决。lib.rs 在 demo-r1 的节里列了，对象这一头没毛病 ⇒ 本该判红，只因为表不合规，列四处。
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf '# demo 第一轮：主 agent 核实\n\n## 这一轮被判过的文件\n\n- crates/demo/src/lib.rs\n' > research/prompts/demo-r1-main-verification.md
printf '# short 第一轮：主 agent 核实\n' > research/prompts/short-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md research/prompts/short-r1-main-verification.md
printf 'research/prompts/gone-r1-main-verification.md\t判决已经不在仓里了的登记\nresearch/prompts/short-r1-main-verification.md\t理由短\ncrates/demo/src/lib.rs\t路径不是判决文件的登记\nresearch/prompts/demo-r1-main-verification.md\t这一节上线之前写的判决没有那一节\nresearch/prompts/demo-r1-main-verification.md\t同一份判决登记了两次\n' > .claude/gate.d/review-section-pending.tsv
