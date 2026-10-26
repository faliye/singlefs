#!/usr/bin/env bash
# 树里只有一份压成 .out.gz 的产物 e97-gz-uncited，没有任何一页点名它（点名 .out 或 .out.gz 都没有）：第一道判红；
# 页里点名的 e97-gz-nowhere-2026-10-22.out 树里没有 .out、也没有 .out.gz，历史里同样没有：第三道判红。
set -e
gzip -n -9 -f research/results/e97-gz-uncited-2026-10-22.out
