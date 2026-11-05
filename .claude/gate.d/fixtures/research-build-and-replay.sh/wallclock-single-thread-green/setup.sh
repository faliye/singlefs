#!/usr/bin/env bash
# research-build-and-replay.sh 的 research-unit-tests 格「挂钟断言在单线程那一趟里跑到」样本的现场：git init（按条复用的判定库放 git common-dir），没有上一版
set -euo pipefail
git init -q .
