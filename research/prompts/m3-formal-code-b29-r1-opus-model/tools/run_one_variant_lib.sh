#!/usr/bin/env bash
# 同 run_one_variant.sh，跑的是 singlefs-harness 的 --lib 单测（全部）。用法：bash run_one_variant_lib.sh <副本树> <target 目录> <日志目录> <变体>
set -u
TREE=$1; TARGET=$2; LOGS=$3; VARIANT=$4
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$LOGS"
BACKUP="$LOGS/src.bak.$VARIANT"
rm -rf "$BACKUP"; cp -a "$TREE/crates/singlefs-harness/src" "$BACKUP"
if [ "$VARIANT" != baseline ]; then
  python3 "$HERE/apply_variant.py" "$TREE" "$VARIANT" || { echo "name=variant variant=$VARIANT result=anchor_failed"; rm -rf "$BACKUP"; exit 3; }
fi
find "$TREE/crates/singlefs-harness/src" -name "*.rs" -exec touch {} +  # cp -a 还原的旧 mtime 会让 cargo 复用上一个变体编出的库：每次跑之前把源码时间戳刷新
log="$LOGS/$VARIANT.lib.log"
( cd "$TREE" && CARGO_TARGET_DIR=$TARGET nice -n 19 cargo test --offline -p singlefs-harness --lib > "$log" 2>&1 )
rc=$?
echo "name=variant variant=$VARIANT target=lib rc=$rc $(grep -E '^test result' "$log" | tail -1)"
grep -E '^test .* FAILED$' "$log" | head -5
rm -rf "$TREE/crates/singlefs-harness/src"; cp -a "$BACKUP" "$TREE/crates/singlefs-harness/src"; rm -rf "$BACKUP"
find "$TREE/crates/singlefs-harness/src" -name "*.rs" -exec touch {} +  # cp -a 还原的旧 mtime 会让 cargo 复用上一个变体编出的库：每次跑之前把源码时间戳刷新
