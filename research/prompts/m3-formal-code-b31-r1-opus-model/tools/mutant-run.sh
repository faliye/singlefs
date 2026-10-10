#!/bin/bash
# 第 31 批代码轮第一轮攻方：在仓副本上打一个坏实现（tools/mutants.py），编四个 harness 档测试目标，并行跑五条判据（第五条只打数、不断言），撤回坏实现。
# 用法：mutant-run.sh <草稿目录> <坏实现名|baseline>；每件退出码写 <草稿目录>/rc/b<坏实现名>-<件号>.rc，日志写 <草稿目录>/mutants/<坏实现名>/。
# 整条经 run-with-memory-cap.sh 起（见 tools/rerun.sh）；线程由外层 capped.sh 定。
set -u
D="$1"; name="$2"
TOOLS="$(cd "$(dirname "$0")" && pwd)"
COPY="$D/copy"
LOG="$D/mutants/$name"; mkdir -p "$LOG" "$D/rc"; rm -f "$D/rc/b$name"-*.rc
if [[ "$name" != baseline ]]; then
  python3 "$TOOLS/mutants.py" "$COPY" apply "$name" > "$LOG/apply.out" 2>&1 || { cat "$LOG/apply.out"; python3 "$TOOLS/mutants.py" "$COPY" restore; exit 2; }
fi
export CARGO_TARGET_DIR="$D/target"
( cd "$COPY" && cargo build --release --offline -p singlefs-harness \
    --test root_ring_turn_in_the_make_filesystem_session --test overwrite_in_one_instance \
    --test random_histories --test b31_opus_n3_reclaim_predicate_and_ring_table \
    --message-format=json 2> "$LOG/build.err" ) > "$LOG/build.json"
build_rc=$?
if [[ "$name" != baseline ]]; then python3 "$TOOLS/mutants.py" "$COPY" restore > "$LOG/restore.out" 2>&1; fi
if [[ $build_rc -ne 0 ]]; then echo "$name build_rc=$build_rc"; tail -20 "$LOG/build.err"; exit 1; fi
binary_of() { python3 -c '
import json,sys
for line in open(sys.argv[1]):
    try: m=json.loads(line)
    except Exception: continue
    if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["name"]==sys.argv[2]: print(m["executable"])
' "$LOG/build.json" "$1"; }
cd "$COPY/crates/singlefs-harness"
{ "$(binary_of root_ring_turn_in_the_make_filesystem_session)" > "$LOG/1-root-ring-turn.log" 2>&1; echo "$?" > "$D/rc/b$name-1.rc"; } &
{ "$(binary_of overwrite_in_one_instance)" --exact allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records > "$LOG/2-overwrite-812.log" 2>&1; echo "$?" > "$D/rc/b$name-2.rc"; } &
{ "$(binary_of random_histories)" --ignored --exact turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_runs_to_the_end_with_the_checker_green_after_every_step > "$LOG/3-random-mkfs-turn.log" 2>&1; echo "$?" > "$D/rc/b$name-3.rc"; } &
{ "$(binary_of b31_opus_n3_reclaim_predicate_and_ring_table)" --exact scenario_a_the_session_of_the_812_record_case --nocapture > "$LOG/4-p3-scenario-a.log" 2>&1; echo "$?" > "$D/rc/b$name-4.rc"; } &
{ "$(binary_of b31_opus_n3_reclaim_predicate_and_ring_table)" --exact scenario_c_unit_writes_persisted_before_the_first_barrier_of_each_overwrite --nocapture > "$LOG/5-p3-scenario-c.log" 2>&1; echo "$?" > "$D/rc/b$name-5.rc"; } &
wait
count=$(ls "$D/rc/b$name"-*.rc | wc -l)
echo "$name rcfiles=$count/5"
[[ $count -eq 5 ]] || exit 1
for piece in 1 2 3 4 5; do echo "$name piece $piece rc=$(cat "$D/rc/b$name-$piece.rc")"; done
