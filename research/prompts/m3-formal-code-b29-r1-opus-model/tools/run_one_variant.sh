#!/usr/bin/env bash
# 攻方副本上打一个变体（baseline 不打）、逐个跑点名的 harness 测试目标、还原 src。
# 用法：bash run_one_variant.sh <副本树> <target 目录> <日志目录> <变体> <测试目标名>...
set -u
TREE=$1; TARGET=$2; LOGS=$3; VARIANT=$4; shift 4
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$LOGS"
BACKUP="$LOGS/src.bak.$VARIANT"
rm -rf "$BACKUP"; cp -a "$TREE/crates/singlefs-harness/src" "$BACKUP"
if [ "$VARIANT" != baseline ]; then
  python3 "$HERE/apply_variant.py" "$TREE" "$VARIANT" || { echo "name=variant variant=$VARIANT result=anchor_failed"; rm -rf "$BACKUP"; exit 3; }
fi
find "$TREE/crates/singlefs-harness/src" -name "*.rs" -exec touch {} +  # cp -a 还原的旧 mtime 会让 cargo 复用上一个变体编出的库：每次跑之前把源码时间戳刷新
for target in "$@"; do
  log="$LOGS/$VARIANT.$target.log"
  ( cd "$TREE" && CARGO_TARGET_DIR=$TARGET nice -n 19 cargo test --offline -p singlefs-harness --test "$target" -- --nocapture --test-threads=3 > "$log" 2>&1 )
  rc=$?
  echo "name=variant variant=$VARIANT target=$target rc=$rc $(grep -E '^test result' "$log" | tail -1)"
  grep -E '^test .* (ok|FAILED)$' "$log" | sed "s/^/  variant=$VARIANT /"
done
rm -rf "$TREE/crates/singlefs-harness/src"; cp -a "$BACKUP" "$TREE/crates/singlefs-harness/src"; rm -rf "$BACKUP"
find "$TREE/crates/singlefs-harness/src" -name "*.rs" -exec touch {} +  # cp -a 还原的旧 mtime 会让 cargo 复用上一个变体编出的库：每次跑之前把源码时间戳刷新
