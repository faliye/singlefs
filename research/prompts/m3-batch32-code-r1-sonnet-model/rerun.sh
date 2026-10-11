#!/usr/bin/env bash
# m3-batch32-code-r1 云端正推（Sonnet）的复跑：只跑 harness 档（singlefs-harness 的 checker_known_bad_images 测试二进制）与纯 Python 模型，
# 不碰 checker-tier 的编译与测试、不跑 GPU。
#
# 用法：rerun.sh <空工作目录> [阶段…]    阶段：prep anchors model build baseline probe i49 mutations broad extra all（默认 all）；整理见 compare.sh
# 环境变量：
#   SINGLEFS_REPO   必填。冻结副本根（~/.local/share/singlefs/work/m3-batch32-code-r1/tree：Cargo.toml、Cargo.lock、crates/、litmus/）
#   SINGLEFS_TOOLS  包装脚本所在仓根（要有 scripts/hosts/run-with-memory-cap.sh 与 capped.sh）；没给取本脚本往上三层
#   SONNET_THREADS  线程上限，默认 4（派发提示的「线程上限」）
#   SONNET_EXTRA_LINES      可选：extra 阶段跑 extra_mutations.tsv 的哪几行，默认 "1 2 3 4 5 6 7 8"（第 9、10 行是候选改法 F1，只在 i49 阶段用）
#   SONNET_MUTATION_LINES   可选：mutations 阶段跑变异表哪几行，默认 "3078 3079 3080 3081 3082 3083"；结果追加到 out/mutations.txt（阶段开头不清）
#   SONNET_STEP_SECONDS     可选：每条经内存包装的命令的挂钟上限（秒，设成 RUN_WITH_MEMORY_CAP_TIME_LIMIT）；没设不限。
#                  退出码 253 是它到点了：重跑同一阶段接着编（cargo 留着已编好的）。
# 内存：编译 6G、跑 4G，都显式给，经 scripts/hosts/run-with-memory-cap.sh；同一时刻只起一条 cargo；编译目录在 <工作目录>/target，不与别处共用。
# 一次调用里跑不完时（前台一次不超过 240 秒）把阶段分几次调用，顺序同 all：prep anchors model build baseline probe mutations extra；每个阶段都能单独重跑。
# 产出（<工作目录>/out/）：anchors.txt（变异表 12 行锚点命中次数与点名测试筛不筛得到）、model.txt（Python 模型三份的输出）、baseline.txt、probe.txt（PROBE 行）、
# mutations.txt 与 extra.txt（每行变异一行 RESULT）。
set -euo pipefail
WORK="${1:?用法：rerun.sh <空工作目录> [阶段…]}"; shift || true
STAGES=("$@"); [ ${#STAGES[@]} -eq 0 ] && STAGES=(all)
: "${SINGLEFS_REPO:?SINGLEFS_REPO 必填（冻结副本根）}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLS="${SINGLEFS_TOOLS:-$(cd "$HERE/../../.." && pwd)}"
THREADS="${SONNET_THREADS:-4}"
mkdir -p "$WORK/out"
WORK="$(cd "$WORK" && pwd)"
TREE="$WORK/tree"
export CARGO_TARGET_DIR="$WORK/target"
[ -n "${SONNET_STEP_SECONDS:-}" ] && export RUN_WITH_MEMORY_CAP_TIME_LIMIT="$SONNET_STEP_SECONDS"

capped_cargo() {  # 用法：capped_cargo <上限> cargo 参数…
  local cap="$1"; shift
  ( cd "$TREE" && bash "$TOOLS/scripts/hosts/run-with-memory-cap.sh" "$cap" \
      bash "$TOOLS/scripts/hosts/capped.sh" "$THREADS" nice -n 19 "$@" )
}
HARNESS_TEST=(-p singlefs-harness --test checker_known_bad_images)
NEW_TEST_FILTERS=(whichever_location_names_a_node_walked_before sixteen_kibibytes_beside_a_node over_a_child_whose_two_location_checksums_are_wrong
  goes_down_the_type_zero_branch_on_either_device does_not_go_down_that_copy below_a_level_one_node_whose verifying_as_a_node)

stage_prep() {
  if [ ! -f "$TREE/.prepared" ]; then
    rm -rf "$TREE"; mkdir -p "$TREE"
    cp -a "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$SINGLEFS_REPO/crates" "$SINGLEFS_REPO/litmus" "$TREE/"
    cp "$TREE/crates/singlefs-harness/tests/checker_known_bad_images.rs" "$WORK/checker_known_bad_images.rs.orig"
    cat "$HERE/probe_append.rs" >> "$TREE/crates/singlefs-harness/tests/checker_known_bad_images.rs"
    find "$TREE" -type f -exec touch {} +   # 拷贝保留了旧修改时刻，cargo 按时刻判新旧，不重设会沿用目录里上一次编出来的
    touch "$TREE/.prepared"
  fi
  echo "prep ok: $TREE"
}

stage_build() { stage_prep >/dev/null; capped_cargo 6G cargo test --offline "${HARNESS_TEST[@]}" --no-run; touch "$WORK/.built"; }

# 基线：这一批新加的 CPU 测试与相关的 F-W 一族（按子串筛）
stage_baseline() {
  capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- "${NEW_TEST_FILTERS[@]}" 2>&1 | tee "$WORK/out/baseline.raw" \
    | grep -E '^test |^test result' | sed -E 's/; finished in [0-9.]+s//' | LC_ALL=C sort > "$WORK/out/baseline.txt" || true
  echo "baseline 行数：$(wc -l < "$WORK/out/baseline.txt")"
}

# 探针三条（tomb lineage zero_slot），各跑各的；SONNET_PROBES 可选择其中几条；每条的 PROBE 行追加进 out/probe.txt（同一条重跑前先从里面删掉它的行）
stage_probe() {
  touch "$WORK/out/probe.txt"
  for which in ${SONNET_PROBES:-tomb lineage zero_slot crc}; do
    capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- --exact "probe_of_the_sonnet_leg_for_batch32_$which" --nocapture 2>&1 | tee "$WORK/out/probe-$which.raw" | grep -E '^PROBE ' >> "$WORK/out/probe.txt" || true
  done
  LC_ALL=C sort -u "$WORK/out/probe.txt" -o "$WORK/out/probe.txt.sorted" && mv "$WORK/out/probe.txt.sorted" "$WORK/out/probe.txt"
  echo "probe 行数：$(wc -l < "$WORK/out/probe.txt")"
}

# 变异：把锚点换成替换文、跑那一行点名的参数、看点名的测试红没红、列出红了哪几条，最后还原（逐字节核与备份相同）
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
    python3 "$HERE/apply_mutation.py" restore "$TREE" "$line" >/dev/null
    if grep -qE "^test .*${named} \.\.\. FAILED" "$WORK/out/mutation-$label-$line.raw"; then verdict=RED; else verdict=SURVIVES; fi
    failed_names="$(grep -E '^test .* \.\.\. FAILED' "$WORK/out/mutation-$label-$line.raw" | sed -E 's/^test ([^ ]+) .*/\1/' | LC_ALL=C sort | tr '\n' ' ' || true)"
    echo "RESULT $label line=$line named_test=$named verdict=$verdict cargo_rc=$rc ran=$(grep -cE '^test .* \.\.\. ' "$WORK/out/mutation-$label-$line.raw") failed=$(grep -cE '^test .* \.\.\. FAILED' "$WORK/out/mutation-$label-$line.raw") failed_tests=[ $failed_names]" >> "$out"
  done
}
stage_mutations() {
  # shellcheck disable=SC2086
  run_mutation_rows "$WORK/out/mutations.txt" table ${SONNET_MUTATION_LINES:-3078 3079 3080 3081 3082 3083}
}
stage_extra() {
  # shellcheck disable=SC2086
  SONNET_MUTATION_TABLE="$HERE/extra_mutations.tsv" run_mutation_rows "$WORK/out/extra.txt" extra ${SONNET_EXTRA_LINES:-1 2 3 4 5 6 7 8}
}

# I-4.9 那一形（P1）：F-KEY 三形两种盘序的红集与 I-4.9 说明；先在没改的副本上跑一次（i49-base），再套候选改法 F1（extra_mutations.tsv 第 9、10 行）跑一次（i49-f1），最后还原
stage_i49() {
  capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- --exact probe_of_the_sonnet_leg_for_batch32_i49 --nocapture 2>&1 | tee "$WORK/out/i49-base.raw" | grep -E '^PROBE i49 ' | sed 's/^PROBE i49 /PROBE i49-base /' | LC_ALL=C sort > "$WORK/out/i49.txt" || true
  for line in 9 10; do SONNET_MUTATION_TABLE="$HERE/extra_mutations.tsv" python3 "$HERE/apply_mutation.py" apply "$TREE" "$line" >/dev/null; done
  capped_cargo 4G cargo test --offline "${HARNESS_TEST[@]}" -- --exact probe_of_the_sonnet_leg_for_batch32_i49 --nocapture 2>&1 | tee "$WORK/out/i49-f1.raw" | grep -E '^PROBE i49 ' | sed 's/^PROBE i49 /PROBE i49-f1 /' | LC_ALL=C sort >> "$WORK/out/i49.txt" || true
  for line in 10 9; do SONNET_MUTATION_TABLE="$HERE/extra_mutations.tsv" python3 "$HERE/apply_mutation.py" restore "$TREE" "$line" >/dev/null; done
  echo "i49 行数：$(wc -l < "$WORK/out/i49.txt")"
}

# 变异表第 3078–3083 行六行换成宽筛（这一批 14 条新测试加 F-W 一族，同基线），看每条新测试各自被哪几行红掉（行自己的参数只筛六条）
stage_mutations_broad() {
  touch "$WORK/out/mutations-broad.txt"
  broad="-p singlefs-harness --test checker_known_bad_images -- ${NEW_TEST_FILTERS[*]}"
  for line in ${SONNET_MUTATION_LINES:-3078 3079 3080 3081 3082 3083}; do
    python3 "$HERE/apply_mutation.py" apply "$TREE" "$line" >/dev/null
    named="$(python3 "$HERE/apply_mutation.py" named "$TREE" "$line")"
    rc=0
    # shellcheck disable=SC2086
    capped_cargo 4G cargo test --offline $broad > "$WORK/out/mutation-broad-$line.raw" 2>&1 || rc=$?
    python3 "$HERE/apply_mutation.py" restore "$TREE" "$line" >/dev/null
    failed_names="$(grep -E '^test .* \.\.\. FAILED' "$WORK/out/mutation-broad-$line.raw" | sed -E 's/^test ([^ ]+) .*/\1/' | LC_ALL=C sort | tr '\n' ' ' || true)"
    echo "RESULT broad line=$line cargo_rc=$rc ran=$(grep -cE '^test .* \.\.\. ' "$WORK/out/mutation-broad-$line.raw") failed=$(grep -cE '^test .* \.\.\. FAILED' "$WORK/out/mutation-broad-$line.raw") failed_tests=[ $failed_names]" >> "$WORK/out/mutations-broad.txt"
  done
}

# 变异表第 3078–3089 行十二行的锚点：文件里恰好命中几次、点名的测试是不是 #[test] fn、参数里的子串各命中几个测试名、点名的测试在不在命中的里面
stage_anchors() {
  python3 "$HERE/apply_mutation.py" anchors "$SINGLEFS_REPO" 3078 3089 > "$WORK/out/anchors.txt"
  python3 "$HERE/gpu_named_tests_exist.py" "$SINGLEFS_REPO" 3084 3089 >> "$WORK/out/anchors.txt"
}

# 纯 Python 模型（不编译、不依赖仓路径之外的东西）：P2 两份副本两两扫的预期、F-零槽 read_at 的 WGSL 算术逐位移植
stage_model() {
  { python3 "$HERE/p2_matrix_model.py"; python3 "$HERE/wgsl_zero_unit_model.py"; python3 "$HERE/crc32c_check.py"; } > "$WORK/out/model.txt"
  echo "model 行数：$(wc -l < "$WORK/out/model.txt")"
}

for stage in "${STAGES[@]}"; do
  case "$stage" in
    all) stage_prep; stage_anchors; stage_model; stage_build; stage_baseline; stage_probe; stage_i49; stage_mutations; stage_mutations_broad; stage_extra ;;
    prep) stage_prep ;;
    build) stage_build ;;
    baseline) stage_baseline ;;
    probe) stage_probe ;;
    mutations) stage_mutations ;;
    i49) stage_i49 ;;
    broad) stage_mutations_broad ;;
    extra) stage_extra ;;
    anchors) stage_anchors ;;
    model) stage_model ;;
    *) echo "未知阶段 $stage" >&2; exit 2 ;;
  esac
done
