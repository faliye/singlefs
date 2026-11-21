//! 六条流与改坏一处重封过的那几条流上逐状态对拍，根槽第三十二次写之后的差别与一次派发占几条暂存道的用例。

use super::*;

/// 场景：新池新建文件（合法流）的每个状态。预期：GPU 判出的红位、池级 checker 违例位、两遍恢复的结局与落到的根、对拍字与 CPU 逐状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_file_creation() {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_file_creation",
        &|| {
            (
                record_new_pool_file_creation("crash-judge-gpu-healthy", "new-pool"),
                RedTheCpuShowsOnTheFlow::NoneRequired,
            )
        },
    );
}

/// 场景：新池新建文件、发布的根指着的树表两份副本各改坏一字节（根落了的状态走读失败）的每个状态。预期：GPU 判出的红位、池级 checker 违例位、两遍恢复的结局与落到的根、对拍字与 CPU 逐状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_tree_table_is_broken()
{
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_tree_table_is_broken",
        &|| (record_new_pool_with_a_broken_tree_table("crash-judge-gpu-broken"), RedTheCpuShowsOnTheFlow::TheCpuJudge),
    );
}

/// 场景：新池新建文件、发布那条 journal 记录的提交标记改成 2（重封）的每个状态。预期：GPU 判出的红位、池级 checker 违例位、两遍恢复的结局与落到的根、对拍字与 CPU 逐状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_journal_record_carries_a_commit_byte_of_two(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_journal_record_carries_a_commit_byte_of_two",
        &|| (record_new_pool_with_a_commit_byte_of_two("crash-judge-gpu-commit-2"), RedTheCpuShowsOnTheFlow::ThePoolChecker),
    );
}

/// 场景：新池新建文件、发布那条 journal 记录的记录标志改成 0b11（重封）的每个状态。预期：GPU 判出的红位、池级 checker 违例位、两遍恢复的结局与落到的根、对拍字与 CPU 逐状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_journal_record_carries_flags_of_three(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_journal_record_carries_flags_of_three",
        &|| (record_new_pool_with_record_flags_of_three("crash-judge-gpu-flags-3"), RedTheCpuShowsOnTheFlow::ThePoolChecker),
    );
}

/// 场景：新池新建文件、最后一次系统配置槽写的固定结构槽距改成 512（重封，两遍恢复都没择到根）的每个状态。预期：GPU 判出的红位、池级 checker 违例位、两遍恢复的结局与落到的根、对拍字与 CPU 逐状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_system_configuration_carries_a_refused_slot_spacing(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_system_configuration_carries_a_refused_slot_spacing",
        &|| (record_new_pool_with_a_refused_slot_spacing("crash-judge-gpu-spacing-512"), RedTheCpuShowsOnTheFlow::TheCpuJudgeAndThePoolChecker),
    );
}

/// 场景：固定脚本到回退 D（合法流）的每个状态。预期：GPU 判出的红位、池级 checker 违例位、两遍恢复的结局与落到的根、对拍字与 CPU 逐状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_the_fixed_script_through_a_rollback() {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_the_fixed_script_through_a_rollback",
        &|| (record_fixed_script_through_rollback("crash-judge-gpu-rollback"), RedTheCpuShowsOnTheFlow::NoneRequired),
    );
}

/// 场景：新池新建文件、inode 记录的 blocks 字段加 1（I-9.15）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_inode_blocks_are_one_too_many(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_inode_blocks_are_one_too_many",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::InodeBlocksPlusOne);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、数据单元五元组里的对象出生代加 1（I-9.10）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_data_unit_object_birth_is_one_too_late(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_data_unit_object_birth_is_one_too_late",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::DataUnitObjectBirthPlusOne);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、分配记录叶第一条记录的分配代加 1（I-3.10）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_allocation_generation_is_one_too_late(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_allocation_generation_is_one_too_late",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::AllocationGenerationPlusOne);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、分配记录叶第二条记录的槽号改成第一条的（I-5.4）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_allocation_records_overlap(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_allocation_records_overlap",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::AllocationRecordsOverlap);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、记账叶第一条的值加 1（I-3.1）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_accounting_value_is_one_too_many(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_accounting_value_is_one_too_many",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::AccountingValuePlusOne);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、树表前两条条目的树 ID 互换（I-9.16）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_tree_table_is_out_of_order(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_tree_table_is_out_of_order",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::TreeTableOutOfOrder);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、映射叶第一条 key 的出生 txg 加 1（I-1.11）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_mapping_key_birth_is_one_too_late(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_mapping_key_birth_is_one_too_late",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::MappingKeyBirthPlusOne);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：固定脚本到回退、最后一版实例表第一行的实例代号加 1（I-3.8）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_rollback_whose_instance_row_is_not_below_the_mount(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_rollback_whose_instance_row_is_not_below_the_mount",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::RollbackInstanceRowNotBelowTheMount);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：新池新建文件、数据单元只改一份副本的载荷首字节（I-1.8）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_data_unit_copies_differ(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_new_pool_whose_data_unit_copies_differ",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::DataUnitCopiesDiffer);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：固定脚本到回退、最后一版树表里 inode 树条目的诞生 txg 加 1（I-9.14）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_rollback_whose_tree_table_birth_txg_is_one_too_late(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_rollback_whose_tree_table_birth_txg_is_one_too_late",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::RollbackTreeTableBirthTxgPlusOne);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 场景：固定脚本到回退、最后一版分配记录叶里带已释放标志的第一条记录的释放代加 8（I-3.9）的每个状态。预期：CPU 池级 checker 红它管的那一条，GPU 逐状态整份相同（分得出差别的自证）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_rollback_whose_release_generation_is_eight_too_late(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_a_rollback_whose_release_generation_is_eight_too_late",
        &|| {
            let (flow, invariant) =
                record_the_resealed_field_world(ResealedFieldWorld::RollbackReleaseGenerationPlusEight);
            (flow, RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant))
        },
    );
}

