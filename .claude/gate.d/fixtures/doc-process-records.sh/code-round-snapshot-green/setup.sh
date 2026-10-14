#!/usr/bin/env bash
# 五份新写的判决，三份点名了产品代码文件、快照都罩住 ⇒ 本该判绿，报 3 份判、2 份不判：
#   fresh-r1     点名 lib.rs 与 other.rs；快照目录这次新加（未跟踪），清单两份都有
#   tracked-r1   点名 other.rs 与 gone.rs；快照目录基准里就已跟踪；gone.rs 这次删了，没有内容可算哈希，清单没有它不碍事
#   中文题-r1    判决文件名是中文，点名 named.rs，快照目录同名（git 不带 core.quotepath=false 时中文文件名是带引号的八进制串，会认不出）
#   kbonly-r1    只点 kb 的推论轮，外加通配写法 crates/*/src/*.rs，不算点名文件，不判
#   testonly-r1  只点名 harness 档的测试代码，不判
# 另有基准里就有的旧判决 old-r1 点名 lib.rs、没有快照，这次只补了一句，不判。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src crates/singlefs-harness/src research/prompts/tracked-r1-snapshot
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn recover() -> u32 { 1 }\n' > crates/demo/src/other.rs
printf 'pub fn gone() -> u32 { 1 }\n' > crates/demo/src/gone.rs
printf 'pub fn named() -> u32 { 1 }\n' > crates/demo/src/named.rs
printf 'pub fn run() -> u32 { 1 }\n' > crates/singlefs-harness/src/runner.rs
printf '# old 第一轮：主 agent 核实\n\n改动：crates/demo/src/lib.rs 的 publish。三条腿都没打中。\n' > research/prompts/old-r1-main-verification.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
sha256sum crates/demo/src/other.rs > research/prompts/tracked-r1-snapshot/crates-sha256.txt
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn recover() -> u32 { 2 }\n' > crates/demo/src/other.rs
printf 'pub fn named() -> u32 { 2 }\n' > crates/demo/src/named.rs
printf 'pub fn run() -> u32 { 2 }\n' > crates/singlefs-harness/src/runner.rs
git rm -q crates/demo/src/gone.rs
printf '\n路径改写：旧路径换成新路径。\n' >> research/prompts/old-r1-main-verification.md
# tracked-r1 的快照目录基准里就在，清单按改后的 other.rs 重记一遍（仍是同一个已跟踪的文件）
sha256sum crates/demo/src/other.rs > research/prompts/tracked-r1-snapshot/crates-sha256.txt
cat > research/prompts/tracked-r1-main-verification.md <<'MD'
# tracked 第一轮：主 agent 核实

开工快照：research/prompts/tracked-r1-snapshot/。
改动：crates/demo/src/other.rs 的 recover 返回 2，crates/demo/src/gone.rs 删除。三条腿都没打中。
MD
mkdir research/prompts/fresh-r1-snapshot
sha256sum crates/demo/src/lib.rs crates/demo/src/other.rs > research/prompts/fresh-r1-snapshot/crates-sha256.txt
cat > research/prompts/fresh-r1-main-verification.md <<'MD'
# fresh 第一轮：主 agent 核实

开工快照：`research/prompts/fresh-r1-snapshot/`（crates 2 份）。
改动：crates/demo/src/lib.rs、crates/demo/src/other.rs:12 的返回值。三条腿都没打中。
MD
mkdir "research/prompts/中文题-r1-snapshot"
sha256sum crates/demo/src/named.rs > "research/prompts/中文题-r1-snapshot/crates-sha256.txt"
printf '# 中文题 第一轮：主 agent 核实\n\n开工快照：research/prompts/中文题-r1-snapshot/。\n改动：crates/demo/src/named.rs 的 named 返回 2。\n' > "research/prompts/中文题-r1-main-verification.md"
cat > research/prompts/kbonly-r1-main-verification.md <<'MD'
# kbonly 第一轮：主 agent 核实

只推论 .claude/kb/pitfalls.md 里的条文；通配写法 crates/*/src/*.rs 说的是整个目录，不是哪一份文件。
MD
cat > research/prompts/testonly-r1-main-verification.md <<'MD'
# testonly 第一轮：主 agent 核实

改动：crates/singlefs-harness/src/runner.rs 的 run 返回 2（harness 档测试代码）。
MD
git add research/prompts/fresh-r1-main-verification.md research/prompts/fresh-r1-snapshot research/prompts/kbonly-r1-main-verification.md
