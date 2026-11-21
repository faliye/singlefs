//! 导出文件的汇总，以及登记的各行次（布局三格、各行次的读数、尾批几何、标签六几何）的输出行。

use super::*;

// ---------------------------------------------------------------------------------------------
// 产物：全部判定行（不带 E7RESULT 前缀；main 交 Emitter 打）。
// ---------------------------------------------------------------------------------------------

const LAYOUT_THREE_CELLS: [&str; 17] = [
    "L03-1", "L03-2", "L03-3", "L03-4", "L03-5", "L03-6", "L03-7", "L03-8", "L03-9", "L03-10", "L03-11", "L03-12", "L03-13", "L03-14", "L03-15", "L03-16",
    "L03-17",
];

/// V6（跑前登记 e172-r2 第十一节）：四个参数；H3 带 `intent_write_time_seconds=` 且 = `sequential_write_time_seconds=` + 60，H1、H2 不带这一格。
pub(super) fn configuration_matches_registration(history: &HistoryExport) -> bool {
    let expected = [("devices", "2"), ("image_bytes", "4294967296"), ("physical_block_size", "512"), ("batch_data_units", "1")];
    let parameters_match = expected.iter().all(|(key, value)| history.configuration.get(*key).map(String::as_str) == Some(*value));
    let intent_seconds = field_number(&history.configuration, "intent_write_time_seconds");
    let sequential_seconds = field_number(&history.configuration, "sequential_write_time_seconds");
    let intent_time_matches = match history.name.as_str() {
        "h3" => intent_seconds.is_some() && intent_seconds == sequential_seconds.map(|seconds| seconds + 60),
        "h1" | "h2" => !history.configuration.contains_key("intent_write_time_seconds"),
        _ => false,
    };
    parameters_match && intent_time_matches
}

/// 两份导出：第一份是不带参数的 H1、H2，第二份是 `--history h3`（跑前登记 e172-r2 第五节 5.8 改1、改2）。
const EXPORT_FILE_LABELS: [&str; 2] = ["h1h2", "h3"];
/// V6 / `histories_present`：每份导出里该有的历史，次序也要一样。
pub(super) const HISTORIES_OF_EACH_EXPORT_FILE: [&[&str]; 2] = [&["h1", "h2"], &["h3"]];

/// 两份导出各解一遍，历史按「第一份在前、第二份在后」接起来。
pub(super) fn parse_export_files(export_texts: &[&str; 2]) -> Vec<ExportFile> {
    export_texts.iter().map(|text| parse_export(text)).collect()
}

