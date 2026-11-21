#!/usr/bin/env bash
# 只改了测试代码：验证包 crates/singlefs-verification/ 的 src 三个（含 src/bin/ 下的入库装置）、
# 池级 checker crates/singlefs-checker/ 的 src 一个，
# 另改了 crates/singlefs-verification/tests/ 下一个测试与 crates/mutations.tsv；没有任何三方判决 ⇒ 测试代码同样是这一格的对象，
# 六份都没人点名，判红逐个列出（同一份改动有判决点名时判绿：crates-adversarial-review-test-code-green）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-verification/src crates/singlefs-verification/tests crates/singlefs-verification/src/bin crates/singlefs-checker/src research/prompts
printf 'pub fn workload() -> u32 { 1 }\n' > crates/singlefs-verification/src/model.rs
printf '#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
printf 'pub fn checked() -> u32 { 1 }\n' > crates/singlefs-verification/src/lib.rs
printf 'fn main() { println!("1"); }\n' > crates/singlefs-verification/src/bin/e1_demo.rs
printf 'pub fn pool_checked() -> u32 { 1 }\n' > crates/singlefs-checker/src/lib.rs
printf 'name\tfile\n' > crates/mutations.tsv
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
# 角色表把两个包登成验证：它们的 src 是非产品代码（弄坏开关 test-code-skipped 去掉的那一类）
printf '# 样本里的角色表\nsinglefs-verification\t验证\tcrates/singlefs-verification\t样本里的验证包\nsinglefs-checker\t验证\tcrates/singlefs-checker\t样本里的池级 checker\n' > .claude/gate.d/crate-roles.tsv
git add -A && git commit -qm base
printf 'pub fn workload() -> u32 { 2 }\n' > crates/singlefs-verification/src/model.rs
printf '#[test]\nfn workload_runs() { assert_eq!(2, 2); }\n' > crates/singlefs-verification/tests/workload.rs
printf 'pub fn checked() -> u32 { 2 }\n' > crates/singlefs-verification/src/lib.rs
printf 'fn main() { println!("2"); }\n' > crates/singlefs-verification/src/bin/e1_demo.rs
printf 'pub fn pool_checked() -> u32 { 2 }\n' > crates/singlefs-checker/src/lib.rs
printf 'name\tfile\nworkload-two\tcrates/singlefs-verification/src/model.rs\n' > crates/mutations.tsv
