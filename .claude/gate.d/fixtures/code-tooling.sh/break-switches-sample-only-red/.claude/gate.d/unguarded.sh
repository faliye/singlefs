#!/usr/bin/env bash
# 样本阶段：读弄坏开关，不经 stage_cells_run 跑格，也没调那一判
if [[ "${SAMPLE_STAGE_BREAK:-}" == skip ]]; then exit 0; fi
echo "判完"