pub(super) fn report_lines(export_texts: &[&str; 2]) -> Vec<String> {
    let export_files = parse_export_files(export_texts);
    let mut lines = Vec::new();
    let mut void_flags: Vec<(String, bool)> = Vec::new();
    let mut histories_present = true;
    for ((label, text), (export, expected_histories)) in EXPORT_FILE_LABELS.iter().zip(export_texts).zip(export_files.iter().zip(HISTORIES_OF_EACH_EXPORT_FILE)) {
        lines.push(format!(
            "name=export_file file={label} sha256={} lines={} anchored_lines={} done_emitted={} non_hexadecimal_lines={} malformed_lines={}",
            sha256_hexadecimal(text.as_bytes()),
            export.line_count,
            export.anchored_line_count,
            export.done_emitted.map_or_else(|| "absent".to_string(), |value| value.to_string()),
            export.non_hexadecimal_line_count,
            export.malformed_lines.len()
        ));
        for malformed in &export.malformed_lines {
            lines.push(format!("name=malformed_line file={label} detail={}", malformed.replace(' ', "_")));
        }
        let lines_anchor = anchor_line(
            "A-lines",
            "all",
            &format!("file={label} lines_anchored_done"),
            format!("{0}/{0}", export.line_count),
            format!("{}/{}", export.anchored_line_count, export.done_emitted.unwrap_or(0)),
        );
        void_flags.push((format!("a_lines_equal_{label}"), lines_anchor.1));
        lines.push(lines_anchor.0);
        let history_names: Vec<&str> = export.histories.iter().map(|history| history.name.as_str()).collect();
        histories_present &= history_names == expected_histories;
    }
    let crc_anchor = anchor_line("A-crc", "all", "ascii_123456789", "e3069283", format!("{:08x}", castagnoli_crc32(b"123456789")));
    void_flags.push(("a_crc_equal".to_string(), crc_anchor.1));
    lines.push(crc_anchor.0);
    void_flags.push(("histories_present".to_string(), histories_present));
    let histories: Vec<HistoryExport> = export_files.into_iter().flat_map(|export| export.histories).collect();
    for history in &histories {
        let content_anchor = anchor_line("A-content", &history.name, "file_bytes", EXPECTED_FILE_BYTES, history.configuration.get("file_bytes").cloned().unwrap_or_default());
        void_flags.push((format!("a_content_equal_{}", history.name), content_anchor.1));
        lines.push(content_anchor.0);
        let configuration_ok = configuration_matches_registration(history);
        void_flags.push((format!("configuration_matches_{}", history.name), configuration_ok));
        lines.push(format!("name=v6_configuration history={} matches_registration={configuration_ok}", history.name));
    }
    for (anchor, detail, expected, actual) in [
        ("A-capacity", "tombstone_container_records", 815, tombstone_container_capacity()),
        ("A-capacity", "deleted_inodes_root_entries", 2031, deleted_inodes_root_capacity()),
        ("A-capacity", "tombstone_root_entry_bytes", 120, tombstone_root_entry_bytes()),
        ("A-capacity", "retirement_container_records", 959, retirement_container_capacity()),
    ] {
        lines.push(anchor_line(anchor, "all", detail, expected, actual).0);
    }
    let judgements: Vec<HistoryJudgement> = histories.iter().map(judge_history).collect();
    let mut anchor_failures = 0usize;
    for judgement in &judgements {
        for (line, equal) in anchor_lines_of_history(judgement) {
            anchor_failures += usize::from(!equal);
            lines.push(line);
        }
    }
    let mut stop_counts = (0usize, 0usize, 0usize, 0usize);
    for judgement in &judgements {
        lines.push(format!(
            "name=structure history={} regions={} s2_structure_failures={} s3_role_mismatches={} s3_index_notation_only={} s4_tiling_failures={} unreferenced_units={}",
            judgement.name,
            judgement.analysis.segments.iter().map(|segment| segment.regions.len()).sum::<usize>(),
            judgement.structure_failures.len(),
            judgement.role_mismatches.len(),
            judgement.role_notation_only_differences.len(),
            judgement.tiling_failures.len(),
            judgement.analysis.segments.iter().map(|segment| segment.unreferenced_units.len()).sum::<usize>()
        ));
        for (label, list) in [("s2", &judgement.structure_failures), ("s3", &judgement.role_mismatches), ("s3_notation", &judgement.role_notation_only_differences), ("s4", &judgement.tiling_failures)] {
            for item in list {
                lines.push(format!("name=stop_detail check={label} detail={}", item.replace(' ', "_")));
            }
        }
        for segment in &judgement.analysis.segments {
            for unreferenced in &segment.unreferenced_units {
                lines.push(format!("name=unreferenced_unit history={} detail={}", judgement.name, unreferenced.replace(' ', "_")));
            }
        }
        stop_counts.0 += judgement.structure_failures.len();
        stop_counts.1 += judgement.role_mismatches.len();
        stop_counts.2 += judgement.role_notation_only_differences.len();
        stop_counts.3 += judgement.tiling_failures.len();
    }
    lines.extend(row_one_lines(&judgements));
    lines.extend(row_three_lines(&judgements));
    lines.extend(row_four_and_five_lines(&judgements));
    lines.extend(row_six_seven_eight_lines(&judgements));
    for judgement in &judgements {
        lines.extend(trajectory_lines(judgement));
    }
    let mut controls_holding = 0usize;
    let mut controls_run = 0usize;
    let mut controls_failed_in_row_two_scope = 0usize;
    let mut controls_failed_by_history: Vec<String> = Vec::new();
    for (history, judgement) in histories.iter().zip(&judgements) {
        let mut failed_here = 0usize;
        for control in PositiveControl::EVERY_CONTROL {
            let (line, holds) = positive_control_line(history, judgement, control);
            if let Some(holds) = holds {
                controls_run += 1;
                controls_holding += usize::from(holds);
                failed_here += usize::from(!holds);
                controls_failed_in_row_two_scope += usize::from(!holds && control.is_in_row_two_scope());
            }
            lines.push(line);
        }
        controls_failed_by_history.push(format!("positive_controls_failed_{}={failed_here}", history.name));
    }
    lines.extend(row_two_lines(&judgements));
    lines.extend(row_c775_lines(&judgements, controls_failed_in_row_two_scope));
    let branch_check = leaf_branch_check(&judgements);
    lines.push(branch_check.0);
    let retirement_check = retirement_unit_history_check(&judgements);
    lines.push(retirement_check.0);
    lines.push(format!(
        "name=verdict check=stops s2_structure_failures={} s3_role_mismatches={} s3_index_notation_only={} s3_l03_7_second_branch_not_exactly_h3={} stop_retirement_unit_histories_not_registered={} s4_tiling_failures={}",
        stop_counts.0,
        stop_counts.1,
        stop_counts.2,
        usize::from(!branch_check.1),
        usize::from(!retirement_check.1),
        stop_counts.3
    ));
    let void_text: Vec<String> = void_flags.iter().map(|(key, value)| format!("{key}={value}")).collect();
    lines.push(format!(
        "name=verdict check=voids {} positive_controls_failed={} {} positive_controls_run={controls_run} positive_controls_expected={} anchor_not_equal={anchor_failures}",
        void_text.join(" "),
        controls_run - controls_holding,
        controls_failed_by_history.join(" "),
        PositiveControl::APPLICABLE_PAIRS_EXPECTED
    ));
    lines
}

/// S3 另加的一条（跑前登记 e172-r2 第一节 1.2、第十一节 S3）：L03-7 选到第二支的历史恰是 H3。
fn leaf_branch_check(judgements: &[HistoryJudgement]) -> (String, bool) {
    let mut second_branch_histories: Vec<&str> = judgements
        .iter()
        .filter(|judgement| judgement.leaf_branch_records.iter().any(|line| line.starts_with("name=l03_7_branch ") && line.contains(" branch=other_records ")))
        .map(|judgement| judgement.name.as_str())
        .collect();
    second_branch_histories.sort_unstable();
    let is_exactly_the_record_two_kept_history = second_branch_histories == ["h3"];
    (
        format!(
            "name=l03_7_branch_check second_branch_histories={} exactly_h3={is_exactly_the_record_two_kept_history}",
            if second_branch_histories.is_empty() { "none".to_string() } else { second_branch_histories.join(",") }
        ),
        is_exactly_the_record_two_kept_history,
    )
}

