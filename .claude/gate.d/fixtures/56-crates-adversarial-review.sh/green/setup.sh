#!/usr/bin/env bash
# 改了 crates/demo/src/lib.rs，这一次改动新写（暂存区新增）的三方判决文件按路径点名了它 ⇒ 本该判绿。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf '# demo 第一轮：主 agent 核实\n\n改动：crates/demo/src/lib.rs 的 publish 返回 2。三条腿都没打中。\n' > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
