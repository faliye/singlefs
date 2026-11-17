#!/usr/bin/env bash
# 基准里的豁免表有两行（other.rs 一行、comment.rs 一行，后一行按改后那一版的内容哈希登记）；工作区删了 other.rs 那一行，只缩不涨不红，
# comment.rs 改了、没有判决点名，按表里留着的那一行豁免 ⇒ 本该判绿、报 1 个豁免。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src .claude/gate.d
printf 'pub fn other() -> u32 { 1 }\n' > crates/demo/src/other.rs
printf 'pub fn comment() -> u32 { 1 }\n' > crates/demo/src/comment.rs
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf 'crates/demo/src/other.rs\t%s\t存量里的另一行豁免\ncrates/demo/src/comment.rs\t%s\t存量里的一行豁免\n' \
  "$(sha256sum crates/demo/src/other.rs | cut -d' ' -f1)" \
  "$(printf 'pub fn comment() -> u32 { 2 }\n' | sha256sum | cut -d' ' -f1)" > .claude/crates-review-exempt
git add -A && git commit -qm base
printf 'pub fn comment() -> u32 { 2 }\n' > crates/demo/src/comment.rs
printf 'crates/demo/src/comment.rs\t%s\t存量里的一行豁免\n' "$(sha256sum crates/demo/src/comment.rs | cut -d' ' -f1)" > .claude/crates-review-exempt
