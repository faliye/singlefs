#!/usr/bin/env bash
# 基准里滞后表只登记 a.rs 一行；这一轮往滞后表加 b.rs 一行：滞后表比基准那一版多 1 行，判红（两份的命中都算滞后）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p crates/demo/src .claude/gate.d
printf '%s\n' '// 这个常量此前只认 --bin' 'pub const A: u32 = 1;' '// 原先的写法是按行读' 'pub const B: u32 = 2;' > crates/demo/src/a.rs
printf '%s\n' '// 曾经在这里读过文件' 'pub const C: u32 = 3;' > crates/demo/src/b.rs
{
  printf '%s\n' '# 样本：注释历史写法的滞后表（doc-text 的 comment-history 格）。一行「<路径><TAB><命中行数><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' 'crates/demo/src/a.rs' '2' '# 样本：等这一批改完'
} > .claude/gate.d/comment-history-lag.tsv
git add -A && git commit -qm base
printf '%s\t%s\t%s\n' 'crates/demo/src/b.rs' '1' '# 样本：新写的句子不改，往表里加了一行' >> .claude/gate.d/comment-history-lag.tsv
