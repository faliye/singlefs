#!/usr/bin/env bash
# 攻方腿 m3-batch52-54-code-r1 opus 的复跑。仓（冻结副本）从 SINGLEFS_REPO 取，只拷 Cargo.toml、Cargo.lock 与 crates/。
# 用法：SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> <阶段>…
#   阶段：base-build base-k1 base-k2-numbers base-k2-random base-k2-product base-k4 base-k3
#         k5（钉值从原理重算，读第 52、54 批实现员的证据目录：OPUS_B52_EVIDENCE、OPUS_B54_EVIDENCE，缺省是那两份草稿目录）
#         mut:<名字>（名字见 mutants.py list）  all-mutants  compare
#   一个阶段一条前台命令；all 依次跑全部（挂钟约 25 分钟，前台超时的环境分阶段起）。
# 每份副本（base 与每一种变异）各用自己的 CARGO_TARGET_DIR；同一时刻只起一条 cargo，随机历史直接跑编出来的二进制（6 个进程并行）。
# 产物：<工作目录>/out/*.txt；compare 拿它们排序、去耗时字段之后与本目录 expected/ 逐份比。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:?要设 SINGLEFS_REPO（冻结副本或仓根）}"
SCRIPTS="${SINGLEFS_SCRIPTS:-$(cd "$MODEL/../../scripts" && pwd)}"
CAP="${OPUS_MEMORY_CAP:-10G}"
THREADS="${OPUS_THREADS:-12}"
WORK="${1:?要给工作目录}"
shift
mkdir -p "$WORK/out"
WORK="$(cd "$WORK" && pwd)"

prepare_copy() {
    local dir="$1"
    if [[ -e "$dir/.prepared" ]]; then return 0; fi
    mkdir -p "$dir/tmp"
    rsync -a "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
    rsync -a --exclude target "$REPO/crates" "$dir/"
    cp "$MODEL/probes/opus_probe_k2_k4.rs" "$dir/crates/singlefs-harness/tests/opus_probe_k2_k4.rs"
    cat "$MODEL/probes/corrupt_on_disk_k1_append.rs" >> "$dir/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs"
    : > "$dir/.prepared"
}

# cargo test，经内存包装与线程上限；交回退出码，日志进 $4。
run_cargo() {
    local dir="$1" args="$2" filter="$3" log="$4"
    local rc=0
    # 只有点名攻方原型（opus_probe…，一次只跑一条）时才 --nocapture：整份用例并行跑时打印会与「test … ok」行交错，判定行被拆开。
    local capture=""
    [[ "$filter" == opus_probe* ]] && capture="--nocapture"
    # shellcheck disable=SC2086
    (cd "$dir" && CARGO_TARGET_DIR="$dir/target" TMPDIR="$dir/tmp" nice -n 19 bash "$SCRIPTS/run-with-memory-cap.sh" "$CAP" \
        bash "$SCRIPTS/capped.sh" "$THREADS" cargo test --offline $args -- $filter $capture --test-threads 4) > "$log" 2>&1 || rc=$?
    echo "$rc"
}

# 一份日志里与耗时无关的判定行：OPUS_ 打印行（去掉 elapsed_seconds）、test result 行（去掉 finished in）、失败的用例名。
summarize() {
    local log="$1"
    grep -h '^OPUS_' "$log" | sed -E 's/ elapsed_seconds=[0-9.]+//' || true
    grep -h '^test result:' "$log" | sed -E 's/; finished in [0-9.]+s//' || true
    grep -hE '^test .* \.\.\. FAILED$' "$log" | sort || true
}

