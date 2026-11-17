#!/usr/bin/env bash
# 过渡挂账表 .claude/gate.d/review-test-code-pending.tsv 在基准里有三行（验证包的一个测试、池级 checker 的一个测试、变异表）；工作区删了变异表那一行，
# 改了前两份、没有判决 ⇒ 登记的路径这一格不要求点名，只缩不涨不红，本该判绿、报 2 个挂账、逐个列出登记的理由。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-verification/tests crates/singlefs-checker/tests .claude/gate.d
printf '#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
printf '#[test]\nfn pool_checked() { assert_eq!(1, 1); }\n' > crates/singlefs-checker/tests/pool.rs
printf 'name\tfile\n' > crates/mutations.tsv
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '# 样本里的过渡挂账表\ncrates/singlefs-verification/tests/workload.rs\t存量测试代码还没补三方\ncrates/singlefs-checker/tests/pool.rs\t存量测试代码还没补三方\ncrates/mutations.tsv\t存量变异表还没补三方\n' > .claude/gate.d/review-test-code-pending.tsv
git add -A && git commit -qm base
printf '#[test]\nfn workload_runs() { assert_eq!(2, 2); }\n' > crates/singlefs-verification/tests/workload.rs
printf '#[test]\nfn pool_checked() { assert_eq!(2, 2); }\n' > crates/singlefs-checker/tests/pool.rs
printf '# 样本里的过渡挂账表\ncrates/singlefs-verification/tests/workload.rs\t存量测试代码还没补三方\ncrates/singlefs-checker/tests/pool.rs\t存量测试代码还没补三方\n' > .claude/gate.d/review-test-code-pending.tsv
