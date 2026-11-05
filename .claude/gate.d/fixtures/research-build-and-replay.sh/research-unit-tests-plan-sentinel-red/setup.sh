#!/usr/bin/env bash
# research-build-and-replay.sh 的 research-unit-tests 格「复用哨兵报红」样本的现场：git init（按条复用的判定库名取 git common-dir，
# 样本根不是 git 工作树时 test-plan 在算键之前就退 2，到不了样本档标记造的假哨兵）
set -euo pipefail
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
