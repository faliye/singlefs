#!/usr/bin/env bash
# 样本：在 cd 之前取好位置；cd 之后注释里写 dirname "$0" 与 echo 出路文字里提到它的不算
set -uo pipefail
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ROOT="${1:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
# 不要在这里再写 dirname "$0"
echo "     → 怎么办：在 cd 之前取位置，别在 cd 之后写 dirname \"$0\""
source "$STAGE_REPOSITORY/scripts/gate/stage-run-or-skip.sh" || exit 1
