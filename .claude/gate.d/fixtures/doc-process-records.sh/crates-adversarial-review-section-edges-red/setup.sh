#!/usr/bin/env bash
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 五份改过的产品代码，两份新判决：
#   节判决 一份，有合格的一节，只列 ok.rs；
#   形不对的判决 一份，没有合格的一节：level3.rs 列在三级标题「### 这一轮被判过的文件」底下、fenced.rs 列在代码围栏里的同名标题底下、
#   suffixed.rs 列在标题带后缀「## 这一轮被判过的文件（共 1 份）」的一节底下（标题要逐字）、nosection.rs 只在正文里提到
# ⇒ 本该判红只列后四份。
mkdir -p crates/demo/src research/prompts .claude/gate.d
for name in ok level3 fenced suffixed nosection; do printf 'pub fn %s() -> u32 { 1 }\n' "$name" > "crates/demo/src/$name.rs"; done
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
for name in ok level3 fenced suffixed nosection; do printf 'pub fn %s() -> u32 { 2 }\n' "$name" > "crates/demo/src/$name.rs"; done
printf '# ok 第一轮：主 agent 核实\n\n## 这一轮被判过的文件\n\n- crates/demo/src/ok.rs\n' > research/prompts/ok-r1-main-verification.md
{
  printf '# bad 第一轮：主 agent 核实\n\ncrates/demo/src/nosection.rs 只在这段正文里提到。\n\n'
  printf '### 这一轮被判过的文件\n\n- crates/demo/src/level3.rs\n\n'
  printf '```\n## 这一轮被判过的文件\n\n- crates/demo/src/fenced.rs\n```\n\n'
  printf '## 这一轮被判过的文件（共 1 份）\n\n- crates/demo/src/suffixed.rs\n'
} > research/prompts/bad-r1-main-verification.md
git add research/prompts/ok-r1-main-verification.md research/prompts/bad-r1-main-verification.md
