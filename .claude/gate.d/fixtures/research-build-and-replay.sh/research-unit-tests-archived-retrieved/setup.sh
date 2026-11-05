#!/usr/bin/env bash
# research-build-and-replay.sh 的 research-unit-tests 格「留存产物只在 git 历史里、取回后单测读得到」样本的现场：
# git init，把 research/results/sample-product.out 先入库再删掉（按「每次提交删上一次的实验记录」归档的样子）
set -euo pipefail
git init -q .
mkdir -p research/results
printf 'sample product\n' >research/results/sample-product.out
git add -f research/results/sample-product.out
git -c user.name=selftest -c user.email=selftest@example.invalid -c commit.gpgsign=false commit -q -m 'sample product'
git rm -q research/results/sample-product.out
git -c user.name=selftest -c user.email=selftest@example.invalid -c commit.gpgsign=false commit -q -m 'archive the sample product'
