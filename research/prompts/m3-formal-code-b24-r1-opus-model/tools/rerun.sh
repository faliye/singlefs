#!/usr/bin/env bash
# m3-formal-code-b24-r1 攻方腿的副本探针复跑（只在副本里改、只在副本里编和跑；不跑 checker 档包的任何测试目标与装置）。
# 用法：bash tools/rerun.sh <段>，段按次序：setup build real-small real-large-batch 1..3 real-dense-batch real-collect
#        f3-begin f3-rows f3-end b1-slots b1-batch 1..3 b1-collect harness h2s-setup h2s 5|15 h2s-compile 4|16；all 依次全跑。
# 环境变量：REPO_ROOT（仓根，取 Cargo.toml / Cargo.lock 与 scripts）、FROZEN_TREE（冻结副本根，下面是 crates/）、
#           WORK（副本与产物目录）、THREADS（线程上限，默认 10）、BUILD_CAP（编译的内存上限，默认 3G；量过冷编峰值 1.4 GiB）、RUN_CAP（跑探针的上限，默认 256M；量过峰值 55 MiB）。
# 产物写进 $WORK/out/，与模型目录 out/ 下同名文件逐字节比（cmp）。每段前台跑得完（各段挂钟见报告「复跑」一节）。
set -euo pipefail
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$MODEL_DIR/../../.." && pwd)}"
FROZEN_TREE="${FROZEN_TREE:-/tmp/claude-1000/m3-formal-code-b24-r1/tree}"
WORK="${WORK:-/tmp/claude-1000/m3-formal-code-b24-r1-opus-rerun}"
THREADS="${THREADS:-10}"
BUILD_CAP="${BUILD_CAP:-3G}"
RUN_CAP="${RUN_CAP:-256M}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-200}"
CAP="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$REPO_ROOT/scripts/hosts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-攻方腿副本探针复跑，与别的活并行}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-探针各子命令峰值差得远，统一给一个上限}"
COPY="$WORK/copy"
COPY2="$WORK/copy-h2s"
BIN="$COPY/target/release/opus-b24-probe"
OUT="$WORK/out"
SEED_BASE=7463871032432355113
FROZEN_CI="$FROZEN_TREE/crates/singlefs-checker-tier/src/crash_injection.rs"
FROZEN_TSV="$FROZEN_TREE/crates/mutations.tsv"
OPUS_TSV="$MODEL_DIR/tools/opus-mutations.tsv"
GEN_DIR="$COPY/crates/opus-b24-probe/src"

# 探针单线程（注入线程变量经 capped.sh 设成 1）；探针的 stderr 进 $OUT/stderr.log，不进产物。
run() { nice -n 19 bash "$CAP" "$RUN_CAP" bash "$CAPPED" 1 "$BIN" "$@" 2>>"$OUT/stderr.log"; }
cargo_in() { local dir="$1"; shift; (cd "$dir" && nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo "$@"); }

# 并行一批：第一个参数是批名，之后每个参数是一件的命令；每件输出进 $OUT/<批名>-<件号>.out、退出码进 .rc，数齐再按次序逐个读。
batch() {
    local name="$1"; shift
    local n=0 spec j
    rm -f "${OUT:?}/${name:?}"-*.rc
    for spec in "$@"; do
        n=$((n + 1))
        { rc=0; eval "$spec" > "$OUT/$name-$n.out" || rc=$?; echo "$rc" > "$OUT/$name-$n.rc"; } &
    done
    wait
    [ "$(ls "$OUT/$name"-*.rc | wc -l)" -eq "$n" ] || { echo "$name：退出码文件数不对，整批作废" >&2; return 1; }
    for j in $(seq 1 "$n"); do [ "$(cat "$OUT/$name-$j.rc")" = 0 ] || { echo "$name 第 $j 件退出码 $(cat "$OUT/$name-$j.rc")" >&2; return 1; }; done
}

