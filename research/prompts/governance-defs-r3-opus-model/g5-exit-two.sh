#!/usr/bin/env bash
# G5：共用约束「2 是包装的用法写错，改写法再跑」与包装的真实退出码表。
# 包装自己的用法错退 2（量：不给参数跑一次，不起 scope）；被包的命令自己退 2 时包装原样传出（读源码，这个容器里 scope 起不来、跑不到那一步）。
# 用法：bash g5-exit-two.sh <仓根>
set -uo pipefail
real="$1"; cd "$real" || exit 1
bash research/scripts/run-with-memory-cap.sh >/dev/null 2>&1; echo "包装不给参数：退 $?"
echo "包装文件头与传出那一行："
grep -nF '#   0–249  那条命令自己的退出码' research/scripts/run-with-memory-cap.sh
grep -nF '#   2      用法错' research/scripts/run-with-memory-cap.sh
grep -nF "exit \"\$command_exit\"'" research/scripts/run-with-memory-cap.sh
echo "执行员第 5 步经包装跑的 replay.sh 自己退 2 的地方："
grep -nE 'exit 2' research/scripts/replay.sh
echo "共用约束那一句："
grep -noF '2 是包装的用法写错，改写法再跑' .claude/agent-common.md
