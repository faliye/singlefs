#!/usr/bin/env bash
# m3-formal-code-b23-r1 攻方腿的副本探针复跑（只在副本里改、只在副本里编和跑；不跑 checker 档包的任何测试目标）。
# 用法：bash tools/rerun.sh <setup|build|build-tests|k3a|f3|a1|b1|b1-sweep|collateral|collateral-one <none|nlink|birth|size>|all>
# 环境变量：REPO_ROOT（仓根，取 Cargo.toml / Cargo.lock 与 research/scripts）、FROZEN_TREE（冻结副本根，下面是 crates/）、
#           WORK（副本与产物目录）、THREADS（线程上限，默认 7）、BUILD_CAP（编译的内存上限，默认 8G）、RUN_CAP（跑探针与 harness 测试的上限，默认 4G）。
# 产物写进 $WORK/out/，与模型目录 out/ 下同名文件逐字节比（cmp）。
set -euo pipefail
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$MODEL_DIR/../../.." && pwd)}"
FROZEN_TREE="${FROZEN_TREE:-/tmp/claude-1000/m3-formal-code-b23-r1/tree}"
WORK="${WORK:-/tmp/claude-1000/m3-formal-code-b23-r1-opus-rerun}"
THREADS="${THREADS:-7}"
BUILD_CAP="${BUILD_CAP:-8G}"
RUN_CAP="${RUN_CAP:-4G}"
CAP="$REPO_ROOT/research/scripts/run-with-memory-cap.sh"
CAPPED="$REPO_ROOT/research/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-攻方腿副本探针复跑，与别的活并行}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-探针各子命令峰值差得远，统一给一个上限}"
BIN="$WORK/copy/target/release/opus-b23-probe"
OUT="$WORK/out"

# 探针的 stderr（包装的提示、panic 钩子带线程号的那一行）进 $OUT/stderr.log，不进产物；探针单线程、注入线程变量经 capped.sh 设成 1。
run() { nice -n 19 bash "$CAP" "$RUN_CAP" bash "$CAPPED" 1 "$BIN" "$@" 2>>"$OUT/stderr.log"; }
TESTS_ARGS=(--offline --release -p singlefs-harness -p singlefs-core -p singlefs-checker --tests)

setup() {
    mkdir -p "$WORK/copy" "$OUT"
    rsync -a --delete --exclude target --exclude .git "$FROZEN_TREE/crates" "$WORK/copy/"
    cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/copy/"
    python3 "$MODEL_DIR/tools/patch_copy.py" "$WORK/copy"
    cp "$MODEL_DIR/probe/opus_probe.rs" "$WORK/copy/crates/singlefs-checker-tier/src/opus_probe.rs"
    mkdir -p "$WORK/copy/crates/opus-b23-probe/src"
    cp "$MODEL_DIR/probe/opus-b23-probe/Cargo.toml" "$WORK/copy/crates/opus-b23-probe/Cargo.toml"
    cp "$MODEL_DIR/probe/opus-b23-probe/src/main.rs" "$WORK/copy/crates/opus-b23-probe/src/main.rs"
    python3 - "$WORK/copy/Cargo.toml" <<'PY'
import sys, os
p = sys.argv[1]
s = open(p).read()
if '"crates/opus-b23-probe"' not in s:
    s = s.replace('    "crates/singlefs-reference-filesystem",\n]', '    "crates/singlefs-reference-filesystem",\n    "crates/opus-b23-probe",\n]')
    open(p + '.new', 'w').write(s)
    os.replace(p + '.new', p)
PY
    grep -q opus-b23-probe "$WORK/copy/Cargo.toml"
    # harness 档有两条用例读仓根的 litmus/（副本里没有就红在读文件上，与变异无关）
    rsync -a --delete "$REPO_ROOT/litmus/" "$WORK/copy/litmus/"
}

build() {
    (cd "$WORK/copy" && nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo build --offline --release -p opus-b23-probe)
}

# collateral 要的测试目标（harness、core、checker 三个包的全部测试目标，不含 checker 档包）
build_tests() {
    (cd "$WORK/copy" && nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo test "${TESTS_ARGS[@]}" --no-run)
}

k3a() {
    : > "$OUT/k3a-variants.txt"
    local w l o
    for w in 384 256 240 4g; do for l in inside0 inside7 empty unit above span3; do for o in 1 2 3; do
        run k3a "$w" "$l" "$o" >> "$OUT/k3a-variants.txt"
    done; done; done
}

