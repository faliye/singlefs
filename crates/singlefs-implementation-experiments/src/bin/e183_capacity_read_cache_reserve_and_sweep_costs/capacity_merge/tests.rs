use super::*;
use crate::helpers_shared_by_the_tests::{parsed, verdict_line};

#[test]
fn copy_bytes_per_slot_interval_includes_both_edges() {
    assert!(copy_bytes_per_slot_ratio_holds(3_500_000));
    assert!(copy_bytes_per_slot_ratio_holds(4_000_000));
    assert!(!copy_bytes_per_slot_ratio_holds(3_499_999));
    assert!(!copy_bytes_per_slot_ratio_holds(4_000_001));
    assert_eq!(
        bytes_per_slot_per_device_millionths(2 * 211_968 * 29 / 8, 4 * GIBIBYTE),
        3_625_000,
        "4 GiB 盘单元区 211968 槽，每盘每槽 3.625 字节"
    );
}

/// 合成片里进程高水位的缺省值：1 GiB（远在阈值内）。
const SYNTHETIC_HIGH_WATER_MARK_KIBIBYTES: u64 = 1 << 20;

/// 合成的一片 H2：分配耗时 = 10 × 记录数（与容量无关、随记录数线性），发布耗时与发布峰值 ∝ 容量，分配峰值恒 64，
/// 拷贝字节 = 每盘每槽 3.625 字节（A37 区间里）、拷贝耗时 ∝ 容量，PC-平 恒 1 MiB；台是本机，A43 照登记值，
/// PC-高水位前后涨 64 MiB（过），进程高水位 1 GiB（阈值内）。
fn synthetic_capacity_shard_lines(cell: CapacityCell) -> Vec<String> {
    synthetic_capacity_shard_lines_on(cell, "本机", SYNTHETIC_HIGH_WATER_MARK_KIBIBYTES)
}

/// 同 [`synthetic_capacity_shard_lines`]，台与进程高水位（KiB）由调用方给。
fn synthetic_capacity_shard_lines_on(
    cell: CapacityCell,
    host: &str,
    high_water_mark_kibibytes: u64,
) -> Vec<String> {
    let records = 100 + cell.sequentially_written_units;
    let mebibytes = cell.device_bytes / MEBIBYTE;
    let copy_bytes = 2 * unit_area_slots(cell.device_bytes, JOURNAL_RING_DEFAULT_BYTES) * 29 / 8;
    let mut lines = vec![
        format!(
            "E7RESULT name=h2_config shard=synthetic binary_sha256=same unit_area_slots={} host={host} service_attempt=0/5:1 image_filesystem=ext2/ext3",
            registered_unit_area_slots(cell.device_bytes).expect("五档容量都有登记值")
        ),
        "E7RESULT name=h2_steps shard=synthetic outcome=ok".to_string(),
        "E7RESULT name=positive_control_heap shard=synthetic peak_over_baseline_bytes=67108864 holds=true high_water_mark_before_kibibytes=20000 high_water_mark_after_kibibytes=85536 high_water_mark_control_holds=true".to_string(),
        format!("E7RESULT name=h2_allocator_copy shard=synthetic records={records} copy_bytes={copy_bytes} copy_nanoseconds={mebibytes} bytes_per_slot_per_device_millionths=3625000"),
    ];
    for call in 0..5 {
        lines.push(format!("E7RESULT name=h2_allocate shard=synthetic call={call} nanoseconds={} peak_over_baseline_bytes=64 baseline_bytes=7 in_use_after_bytes=7 slots_from_unit_area_start=0", 10 * records));
    }
    lines.push("E7RESULT name=positive_control_flat shard=synthetic nanoseconds=1 peak_over_baseline_bytes=1048576 baseline_bytes=7 in_use_after_bytes=7".to_string());
    for call in 0..5 {
        lines.push(format!("E7RESULT name=h2_publish shard=synthetic call={call} outcome=ok floor_raises=0 nanoseconds={mebibytes} peak_over_baseline_bytes={} baseline_bytes=7 in_use_after_bytes=7 records_after={records}", copy_bytes));
    }
    lines.push(format!(
        "E7RESULT name=h2_process shard=synthetic vm_high_water_mark_kibibytes={high_water_mark_kibibytes}"
    ));
    lines
}

