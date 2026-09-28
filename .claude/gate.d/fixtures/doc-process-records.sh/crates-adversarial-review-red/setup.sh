#!/usr/bin/env bash
# 改了 crates/demo/src/lib.rs，没有任何三方判决文件点名它 ⇒ 本该判红。
# 另改了 crates/demo/src/other.rs，唯一点名它的是一份基准里就有、这次只被改过（补了一句）的旧判决 ⇒ 旧判决不算点名，它也该列进缺失。
# 再改了 crates/demo/src/named.rs，点名它的是一份中文文件名、未跟踪的新判决 ⇒ 它算点名过，不许列进缺失
# （git 不带 core.quotepath=false 时那个文件名是带引号的八进制串，对不上 *-main-verification.md，缺失就成了 3 个）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn recover() -> u32 { 1 }\n' > crates/demo/src/other.rs
printf 'pub fn named() -> u32 { 1 }\n' > crates/demo/src/named.rs
printf '# old 第一轮：主 agent 核实\n\n改动：crates/demo/src/other.rs 的 recover。三条腿都没打中。\n' > research/prompts/old-r1-main-verification.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn recover() -> u32 { 2 }\n' > crates/demo/src/other.rs
printf 'pub fn named() -> u32 { 2 }\n' > crates/demo/src/named.rs
printf '# 中文题 第一轮：主 agent 核实\n\n改动：crates/demo/src/named.rs 的 named 返回 2。三条腿都没打中。\n' > research/prompts/中文题-r1-main-verification.md
printf '\n路径改写：旧路径换成新路径。\n' >> research/prompts/old-r1-main-verification.md
