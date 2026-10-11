#!/usr/bin/env bash
# 正推腿（m3-batch16-code-r1）的复核脚本：在冻结树的拷贝上跑 harness 档的三个测试目标，并逐个施加变异看红不红。
# 只跑 harness 与 core 库的测试，不碰 checker 档。整条脚本要经内存包装与线程上限起：
#   WORKDIR=<空目录> RUN_WITH_MEMORY_CAP_PARALLEL_REASON=... bash scripts/run-with-memory-cap.sh 8G \
#     bash scripts/capped.sh 15 bash research/prompts/m3-batch16-code-r1-sonnet-model/rerun.sh <阶段…>
# 阶段（每个阶段单独起一次，前台跑得完；WORKDIR 在阶段之间共用，第一次起是空目录）：
#   baseline  三个目标 + core 库单测，不变异
#   rows      mutations.tsv 第 2386-2393 行逐行施加、跑点名的目标
#   x1        去掉 establish_instance 里暖机之后的 release_reclaim_holds()，跑 13 个相关目标
#   x2        重建分配器「某块盘没有有效根」那一支改成当场放开到 0，跑 5 个目标
#   x3        重建分配器里扣住的那一遍回收整句去掉，跑 5 个目标
#   x4        mutations.tsv 第 2388 行（不滤被抛弃的根）施加后，跑 13 个相关 harness 目标（看系统层面有没有红）
# 环境：FROZEN_TREE（默认 /tmp/claude-1000/m3-batch16-code-r1/tree）、WORKDIR（必填）。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FROZEN_TREE="${FROZEN_TREE:-/tmp/claude-1000/m3-batch16-code-r1/tree}"
: "${WORKDIR:?WORKDIR 必填}"
export FROZEN_TREE WORKDIR CARGO_TARGET_DIR="$WORKDIR/target"
mkdir -p "$WORKDIR/out"
if [ ! -d "$WORKDIR/w" ]; then mkdir -p "$WORKDIR/w" && cp -r "$FROZEN_TREE/." "$WORKDIR/w/"; fi
summary="$WORKDIR/out/summary.txt"

run_target() { # <标签> <目标或 --lib> [过滤]
    local label="$1" target="$2" filter="${3:-}" log="$WORKDIR/out/$1.$2.log"
    if [ "$target" = "--lib" ]; then
        (cd "$WORKDIR/w" && cargo test --offline -p singlefs-core --lib $filter) > "$log" 2>&1
    else
        (cd "$WORKDIR/w" && cargo test --offline -p singlefs-harness --test "$target") > "$log" 2>&1
    fi
    local code=$?
    echo "$label | $target | exit=$code | $(grep -E '^test result' "$log" | tail -1)" | tee -a "$summary"
    grep -E '^test .* FAILED' "$log" | cut -c1-200 | sed 's/^/    /' | tee -a "$summary"
}

three_targets=(the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads
    a_lone_surviving_device_reads_back_what_it_read_before_the_unmount
    unlink_mount_continuation_and_the_emptied_version)
related_targets=("${three_targets[@]}" admission_raises_the_floor_before_refusing reuse_after_raising_the_floor
    whole_device_loss_after_warm_up row_publish_admission warm_up_journal_counter writable_mount_takes_a_second_instance
    writable_mount_of_a_formatted_pool an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor
    raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic admission_formula_on_the_mounted_state)
few_targets=(the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads
    a_lone_surviving_device_reads_back_what_it_read_before_the_unmount writable_mount_of_a_formatted_pool
    admission_raises_the_floor_before_refusing whole_device_loss_after_warm_up)

for stage in "$@"; do
    echo "### 阶段 $stage" | tee -a "$summary"
    python3 "$here/apply.py" restore >/dev/null
    case "$stage" in
        baseline)
            for t in "${three_targets[@]}"; do run_target base "$t"; done
            run_target base --lib newest_valid_root_on_every_device ;;
        rows)
            for n in 2386 2387 2388 2389 2390 2391 2392 2393; do
                info="$(python3 "$here/apply.py" row "$n")"
                target="$(sed -n 's/^target=\([^ ]*\) .*/\1/p' <<<"$info")"
                test_name="$(sed -n 's/.* test=\(.*\)$/\1/p' <<<"$info")"
                echo "row $n | 必须红的测试：$test_name" | tee -a "$summary"
                if [ "$target" = "--lib" ]; then run_target "row$n" --lib "$test_name"; else run_target "row$n" "$target"; fi
                python3 "$here/apply.py" restore >/dev/null
            done ;;
        x1)
            python3 "$here/apply.py" edit crates/singlefs-core/src/mount.rs \
                '    allocator.release_reclaim_holds();\n    // 取号之前在拷贝上取的落点就是真发起来取的' \
                '    // MUT-removed\n    // 取号之前在拷贝上取的落点就是真发起来取的'
            for t in "${related_targets[@]}"; do run_target x1 "$t"; done ;;
        x2)
            python3 "$here/apply.py" edit crates/singlefs-core/src/mount.rs \
                '        None => floor_of_the_reclaim,\n    };\n    allocator.reclaim_released_up_to(floor_released_immediately' \
                '        None => CheckpointTxg(0),\n    };\n    allocator.reclaim_released_up_to(floor_released_immediately'
            for t in "${few_targets[@]}"; do run_target x2 "$t"; done ;;
        x3)
            python3 "$here/apply.py" edit crates/singlefs-core/src/mount.rs \
                '    allocator.reclaim_released_up_to(\n        floor_of_the_reclaim,\n        ReclaimedReuse::HeldUntilFloorTakesEffect,\n    );\n    let shadow_ledger_computed' \
                '    let shadow_ledger_computed'
            for t in "${few_targets[@]}"; do run_target x3 "$t"; done ;;
        x4)
            python3 "$here/apply.py" row 2388 >/dev/null
            for t in "${related_targets[@]}"; do run_target x4 "$t"; done ;;
        *) echo "不认识的阶段 $stage" >&2; exit 2 ;;
    esac
    python3 "$here/apply.py" restore >/dev/null
done
