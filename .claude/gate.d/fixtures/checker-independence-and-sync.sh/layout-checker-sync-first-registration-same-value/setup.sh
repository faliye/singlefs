#!/usr/bin/env bash
# 绿样本的现场：SAMPLE_UNIT_BYTES 一直是 16384，.rs 源码这次没变；这次改动只是第一次给它
# 挂上 format-const 标记（补写一条早就成立的事实），checker 没被碰,也不该被要求跟。
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
printf '# 样本格式定义\n\n<!-- format-const: SAMPLE_UNIT_BYTES = 16384 -->\n' > format/布局.md
