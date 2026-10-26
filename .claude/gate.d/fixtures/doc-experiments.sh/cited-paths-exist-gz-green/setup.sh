#!/usr/bin/env bash
# 产物压成 .out.gz 入库之后，页里点名 <名>.out：e9-gz-tree 树里只有 .out.gz；e9-gz-gone 的 .out.gz 先随基准提交、再从工作区删掉（只在历史里）→ 都算找得到，绿。
# 样本里放的是 .out，这里压成 .out.gz 并删掉原文件（gzip -n 不写文件名与时刻，字节每次相同）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=cited-path-gz-unmatched 只查原样的 .out，两个都算缺，这份样本判红，判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
gzip -n -9 -f research/results/e9-gz-tree.out research/results/e9-gz-gone.out
git init -q -b master .
git add -A
git commit -qm '基准：压成 .gz 的产物与 kb 页'
git rm -q research/results/e9-gz-gone.out.gz
git commit -qm '删掉一份压缩产物（留在历史里）'
