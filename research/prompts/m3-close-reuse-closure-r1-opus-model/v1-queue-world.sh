#!/usr/bin/env bash
# V1 装置：在仓副本里起真内存包装，造「排队」那一种本机状态（账上一笔挡路的假账要 250M、总上限 256M、这一条要 200M），
# 再用 systemd-run --user --scope -p TasksMax=<N> 限这一条包装能开的任务数（进程与线程都算），看包装退几。
# 用法：v1-queue-world.sh <仓副本根> <名> <TasksMax 或 infinity> <等待上限秒> [变量=值…]
set -u
root="$1" name="$2" tasks="$3" wait_seconds="$4"; shift 4
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
scratch="${ATTACK_OUT:-$here/out}/world-$name"
rm -rf -- "$scratch"; mkdir -p "$scratch/state/ledger"
slice="singlefs_opusattack_$$_${RANDOM}.slice"
IFS= read -r stat_line < "/proc/$$/stat"
read -r -a stat_fields <<<"${stat_line##*) }"
printf '{"unit": "singlefs-memory-cap-attack-blocker", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "250M", "key": "blocker", "admitted_at": "2026-10-22T00:00:00Z"}' \
  "$$" "${stat_fields[19]:-}" $((250 * 1024 * 1024)) > "$scratch/state/ledger/singlefs-memory-cap-attack-blocker.json"
started=$SECONDS
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$scratch/alerts.jsonl" \
  RUN_WITH_MEMORY_CAP_KEY="attack-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS="$wait_seconds" "$@" \
  systemd-run --user --scope --quiet -p TasksMax="$tasks" \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" 200M bash -c ': > "$1"' touch-marker "$scratch/ran" \
  > "$scratch/stdout" 2> "$scratch/stderr"
status=$?
echo "name=$name tasks=$tasks status=$status seconds=$((SECONDS - started)) ran=$([[ -e $scratch/ran ]] && echo yes || echo no)"
grep -E "can't start new thread|RuntimeError|排队那一步退|内存不够排不上|没查到别的机器|✗" "$scratch/stderr" | cut -c1-200 | head -8
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