/// 第 1 行（原第 1 行 R1.a、R1.b 与 R1.t）：三段历史逐段分开计数（跑前登记 e172-r2 第六节 6.2）。
fn row_one_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut not_equal = 0usize;
    let mut not_discriminable = 0usize;
    let mut per_history: Vec<String> = Vec::new();
    for judgement in judgements {
        let mut not_equal_here = 0usize;
        for row in &judgement.cell_rows {
            lines.push(row.line());
            if row.discriminable {
                not_equal_here += usize::from(!row.is_equal());
            } else {
                not_discriminable += 1;
            }
        }
        lines.extend(judgement.leaf_branch_records.iter().cloned());
        not_equal += not_equal_here;
        let mut cells_without_hits = Vec::new();
        for cell in LAYOUT_THREE_CELLS {
            let hits = judgement.cell_rows.iter().filter(|row| row.cell == cell && row.discriminable && row.actual != "absent").count();
            let not_equal_in_cell = judgement.cell_rows.iter().filter(|row| row.cell == cell && row.discriminable && !row.is_equal()).count();
            lines.push(format!("name=r1b history={} cell={cell} discriminable_hits={hits} not_equal={not_equal_in_cell}", judgement.name));
            if hits == 0 {
                cells_without_hits.push(cell);
            }
        }
        per_history.push(format!(
            "not_equal_cells_{0}={not_equal_here} cells_without_hits_{0}={1} cells_without_hits_list_{0}={2}",
            judgement.name,
            cells_without_hits.len(),
            if cells_without_hits.is_empty() { "none".to_string() } else { cells_without_hits.join(",") }
        ));
    }
    // R1.t：每段历史窗口里每一次系统配置槽写一行，再一行计数（分母 = 槽写次数）。
    let mut time_threshold_not_expected = 0usize;
    for judgement in judgements {
        for row in &judgement.time_threshold_rows {
            lines.push(format!(
                "name=r1t history={} step={} device={} offset={} expected={EXPECTED_TIME_THRESHOLD_MILLISECONDS} actual={} equal={}",
                judgement.name,
                row.step,
                row.device,
                row.offset,
                row.actual_milliseconds,
                row.is_expected()
            ));
        }
        let not_expected_here = judgement.time_threshold_rows_not_expected().len();
        time_threshold_not_expected += not_expected_here;
        lines.push(format!(
            "name=r1t_summary history={} system_configuration_slot_writes={} t_time_not_5000={not_expected_here}",
            judgement.name,
            judgement.time_threshold_rows.len()
        ));
        per_history.push(format!("t_time_not_5000_{}={not_expected_here}", judgement.name));
    }
    lines.push(format!(
        "name=verdict row=1 not_equal_cells={not_equal} discriminable_false_rows={not_discriminable} t_time_not_5000={time_threshold_not_expected} {}",
        per_history.join(" ")
    ));
    lines
}

fn row_three_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut mismatch_counts: BTreeMap<String, [usize; 4]> = BTreeMap::new();
    for row in judgements.iter().flat_map(|judgement| judgement.segment_rows.iter()) {
        lines.push(format!(
            "name=r3 history={} txg={} publish={} counts_for_row3={} expected_segments={} actual_segments={} segments_equal={} expected_kinds={} actual_kinds={} kinds_equal={} expected_roles={} actual_roles={} roles_equal={} root_fua_writes={} expected_root={} actual_root={} root_equal={}",
            row.history,
            row.txg,
            row.publish_kind,
            row.counts_for_row_three,
            row.expected_count_text,
            row.actual_count_text,
            row.count_equal(),
            row.expected_kind_text,
            row.actual_kind_text,
            row.kind_equal(),
            row.expected_roles,
            row.actual_roles,
            row.roles_equal(),
            row.root_force_unit_access_writes,
            row.expected_root_location,
            row.actual_root_location,
            row.root_equal()
        ));
        if !row.compares_role_names_verbatim {
            lines.push(format!(
                "name=r3c_ordinal history={} txg={} publish={} expected_ordinals={} actual_ordinals={} judged=false",
                row.history, row.txg, row.publish_kind, row.expected_ordinals, row.actual_ordinals
            ));
        }
        if row.counts_for_row_three {
            let counts = mismatch_counts.entry(row.history.clone()).or_insert([0usize; 4]);
            counts[0] += usize::from(!row.count_equal());
            counts[1] += usize::from(!row.kind_equal());
            counts[2] += usize::from(!row.roles_equal());
            counts[3] += usize::from(!row.root_equal());
        }
    }
    // R3.d：H1、H3 各一行整窗段串（H2 的窗口里有不是删除类发布的段，不报整窗）。
    let mut window_equal: BTreeMap<String, bool> = BTreeMap::new();
    for (history_name, units_per_publish) in WINDOW_UNITS_PER_DELETION_PUBLISH {
        let Some(judgement) = judgements.iter().find(|judgement| judgement.name == history_name) else { continue };
        let (expected, actual, equal) = window_string_of(judgement, &units_per_publish);
        window_equal.insert(history_name.to_string(), equal);
        lines.push(format!("name=r3d history={history_name} expected={expected} actual={actual} equal={equal}"));
    }
    lines.extend(final_batch_geometry_lines(judgements));
    let mut verdict_fields = Vec::new();
    for judgement in judgements {
        let Some(counts) = mismatch_counts.get(&judgement.name) else { continue };
        let window_text = window_equal.get(&judgement.name).map_or_else(|| "not_reported".to_string(), bool::to_string);
        verdict_fields.push(format!(
            "{0}_segment_mismatches={1} {0}_kind_mismatches={2} {0}_role_mismatches={3} {0}_root_mismatches={4} {0}_window_string_equal={window_text}",
            judgement.name, counts[0], counts[1], counts[2], counts[3]
        ));
    }
    lines.push(format!("name=verdict row=3 {}", verdict_fields.join(" ")));
    lines
}

