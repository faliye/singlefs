#!/usr/bin/env bash
# W1 装置：真内存包装（树里那一份）排队，查机器开着，第二台是假的（写法照 run-with-memory-cap.sh 自证 host_lookup_world：假 ssh、假 /proc，第二台可用 ATTACK_PEER_AVAILABLE_GIB，默认 43 GiB）。
# 本机状态：账上一笔挡路的假账要 250M、这一趟自己的 slice 总上限 256M、这一条要 200M（峰值表里没有，按上限记账）。
# <放行秒数> 是 none：一直挡着，等满 <等待上限秒> 退 252（finish 那一趟在前台查）；是数：那么多秒后删掉假账，这一条放行（begin 那一趟在后台查）。
# 用法：w1-queue-world.sh <树> <名> <等待上限秒> <放行秒数|none> [变量=值…]；输出目录 $ATTACK_OUT/world-<名>（stdout、stderr 原样留着）。
set -u
root="$1" name="$2" wait_seconds="$3" release_after="$4"; shift 4
scratch="${ATTACK_OUT:?要设 ATTACK_OUT}/world-$name"
rm -rf -- "$scratch"; mkdir -p "$scratch/state/ledger" "$scratch/world/bin" "$scratch/world/peer/proc" "$scratch/world/peer/state/ledger"
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
printf 'MemTotal: %d kB\nMemAvailable: %d kB\n' $((60 * 1024 * 1024)) $((${ATTACK_PEER_AVAILABLE_GIB:-43} * 1024 * 1024)) > "$world/peer/proc/meminfo"
printf 'cpu  0 0 0 800 0 0 0 0 0 0\ncpu0 0 0 0 800 0 0 0 0 0 0\n' > "$world/peer/proc/stat"
cp -- "$world/peer/proc/stat" "$world/peer/proc/stat.later"
printf '0.00 0.00 0.00 1/100 1\n' > "$world/peer/proc/loadavg"
printf '%s\n' PEER_SSH_HOST=selftest-peer-host-name PEER_REPOSITORY_DIRECTORY=/selftest/peer/home PEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin PEER_MEMORY_CAP=40G \
  QUIESCE_STOP_COMMAND=true QUIESCE_STOPPED_CHECK_COMMAND=true QUIESCE_START_COMMAND=true QUIESCE_STARTED_CHECK_COMMAND=true ENABLE_ACROSS_MACHINES=1 > "$world/multi-host.env"
slice="singlefs_opusattack_r2_$$_${RANDOM}.slice"
IFS= read -r stat_line < "/proc/$$/stat"
read -r -a stat_fields <<<"${stat_line##*) }"
blocker="$scratch/state/ledger/singlefs-memory-cap-attack-blocker.json"
printf '{"unit": "singlefs-memory-cap-attack-blocker", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "250M", "key": "blocker", "admitted_at": "2026-10-22T00:00:00Z"}' \
  "$$" "${stat_fields[19]:-}" $((250 * 1024 * 1024)) > "$blocker"
if [[ "$release_after" != none ]]; then
  ( sleep "$release_after"; rm -f -- "$blocker" ) &
fi
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u RUN_WITH_MEMORY_CAP_HOST_LOOKUP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$scratch/alerts.jsonl" \
  HOST_LOOKUP_SELFTEST_WORLD="$world" PATH="$world/bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$world/multi-host.env" \
  RUN_WITH_MEMORY_CAP_KEY="attack-r2-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS="$wait_seconds" "$@" \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" 200M bash -c ': > "$1"; echo "命令自己的 stderr" >&2' touch-marker "$scratch/ran" \
  > "$scratch/stdout" 2> "$scratch/stderr"
status=$?
wait
echo "$status" > "$scratch/status"
unprefixed=$(grep -cv '^run-with-memory-cap:' "$scratch/stderr")
echo "name=$name status=$status ran=$([[ -e $scratch/ran ]] && echo yes || echo no) traceback=$(grep -c '^Traceback' "$scratch/stderr") typeerror=$(grep -c '^TypeError' "$scratch/stderr") peer_fits_lines=$(grep -c '此刻放得下' "$scratch/stderr") stderr_lines_not_starting_with_prefix=$unprefixed"
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
