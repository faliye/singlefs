#!/usr/bin/env bash
# V2 装置：包装的 need_of → predicted_bytes 按文件路径 exec_module 整份 host-dispatch.py，它模块级 import lib_long_run（.claude/hooks）。
# 总上限 9G、这一条上限 10G、峰值表里没有它：预测算得出就按预测 8G 记账、放得下、起跑；预测算不出就按上限 10G 记账、比总上限大、当场退 252。
# 用法：v2-import-side-effect.sh <仓副本根> <名>
set -u
root="$1" name="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
scratch="${ATTACK_OUT:-$here/out}/world-$name"
rm -rf -- "$scratch"; mkdir -p "$scratch/state"
slice="singlefs_opusattack_$$_${RANDOM}.slice"
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=9G RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$scratch/alerts.jsonl" RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off \
  RUN_WITH_MEMORY_CAP_KEY="attack-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=5 \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" 10G bash -c ': > "$1"' touch-marker "$scratch/ran" > "$scratch/stdout" 2> "$scratch/stderr"
status=$?
echo "name=$name status=$status ran=$([[ -e $scratch/ran ]] && echo yes || echo no)"
grep -E '峰值表里没有这条|✗' "$scratch/stderr" | cut -c1-230 | head -4
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
