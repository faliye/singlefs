#!/usr/bin/env bash
# 样本：cd 之前取好阶段自己的位置，cd 之后用变量
set -uo pipefail
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ROOT="${1:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
source "$STAGE_REPOSITORY/scripts/gate/stage-run-or-skip.sh" || exit 1