/// R3.d 的期望（跑前登记 e172-r2 第六节 6.2）：H1 = layout/03 第 59 行四串首尾相接；H3 = 每盘单元数 13、12、11、11 四串首尾相接。
/// 跑前登记 e172-r3 第七节 A-roles-final：H1 最后一批 12（多 tr@0、tr，少 t3），整窗四串 26、24、22、24；H3 照旧。
const WINDOW_UNITS_PER_DELETION_PUBLISH: [(&str, [usize; 4]); 2] = [("h1", [13, 12, 11, 12]), ("h3", [13, 12, 11, 11])];

fn window_string_of(judgement: &HistoryJudgement, units_per_publish: &[usize]) -> (String, String, bool) {
    let operations: Vec<OperationView> = judgement.analysis.segments.iter().flat_map(|segment| segment.operations.iter().cloned()).collect();
    let pieces = cut_into_pieces(&operations);
    let expected = units_per_publish.iter().map(|units| expected_count_text(*units)).collect::<Vec<_>>().join("+");
    let actual = segment_count_text(&pieces);
    let equal = expected == actual;
    (expected, actual, equal)
}

/// 几何敏感性（跑前登记 e172-r2 第八节 8.2 第 1 行）：H1 的最后一批期望（第一支，10 个单元）与 H3 的（第二支，11 个）都拿去判 H3 的最后一批。
fn final_batch_geometry_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let Some(judgement) = judgements.iter().find(|judgement| judgement.name == "h3") else { return Vec::new() };
    let Some((_, final_batch)) = deletion_segment_with_units_after(judgement, "unlink_batch", 0) else { return Vec::new() };
    let actual = segment_count_text(&cut_into_pieces(&final_batch.operations));
    let units_of = |leaf: DeletedRecordLeaf| registered_roles("unlink_batch", 0, leaf).map_or(0, |roles| roles.len());
    let only_record_expectation = expected_count_text(units_of(DeletedRecordLeaf::OnlyTheDeletedRecord));
    let other_records_expectation = expected_count_text(units_of(DeletedRecordLeaf::BesideOtherRecords));
    let only_record_green = only_record_expectation == actual;
    let other_records_green = other_records_expectation == actual;
    vec![format!(
        "name=r3_geometry history=h3 txg={} expected_only_record={only_record_expectation} expected_other_records={other_records_expectation} actual={actual} only_record_green={only_record_green} other_records_green={other_records_green} expectations_differ={} exactly_one_green={}",
        final_batch.label_txg(),
        only_record_expectation != other_records_expectation,
        only_record_green != other_records_green
    )]
}

fn deletion_segment_with_units_after<'judgement>(judgement: &'judgement HistoryJudgement, publish_kind: &str, data_units_after: u64) -> Option<(usize, &'judgement SegmentAnalysis)> {
    judgement.analysis.segments.iter().enumerate().find(|(segment_index, segment)| {
        segment.label_kind() == publish_kind && data_units_of_version(&judgement.analysis.states_after[*segment_index]) == Some(data_units_after)
    })
}

fn unique_unit_lines(history_name: &str, judgement: &HistoryJudgement, segment_index: usize, unique: &BTreeSet<RoleIdentity>, without_basis: &mut usize) -> Vec<String> {
    let segment = &judgement.analysis.segments[segment_index];
    let mut lines = Vec::new();
    for unit in segment.units.iter().filter(|unit| unique.contains(&unit.identity)) {
        let unit_judgement = judge_unique_unit(unit, segment, &judgement.analysis.states_before[segment_index], &judgement.analysis.states_after[segment_index]);
        let count_of = |wanted: RecordChangeBasis| unit_judgement.changes.iter().filter(|(_, _, _, basis)| *basis == wanted).count();
        lines.push(format!(
            "name=r4b history={history_name} txg={} role={} previous_in_window={} changed_records={} allocated_in_this_publish={} released_in_this_publish={} reclaimed_under_the_floor={} unmatched={} has_clause_basis={}",
            segment.label_txg(),
            unit_judgement.role_name,
            unit_judgement.previous_in_window,
            unit_judgement.changes.len(),
            count_of(RecordChangeBasis::AllocatedInThisPublish),
            count_of(RecordChangeBasis::ReleasedInThisPublish),
            count_of(RecordChangeBasis::ReclaimedUnderTheFloor),
            count_of(RecordChangeBasis::Unmatched),
            unit_judgement.has_clause_basis()
        ));
        for ((device, slot), before_value, after_value, basis) in &unit_judgement.changes {
            lines.push(format!(
                "name=r4b_record history={history_name} txg={} role={} device={device} slot={slot} before={} after={} class={}",
                segment.label_txg(),
                unit_judgement.role_name,
                value_text(*before_value),
                value_text(*after_value),
                basis.name()
            ));
        }
        *without_basis += usize::from(!unit_judgement.has_clause_basis());
    }
    lines
}