f3() {
    { run f3-bitwise 200000; run f3-segments 384 64 16; run f3-segments 4g 64 16; } > "$OUT/f3-segments.txt"
    run f3-variants 384 16 12 > "$OUT/f3-variants.txt"
    run f3-fix > "$OUT/f3-fix.txt"
    nice -n 19 python3 "$MODEL_DIR/tools/f3_model.py" "$FROZEN_TREE" > "$OUT/f3-model.txt"
}

a1() {
    {
        run a1
        local v
        for v in being-deleted old-birth size change-previous birth-next txg-skip; do OPUS_K2A_VARIANT=$v run a1; done
        for v in time-zero count-zero count-two old-instance; do OPUS_ARM=$v run a1; done
        for v in nlink birth size; do OPUS_CORE_MUT=$v run a1; done
    } > "$OUT/a1.txt"
}

b1() {
    run b1-sweep 112 208 3 14 1 > "$OUT/b1-slots.txt"
}

# 覆盖写 2..4、单元 13..15、批 1 九组，每组单元区 120..176 槽；每批至多 6 件并行（各件单线程，合起来不超过线程上限 7）
b1_sweep() {
    local combos=() ow u batch=0 start n c
    for ow in 2 3 4; do for u in 13 14 15; do combos+=("$ow $u"); done; done
    for start in 0 6; do
        batch=$((batch + 1)); rm -f "$OUT"/b1n-b$batch-*.rc; n=0
        for c in "${combos[@]:$start:6}"; do
            n=$((n + 1))
            # shellcheck disable=SC2086
            { run b1-sweep 120 176 $c 1 > "$OUT/b1n-b$batch-$n.out"; echo "$?" > "$OUT/b1n-b$batch-$n.rc"; } &
        done
        wait
        [ "$(ls "$OUT"/b1n-b$batch-*.rc | wc -l)" -eq "$n" ] || { echo "b1-sweep 第 $batch 批退出码文件数不对，整批作废" >&2; return 1; }
        for i in $(seq 1 "$n"); do [ "$(cat "$OUT/b1n-b$batch-$i.rc")" = 0 ] || { echo "b1-sweep 第 $batch 批第 $i 件退出码 $(cat "$OUT/b1n-b$batch-$i.rc")" >&2; return 1; }; done
    done
    cat "$OUT"/b1n-b1-{1..6}.out "$OUT"/b1n-b2-{1..3}.out > "$OUT/b1-neighbourhood.txt"
}

# 23-1 那三行 core 变异（OPUS_CORE_MUT，每个调用都生效）各跑一遍 harness、core、checker 三个包的全部测试目标，外加不设变异的基线；
# 产物只留每次的 ok 目标数、红的目标与红的用例名（不留耗时）。每次约 3 分钟。
collateral_one() {
    local m="$1" log="$OUT/collateral-$1.log"
    if [ "$m" = none ]; then
        (cd "$WORK/copy" && nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo test "${TESTS_ARGS[@]}" --no-fail-fast) > "$log" 2>&1 || true
    else
        (cd "$WORK/copy" && OPUS_CORE_MUT=$m nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo test "${TESTS_ARGS[@]}" --no-fail-fast) > "$log" 2>&1 || true
    fi
    {
        echo "core_mutation=$m targets_ok=$(grep -c '^test result: ok' "$log") targets_failed=$(grep '^test result' "$log" | grep -vc 'ok\.')"
        { grep -E '^    `-p ' "$log" || true; } | sed 's/^ */  failed_target /'
        { grep -E '^test .* FAILED$' "$log" || true; } | sort -u | sed 's/^/  /'
    } > "$OUT/collateral-$m.txt"
}

collateral() {
    local m
    for m in none nlink birth size; do collateral_one "$m"; done
    cat "$OUT"/collateral-none.txt "$OUT"/collateral-nlink.txt "$OUT"/collateral-birth.txt "$OUT"/collateral-size.txt > "$OUT/collateral.txt"
}

case "${1:-}" in
    setup) setup ;;
    build) build ;;
    build-tests) build_tests ;;
    k3a) k3a ;;
    f3) f3 ;;
    a1) a1 ;;
    b1) b1 ;;
    b1-sweep) b1_sweep ;;
    collateral) collateral ;;
    collateral-one) collateral_one "$2" ;;
    all) setup; build; build_tests; k3a; f3; a1; b1; b1_sweep; collateral ;;
    *) echo "用法：$0 <setup|build|build-tests|k3a|f3|a1|b1|b1-sweep|collateral|all>" >&2; exit 2 ;;
esac
