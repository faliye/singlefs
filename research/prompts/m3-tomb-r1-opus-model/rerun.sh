#!/bin/bash
# m3-tomb-r1 攻方腿复跑。用法（从仓根，整条脚本经内存包装）：
#   nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 8 \
#     bash research/prompts/m3-tomb-r1-opus-model/rerun.sh <带 crates/ 的树根> <还不存在的草稿目录>
# 树根取冻结副本 /tmp/claude-1000/m3-delete-r2/tree（清单 research/prompts/m3-tomb-r1-snapshot/crates-sha256.txt）。
# 输出：草稿目录里的 run-e-lines.txt（Rust 原型）与 model-output.txt（计数模型），与本目录同名文件逐行比。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
source_root="$1"
scratch="$2"
if [ -e "$scratch" ]; then echo "草稿目录已存在：$scratch" >&2; exit 2; fi
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$source_root"/ "$scratch/tree/"
cp "$here/tomb_r1_attack_scan_after_rollback.rs" "$scratch/tree/crates/singlefs-harness/tests/"
(cd "$scratch/tree" && cargo test --offline -p singlefs-harness --test tomb_r1_attack_scan_after_rollback -- --nocapture --test-threads 1) > "$scratch/cargo.log" 2>&1
grep -o 'E name=.*' "$scratch/cargo.log" > "$scratch/run-e-lines.txt"
M="$here/tomb_rollback_model.py"
{
  python3 "$M" named
  python3 "$M" exhaustive 8
  TOMB_CAP=1 TOMB_RING=4 python3 "$M" exhaustive 8
  python3 "$M" random 20000 16 1
  TOMB_CAP=3 TOMB_RING=8 python3 "$M" random 20000 20 2
} > "$scratch/model-output.txt"
diff "$here/run-e-lines.txt" "$scratch/run-e-lines.txt" && echo "run-e-lines 相同"
diff "$here/model-output.txt" "$scratch/model-output.txt" && echo "model-output 相同"
