#!/usr/bin/env bash
# 第 21 批代码轮第一轮攻方腿（Opus）复跑：冻结树拷四份（原样 tree、N7 错实现 c2cap32、N5 错实现 w1 / w2），编原型与探针用例，逐步跑，出摘要与 expected-summary.txt 逐字比。
#   SINGLEFS_REPO=<仓根，必给> FROZEN_TREE=<冻结树，缺省 /tmp/claude-1000/m3-batch21-code-r1/tree> bash rerun.sh <工作目录> <步骤> [参数…]
# 步骤：prepare | build <tree|c2cap32|w1|w2> | n7-tests | n7-probe | n7-fix | n5-tests | n5-fix | n5-probe <tree|w1|w2> <K 列表，如 0,1,2,3>
#       | n5-w2-targets | sweep | shapes | badimages | count | summary | compare | all
# 每一步一条前台命令跑得完（单步约 4 分钟以内）；all 依次跑全部（约 25 分钟，给主 agent 放后台用）。compare = summary 之后与 expected-summary.txt 逐字比。
# 编译经 research/scripts/run-with-memory-cap.sh 6G、跑 4G，线程经 research/scripts/capped.sh（M3B21_CAP_THREADS，缺省 10）。只跑 harness 档：
# 不起 checker 档的测试目标与 bin（原型只把 singlefs-checker-tier 当库调，crash 模式在全量状态数超过 10^6 时不跑，这里只跑 count）。
set -euo pipefail
model_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="${SINGLEFS_REPO:?SINGLEFS_REPO 要给仓根}"
frozen_tree="${FROZEN_TREE:-/tmp/claude-1000/m3-batch21-code-r1/tree}"
work="${1:?工作目录}"
step="${2:?步骤}"
shift 2
threads="${M3B21_CAP_THREADS:-10}"
cap="$repo/research/scripts/run-with-memory-cap.sh"
capped="$repo/research/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-第21批代码轮攻方腿原型复跑，与同机别的活互不依赖}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-新编译目录从头编、探针用例与峰值表记的不是同一次}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-180}"
unlink_writes=crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
crashed_unmount=crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs

cargo_in() {
  local variant="$1" cap_size="$2"
  shift 2
  (cd "$work/$variant" && CARGO_TARGET_DIR="$work/target-$variant" nice -n 19 bash "$capped" "$threads" bash "$cap" "$cap_size" cargo "$@")
}
test_in() {
  local variant="$1" output="$2"
  shift 2
  cargo_in "$variant" 4G test --release --offline -p "$@" > "$work/out/$output.out" 2>&1 || echo "exit=$?" >> "$work/out/$output.out"
}
proto() {
  M3B21_THREADS="$threads" nice -n 19 bash "$cap" 4G "$work/target-tree/release/m3b21-opus-proto" "$@"
}

