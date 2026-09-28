#!/bin/bash
# m3-prune-gpu-r3 云端攻方腿：同一份仓副本上逐个换一处真代码、增量重编、跑点名的世界，每个变体跑完把那个文件换回原样。
# 用法：bash variants.sh <已放好原型的仓副本> <编译目录> <输出目录>
# 线程上限 4、内存上限 8G；历史全在内存稀疏盘上录。
set -euo pipefail
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
copy="${1:?仓副本}"; target="${2:?编译目录}"; out="${3:?输出目录}"
mkdir -p "$out" "$out/pristine"
rsync -a --include '*/' --include '*.rs' --exclude '*' "$copy/crates/" "$out/pristine/crates/"
build() {
  (cd "$copy" && CARGO_TARGET_DIR="$target" nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    cargo build --release --offline -p singlefs-checker-tier --features verdict-store --bin e161_crash_state_dedup_and_time_split) > "$out/build-$1.log" 2>&1
}
run_world() {
  local variant="$1" world="$2"
  (cd "$copy" && nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    "$target/release/e161_crash_state_dedup_and_time_split" attack "$world" 2>/dev/null | grep -v '^LAYER0' > "$out/$variant--$world.out")
}
# 换回原样时不带原来的时刻（rsync 不加 -t）：cargo 按修改时刻判新旧，带回旧时刻的文件会被当成没改过、它那个包不重编。
restore() { rsync -rlpgoD --checksum "$out/pristine/crates/" "$copy/crates/"; }
variant() {
  local name="$1"; shift
  local worlds="$1"; shift
  "$@"
  build "$name"
  for world in $worlds; do run_world "$name" "$world"; done
  python3 "$model/rust_tokens.py" tree-compare "$out/pristine" "$copy" > "$out/$name--tree-compare.out"
  restore
}
variant base "r3-dump r3-panic-location r2-f1-shifted" true
variant m57-no-barrier-between-records-and-root "r3-dump" python3 "$model/apply_mutation_row.py" "$copy" "步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）"
variant m23-recovery-chains-other-instances "r3-dump" python3 "$model/apply_mutation_row.py" "$copy" "步 3：前缀跨实例边界（把别的实例的记录也接上）"
variant root-before-records "r3-dump" patch -s "$copy/crates/singlefs-core/src/transaction.rs" "$model/transaction-root-before-records.diff"
variant f1-candidates-in-the-harness "r3-dump r2-f1-shifted" patch -s -d "$copy" -p1 -i "$model/f1-harness.patch"
variant m457-checker-payload-checksum-offset "r3-dump" python3 "$model/apply_mutation_row.py" "$copy" "记录头 311：池级 checker 的载荷校验和偏移没跟着后挪 4 字节"
variant line-shift-in-the-allocator "r3-panic-location" patch -s -d "$copy" -p1 -i "$model/line-shift.patch"
variant mkfs-watermark "r3-dump" patch -s -d "$copy" -p1 -i "$model/mkfs-watermark.patch"
echo "variants done"
