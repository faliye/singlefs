//! journal（I-8 类）：走读内核逐盘 journal 组判按段走之后的两支，没提交的组之后还有记录、一次发布带两条末条标志。

use super::*;

// ─── 走读内核逐盘 journal 组判按段走（E11）之后的两支：I-8.8「没提交的组之后本实例还有记录」、I-8.10「恰一条带末条标志才比根」 ───

/// journal 记录头里事务号（8 字节）、提交标记（1 字节）、记录标志（1 字节）的偏移（D23（journal 的角色与格式） 已定项 17 的字段表；
/// 走读内核 `order_the_records_of_device_by_group` 取事务号 78，`judge_commit_markers_of_device` 读提交标记 86，I-8.9 与 I-8.10 读记录标志 7）。
const JOURNAL_RECORD_TRANSACTION_OFFSET: usize = 78;

const JOURNAL_RECORD_COMMIT_MARKER_OFFSET: usize = 86;

const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;

/// 提交标记字节取 1 是带提交标记、取 0 是不带（D23（journal 的角色与格式） 已定项 17；只许这两个值）。
const JOURNAL_RECORD_COMMIT_MARKER_PRESENT: u8 = 1;

const JOURNAL_RECORD_COMMIT_MARKER_ABSENT: u8 = 0;

/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。
const JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;

/// 新根段里树 ID 水位 8 字节的段内偏移（树表指针 86、映射根指针 86 之后）。
const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK: usize = 172;

/// 三个数据单元那条流（[`record_new_pool_through_a_three_unit_file`]）里新建文件那次发布（实例 1、txg 3）与顺序写那次发布（实例 1、txg 4）。
const CHECKPOINT_TXG_OF_THE_FILE_CREATION: u64 = 3;

const CHECKPOINT_TXG_OF_THE_THREE_RECORD_PUBLISH: u64 = 4;

/// I-8.8 判据④（一组一条提交标记都不带时，本实例之后不许再有记录）：三个数据单元那条流上，新建文件那次发布（实例 1、txg 3，一条记录）
/// 两块盘各一份的提交标记写成 0、重封、同实例后面的记录重链。这一组一条提交标记都不带，同一块盘上同一实例之后还有顺序写那次发布（txg 4）的三条记录，
/// CPU 池级 checker 红 I-8.8 判据④（walk.rs `judge_commit_markers_per_transaction`）。
/// 走读内核按 (实例, 事务号) 排好次序之后，这一判比的是「本实例在表里最靠后那一条的下标 > 组末的下标」（`crash_judge_checker.wgsl`
/// `judge_commit_markers_of_device`）：比较写反或永不成立时，txg 3 那条记录与 txg 4 的记录都落了的状态上 GPU 不红而 CPU 红，对拍字不同。
fn record_three_unit_file_whose_file_creation_record_carries_no_commit_marker(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_three_unit_file(
        tag,
        "three-unit-file-creation-record-without-commit-marker",
    );
    let records_of_the_file_creation =
        journal_record_writes_of_the_publish(&flow, CHECKPOINT_TXG_OF_THE_FILE_CREATION);
    assert_eq!(
        records_of_the_file_creation.len(),
        2,
        "流 {}：新建文件那次发布一条记录、两块盘各一份",
        flow.name
    );
    let records_of_the_three_record_publish =
        journal_record_writes_of_the_publish(&flow, CHECKPOINT_TXG_OF_THE_THREE_RECORD_PUBLISH);
    for (index, counter) in &records_of_the_file_creation {
        let write = &flow.writes[*index];
        let bytes = write.bytes().expect("记录写带字节");
        assert_ne!(
            read_u64_at(bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
            0,
            "流 {}：新建文件那次发布的事务号不是 0（事务号 0 的空发布不进 I-8.8 的组）",
            flow.name
        );
        assert_eq!(
            bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET], JOURNAL_RECORD_COMMIT_MARKER_PRESENT,
            "流 {}：改之前新建文件那条记录带提交标记",
            flow.name
        );
        assert!(
            records_of_the_three_record_publish
                .iter()
                .any(|(later_index, later_counter)| {
                    flow.writes[*later_index].device == write.device && later_counter > counter
                }),
            "流 {}：同一块盘上同一实例在新建文件那条记录之后还有顺序写那次发布的记录",
            flow.name
        );
    }
    for (index, _) in records_of_the_file_creation {
        rewrite_journal_record_and_reseal(&mut flow, index, &|bytes| {
            bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] = JOURNAL_RECORD_COMMIT_MARKER_ABSENT;
        });
    }
    flow
}

