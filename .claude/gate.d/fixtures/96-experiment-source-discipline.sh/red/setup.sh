#!/usr/bin/env bash
# 红样本的现场：豁免表进了基准提交，再把基准里那一版的 blob 从对象库里删掉——git 取不到基准那一版，
# 「只缩不涨」必须判红，不许当成「基准里还没有这张表、初次登记」放过去。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：一张挂在认不出的欠账上的豁免表'
blob="$(git rev-parse HEAD:.claude/gate.d/experiment-seed-fold-lag.tsv)"
rm -f ".git/objects/${blob:0:2}/${blob:2}"
