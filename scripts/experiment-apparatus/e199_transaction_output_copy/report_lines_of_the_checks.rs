//! 增长次序、阳性对照、判定、丁历史、累计、常量核对与计数模型的各输出行。

use super::*;

/// 甲的增长阶：u = 10⁵→10⁶、10⁶→10⁷ 两段上的 log-log 斜率（照抄准入前；现算重建准入前与准入后）。
fn growth_order_lines(points: &[(String, Knobs)]) -> Vec<String> {
    let geometries = [
        ("1e5", geometry_of_point(points, "tombstone_records_100000")),
        ("1e6", geometry_of_point(points, "main_geometry")),
        ("1e7", geometry_of_point(points, "tombstone_records_10000000")),
    ];
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let quantities: [(&str, fn(&Geometry) -> u64); 3] = [
        ("jia_after_patch_copied_before", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).copied_before),
        ("jia_after_patch_recomputed_before", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).recomputed_before),
        ("jia_after_patch_recomputed_after", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).recomputed_after),
    ];
    let mut lines = Vec::new();
    for (quantity, measure) in quantities {
        for pair in geometries.windows(2) {
            let (lower, upper) = (measure(&pair[0].1), measure(&pair[1].1));
            lines.push(format!(
                "name=growth_order quantity={quantity} from_tombstone_records={} to_tombstone_records={} lower={lower} upper={upper} slope={:.5}",
                pair[0].0,
                pair[1].0,
                (upper as f64 / lower as f64).log10()
            ));
        }
    }
    lines
}

/// 阳性对照（每条臂都跑）：u = 10⁶ 对 u = 10⁵ 的比；照抄 ≥ 9，现算重建 ≥ 8（第一份第五节那张表，第二份另加一条；
/// 乙2、乙1、丙、丁的退化形态照第四份第五节写死）。
pub(super) fn positive_control_lines(points: &[(String, Knobs)]) -> (Vec<String>, Vec<(String, String)>) {
    let lower = geometry_of_point(points, "tombstone_records_100000");
    let upper = geometry_of_point(points, "main_geometry");
    let mut lines = Vec::new();
    let mut verdict_fields = Vec::new();
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let degenerate_forms: [(&str, fn(&Geometry) -> u64); 5] = [
        ("jia_after_patch_copied_before", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).copied_before),
        ("yi2_persistent_sharing_turned_off", |geometry| yi2_sharing_turned_off_copied_before_admission(geometry).total(geometry.knobs.settle_rounds)),
        ("yi1_reference_counted_sharing_turned_off", |geometry| {
            yi1_copied_before_admission(geometry, ReferenceSharing::TurnedOff).total(geometry.knobs.settle_rounds)
        }),
        ("bing_cloned_before_handing_in", |geometry| bing_copied_after_admission(geometry) + version_bytes(geometry)),
        ("ding_period_1_whole_new_base", |geometry| ding_copied_after_admission(geometry, true, Compaction::WholeNewBase)),
    ];
    for (arm, measure) in degenerate_forms {
        let (lower_bytes, upper_bytes) = (measure(&lower), measure(&upper));
        let holds = side_of(upper_bytes, lower_bytes, (9, 1)) == Side::AtOrAboveThreshold && lower_bytes > 0;
        lines.push(format!("name=positive_control arm={arm} tombstone_records_1e5={lower_bytes} tombstone_records_1e6={upper_bytes} ratio={} threshold=9 holds={holds}", ratio_text(upper_bytes, lower_bytes)));
        verdict_fields.push((format!("positive_control_{arm}"), holds.to_string()));
    }
    // 现算重建的阳性对照：乙2、丙、丁的残余式关掉（改成按全部单元现算），准入后比 ≥ 8。
    for arm in [Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction] {
        let (lower_bytes, upper_bytes) = (recomputed_in_full(&lower).1, recomputed_in_full(&upper).1);
        let holds = side_of(upper_bytes, lower_bytes, (8, 1)) == Side::AtOrAboveThreshold && lower_bytes > 0;
        lines.push(format!(
            "name=recompute_positive_control arm={} tombstone_records_1e5={lower_bytes} tombstone_records_1e6={upper_bytes} ratio={} threshold=8 holds={holds}",
            arm.label(),
            ratio_text(upper_bytes, lower_bytes)
        ));
        verdict_fields.push((format!("recompute_positive_control_{}", arm.label()), holds.to_string()));
    }
    // 附带（第四份第六节另加那一行）：执行员头一种读法那一比，报数、不判，不进判决行、不触发作废条款第 3 条。
    let touched_whole = |geometry: &Geometry| yi2_touched_collections_whole_copied_before_admission(geometry).total(geometry.knobs.settle_rounds);
    let (lower_bytes, upper_bytes) = (touched_whole(&lower), touched_whole(&upper));
    lines.push(format!(
        "name=positive_control_reported_only arm=yi2_persistent_touched_collections_whole tombstone_records_1e5={lower_bytes} tombstone_records_1e6={upper_bytes} ratio={} judged=no reason=registered_reading_is_sharing_turned_off",
        ratio_text(upper_bytes, lower_bytes)
    ));
    (lines, verdict_fields)
}

