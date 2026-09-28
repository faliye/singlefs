#!/usr/bin/env bash
# 照 research/scripts/replay.sh 的 driver_e142 两条命令原样跑，按它的次序拼：模型 stdout 在前、导出在后。
# 用法：driver.sh <标签> <产物路径>；模型 stderr（准入）存 <草稿>/run/<标签>.model-stderr.txt
set -u
label="$1"; product="$2"
scratch=/tmp/claude-1000/e142-r19-runner/run
repo=<仓根>
cap=8G
snapshot="$scratch/$label.dump.txt"
[ -e "$product" ] && { echo "产物已存在：$product" >&2; exit 9; }
(cd "$repo" && nice -n 19 bash research/scripts/run-with-memory-cap.sh "$cap" bash research/scripts/capped.sh 4 cargo run -q -p singlefs-harness --bin e142_first_transaction_write_dump) > "$snapshot" 2> "$scratch/$label.dump-stderr.txt"
dump_exit=$?
echo "dump_exit=$dump_exit" > "$scratch/$label.exits.txt"
[ "$dump_exit" = 0 ] || exit 3
(cd "$repo/research" && nice -n 19 bash scripts/run-with-memory-cap.sh "$cap" ./target/release/e142-first-txn-dry-run "$snapshot" results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out) > "$scratch/$label.model-stdout.txt" 2> "$scratch/$label.model-stderr.txt"
model_exit=$?
echo "model_exit=$model_exit" >> "$scratch/$label.exits.txt"
[ "$model_exit" = 0 ] || exit 4
set -o noclobber
cat "$scratch/$label.model-stdout.txt" "$snapshot" > "$product"
echo "product_written=$product" >> "$scratch/$label.exits.txt"
