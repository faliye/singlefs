#!/usr/bin/env bash
# 绿样本的现场：同一处常量改动，checker 在同一次改动里跟着改了。
# 布局清单登记的 checker 判定路径是中文文件名 checker/判定.rs：git 默认把它打成八进制引号串，
# 门禁不带 core.quotepath=false 时认不出它被改过，这一格就会假红。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 16384;\n' > format/src/lib.rs
printf '//! 样本 checker。\npub fn check() -> u64 { 16384 }\n' > checker/src/lib.rs
printf '//! 样本 checker 里文件名是中文的那一份。\npub fn check_by_chinese_path() -> u64 { 16384 }\n' > checker/判定.rs
git add -A
git commit -qm '样本基准：常量 16384'
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 32768;\n' > format/src/lib.rs
printf '//! 样本 checker。\npub fn check() -> u64 { 32768 }\n' > checker/src/lib.rs
printf '//! 样本 checker 里文件名是中文的那一份。\npub fn check_by_chinese_path() -> u64 { 32768 }\n' > checker/判定.rs