/// I-8.8 判据④：[`record_three_unit_file_whose_file_creation_record_carries_no_commit_marker`]。CPU 一侧先钉流造对了：整条落盘的镜像上
/// I-8.8 只红在判据④（判据③与提交标记字节不红）；再整份比，GPU 逐状态与 CPU 相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_uncommitted_transaction_is_followed_by_more_records_of_its_instance(
) {
    let flow = record_three_unit_file_whose_file_creation_record_carries_no_commit_marker(
        "crash-judge-gpu-i88-uncommitted-transaction-then-more-records",
    );
    let mut image_with_every_write_landed = flow.base.clone();
    image_with_every_write_landed.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_write_landed, "I-8.8") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.starts_with("判据④：") && !detail.contains("；"),
            "流 {}：整条落盘的镜像上 I-8.8 只红在判据④，实际：{detail}",
            flow.name
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "流 {}：整条落盘的镜像上 I-8.8 要红在判据④，实际 {other_verdict:?}",
            flow.name
        ),
    }
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_uncommitted_transaction_is_followed_by_more_records_of_its_instance",
        &flow,
        "I-8.8",
    );
}

/// I-8.10 记录与根那一半只比带末条标志恰一条的组：三个数据单元那条流上，顺序写那次发布（实例 1、txg 4，三条记录）两块盘各一份的第一条记录也带上末条标志、
/// 三条记录新根段里的树 ID 水位都加 1，都重封、同实例后面的记录重链。那次发布带末条标志的有两条：CPU 池级 checker 红 I-8.9（多于一条末条），
/// I-8.10 记录与根那一半不比这一组（walk.rs `judge_new_root_segment_of_the_last_record_against_the_roots`：末条标志不唯一的组归 I-8.9）。
/// 三条记录一起改水位是因为 I-8.10 往非末条扩的那一半（C738（同一次发布几条记录的新根段一致没有不变量判））不给末条标志不唯一的组让路：
/// 条文写「同（实例代号, `checkpoint_txg`）的每条记录（一次发布写出的全部记录，带不带末条标志都算），新根段整段逐字节相同」，
/// 只改末条的水位时组内三条不相同，那一半在 CPU 与 GPU 两侧都红 I-8.10，「至少一条就比」的变异就分不出来了；三条一起改，组内照旧相同、那一半不红，
/// 红不红只看记录与根那一半比不比。
/// 走读内核按 (实例, checkpoint_txg) 排好次序之后数一遍这一组带末条标志的，恰一条才比（`crash_judge_checker.wgsl`
/// `judge_new_root_segment_of_the_last_record_against_the_roots_of_device`）：放成「至少一条就比」时，它拿段里最后那条带标志的
/// （水位加了 1）去比 (1, 4) 的根，三条记录与那条根都落了的状态上 GPU 红 I-8.10 而 CPU 不红，对拍字不同。
fn record_three_unit_file_whose_three_record_publish_carries_two_last_record_flags(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_three_unit_file(
        tag,
        "three-unit-file-three-record-publish-with-two-last-record-flags",
    );
    let records =
        journal_record_writes_of_the_publish(&flow, CHECKPOINT_TXG_OF_THE_THREE_RECORD_PUBLISH);
    assert_eq!(
        records.len(),
        6,
        "流 {}：顺序写那次发布切成三条记录、两块盘各一份",
        flow.name
    );
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    let last_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .max()
        .expect("顺序写那次发布有记录");
    assert_eq!(
        last_counter - first_counter,
        2,
        "流 {}：三条记录的计数器连着",
        flow.name
    );
    for (index, counter) in &records {
        let flags = flow.writes[*index].bytes().expect("记录写带字节")[JOURNAL_RECORD_FLAGS_OFFSET];
        assert_eq!(
            flags,
            if *counter == last_counter {
                JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH
            } else {
                0
            },
            "流 {}：改之前只有末条（计数器 {last_counter}）带末条标志，计数器 {counter} 那条的记录标志是 {flags:#010b}",
            flow.name
        );
    }
    for (index, counter) in records {
        rewrite_journal_record_and_reseal(&mut flow, index, &|bytes| {
            if counter == first_counter {
                bytes[JOURNAL_RECORD_FLAGS_OFFSET] |=
                    JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH;
            }
            let watermark =
                JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK;
            bytes[watermark] = bytes[watermark].wrapping_add(1);
        });
    }
    flow
}

/// I-8.10 末条标志不唯一的组不比：[`record_three_unit_file_whose_three_record_publish_carries_two_last_record_flags`]。CPU 一侧先钉流造对了：
/// 整条落盘的镜像上 I-8.9 红在「多于一条末条」、I-8.10 不红（三条记录新根段相同，往非末条扩的那一半成立；水位加了 1 的这一组带两条末条标志，
/// 记录与根那一半不比）；再整份比，GPU 逐状态与 CPU 相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_with_two_last_record_flags_is_not_compared_with_its_root(
) {
    let flow = record_three_unit_file_whose_three_record_publish_carries_two_last_record_flags(
        "crash-judge-gpu-i810-two-last-record-flags",
    );
    let mut image_with_every_write_landed = flow.base.clone();
    image_with_every_write_landed.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_write_landed, "I-8.9") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("多于一条末条："),
            "流 {}：整条落盘的镜像上 I-8.9 红在多于一条末条，实际：{detail}",
            flow.name
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "流 {}：整条落盘的镜像上 I-8.9 要红在多于一条末条，实际 {other_verdict:?}",
            flow.name
        ),
    }
    let verdict_of_the_new_root_segment =
        pool_checker_verdict_on_the_image(&image_with_every_write_landed, "I-8.10");
    assert!(
        !matches!(verdict_of_the_new_root_segment, InvariantVerdict::Violated(_)),
        "流 {}：整条落盘的镜像上 I-8.10 不比带两条末条标志的那一组、不红，实际 {verdict_of_the_new_root_segment:?}",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_with_two_last_record_flags_is_not_compared_with_its_root",
        &flow,
        "I-8.9",
    );
}