/// G0 上判定那几格，与几何敏感性：同一格在敏感性各点上重跑，翻面记「不稳定」并写明哪一点翻；a_T = 0、D = 32768 只报数。
pub(super) fn judgement_lines(points: &[(String, Knobs)]) -> (Vec<String>, bool) {
    let main = geometry_of_point(points, "main_geometry");
    let mut lines = Vec::new();
    for cell in judged_cells() {
        let main_verdict = cell.verdict(&main, JUDGED_THRESHOLD);
        let mut flipped = Vec::new();
        let mut at_points = Vec::new();
        for point in SENSITIVITY_POINTS {
            let verdict = cell.verdict(&geometry_of_point(points, point), JUDGED_THRESHOLD);
            if verdict != main_verdict {
                flipped.push(point);
            }
            at_points.push(format!("{point}:{verdict}"));
        }
        let reported_only: Vec<String> =
            REPORTED_ONLY_POINTS.iter().map(|point| format!("{point}:{}", cell.verdict(&geometry_of_point(points, point), JUDGED_THRESHOLD))).collect();
        let stability = if flipped.is_empty() { "stable".to_string() } else { format!("unstable_flipped_at_{}", flipped.join("+")) };
        lines.push(format!(
            "name=judgement point=main_geometry cell={} verdict={main_verdict} sensitivity={stability} at_points={} reported_only={}",
            cell.label(),
            at_points.join(","),
            reported_only.join(",")
        ));
    }
    // 判别力自证：门槛挪到 G0 与 G-small 两个值之间，那一格的检查必须由一侧转到另一侧（登记给的两例：照抄门槛 1，现算重建门槛 5）。
    let small = geometry_of_point(points, "small_geometry");
    let mut every_example_flips = true;
    let examples = [
        (JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: false }, (1, 1)),
        (JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: true }, (5, 1)),
        (JudgedCell { compared_arm: Arm::Yi2, after_admission: true, recomputed: true }, (5, 1)),
    ];
    for (cell, threshold) in examples {
        let (main_side, small_side) = (cell.verdict(&main, threshold), cell.verdict(&small, threshold));
        let flips = main_side != small_side;
        every_example_flips &= flips;
        lines.push(format!(
            "name=discriminative_power cell={} threshold={}/{} main_geometry={main_side} small_geometry={small_side} flips={flips}",
            cell.label(),
            threshold.0,
            threshold.1
        ));
    }
    for cell in judged_cells() {
        let forms = cell.forms(&main).into_iter().zip(cell.forms(&small));
        for (form_index, ((main_numerator, main_denominator), (small_numerator, small_denominator))) in forms.enumerate() {
            let form = if form_index == 0 { "a" } else { "b" };
            let outcome = if main_denominator == 0 || small_denominator == 0 {
                "zero_denominator_no_threshold_between".to_string()
            } else {
                // 门槛取两点比值的中点：(p₀/q₀ + p₁/q₁) ÷ 2。
                let threshold = (main_numerator * small_denominator + small_numerator * main_denominator, 2 * main_denominator * small_denominator);
                let flips = side_of(main_numerator, main_denominator, threshold) != side_of(small_numerator, small_denominator, threshold);
                format!("midpoint_flips_{flips}")
            };
            lines.push(format!("name=discriminative_power_midpoint cell={} form={form} outcome={outcome}", cell.label()));
        }
    }
    (lines, every_example_flips)
}

