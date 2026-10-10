#!/usr/bin/env bash
# 正推腿复跑：在主 agent 自己的草稿目录里照这份重跑（不动仓、不动冻结副本）。
#   S=<自己的草稿目录> bash research/prompts/m3-formal-code-b34-r1-sonnet-model/tools/rerun.sh <阶段…>
# 阶段（次序即依赖）：
#   setup    把冻结副本拷到 $S/work、核 sha256 清单（0 行不对才算数）
#   build    只编（日志 $S/build.log 要有 Compiling singlefs-checker / singlefs-core / singlefs-harness，没有这一趟不作数）
#   tests    被测 harness 档测试基线（P3 新用例 / 两份改过的测试 / mount_transparency / B4 / checker_known_bad_images / 抬 F 慢档那一条）
#   rows     36 行变异（34-1..34-4）：整行只出现一次、锚点命中一次；harness 档 34 行逐行施加、跑、还原（checker-tier 的 2 行不跑）→ $S/mut-results3.jsonl
#   custom   自拟变异：取错新根段字段（M1–M4）与不越界的替代变异（ALT1）→ $S/custom*.out
#   b4       B4 窗口敏感性 → $S/b4-windows.log
#   probes   探针：全零树表指针（P2 / Q2）、Q1 两形、I-5.5 两形、合法历史前缀状态（P1）、I-5.4 逐状态计数（#9 的旁证）、低于下界的根
#   patches  临时补丁：q1a（Q1 候选）、q2（Q2 候选）、q4（Q4 候选）、rvoff 与 instrument（RV 开 / 关对照）；每个补丁跑完还原并核 sha256
# 每条 cargo 经 run-with-memory-cap（8G）与 capped.sh 10（tools/env.sh）。同会话并行时先设 RUN_WITH_MEMORY_CAP_PARALLEL_REASON。
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/env.sh"
W="$S/work"
pt() { bash "$HERE/run-tests.sh" "$@"; }
for stage in "$@"; do
  case "$stage" in
  setup)
    mkdir -p "$S"; rm -rf "$W"; mkdir -p "$W"; cp -r "$FROZEN"/. "$W"/
    (cd "$W" && sha256sum -c "$(dirname "$FROZEN")/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/不对的行数: /' ;;
  build) bash "$HERE/build.sh"; grep -E 'Compiling singlefs|Finished|exit=' "$S/build.log" ;;
  tests)
    pt "$S/t-newcore.log" -p singlefs-harness --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts --test pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader --test pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader
    pt "$S/t-kbi.log" -p singlefs-harness --test checker_known_bad_images
    pt "$S/t-mt.log" -p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount
    pt "$S/t-b4.log" -p singlefs-harness --lib -- a_rollback_takes_exactly_the_inode_leaf_containers
    pt "$S/t-base1.log" -p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots --test writable_mount_of_a_formatted_pool --test reuse_after_raising_the_floor -- --include-ignored --exact overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
    grep -hE 'test result|exit=' "$S"/t-newcore.log "$S"/t-kbi.log "$S"/t-mt.log "$S"/t-b4.log "$S"/t-base1.log ;;
  rows)
    (cd "$S" && python3 "$HERE/select_rows.py" "$W" "$REPO/research/prompts/_m3-formal-code-b34-r1-diff.md")
    rm -f "$S/mut-results3.jsonl"
    for range in "0 12" "12 24" "24 36"; do
      set -- $range
      python3 "$HERE/mutate_rows2.py" "$W" "$S/rows.json" "$1" "$2" "$S/mut-results3.jsonl" | cut -c1-200
    done
    (cd "$W" && sha256sum -c "$(dirname "$FROZEN")/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/还原后不对的行数: /' ;;
  custom)
    for f in custom1 custom1b custom2; do python3 "$HERE/custom_mut.py" "$W" "$MODEL/data/$f.json" | tee "$S/$f.out" | cut -c1-260; done ;;
  b4) python3 "$HERE/b4_windows.py" "$W" 6 58 119 134 6 99 101 134 60 118 5 98 5 99 100 133 | tee "$S/b4-windows.log" ;;
  probes)
    python3 "$HERE/build_probes.py" "$W" "$MODEL"
    for t in probe_clean probe_newest_root_tree_table_pointer_all_zero probe_older_root_tree_table_pointer_all_zero probe_i55_verdicts_by_form probe_a_root_below_the_floor_with_an_all_zero_tree_table_pointer_is_not_a_candidate; do
      pt "$S/t-$t.log" -p singlefs-harness --test zz_probe_all_zero_tree_table -- --nocapture --exact "$t"; grep -hE 'PROBE|exit=' "$S/t-$t.log" | cut -c1-300
    done
    for t in probe_q1_next_mount_version_with_all_zero_tree_table_pointer_in_the_new_root_segment probe_q1_applied_only_by_records_version_with_all_zero_tree_table_pointer probe_prefix_states_of_legal_publishes_never_redden_or_partially_judge_i54 probe_i54_tally_over_the_prefix_states_of_the_new_pool_file_creation; do
      pt "$S/t-$t.log" -p singlefs-harness --test zz_probe_q1 -- --nocapture --exact "$t"; grep -hE 'PROBE|exit=' "$S/t-$t.log" | cut -c1-300
    done ;;
  patches)
    P="python3 $HERE/patch_walk.py $W $FROZEN"
    for name in q1a q2 q4 rvoff instrument; do
      echo "== 补丁 $name"; $P apply $name || break
      case $name in
        q1a) pt "$S/t-p-q1a.log" -p singlefs-harness --test checker_known_bad_images --test zz_probe_q1 -- --include-ignored
             pt "$S/t-p-q1a2.log" -p singlefs-harness --test zz_probe_q1 -- --nocapture --exact probe_q1_applied_only_by_records_version_with_all_zero_tree_table_pointer
             grep -hE 'test result|PROBE q1b.zeroed.*violated|exit=' "$S/t-p-q1a.log" "$S/t-p-q1a2.log" | cut -c1-260 ;;
        q2) pt "$S/t-p-q2.log" -p singlefs-harness --test checker_known_bad_images; grep -hE 'test result|left:|right:|exit=' "$S/t-p-q2.log" | cut -c1-200 ;;
        q4) pt "$S/t-p-q4.log" -p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement --test checker_known_bad_images --test reuse_after_raising_the_floor --test writable_mount_of_a_formatted_pool
            grep -hE 'test result|exit=' "$S/t-p-q4.log" ;;
        rvoff) pt "$S/t-p-rvoff.log" -p singlefs-harness --test zz_probe_q1 -- --nocapture --exact probe_i54_tally_over_the_prefix_states_of_the_new_pool_file_creation; grep -hE 'PROBE-TALLY|exit=' "$S/t-p-rvoff.log" | cut -c1-260 ;;
        instrument) PROBE_RV=1 pt "$S/t-p-instr.log" -p singlefs-harness --test zz_probe_q1 -- --nocapture --exact probe_prefix_states_of_legal_publishes_never_redden_or_partially_judge_i54
             grep -h RVVERSIONS "$S/t-p-instr.log" | sort | uniq -c | sort -rn; grep -hE 'PROBE-PREFIX' "$S/t-p-instr.log" | cut -c1-250 ;;
      esac
      $P restore
    done ;;
  *) echo "不认识的阶段 $stage" >&2; exit 2 ;;
  esac
done