fn row_four_and_five_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let session = judgements.iter().find(|judgement| judgement.name == "h1");
    let continuation = judgements.iter().find(|judgement| judgement.name == "h2");
    let mut pairs_with_unique_roles = 0usize;
    let mut without_basis = 0usize;
    if let (Some(session), Some(continuation)) = (session, continuation) {
        for data_units_after in [2u64, 1, 0] {
            let session_segment = deletion_segment_with_units_after(session, "unlink_batch", data_units_after);
            let continuation_segment = deletion_segment_with_units_after(continuation, "mount_continuation_unlink_batch", data_units_after);
            let (Some((session_index, session_segment)), Some((continuation_index, continuation_segment))) = (session_segment, continuation_segment) else {
                lines.push(format!("name=r4a data_units_after={data_units_after} pair=missing"));
                continue;
            };
            let session_identities = session_segment.identities();
            let continuation_identities = continuation_segment.identities();
            let continuation_only: BTreeSet<RoleIdentity> = continuation_identities.difference(&session_identities).cloned().collect();
            let session_only: BTreeSet<RoleIdentity> = session_identities.difference(&continuation_identities).cloned().collect();
            let same_shape = continuation_only.is_empty() && session_only.is_empty();
            pairs_with_unique_roles += usize::from(!same_shape);
            lines.push(format!(
                "name=r4a data_units_after={data_units_after} continuation_txg={} session_txg={} continuation_units={} session_units={} continuation_only={} session_only={} same_shape={same_shape}",
                continuation_segment.label_txg(),
                session_segment.label_txg(),
                continuation_identities.len(),
                session_identities.len(),
                names_of_identities(continuation_segment, &continuation_only).join(","),
                names_of_identities(session_segment, &session_only).join(",")
            ));
            lines.extend(unique_unit_lines("h2", continuation, continuation_index, &continuation_only, &mut without_basis));
            lines.extend(unique_unit_lines("h1", session, session_index, &session_only, &mut without_basis));
        }
    }
    // R4.c：H1、H2、H3 都报（H3 是第三个分配器位置，也是第 1 行 R3.c 序号那一格的出处）。
    for judgement in judgements.iter().filter(|judgement| ["h1", "h2", "h3"].contains(&judgement.name.as_str())) {
        for segment in judgement.analysis.segments.iter().filter(|segment| segment.is_deletion_publish) {
            lines.push(allocation_geometry_line(&judgement.name, segment));
        }
    }
    lines.push(format!("name=verdict row=4 pairs_with_unique_roles={pairs_with_unique_roles} unique_units_without_basis={without_basis}"));
    // R5.a（附带，不进任何判定）：H1 最后一批与 2 → 1 那一批的角色集合差（两个方向）。
    if let Some((two_to_one, final_batch, only_two_to_one, only_final)) = session.and_then(final_batch_difference) {
        let row_five_harness_history_matches = matches_the_registered_final_batch_difference(&only_two_to_one, &only_final);
        lines.push(format!(
            "name=r5a history=h1 two_to_one_txg={} final_txg={} only_in_two_to_one={} only_in_final={} matches_registered={row_five_harness_history_matches} incidental=true",
            two_to_one.label_txg(),
            final_batch.label_txg(),
            names_text(&only_two_to_one),
            names_text(&only_final)
        ));
    }
    // R5.b（跑前登记 e172-r2 第六节 6.3）：H3 最后一批写了 t3（叶里还剩 2 号）且 2 → 1 独有里没有 t3 ⇒「照现写」。
    let mut row_five_record_two_kept_history_matches = false;
    let mut final_writes_inode_leaf = false;
    let record_two_kept_judgement = judgements.iter().find(|judgement| judgement.name == "h3");
    if let Some((two_to_one, final_batch, only_two_to_one, only_final)) = record_two_kept_judgement.and_then(final_batch_difference) {
        final_writes_inode_leaf = final_batch.role_names().iter().any(|role| role == "t3");
        let two_to_one_only_has_inode_leaf = only_two_to_one.iter().any(|role| role == "t3");
        row_five_record_two_kept_history_matches = final_writes_inode_leaf && !two_to_one_only_has_inode_leaf;
        lines.push(format!(
            "name=r5b history=h3 two_to_one_txg={} final_txg={} final_writes_t3={final_writes_inode_leaf} only_in_two_to_one={} only_in_final={} matches_registered={row_five_record_two_kept_history_matches}",
            two_to_one.label_txg(),
            final_batch.label_txg(),
            names_text(&only_two_to_one),
            names_text(&only_final)
        ));
        // 判别力自证（第八节 8.2 第 3 行）：H1 的期望（2 → 1 独有 = {t3}、最后一批独有为空）拿去判 H3。
        let harness_expectation_green_on_the_record_two_kept_history = matches_the_registered_final_batch_difference(&only_two_to_one, &only_final);
        lines.push(format!(
            "name=r5b_discrimination history=h3 h1_expectation_green={harness_expectation_green_on_the_record_two_kept_history} h3_expectation_green={row_five_record_two_kept_history_matches} exactly_one_green={}",
            harness_expectation_green_on_the_record_two_kept_history != row_five_record_two_kept_history_matches
        ));
    }
    lines.push(format!("name=verdict row=5 h3_final_writes_t3={final_writes_inode_leaf} h3_matches_registered={row_five_record_two_kept_history_matches}"));
    lines
}

/// R5.a 的登记差（layout/03 第 60 行，那片叶删空那一支）：2 → 1 独有 = {t3}，最后一批独有 = {tr, tr@0}；
/// 由 `registered_roles` 的两份角色集合现算（跑前登记 e172-r3 改18 之后最后一批多了退役单元，第 2 次的「最后一批独有为空」跟着变）。
pub(super) fn matches_the_registered_final_batch_difference(only_two_to_one: &[String], only_final: &[String]) -> bool {
    let two_to_one: BTreeSet<&str> = registered_roles("unlink_batch", 1, DeletedRecordLeaf::OnlyTheDeletedRecord).unwrap_or_default().into_iter().collect();
    let final_batch: BTreeSet<&str> = registered_roles("unlink_batch", 0, DeletedRecordLeaf::OnlyTheDeletedRecord).unwrap_or_default().into_iter().collect();
    let expected_only_two_to_one: Vec<&str> = two_to_one.difference(&final_batch).copied().collect();
    let expected_only_final: Vec<&str> = final_batch.difference(&two_to_one).copied().collect();
    only_two_to_one == expected_only_two_to_one.as_slice() && only_final == expected_only_final.as_slice()
}

