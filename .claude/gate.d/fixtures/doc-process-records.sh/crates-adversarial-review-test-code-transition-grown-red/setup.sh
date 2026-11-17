#!/usr/bin/env bash
# 过渡挂账表在基准里已经有一行（验证包的一个测试）；工作区又加了一行（池级 checker 的一个测试，路径合规、理由够长），两份测试都改了、没有判决
# ⇒ 只缩不涨，新加的一行判红：改了测试代码不能靠往表里加行放过去。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-verification/tests crates/singlefs-checker/tests .claude/gate.d
printf '#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
printf '#[test]\nfn pool_checked() { assert_eq!(1, 1); }\n' > crates/singlefs-checker/tests/pool.rs
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf 'crates/singlefs-verification/tests/workload.rs\t存量测试代码还没补三方\n' > .claude/gate.d/review-test-code-pending.tsv
git add -A && git commit -qm base
printf '#[test]\nfn workload_runs() { assert_eq!(2, 2); }\n' > crates/singlefs-verification/tests/workload.rs
printf '#[test]\nfn pool_checked() { assert_eq!(2, 2); }\n' > crates/singlefs-checker/tests/pool.rs
printf 'crates/singlefs-verification/tests/workload.rs\t存量测试代码还没补三方\ncrates/singlefs-checker/tests/pool.rs\t新改的测试想靠挂账表放过去\n' > .claude/gate.d/review-test-code-pending.tsv
