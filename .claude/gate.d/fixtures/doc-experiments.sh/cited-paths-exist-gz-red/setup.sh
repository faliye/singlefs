#!/usr/bin/env bash
# 页里点名两个 .out：e9-gz-ok 树里只有压成的 .out.gz，找得到；e9-gz-nowhere 树里与历史里 .out 和 .out.gz 都没有 → 只红这一个（1 处引用）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=existence-unchecked 把每个路径都当成在，这份样本判绿，判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
gzip -n -9 -f research/results/e9-gz-ok.out
git init -q -b master .
git add -A
git commit -qm '基准：压成 .gz 的产物与 kb 页'
