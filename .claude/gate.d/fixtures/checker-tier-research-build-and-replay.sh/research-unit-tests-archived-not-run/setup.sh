#!/usr/bin/env bash
# checker-tier-research-build-and-replay.sh 的 research-unit-tests 格「读不到留存产物本次未跑」样本的现场：git init（按条复用的判定库放 git common-dir），
# 没有上一版，两条单测都现判
set -euo pipefail
git init -q .
