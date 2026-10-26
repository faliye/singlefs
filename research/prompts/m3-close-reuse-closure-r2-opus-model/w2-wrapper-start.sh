#!/usr/bin/env bash
# W2 装置：树里那一份真内存包装起一条 true（这一趟自己的 slice、查机器关着、不排队），报包装退几、命令跑没跑。
# 用法：w2-wrapper-start.sh <树> <名> <输出目录>
set -u
root="$1" name="$2" out="$3/start-$name"
rm -rf -- "$out"; mkdir -p "$out/state/ledger"
slice="singlefs_opusattack_r2_start_$$_${RANDOM}.slice"
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=1G RUN_WITH_MEMORY_CAP_STATE_DIR="$out/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$out/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$out/alerts.jsonl" RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off \
  RUN_WITH_MEMORY_CAP_KEY="attack-r2-start-$name" \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" 64M bash -c ': > "$1"' touch-marker "$out/ran" > "$out/stdout" 2> "$out/stderr"
status=$?
echo "name=wrapper_start tree=$name status=$status ran=$([[ -e $out/ran ]] && echo yes || echo no) last_error=$(grep -m1 -E 'Error' "$out/stderr" | cut -c1-120)"
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