/// 丁的 256 次连发：每个 p、每种压实方式一行（轨迹、合计、摊销、甲 ÷ 丁），另报甲的合计与「比值 ≥ 2 的最小 p」。
fn ding_history_lines() -> Vec<String> {
    let geometries = ding_history_geometries();
    let mut jia = HistoryTotals::default();
    for geometry in &geometries {
        jia.add_publish(geometry, ArmBytes::default());
    }
    let mut lines = vec![format!(
        "name=ding_history_jia publishes={} jia_copied_before_total={} jia_copied_after_total={} jia_recomputed_before_total={} jia_recomputed_after_total={} \
         installed_before_total={} installed_after_total={}",
        jia.publishes,
        jia.jia.copied_before,
        jia.jia.copied_after,
        jia.jia.recomputed_before,
        jia.jia.recomputed_after,
        jia.installed_before,
        jia.installed_after
    )];
    let jia_peaks: Vec<u64> = geometries.iter().map(jia_resident_peak).collect();
    let jia_peak = jia_peaks.iter().copied().max().unwrap_or(0);
    for compaction in [Compaction::WholeNewBase, Compaction::FoldInPlace] {
        let mut smallest_period_at_least_two = None;
        for period in DING_PERIODS {
            let history = ding_history(&geometries, period, compaction);
            let copied_without_installed = side_of(jia.jia.copied_after, history.ding.copied_after, JUDGED_THRESHOLD);
            if copied_without_installed == Side::AtOrAboveThreshold && smallest_period_at_least_two.is_none() {
                smallest_period_at_least_two = Some(period);
            }
            lines.push(format!(
                "name=ding_history period={} compaction={} compaction_copied_total={} amortized_per_publish={} layer_peak={} layer_positive_publishes={} \
                 layer_final={} jia_over_ding_copied_after_without_installed={} jia_over_ding_copied_after_with_installed={} jia_over_ding_copied_before={} \
                 jia_over_ding_recomputed_before={} jia_over_ding_recomputed_after={} ding_resident_peak={} ding_resident_final={} \
                 ding_resident_peak_over_jia={} ding_resident_side_at_one={}",
                history.period,
                history.compaction.label(),
                history.compaction_copied_total,
                ratio_text(history.compaction_copied_total, DING_HISTORY_PUBLISHES),
                history.layer_peak,
                history.layer_positive_publishes,
                history.layer_final,
                ratio_text(jia.jia.copied_after, history.ding.copied_after),
                ratio_text(jia.jia.copied_after + jia.installed_after, history.ding.copied_after + jia.installed_after),
                ratio_text(jia.jia.copied_before, history.ding.copied_before),
                ratio_text(jia.jia.recomputed_before, history.ding.recomputed_before),
                ratio_text(jia.jia.recomputed_after, history.ding.recomputed_after),
                history.resident_peak,
                history.resident_final,
                ratio_text(history.resident_peak, jia_peak),
                resident_peak_side(history.resident_peak, jia_peak)
            ));
        }
        // 落在扫描端点照「扫描出来的极值落在端点」写成 ≤ / ≥ 端点。
        let smallest = match smallest_period_at_least_two {
            Some(period) if period == DING_PERIODS[0] => format!("at_most_{period}"),
            Some(period) => period.to_string(),
            None => format!("above_{}", DING_PERIODS[DING_PERIODS.len() - 1]),
        };
        lines.push(format!("name=ding_smallest_period compaction={} jia_resident_peak={jia_peak} smallest_period_with_copied_after_ratio_at_least_two={smallest}", compaction.label()));
    }
    lines
}

