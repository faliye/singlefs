#!/usr/bin/env bash
# 第 19 批代码轮第一轮云端攻方腿的复跑：把冻结副本拷进工作目录、打 patches/b19-opus.patch、装原型，逐格跑，输出落进 $WORK/out/。
# 环境变量：REPO（仓根，缺省本文件往上四级）、FROZEN（冻结副本，缺省 /tmp/claude-1000/m3-formal-code-b19-r1/tree）、
# WORK（工作目录，缺省 /tmp/claude-1000/m3-formal-code-b19-r1-opus-rerun，须不存在）、THREADS（缺省 8）、CAP（缺省 4G）。
# 副本上的数，不是入库装置上的数。约 10 分钟（本机 2026-10-03 各格挂钟见报告）。
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL="$(cd "$HERE/.." && pwd)"
REPO="${REPO:-$(cd "$MODEL/../../.." && pwd)}"
FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-code-b19-r1/tree}"
WORK="${WORK:-/tmp/claude-1000/m3-formal-code-b19-r1-opus-rerun}"
THREADS="${THREADS:-8}"
CAP="${CAP:-4G}"
if [[ -e "$WORK" ]]; then
  echo "WORK=$WORK 已存在，换一个或先删掉" >&2
  exit 2
fi
mkdir -p "$WORK/ws" "$WORK/out"
rsync -a --exclude target --exclude .git "$FROZEN/crates" "$WORK/ws/"
cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/ws/"
(cd "$WORK/ws" && patch -p1 < "$MODEL/patches/b19-opus.patch")
rsync -a "$MODEL/prototype/" "$WORK/ws/crates/b19-opus-prototype/"
python3 - "$WORK/ws/Cargo.toml" <<'PY'
import sys
path = sys.argv[1]
text = open(path).read()
anchor = '    "crates/singlefs-reference-filesystem",\n]'
assert text.count(anchor) == 1
open(path, 'w').write(text.replace(anchor, '    "crates/singlefs-reference-filesystem",\n    "crates/b19-opus-prototype",\n]'))
PY
export CARGO_TARGET_DIR="$WORK/target"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-复跑攻方腿副本上的探针与原型}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-复跑攻方腿各格，各格规模不同}"
CAPPED=(nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" "$CAP" bash "$REPO/scripts/capped.sh" "$THREADS")
probe() { (cd "$WORK/ws" && "${CAPPED[@]}" cargo test --offline -p singlefs-harness --test b19_opus_probe "$1" -- --nocapture --test-threads 1); }
gpu_check() { (cd "$WORK/ws" && "${CAPPED[@]}" cargo test --offline -p singlefs-checker --lib b19_opus -- --nocapture); }
# cell 名字 [变量=值 …] 命令…：变量只在这一格的子 shell 里设；ONLY 设了时只跑名字里含它的格。
cell() {
  local name="$1"; shift
  if [[ -n "${ONLY:-}" && "$name" != *"$ONLY"* ]]; then return 0; fi
  echo "== $name"
  local status=0
  ( while [[ $# -gt 0 && "$1" == *=* ]]; do export "$1"; shift; done; "$@" ) > "$WORK/out/$name.log" 2>&1 || status=$?
  echo "exit=$status" >> "$WORK/out/$name.log"
}
(cd "$WORK/ws" && "${CAPPED[@]}" cargo build --offline --release -p b19-opus-prototype) > "$WORK/out/build-prototype.log" 2>&1
cell f0-small B19_F0_SLOTS=64,100,127,128,129,192,256,320,1000,680,690,700,704,710,720,740,768 probe b19_opus_form_f0
cell f1-sweep B19_F1_SLOTS="$(seq -s, 740 20 1600)" B19_F1_MOUNTS="$(seq -s, 1 12)" probe b19_opus_form_f1
cell f1-zero B19_F1_SLOTS=2000,800 B19_F1_MOUNTS=1,2,3 B19_F1_ZERO="$(seq -s, 0 1 30)" probe b19_opus_form_f1
cell z-default-ring B19_RING_MIB=768 B19_Z_MOUNTS=2,3,9,10 B19_RED_DETAIL=1 probe b19_opus_form_z
for slack in 0 1 3; do
  cell "h2-slack$slack" B19_GATE_SLACK=$slack B19_F1_SLOTS=2000 B19_F1_MOUNTS="$(seq -s, 1 12)" B19_RED_DETAIL=1 probe b19_opus_form_f1
done
for slack in 0 1; do
  cell "h2-default-ring-slack$slack" B19_GATE_SLACK=$slack B19_RING_MIB=768 B19_F1_SLOTS=211968 B19_F1_MOUNTS=9,10 B19_RED_DETAIL=1 probe b19_opus_form_f1
done
for bad in h1_from_u_plus_4 skip_first_free_pair ignore_cluster_segments; do
  cell "bad-$bad" B19_BAD=$bad B19_F1_SLOTS=2000 B19_F1_MOUNTS=1,2,9,10 probe b19_opus_form_f1
done
cell gpu-transliteration B19_GPU_CASES=300000 gpu_check
cell gpu-transliteration-mutant B19_GPU_CASES=300000 B19_GPU_MUTANT=1 gpu_check
for slack in 0 1; do
  cell "layer0-sample-slack$slack" B19_EXPAND_FROM=1 B19_GATE_SLACK=$slack B19_MOUNTS=9 "${CAPPED[@]}" "$WORK/target/release/b19-opus-prototype"
done
grep -ho 'name=.*' "$WORK"/out/*.log > "$WORK/out/all-name-lines.txt" || true
sha256sum "$WORK"/out/*.log
