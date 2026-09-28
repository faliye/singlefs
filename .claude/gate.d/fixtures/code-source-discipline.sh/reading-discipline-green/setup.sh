#!/usr/bin/env bash
# 绿样本的现场（reading-discipline 那一格要的）：两张豁免表在基准提交里就已经是现在这个样子，「只缩不涨」那一条才有基线可比。
# 别的两格不读 git，在这个仓里照判。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