/// 累计里每臂在产物行里的名字：累计里的丁取每 `DING_MAIN_PERIOD` 次发布整份新基压实的那一条臂，名字说出这一点；
/// 每臂一行与三对比值的 `pair=` 都取它，分母用的是哪一条臂从名字就看得出。
fn cumulative_arm_name(arm: Arm) -> &'static str {
    if arm == Arm::DingWithoutCompaction { "ding_period_64_whole_new_base" } else { arm.label() }
}

/// 10⁶ 次 unlink 累计：每臂一行，及三对比值（照抄两种形式；现算重建一种形式）。
pub(super) fn cumulative_lines(totals: &HistoryTotals) -> Vec<String> {
    let mut lines = vec![format!(
        "name=cumulative_unlinks unlinks={CUMULATIVE_UNLINKS} publishes={} ding_period={DING_MAIN_PERIOD} installed_before={} installed_after={}",
        totals.publishes, totals.installed_before, totals.installed_after
    )];
    for arm in [Arm::JiaAfterPatch, Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction] {
        let bytes = totals.arm(arm);
        let label = cumulative_arm_name(arm);
        lines.push(format!(
            "name=cumulative_arm arm={label} copied_before={} copied_after={} recomputed_before={} recomputed_after={}",
            bytes.copied_before, bytes.copied_after, bytes.recomputed_before, bytes.recomputed_after
        ));
    }
    for arm in COMPARED_ARMS {
        let (jia, other) = (totals.jia, totals.arm(arm));
        for (segment, jia_copied, other_copied, installed, jia_recomputed, other_recomputed) in [
            ("before_admission", jia.copied_before, other.copied_before, totals.installed_before, jia.recomputed_before, other.recomputed_before),
            ("after_admission", jia.copied_after, other.copied_after, totals.installed_after, jia.recomputed_after, other.recomputed_after),
        ] {
            let sides = [side_of(jia_copied, other_copied, JUDGED_THRESHOLD), side_of(jia_copied + installed, other_copied + installed, JUDGED_THRESHOLD)];
            lines.push(format!(
                "name=cumulative_ratio pair=jia_after_patch_over_{} segment={segment} copied_without_installed={} copied_with_installed={} copied_verdict={} recomputed={} recomputed_verdict={}",
                cumulative_arm_name(arm),
                ratio_text(jia_copied, other_copied),
                ratio_text(jia_copied + installed, other_copied + installed),
                verdict_of_sides(&sides),
                ratio_text(jia_recomputed, other_recomputed),
                verdict_of_sides(&[side_of(jia_recomputed, other_recomputed, JUDGED_THRESHOLD)])
            ));
        }
    }
    lines
}

