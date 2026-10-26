#!/usr/bin/env bash
# 样本阶段：读弄坏开关，自己跑格，cd 到根之后调那一判
source "$(dirname "$0")/lib/stage-cells.sh"
cd "${1:-.}" || exit 2
stage_cells_refuse_unsampled_break_switches "$PWD"
if [[ "${GUARDED_BY_OWN_CALL_BREAK:-}" == skip ]]; then exit 0; fi
