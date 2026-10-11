#!/usr/bin/env bash
# 一个攻方变异：从冻结副本另拷一份、改、拼原型、编，跑那份 harness 测试文件与攻方原型里点名的几条。
# run-mutant.sh <变异名> <工作目录> [<原型用例名>@<环境变量，用 + 分隔…> …]；每件经 run-with-memory-cap.sh，线程经 capped.sh。
set -euo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree
mutant=${1:?}; WORK=${2:?}; shift 2
CAP=$ROOT/scripts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/capped.sh
OUT=$WORK/runs
mkdir -p "$OUT"
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo-mut/
python3 "$MODEL"/mutants.py "$WORK"/repo-mut "$mutant"
bash "$MODEL"/assemble.sh "$WORK"/repo-mut "$MODEL"
( cd "$WORK"/repo-mut && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target-mut nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test atk_vw_opus --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --no-run ) > "$OUT"/mut-$mutant-build.log 2>&1
ABIN=$(ls -t "$WORK"/target-mut/release/deps/atk_vw_opus-* | grep -v '\.d$' | head -1)
RBIN=$(ls -t "$WORK"/target-mut/release/deps/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table-* | grep -v '\.d$' | head -1)
bash "$CAP" 12G nice -n 19 bash "$THREADS" 12 "$RBIN" --test-threads 12 > "$OUT"/mut-$mutant-file.log 2>&1 || true
i=0
for spec in "$@"; do
  i=$((i+1))
  name=${spec%%@*}; envs=${spec#*@}
  bash "$CAP" 3G env ${envs//+/ } nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact "$name" > "$OUT"/mut-$mutant-$i-$name.log 2>&1 || true
done
echo "$mutant | $(grep -h '^test result' "$OUT"/mut-$mutant-file.log | head -1)"