/// 第七节「出自被测条款本身」「取实现值的模型常量」两组与内存宽度，在产物里现算一遍（单测另逐条钉）。
fn constant_check_lines() -> (Vec<String>, [bool; 3]) {
    let clause = [
        CODE_TWO_HEADER_BYTES + 2 * CODE_TWO_KEY_BYTES + CODE_TWO_FANOUT * CODE_TWO_ENTRY_BYTES + 53 == NODE_BYTES,
        CODE_THREE_RECORD_AREA_START + TOMBSTONE_CONTAINER_RECORDS * TOMBSTONE_RECORD_DISK_BYTES + 32 == UNIT_BYTES,
        CODE_THREE_RECORD_AREA_START + LINEAGE_CONTAINER_RECORDS * LINEAGE_RECORD_DISK_BYTES + 16 == UNIT_BYTES,
        CODE_THREE_RECORD_AREA_START + RETIREMENT_CONTAINER_RECORDS * RETIREMENT_RECORD_DISK_BYTES + 26 == UNIT_BYTES,
        CODE_THREE_RECORD_AREA_START + INODE_LEAF_RECORDS * INODE_RECORD_DISK_BYTES + 12 == UNIT_BYTES,
    ];
    let implementation = [
        ALLOCATION_RECORD_TREE_INTERNAL_HEADER_BYTES + ALLOCATION_RECORD_TREE_INTERNAL_FANOUT * ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES + 25 == NODE_BYTES,
        ACCOUNTING_ENTRIES == 16,
        DATA_POINTER_HEADER_BYTES + POSITION_ENTRY_BYTES * 2 + DATA_POINTER_WRITE_ORDER_BYTES == DATA_POINTER_BYTES,
        DATA_POINTER_MEMORY_BYTES == DATA_POINTER_BYTES,
    ];
    let registered_widths: [(&str, u64, u64); 23] = [
        ("reference_counted_pointer", REFERENCE_COUNTED_POINTER_BYTES, 8),
        ("tombstone_record", TOMBSTONE_RECORD_MEMORY_BYTES, 40),
        ("lineage_record", LINEAGE_RECORD_MEMORY_BYTES, 24),
        ("retirement_record", RETIREMENT_RECORD_MEMORY_BYTES, 40),
        ("inode_record", INODE_RECORD_MEMORY_BYTES, 64),
        ("allocation_record", ALLOCATION_RECORD_MEMORY_BYTES, 40),
        ("container_shell", CONTAINER_SHELL_BYTES, 56),
        ("unit_shell", UNIT_SHELL_BYTES, 48),
        ("shape_node", SHAPE_NODE_BYTES, 8),
        ("shape_level", SHAPE_LEVEL_BYTES, 24),
        ("mapping_key_structure", key_structure_bytes(CENTRAL_MAPPING_TREE), 115),
        ("accounting_key_structure", key_structure_bytes(ACCOUNTING_TREE), 102),
        ("deadlist_key_structure", key_structure_bytes(DEADLIST_TREE), 147),
        ("mapped_unit", MAPPED_UNIT_ENTRY_BYTES, 67),
        ("mapped_unit_with_locations", MAPPED_UNIT_WITH_LOCATIONS_BYTES, 95),
        ("mapping_entry_table", MAPPING_ENTRY_TABLE_BYTES, 194),
        ("mapping_key_bytes", MAPPING_KEY_BYTES_ENTRY_BYTES, 51),
        ("mapping_leaf_entry", MAPPING_LEAF_ENTRY_BYTES, 79),
        ("multi_level_shape_node", MULTI_LEVEL_SHAPE_NODE_BYTES, 32),
        ("allocation_tree_node_identity", ALLOCATION_TREE_NODE_IDENTITY_BYTES, 16),
        ("deleted_inode_number", DELETED_INODE_NUMBER_BYTES, 32),
        ("data_pointer", DATA_POINTER_MEMORY_BYTES, 88),
        ("carried_data_unit", CARRIED_DATA_UNIT_BYTES, 32792),
    ];
    let widths_hold = registered_widths.iter().all(|&(_, measured, registered)| measured == registered);
    let widths: Vec<String> = registered_widths.iter().map(|(name, measured, _)| format!("{name}={measured}")).collect();
    let holds = [clause.iter().all(|holds| *holds), implementation.iter().all(|holds| *holds), widths_hold];
    let lines = vec![
        format!("name=constant_checks clause_constants_hold={} implementation_constants_hold={} memory_widths_hold={}", holds[0], holds[1], holds[2]),
        format!("name=memory_widths {}", widths.join(" ")),
    ];
    (lines, holds)
}

