#!/usr/bin/env bash
# W1 装置：树里那一份真内存包装，各在这一趟自己的 slice 里撞一次限时（退 253）与一次内存上限（退 250），stderr 原样落盘，给 w1-strip-judges.py 当广谱变异升档那两趟的日志。
# 限时两趟：限 1 秒跑 sleep 3（原来那趟）、限 2 秒跑 sleep 4（升档那趟）；上限两趟：64M 与 128M 里各分配 512 MiB。查机器关着（这里只要包装撞顶那一句）。
# 用法：w1-wrapper-hits.sh <树> <输出目录>
set -u
root="$1" out="$2"
mkdir -p "$out/state/ledger"
slice="singlefs_opusattack_r2_hits_$$_${RANDOM}.slice"
run() {   # run <名> <上限> [变量=值…] -- <命令…>
  local name="$1" cap="$2"; shift 2
  local settings=()
  while [[ "$1" != -- ]]; do settings+=("$1"); shift; done
  shift
  env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP \
    RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=1G RUN_WITH_MEMORY_CAP_STATE_DIR="$out/state" \
    RUN_WITH_MEMORY_CAP_PEAKS="$out/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$out/alerts.jsonl" RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off \
    RUN_WITH_MEMORY_CAP_KEY="attack-r2-$name" "${settings[@]}" \
    bash "$root/scripts/hosts/run-with-memory-cap.sh" "$cap" "$@" > "$out/$name.log" 2>&1
  echo "name=wrapper_hit case=$name status=$?"
}
run time-first 64M RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 -- sleep 3
run time-escalated 64M RUN_WITH_MEMORY_CAP_TIME_LIMIT=2 -- sleep 4
run memory-first 64M -- python3 -c 'block = bytearray(512 << 20); print(len(block))'
run memory-escalated 128M -- python3 -c 'block = bytearray(512 << 20); print(len(block))'
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
