#!/usr/bin/env bash
# 新增的两句断言都写明是推的：「触发不了」那一句写「推的，没量过」，「不可达」那一句只写「推的」 ⇒ 判绿。新 kb 页没进 git（未跟踪），整份算新增。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb crates/demo/src
printf 'pub fn noop() {}\n' > crates/demo/src/lib.rs
printf '# 欠账\n\n- C1 已还清。\n' > .claude/kb/checks-owed.md
git add -A && git commit -qm base
printf '# 掉线\n\n第二台掉线时重放那一格触发不了（推的，没量过）。\n\n空盘挂载之后回到只读那一支不可达，推的。\n' > .claude/kb/drop.md
