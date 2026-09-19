#!/usr/bin/env bash
# 从补过的 54 号里抽出两个判线程数的函数，拿合成日志跑几种情形：该过的过、该红的红。
set -uo pipefail
gate=/tmp/claude-1000/m2-layer0-parallel/copy/.claude/gate.d/54-layer0-replay.sh
eval "$(sed -n '/^worker_threads_of_full_run() {/,/^}/p' "$gate")"
eval "$(sed -n '/^worker_threads_are_acceptable() {/,/^}/p' "$gate")"
work="$(mktemp -d)"
line='LAYER0 states=262165 closed_form=262165 violations=0 exhaustive=true'
printf '%s\n' 'LAYER0_PARALLEL_FINISHED states=22 slices=2 worker_threads=2 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=1.0' \
  'LAYER0_PARALLEL_FINISHED states=262165 slices=512 worker_threads=32 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=35.8' >"$work/thirty_two.log"
printf '%s\n' 'LAYER0_PARALLEL_FINISHED states=22 slices=2 worker_threads=2 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=1.0' \
  'LAYER0_PARALLEL_FINISHED states=262165 slices=512 worker_threads=1 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=200.0' >"$work/one.log"
printf '%s\n' 'LAYER0_PARALLEL_FINISHED states=22 slices=2 worker_threads=2 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=1.0' >"$work/missing.log"
report() { echo "情形：$1 → 取到的线程数「$2」、判定 $3"; }
for case_name in thirty_two one missing; do
  for origin in "没设，取本机核数" "显式设的"; do
    for requested in 32 1; do
      machine_cores=32; threads_origin="$origin"; SINGLEFS_LAYER0_THREADS="$requested"
      threads="$(worker_threads_of_full_run "$work/$case_name.log" "$line")"
      if worker_threads_are_acceptable "第一条流" "$threads" >"$work/out" 2>&1; then verdict=过; else verdict=红; fi
      report "$case_name / $origin / SINGLEFS_LAYER0_THREADS=$requested" "$threads" "$verdict"
    done
  done
done
machine_cores=1; threads_origin="没设，取本机核数"; SINGLEFS_LAYER0_THREADS=1
threads="$(worker_threads_of_full_run "$work/one.log" "$line")"
if worker_threads_are_acceptable "第一条流" "$threads" >/dev/null 2>&1; then verdict=过; else verdict=红; fi
report "one / 本机 1 核 / 没设" "$threads" "$verdict"
line_b='LAYER0B states=2104413 closed_form=2104413 exhaustive=true'
printf '%s\n' 'LAYER0_PARALLEL_FINISHED states=108 slices=7 worker_threads=7 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=6.0' \
  'LAYER0_PARALLEL_FINISHED states=2104413 slices=512 worker_threads=32 configured_worker_threads=32 worker_threads_source=environment_variable elapsed_seconds=900.0' >"$work/b.log"
echo "第二条流取到的线程数：$(worker_threads_of_full_run "$work/b.log" "$line_b")"
rm -rf "$work"
