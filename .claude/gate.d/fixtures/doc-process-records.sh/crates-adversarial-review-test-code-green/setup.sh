#!/usr/bin/env bash
# 改了验证包 crates/singlefs-verification/ 的 src 三个（含 src/bin/ 下的入库装置）、池级 checker crates/singlefs-checker/ 的 src 一个、
# 验证包 tests/ 下一个测试与 crates/mutations.tsv；这一次新写的判决在「## 这一轮被判过的文件」一节里逐个点名了这六份 ⇒ 本该判绿、报 6 个。
# 测试代码不再从对象里去掉：同一份改动没有判决时判红（crates-adversarial-review-test-code-only）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-verification/src/bin crates/singlefs-verification/tests crates/singlefs-checker/src research/prompts .claude/gate.d
printf 'pub fn workload() -> u32 { 1 }\n' > crates/singlefs-verification/src/model.rs
printf '#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
printf 'pub fn checked() -> u32 { 1 }\n' > crates/singlefs-verification/src/lib.rs
printf 'fn main() { println!("1"); }\n' > crates/singlefs-verification/src/bin/e1_demo.rs
printf 'pub fn pool_checked() -> u32 { 1 }\n' > crates/singlefs-checker/src/lib.rs
printf 'name\tfile\n' > crates/mutations.tsv
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
# 角色表把两个包登成验证：它们的 src 是非产品代码（弄坏开关 test-code-skipped 去掉的那一类）
printf '# 样本里的角色表\nsinglefs-verification\t验证\tcrates/singlefs-verification\t样本里的验证包\nsinglefs-checker\t验证\tcrates/singlefs-checker\t样本里的池级 checker\n' > .claude/gate.d/crate-roles.tsv
git add -A && git commit -qm base
printf 'pub fn workload() -> u32 { 2 }\n' > crates/singlefs-verification/src/model.rs
printf '#[test]\nfn workload_runs() { assert_eq!(2, 2); }\n' > crates/singlefs-verification/tests/workload.rs
printf 'pub fn checked() -> u32 { 2 }\n' > crates/singlefs-verification/src/lib.rs
printf 'fn main() { println!("2"); }\n' > crates/singlefs-verification/src/bin/e1_demo.rs
printf 'pub fn pool_checked() -> u32 { 2 }\n' > crates/singlefs-checker/src/lib.rs
printf 'name\tfile\nworkload-two\tcrates/singlefs-verification/src/model.rs\n' > crates/mutations.tsv
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

三条腿都没打中。

## 这一轮被判过的文件

- crates/singlefs-verification/src/model.rs
- crates/singlefs-verification/src/lib.rs
- crates/singlefs-verification/src/bin/e1_demo.rs
- crates/singlefs-checker/src/lib.rs
- crates/singlefs-verification/tests/workload.rs
- crates/mutations.tsv
MD