base_dir="$WORK/base"
case_one() {
    local phase="$1"
    case "$phase" in
    base-build)
        prepare_copy "$base_dir"
        rc=$(run_cargo "$base_dir" "--release -p singlefs-harness --test opus_probe_k2_k4 --test corrupt_on_disk_content_is_refused_instead_of_panicking --no-run" "" "$WORK/base-build.log")
        echo "base-build rc=$rc"; [[ "$rc" == 0 ]] ;;
    base-k1)
        rc=$(run_cargo "$base_dir" "--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking" "opus_probe" "$WORK/base-k1.log")
        { echo "rc=$rc"; summarize "$WORK/base-k1.log"; } > "$WORK/out/base-k1.txt"; cat "$WORK/out/base-k1.txt" ;;
    base-k2-numbers)
        rc=$(run_cargo "$base_dir" "--release -p singlefs-harness --test opus_probe_k2_k4" "opus_probe_k2_container_numbers_under_the_knobs" "$WORK/base-k2-numbers.log")
        { echo "rc=$rc"; summarize "$WORK/base-k2-numbers.log"; } > "$WORK/out/base-k2-numbers.txt"; cat "$WORK/out/base-k2-numbers.txt" ;;
    base-k2-random)
        binary=$(grep -o '([^)]*release/deps/opus_probe_k2_k4-[0-9a-f]*)' "$WORK/base-build.log" | tr -d '()' | head -n 1)
        [[ "$binary" == /* ]] || binary="$base_dir/$binary"
        [[ -x "$binary" ]] || { echo "没找到随机历史的二进制：$binary"; exit 1; }
        rm -f "$WORK"/random-b1-*.rc
        for piece in 1 2 3 4 5 6; do
            { OPUS_K2_SEED_FIRST=$((2 * piece - 1)) OPUS_K2_SEED_LAST=$((2 * piece)) OPUS_K2_OPS=150 nice -n 19 \
                bash "$SCRIPTS/run-with-memory-cap.sh" 4G "$binary" opus_probe_k2_random_knob_histories --exact --nocapture \
                > "$WORK/random-b1-$piece.log" 2>&1; echo "$?" > "$WORK/random-b1-$piece.rc"; } &
        done
        wait
        count=$(ls "$WORK"/random-b1-*.rc | wc -l)
        [[ "$count" == 6 ]] || { echo "random 批 1：.rc 有 $count 份，不是 6，整批作废"; exit 1; }
        {
            for piece in 1 2 3 4 5 6; do echo "piece=$piece rc=$(cat "$WORK/random-b1-$piece.rc")"; summarize "$WORK/random-b1-$piece.log"; done
        } > "$WORK/out/base-k2-random.txt"
        grep -c 'violating_checks=0' "$WORK/out/base-k2-random.txt" ;;
    base-k2-product)
        rc=$(run_cargo "$base_dir" "--release -p singlefs-harness --test opus_probe_k2_k4" "opus_probe_k2_product_boundary --include-ignored" "$WORK/base-k2-product.log")
        { echo "rc=$rc"; summarize "$WORK/base-k2-product.log"; } > "$WORK/out/base-k2-product.txt"; cat "$WORK/out/base-k2-product.txt" ;;
    base-k4)
        rc=$(export OPUS_K4_SEED_FIRST=1 OPUS_K4_SEED_LAST=12; run_cargo "$base_dir" "--release -p singlefs-harness --test opus_probe_k2_k4" "opus_probe_k4_shapes_reached_by_the_reference_history" "$WORK/base-k4.log")
        { echo "rc=$rc"; summarize "$WORK/base-k4.log"; } > "$WORK/out/base-k4.txt"; cat "$WORK/out/base-k4.txt" ;;
    base-k3)
        rc=$(run_cargo "$base_dir" "--release -p singlefs-harness --test opus_probe_k2_k4" "opus_probe_k3_rollback_history_digest" "$WORK/base-k3.log")
        { echo "rc=$rc"; summarize "$WORK/base-k3.log"; } > "$WORK/out/base-k3.txt"; cat "$WORK/out/base-k3.txt" ;;
    k5)
        python3 -I "$MODEL/k5.py" "$REPO" "${OPUS_B52_EVIDENCE:-$HOME/.local/share/singlefs/work/m3-batch52/fix-reds/evidence}" \
            "${OPUS_B54_EVIDENCE:-$HOME/.local/share/singlefs/work/m3-batch54/repin}" > "$WORK/out/k5.txt"
        cat "$WORK/out/k5.txt" ;;
    mut:*)
        name="${phase#mut:}"
        dir="$WORK/mut-$name"
        prepare_copy "$dir"
        if [[ ! -e "$dir/.mutated" ]]; then python3 -I "$MODEL/mutants.py" apply "$name" "$dir"; : > "$dir/.mutated"; fi
        index=0
        {
            while IFS=$'\t' read -r args filter; do
                index=$((index + 1))
                rc=$(run_cargo "$dir" "$args" "$filter" "$WORK/mut-$name-$index.log")
                echo "command=$index args=$args filter=$filter rc=$rc"
                summarize "$WORK/mut-$name-$index.log"
            done < <(python3 -I "$MODEL/mutants.py" tests "$name")
        } > "$WORK/out/mut-$name.txt"
        cat "$WORK/out/mut-$name.txt" ;;
    all-mutants)
        for name in $(python3 -I "$MODEL/mutants.py" list); do case_one "mut:$name"; done ;;
    compare)
        python3 -I "$MODEL/compare.py" "$MODEL/expected" "$WORK/out" ;;
    all)
        for each in base-build base-k1 base-k2-numbers base-k2-random base-k2-product base-k4 base-k3 k5 all-mutants compare; do case_one "$each"; done ;;
    *) echo "不认得的阶段 $phase" >&2; exit 2 ;;
    esac
}

for phase in "$@"; do case_one "$phase"; done
