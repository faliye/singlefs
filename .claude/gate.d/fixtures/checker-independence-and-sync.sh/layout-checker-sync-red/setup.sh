#!/usr/bin/env bash
# 红样本的现场：基准提交里常量是 16384，工作区把它改成 32768，而 checker 一个字没动。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 16384;\n' > format/src/lib.rs
git add -A
git commit -qm '样本基准：常量 16384'
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 32768;\n' > format/src/lib.rs
