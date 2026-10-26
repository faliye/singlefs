#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 三份改过的产品代码，同一份新判决的「## 这一轮被判过的文件」一节里三份都出现：
# lib.rs 那一行写判了什么；skipped.rs 那一行写「这一轮没攻它」（列表项）；held.rs 那一行写「未判」（表格行）
# ⇒ 写了否定词的两行不算点名，本该判红只列 skipped.rs 与 held.rs，并提示它们只在写了否定词的行上出现。lib.rs 不许列进缺失。
mkdir -p crates/demo/src research/prompts .claude/gate.d
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn skipped() -> u32 { 1 }\n' > crates/demo/src/skipped.rs
printf 'pub fn held() -> u32 { 1 }\n' > crates/demo/src/held.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn skipped() -> u32 { 2 }\n' > crates/demo/src/skipped.rs
printf 'pub fn held() -> u32 { 2 }\n' > crates/demo/src/held.rs
{
  printf '# demo 第一轮：主 agent 核实\n\n'
  printf '## 一、证据核对\n\n三条腿只攻了 publish。\n\n'
  printf '## 这一轮被判过的文件\n\n'
  printf -- '- crates/demo/src/lib.rs：publish 返回 2，三条腿没打中\n'
  printf -- '- crates/demo/src/skipped.rs：这一轮没攻它\n\n'
  printf '| 文件 | 判了什么 |\n|---|---|\n| crates/demo/src/held.rs | 未判，三条腿都没读到它 |\n'
} > research/prompts/demo-r1-main-verification.md
git add research/prompts/demo-r1-main-verification.md
