#!/usr/bin/env bash
# 攻方腿模型的副本搭建（m3-leftover-code-r1-opus）：把冻结副本拷到 $OPUS_DIR/work，往 mount.rs 末尾加探针，放进三份探针用例，
# 再把会被变异改的四份文件存一份到 $OPUS_DIR/clean（runner.py / apply.py 每次从这里恢复）。
set -euo pipefail
FROZEN="${FROZEN:-/tmp/claude-1000/m3-leftover-code-r1/tree}"
OPUS_DIR="${OPUS_DIR:?设成一个空的草稿目录}"
MODEL="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$OPUS_DIR/work" "$OPUS_DIR/mut"
rsync -a --exclude target --exclude .git "$FROZEN/" "$OPUS_DIR/work/"
cat "$MODEL/mount-probe-append.rs" >> "$OPUS_DIR/work/crates/singlefs-core/src/mount.rs"
for f in opus_probe_account_vs_mapping.rs opus_probe_swallowed_system_configuration_write.rs opus_probe_k3_outlets.rs; do
  cp "$MODEL/$f" "$OPUS_DIR/work/crates/singlefs-harness/tests/$f"
done
for f in crates/singlefs-core/src/mount.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/recovery.rs crates/singlefs-checker/src/walk.rs; do
  mkdir -p "$OPUS_DIR/clean/$(dirname "$f")"
  cp "$OPUS_DIR/work/$f" "$OPUS_DIR/clean/$f"
done
cp "$MODEL/mutants.json" "$MODEL"/g_*.list "$OPUS_DIR/mut/"
echo "setup done: $OPUS_DIR"