setup() {
    mkdir -p "$COPY" "$OUT"
    rsync -a --delete --exclude target --exclude .git "$FROZEN_TREE/crates" "$COPY/"
    # 冻结副本里 E171R1 装置是别的活的半成品，编不过 clippy、不在这一轮（派发提示允许挪开）
    rm -f "${COPY:?}/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
    cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$COPY/"
    python3 "$MODEL_DIR/tools/patch_copy.py" "$COPY" "$MODEL_DIR"
    python3 "$MODEL_DIR/tools/gen_seedfn.py" "$FROZEN_CI" "$GEN_DIR/seedfn_pristine.rs" >/dev/null
    python3 "$MODEL_DIR/tools/gen_seedfn.py" "$FROZEN_CI" "$GEN_DIR/seedfn.rs" >/dev/null
    # C670 那一句：harness 档用例原样拷进探针包（去掉文件头的 //! 模块文档），后面接攻方一条
    { grep -v '^//!' "$FROZEN_TREE/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs"
      cat "$MODEL_DIR/probe/opus-b24-harness-probe/c670_extra.rs.in"; } > "$COPY/crates/opus-b24-harness-probe/tests/c670_other_disk.rs"
}

build() {
    cargo_in "$COPY" build --offline --release -p opus-b24-probe
    cargo_in "$COPY" test --offline --release -p opus-b24-harness-probe --no-run
}

# 真历史：战役各档（tests/crash_injection_campaign.rs：快档 24 段 24 步每段 4 个、偏回退 12 段 20 步 6 个、线程那条 8 段 16 步 3 个、
# 大档 500 段 40 步 8 个，都是 4 GiB、种子基起），另加稠密摆法（短段全部真子集、长段各 64 个）24 段 40 步。
real_small() {
    batch real-small "run real $SEED_BASE 24 24 broad 4g 4" "run real $SEED_BASE 12 20 rollback 4g 6" "run real $SEED_BASE 8 16 broad 4g 3"
}
# 大档 500 段分 30 片、每片 17 段（末片 7 段），每批 10 片并行（各片单线程）。
real_large_batch() {
    local b="$1" j first count specs=()
    for j in $(seq 1 10); do
        first=$(( ((b - 1) * 10 + (j - 1)) * 17 ))
        count=$(( first + 17 > 500 ? 500 - first : 17 ))
        specs+=("run real $((SEED_BASE + first)) $count 40 broad 4g 8")
    done
    batch "real-large-b$b" "${specs[@]}"
}
real_dense_batch() {
    local j specs=()
    for j in 0 1 2 3 4 5; do specs+=("run real $((SEED_BASE + j * 4)) 4 40 broad 4g 0"); done
    batch real-dense "${specs[@]}"
}
real_collect() {
    cat "$OUT"/real-small-1.out > "$OUT/real-fast.txt"
    cat "$OUT"/real-small-2.out > "$OUT/real-rollback.txt"
    cat "$OUT"/real-small-3.out > "$OUT/real-threads.txt"
    local b j
    for b in 1 2 3; do for j in $(seq 1 10); do cat "$OUT/real-large-b$b-$j.out"; done; done > "$OUT/real-large.txt"
    for j in 1 2 3 4 5 6; do cat "$OUT/real-dense-$j.out"; done > "$OUT/real-dense.txt"
    python3 "$MODEL_DIR/tools/summarize_segments.py" "$OUT"/real-fast.txt "$OUT"/real-rollback.txt "$OUT"/real-threads.txt "$OUT"/real-large.txt "$OUT"/real-dense.txt > "$OUT/segments.txt"
}

