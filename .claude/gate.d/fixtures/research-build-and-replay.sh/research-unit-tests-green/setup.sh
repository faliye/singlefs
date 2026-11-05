#!/usr/bin/env bash
# research-build-and-replay.sh 的 research-unit-tests 格绿样本的现场：git init（按条复用的判定库名取 git common-dir，
# 不是 git 工作树的样本根算不出库名，按条复用算不出，这一格判红）
set -euo pipefail
git init -q .
