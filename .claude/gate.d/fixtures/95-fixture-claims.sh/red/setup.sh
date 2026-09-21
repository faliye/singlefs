#!/usr/bin/env bash
# 红样本要判的第三条是「样本目录是空壳」：目录在，里面一个 red / green 都没有。
# 这个空目录只能在这里建——git 不跟踪空目录，放在样本的文件树里带不进 `gate.sh --staged`
# 的临时 worktree，那一格就判不出来（stage-selftest.sh 先把样本拷进临时目录再跑这个脚本）。
set -euo pipefail
mkdir -p .claude/gate.d/fixtures/51-hollow.sh
