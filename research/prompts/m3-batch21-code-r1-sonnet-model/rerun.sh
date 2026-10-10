#!/usr/bin/env bash
# m3-batch21-code-r1 正推腿的复跑脚本：在空工作目录里把冻结副本拷出来、核 22 份 sha256、编译、跑基线、逐行改坏变异表里本批点名的行再跑点名的测试。
# 可续跑：每次调用至多干 ~215 秒（前台命令不超过 240 秒），没干完退 99，原样再调一次接着干；全做完退 0；某一步判定不对退 1。
# 环境变量：SINGLEFS_REPO（仓路径，必填）；FROZEN_TREE（冻结副本，默认 /tmp/claude-1000/m3-batch21-code-r1/tree）；
#          WORK（工作目录，默认 /tmp/claude-1000/m3-batch21-code-r1-sonnet/work，复跑时给一个空目录）。
# 编译经 run-with-memory-cap.sh 6G，跑经 4G，线程上限 10。
set -uo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（仓路径）}"
HERE="$(cd "$(dirname "$0")" && pwd)"
export SINGLEFS_REPO
export FROZEN_TREE="${FROZEN_TREE:-/tmp/claude-1000/m3-batch21-code-r1/tree}"
export WORK="${WORK:-/tmp/claude-1000/m3-batch21-code-r1-sonnet/work}"
mkdir -p "$WORK"
exec python3 "$HERE/rerun.py"
