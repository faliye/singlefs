#!/usr/bin/env bash
# 基准里存量清单登了一行；工作区往清单里又加一行（登的那一句「走不到」基准里就在 kb 里，kb 这次没改），只缩不涨 ⇒ 判红点名清单第 3 行。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb .claude/gate.d crates/demo/src
printf 'pub fn noop() {}\n' > crates/demo/src/lib.rs
printf '# 欠账\n\n- C1 第二台掉线那一格触发不了。\n- C2 两盘都旧时走不到重同步那一支。\n' > .claude/kb/checks-owed.md
printf '# 这一格立起来时 kb 里已有的断言行\n.claude/kb/checks-owed.md\t- C1 第二台掉线那一格触发不了。\n' > .claude/gate.d/unreachable-claims-stock.tsv
git add -A && git commit -qm base
printf '.claude/kb/checks-owed.md\t- C2 两盘都旧时走不到重同步那一支。\n' >> .claude/gate.d/unreachable-claims-stock.tsv
