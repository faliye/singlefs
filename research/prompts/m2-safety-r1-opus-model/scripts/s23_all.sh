#!/usr/bin/env bash
# S2 / S3 各候选（none 是判别子自证用的对照臂：一条都不补）：第四轮攻方的 Z21-A / B / B2 / C 原样用例，加这一轮的随机历史判别子（两个种子基）。线程上限 8。
set -euo pipefail
BIN=$1; L=$2; mkdir -p "$L"
export TMPDIR=/tmp/claude-1000/m2-safety-r1-opus/tmpimg; mkdir -p "$TMPDIR"
run_variant() {
  local name=$1; shift
  env "$@" nice -n 19 "$BIN" --nocapture --test-threads 4 \
    z21_a_rollback_to_the_generation_zero_root_crashing_in_the_post_window_is_never_restored \
    z21_b_rollback_rows_resurrect_deleted_entries_when_one_ring_slot_goes_bad \
    z21_b2_after_the_refusal_every_writable_path_is_refused_while_the_slot_stays_bad \
    z21_c_random_rollback_series_with_crashes_in_the_post_window_restore_exactly_the_intended_entries \
    > "$L/s23-$name-z21.log" 2>&1 || true
  env "$@" S_OPUS_BASE=510000 nice -n 19 "$BIN" --exact s23_random_series_oracle --nocapture > "$L/s23-$name-oracle-510000.log" 2>&1 &
  env "$@" S_OPUS_BASE=520000 nice -n 19 "$BIN" --exact s23_random_series_oracle --nocapture > "$L/s23-$name-oracle-520000.log" 2>&1 &
  env "$@" S_OPUS_BASE=530000 S_OPUS_ZERO_SHARE=1 nice -n 19 "$BIN" --exact s23_random_series_oracle --nocapture > "$L/s23-$name-oracle-zero-530000.log" 2>&1 &
  env "$@" S_OPUS_BASE=540000 S_OPUS_DEGRADE_SHARE=30 nice -n 19 "$BIN" --exact s23_random_series_oracle --nocapture > "$L/s23-$name-oracle-degrade-540000.log" 2>&1 &
  wait
  echo "done $name"
}
run_variant none S_OPUS_S2=none
run_variant today S_OPUS_NONE=1
run_variant narrow S_OPUS_S2=narrow
run_variant clearflag S_OPUS_S2=clearflag
run_variant infer S_OPUS_S3=infer
run_variant narrow-infer S_OPUS_S2=narrow S_OPUS_S3=infer
run_variant rowzero S_OPUS_S3=row-zero
run_variant narrow-rowzero S_OPUS_S2=narrow S_OPUS_S3=row-zero
run_variant refuse S_OPUS_S3=refuse
run_variant pre S_OPUS_S3=pre
run_variant clearflag-rowzero S_OPUS_S2=clearflag S_OPUS_S3=row-zero