# 一行变异：抽种子函数与四条用例 → 只重编探针包 → 跑四条用例与真历史上的撞种子（战役各档合起来一行、稠密一行）
f3_one() {
    if [ "$#" -eq 0 ]; then
        python3 "$MODEL_DIR/tools/gen_seedfn.py" "$FROZEN_CI" "$GEN_DIR/seedfn.rs" >/dev/null
    else
        python3 "$MODEL_DIR/tools/gen_seedfn.py" "$FROZEN_CI" "$GEN_DIR/seedfn.rs" "$1" "$2" >/dev/null
    fi
    cargo_in "$COPY" build --offline --release -p opus-b24-probe >>"$OUT/build-f3.log" 2>&1
    run f3-tests
    run f3-proposed
    run harm "$OUT/real-fast.txt" "$OUT/real-rollback.txt" "$OUT/real-threads.txt" "$OUT/real-large.txt"
    run harm "$OUT/real-dense.txt"
}
f3_begin() { : > "$OUT/build-f3.log"; f3_one > "$OUT/f3.txt"; }
# 一张表里行号落在 [from, to] 的 F3 行逐行跑（frozen = 冻结副本 mutations.tsv 的 F3-c（23-1）与 F3-d（24-1）行，opus = 攻方 M7 行）
f3_rows_from() {
    local tsv="$1" pattern="$2" from="$3" to="$4" line
    for line in $(grep -n -E "$pattern" "$tsv" | cut -d: -f1); do
        if [ "$line" -ge "$from" ] && [ "$line" -le "$to" ]; then f3_one "$tsv" "$line" >> "$OUT/f3.txt"; fi
    done
}
f3_rows() {
    f3_rows_from "$FROZEN_TSV" '^F3-c（23-1）|^F3-d（24-1）' 1 999999
    f3_rows_from "$OPUS_TSV" '^攻方 M7' 1 999999
}
# 收尾：探针包换回不施加变异的那一份
f3_end() {
    python3 "$MODEL_DIR/tools/gen_seedfn.py" "$FROZEN_CI" "$GEN_DIR/seedfn.rs" >/dev/null
    cargo_in "$COPY" build --offline --release -p opus-b24-probe >>"$OUT/build-f3.log" 2>&1
}

# B1：入库那组（覆盖写 3、2999 字节、单元 14、批 1）逐槽 112..208；再放开覆盖写次数 1、3、5，长度 100、2999、9000 字节，批数 1、2、3，
# 单元 4、8、12、14、16、20，单元区 112..240 槽（27 件，每批 10 件并行、各件单线程）。
B1_JOBS=()
for ow in 1 3 5; do for owb in 100 2999 9000; do for b in 1 2 3; do B1_JOBS+=("$ow $owb $b"); done; done; done
b1_batch() {
    local b="$1" j spec specs=()
    for j in $(seq 1 10); do
        spec="${B1_JOBS[$(( (b - 1) * 10 + j - 1 ))]:-}"
        [ -n "$spec" ] || break
        # shellcheck disable=SC2086
        set -- $spec
        specs+=("run b1-sweep 112 240 $1 $2 4,8,12,14,16,20 $3")
    done
    batch "b1w-b$b" "${specs[@]}"
}
b1_collect() {
    local b j
    for b in 1 2 3; do for j in $(seq 1 10); do
        if [ -f "$OUT/b1w-b$b-$j.out" ]; then cat "$OUT/b1w-b$b-$j.out"; fi
    done; done > "$OUT/b1-wide.txt"
    python3 "$MODEL_DIR/tools/summarize_b1.py" "$OUT/b1-slots.txt" "$OUT/b1-wide.txt" > "$OUT/b1-summary.txt"
}

harness() {
    (cd "$COPY" && OPUS_H2_MAX_K=40 nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo test --offline --release -p opus-b24-harness-probe --test h2_txg -- --nocapture) > "$OUT/h2-cargo.log" 2>&1
    grep '^h2 ' "$OUT/h2-cargo.log" > "$OUT/h2-txg.txt"
    (cd "$COPY" && nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo test --offline --release -p opus-b24-harness-probe --test c670_other_disk -- --nocapture --test-threads 1) > "$OUT/c670-cargo.log" 2>&1
    { grep -E '^test .* \.\.\. |^c670-other-disk ' "$OUT/c670-cargo.log"; grep '^test result' "$OUT/c670-cargo.log" | sed -E 's/; finished in [0-9.]+s$//'; } > "$OUT/c670.txt"
}