/// 一个根槽上发布的根是第 33 次写（`record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times`）：发布的根落了与没落的
/// 两个版本组合掩码低 32 位相同、只差高位，字节也不同（一份是发布的根、一份全零）；GPU 判器在这条流的每个状态上与 CPU 判器红位逐位相同、
/// 落到同一条根。判器查版本只比低 32 位时，发布的根落了的状态被读成全零那一版、恢复落到暖机的根，这条红
/// （C601（判器查版本不比组合掩码高 32 位时没有会红的流））。表上的前提在取卡之前判，没卡的机器上也判到。
#[test]
fn the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write(
) {
    const EXPAND_UP_TO: usize = 8;
    const {
        assert!(
            FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT > EXPAND_UP_TO,
            "垫的那一段不展开（展开就是 2^32 个状态）"
        );
    }
    let flow = record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times(
        "crash-judge-gpu-past-32-writes",
    );
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("一个落点 64 次写以内，建得出表");
    let publish_root = &flow.writes[flow.judged_root_index];
    let root_slot = tables
        .locations
        .iter()
        .find(|location| {
            location.device == publish_root.device.0 && location.offset == publish_root.offset.0
        })
        .expect("发布的根槽是判器表里的一个落点");
    assert_eq!(
        root_slot.writes.len(),
        FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT + 1,
        "这个根槽上是垫的 32 次写加发布的根那一次"
    );
    let told_apart_only_by_the_high_bits = root_slot.versions.iter().any(|first| {
        root_slot.versions.iter().any(|second| {
            first.combination != second.combination
                && first.combination & 0xFFFF_FFFF == second.combination & 0xFFFF_FFFF
                && first.bytes != second.bytes
        })
    });
    assert!(
        told_apart_only_by_the_high_bits,
        "这个根槽要有两个低 32 位相同、只差高位、字节不同的版本，不然只比低 32 位的内核也判得对：{:?}",
        root_slot
            .versions
            .iter()
            .map(|version| version.combination)
            .collect::<Vec<_>>()
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, EXPAND_UP_TO) else {
        return;
    };
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

/// 重封到根之后，发布那条 journal 记录要自洽：头校验和、载荷校验和对得上，每个点名项的两份位置条目都与全部写落盘的镜像上那份单元的
/// 整单元 CRC 相同——这是 `corrupt_units_and_reseal_up_to_the_root` 的自证（它错了，必红流红的就不只是被改的那个字段）。
#[test]
fn resealed_publish_record_names_units_that_match_on_disk() {
    use singlefs_core::recovery::PoolReader;
    let flow = record_new_pool_with_a_resealed_field(
        "crash-judge-gpu-diagnose-reseal",
        "new-pool-data-unit-object-birth-plus-one",
        &|bytes| unit_class(bytes) == 1,
        false,
        &|bytes| {
            bytes[59] = bytes[59].wrapping_add(1);
        },
    );
    let mut pool = flow.base.clone();
    pool.apply_writes(&flow.writes);
    let record_index = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("有记录写");
    let record = flow.writes[record_index].bytes().expect("记录写带字节");
    let named = usize::try_from(u32::from_le_bytes(record[12..16].try_into().expect("4")))
        .expect("点名项数");
    assert!(named >= 1, "发布的记录点名了单元");
    assert!(
        singlefs_core::checksum::wide_checksum_field_holds(record, 4096, 46),
        "记录头校验和重封过"
    );
    let payload_crc = crc32c(
        &record[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    assert_eq!(
        payload_crc,
        u32::from_le_bytes(record[95..99].try_into().expect("4")),
        "记录载荷校验和重封过"
    );
    for entry in 0..named {
        let named_entry_offset =
            JOURNAL_RECORD_HEADER_BYTES + entry * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD;
        let class = record[named_entry_offset + 28];
        let length = if class == 2 { 16384 } else { 32768 };
        for copy in 0..2 {
            let location = named_entry_offset + copy * 14;
            let device = u32::from_le_bytes(record[location..location + 4].try_into().expect("4"));
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&record[location + 4..location + 10]);
            let slot = u64::from_le_bytes(slot);
            let stored =
                u32::from_le_bytes(record[location + 10..location + 14].try_into().expect("4"));
            let on_disk = PoolReader::read(
                &pool,
                singlefs_core::address::DeviceIdentity(device),
                singlefs_core::address::DeviceOffsetInBytes(slot * 16384),
                length,
            )
            .map(|bytes| crc32c(&bytes));
            assert_eq!(
                on_disk,
                Some(stored),
                "点名项 {entry} 第 {copy} 份（类 {class}、盘 {device}、槽 {slot}）的校验和与盘上那份单元不符"
            );
        }
    }
}

/// 一次派活跨几条草稿缓冲（每条各派一次活、排在同一趟里）与一次派活只用一条缓冲，判出来的结论逐状态相同。
/// 流取固定脚本到回退 D、展开 16（131 195 态；这条流的段不是 6 次写以内就是 16 次以上，展开 14 只有 125 态、跨不过一条缓冲）；
/// 这张卡的显存额度装不下两条草稿缓冲就是环境不够，报出来，不悄悄只比一条。
/// 一条缓冲那一档与 CPU 判器的逐状态对拍在 `the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_the_fixed_script_through_a_rollback`。
#[test]
fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several() {
    const EXPAND_UP_TO: usize = 16;
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
    ) else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-lanes");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
        &tables,
    ) else {
        return;
    };
    // 两个判器同时活着：借的是窄的那一档（256 个）加宽的那一档（三条缓冲）各自要的，两份表各算一份（约 6.45 GiB）
    let states_per_lane = states_of_one_scratch_lane_on_every_card_of(cards);
    let Some(lease) = cards.lease_card_or_report_not_run(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
        mebibytes_a_judge_needs(&packed, 256)
            + mebibytes_a_judge_needs(&packed, 3 * states_per_lane),
    ) else {
        return;
    };
    let card = lease.card();
    assert_eq!(
        states_per_scratch_lane(card.device().limits().max_storage_buffer_binding_size),
        states_per_lane,
        "借到的卡一条草稿缓冲装的状态数与借之前按池里最小的绑定上限估的不同：借的量对不上这张卡"
    );
    let Some((narrow, wide)) = on_the_card_or_report_not_run_when_out_of_memory(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
        card,
        "建两个判器与判这条流",
        || {
            let within_one_lane = GpuCrashJudge::on_card_with_packed_tables(card, &packed, 256);
            let across_lanes =
                GpuCrashJudge::on_card_with_packed_tables(card, &packed, 3 * states_per_lane);
            println!(
                "CRASH_JUDGE_LANES states={state_count} states_per_lane={states_per_lane} states_per_dispatch_within_one_lane={} states_per_dispatch_across_lanes={}",
                within_one_lane.states_per_dispatch(),
                across_lanes.states_per_dispatch()
            );
            assert!(
                within_one_lane.states_per_dispatch() <= states_per_lane,
                "窄的那一档一次派活只用一条草稿缓冲"
            );
            assert!(
                across_lanes.states_per_dispatch() > states_per_lane,
                "这张卡的显存额度装不下两条草稿缓冲（一次派活只带得了 {} 个状态，一条缓冲 {states_per_lane} 个）：腾出显存再跑",
                across_lanes.states_per_dispatch()
            );
            assert!(
                state_count > 2 * u64::try_from(states_per_lane).expect("装得进 u64"),
                "这条流只有 {state_count} 个状态，跨不过两条草稿缓冲"
            );
            (
                within_one_lane.judge_states(0..state_count),
                across_lanes.judge_states(0..state_count),
            )
        },
    ) else {
        return;
    };
    let first_difference = narrow
        .iter()
        .zip(&wide)
        .position(|(narrow_words, wide_words)| narrow_words != wide_words);
    assert_eq!(
        first_difference, None,
        "跨几条草稿缓冲判出来的结论与一条缓冲的不同，第一个不同的状态序号在这里"
    );
    assert_eq!(narrow.len(), wide.len());
}