fn names_text(names: &[String]) -> String {
    if names.is_empty() { "none".to_string() } else { names.join(",") }
}

/// 一段历史里 2 → 1 那一批与最后一批（会话批，`unlink_batch`）的角色集合差：（2 → 1 那一段、最后一批那一段、2 → 1 独有、最后一批独有）。
fn final_batch_difference(judgement: &HistoryJudgement) -> Option<(&SegmentAnalysis, &SegmentAnalysis, Vec<String>, Vec<String>)> {
    let (_, two_to_one) = deletion_segment_with_units_after(judgement, "unlink_batch", 1)?;
    let (_, final_batch) = deletion_segment_with_units_after(judgement, "unlink_batch", 0)?;
    let two_to_one_identities = two_to_one.identities();
    let final_identities = final_batch.identities();
    let only_two_to_one = names_of_identities(two_to_one, &two_to_one_identities.difference(&final_identities).cloned().collect());
    let only_final = names_of_identities(final_batch, &final_identities.difference(&two_to_one_identities).cloned().collect());
    Some((two_to_one, final_batch, only_two_to_one, only_final))
}

fn row_six_seven_eight_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    // 第 4 行（原第 6 行，跑前登记 e172-r2 第一节 1.2、第六节 6.3）：R6.a = H2 的意图与续做批里有这条记录的两批；
    // R6.b = H3 的意图与 3 → 2、2 → 1 两批。最后一批不写这条记录（照 L03-7 判），本来就没有 L03-4、L03-5 行。
    let mut counts: BTreeMap<&str, (usize, usize)> = BTreeMap::from([("r6a", (0, 0)), ("r6b", (0, 0))]);
    for row in judgements.iter().flat_map(|judgement| judgement.cell_rows.iter()) {
        let line_name = match row.history.as_str() {
            "h2" => "r6a",
            "h3" => "r6b",
            _ => continue,
        };
        if row.cell != "L03-4" && row.cell != "L03-5" {
            continue;
        }
        lines.push(format!(
            "name={line_name} cell={} history={} txg={} publish={} expected={} actual={} equal={} discriminable={}",
            row.cell,
            row.history,
            row.txg,
            row.publish_kind,
            row.expected,
            row.actual,
            row.is_equal(),
            row.discriminable
        ));
        let entry = counts.entry(line_name).or_insert((0, 0));
        if row.discriminable {
            entry.0 += usize::from(!row.is_equal());
        } else {
            entry.1 += 1;
        }
    }
    // R6.b 的前提（V6 也核）：H3 意图的调用方时间与顺序写的时间不同，意图那一格才分辨得开。
    let mut intent_differs_from_sequential = false;
    if let Some(record_two_kept_judgement) = judgements.iter().find(|judgement| judgement.name == "h3") {
        let configuration = &record_two_kept_judgement.configuration;
        let intent_seconds = configuration.get("intent_write_time_seconds").cloned().unwrap_or_else(|| "absent".to_string());
        let sequential_seconds = configuration.get("sequential_write_time_seconds").cloned().unwrap_or_else(|| "absent".to_string());
        intent_differs_from_sequential = intent_seconds != "absent" && intent_seconds != sequential_seconds;
        lines.push(format!(
            "name=r6b_intent_time history=h3 intent_write_time_seconds={intent_seconds} sequential_write_time_seconds={sequential_seconds} differ={intent_differs_from_sequential}"
        ));
    }
    let (r6a_not_equal, r6a_not_discriminable) = counts["r6a"];
    let (r6b_not_equal, r6b_not_discriminable) = counts["r6b"];
    lines.push(format!(
        "name=verdict row=6 r6a_not_equal_rows={r6a_not_equal} r6a_discriminable_false_rows={r6a_not_discriminable} r6b_not_equal_rows={r6b_not_equal} r6b_discriminable_false_rows={r6b_not_discriminable} r6b_intent_differs_from_sequential={intent_differs_from_sequential}"
    ));
    let mut recorded = 0usize;
    for judgement in judgements {
        recorded += judgement.key_range_lines.iter().filter(|line| line.starts_with("name=r7 ")).count();
        lines.extend(judgement.key_range_lines.iter().cloned());
    }
    lines.push(format!("name=verdict row=7 recorded_nodes={recorded}"));
    // 第 5 行（原第 8 行）：判据只用 R8.c（跑前登记 e172-r2 第六节 6.4）；R8.b 不登记、不报。
    let mut attributed = 0usize;
    let mut misattributed = 0usize;
    for judgement in judgements {
        for row in &judgement.attribution_rows {
            attributed += 1;
            misattributed += usize::from(!row.is_match);
            lines.push(format!(
                "name=r8c history={} step={} device={} kind={} decoded={} exported_segment={} match={}",
                judgement.name,
                row.step,
                row.device,
                row.region_kind.step_kind_name(),
                row.decoded.text(),
                row.exported_segment,
                row.is_match
            ));
        }
    }
    lines.push(format!("name=verdict row=8 attributed_writes={attributed} r8c_misattributed_writes={misattributed}"));
    lines
}

