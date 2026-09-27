#!/usr/bin/env bash
# 绿样本的现场：.rs 里的整数字面量带下划线分隔（805_306_368），format-const 标记的文法不许
# 带下划线，只能写 805306368；两边字面不同、数值相同，第一次挂标记时不能被误判成「变了」——
# 这正是真仓 5 个常量里 JOURNAL_RING_DEFAULT_BYTES、ROOT_RING_CHUNK_BYTES 会撞上的那种写法。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
printf '//! 样本格式常量模块。\npub const SAMPLE_RING_BYTES: u64 = 805_306_368;\n' > format/src/lib.rs
printf '# 样本格式定义\n\n环长固定为 805306368 字节，这里先不登记标记。\n' > format/布局.md
printf '//! 样本 checker。\npub fn check() -> u64 { 805306368 }\n' > checker/src/lib.rs
git add -A
git commit -qm '样本基准：常量 805_306_368，还没登记标记'
printf '# 样本格式定义\n\n<!-- format-const: SAMPLE_RING_BYTES = 805306368 -->\n' > format/布局.md
