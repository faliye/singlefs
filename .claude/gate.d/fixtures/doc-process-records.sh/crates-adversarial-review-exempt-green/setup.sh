#!/usr/bin/env bash
# 改了 crates/demo/src/lib.rs 与 crates/demo/src/comment.rs：前一份有新判决按路径点名，后一份用户豁免了三方，
# 豁免表 .claude/crates-review-exempt 按它改后的内容哈希登记 ⇒ 本该判绿、报 1 个豁免。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf '// 依据：已定项 3\npub fn comment() -> u32 { 1 }\n' > crates/demo/src/comment.rs
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf '// 依据：D3（样本） 已定项 3\npub fn comment() -> u32 { 2 }\n' > crates/demo/src/comment.rs
printf '# demo 第一轮：主 agent 核实\n\n改动：crates/demo/src/lib.rs。三条腿都没打中。\n' > research/prompts/demo-r1-main-verification.md
printf 'crates/demo/src/comment.rs\t%s\t用户说这次不走三方\n' \
  "$(sha256sum crates/demo/src/comment.rs | cut -d' ' -f1)" > .claude/crates-review-exempt