/// S3 另加的一条（跑前登记 e172-r3 第十一节 S3）：写了退役单元的历史恰是 H1、H2。
fn retirement_unit_history_check(judgements: &[HistoryJudgement]) -> (String, bool) {
    let mut histories: Vec<&str> = judgements
        .iter()
        .filter(|judgement| judgement.analysis.segments.iter().any(|segment| segment.units.iter().any(|unit| unit.identity.tree_identifier == RETIREMENT_TREE)))
        .map(|judgement| judgement.name.as_str())
        .collect();
    histories.sort_unstable();
    let matches_the_registered_histories = histories == ["h1", "h2"];
    (
        format!(
            "name=retirement_unit_histories histories={} matches_registered={matches_the_registered_histories}",
            if histories.is_empty() { "none".to_string() } else { histories.join(",") }
        ),
        matches_the_registered_histories,
    )
}

/// 几何敏感性（跑前登记 e172-r3 第八节 8.2 第 2、4 行）：A-tag6 的两套常量互判（H1 的拿去判 H3、H3 的拿去判 H1），
/// 钉值层 H2 对 H1 逐版同不同值。
fn tag_six_geometry_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let named = |name: &str| judgements.iter().find(|judgement| judgement.name == name);
    for (history, other_history) in [("h1", "h3"), ("h3", "h1")] {
        let Some(judgement) = named(history) else { continue };
        let all_equal_to = |constants_of: &str| {
            !judgement.tag_six_rows.is_empty()
                && judgement.tag_six_rows.iter().all(|row| {
                    let registered = registered_committed_reservation_of_publish(constants_of, row.publish);
                    registered.is_some() && registered == row.actual
                })
        };
        let own_green = all_equal_to(history);
        let other_green = all_equal_to(other_history);
        lines.push(format!(
            "name=a_tag6_geometry history={history} own_constants_green={own_green} constants_of={other_history} other_constants_green={other_green} exactly_one_green={}",
            own_green != other_green
        ));
    }
    if let (Some(session), Some(continuation)) = (named("h1"), named("h2")) {
        for publish in [DeletionPublish::Intent, DeletionPublish::BatchLeavingTwoUnits, DeletionPublish::BatchLeavingOneUnit, DeletionPublish::FinalBatch] {
            let actual_of = |judgement: &HistoryJudgement| judgement.tag_six_rows.iter().find(|row| row.publish == publish).and_then(|row| row.actual);
            let (session_actual, continuation_actual) = (actual_of(session), actual_of(continuation));
            lines.push(format!(
                "name=harness_pin_geometry publish={} session_actual={} continuation_actual={} same_value={}",
                publish.name(),
                optional_text(session_actual),
                optional_text(continuation_actual),
                session_actual.is_some() && session_actual == continuation_actual
            ));
        }
    }
    lines
}

/// 几何敏感性（跑前登记 e172-r3 第八节 8.2 第 1 行）：最后一批的两套角色期望（那片叶删空：含 tr@0、tr 不含 t3；叶里还有别的记录：含 t3 不含 tr）
/// 都拿去判 H1 与 H3 的最后一批；恰一套判绿才说明这一格分得出两种形状。
fn final_batch_role_geometry_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    for history in ["h1", "h3"] {
        let Some(judgement) = judgements.iter().find(|judgement| judgement.name == history) else { continue };
        let Some((_, final_batch)) = deletion_segment_with_units_after(judgement, "unlink_batch", 0) else { continue };
        let verbatim = compares_role_names_verbatim(history);
        let comparable = |role: &str| if verbatim { role.to_string() } else { role_name_without_allocation_ordinal(role) };
        let mut actual: Vec<String> = final_batch.role_names().iter().map(|role| comparable(role)).collect();
        actual.sort();
        let green_for = |leaf: DeletedRecordLeaf| {
            let mut expected: Vec<String> = registered_roles("unlink_batch", 0, leaf).unwrap_or_default().iter().map(|role| comparable(role)).collect();
            expected.sort();
            expected == actual
        };
        let only_record_green = green_for(DeletedRecordLeaf::OnlyTheDeletedRecord);
        let other_records_green = green_for(DeletedRecordLeaf::BesideOtherRecords);
        lines.push(format!(
            "name=r3_geometry_roles history={history} txg={} only_record_green={only_record_green} other_records_green={other_records_green} exactly_one_green={}",
            final_batch.label_txg(),
            only_record_green != other_records_green
        ));
    }
    lines
}

/// `retirement_cells_not_equal` 数的那几格（跑前登记 e172-r3 第六节 6.2）：L03-12 的条目数、key 区间与退役树那一条，L03-13 到 L03-17。
fn is_retirement_cell_row(row: &CellRow) -> bool {
    match row.cell {
        "L03-13" | "L03-14" | "L03-15" | "L03-16" | "L03-17" => true,
        "L03-12" => row.field == "entry_count" || row.field == "tree_table_key_range" || row.field == format!("tree{RETIREMENT_TREE}"),
        _ => false,
    }
}

fn optional_text(value: Option<u64>) -> String {
    value.map_or_else(|| "absent".to_string(), |value| value.to_string())
}

