#!/usr/bin/env bash
# 滞后表绿样本的现场：滞后表在基准提交里就已经是现在这个样子，「只缩不涨」那一条才有基线可比。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两组重复行登记为滞后'
