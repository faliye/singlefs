#!/usr/bin/env bash
# 第二台上在这一趟的树根跑：一批一个单元的全量扫（删除与截断的中间版都核），生成器起点的整条 I-1.13 说明，变异 6 在别的几份删除用例上红不红。
set -u
root="$PWD"
mkdir -p "$root/out"
targets="writes_and_unlinks_after_the_first_file_was_deleted_in_histories_judged_by_the_model unlink_of_files_other_than_the_first unlink_mount_continuation_and_the_emptied_version unlink_single_file_publishes unlink_in_histories_judged_by_the_model"
target_arguments=""
for target in $targets; do target_arguments="$target_arguments --test $target"; done
cargo test --release --no-run -p singlefs-harness --test r3_attack_probe $target_arguments > "$root/out/build.txt" 2>&1
echo "build_rc=$?" >> "$root/out/build.txt"
R3_PROBE_BATCH_ONE=1 R3_PROBE_STARTS=normal,hand_made_zero,generated_zero,generated_3000 R3_PROBE_LENGTH=3 R3_PROBE_JUDGE_OLD=1 R3_PROBE_OUT="$root/out/sweep-batch-one.txt" \
  cargo test --release -p singlefs-harness --test r3_attack_probe -- --nocapture --test-threads=1 > "$root/out/sweep-batch-one-log.txt" 2>&1
echo "sweep_rc=$?" >> "$root/out/sweep-batch-one-log.txt"
R3_PROBE_DETAIL_CHARS=3000 R3_PROBE_STARTS=generated_zero,generated_3000 R3_PROBE_LENGTH=0 R3_PROBE_OUT="$root/out/detail.txt" \
  cargo test --release -p singlefs-harness --test r3_attack_probe -- --nocapture --test-threads=1 > "$root/out/detail-log.txt" 2>&1
echo "detail_rc=$?" >> "$root/out/detail-log.txt"
for target in $targets; do
  R3_PROBE_MUTANT=6 cargo test --release -p singlefs-harness --test "$target" > "$root/out/mutant-6-$target.txt" 2>&1
  echo "mutant_rc=$?" >> "$root/out/mutant-6-$target.txt"
done
