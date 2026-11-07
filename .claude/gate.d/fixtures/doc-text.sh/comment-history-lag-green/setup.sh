#!/usr/bin/env bash
# 基准里两份 .rs 各有命中、滞后表登记两行；这一轮把第二份改净、删掉它那一行（只缩）：判绿，报滞后 2 行、与基准比没涨。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p crates/demo/src .claude/gate.d
printf '%s\n' '// 这个常量此前只认 --bin' 'pub const A: u32 = 1;' '// 原先的写法是按行读' 'pub const B: u32 = 2;' > crates/demo/src/a.rs
printf '%s\n' '// 曾经在这里读过文件' 'pub const C: u32 = 3;' > crates/demo/src/b.rs
{
  printf '%s\n' '# 样本：注释历史写法的滞后表（doc-text 的 comment-history 格）。一行「<路径><TAB><命中行数><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' 'crates/demo/src/a.rs' '2' '# 样本：等这一批改完'
  printf '%s\t%s\t%s\n' 'crates/demo/src/b.rs' '1' '# 样本：等这一批改完'
} > .claude/gate.d/comment-history-lag.tsv
git add -A && git commit -qm base
printf '%s\n' '// 读文件。' 'pub const C: u32 = 3;' > crates/demo/src/b.rs
python3 - <<'PY'
table = '.claude/gate.d/comment-history-lag.tsv'
rows = [row for row in open(table, encoding='utf-8').read().split('\n') if not row.startswith('crates/demo/src/b.rs')]
open(table, 'w', encoding='utf-8').write('\n'.join(rows))
PY
