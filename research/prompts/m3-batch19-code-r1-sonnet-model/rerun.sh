#!/usr/bin/env bash
# 用法：bash rerun.sh <空的工作目录> <阶段>   阶段依次：setup base rows:2670-2676 rows:2677-2682 variants probes（每阶段单独调，各自不超过 240 秒）
# 需要先有冻结副本 /tmp/claude-1000/m3-batch19-code-r1/tree/。会话里已有别的日常 cargo 在跑时，设
# RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<至少 8 个字的理由>。
set -euo pipefail
exec python3 "$(dirname "$(realpath "$0")")/drive.py" "$@"
