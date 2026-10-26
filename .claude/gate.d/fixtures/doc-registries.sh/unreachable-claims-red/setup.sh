#!/usr/bin/env bash
# 基准之后 kb 新增两句断言：一句「不可达」什么都没带；一句「走不到」带了 ghost_case_rejects，可 crates/ 下只有 #[test] fn real_case_rejects，
# 名字对不上 ⇒ 两句逐句点名判红（后一句另注找不到定义）。基准里早就有的「触发不了」那一句不是新增，不判。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb crates/demo/tests
printf '#[test]\nfn real_case_rejects() {\n    assert!(true);\n}\n' > crates/demo/tests/reject.rs
printf '# 欠账\n\n- C1 第二台掉线那一格触发不了。\n' > .claude/kb/checks-owed.md
git add -A && git commit -qm base
printf '# 欠账\n\n- C1 第二台掉线那一格触发不了。\n- C2 掉盘之后挂成可写这一格不可达。\n- C3 两盘都旧时走不到重同步那一支（用例 `ghost_case_rejects`）。\n' > .claude/kb/checks-owed.md
