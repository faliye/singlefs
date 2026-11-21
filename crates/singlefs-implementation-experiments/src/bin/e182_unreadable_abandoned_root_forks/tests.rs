use super::*;
use crate::helpers_shared_by_the_tests::{small_history, small_specification};

#[test]
fn both_single_device_modes_exit_two_before_building_any_history_without_a_verdict_library() {
    for shape in [
        OverwriteShape::OneUnit,
        OverwriteShape::KeepingTheFirstFileUnits,
    ] {
        assert_eq!(
            run_single_device(&[], shape, None),
            2,
            "{shape:?}：没设 {LIBRARY_ENVIRONMENT_VARIABLE} 退 2"
        );
    }
}

fn is_a_write(operation: &RetainedOperation) -> bool {
    matches!(
        operation.operation.kind,
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
    )
}

/// 被藏根那次发布写下的每个单元、每条 journal 记录、根槽都在读回全零的区间里（未确认档不罩系统配置，已确认档罩每块盘较新的那一槽）。
#[test]
fn the_unreadable_ranges_cover_every_unit_record_and_root_slot_of_the_hidden_publish() {
    let first = run_the_first_instance(&small_specification()).expect("实例 1 跑得完");
    let fixed = fixed_geometry_of_the_pool();
    let (_, unconfirmed_ranges) =
        base_and_fault_ranges(&small_specification(), &first).expect("基镜像建得起来");
    let confirmed_specification = HistorySpecification {
        family: Family::HiddenRoot(Confirmation::Confirmed),
        ..small_specification()
    };
    let (_, confirmed_ranges) =
        base_and_fault_ranges(&confirmed_specification, &first).expect("基镜像建得起来");
    let final_operations = &first.operations[first.last_publish_start..];
    for (kind, label) in [
        (StepKind::UnitWrite, "unit"),
        (StepKind::JournalRecord, "journal_record"),
        (StepKind::RootRecordFua, "root_slot"),
    ] {
        let writes: Vec<(u32, u64, u64)> = final_operations
            .iter()
            .filter(|operation| {
                is_a_write(operation) && fixed.classify(&operation.operation) == kind
            })
            .map(|operation| {
                (
                    operation.operation.device.0,
                    operation.operation.offset.0,
                    operation.operation.length,
                )
            })
            .collect();
        assert!(!writes.is_empty(), "被藏根那次发布里有 {label} 这一种写");
        for (device, offset, length) in writes {
            for ranges in [&unconfirmed_ranges, &confirmed_ranges] {
                assert!(
                    ranges.iter().any(|range| range.label == label
                        && range.device.0 == device
                        && range.offset == offset
                        && range.length == length),
                    "{label} 写（盘 {device}，偏移 {offset}）没被读回全零的区间罩住"
                );
            }
        }
    }
    let count_of = |ranges: &[ReadsZeroRange], label: &str| {
        ranges.iter().filter(|range| range.label == label).count()
    };
    assert_eq!(
        count_of(&unconfirmed_ranges, "root_slot"),
        1,
        "被藏根只有一个根槽写"
    );
    assert_eq!(
        count_of(&unconfirmed_ranges, "newer_system_configuration_slot"),
        0,
        "未确认档不罩系统配置槽"
    );
    assert_eq!(
        count_of(&confirmed_ranges, "newer_system_configuration_slot"),
        2,
        "已确认档罩每块盘较新的那一个系统配置槽"
    );
}

#[test]
fn shard_is_judged_the_same_way_twice_and_counts_the_states_of_its_segment() {
    let history = small_history();
    let first = judge_the_shard(history, 0).expect("第 0 段判得完");
    let second = judge_the_shard(history, 0).expect("第 0 段判得完");
    assert_eq!(first, second, "同一片判两次，计数逐名相同");
    let expected = state_count_of_segment(&history.writes, &history.segments[0]).expect("展得开");
    assert_eq!(first.count_named("states"), expected);
    assert_eq!(
        first.count_named("both.outcome.file_read")
            + first.count_named("both.outcome.no_file")
            + first.count_named("both.outcome.failed_with_root")
            + first.count_named("both.outcome.failed_without_root"),
        expected
    );
    for arm in Arm::EVERY_ARM {
        for device in 0..2 {
            assert_eq!(
                first.count_named(&format!("{}.kept{device}.states", arm.name())),
                expected
            );
        }
    }
}

/// 准入判法的第一行以 `forced` 起头时才交回摘要（最后一段，空白换成下划线）；满足或没声明时不交。
#[test]
fn the_preflight_verdict_line_gives_a_summary_only_when_forced() {
    assert_eq!(
        forced_summary_of_the_preflight_verdict("forced\tinputs-unchanged\ta b\tc d"),
        Some("c_d".to_string())
    );
    assert_eq!(forced_summary_of_the_preflight_verdict("met\tabc123"), None);
    assert_eq!(forced_summary_of_the_preflight_verdict("undeclared"), None);
    assert_eq!(forced_summary_of_the_preflight_verdict(""), None);
}

/// 文件头写了准入与运行条件（在第一行代码之前），`main` 的第一句是 `preflight`（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」）。
#[test]
fn the_source_declares_admission_and_run_condition_before_the_code_and_main_starts_with_preflight()
{
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/bin/e182_unreadable_abandoned_root_forks/main.rs");
    let text = std::fs::read_to_string(&path).expect("preflight 要判的源文件读得到");
    let first_code_line = text
        .lines()
        .position(|line| line.starts_with("#![") || line.starts_with("use "))
        .expect("有第一行代码");
    let header: Vec<&str> = text.lines().take(first_code_line).collect();
    assert_eq!(
        header
            .iter()
            .filter(|line| line.starts_with("// admission: always "))
            .count(),
        1,
        "文件头恰有一行准入"
    );
    assert_eq!(
        header
            .iter()
            .filter(|line| line.trim_end() == "// run-condition: command python3")
            .count(),
        1,
        "文件头恰有一行运行条件"
    );
    let main_marker = ["fn ", "main() {\n"].concat();
    let after_main = text.split(main_marker.as_str()).nth(1).expect("有入口函数");
    assert!(
        after_main.starts_with("    let forced_summary = preflight();\n"),
        "入口的第一句是 preflight"
    );
}
