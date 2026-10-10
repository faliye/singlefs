#!/usr/bin/env bash
# 复跑攻方 m3-formal-code-b20-r1-opus 的三份 harness 档副本探针（只跑 singlefs-harness 的三个测试目标，不碰 checker 档）。
# 环境变量：REPO（仓根，取 Cargo.toml、Cargo.lock 与内存包装脚本；缺省本仓）、FROZEN（冻结副本的树根，缺省本轮冻结副本）、
# WORK（拷仓副本与编译目录的地方，缺省新建一个临时目录）、THREADS（线程上限，缺省 10）、CAP（每条命令的内存上限，缺省 2G）。
# 跑完看：$WORK/out/k1.log 里 F1、F2、K2 fixed、F5 开头的行；$WORK/out/k2a.log 里 SUMMARY 与 ARM 行；$WORK/out/k2b-map-*.log 里 TALLY 行。
set -euo pipefail
MODEL_DIRECTORY="$(cd "$(dirname "$0")/.." && pwd)"
REPO="${REPO:-$(cd "$MODEL_DIRECTORY/../../.." && pwd)}"
FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-code-b20-r1/tree}"
WORK="${WORK:-$(mktemp -d /tmp/claude-1000/b20-opus-rerun.XXXXXX)}"
THREADS="${THREADS:-10}"
CAP="${CAP:-2G}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑攻方副本探针，上限按复跑者给的"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑攻方副本探针，与别的会话各跑各的"
mkdir -p "$WORK/repo" "$WORK/out"
rsync -a --exclude target --exclude .git "$FROZEN/crates" "$WORK/repo/"
cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/repo/"
cp "$MODEL_DIRECTORY"/probes/*.rs "$WORK/repo/crates/singlefs-harness/tests/"
sha256sum "$MODEL_DIRECTORY"/probes/*.rs | tee "$WORK/out/probes.sha256"
cd "$WORK/repo"
export CARGO_TARGET_DIR="$WORK/target"
run() {
  bash "$REPO/research/scripts/run-with-memory-cap.sh" "$CAP" bash "$REPO/research/scripts/capped.sh" "$THREADS" nice -n 19 "$@"
}
for target in b20_opus_k1_targeted_states b20_opus_k2_short_continuation b20_opus_k2_histories_on_narrow_widths; do
  bash "$REPO/research/scripts/capped.sh" "$THREADS" nice -n 19 cargo test --offline --release --no-run -p singlefs-harness --test "$target"
done
run cargo test --offline --release -p singlefs-harness --test b20_opus_k1_targeted_states -- --nocapture --test-threads=1 > "$WORK/out/k1.log" 2>&1
B20_SLOTS_FROM=100 B20_SLOTS_TO=320 B20_SLOTS_STEP=2 \
  run cargo test --offline --release -p singlefs-harness --test b20_opus_k2_short_continuation -- --nocapture --test-threads=1 > "$WORK/out/k2a.log" 2>&1
for width in 384 256 240; do
  B20_BOUNDARY_ONLY=1 B20_WIDTH="$width" B20_WEIGHTS=deleting B20_SEED_FROM=0 B20_SEED_COUNT=60 B20_OPS=24 \
    run cargo test --offline --release -p singlefs-harness --test b20_opus_k2_histories_on_narrow_widths -- --nocapture > "$WORK/out/k2b-map-$width.log" 2>&1
done
grep -E '^(F1|F2|K2 fixed|   parsed)|F5 abandoning|test result' "$WORK/out/k1.log" | cut -c1-400
grep -E 'SUMMARY' "$WORK/out/k2a.log"
grep -E 'ARM ' "$WORK/out/k2a.log" | sed -E 's/(拒 [A-Za-z]+\(?[A-Za-z]*)[^；|]*/\1/' | sort | uniq -c
for width in 384 256 240; do echo "== $width"; grep TALLY "$WORK/out/k2b-map-$width.log"; done
echo "WORK=$WORK（拷的仓副本与编译目录，看完自行删除）"
