#!/usr/bin/env bash
# 红样本的现场：SAMPLE_UNIT_BYTES 第一次挂标记的同一次改动里，.rs 源码的值也真的变了
# （16384 → 32768）；新标记记的是新值，不该被「首次登记」那条豁免——常量真的变了，checker
# 判定路径一个都没被碰,仍要判红。
set -euo pipefail
git init -q .
mkdir -p format/src checker/src   # git 不收空目录：干净的检出里没有这两个目录，不建的话下面的 printf 写不进去
git config user.email selftest@example.invalid
git config user.name selftest
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 16384;\n' > format/src/lib.rs
printf '# 样本格式定义\n\n单元大小固定为 16384 字节，这里先不登记标记。\n' > format/布局.md
printf '//! 样本 checker。\npub fn check() -> u64 { 16384 }\n' > checker/src/lib.rs
git add -A
git commit -qm '样本基准：常量 16384，还没登记标记'
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 32768;\n' > format/src/lib.rs
printf '# 样本格式定义\n\n<!-- format-const: SAMPLE_UNIT_BYTES = 32768 -->\n' > format/布局.md
