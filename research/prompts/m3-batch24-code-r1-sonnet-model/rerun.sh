#!/usr/bin/env bash
# 正推腿 m3-batch24-code-r1 的复跑：第 24 批 K / G 合入的代码，harness 档与纯算术。
#   anchors  变异表第 24 批 27 行的锚点与全表锚点命中数（不编译）
#   tests    第 24 批新增的 11 + 6 条 harness 用例、reuse_after_raising_the_floor 全 16 条（--include-ignored）、第 23 批已有的 I-9.29 部分没判那一条
#   mut      第 24 批 K 的 19 行 harness 档变异逐条施加、跑点名的测试、还原，记红在哪条断言
#   matrix   四种树 × 条目类型段 × 孩子头里的类标签，CPU 池级 checker 实际判什么 vs 条款推出的该判什么
#   reasons  合法历史上新九条有没有落到缺省理由上
#   arith    E142 导出 bin 的拒绝判据整数算术复算（bin 本身属 checker 档，不跑）
#   summary  合成 summary.txt，与本目录 expected-summary.txt 逐字比
# 用法：rerun.sh <空目录> [步名 …]；不给步名 = 全部按序跑完再 summary。前台一次放不下时分几次给步名，最后单独给 summary。
# 环境：SINGLEFS_REPO（仓路径：取 run-with-memory-cap.sh、capped.sh 与快照 sha256 清单）；FROZEN_TREE（冻结副本，默认 /tmp/claude-1000/m3-batch24-code-r1/tree）；THREADS（默认 10）。
#   编译 6G、跑 4G，都经 run-with-memory-cap.sh（显式上限，不写 auto）；编译目录是 <目录>/target。
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
dir="$(cd "$1" && pwd)"; shift
frozen="${FROZEN_TREE:-/tmp/claude-1000/m3-batch24-code-r1/tree}"
threads="${THREADS:-10}"
mkdir -p "$dir/out"
if [ ! -d "$dir/work" ]; then
  mkdir "$dir/work" && cp -r "$frozen/." "$dir/work/" || exit 3
  # 探针 = checker_known_bad_images.rs 去掉 `//!` 行的副本 + 探针尾巴（借它的搭建函数；include! 不收内层文档注释）
  (grep -v '^//!' "$dir/work/crates/singlefs-harness/tests/checker_known_bad_images.rs"; cat "$here/probes/zs_probe_tail.rs" "$here/probes/zs_probe_tail2.rs") > "$dir/work/crates/singlefs-harness/tests/zs_probe.rs" || exit 3
  (cd "$dir/work" && sha256sum -c "$SINGLEFS_REPO/research/prompts/m3-batch24-code-r1-snapshot/crates-sha256.txt" 2>&1 | grep -c ': OK$' | sed 's/^/SHA256_OK_OF_10 /') > "$dir/out/sha256.txt"
fi
export CARGO_TARGET_DIR="$dir/target"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-batch24-code-r1 正推腿在自己的仓副本与编译目录里复跑，不与别的会话共用副本"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示要求编译 6G 跑 4G 显式写上限，不用 auto"
cap="nice -n 19 bash $SINGLEFS_REPO/scripts/hosts/run-with-memory-cap.sh"
capped="bash $SINGLEFS_REPO/scripts/hosts/capped.sh $threads"
build_prefix="$cap 6G $capped"
run_prefix="$cap 4G $capped"
strip_times() { sed -E 's/; finished in [0-9.]+s//'; }

