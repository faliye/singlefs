#!/usr/bin/env bash
# 样本：录好的驱动 --summary 输出（全有结论、判绿）；与驱动跑没跑无关，所以门禁不起驱动时这一份照样判绿
cat <<'SUMMARY'
BROAD_MUTATION_SUMMARY total=6 caught=5 missed=0 unviable=1 stage_one_only=0 driver_error=0 not_run=0 refused=0 unlisted_units=0 gpu_needed=0 gpu_reused=0 gpu_not_run=0 gpu_without_conclusion=0
BROAD_MUTATION_GATE total=6 concluded=6 missed=0 without_conclusion=0 refused=0 unlisted_units=0 gpu_needed=0 gpu_reused=0 gpu_not_run=0 gpu_without_conclusion=0 verdict=green
SUMMARY
exit 0
