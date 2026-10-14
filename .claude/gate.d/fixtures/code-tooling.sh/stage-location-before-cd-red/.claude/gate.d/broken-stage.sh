#!/usr/bin/env bash
# 样本：cd 到项目根之后才用 dirname "$0" 找旁边的共用脚本
set -uo pipefail
ROOT="${1:-$(pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
source "$(cd "$(dirname "$0")/../.." && pwd)/scripts/gate/stage-run-or-skip.sh" || exit 1