new_known_bad=(
  a_tombstone_tree_entry_of_type_zero_over_a_container_reddens_only_the_tombstone_entry_invariant
  a_tombstone_tree_entry_of_type_zero_with_a_zero_identity_over_a_container_reddens_only_the_tombstone_entry_invariant
  a_lineage_tree_entry_of_type_zero_over_a_container_reddens_only_the_lineage_entry_invariant
  a_lineage_tree_entry_with_an_unregistered_type_reddens_only_the_lineage_entry_invariant
  an_inode_tree_entry_of_type_zero_over_a_leaf_container_reddens_only_the_inode_entry_invariant
  an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant
  a_tombstone_tree_entry_of_the_container_type_over_a_node_reddens_only_the_tombstone_entry_invariant
  a_tombstone_tree_entry_of_the_container_type_over_a_node_after_the_root_ring_turned_over_reddens_only_the_tombstone_entry_invariant
  a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant
  two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant
  an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type
)
new_pool_checker=(
  a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_reddens_the_one_birth_per_number_invariant
  a_tombstone_that_died_after_every_version_reddens_only_the_tombstone_record_invariant
  a_leaf_holding_two_inode_numbers_in_descending_order_reddens_only_the_container_number_invariant
  a_queued_inode_whose_tombstone_names_another_object_birth_reddens_the_queued_tombstone_invariant
  a_queued_inode_whose_tombstone_does_not_cover_the_whole_object_reddens_the_queued_tombstone_invariant
  a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant
)
# 一步里跑一个测试二进制、把 `test … ok` 行排序后写出（并行测试的行序不定），再写 test result 行
run_tests() {  # <二进制> <额外参数…> -- <测试名…>
  local binary="$1"; shift
  local extra=(); while [ "$1" != "--" ]; do extra+=("$1"); shift; done; shift
  ( cd "$dir/work"
    $build_prefix cargo test --offline -p singlefs-harness --test "$binary" --no-run > "$dir/out/$binary.build.log" 2>&1; echo "build exit=$?"
    $run_prefix cargo test --offline -p singlefs-harness --test "$binary" -- "${extra[@]}" "$@" > "$dir/out/$binary.run.log" 2>&1; echo "run exit=$?"
    # 并行测试的输出会穿插（有的行 `... ` 后面的 ok 落到下一行），所以只列测试名并排序，成败看下面的 test result 行
    grep -E '^test .* \.\.\. ' "$dir/out/$binary.run.log" | sed -E 's/^(test [^ ]+) \.\.\. .*/\1/' | sort
    grep -E '^test result' "$dir/out/$binary.run.log" | strip_times )
}

all_steps=(anchors tests mut matrix reasons arith)
steps=("$@")
if [ ${#steps[@]} -eq 0 ]; then steps=("${all_steps[@]}" summary); fi
for step in "${steps[@]}"; do
  case "$step" in
    anchors) python3 "$here/anchors.py" "$dir/work" > "$dir/out/anchors.txt" 2>&1; echo "exit=$?" >> "$dir/out/anchors.txt" ;;
    tests)
      { run_tests checker_known_bad_images --exact -- "${new_known_bad[@]}"
        run_tests pool_checker_judges_the_deletion_state_of_every_walked_version --exact -- "${new_pool_checker[@]}"
        run_tests reuse_after_raising_the_floor --include-ignored --
        # 第 23 批已有的一条：CPU 在「最新根的实例表读不出」那一形上报 I-9.29 部分没判（第 24 批 GPU 对拍新用例的 CPU 一侧预设）
        run_tests checker_known_bad_images --exact -- an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back ; } > "$dir/out/tests.txt" 2>&1 ;;
    mut)
      MUTATE_CMD_PREFIX="$build_prefix" python3 "$here/mutate_rows.py" "$dir/work" 0 19 > "$dir/out/mut.txt" 2>&1 ;;
    matrix)
      ( cd "$dir/work"
        $build_prefix cargo test --offline -p singlefs-harness --test zs_probe --no-run > "$dir/out/zs_probe.build.log" 2>&1; echo "build exit=$?"
        $run_prefix cargo test --offline -p singlefs-harness --test zs_probe -- zs_matrix --nocapture --test-threads=1 > "$dir/out/matrix.run.log" 2>&1; echo "run exit=$?"
        grep -E '^ZS_FIXTURE|^ZS_CPU|^ZS_SUMMARY' "$dir/out/matrix.run.log" | sed -E 's/^test [a-z_]+ \.\.\. //' ) > "$dir/out/matrix.txt" 2>&1 ;;
    reasons)
      ( cd "$dir/work"
        $run_prefix cargo test --offline -p singlefs-harness --test zs_probe -- zs_new_nine --nocapture --test-threads=1 > "$dir/out/reasons.run.log" 2>&1; echo "run exit=$?"
        grep -E 'ZS_RED|ZS_REASON' "$dir/out/reasons.run.log" | sed -E 's/^test [a-z_]+ \.\.\. //' ) > "$dir/out/reasons.txt" 2>&1 ;;
    arith) python3 "$here/bin_arith.py" > "$dir/out/arith.txt" 2>&1 ;;
    summary)
      {
        for f in sha256 anchors tests mut matrix reasons arith; do
          if [ -f "$dir/out/$f.txt" ]; then echo "== $f"; cat "$dir/out/$f.txt"; else echo "== $f MISSING"; fi
        done
      } > "$dir/summary.txt"
      if cmp -s "$dir/summary.txt" "$here/expected-summary.txt"; then echo "SUMMARY_MATCHES lines=$(wc -l < "$dir/summary.txt")"; else echo "SUMMARY_DIFFERS"; diff "$dir/summary.txt" "$here/expected-summary.txt" | head -40; fi ;;
    *) echo "不认识的步名 $step"; exit 2 ;;
  esac
  echo "step $step done"
done
