#!/usr/bin/env bash
# m3-batch31-code-r1 云端正推（Sonnet）的复跑：只跑 harness 档（singlefs-harness 的 checker_known_bad_images 测试二进制），不碰 checker-tier、不跑 GPU。
#
# 用法：rerun.sh <空工作目录> [阶段…]    阶段：prep build baseline probe mutations extra anchors all（默认 all）；compare 见 compare.sh
# 环境变量：
#   SINGLEFS_REPO   必填。冻结副本根（~/.local/share/singlefs/work/m3-batch31-code-r1/tree：Cargo.toml、Cargo.lock、crates/、litmus/）
#   SINGLEFS_TOOLS  包装脚本所在仓根（要有 scripts/run-with-memory-cap.sh 与 capped.sh）；没给取本脚本往上四层
#   SONNET_THREADS  线程上限，默认 10（派发提示的「线程上限」）
#   SONNET_EXTRA_LINES  可选：extra 阶段跑 extra_mutations.tsv 的哪几行，默认 "1 2 3 4"
#   SONNET_MUTATION_LINES  可选：mutations 阶段只跑哪几行（空格分隔的变异表行号），默认 "3054 3055 3056 3057"；结果追加到 out/mutations.txt（阶段开头不清）
#   SONNET_STEP_SECONDS  可选：每一条经内存包装的命令的挂钟上限（秒，设成 RUN_WITH_MEMORY_CAP_TIME_LIMIT）；没设不限。
#                  退出码 253 是它到点了：重跑同一阶段接着编（cargo 留着已编好的）。
# 内存：编译 6G、跑 4G，都显式给，经 scripts/run-with-memory-cap.sh；同一时刻只起一条 cargo；编译目录在 <工作目录>/target，不与别处共用。
# 一次调用里跑不完时（前台一次不超过 240 秒的环境）把阶段分几次调用，顺序同 all：prep anchors build baseline probe mutations extra；每个阶段都能单独重跑。
# 产出（<工作目录>/out/）：probe.txt（PROBE 行）、baseline.txt、mutations.txt 与 extra.txt（每行变异一行 RESULT）、extra-probe.txt（变异在时探针 G 那几行）、debug.txt（D1：层级 ≤ 1 那一处每份副本比得上没有）、anchors.txt（变异表 12 行的锚点命中次数）。
set -euo pipefail
WORK="${1:?用法：rerun.sh <空工作目录> [阶段…]}"; shift || true
STAGES=("$@"); [ ${#STAGES[@]} -eq 0 ] && STAGES=(all)
: "${SINGLEFS_REPO:?SINGLEFS_REPO 必填（冻结副本根）}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLS="${SINGLEFS_TOOLS:-$(cd "$HERE/../../.." && pwd)}"
THREADS="${SONNET_THREADS:-10}"
mkdir -p "$WORK/out"
WORK="$(cd "$WORK" && pwd)"
TREE="$WORK/tree"
export CARGO_TARGET_DIR="$WORK/target"
[ -n "${SONNET_STEP_SECONDS:-}" ] && export RUN_WITH_MEMORY_CAP_TIME_LIMIT="$SONNET_STEP_SECONDS"

capped_cargo() {  # 用法：capped_cargo <上限> cargo 参数…
  local cap="$1"; shift
  ( cd "$TREE" && bash "$TOOLS/scripts/run-with-memory-cap.sh" "$cap" \
      bash "$TOOLS/scripts/capped.sh" "$THREADS" nice -n 19 "$@" )
}
HARNESS_TEST=(-p singlefs-harness --test checker_known_bad_images)

stage_prep() {
  if [ ! -f "$TREE/.prepared" ]; then
    rm -rf "$TREE"; mkdir -p "$TREE"
    cp -a "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$SINGLEFS_REPO/crates" "$SINGLEFS_REPO/litmus" "$TREE/"
    cp "$TREE/crates/singlefs-harness/tests/checker_known_bad_images.rs" "$WORK/checker_known_bad_images.rs.orig"
    cat "$HERE/probe_append.rs" >> "$TREE/crates/singlefs-harness/tests/checker_known_bad_images.rs"
    find "$TREE" -type f -exec touch {} +   # 拷贝保留了冻结副本的旧修改时刻，cargo 按时刻判新旧，不重设会沿用目录里上一次（可能是变异版）编出来的
    touch "$TREE/.prepared"
  fi
  echo "prep ok: $TREE"
}

stage_build() {
  capped_cargo 6G cargo test --offline "${HARNESS_TEST[@]}" --no-run
  touch "$WORK/.built"
}

# 基线：F-L0 两条、X3、X4、X5 共 5 条加探针一条
stage_baseline() {
  : > "$WORK/out/baseline.txt"
  capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- \
      of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong \
      first_location_checksum_matches_only_its_first_sixteen_kibibytes \
      of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes \
      over_a_child_whose_second_copy_does_not_read_back 2>&1 | tee "$WORK/out/baseline.raw" | grep -E '^test |^test result' >> "$WORK/out/baseline.txt" || true
}

stage_probe() {
  capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- --exact probe_of_the_sonnet_leg_for_batch31 --nocapture 2>&1 | tee "$WORK/out/probe.raw" | grep -E '^PROBE ' > "$WORK/out/probe.txt" || true
  echo "probe 行数：$(wc -l < "$WORK/out/probe.txt")"
}

# 变异表第 3054–3057 行（CPU 四行）与本腿自己加的三行（extra_mutations.tsv 第 1–3 行）：在 <工作目录>/tree 里把锚点换成替换文、跑那一行点名的参数、
# 看点名的测试红没红、再列出红了哪几条，最后还原（逐字节核与备份相同）
run_mutation_rows() {  # 用法：run_mutation_rows <结果文件> <标签> <行号…>；表由 SONNET_MUTATION_TABLE 定（没设取工作副本里的变异表）
  local out="$1" label="$2"; shift 2
  touch "$out"
  for line in "$@"; do
    python3 "$HERE/apply_mutation.py" apply "$TREE" "$line" >/dev/null
    args="$(python3 "$HERE/apply_mutation.py" args "$TREE" "$line")"
    named="$(python3 "$HERE/apply_mutation.py" named "$TREE" "$line")"
    rc=0
    # shellcheck disable=SC2086
    capped_cargo 4G cargo test --offline $args > "$WORK/out/mutation-$label-$line.raw" 2>&1 || rc=$?
    if [ "$label" = extra ] && [ "$line" = 4 ]; then
      # D1：只留 DBG 与 PROBE 行（按出现的先后，不排序）：DBG 行紧挨在它那一份探针的 PROBE 行之前
      grep -E '^(DBG|PROBE [A-G]-)' "$WORK/out/mutation-$label-$line.raw" | sed -E 's/^(PROBE [A-G][^ ]*) .*/\1/' > "$WORK/out/debug.txt" || true
    elif [ "$label" = extra ]; then
      # 变异还在时再跑一遍探针（同一份编译产物），只留 G 那几行：层级 0 那一形有没有 I-2.1
      capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- --exact probe_of_the_sonnet_leg_for_batch31 --nocapture 2>&1 \
        | grep -E '^PROBE G-tomb-root' | sed -E "s/^PROBE /PROBE-UNDER-EXTRA-$line /" >> "$WORK/out/extra-probe.txt" || true
    fi
    python3 "$HERE/apply_mutation.py" restore "$TREE" "$line" >/dev/null
    if grep -qE "^test .*${named} \.\.\. FAILED" "$WORK/out/mutation-$label-$line.raw"; then verdict=RED; else verdict=SURVIVES; fi
    failed_names="$(grep -E '^test .* \.\.\. FAILED' "$WORK/out/mutation-$label-$line.raw" | sed -E 's/^test ([^ ]+) .*/\1/' | LC_ALL=C sort | tr '\n' ' ' || true)"
    echo "RESULT $label line=$line named_test=$named verdict=$verdict cargo_rc=$rc ran=$(grep -cE '^test .* \.\.\. ' "$WORK/out/mutation-$label-$line.raw") failed=$(grep -cE '^test .* \.\.\. FAILED' "$WORK/out/mutation-$label-$line.raw") failed_tests=[ $failed_names]" >> "$out"
  done
}
stage_mutations() {
  # shellcheck disable=SC2086
  run_mutation_rows "$WORK/out/mutations.txt" table ${SONNET_MUTATION_LINES:-3054 3055 3056 3057}
}
stage_extra() {
  touch "$WORK/out/extra-probe.txt"
  # shellcheck disable=SC2086
  SONNET_MUTATION_TABLE="$HERE/extra_mutations.tsv" run_mutation_rows "$WORK/out/extra.txt" extra ${SONNET_EXTRA_LINES:-1 2 3 4}
}

# 变异表第 3054–3065 行十二行的锚点：文件里恰好命中几次、点名的测试名是不是参数里某个子串的超串、参数子串在测试源码里各命中几个测试名
stage_anchors() {
  python3 "$HERE/apply_mutation.py" anchors "$SINGLEFS_REPO" 3054 3065 > "$WORK/out/anchors.txt"
}

for stage in "${STAGES[@]}"; do
  case "$stage" in
    all) stage_prep; stage_anchors; stage_build; stage_baseline; stage_probe; stage_mutations; stage_extra ;;
    prep) stage_prep ;;
    build) stage_prep; stage_build ;;
    baseline) stage_baseline ;;
    probe) stage_probe ;;
    mutations) stage_mutations ;;
    extra) stage_extra ;;
    anchors) stage_anchors ;;
    *) echo "未知阶段 $stage" >&2; exit 2 ;;
  esac
done