/// 一格一轮的合成结局：台与进程高水位由调用方给，其余同 [`synthetic_capacity_shard_lines`]。
fn synthetic_status_on(
    cell: CapacityCell,
    host: &str,
    high_water_mark_kibibytes: u64,
) -> CellRoundStatus {
    status_of_a_capacity_shard(
        &parsed(&synthetic_capacity_shard_lines_on(
            cell,
            host,
            high_water_mark_kibibytes,
        )),
        "same",
    )
}

/// 合成的 45 片，`replace` 交回 `Some` 的那几格一轮换成它给的结局。
fn synthetic_statuses_with(
    replace: impl Fn(CapacityCell, u64) -> Option<CellRoundStatus>,
) -> CapacityStatuses {
    synthetic_capacity_statuses()
        .into_iter()
        .map(|(cell, round, status)| (cell, round, replace(cell, round).unwrap_or(status)))
        .collect()
}

/// 合并时给「贴着阈值」用的程序文件字节数（合成）。
const SYNTHETIC_BINARY_BYTES: Option<u64> = Some(30 << 20);

fn synthetic_capacity_statuses() -> CapacityStatuses {
    capacity_shards()
        .into_iter()
        .map(|(_, cell, round)| {
            (
                cell,
                round,
                status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "same"),
            )
        })
        .collect()
}

#[test]
fn capacity_shard_reads_back_every_measured_field() {
    let cell = capacity_cells()[1];
    let status = status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "same");
    assert_eq!(
        status.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::WithinTheThreshold
    );
    let reading = status.reading().expect("合成的一片应当读成「阈值内跑完」");
    let facts = status.facts().expect("片做完就有读数");
    assert_eq!(facts.host.as_deref(), Some("本机"));
    assert_eq!(facts.service_attempt.as_deref(), Some("0/5:1"));
    assert_eq!(facts.image_filesystem.as_deref(), Some("ext2/ext3"));
    assert_eq!(facts.high_water_mark_kibibytes, Some(1 << 20));
    assert_eq!(
        facts.high_water_mark(),
        HighWaterMark::Usable { bytes: GIBIBYTE }
    );
    assert_eq!(reading.records, 100);
    assert_eq!(reading.allocate.len(), 5);
    assert_eq!(reading.publish.len(), 5);
    assert_eq!(
        CapacityQuantity::AllocateNanoseconds.round_median(reading),
        Some(1000)
    );
    assert_eq!(
        CapacityQuantity::PublishNanoseconds.round_median(reading),
        Some(65536)
    );
    assert_eq!(
        CapacityQuantity::FlatControlPeakOverBaselineBytes.round_median(reading),
        Some(1 << 20)
    );
}

#[test]
fn capacity_shard_with_a_missing_line_or_another_binary_is_not_measured_rather_than_zero() {
    let cell = capacity_cells()[0];
    let mut lines = synthetic_capacity_shard_lines(cell);
    lines.retain(|line| !line.contains("name=h2_allocator_copy"));
    let status = status_of_a_capacity_shard(&parsed(&lines), "same");
    assert_eq!(status.text(), "not_measured:fields_missing");
    assert_eq!(status.reading(), None);
    assert_eq!(
        status.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::NoClass
    );
    assert_eq!(
        status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "other"),
        CellRoundStatus::BinaryOtherThanTheMerge("same".to_string())
    );
}

#[test]
fn round_whose_publish_failed_gives_no_publish_quantity() {
    let cell = capacity_cells()[0];
    let lines: Vec<String> = synthetic_capacity_shard_lines(cell)
        .into_iter()
        .map(|line| {
            if line.contains("call=2 outcome=ok") {
                line.replace("outcome=ok", "outcome=failed:NoSpace")
            } else {
                line
            }
        })
        .collect();
    let status = status_of_a_capacity_shard(&parsed(&lines), "same");
    let reading = status.reading().expect("量到了");
    assert_eq!(
        CapacityQuantity::PublishNanoseconds.round_median(reading),
        None
    );
    assert_eq!(
        CapacityQuantity::PublishPeakOverBaselineBytes.round_median(reading),
        None
    );
    assert_eq!(
        CapacityQuantity::AllocateNanoseconds.round_median(reading),
        Some(1000)
    );
}

