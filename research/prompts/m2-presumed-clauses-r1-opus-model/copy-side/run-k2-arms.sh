#!/usr/bin/env bash
# 副本上 K2 七条臂并行跑：每条臂一个进程、各写各的日志与退出码。
set -u
binary=/tmp/claude-1000/presumed-r1-opus/target/release/deps/presumed_r1_opus_k2-6b1935bd79bdd91e
out=/tmp/claude-1000/presumed-r1-opus/k2-arms
mkdir -p "$out"
export TMPDIR=/tmp/claude-1000/presumed-r1-opus/tmp
declare -a names=(A1-today A2-any A3-any-cap0 A4-any-cap1 A5-any-jump A6-cap0 A7-jump)
declare -a envs=("" "PRESUMED_R1_NAMED_ANY=1" "PRESUMED_R1_NAMED_ANY=1 PRESUMED_R1_WARM_UP_CAP=0" "PRESUMED_R1_NAMED_ANY=1 PRESUMED_R1_WARM_UP_CAP=1" "PRESUMED_R1_NAMED_ANY=1 PRESUMED_R1_ROW_TXG_JUMP=1" "PRESUMED_R1_WARM_UP_CAP=0" "PRESUMED_R1_ROW_TXG_JUMP=1")
pids=()
for index in "${!names[@]}"; do
  name=${names[$index]}
  rm -f "${out:?}/$name.log" "${out:?}/$name.exit"
  (
    # shellcheck disable=SC2086
    env ${envs[$index]} nice -n 19 "$binary" --nocapture --test-threads=1 > "$out/$name.log" 2>&1
    echo $? > "$out/$name.exit"
  ) &
  pids+=($!)
done
for pid in "${pids[@]}"; do
  wait "$pid"
done
for name in "${names[@]}"; do
  echo "$name exit=$(cat "$out/$name.exit")"
  grep -E 'K2-SUMMARY|K2-CHAIN|test result' "$out/$name.log"
done
