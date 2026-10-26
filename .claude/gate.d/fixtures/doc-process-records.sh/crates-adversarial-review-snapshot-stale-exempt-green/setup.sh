#!/usr/bin/env bash
# 与 crates-adversarial-review-snapshot-stale-red 同一个现场（判决之后 crates/demo/src/lib.rs 又从返回 2 改成返回 3），
# 另在 .claude/crates-review-exempt 按此刻那一版（返回 3）的内容哈希登记了豁免 ⇒ 判绿、报 1 个豁免；kept.rs 与快照对得上。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn kept() -> u32 { 1 }\n' > crates/demo/src/kept.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：只放一条碰不到这份样本改动的正则
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn kept() -> u32 { 2 }\n' > crates/demo/src/kept.rs
mkdir -p research/prompts/demo-r1-snapshot
sha256sum crates/demo/src/lib.rs crates/demo/src/kept.rs > research/prompts/demo-r1-snapshot/crates-sha256.txt
printf '# demo 第一轮：主 agent 核实\n\n开工快照：research/prompts/demo-r1-snapshot/。三条腿都没打中。\n\n## 这一轮被判过的文件\n\n- crates/demo/src/lib.rs：publish 返回 2\n- crates/demo/src/kept.rs：kept 返回 2\n' \
  > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md research/prompts/demo-r1-snapshot
# 判决之后又改了一处，用户明说这一版不再走三方：按改后的内容哈希登记
printf 'pub fn publish() -> u32 { 3 }\n' > crates/demo/src/lib.rs
printf 'crates/demo/src/lib.rs\t%s\t用户说判决之后这一处小改不再走三方\n' \
  "$(sha256sum crates/demo/src/lib.rs | cut -d' ' -f1)" > .claude/crates-review-exempt