#[test]
fn rounds_that_differ_in_a_deterministic_field_are_named() {
    let cell = capacity_cells()[0];
    let first = status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "same");
    let changed_lines: Vec<String> = synthetic_capacity_shard_lines(cell)
        .into_iter()
        .map(|line| {
            if line.contains("name=h2_allocate shard=synthetic call=4") {
                line.replace("peak_over_baseline_bytes=64", "peak_over_baseline_bytes=65")
            } else {
                line
            }
        })
        .collect();
    let second = status_of_a_capacity_shard(&parsed(&changed_lines), "same");
    let readings = [
        first.reading().expect("量到了"),
        second.reading().expect("量到了"),
    ];
    assert_eq!(
        fields_differing_across_rounds(&readings),
        vec!["allocate_peaks"]
    );
    assert!(fields_differing_across_rounds(&[readings[0], readings[0]]).is_empty());
}

#[test]
fn capacity_summary_judges_every_row_of_answer_known_synthetic_readings() {
    let mut emitter = Emitter::collecting_lines();
    let end = emit_capacity_summary(
        &mut emitter,
        &synthetic_capacity_statuses(),
        SYNTHETIC_BINARY_BYTES,
    );
    assert_eq!(end, MergeEnd::Complete);
    let lines = emitter.into_collected_lines();
    let verdict = verdict_line(&lines, "h2_summary");
    let expected = [
        ("r2a_class", "basically_unrelated"),
        ("r2b_class", "linear"),
        ("r2d_allocate_class", "basically_unrelated"),
        ("r2d_publish_class", "linear"),
        ("r2e_allocate_class", "linear"),
        ("r2e_publish_class", "x5_as_registered_cannot_be_made"),
        ("r2e_publish_peak_class", "x5_as_registered_cannot_be_made"),
        ("r2c_class", "linear"),
        ("completed_capacities", "5"),
        ("fewer_than_four_capacities", "false"),
        ("positive_control_linear_copy_bytes_holds", "true"),
        ("positive_control_linear_copy_time_holds", "true"),
        ("positive_control_flat_holds", "true"),
        ("heap_control_failed_shards", "0"),
        ("unstable_slope_rows", "none"),
        ("predictions_contradicted", "none"),
        ("discrimination_self_proof_holds", "true"),
        ("hidden_state_cells", "0"),
        ("cell_rounds_within_the_threshold", "45"),
        ("cell_rounds_beyond_the_memory_threshold", "0"),
        ("cell_rounds_undeterminable", "0"),
        ("cell_rounds_filesystem_refused", "0"),
        ("cell_rounds_voided_high_water_mark", "0"),
        ("cell_rounds_near_the_threshold", "0"),
        ("ladder_rounds_across_hosts", "0"),
        ("unit_area_slots_differing_cell_rounds", "0"),
        ("largest_depends_on_the_threshold", "false"),
        ("threshold_discrimination_self_proof_holds", "true"),
    ];
    for (key, value) in expected {
        assert_eq!(verdict.field(key), Some(value), "{key}");
    }
    let largest = parsed(&lines)
        .into_iter()
        .find(|line| line.name() == Some("h2_largest_disk"))
        .expect("有这一行");
    assert_eq!(
        largest.field("largest_is_the_ladder_endpoint"),
        Some("true")
    );
    let occupancy: Vec<ResultLine> = parsed(&lines)
        .into_iter()
        .filter(|line| line.name() == Some("h2_occupancy_sensitivity"))
        .collect();
    assert_eq!(occupancy.len(), 2);
    let agreement_of = |row: &str| {
        occupancy
            .iter()
            .find(|line| line.field("row") == Some(row))
            .and_then(|line| line.field("agreement"))
    };
    assert_eq!(agreement_of("r2a"), Some("two_points_agree"));
    assert_eq!(
        agreement_of("r2b"),
        Some("not_judged_x5_as_registered_cannot_be_made"),
        "占用格上的发布不是登记的 X5：不拿它同基线比类"
    );
}

#[test]
fn the_publish_rows_on_cells_with_many_units_have_no_class_while_the_ladder_keeps_its_class() {
    let statuses = synthetic_capacity_statuses();
    let judgement_of = |row: &str| {
        capacity_slope_rows()
            .into_iter()
            .find(|slope_row| slope_row.row == row)
            .map(|slope_row| judge_slope_row(&statuses, slope_row))
            .expect("行名取自 capacity_slope_rows")
    };
    for row in ["r2e_publish", "r2e_publish_peak", "r2b_at_occupancy"] {
        let judgement = judgement_of(row);
        assert_eq!(
            judgement.class,
            ClassAcrossRounds::RegisteredPublishCannotBeMade,
            "{row}"
        );
        assert!(
            judgement.slopes.iter().all(Option::is_none),
            "{row} 不当 0 也不出斜率"
        );
    }
    for row in ["r2b", "r2d_publish"] {
        assert_eq!(
            judgement_of(row).class,
            ClassAcrossRounds::Stable(SlopeClass::Linear),
            "{row}"
        );
    }
    assert_eq!(
        judgement_of("r2e_allocate").class,
        ClassAcrossRounds::Stable(SlopeClass::Linear),
        "分配那一行不受 X5 影响"
    );
}

