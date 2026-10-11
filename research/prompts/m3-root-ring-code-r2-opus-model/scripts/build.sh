#!/bin/bash
# 用法：build.sh <草稿目录>。把冻结副本拷进 <草稿目录>/tree（已在就不拷）、放进原型 crate e2-attack-proto、经内存包装 6G 与 capped.sh 15 编 release。
# 冻结副本的 crates 不改一个字节，只往工作区 members 里加原型。
set -euo pipefail
scratch=$1
tree=${TREE:-/tmp/claude-1000/m3-root-ring-code-r2/tree}
model=$(cd "$(dirname "$0")/.." && pwd)
if [ ! -d "$scratch/tree" ]; then
  rsync -a --exclude target --exclude .git "$tree/" "$scratch/tree/"
fi
if ! grep -q 'crates/e2-attack-proto' "$scratch/tree/Cargo.toml"; then
  sed -i 's|    "crates/singlefs-reference-filesystem",|    "crates/singlefs-reference-filesystem",\n    "crates/e2-attack-proto",|' "$scratch/tree/Cargo.toml"
fi
rm -rf "${scratch:?}/tree/crates/e2-attack-proto"
cp -r "$model/proto" "$scratch/tree/crates/e2-attack-proto"
cd "$scratch/tree"
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-root-ring-code-r2 攻方腿原型编译，与别的会话各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="改了原型源码要增量重编，峰值随改动不同" CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 5G bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 15 cargo build --offline --release -p e2-attack-proto > "$scratch/build.log" 2>&1 || echo "build_failed" >> "$scratch/build.log"
grep -E '^error' -A12 "$scratch/build.log" | head -80 || true
tail -3 "$scratch/build.log"
