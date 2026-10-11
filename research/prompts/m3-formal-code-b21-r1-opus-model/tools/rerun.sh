#!/usr/bin/env bash
# m3-formal-code-b21-r1 攻方腿的副本探针复跑（只在副本里改、只在副本里编和跑；不跑 checker 档包的测试）。
# 用法：bash tools/rerun.sh <setup|build|k3|k3-sweep|k2a|k2b|k2b-sweep|f3|all>
# 环境变量：REPO_ROOT（仓根，取 Cargo.toml / Cargo.lock 与 scripts）、FROZEN_TREE（冻结副本根，下面是 crates/）、
#           WORK（副本与产物目录）、THREADS（线程上限，默认 7；k2b-sweep 每批并行 6 件、k3-sweep 并行 4 件，各件单线程）、BUILD_CAP（编译的内存上限，默认 8G）、RUN_CAP（跑探针的上限，默认 4G）。
# 产物写进 $WORK/out/，与模型目录 out/ 下同名文件逐行比。
set -euo pipefail
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$MODEL_DIR/../../.." && pwd)}"
FROZEN_TREE="${FROZEN_TREE:-/tmp/claude-1000/m3-formal-code-b21-r1/tree}"
WORK="${WORK:-/tmp/claude-1000/m3-formal-code-b21-r1-opus-rerun}"
THREADS="${THREADS:-7}"
BUILD_CAP="${BUILD_CAP:-8G}"
RUN_CAP="${RUN_CAP:-4G}"
CAP="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$REPO_ROOT/scripts/hosts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-攻方腿副本探针复跑，与别的活并行}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-探针各子命令峰值差得远，统一给一个上限}"
BIN="$WORK/copy/target/release/opus-b21-probe"
OUT="$WORK/out"

run() { nice -n 19 bash "$CAP" "$RUN_CAP" "$BIN" "$@" 2>&1 | grep -v '^run-with-memory-cap'; }

setup() {
    mkdir -p "$WORK/copy" "$OUT"
    rsync -a --delete --exclude target --exclude .git "$FROZEN_TREE/crates" "$WORK/copy/"
    cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/copy/"
    (cd "$WORK/copy" && patch -p1 < "$MODEL_DIR/probe/copy-probe.patch")
    cp "$MODEL_DIR/probe/opus_probe.rs" "$WORK/copy/crates/singlefs-checker-tier/src/opus_probe.rs"
    mkdir -p "$WORK/copy/crates/opus-b21-probe/src"
    cp "$MODEL_DIR/probe/opus-b21-probe/Cargo.toml" "$WORK/copy/crates/opus-b21-probe/Cargo.toml"
    cp "$MODEL_DIR/probe/opus-b21-probe/src/main.rs" "$WORK/copy/crates/opus-b21-probe/src/main.rs"
    python3 - "$WORK/copy/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
if '"crates/opus-b21-probe"' not in s:
    s = s.replace('    "crates/singlefs-reference-filesystem",\n]', '    "crates/singlefs-reference-filesystem",\n    "crates/opus-b21-probe",\n]')
    open(p + '.new', 'w').write(s)
PY
    if [ -f "$WORK/copy/Cargo.toml.new" ]; then mv "$WORK/copy/Cargo.toml.new" "$WORK/copy/Cargo.toml"; fi
    grep -q opus-b21-probe "$WORK/copy/Cargo.toml"
}

build() {
    (cd "$WORK/copy" && nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo build --offline --release -p opus-b21-probe)
}

k3() {
    for c in "control with-rotation" "control no-rotation" "ff-record-lost with-rotation" "ff-record-lost no-rotation"; do
        echo "== k3 $c"
        # shellcheck disable=SC2086
        run k3 $c
    done > "$OUT/k3.txt"
}