#[test]
fn cell_value_line_says_whether_its_publish_is_the_registered_overwrite_inside_data_unit_zero() {
    let mut emitter = Emitter::collecting_lines();
    emit_capacity_summary(
        &mut emitter,
        &synthetic_capacity_statuses(),
        SYNTHETIC_BINARY_BYTES,
    );
    let cells: Vec<ResultLine> = parsed(&emitter.into_collected_lines())
        .into_iter()
        .filter(|line| line.name() == Some("h2_cell_values"))
        .collect();
    assert_eq!(cells.len(), 9);
    for line in &cells {
        let units = line.number("sequentially_written_units").expect("有单元数");
        assert_eq!(
            line.truth("publish_is_the_registered_single_unit_overwrite"),
            Some(units == 0),
            "{:?}",
            line.field("cell")
        );
    }
}

#[test]
fn deterministic_field_that_changes_between_rounds_stops_the_capacity_merge() {
    let statuses: CapacityStatuses = synthetic_capacity_statuses()
        .into_iter()
        .map(|(cell, round, status)| {
            let changed = match status {
                CellRoundStatus::Finished(FinishedCellRound {
                    facts,
                    measured: Ok(mut reading),
                }) if round == 3 && cell == capacity_cells()[0] => {
                    reading.copy_bytes += 1;
                    CellRoundStatus::Finished(FinishedCellRound {
                        facts,
                        measured: Ok(reading),
                    })
                }
                unchanged @ (CellRoundStatus::Finished(_)
                | CellRoundStatus::FilesystemRefused { .. }
                | CellRoundStatus::ServiceGaveUpAtTheMaximum { .. }
                | CellRoundStatus::NotRunAfterAnotherRoundWentBeyond
                | CellRoundStatus::NotMeasured(_)
                | CellRoundStatus::Missing
                | CellRoundStatus::BinaryOtherThanTheMerge(_)) => unchanged,
            };
            (cell, round, changed)
        })
        .collect();
    let mut emitter = Emitter::collecting_lines();
    let end = emit_capacity_summary(&mut emitter, &statuses, SYNTHETIC_BINARY_BYTES);
    assert!(matches!(end, MergeEnd::Stopped { .. }), "{end:?}");
    assert_eq!(
        verdict_line(&emitter.into_collected_lines(), "h2_summary").field("hidden_state_cells"),
        Some("1")
    );
}

#[test]
fn high_water_mark_one_mebibyte_under_the_threshold_is_already_beyond_its_lower_edge() {
    assert_eq!(MEMORY_THRESHOLD_BYTES, 25_769_803_776);
    assert_eq!(
        memory_threshold_lower_edge(MEMORY_THRESHOLD_BYTES) + 67_108_864,
        25_769_803_776
    );
    let cell = capacity_cells()[0];
    let just_under =
        synthetic_status_on(cell, "本机", (MEMORY_THRESHOLD_BYTES - MEBIBYTE) / KIBIBYTE);
    assert_eq!(
        just_under.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::BeyondTheMemoryThreshold
    );
    assert_eq!(just_under.text(), "beyond_the_memory_threshold");
    assert_eq!(just_under.reading(), None, "阈值之外的读数不算结果");
    let at_the_lower_edge = synthetic_status_on(
        cell,
        "本机",
        memory_threshold_lower_edge(MEMORY_THRESHOLD_BYTES) / KIBIBYTE,
    );
    assert_eq!(
        at_the_lower_edge.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::WithinTheThreshold,
        "恰好等于下沿算阈值内"
    );
}

#[test]
fn high_water_mark_of_25_gibibytes_read_in_kibibytes_is_beyond_the_threshold() {
    let status = synthetic_status_on(capacity_cells()[0], "本机", 26_214_400);
    assert_eq!(
        status.facts().map(CellRoundFacts::high_water_mark),
        Some(HighWaterMark::Usable {
            bytes: 26_843_545_600
        })
    );
    assert_eq!(
        status.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::BeyondTheMemoryThreshold
    );
}

