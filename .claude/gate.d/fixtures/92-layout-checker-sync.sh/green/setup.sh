#!/usr/bin/env bash
# 绿样本的现场：同一处常量改动，checker 在同一次改动里跟着改了。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 16384;\n' > format/src/lib.rs
printf '//! 样本 checker。\npub fn check() -> u64 { 16384 }\n' > checker/src/lib.rs
git add -A
git commit -qm '样本基准：常量 16384'
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 32768;\n' > format/src/lib.rs
printf '//! 样本 checker。\npub fn check() -> u64 { 32768 }\n' > checker/src/lib.rs
