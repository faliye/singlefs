#!/usr/bin/env bash
# 复跑 m3-blanks-r1 正推腿的六个小模型。每个模型经内存包装 8G、线程上限 15 跑；输出落到本目录 <名>.out。
# 用法：bash research/prompts/m3-blanks-r1-sonnet-model/rerun.sh [输出目录]   （默认本目录；核实时指到草稿目录再 diff）
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${1:-$here}"
root="$(cd "$here/../../.." && pwd)"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示指定上限 8G，模型目录小模型"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-1500}"
for m in b1_kind_vs_id b2_container_numbers b4_queue_and_capacity b5_tombstone_growth b6_ring_window b7_tombstone_prepay b8_publish_counts; do
    nice -n 19 bash "$root/research/scripts/capped.sh" 15 bash "$root/research/scripts/run-with-memory-cap.sh" 8G \
        python3 "$here/$m.py" 2>&1 | grep -v '^run-with-memory-cap:' > "$out/$m.out"
done