#[test]
fn service_outcome_below_the_threshold_is_undeterminable_and_at_the_threshold_is_beyond() {
    let below = status_of_a_service_outcome(
        "type=memory-cap-at-maximum reached=21743271936 peak=none task=7 shard=0/5 attempt=3\n",
    );
    assert_eq!(
        below,
        CellRoundStatus::ServiceGaveUpAtTheMaximum {
            reached_bytes: 21_743_271_936
        }
    );
    assert_eq!(
        below.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::Undeterminable
    );
    assert_eq!(
        below.text(),
        "undeterminable_by_the_service:reached_bytes_21743271936"
    );
    let at = status_of_a_service_outcome(
        "type=memory-cap-at-maximum reached=25769803776 peak=25000000000 task=7 shard=4/5 attempt=9",
    );
    assert_eq!(
        at.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::BeyondTheMemoryThreshold
    );
    for unreadable in [
        "type=stalled reached=1",
        "type=memory-cap-at-maximum reached=none",
        "garbage",
    ] {
        assert_eq!(
            status_of_a_service_outcome(unreadable),
            CellRoundStatus::NotMeasured("service_outcome_unreadable".to_string()),
            "{unreadable}"
        );
    }
}

#[test]
fn an_unfinished_shard_reads_its_service_outcome_and_the_rest_of_a_beyond_cell_is_not_run() {
    // 守卫认领临时目录：用例做完、断言失败、panic 都删掉它
    let directory = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e183-h2-service-outcome-{}", std::process::id())),
    );
    std::fs::create_dir_all(&*directory).expect("临时目录");
    let beyond_cell = capacity_cells()[4];
    let undeterminable_cell = capacity_cells()[3];
    let shard_of = |cell: CapacityCell, round: u64| format!("{}-round-{round}", cell.name());
    std::fs::write(
        directory.join(format!("{}.service-outcome", shard_of(beyond_cell, 1))),
        "type=memory-cap-at-maximum reached=52613349376 peak=none task=7 shard=0/5 attempt=4\n",
    )
    .expect("写结局文件");
    std::fs::write(
        directory.join(format!(
            "{}.service-outcome",
            shard_of(undeterminable_cell, 1)
        )),
        "type=memory-cap-at-maximum reached=21743271936 peak=none task=7 shard=0/5 attempt=4\n",
    )
    .expect("写结局文件");
    let statuses = read_capacity_statuses(&directory, "same");
    assert_eq!(
        status_of(&statuses, beyond_cell, 1),
        Some(&CellRoundStatus::ServiceGaveUpAtTheMaximum {
            reached_bytes: 52_613_349_376
        })
    );
    assert_eq!(
        status_of(&statuses, beyond_cell, 2),
        Some(&CellRoundStatus::NotRunAfterAnotherRoundWentBeyond),
        "同一格已有一轮在阈值之外：别的轮够判后未跑"
    );
    assert_eq!(
        status_of(&statuses, undeterminable_cell, 2),
        Some(&CellRoundStatus::Missing),
        "判不出不让别的轮停跑"
    );
    assert_eq!(
        status_of(&statuses, capacity_cells()[0], 1),
        Some(&CellRoundStatus::Missing)
    );
}

#[test]
fn largest_capacity_with_one_round_beyond_the_threshold_leaves_four_completed_capacities() {
    let endpoint = ladder_cells()[4];
    let statuses = synthetic_statuses_with(|cell, round| {
        (cell == endpoint && round == 2).then(|| synthetic_status_on(cell, "本机", 26_214_400))
    });
    assert_eq!(
        largest_completed_capacity(&statuses, MEMORY_THRESHOLD_BYTES),
        Some(4 * TEBIBYTE)
    );
    let mut emitter = Emitter::collecting_lines();
    emit_capacity_summary(&mut emitter, &statuses, SYNTHETIC_BINARY_BYTES);
    let lines = emitter.into_collected_lines();
    let verdict = verdict_line(&lines, "h2_summary");
    assert_eq!(verdict.field("completed_capacities"), Some("4"));
    assert_eq!(verdict.field("fewer_than_four_capacities"), Some("false"));
    assert_eq!(verdict.field("r2b_class"), Some("linear"), "四档照样判出类");
    assert_eq!(
        verdict.field("cell_rounds_beyond_the_memory_threshold"),
        Some("1")
    );
    let line_named = |name: &str| {
        parsed(&lines)
            .into_iter()
            .find(|line| {
                line.name() == Some(name)
                    && line
                        .field("cell")
                        .is_none_or(|cell| cell == endpoint.name())
            })
            .expect("有这一行")
    };
    let largest = line_named("h2_largest_disk");
    assert_eq!(
        largest.field("largest_completed_mebibytes"),
        Some("4194304")
    );
    assert_eq!(
        largest.field("beyond_the_memory_threshold_mebibytes"),
        Some("16777215")
    );
    assert_eq!(
        line_named("h2_cell_values").field("rounds_measured"),
        Some("0"),
        "有一轮在阈值之外的格，5 轮都不算结果"
    );
}

