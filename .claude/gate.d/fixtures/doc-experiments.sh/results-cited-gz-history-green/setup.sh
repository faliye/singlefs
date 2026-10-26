#!/usr/bin/env bash
# 两份产物都压成 .out.gz（超过单文件推送上限、压成 .gz 入库的样子），先随基准提交；archived 那一份再从树里删掉（提交过、树里没有，只在历史里有 .out.gz）。
# 实验页停在未跟踪状态：共用脚本把未跟踪的页整份算新增，第三道才判得到它点名的产物。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-citation-unmatched：第三道只查原样的 .out，历史里的 .out.gz 找不到，这份样本判红，判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
gzip -n -9 -f research/results/e98-gz-kept-2026-10-22.out research/results/e98-gz-archived-2026-10-22.out
git init -q -b master .
git add research
git commit -qm '基准：两份压成 .gz 的产物'
git rm -q research/results/e98-gz-archived-2026-10-22.out.gz
git commit -qm '删掉一份压缩产物（留在历史里）'
