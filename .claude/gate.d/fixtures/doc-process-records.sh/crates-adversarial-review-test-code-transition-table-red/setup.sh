#!/usr/bin/env bash
# 过渡挂账表写坏五行：理由太短（验证包的测试）、路径是产品代码不是这张表认的那几类、变异表登记两次、只有一段没有理由（池级 checker 的测试）；
# 另一行合规。改了的两份（验证包的测试与变异表）都登记着、没有判决 ⇒ 表本身判红列四处，而不是靠登记放过去。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-verification/tests crates/singlefs-checker/tests crates/demo/src .claude/gate.d
printf '#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
printf '#[test]\nfn pool_checked() { assert_eq!(1, 1); }\n' > crates/singlefs-checker/tests/pool.rs
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf 'name\tfile\n' > crates/mutations.tsv
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '#[test]\nfn workload_runs() { assert_eq!(2, 2); }\n' > crates/singlefs-verification/tests/workload.rs
printf 'name\tfile\nworkload-two\tcrates/singlefs-verification/tests/workload.rs\n' > crates/mutations.tsv
printf 'crates/singlefs-verification/tests/workload.rs\t短理由\n' > .claude/gate.d/review-test-code-pending.tsv
printf 'crates/demo/src/lib.rs\t这是产品代码的路径不该登记在这里\n' >> .claude/gate.d/review-test-code-pending.tsv
printf 'crates/mutations.tsv\t第一次登记变异表的理由\n' >> .claude/gate.d/review-test-code-pending.tsv
printf 'crates/mutations.tsv\t第二次登记变异表的理由\n' >> .claude/gate.d/review-test-code-pending.tsv
printf 'crates/singlefs-checker/tests/pool.rs\n' >> .claude/gate.d/review-test-code-pending.tsv
