#!/usr/bin/env bash
# 正推腿复跑：在主 agent 自己的草稿目录里照这份重跑（不动仓、不动冻结副本）。
#   S=<自己的草稿目录> bash research/prompts/m3-formal-code-b35-r1-sonnet-model/tools/rerun.sh <阶段…>
# 阶段（次序即依赖）：
#   setup    把冻结副本拷到 $S/work、核 sha256 清单（0 行不对才算数）
#   build    只编（日志 $S/build.log 要有 Compiling singlefs-checker / singlefs-core / singlefs-harness，没有这一趟不作数）
#   tests    被测 harness 档测试基线（checker_known_bad_images 全文件、四份别的测试文件、reuse 那一条、--lib）→ $S/t-*.log
#   probes   探针（拼进 zz_probe_b35.rs，opt-level 2 的编译目录 $S/target-opt）：P1 子集矩阵、H2 / H3 / H3Z / H3S 合法与坏流逐状态签名、
#            P2 形态矩阵、P4 I-5.5 判定、P5 逐字节、P6 新池流、P1F 退路 → $S/t-*.log
#   fl-off   把 FL 两行变异（FL 关掉）临时施加到副本，重跑 P1 子集矩阵与 H2 / H3 / H3S，还原并核 sha256 → $S/t-*-off.log
#   rows     第 35 批 21 行变异：select_rows35.py 核行与锚点；harness 档 13 行逐行施加、跑、还原（checker-tier 的 8 行不跑）→ $S/mut-results.jsonl
#   b4       B4 窗口敏感性（b34 判决点名的三个窗口与取样点之间的四个空档窗口）→ $S/b4-windows2.log
# 每条 cargo 经 run-with-memory-cap（8G）与 capped.sh 10（tools/env.sh）。同会话并行时先设 RUN_WITH_MEMORY_CAP_PARALLEL_REASON。
# 峰值表里没有这几组命令的键，上限 8G 不触发「超过峰值两倍」那一条；若退 251，设 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON（env.sh 已给默认理由）。
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/env.sh"
W="$S/work"
D="$(dirname "$FROZEN")"
probe() { OPT=1 bash "$HERE/rerun_probe.sh" "$@"; }
for stage in "$@"; do
  case "$stage" in
  setup)
    mkdir -p "$S"; rm -rf "$W"; mkdir -p "$W"; cp -r "$FROZEN"/. "$W"/
    (cd "$W" && sha256sum -c "$D/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/不对的行数: /' ;;
  build) bash "$HERE/build.sh"; grep -E 'Compiling singlefs|Finished|exit=' "$S/build.log" ;;
  tests)
    bash "$HERE/run-tests.sh" "$S/t-kbi.log" -p singlefs-harness --test checker_known_bad_images
    bash "$HERE/run-tests.sh" "$S/t-base2.log" -p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots --test writable_mount_of_a_formatted_pool --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts
    bash "$HERE/run-tests.sh" "$S/t-base3.log" -p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored --exact raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
    bash "$HERE/run-tests.sh" "$S/t-base4.log" -p singlefs-harness --lib
    grep -hE 'test result|exit=' "$S"/t-kbi.log "$S"/t-base2.log "$S"/t-base3.log "$S"/t-base4.log ;;
  probes)
    python3 "$HERE/build_probes.py" "$W" "$MODEL"
    OPT=1 bash "$HERE/run-tests.sh" "$S/t-probe-build.log" -p singlefs-harness --test zz_probe_b35 --no-run
    probe t-p1b.log probe_p1_subsets_of_the_record_copies_of_a_three_record_publish
    probe t-h2-on.log probe_h2_states_of_a_legal_three_record_publish
    probe t-h3-on.log probe_h3_states_after_a_mount_over_a_version_applied_only_by_its_three_records
    probe t-h3z-on.log probe_h3_all_zero_tree_table_pointer_in_the_records_of_b
    probe t-h3s-on.log probe_h3_stale_first_record_of_b
    probe t-order.log probe_invariant_order
    probe t-p2.log probe_p2_matrix_of_the_all_zero_tree_table_pointer_forms
    probe t-p2c.log probe_p2_older_candidate_root_with_units_of_its_own_all_zero
    probe t-p4.log probe_p4_first_data_unit_verdicts_by_form
    probe t-p5.log probe_p5_core_root_choice_against_the_gpu_rule_byte_by_byte
    probe t-p6.log probe_p6_new_pool_stream_i54_and_i310_evaluated_sets
    probe t-p1f.log probe_p1_fallback_when_the_last_record_of_an_applied_only_version_is_not_readable
    probe t-flow4.log probe_flow4_new_pool_publish_with_the_tree_table_pointer_of_the_root_and_its_records_all_zero
    probe t-p4b.log probe_p4_what_else_changes_when_the_extent_root_pointer_is_zeroed
    grep -hE 'PROBE-(P1F|P2 |P4|P4B|P5 total|P6|H2|H3|H3Z|H3S|FLOW4)' "$S"/t-*.log | cut -c1-240 ;;
  fl-off)
    P="python3 $HERE/patch_b35.py $W $FROZEN"
    $P apply "第35批 35-1 FL：下一次" "第35批 35-1 FL：由记录" || exit 1
    OPT=1 bash "$HERE/run-tests.sh" "$S/t-fl-off-build.log" -p singlefs-harness --test zz_probe_b35 --no-run
    probe t-p1-fl-off.log probe_p1_subsets_of_the_record_copies_of_a_three_record_publish
    probe t-h2-off.log probe_h2_states_of_a_legal_three_record_publish
    probe t-h3-off.log probe_h3_states_after_a_mount_over_a_version_applied_only_by_its_three_records
    probe t-h3s-off.log probe_h3_stale_first_record_of_b
    $P restore
    (cd "$W" && sha256sum -c "$D/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/还原后不对的行数: /'
    for p in h2 h3; do diff <(grep '^SIG' "$S/t-$p-on.log") <(grep '^SIG' "$S/t-$p-off.log") >/dev/null && echo "$p 逐状态签名 FL 开 / 关逐行相同" || echo "$p 逐状态签名 FL 开 / 关有不同"; done
    grep -h 'PROBE-H3S' "$S/t-h3s-on.log" "$S/t-h3s-off.log" ;;
  rows)
    (cd "$S" && python3 "$HERE/select_rows35.py" "$W" "$REPO/research/prompts/_m3-formal-code-b35-r1-diff.md")
    rm -f "$S/mut-results.jsonl"
    for range in "0 2" "2 7" "7 21"; do
      set -- $range
      python3 "$HERE/mutate_rows2.py" "$W" "$S/rows.json" "$1" "$2" "$S/mut-results.jsonl" | cut -c1-200
    done
    (cd "$W" && sha256sum -c "$D/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/还原后不对的行数: /' ;;
  b4) python3 "$HERE/b4_windows2.py" "$W" 6 58 5 98 100 133 5 28 30 98 100 118 120 133 | tee "$S/b4-windows2.log" ;;
  *) echo "不认识的阶段 $stage" >&2; exit 2 ;;
  esac
done
