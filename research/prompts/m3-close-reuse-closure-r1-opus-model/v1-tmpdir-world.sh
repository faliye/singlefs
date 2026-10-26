#!/usr/bin/env bash
# V1 装置：真内存包装排队（同 v1-queue-world.sh 的挡路假账），第二台是假的（照 run-with-memory-cap.sh 自证 host_lookup_world 的写法：假 ssh、假 /proc），
# HostLookup 开着。TMPDIR 指到这一例自己的目录；<删的秒数> 不是 none 时，排队开始后那么多秒把 TMPDIR 删掉（本机状态：会话暂存目录被清掉、或盘满）。
# 用法：v1-tmpdir-world.sh <仓副本根> <名> <等待上限秒> <删的秒数|none> [变量=值…]
set -u
root="$1" name="$2" wait_seconds="$3" remove_after="$4"; shift 4
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
scratch="${ATTACK_OUT:-$here/out}/world-$name"
rm -rf -- "$scratch"; mkdir -p "$scratch/state/ledger" "$scratch/tmp" "$scratch/world/bin" "$scratch/world/peer/proc" "$scratch/world/peer/state/ledger"
world="$scratch/world"
cat > "$world/bin/ssh" <<'FAKE_SSH'
#!/usr/bin/env bash
while [[ "${1:-}" == -o ]]; do shift 2; done
shift
here="$HOST_LOOKUP_SELFTEST_WORLD/peer"
exec env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL RUN_WITH_MEMORY_CAP_SLICE=singlefs_host_lookup_selftest_absent.slice RUN_WITH_MEMORY_CAP_RESERVE=20G \
  HOST_RESOURCES_PROC_ROOT="$here/proc" SINGLEFS_DISPATCH_PROBE_MEMINFO="$here/proc/meminfo" RUN_WITH_MEMORY_CAP_STATE_DIR="$here/state" bash -c "$*"
FAKE_SSH
printf '#!/usr/bin/env bash\necho "假 nvidia-smi：没有卡" >&2\nexit 9\n' > "$world/bin/nvidia-smi"
chmod +x "$world/bin/ssh" "$world/bin/nvidia-smi"
printf 'MemTotal: %d kB\nMemAvailable: %d kB\n' $((60 * 1024 * 1024)) $((43 * 1024 * 1024)) > "$world/peer/proc/meminfo"
printf 'cpu  0 0 0 800 0 0 0 0 0 0\ncpu0 0 0 0 800 0 0 0 0 0 0\n' > "$world/peer/proc/stat"
cp -- "$world/peer/proc/stat" "$world/peer/proc/stat.later"
printf '0.00 0.00 0.00 1/100 1\n' > "$world/peer/proc/loadavg"
printf '%s\n' PEER_SSH_HOST=selftest-peer-host-name PEER_REPOSITORY_DIRECTORY=/selftest/peer/home PEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin PEER_MEMORY_CAP=40G \
  QUIESCE_STOP_COMMAND=true QUIESCE_STOPPED_CHECK_COMMAND=true QUIESCE_START_COMMAND=true QUIESCE_STARTED_CHECK_COMMAND=true ENABLE_ACROSS_MACHINES=1 > "$world/multi-host.env"
slice="singlefs_opusattack_$$_${RANDOM}.slice"
IFS= read -r stat_line < "/proc/$$/stat"
read -r -a stat_fields <<<"${stat_line##*) }"
printf '{"unit": "singlefs-memory-cap-attack-blocker", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "250M", "key": "blocker", "admitted_at": "2026-10-22T00:00:00Z"}' \
  "$$" "${stat_fields[19]:-}" $((250 * 1024 * 1024)) > "$scratch/state/ledger/singlefs-memory-cap-attack-blocker.json"
if [[ "$remove_after" != none ]]; then
  ( sleep "$remove_after"; rm -rf -- "$scratch/tmp" ) &
fi
started=$SECONDS
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u RUN_WITH_MEMORY_CAP_HOST_LOOKUP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$scratch/alerts.jsonl" \
  HOST_LOOKUP_SELFTEST_WORLD="$world" PATH="$world/bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$world/multi-host.env" TMPDIR="$scratch/tmp" \
  RUN_WITH_MEMORY_CAP_KEY="attack-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS="$wait_seconds" "$@" \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" 200M bash -c ': > "$1"' touch-marker "$scratch/ran" \
  > "$scratch/stdout" 2> "$scratch/stderr"
status=$?
wait
echo "name=$name remove_after=$remove_after status=$status seconds=$((SECONDS - started)) ran=$([[ -e $scratch/ran ]] && echo yes || echo no)"
grep -E "Error|排队那一步退|内存不够排不上|没查到别的机器|此刻放得下|^  ✗" "$scratch/stderr" | cut -c1-180 | head -10
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