#[test]
fn round_whose_ladder_ran_on_two_hosts_gives_no_r2a_slope_in_that_round() {
    let terabyte = ladder_cells()[2];
    let statuses = synthetic_statuses_with(|cell, round| {
        (cell == terabyte && round == 3)
            .then(|| synthetic_status_on(cell, "第二台", SYNTHETIC_HIGH_WATER_MARK_KIBIBYTES))
    });
    let judgement_of = |row: &str| {
        capacity_slope_rows()
            .into_iter()
            .find(|slope_row| slope_row.row == row)
            .map(|slope_row| judge_slope_row(&statuses, slope_row))
            .expect("行名取自 capacity_slope_rows")
    };
    for row in ["r2a", "r2b"] {
        let judgement = judgement_of(row);
        assert!(judgement.slopes[2].is_none(), "{row} 第 3 轮跨台不出 b");
        assert_eq!(judgement.slopes.iter().flatten().count(), 4, "{row}");
        assert_eq!(
            judgement.class,
            ClassAcrossRounds::NotJudged {
                rounds_with_a_slope: 4
            }
        );
    }
    assert!(
        judgement_of("r2d_allocate").slopes[2].is_some(),
        "堆量不看台"
    );
    let mut emitter = Emitter::collecting_lines();
    emit_capacity_summary(&mut emitter, &statuses, SYNTHETIC_BINARY_BYTES);
    let lines = parsed(&emitter.into_collected_lines());
    let verdict = lines
        .iter()
        .find(|line| line.name() == Some("verdict") && line.field("item") == Some("h2_summary"))
        .expect("有判决行");
    assert_eq!(verdict.field("ladder_rounds_across_hosts"), Some("1"));
    let round_three = lines
        .iter()
        .find(|line| line.name() == Some("h2_round_hosts") && line.field("round") == Some("3"))
        .expect("有第 3 轮那一行");
    assert_eq!(round_three.field("ladder_hosts"), Some("本机,第二台"));
    assert_eq!(round_three.field("one_host"), Some("false"));
    let by_host: Vec<&ResultLine> = lines
        .iter()
        .filter(|line| {
            line.name() == Some("h2_cell_values_by_host")
                && line.field("cell") == Some(terabyte.name().as_str())
        })
        .collect();
    assert_eq!(by_host.len(), 2, "R2f 按台各报一行");
    assert_eq!(by_host[1].field("host"), Some("第二台"));
    assert_eq!(by_host[1].field("rounds_measured"), Some("1"));
}

#[test]
fn the_high_water_mark_control_fails_unless_the_high_water_mark_rose_by_sixty_mebibytes() {
    assert_eq!(
        HIGH_WATER_MARK_CONTROL_MINIMUM_BYTES + 4 * MEBIBYTE,
        HEAP_POSITIVE_CONTROL_BYTES
    );
    assert!(!high_water_mark_control_holds(20_000, 20_000));
    assert!(high_water_mark_control_holds(20_000, 20_000 + 61_440));
    assert!(!high_water_mark_control_holds(20_000, 20_000 + 61_439));
    let lines: Vec<String> = synthetic_capacity_shard_lines(capacity_cells()[0])
        .into_iter()
        .map(|line| {
            line.replace(
                "high_water_mark_after_kibibytes=85536",
                "high_water_mark_after_kibibytes=20000",
            )
        })
        .collect();
    let status = status_of_a_capacity_shard(&parsed(&lines), "same");
    assert_eq!(status.text(), "voided_high_water_mark_control_failed");
    assert_eq!(
        status.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::NoClass
    );
}

