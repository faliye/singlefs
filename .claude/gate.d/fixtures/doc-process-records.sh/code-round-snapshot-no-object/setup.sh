#!/usr/bin/env bash
# 两份新写的判决，一份只点 kb（含通配写法 crates/*/src/*.rs）、一份只点名测试代码 ⇒ 没有对象可判，退 77，逐份报出不判的原因。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-checker/src research/prompts
printf 'pub fn check() -> u32 { 1 }\n' > crates/singlefs-checker/src/lib.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn check() -> u32 { 2 }\n' > crates/singlefs-checker/src/lib.rs
cat > research/prompts/kbonly-r1-main-verification.md <<'MD'
# kbonly 第一轮：主 agent 核实

只推论 .claude/kb/pitfalls.md 里的条文；通配写法 crates/*/src/*.rs 说的是整个目录，不是哪一份文件。
MD
cat > research/prompts/testonly-r1-main-verification.md <<'MD'
# testonly 第一轮：主 agent 核实

改动：crates/singlefs-checker/src/lib.rs 的 check 返回 2（池级 checker，测试代码）。
MD
git add research/prompts/kbonly-r1-main-verification.md
