#!/usr/bin/env bash
# 一个攻方变异：从冻结副本拷一份仓、改、编那份 harness 测试文件与攻方原型、整份跑测试文件（--include-ignored），
# 再跑原型里的两小段（K1 主形 pre=4 fill=16、实现员三形）。run-mutant.sh <变异名> <工作目录>
set -euo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree
name=${1:?}; WORK=${2:?}
CAP=$ROOT/research/scripts/run-with-memory-cap.sh
THREADS=$ROOT/research/scripts/capped.sh
OUT=$WORK/runs
mkdir -p "$OUT"
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo-mut/
python3 "$MODEL"/mutants.py "$WORK"/repo-mut "$name"
bash "$MODEL"/assemble.sh "$WORK"/repo-mut "$MODEL"
cd "$WORK"/repo-mut
bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target-mut nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --test atk_tu_opus --no-run > "$OUT"/mut-$name-build.log 2>&1
RBIN=$(ls -t "$WORK"/target-mut/release/deps/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table-* | grep -v '\.d$' | head -1)
ABIN=$(ls -t "$WORK"/target-mut/release/deps/atk_tu_opus-* | grep -v '\.d$' | head -1)
bash "$CAP" 12G nice -n 19 bash "$THREADS" 12 "$RBIN" --include-ignored --test-threads 12 > "$OUT"/mut-$name-file.log 2>&1 || true
bash "$CAP" 3G env ATK_PRES=4 ATK_FILLS=16 ATK_EXTRA=2 nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact atk_k1_rehearsal_vs_real_sweep > "$OUT"/mut-$name-atk-k1.log 2>&1 || true
bash "$CAP" 3G nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact atk_k1_implementer_forms_rehearsal_vs_real > "$OUT"/mut-$name-atk-impl.log 2>&1 || true
echo "$name file: $(grep -h 'test result' "$OUT"/mut-$name-file.log | tail -1)"
echo "$name atk-k1: $(grep -h 'totals\|test result' "$OUT"/mut-$name-atk-k1.log | tr '\n' ' ')"
echo "$name atk-impl: $(grep -h 'totals\|test result' "$OUT"/mut-$name-atk-impl.log | tr '\n' ' ')"