#[test]
fn an_unreadable_high_water_mark_voids_the_round_instead_of_reading_as_zero() {
    let unreadable: Vec<String> = synthetic_capacity_shard_lines(capacity_cells()[0])
        .into_iter()
        .map(|line| {
            if line.contains("name=h2_process") {
                "E7RESULT name=h2_process shard=synthetic vm_high_water_mark_kibibytes=none"
                    .to_string()
            } else {
                line
            }
        })
        .collect();
    let without_the_line: Vec<String> = synthetic_capacity_shard_lines(capacity_cells()[0])
        .into_iter()
        .filter(|line| !line.contains("name=h2_process"))
        .collect();
    for lines in [unreadable, without_the_line] {
        let status = status_of_a_capacity_shard(&parsed(&lines), "same");
        assert_eq!(status.text(), "voided_high_water_mark_unreadable");
        assert_eq!(
            status.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
            MemoryThresholdClass::NoClass
        );
        assert_eq!(status.reading(), None);
    }
}

#[test]
fn cell_whose_unit_area_slots_differ_from_the_registered_value_stops_the_capacity_merge() {
    for (device_bytes, slots) in REGISTERED_UNIT_AREA_SLOTS_BY_DEVICE_BYTES {
        assert_eq!(
            unit_area_slots(device_bytes, JOURNAL_RING_DEFAULT_BYTES),
            slots
        );
    }
    assert_eq!(registered_unit_area_slots(4 * GIBIBYTE), Some(211_968));
    assert_eq!(
        registered_unit_area_slots(16 * TEBIBYTE - MEBIBYTE),
        Some(1_073_691_584)
    );
    let cell = capacity_cells()[2];
    let registered = registered_unit_area_slots(cell.device_bytes).expect("有登记值");
    let statuses = synthetic_statuses_with(|status_cell, round| {
        (status_cell == cell && round == 1).then(|| {
            let lines: Vec<String> = synthetic_capacity_shard_lines(cell)
                .into_iter()
                .map(|line| {
                    line.replace(
                        &format!("unit_area_slots={registered} "),
                        &format!("unit_area_slots={} ", registered - 1),
                    )
                })
                .collect();
            status_of_a_capacity_shard(&parsed(&lines), "same")
        })
    });
    let mut emitter = Emitter::collecting_lines();
    let end = emit_capacity_summary(&mut emitter, &statuses, SYNTHETIC_BINARY_BYTES);
    assert_eq!(
        end,
        MergeEnd::Stopped {
            reason: "t9_unit_area_slots_differ_from_the_registered_values".to_string()
        }
    );
    assert_eq!(
        verdict_line(&emitter.into_collected_lines(), "h2_summary")
            .field("unit_area_slots_differing_cell_rounds"),
        Some("1")
    );
}

#[test]
fn the_largest_disk_follows_the_threshold_and_moving_it_across_a_real_high_water_mark_flips_it() {
    // 容量阶梯五档的进程高水位：1、2、8、20、30 GiB（KiB 记）；别的格照缺省 1 GiB。
    let gibibytes_by_capacity = [1u64, 2, 8, 20, 30];
    let ladder = ladder_cells();
    let statuses = synthetic_statuses_with(|cell, _| {
        let index = ladder.iter().position(|ladder_cell| *ladder_cell == cell)?;
        Some(synthetic_status_on(
            cell,
            "本机",
            gibibytes_by_capacity[index] << 20,
        ))
    });
    assert_eq!(
        largest_completed_capacity(&statuses, MEMORY_THRESHOLD_BYTES),
        Some(4 * TEBIBYTE)
    );
    assert_eq!(
        largest_completed_capacity(&statuses, MEMORY_THRESHOLD_BYTES / 2),
        Some(TEBIBYTE)
    );
    assert_eq!(
        largest_completed_capacity(&statuses, 2 * MEMORY_THRESHOLD_BYTES),
        Some(16 * TEBIBYTE - MEBIBYTE)
    );
    let moved = memory_threshold_moved_across_a_real_high_water_mark(&statuses)
        .expect("真读数里挪得出一处");
    assert_eq!(moved.capacity_device_bytes, 4 * GIBIBYTE);
    assert_eq!(moved.just_within_threshold_bytes, GIBIBYTE + 64 * MEBIBYTE);
    assert_eq!(moved.largest_just_within, Some(4 * GIBIBYTE));
    assert_eq!(moved.largest_just_beyond, None);
    assert!(synthetic_high_water_marks_fall_on_two_sides_of_the_threshold());
    let mut emitter = Emitter::collecting_lines();
    emit_capacity_summary(&mut emitter, &statuses, SYNTHETIC_BINARY_BYTES);
    let lines = parsed(&emitter.into_collected_lines());
    let dependence = lines
        .iter()
        .find(|line| line.name() == Some("h2_threshold_dependence"))
        .expect("有这一行");
    assert_eq!(
        dependence.field("largest_at_half_mebibytes"),
        Some("1048576")
    );
    assert_eq!(
        dependence.field("largest_at_registered_mebibytes"),
        Some("4194304")
    );
    assert_eq!(
        dependence.field("largest_at_double_mebibytes"),
        Some("16777215")
    );
    assert_eq!(
        dependence.field("switches_between"),
        Some("half_and_registered,registered_and_double")
    );
    let self_proof = lines
        .iter()
        .find(|line| {
            line.name() == Some("discrimination_self_proof")
                && line.field("judgement") == Some("h2_largest_disk")
        })
        .expect("有自证 ④ 那一行");
    assert_eq!(self_proof.field("used"), Some("real_high_water_marks"));
    assert_eq!(self_proof.field("holds"), Some("true"));
}

