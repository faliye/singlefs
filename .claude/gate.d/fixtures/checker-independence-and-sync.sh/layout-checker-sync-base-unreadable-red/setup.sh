#!/usr/bin/env bash
# 红样本的现场：基准里 .rs 与布局.md 的标记都是 16384，这次两份都没改；再把基准那一版布局.md 的对象从对象库里删掉：
# git ls-tree 认得出名字、git show 读不出（工作区那一份没改，改动范围照取得到）。
# 取不到基准那一版不许当成「基准里没有这份」：改前取不到交空串，布局.md 的标记被当成新出现、又撞上 .rs 没变且同值，
# 报「首次登记」判绿（这一份的基准其实没比）；改后经 lib-lag-baseline.py 的 baseline_text 取，取不到判红、点名那一份。
set -euo pipefail
git init -q .
mkdir -p format/src checker/src   # git 不收空目录：干净的检出里没有这两个目录，不建的话下面的 printf 写不进去
git config user.email selftest@example.invalid
git config user.name selftest
printf '//! 样本格式常量模块。\npub const SAMPLE_UNIT_BYTES: u64 = 16384;\n' > format/src/lib.rs
printf '# 样本格式定义\n\n<!-- format-const: SAMPLE_UNIT_BYTES = 16384 -->\n' > format/布局.md
printf '//! 样本 checker。\npub fn check() -> u64 { 16384 }\n' > checker/src/lib.rs
git add -A
git commit -qm '样本基准：.rs 与标记都是 16384'
layout_blob="$(git rev-parse HEAD:format/布局.md)"
rm -f ".git/objects/${layout_blob:0:2}/${layout_blob:2}"
