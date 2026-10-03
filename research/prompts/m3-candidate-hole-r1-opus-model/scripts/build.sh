#!/bin/bash
# 用法：build.sh <草稿目录>。把冻结副本拷进 <草稿目录>/tree（已在就不拷）、打原型补丁（apply-walk-patch.py）、放进原型 crate hole-attack-proto、
# 经内存包装 5G 与 capped.sh 8 编 release。冻结副本不动。
set -euo pipefail
scratch=$1
tree=${TREE:-/tmp/claude-1000/m3-candidate-hole-r1/tree}
model=$(cd "$(dirname "$0")/.." && pwd)
if [ ! -d "$scratch/tree" ]; then
  rsync -a --exclude target --exclude .git "$tree/" "$scratch/tree/"
  python3 "$model/scripts/apply-walk-patch.py" "$scratch/tree"
fi
cp "$model/walk-patch/hole_proto.rs" "$scratch/tree/crates/singlefs-checker/src/hole_proto.rs"
if ! grep -q 'crates/hole-attack-proto' "$scratch/tree/Cargo.toml"; then
  sed -i 's|    "crates/singlefs-reference-filesystem",|    "crates/singlefs-reference-filesystem",\n    "crates/hole-attack-proto",|' "$scratch/tree/Cargo.toml"
fi
rm -rf "${scratch:?}/tree/crates/hole-attack-proto"
cp -r "$model/proto" "$scratch/tree/crates/hole-attack-proto"
cd "$scratch/tree"
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-candidate-hole-r1 攻方腿原型编译，与别的会话各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="改了原型源码要增量重编，峰值随改动不同" CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 5G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 8 cargo build --offline --release -p hole-attack-proto > "$scratch/build.log" 2>&1 || echo "build_failed" >> "$scratch/build.log"
grep -E '^(error|warning: unused)' -A12 "$scratch/build.log" | head -80 || true
tail -3 "$scratch/build.log"
