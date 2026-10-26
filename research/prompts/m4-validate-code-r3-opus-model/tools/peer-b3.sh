#!/usr/bin/env bash
# 第二台上在这一趟的树根跑：清零起点的全量扫（对照旧规则），再在七个起点上各拿变异 6 作对照扫一遍。
set -u
root="$PWD"
mkdir -p "$root/out"
cargo test --release --no-run -p singlefs-harness --test r3_attack_probe > "$root/out/build.txt" 2>&1
echo "build_rc=$?" >> "$root/out/build.txt"
R3_PROBE_STARTS=scrubbed_3000,scrubbed_zero R3_PROBE_LENGTH=3 R3_PROBE_JUDGE_OLD=1 R3_PROBE_OUT="$root/out/sweep-scrubbed.txt" \
  cargo test --release -p singlefs-harness --test r3_attack_probe -- --nocapture --test-threads=1 > "$root/out/sweep-scrubbed-log.txt" 2>&1
echo "sweep_rc=$?" >> "$root/out/sweep-scrubbed-log.txt"
for mutant in 6; do
  R3_PROBE_COMPARE_MUTANT=$mutant R3_PROBE_STARTS=normal,hand_made_zero,scrubbed_zero R3_PROBE_LENGTH=3 R3_PROBE_JUDGE_OLD=1 R3_PROBE_OUT="$root/out/sweep-mutant-$mutant.txt" \
    cargo test --release -p singlefs-harness --test r3_attack_probe -- --nocapture --test-threads=1 > "$root/out/sweep-mutant-$mutant-log.txt" 2>&1
  echo "sweep_rc=$?" >> "$root/out/sweep-mutant-$mutant-log.txt"
done
