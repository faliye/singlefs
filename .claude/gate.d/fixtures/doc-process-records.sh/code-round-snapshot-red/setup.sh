#!/usr/bin/env bash
# 改了 crates/demo/src/lib.rs 与 other.rs，新判决（暂存新增）点名了两份，也写了开工快照目录，目录在树里、有 crates-sha256.txt，
# 但清单只列了 lib.rs ⇒ 本该判红，只列缺的 other.rs。
# 漏的只有清单覆盖：CODE_ROUND_SNAPSHOT_BREAK=snapshot-coverage-skipped 下这一例转绿，门禁判别力自证因此判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/demo/src research/prompts
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'pub fn recover() -> u32 { 1 }\n' > crates/demo/src/other.rs
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf 'pub fn recover() -> u32 { 2 }\n' > crates/demo/src/other.rs
mkdir research/prompts/demo-r1-snapshot
sha256sum crates/demo/src/lib.rs > research/prompts/demo-r1-snapshot/crates-sha256.txt
: > research/prompts/demo-r1-snapshot/kb-sha256.txt
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

开工快照：research/prompts/demo-r1-snapshot/（crates 文件与 kb 文件的 sha256）。
改动：crates/demo/src/lib.rs 的 publish 返回 2，crates/demo/src/other.rs 的 recover 返回 2。三条腿都没打中。
MD
git add research/prompts/demo-r1-main-verification.md research/prompts/demo-r1-snapshot