/// I-7.15：两块盘择到的系统配置设备数不同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_chosen_system_configurations_disagree_on_the_device_count(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_chosen_system_configurations_disagree_on_the_device_count",
        &record_new_pool_with_a_system_configuration_of_another_device_count("crash-judge-gpu-i715"),
        "I-7.15",
    );
}

/// I-4.1：重放施加出来的那一版走不下去。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_replayed_version_cannot_be_walked() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_replayed_version_cannot_be_walked",
        &record_new_pool_with_a_replayed_version_whose_tree_table_cannot_be_read(
            "crash-judge-gpu-i41",
        ),
        "I-4.1",
    );
}

/// I-7.13（系统配置池级字段在读者收的范围里）、C663（系统配置槽越界与不收之值两读者分类不同）：系统配置槽自述 S = 17，整池不作保；GPU 走读内核把这一槽读成带不收之值、不是「这一槽不可择」，逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_seventeen_slots_per_region(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_seventeen_slots_per_region",
        &record_new_pool_with_seventeen_slots_per_region("crash-judge-gpu-i713-slots-per-region"),
        "I-7.13",
    );
}

/// I-7.13（系统配置池级字段在读者收的范围里）、C663（系统配置槽越界与不收之值两读者分类不同）：系统配置槽自述 R = 2，整池不作保；GPU 走读内核同判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_two_regions() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_two_regions",
        &record_new_pool_with_two_regions("crash-judge-gpu-i713-regions"),
        "I-7.13",
    );
}

/// I-2.4（C728）：系统配置槽字段表之后有一个非 0 字节；GPU 走读内核对每个可择的槽判 [489, 4096) 全 0，逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_carries_a_non_zero_byte_past_its_field_table(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_carries_a_non_zero_byte_past_its_field_table",
        &record_new_pool_with_a_non_zero_last_byte_in_a_system_configuration_slot(
            "crash-judge-gpu-i24-system-configuration-tail",
        ),
        "I-2.4",
    );
}

/// I-8.1 三个字段那一支：系统配置槽里的 F 不是格式常量，整池不作保；I-1.13 两边都报环几何三格不等整池拒那一句（码 27）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_records_a_safety_factor_other_than_the_format_constant(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_and_ending_the_scan_rebuild_with(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_records_a_safety_factor_other_than_the_format_constant",
        &record_new_pool_with_a_safety_factor_of_two("crash-judge-gpu-i81-fields"),
        "I-8.1",
        "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS",
    );
}

/// 两支整池不作保都中（槽距读者不收、F 不是格式常量）：走 I-7.13 那一支，CPU 只红 I-7.13、一个状态都不红 I-8.1，GPU 逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot(
) {
    let flow = record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
        "crash-judge-gpu-i713-over-i81",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[
            states_whose_scan_rebuild_ends_with(
                "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT",
            ),
            states_whose_scan_rebuild_ends_with(
                "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS",
            ),
        ],
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-7.13")
            && !comparison.cpu_checker_red_invariants.contains("I-8.1"),
        "流 {} 上 CPU 池级 checker 红 I-7.13、不红 I-8.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    // I-1.13 跟着走带越界值整池拒那一支（码 26），一个状态都不报环几何三格不等那一句（码 27）；对拍逐位比这一字
    assert!(
        comparison.examined[0].states_counted > 0 && comparison.examined[1].states_counted == 0,
        "流 {} 上 I-1.13 报带越界值整池拒那一句、不报环几何三格不等那一句",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// I-8.5：全池一个自证过的系统配置槽都没有；I-1.13 两边都报一块盘都交不出有效系统配置那一句（码 28）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_no_system_configuration_slot_verifies_itself() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_and_ending_the_scan_rebuild_with(
        "the_gpu_judge_agrees_with_the_cpu_when_no_system_configuration_slot_verifies_itself",
        &record_new_pool_with_every_system_configuration_slot_zeroed_at_the_end(
            "crash-judge-gpu-i85",
        ),
        "I-8.5",
        "SCAN_REBUILD_OUTCOME_NO_DEVICE_YIELDS_A_VALID_SYSTEM_CONFIGURATION",
    );
}

/// I-7.5：根槽不是声明的 io_min 的整数倍。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_declared_minimum_input_output(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_declared_minimum_input_output",
        &record_new_pool_with_a_minimum_input_output_the_root_slots_are_not_aligned_to("crash-judge-gpu-i75"),
        "I-7.5",
    );
}