/// 这一段没量、归第二段或不量的几样（第五节「实现量与分段」、第十一节「够判停机」），各写一行，不当 0。
fn not_in_this_segment_lines() -> Vec<String> {
    [
        "execution_clock_ratio reason=segment_two_measures_it",
        "estimated_clock reason=segment_two_measures_the_copy_bandwidth",
        "yi2_grows_with_containers reason=segment_two_counting_allocator_at_the_equal_height_points",
        "read_path_nanoseconds reason=segment_two",
        "resident_peak_of_yi1_and_yi2 reason=segment_two_counting_allocator",
        "whole_allocator_copy_per_round reason=out_of_range_not_measured",
        "extent_tree_nodes_and_other_files reason=zero_in_the_model_not_measured",
        "compaction_batch_cloning_the_previous_version reason=not_reached_by_the_main_geometry_not_measured",
        "implementation_comparison_segment reason=main_agent_decided_not_to_run_it",
    ]
    .iter()
    .map(|quantity| format!("name=not_in_this_segment quantity={quantity}"))
    .collect()
}

/// 整份产物（不含收尾行）：配置、常量、各取样点、增长阶、阳性对照、判定与敏感性、丁的历史、10⁶ 次 unlink 累计、这一段不量的、判决行。
pub(super) fn counting_model_lines(threads: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config experiment=e199_transaction_output_copy segment=counting_model registration=research/prompts/e199-preregistration-r4.md \
         threads_cumulative_unlinks={threads} threads_sampling_points=1 threads_ding_history=1 unit_bytes={UNIT_BYTES} node_bytes={NODE_BYTES} \
         fanout={CODE_TWO_FANOUT} tombstone_capacity={TOMBSTONE_CONTAINER_RECORDS} lineage_capacity={LINEAGE_CONTAINER_RECORDS} \
         retirement_capacity={RETIREMENT_CONTAINER_RECORDS} inode_leaf_capacity={INODE_LEAF_RECORDS} allocation_leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} \
         allocation_fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} accounting_entries={ACCOUNTING_ENTRIES} fixed_point_nodes={FIXED_POINT_NODES} \
         instance_table_units={INSTANCE_TABLE_UNITS} mapping_entry_bytes={MAPPING_ENTRY_BYTES} data_pointer_bytes={DATA_POINTER_BYTES} \
         judged_threshold={}/{} judged_arm=jia_after_patch baseline_patch=b_cluster_item_two_in_the_main_worktree",
        JUDGED_THRESHOLD.0, JUDGED_THRESHOLD.1
    )];
    let (constant_lines, constant_holds) = constant_check_lines();
    lines.extend(constant_lines);
    let points = sampling_points();
    for (label, knobs) in &points {
        lines.extend(point_lines(label, &Geometry::new(*knobs)));
    }
    lines.extend(growth_order_lines(&points));
    let (positive_lines, positive_fields) = positive_control_lines(&points);
    lines.extend(positive_lines);
    let (judgement, every_example_flips) = judgement_lines(&points);
    lines.extend(judgement);
    lines.extend(ding_history_lines());
    lines.extend(cumulative_lines(&cumulative_unlinks(CUMULATIVE_UNLINKS, threads)));
    lines.extend(not_in_this_segment_lines());
    let positive: Vec<String> = positive_fields.iter().map(|(name, holds)| format!("{name}={holds}")).collect();
    lines.push(format!(
        "name=verdict segment=counting_model clause_constants_hold={} implementation_constants_hold={} memory_widths_hold={} \
         discriminative_power_examples_flip={every_example_flips} {} execution_clock=not_run",
        constant_holds[0],
        constant_holds[1],
        constant_holds[2],
        positive.join(" ")
    ));
    lines
}

#[cfg(test)]
#[path = "report_lines_of_the_checks_tests.rs"]
mod tests;
