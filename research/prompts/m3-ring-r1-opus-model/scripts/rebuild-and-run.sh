#!/usr/bin/env bash
# m3-ring-r1 攻方腿的复跑：把仓拷到草稿目录，打上副本改法，编原型，按批跑。
# 用法（仓根下）：
#   bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 6 \
#     bash research/prompts/m3-ring-r1-opus-model/scripts/rebuild-and-run.sh <草稿目录> [build|direct|zero-single|cuts|enumerate|enumerate2|enumerate3|e156|all]
# 改法打在 HEAD fcc4d340 的 crates/ 上（开工快照 research/prompts/m3-ring-r1-snapshot/crates-sha256.txt）；
# 各批的全量状态数：enumerate 3 × 65569，enumerate2 65568 + 3 × 327712，enumerate3 2 × 65569（每段都不超过 10⁶；带文件后洞那段 4325419 态不跑）。
set -eu
scratch=$1
batch=${2:-direct}
model=$(cd "$(dirname "$0")/.." && pwd)
repository=$(cd "$model/../../.." && pwd)
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$repository/" "$scratch/repo/"
cd "$scratch/repo"
patch -p1 --forward < "$model/core-and-checker.diff"
mkdir -p crates/ring-hole-proto/src/bin
cp "$model/ring-hole-proto/Cargo.toml" crates/ring-hole-proto/
cp "$model/ring-hole-proto/src/lib.rs" crates/ring-hole-proto/src/
cp "$model"/ring-hole-proto/src/bin/*.rs crates/ring-hole-proto/src/bin/
cp crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs crates/ring-hole-proto/src/bin/e156_fill_probe.rs
patch crates/ring-hole-proto/src/bin/e156_fill_probe.rs < "$model/e156_fill_probe.diff"
grep -q 'crates/ring-hole-proto' Cargo.toml || python3 - <<'PY'
text = open('Cargo.toml').read()
anchor = '    "crates/singlefs-harness",\n'
assert text.count(anchor) == 1
open('Cargo.toml', 'w').write(text.replace(anchor, anchor + '    "crates/ring-hole-proto",\n'))
PY
export CARGO_TARGET_DIR="$scratch/target"
nice -n 19 cargo build --offline --release -p ring-hole-proto
export RING_HOLE_BIN_DIR="$scratch/target/release"
run() { mkdir -p "$scratch/$1"; RING_HOLE_OUT="$scratch/$1" bash "$model/scripts/$2" | tee "$scratch/$1/$2.log"; }
case "$batch" in
  direct) run h1 run_direct.sh ;;
  zero-single) run h1 run_zero_single.sh ;;
  cuts) run h2 run_cuts.sh ;;
  enumerate) run h2 run_enumerations.sh ;;
  enumerate2) mkdir -p "$scratch/h1"; run h2 run_enumerations_2.sh ;;
  e156) run e156 run_e156_modes.sh; for m in 0 1 2 3 4; do :; done; python3 "$model/scripts/e156_peaks.py" "$scratch"/e156/mode{0,1,2,3,4}.out > "$scratch/e156/peaks.txt" ;;
  enumerate3) run h2 run_enumerations_3.sh ;;
  build) ;;
  all) for b in direct zero-single cuts enumerate enumerate2 enumerate3 e156; do bash "$0" "$scratch" "$b"; done ;;
  *) echo "不认识的批：$batch" >&2; exit 2 ;;
esac
