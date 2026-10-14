#!/usr/bin/env bash
# 一个仓：kb 页点名四个具体路径，实验源码在树里；产物 e9-missing.out 与变异表 e9-no-such-table.tsv 树里没有、历史里也从没有过 → 红两处（3 处引用）。
# 同一页里写成区间（c9-r1..r6-local.out）、带通配（e9-*.out）、只写文件名（e9-bare.out）的三种不是具体路径，不认，不进计数。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=existence-unchecked 把每个路径都当成在，这份样本判绿，判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add -A
git commit -qm '基准：kb 页与实验源码'
