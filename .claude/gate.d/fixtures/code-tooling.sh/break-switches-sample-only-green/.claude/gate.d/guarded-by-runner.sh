#!/usr/bin/env bash
# 样本阶段：读弄坏开关，经共用库跑格
source "$(dirname "$0")/lib/stage-cells.sh"
if [[ "${GUARDED_BY_RUNNER_BREAK:-}" == skip ]]; then exit 0; fi
stage_cells_run .