case "$step" in
  prepare)
    [[ -z "$(ls -A "$work" 2>/dev/null)" ]] || { echo "工作目录要是空的：$work" >&2; exit 2; }
    (cd "$frozen_tree" && sha256sum --quiet -c "$repo/research/prompts/m3-batch21-code-r1-snapshot/crates-sha256.txt")
    mkdir -p "$work/out"
    for variant in tree c2cap32 w1 w2; do
      rsync -a --exclude target --exclude .git "$frozen_tree/" "$work/$variant/"
    done
    (cd "$work/tree" && patch -s -p1 < "$model_dir/patches/workspace.patch")
    mkdir -p "$work/tree/crates/m3b21-opus-proto"
    cp -r "$model_dir/proto/Cargo.toml" "$model_dir/proto/src" "$work/tree/crates/m3b21-opus-proto/"
    (cd "$work/c2cap32" && patch -s -p1 < "$model_dir/patches/c2cap32.patch")
    (cd "$work/w1" && patch -s -p1 < "$model_dir/patches/w1.patch")
    (cd "$work/w2" && patch -s -p1 < "$model_dir/patches/w2.patch")
    for variant in tree c2cap32; do
      cat "$frozen_tree/$unlink_writes" "$model_dir/probes/probe_n7_tail.rs" > "$work/$variant/crates/singlefs-harness/tests/n7_probe_unlink_batch_points.rs"
      cp "$model_dir/probes/n7_fix_test.rs" "$work/$variant/crates/singlefs-harness/tests/n7_fix_product_batch_on_large_files.rs"
    done
    for variant in tree w1 w2; do
      cat "$frozen_tree/$crashed_unmount" "$model_dir/probes/probe_n5_tail.rs" "$model_dir/probes/probe_n5_fix_tail.rs" > "$work/$variant/crates/singlefs-harness/tests/n5_probe_crashed_unmount_sweep.rs"
    done
    echo "prepared $work"
    ;;
  build)
    variant="${1:?tree|c2cap32|w1|w2}"
    case "$variant" in
      tree) cargo_in tree 6G build -q --release --offline -p m3b21-opus-proto
            cargo_in tree 6G test -q --release --offline -p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file --test n7_probe_unlink_batch_points --test n7_fix_product_batch_on_large_files --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads --test n5_probe_crashed_unmount_sweep --test rollback_by_a_forward_publish --no-run ;;
      c2cap32) cargo_in c2cap32 6G test -q --release --offline -p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file --test n7_probe_unlink_batch_points --test n7_fix_product_batch_on_large_files --test unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches --test container_trees_grow_a_level_when_the_root_is_full --no-run ;;
      w1|w2) cargo_in "$variant" 6G test -q --release --offline -p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads --test n5_probe_crashed_unmount_sweep --no-run ;;
      *) echo "不认得的变体：$variant" >&2; exit 2 ;;
    esac
    echo "built $variant"
    ;;
  n7-tests)
    test_in tree n7-tree-tests singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file -- --include-ignored
    test_in c2cap32 n7-c2cap32-tests singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file -- --include-ignored
    test_in c2cap32 n7-c2cap32-core-lib singlefs-core --lib unlink
    test_in c2cap32 n7-c2cap32-other-targets singlefs-harness --test unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches --test container_trees_grow_a_level_when_the_root_is_full -- --include-ignored
    ;;
  n7-probe)
    for variant in tree c2cap32; do
      N7_PROBE_SIZES=4609,9216 test_in "$variant" "n7-probe-$variant" singlefs-harness --test n7_probe_unlink_batch_points -- --include-ignored --exact probe_points_outside_the_sampled_sizes --nocapture
    done
    ;;
  n7-fix)
    for variant in tree c2cap32; do test_in "$variant" "n7-fix-g7-$variant" singlefs-harness --test n7_fix_product_batch_on_large_files; done
    ;;
  n5-tests)
    for variant in tree w1 w2; do test_in "$variant" "n5-tests-$variant" singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- --include-ignored; done
    ;;
  n5-fix)
    for variant in tree w1 w2; do test_in "$variant" "n5-fix-g5-$variant" singlefs-harness --test n5_probe_crashed_unmount_sweep -- --include-ignored --exact fix_g5_every_version_after_the_newest_root_on_device_one_released_slots_stay_untouched_in_either_warm_up --nocapture; done
    ;;
  n5-probe)
    variant="${1:?tree|w1|w2}"; ks="${2:?K 列表}"
    N5_K="$ks" test_in "$variant" "n5-probe-$variant-k${ks//,/}" singlefs-harness --test n5_probe_crashed_unmount_sweep -- --include-ignored --exact probe_unmount_crash_points_and_overwrite_counts --nocapture
    ;;
  n5-w2-targets)
    targets=()
    for target in a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write checker_judges_reference_intervals_and_every_new_root_segment_field admission_raises_the_floor_before_refusing model_answers_unmount_with_the_floor_at_the_current_version instance_table_sweep_at_the_end_of_a_writable_mount pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window mount_transparency_judged_field_by_field_after_a_writable_mount rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read writable_mount_of_a_formatted_pool a_lone_surviving_device_reads_back_what_it_read_before_the_unmount entries_after_mount_refuse_swapped_or_behind_devices entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration unmount_marker_of_a_root_rebuilt_from_the_records_of_the_unmount_sequence rebuilt_root_takes_every_new_root_segment_field_from_the_record rollback_by_a_forward_publish checker_known_bad_images; do
      targets+=(--test "$target")
    done
    test_in w2 n5-w2-unmount-targets singlefs-harness "${targets[@]}"
    test_in w2 n5-w2-ignored singlefs-harness --test admission_raises_the_floor_before_refusing --test rollback_by_a_forward_publish -- --ignored
    test_in tree n5-tree-ignored-rollback singlefs-harness --test rollback_by_a_forward_publish -- --ignored
    ;;
  sweep) proto sweep 0 100000 > "$work/out/sweep.out" 2>&1 ;;
  shapes) proto shapes > "$work/out/shapes.out" 2>&1 ;;
  badimages) proto badimages > "$work/out/badimages.out" 2>&1 ;;
  count) proto count > "$work/out/count.out" 2>&1 ;;
  summary) python3 "$model_dir/summarize.py" "$work/out" > "$work/out/summary.txt"; wc -l < "$work/out/summary.txt" ;;
  compare)
    bash "$0" "$work" summary > /dev/null
    if diff "$model_dir/expected-summary.txt" "$work/out/summary.txt"; then echo "SUMMARY_MATCHES lines=$(wc -l < "$work/out/summary.txt")"; else echo "SUMMARY_DIFFERS" >&2; exit 1; fi
    ;;
  all)
    bash "$0" "$work" prepare
    for variant in tree c2cap32 w1 w2; do bash "$0" "$work" build "$variant"; done
    for one in n7-tests n7-probe n7-fix n5-tests n5-fix n5-w2-targets sweep shapes badimages count; do bash "$0" "$work" "$one"; done
    for variant in tree w1 w2; do bash "$0" "$work" n5-probe "$variant" 0,1,2,3; bash "$0" "$work" n5-probe "$variant" 4,5,6; done
    bash "$0" "$work" compare
    ;;
  *) echo "不认得的步骤：$step" >&2; exit 2 ;;
esac