# H2 换 S：第二份副本只改格式常量 ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM，S = 5、15 跑 k = 1..40；S = 4、16 只编看编不编得过。
h2s_setup() {
    mkdir -p "$COPY2"
    rsync -a --delete --exclude target "$COPY/crates" "$COPY2/"
    cp "$COPY/Cargo.toml" "$COPY/Cargo.lock" "$COPY2/"
}
set_s() {
    python3 - "$COPY2/crates/singlefs-format/src/lib.rs" "$1" <<'PY'
import re, sys, os
p, s = sys.argv[1], sys.argv[2]
t = open(p, encoding="utf-8").read()
new, n = re.subn(r"pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = \d+;", f"pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = {s};", t)
assert n == 1, n
open(p + ".opus-new", "w", encoding="utf-8").write(new); os.replace(p + ".opus-new", p)
PY
}
h2s() {
    set_s "$1"
    (cd "$COPY2" && OPUS_H2_MAX_K=40 nice -n 19 bash "$CAP" "$BUILD_CAP" bash "$CAPPED" "$THREADS" cargo test --offline --release -p opus-b24-harness-probe --test h2_txg -- --nocapture) > "$OUT/h2-s$1-cargo.log" 2>&1
    grep '^h2 ' "$OUT/h2-s$1-cargo.log" > "$OUT/h2-txg-s$1.txt"
}
h2s_compile() {
    set_s "$1"
    local rc=0
    cargo_in "$COPY2" build --offline --release -p singlefs-harness > "$OUT/h2-s$1-build.log" 2>&1 || rc=$?
    { echo "S=$1 cargo_build_singlefs_harness_rc=$rc"; grep -E '^error(\[|:)' "$OUT/h2-s$1-build.log" | sort | uniq -c; grep -F 'mkfs 写的 S 碰到了区间的边' "$OUT/h2-s$1-build.log" | head -1; } > "$OUT/h2-compile-s$1.txt"
}

case "${1:-}" in
    setup) setup ;;
    build) build ;;
    real-small) real_small ;;
    real-large-batch) real_large_batch "$2" ;;
    real-dense-batch) real_dense_batch ;;
    real-collect) real_collect ;;
    f3-begin) f3_begin ;;
    f3-rows) f3_rows ;;
    f3-rows-from)
        if [ "$2" = frozen ]; then f3_rows_from "$FROZEN_TSV" '^F3-c（23-1）|^F3-d（24-1）' "$3" "$4"; else f3_rows_from "$OPUS_TSV" '^攻方 M7' "$3" "$4"; fi ;;
    f3-end) f3_end ;;
    b1-slots) run b1-sweep 112 208 3 2999 14 1 > "$OUT/b1-slots.txt" ;;
    b1-batch) b1_batch "$2" ;;
    b1-collect) b1_collect ;;
    harness) harness ;;
    h2s-setup) h2s_setup ;;
    h2s) h2s "$2" ;;
    h2s-compile) h2s_compile "$2" ;;
    all)
        setup; build; real_small; real_large_batch 1; real_large_batch 2; real_large_batch 3; real_dense_batch; real_collect
        f3_begin; f3_rows; f3_end
        run b1-sweep 112 208 3 2999 14 1 > "$OUT/b1-slots.txt"; b1_batch 1; b1_batch 2; b1_batch 3; b1_collect
        harness; h2s_setup; h2s 5; h2s 15; h2s_compile 4; h2s_compile 16 ;;
    *) echo "用法：$0 <段>（见文件头）" >&2; exit 2 ;;
esac
