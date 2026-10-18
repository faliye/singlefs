#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 三份改过的产品代码，同一份新判决：lib.rs 列在「## 这一轮被判过的文件」一节里；
# moved.rs 只在节前面的「一、证据核对」正文里被提到；after.rs 只在节后面的「三、处置」正文里被提到（节到下一个二级标题为止）
# ⇒ 后两份不算被判过，本该判红只列它们，并提示判决在正文别处提到过。lib.rs 不许列进缺失。
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn moved() -> u32 { 1 }\n' > crates/demo/src/moved.rs
printf 'pub fn after() -> u32 { 1 }\n' > crates/demo/src/after.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn moved() -> u32 { 2 }\n' > crates/demo/src/moved.rs
printf 'pub fn after() -> u32 { 2 }\n' > crates/demo/src/after.rs
{
  printf '# demo 第一轮：主 agent 核实\n\n'
  printf '## 一、证据核对\n\ncrates/demo/src/moved.rs 的 moved 返回 2，三条腿没打中。\n\n'
  printf '## 这一轮被判过的文件\n\n- crates/demo/src/lib.rs\n\n'
  printf '## 三、处置\n\ncrates/demo/src/after.rs 的 after 也判了，没打中。\n'
} > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