k3_sweep() {
    rm -f "$OUT"/k3s-b1-*.rc
    local n=0 piece
    for piece in "0 0 8" "0 8 15" "1 0 8" "1 8 15"; do
        n=$((n + 1))
        # shellcheck disable=SC2086
        { run k3-sweep $piece > "$OUT/k3s-b1-$n.out"; echo "$?" > "$OUT/k3s-b1-$n.rc"; } &
    done
    wait
    [ "$(ls "$OUT"/k3s-b1-*.rc | wc -l)" -eq 4 ] || { echo "k3-sweep 这一批的退出码文件数不是 4，整批作废" >&2; return 1; }
    for i in 1 2 3 4; do echo "k3s-b1-$i rc=$(cat "$OUT/k3s-b1-$i.rc")"; done
    cat "$OUT"/k3s-b1-1.out "$OUT"/k3s-b1-2.out "$OUT"/k3s-b1-3.out "$OUT"/k3s-b1-4.out > "$OUT/k3-sweep.txt"
}

k2a() {
    for v in correct zero old-instance being-deleted old-birth size time-zero skip-one change-zero reuse-first; do run k2a "$v"; done > "$OUT/k2a.txt"
}

k2b() {
    {
        OPUS_MEMBER=1 run k2b 128 3 14 1
        OPUS_MEMBER=1 run k2b 136 3 14 1
        OPUS_MEMBER=1 OPUS_K2B_SOME=1 run k2b 128 3 14 1
        OPUS_MEMBER=1 OPUS_K2B_SOME=1 run k2b 136 3 14 1
        OPUS_MEMBER=1 OPUS_K2A_VARIANT=zero run k2b 128 3 14 1
        OPUS_MEMBER=1 OPUS_K2A_VARIANT=zero run k2b 136 3 14 1
    } > "$OUT/k2b.txt"
}

k2b_sweep() {
    local combos=() ow u b batch=0 start n c
    for ow in 0 1 3; do for u in 4 8 14 20; do for b in 1 2; do combos+=("$ow $u $b"); done; done; done
    for start in 0 6 12 18; do
        batch=$((batch + 1)); rm -f "$OUT"/k2b-b$batch-*.rc; n=0
        for c in "${combos[@]:$start:6}"; do
            n=$((n + 1))
            # shellcheck disable=SC2086
            { run k2b-sweep 128 400 4 $c > "$OUT/k2b-b$batch-$n.out"; echo "$?" > "$OUT/k2b-b$batch-$n.rc"; } &
        done
        wait
        [ "$(ls "$OUT"/k2b-b$batch-*.rc | wc -l)" -eq "$n" ] || { echo "k2b-sweep 第 $batch 批退出码文件数不对，整批作废" >&2; return 1; }
        for i in $(seq 1 "$n"); do echo "k2b-b$batch-$i rc=$(cat "$OUT/k2b-b$batch-$i.rc")"; done
    done
    cat "$OUT"/k2b-b*-*.out | grep '^slots=' | sort -t= -k2 > "$OUT/k2b-sweep.txt"
    python3 "$MODEL_DIR/tools/summarize_k2b.py" "$OUT/k2b-sweep.txt" > "$OUT/k2b-sweep-classes.txt"
}

f3() {
    {
        run f3-arith
        for args in "7 5 3" "7 16 8" "0 16 12" "32768 3 12" "245760 20 10"; do
            # shellcheck disable=SC2086
            run f3 $args
        done
        for fp in fnv1a empty first-only popcount first-withheld; do
            OPUS_F3_FP=$fp run f3-inline
            OPUS_F3_FP=$fp run f3 0 16 12 | sed "s/^/fp=$fp /"
        done
    } > "$OUT/f3.txt"
}

case "${1:-}" in
    setup) setup ;;
    build) build ;;
    k3) k3 ;;
    k3-sweep) k3_sweep ;;
    k2a) k2a ;;
    k2b) k2b ;;
    k2b-sweep) k2b_sweep ;;
    f3) f3 ;;
    all) setup; build; k3; k3_sweep; k2a; k2b; k2b_sweep; f3 ;;
    *) echo "用法：$0 <setup|build|k3|k3-sweep|k2a|k2b|k2b-sweep|f3|all>" >&2; exit 2 ;;
esac
