#!/usr/bin/env bash
# 验证包 src 里的一份与验证包 tests/ 下的一份都只改了注释（行注释、行尾注释），没有判决 ⇒ 测试代码只改注释同样不走三方，这一格无对象可判、逐个报出来。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/singlefs-verification/src crates/singlefs-verification/tests .claude/gate.d
printf '// 旧说明\npub fn workload() -> u32 { 1 }\n' > crates/singlefs-verification/src/model.rs
printf '// 旧说明\n#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '// 新说明，写清为什么\npub fn workload() -> u32 { 1 } // 行尾补一句\n' > crates/singlefs-verification/src/model.rs
printf '// 新说明，写清这条测试压着什么\n#[test]\nfn workload_runs() { assert_eq!(1, 1); }\n' > crates/singlefs-verification/tests/workload.rs
