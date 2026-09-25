#!/usr/bin/env bash
# Z12 容量扫描：每个组合一个进程，4 个并行、每个 2 线程（合计 8）。
set -u
BIN=$(ls -t /tmp/claude-1000/m2-final-code-r2-opus/target/debug/deps/r2_opus_z12-* | grep -v '\.d$' | head -1)
OUT=/tmp/claude-1000/m2-final-code-r2-opus/logs/z12
combos=()
for al in 1 2 3 5; do for ai in 2 3; do for ml in 1 2 3 4; do for mi in 2 3; do combos+=("$al,$ai,$ml,$mi"); done; done; done; done
run_one() {
  caps=$1
  name=$(echo "$caps" | tr ',' '_')
  SINGLEFS_R2_OPUS_CAPS=$caps R2_OPUS_ROUNDS=7 R2_OPUS_INODES_EVERY=2 RUST_TEST_THREADS=2 nice -n 19 "$BIN" --nocapture --test-threads 2 > "$OUT/caps_$name.log" 2>&1
  echo "caps=$caps rc=$?"
}
export -f run_one; export BIN OUT
printf '%s\n' "${combos[@]}" | xargs -P 4 -I{} bash -c 'run_one {}'
for named in 1 2 3; do
  SINGLEFS_R2_OPUS_NAMED=$named R2_OPUS_ROUNDS=8 RUST_TEST_THREADS=2 nice -n 19 "$BIN" --nocapture --test-threads 2 > "$OUT/named_$named.log" 2>&1; echo "named=$named rc=$?"
  SINGLEFS_R2_OPUS_NAMED=$named SINGLEFS_R2_OPUS_CAPS=2,2,2,2 R2_OPUS_ROUNDS=8 RUST_TEST_THREADS=2 nice -n 19 "$BIN" --nocapture --test-threads 2 > "$OUT/named_${named}_caps2222.log" 2>&1; echo "named=$named caps=2222 rc=$?"
done
echo done