#[test]
fn high_water_mark_within_the_binary_size_and_the_margin_of_the_threshold_is_marked_near_it() {
    let cell = capacity_cells()[0];
    let binary_bytes = SYNTHETIC_BINARY_BYTES.expect("合成值");
    let edge_kibibytes =
        (MEMORY_THRESHOLD_BYTES + binary_bytes + MEMORY_THRESHOLD_MARGIN_BYTES) / KIBIBYTE;
    let near_of = |kibibytes: u64, binary: Option<u64>| {
        synthetic_status_on(cell, "本机", kibibytes)
            .facts()
            .and_then(|facts| facts.is_near_the_threshold(binary))
    };
    assert_eq!(near_of(edge_kibibytes, SYNTHETIC_BINARY_BYTES), Some(true));
    assert_eq!(
        near_of(edge_kibibytes + 1, SYNTHETIC_BINARY_BYTES),
        Some(false)
    );
    assert_eq!(
        near_of(SYNTHETIC_HIGH_WATER_MARK_KIBIBYTES, SYNTHETIC_BINARY_BYTES),
        Some(false)
    );
    assert_eq!(
        near_of(edge_kibibytes, None),
        None,
        "程序文件大小读不到不判"
    );
}

#[test]
fn shard_whose_images_could_not_be_made_reads_as_filesystem_refused_on_its_host() {
    let lines: Vec<String> = synthetic_capacity_shard_lines_on(
        capacity_cells()[4],
        "第二台",
        SYNTHETIC_HIGH_WATER_MARK_KIBIBYTES,
    )
    .into_iter()
    .map(|line| {
        line.replace(
            "name=h2_steps shard=synthetic outcome=ok",
            "name=h2_steps shard=synthetic outcome=filesystem_refused:FileTooLarge",
        )
    })
    .collect();
    let status = status_of_a_capacity_shard(&parsed(&lines), "same");
    assert_eq!(
        status.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
        MemoryThresholdClass::FilesystemRefused
    );
    assert_eq!(status.text(), "filesystem_refused:FileTooLarge");
    assert_eq!(
        status.facts().and_then(|facts| facts.host.as_deref()),
        Some("第二台")
    );
    assert_eq!(status.reading(), None);
}

#[test]
fn the_service_attempt_line_gives_the_largest_high_water_mark_and_the_peak_the_service_must_reach()
{
    let mut emitter = Emitter::collecting_lines();
    emit_capacity_summary(
        &mut emitter,
        &synthetic_capacity_statuses(),
        SYNTHETIC_BINARY_BYTES,
    );
    let attempts: Vec<ResultLine> = parsed(&emitter.into_collected_lines())
        .into_iter()
        .filter(|line| line.name() == Some("h2_service_attempt"))
        .collect();
    assert_eq!(attempts.len(), 1, "合成的 45 格都记同一次尝试");
    let attempt = &attempts[0];
    assert_eq!(attempt.field("service_attempt"), Some("0/5:1"));
    assert_eq!(attempt.field("hosts"), Some("本机"));
    assert_eq!(attempt.number("cells_finished"), Some(45));
    assert_eq!(
        attempt.number("high_water_mark_maximum_bytes"),
        Some(GIBIBYTE)
    );
    assert_eq!(attempt.number("binary_bytes"), Some(30 << 20));
    assert_eq!(
        attempt
            .number("service_peak_needed_at_least_bytes")
            .map(|needed| needed + (30 << 20) + 67_108_864),
        Some(GIBIBYTE)
    );
}
