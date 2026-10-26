#!/usr/bin/env bash
# 第二台上在这一趟的树根跑：编两个测试目标，跑全量扫，再把修补一的用例文件在变异 0 到 7 下各跑一遍。
set -u
root="$PWD"
mkdir -p "$root/out"
cargo test --release --no-run -p singlefs-harness --test r3_attack_probe --test whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session > "$root/out/build.txt" 2>&1
echo "build_rc=$?" >> "$root/out/build.txt"
R3_PROBE_STARTS=normal,hand_made,hand_made_zero,generated_3000,generated_zero R3_PROBE_LENGTH=3 R3_PROBE_JUDGE_OLD=1 R3_PROBE_OUT="$root/out/sweep.txt" \
  cargo test --release -p singlefs-harness --test r3_attack_probe -- --nocapture --test-threads=1 > "$root/out/sweep-log.txt" 2>&1
echo "sweep_rc=$?" >> "$root/out/sweep-log.txt"
for mutant in 0 1 2 3 4 5 6 7; do
  R3_PROBE_MUTANT=$mutant cargo test --release -p singlefs-harness --test whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session > "$root/out/mutant-$mutant.txt" 2>&1
  echo "mutant_rc=$?" >> "$root/out/mutant-$mutant.txt"
done
