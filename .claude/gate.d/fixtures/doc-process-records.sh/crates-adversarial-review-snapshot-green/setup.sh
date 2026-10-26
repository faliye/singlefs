#!/usr/bin/env bash
# 判决之后没再改：crates/demo/src/lib.rs 被 demo-r1 的判决在「## 这一轮被判过的文件」一节里点名，那一轮开工快照 research/prompts/demo-r1-snapshot/crates-sha256.txt
# 记的哈希与此刻的内容对得上 ⇒ 判绿、报 1 个对得上。另改了 crates/demo/src/plain.rs，点名它的 plain-r1 那一轮没有快照目录：没比、逐个列出，不判红
# （开工快照立没立住归 code-round-snapshot 格）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn plain() -> u32 { 1 }\n' > crates/demo/src/plain.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：只放一条碰不到这份样本改动的正则
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
mkdir -p research/prompts/demo-r1-snapshot
sha256sum crates/demo/src/lib.rs > research/prompts/demo-r1-snapshot/crates-sha256.txt
printf '# demo 第一轮：主 agent 核实\n\n开工快照：research/prompts/demo-r1-snapshot/。三条腿都没打中。\n\n## 这一轮被判过的文件\n\n- crates/demo/src/lib.rs：publish 返回 2\n' \
  > research/prompts/demo-r1-main-verification.md
printf 'pub fn plain() -> u32 { 2 }\n' > crates/demo/src/plain.rs
printf '# plain 第一轮：主 agent 核实\n\n三条腿都没打中。\n\n## 这一轮被判过的文件\n\n- crates/demo/src/plain.rs：plain 返回 2\n' \
  > research/prompts/plain-r1-main-verification.md
