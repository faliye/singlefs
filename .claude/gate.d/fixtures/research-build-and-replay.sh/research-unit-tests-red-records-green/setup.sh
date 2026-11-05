#!/usr/bin/env bash
# research-build-and-replay.sh 的 research-unit-tests 格「有红照记」样本的现场：git init（按条复用的判定库放 git common-dir），
# 没有上一版，三条单测都现判
set -euo pipefail
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
