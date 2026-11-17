#!/usr/bin/env bash
# 基准里已有豁免表 .claude/crates-review-exempt（一行：comment.rs）；工作区又加了一行，按改后那一版的内容哈希豁免 lib.rs，lib.rs 没有判决点名
# ⇒ 豁免表只缩不涨，新加的一行判红（行本身的哈希对得上，所以靠它能放过去的路被堵死）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn comment() -> u32 { 1 }\n' > crates/demo/src/comment.rs
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf 'crates/demo/src/comment.rs\t%s\t存量里的一行豁免\n' "$(sha256sum crates/demo/src/comment.rs | cut -d' ' -f1)" > .claude/crates-review-exempt
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'crates/demo/src/lib.rs\t%s\t想靠新加一行豁免过去\n' "$(sha256sum crates/demo/src/lib.rs | cut -d' ' -f1)" >> .claude/crates-review-exempt