/// 问题单第 2 行（C775）的量：钉值层 P-pin、判别臂 R-ret、条款层的计数，各报各的（跑前登记 e172-r3 第五节 5.8 改26、改27、改30，第六节 6.2）。
fn row_c775_lines(judgements: &[HistoryJudgement], positive_controls_failed_in_scope: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pin_not_equal = 0usize;
    let mut committed_reservation_anchor_not_equal = 0usize;
    let mut clause_not_equal = 0usize;
    let mut retirement_cells_not_equal = 0usize;
    let mut retirement_arm_not_discriminating = 0usize;
    let mut retirement_arm_both_expectations_equal = 0usize;
    let mut per_history: Vec<String> = Vec::new();
    for judgement in judgements {
        let history = judgement.name.as_str();
        let compared_with_pins = history_is_compared_with_the_harness_pins(history);
        let mut pin_not_equal_here = 0usize;
        for row in &judgement.tag_six_rows {
            if compared_with_pins {
                let harness_index = harness_index_of(row.publish);
                let pinned = harness_pinned_value(harness_index);
                let equal = pinned.is_some() && pinned == row.actual;
                pin_not_equal_here += usize::from(!equal);
                lines.push(format!(
                    "name=harness_pin history={history} txg={} publish={} harness_index={harness_index} pinned={} actual={} equal={equal}",
                    row.txg,
                    row.publish.name(),
                    optional_text(pinned),
                    row.actual_text()
                ));
            }
            let registered = registered_committed_reservation_of_publish(history, row.publish);
            committed_reservation_anchor_not_equal += usize::from(!(registered.is_some() && registered == row.actual));
            let (with_retirement, without_retirement) = retirement_arm_expectations(row.tombstone_levels, row.retirement_levels, row.live_inodes, row.inode_records);
            let with_equal = Some(with_retirement) == row.actual;
            let without_equal = Some(without_retirement) == row.actual;
            let discriminating = !without_equal;
            retirement_arm_not_discriminating += usize::from(!discriminating);
            retirement_arm_both_expectations_equal += usize::from(with_equal && without_equal);
            lines.push(format!(
                "name=r_ret history={history} txg={} publish={} with_retirement={with_retirement} without_retirement={without_retirement} actual={} with_equal={with_equal} without_equal={without_equal} discriminating={discriminating}",
                row.txg,
                row.publish.name(),
                row.actual_text()
            ));
        }
        // 删除之前那一版（harness 第 1 格）不在窗口里，每段历史报一行（跑前登记 e172-r3 第五节 5.8 改26）。
        lines.push(format!("name=harness_pin_out_of_window history={history} harness_index=1 pinned={} in_window=false", optional_text(harness_pinned_value(1))));
        if compared_with_pins {
            pin_not_equal += pin_not_equal_here;
            per_history.push(format!("pin_not_equal_{history}={pin_not_equal_here}"));
        }
        let final_row_beside_another_record = judgement.tag_six_rows.iter().find(|row| history == "h3" && row.publish == DeletionPublish::FinalBatch);
        if let Some(final_row) = final_row_beside_another_record {
            lines.push(format!(
                "name=harness_pin_same_value history=h3 txg={} actual={} same_value_as_harness_first={}",
                final_row.txg,
                final_row.actual_text(),
                final_row.actual.is_some() && final_row.actual == harness_pinned_value(1)
            ));
        }
        clause_not_equal += judgement.cell_rows.iter().filter(|row| row.discriminable && !row.is_equal()).count();
        let retirement_here = judgement.cell_rows.iter().filter(|row| is_retirement_cell_row(row) && !row.is_equal()).count();
        retirement_cells_not_equal += retirement_here;
        per_history.push(format!("retirement_cells_not_equal_{history}={retirement_here}"));
    }
    lines.extend(tag_six_geometry_lines(judgements));
    lines.extend(final_batch_role_geometry_lines(judgements));
    lines.push(format!(
        "name=verdict row=c775 pin_not_equal={pin_not_equal} committed_reservation_anchor_not_equal={committed_reservation_anchor_not_equal} clause_not_equal={clause_not_equal} retirement_cells_not_equal={retirement_cells_not_equal} retirement_arm_not_discriminating={retirement_arm_not_discriminating} retirement_arm_both_expectations_equal={retirement_arm_both_expectations_equal} positive_controls_failed_in_scope={positive_controls_failed_in_scope} {}",
        per_history.join(" ")
    ));
    lines
}

fn row_two_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut unregistered_total = 0u64;
    let mut nonzero_total = 0u64;
    let mut double_claimed_total = 0u64;
    for judgement in judgements {
        let coverage = &judgement.coverage;
        lines.push(format!(
            "name=r2a history={} total_bytes={} layout03_cell_bytes={} layout01_cell_bytes={} constant_zero_bytes={} unregistered_bytes={} double_claimed_bytes={} sum_matches_total={}",
            judgement.name,
            coverage.total_bytes,
            coverage.layout_three_bytes,
            coverage.layout_one_bytes,
            coverage.constant_zero_bytes,
            coverage.unregistered_bytes,
            coverage.double_claimed_bytes,
            coverage.layout_three_bytes + coverage.layout_one_bytes + coverage.constant_zero_bytes + coverage.unregistered_bytes == coverage.total_bytes
        ));
        for range in &coverage.unregistered_ranges {
            lines.push(format!("name=r2b {range}"));
        }
        lines.push(format!("name=r2c history={} constant_zero_nonzero_bytes={}", judgement.name, coverage.constant_zero_nonzero_bytes));
        for range in &coverage.nonzero_padding_ranges {
            lines.push(format!("name=r2c_range {range}"));
        }
        unregistered_total += coverage.unregistered_bytes;
        nonzero_total += coverage.constant_zero_nonzero_bytes;
        double_claimed_total += coverage.double_claimed_bytes;
    }
    lines.push(format!(
        "name=verdict row=2 unregistered_bytes={unregistered_total} constant_zero_nonzero_bytes={nonzero_total} double_claimed_bytes={double_claimed_total}"
    ));
    lines
}
