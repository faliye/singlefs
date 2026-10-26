#!/usr/bin/env bash
# 登记着的产物 e98-archived-gz-2026-10-22.out.gz 只在历史里有：先随基准提交（压成 .out.gz 入库的样子，gzip -n 压出来的字节每次相同），再按归档规则从树里删掉。
# 历史里从没有过 .out 收尾的那一份。实验页停在未跟踪状态，共用脚本把它整份算新增，整行抄的产物行才被判。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=archived-gz-not-searched：去历史里找归档产物时只认 .out 收尾的路径，.out.gz 找不到。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
gzip -n -9 -f research/results/e98-archived-gz-2026-10-22.out
git init -q -b master .
git add research scripts
git commit -qm '基准：压成 .gz 的登记产物'
git rm -q research/results/e98-archived-gz-2026-10-22.out.gz
git commit -qm '按归档规则删掉登记产物（留在历史里）'
