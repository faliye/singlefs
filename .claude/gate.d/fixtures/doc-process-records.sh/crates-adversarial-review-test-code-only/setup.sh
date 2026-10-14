#!/usr/bin/env bash
# 只改了测试代码：harness 档包 crates/singlefs-harness/ 的 src 一个、checker 档包 crates/singlefs-checker-tier/ 的 src 两个（含 src/bin/ 下的入库装置）、
# 池级 checker crates/singlefs-checker/ 的 src 一个，
# 另改了 crates/singlefs-harness/tests/ 下一个测试与 crates/mutations.tsv；没有任何三方判决 ⇒ 这一格无对象可判、退 77，
# 逐个报出不判的四个 src 文件（tests/ 与 mutations.tsv 不是 crates/*/src/*.rs，不列）。把测试代码也算进对象就会判红（期望 77，实测 1）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-harness/src crates/singlefs-harness/tests crates/singlefs-checker-tier/src/bin crates/singlefs-checker/src research/prompts
printf 'pub fn workload() -> u32 { 1 }\n' > crates/singlefs-harness/src/lib.rs
printf '#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-harness/tests/workload.rs
printf 'pub fn checked() -> u32 { 1 }\n' > crates/singlefs-checker-tier/src/lib.rs
printf 'fn main() { println!("1"); }\n' > crates/singlefs-checker-tier/src/bin/e1_demo.rs
printf 'pub fn pool_checked() -> u32 { 1 }\n' > crates/singlefs-checker/src/lib.rs
printf 'name\tfile\n' > crates/mutations.tsv
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'pub fn workload() -> u32 { 2 }\n' > crates/singlefs-harness/src/lib.rs
printf '#[test]\nfn workload_runs() { assert_eq!(2, 2); }\n' > crates/singlefs-harness/tests/workload.rs
printf 'pub fn checked() -> u32 { 2 }\n' > crates/singlefs-checker-tier/src/lib.rs
printf 'fn main() { println!("2"); }\n' > crates/singlefs-checker-tier/src/bin/e1_demo.rs
printf 'pub fn pool_checked() -> u32 { 2 }\n' > crates/singlefs-checker/src/lib.rs
printf 'name\tfile\nworkload-two\tcrates/singlefs-harness/src/lib.rs\n' > crates/mutations.tsv
