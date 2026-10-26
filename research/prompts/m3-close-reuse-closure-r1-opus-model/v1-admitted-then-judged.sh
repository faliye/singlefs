#!/usr/bin/env bash
# V1 装置（字面通道）：真内存包装先排队、HostLookup 开着（假第二台，同 v1-tmpdir-world.sh），排队开始后 <放行秒数> 秒删掉挡路的假账，这一条随之放行；
# 放行后跑的「命令」照 Verus 的样子往 stderr 打一段证明错、往 stdout 打 verification-results JSON（这一份不跑 Verus）。
# 包装的 stdout、stderr 原样落盘，再交 v1-judge-diagnostics.py 照 verus-proof-check.py 的拼法（stderr + "\n" + stdout）判一行必红世界。
# 用法：v1-admitted-then-judged.sh <仓副本根> <名> <放行秒数> <假第二台 stderr 最后一行|none>
set -u
root="$1" name="$2" release_after="$3" peer_tail="$4"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
scratch="${ATTACK_OUT:-$here/out}/world-$name"
rm -rf -- "$scratch"; mkdir -p "$scratch/state/ledger" "$scratch/world/bin" "$scratch/world/peer/proc" "$scratch/world/peer/state/ledger"
world="$scratch/world"
if [[ "$peer_tail" == none ]]; then
cat > "$world/bin/ssh" <<'FAKE_SSH'
#!/usr/bin/env bash
while [[ "${1:-}" == -o ]]; do shift 2; done
shift
here="$HOST_LOOKUP_SELFTEST_WORLD/peer"
exec env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL RUN_WITH_MEMORY_CAP_SLICE=singlefs_host_lookup_selftest_absent.slice RUN_WITH_MEMORY_CAP_RESERVE=20G \
  HOST_RESOURCES_PROC_ROOT="$here/proc" SINGLEFS_DISPATCH_PROBE_MEMINFO="$here/proc/meminfo" RUN_WITH_MEMORY_CAP_STATE_DIR="$here/state" bash -c "$*"
FAKE_SSH
else
printf '#!/usr/bin/env bash\nprintf "%%s\\n" %q >&2\nexit 1\n' "$peer_tail" > "$world/bin/ssh"
fi
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
blocker="$scratch/state/ledger/singlefs-memory-cap-attack-blocker.json"
printf '{"unit": "singlefs-memory-cap-attack-blocker", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "250M", "key": "blocker", "admitted_at": "2026-10-22T00:00:00Z"}' \
  "$$" "${stat_fields[19]:-}" $((250 * 1024 * 1024)) > "$blocker"
( sleep "$release_after"; rm -f -- "$blocker" ) &
fake_verus='printf "%s\n" "error: assertion failed" "  --> src/kernel.rs:12:9" "error: aborting due to 1 previous error" >&2; printf "%s" "{\"verification-results\": {\"verified\": 3, \"errors\": 1, \"encountered-vir-error\": false, \"success\": false}}"'
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u RUN_WITH_MEMORY_CAP_HOST_LOOKUP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$scratch/alerts.jsonl" \
  HOST_LOOKUP_SELFTEST_WORLD="$world" PATH="$world/bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$world/multi-host.env" \
  RUN_WITH_MEMORY_CAP_KEY="attack-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30 \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" 200M bash -c "$fake_verus" \
  > "$scratch/stdout" 2> "$scratch/stderr"
status=$?
wait
echo "name=$name wrapper_status=$status"
python3 -B "$here/v1-judge-diagnostics.py" "$root" "$scratch/stdout" "$scratch/stderr" "$status"
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
