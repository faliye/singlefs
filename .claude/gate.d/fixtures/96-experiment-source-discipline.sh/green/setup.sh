#!/usr/bin/env bash
# 绿样本的现场：两张豁免表在基准提交里就已经是现在这个样子，「只缩不涨」那一条才有基线可比。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
