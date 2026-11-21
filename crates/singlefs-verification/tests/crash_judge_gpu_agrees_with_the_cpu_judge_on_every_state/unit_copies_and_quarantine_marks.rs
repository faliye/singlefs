//! 单元副本与隔离（I-1.8 两句、I-3.11 的隔离豁免）：数据单元几份副本里一份载荷不同、inode 叶两份副本只差出生序号、隔离记号。

use super::*;

// ───────────── E14：GPU 扫描内核 I-1.8 两句与走读内核位置项校验和那张表各一个敏感的取样点 ─────────────

/// I-1.8 ①「同一组的成员载荷相同」的一个敏感取样点：三个数据单元那条流（[`record_new_pool_through_a_three_unit_file`]）上，
/// 最后一个数据单元只改盘号小的那一份副本的载荷首字节、重封到根（与 `new-pool-data-unit-copies-differ` 同一形）。CPU 判 I-1.8 红。
/// GPU 扫描内核收扫描项的次序是落点表的 (盘, 偏移) 升序：码 1 的扫描项在一块盘上不止一个时，被改的那个单元的两份副本在收的次序里
/// 隔着别的数据单元，只有按归并键排过序才挨在一起、归成一组；新池新建文件那条流上码 1 只有一个数据单元，两份副本按收的次序本来就挨着，
/// 排没排序看不出来。
fn record_three_unit_file_with_one_copy_of_its_last_data_unit_carrying_another_payload(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-one-copy-of-the-last-data-unit-carries-another-payload";
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    let is_a_data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let targets = last_unit_copies(&flow, name, &is_a_data_unit, true);
    let changed_device = flow.writes[targets[0]].device;
    let changed_offset = flow.writes[targets[0]].offset;
    // 收的次序里两份副本之间夹着别的数据单元：被改的那一盘上偏移更大的，或另一块盘上偏移更小的
    let data_units_between_the_two_copies = flow
        .writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(is_a_data_unit)
                && ((write.device == changed_device && write.offset.0 > changed_offset.0)
                    || (write.device != changed_device && write.offset.0 < changed_offset.0))
        })
        .count();
    assert!(
        data_units_between_the_two_copies >= 1,
        "流 {name}：按 (盘, 偏移) 收的次序里，被改的数据单元的两份副本之间要夹着别的数据单元（这条流要的就是这一点）"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        bytes[134] ^= 0xFF;
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_one_copy_of_a_data_unit_among_several_carries_another_payload(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_one_copy_of_a_data_unit_among_several_carries_another_payload",
        &record_three_unit_file_with_one_copy_of_its_last_data_unit_carrying_another_payload(
            "crash-judge-gpu-i18-among-three-data-units",
        ),
        "I-1.8",
    );
}

/// 码 3 单元头里的出生序号（4 字节，偏移 103）：在码 3 的归并键 [93, 107) 那一段里，不在全序键（诞生代号 [73, 81)、实例与事务号 [93, 103)）里。
const PACKED_UNIT_BIRTH_SEQUENCE_OFFSET: usize = 103;

/// I-1.8 ②「同 key 的各组两两全序键不等」那一支：新池新建文件的流上 inode 叶容器只改盘号大的那一份副本的出生序号（加 1）、重封到根。
/// 两份副本的归并键不同、各成一组，对象 key 与全序键都相同：CPU 判 I-1.8 第二句红（两份载荷相同，第一句成立）。
/// GPU 扫描内核按换过次序的归并键（对象 key、全序键、其余）排，这两组挨着，相邻两组的公共前缀到出生序号那一段才断（第 57 个字节），
/// 比相邻两组的那一句要认出「对象 key 连着全序键都相同」。
fn record_new_pool_with_the_second_copy_of_the_inode_leaf_carrying_another_birth_sequence(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-second-copy-of-the-inode-leaf-carries-another-birth-sequence";
    let mut flow = record_new_pool_file_creation(tag, name);
    let copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    let second_copy = *copies
        .iter()
        .max_by_key(|index| flow.writes[**index].device.0)
        .expect("一个单元两份副本");
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &[second_copy], &|bytes| {
        let birth_sequence = read_u32_at(bytes, PACKED_UNIT_BIRTH_SEQUENCE_OFFSET);
        bytes[PACKED_UNIT_BIRTH_SEQUENCE_OFFSET..PACKED_UNIT_BIRTH_SEQUENCE_OFFSET + 4]
            .copy_from_slice(&birth_sequence.wrapping_add(1).to_le_bytes());
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_copies_of_an_inode_leaf_split_into_two_groups_of_one_total_order_key(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_copies_of_an_inode_leaf_split_into_two_groups_of_one_total_order_key",
        &record_new_pool_with_the_second_copy_of_the_inode_leaf_carrying_another_birth_sequence(
            "crash-judge-gpu-i18-two-groups-of-one-total-order-key",
        ),
        "I-1.8",
    );
}

/// 分配记录叶：key 宽 10（设备 4、槽号 6）、条目 20（key、跨度 2 含已释放标志、分配代或释放代 8）。
pub(super) const ALLOCATION_LEAF_RECORD_BYTES: usize = 20;

/// 记账树里「defer 待释放」那一项（D5（快照 / 空间记账机制） 已定项 4 第 5 项，`singlefs_core::records::STATISTIC_DEFER_QUEUE_BYTES`）。
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;

/// 改回仍分配的那两条记录带不带隔离记号（[`record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined`]）。
#[derive(Clone, Copy, Debug)]
enum QuarantineMarkOfTheRecords {
    /// 两块盘那两条都带记号（跨度段 0x4000）：写者照 D19（块指针的结构与宽度预算） 已定项 5 隔离的样子，整组豁免。
    OnBothDevices,
    /// 两条都不带：与「写者把一个单元泄漏在已分配里」逐字节同形，只认记号就不豁免。
    Absent,
}

/// 隔离豁免那一支（I-3.1 / I-3.11 的隔离的记录，D19（块指针的结构与宽度预算） 已定项 5 只认记号；池级 checker `quarantined_slots_exempted_per_device`，
/// GPU 扫描内核 `c_quarantine_exempted`、读分配记录那一行的记号位）：
/// 三个数据单元的文件发完意图（txg 5）与按批大小 1 的前两批（txg 6 放掉第三个单元、txg 7 放掉第二个单元）。txg 7 那一版改成「写者把放掉的
/// 第二个单元隔离了」：分配记录叶里那个单元两块盘上的记录改回第一批那一版的样子（仍分配、分配代），按 `mark` 带不带隔离记号，记账叶里两块盘的
/// defer 待释放各减一个单元，重封到根。最新根不引用它、更早的候选根（txg 4、5、6）引用它，判 I-3.11 那一遍它是一条已分配而没有根引用的记录，
/// 两条都带记号时豁免、I-3.11 成立，不带时不豁免、I-3.11 红（已分配多一个单元）。I-3.1 那一遍走的是全部候选根，这个单元有根引用，不进隔离。
fn record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined(
    tag: &str,
    mark: QuarantineMarkOfTheRecords,
) -> RecordedFlow {
    let name = match mark {
        QuarantineMarkOfTheRecords::OnBothDevices => {
            "second-deletion-batch-quarantined-data-unit-records-carry-the-mark"
        }
        QuarantineMarkOfTheRecords::Absent => {
            "second-deletion-batch-quarantined-data-unit-records-carry-no-mark"
        }
    };
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    let after_the_first_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    let after_the_second_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    let released_by_the_second_batch: Vec<_> = after_the_first_batch
        .data_pointers
        .iter()
        .filter(|pointer| !after_the_second_batch.data_pointers.contains(pointer))
        .copied()
        .collect();
    assert_eq!(
        released_by_the_second_batch.len(),
        1,
        "按批大小 1 的第二批放掉一个数据单元（第一批之后 {} 个、第二批之后 {} 个）",
        after_the_first_batch.data_pointers.len(),
        after_the_second_batch.data_pointers.len()
    );
    let quarantined = released_by_the_second_batch[0];
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    );

    // 分配记录：第二批那一版两块盘上那一条（已释放、释放代）改回第一批那一版的（仍分配、分配代）
    let record_of = |records: &[singlefs_core::allocator::AllocationRecord],
                     location: &singlefs_core::pointer::LocationEntry| {
        *records
            .iter()
            .find(|record| record.device == location.device && record.slot == location.slot)
            .unwrap_or_else(|| panic!("那一版的分配记录里有 {location:?} 那一条"))
    };
    let record_replacements: Vec<(Vec<u8>, Vec<u8>)> = quarantined
        .locations
        .iter()
        .map(|location| {
            let released = record_of(&after_the_second_batch.allocation_records, location);
            let allocated = record_of(&after_the_first_batch.allocation_records, location);
            assert!(
                released.is_released && !allocated.is_released,
                "第二批那一版这一条已释放、第一批那一版仍分配：{released:?} / {allocated:?}"
            );
            let quarantined_record = singlefs_core::allocator::AllocationRecord {
                device: allocated.device,
                slot: allocated.slot,
                span_slots: allocated.span_slots,
                generation: allocated.generation,
                is_released: false,
                quarantine_mark: match mark {
                    QuarantineMarkOfTheRecords::OnBothDevices => {
                        singlefs_core::allocator::AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck
                    }
                    QuarantineMarkOfTheRecords::Absent => {
                        singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined
                    }
                },
            };
            (released.to_bytes(), quarantined_record.to_bytes())
        })
        .collect();
    let records_of_an_allocation_leaf = |bytes: &[u8]| -> Vec<usize> {
        if unit_class(bytes) != 2
            || usize::from(index_node_key_width(bytes)) != ALLOCATION_RECORD_TREE_KEY_BYTES
            || usize::from(index_node_entry_width(bytes)) != ALLOCATION_LEAF_RECORD_BYTES
            || bytes[50] != 0
        {
            return Vec::new();
        }
        let entry_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(ALLOCATION_RECORD_TREE_KEY_BYTES),
        ));
        (0..entry_count)
            .map(|index| {
                index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES)
                    + index * ALLOCATION_LEAF_RECORD_BYTES
            })
            .collect()
    };
    let leaves_with_the_released_records: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| {
                    records_of_an_allocation_leaf(bytes).iter().any(|record| {
                        record_replacements.iter().any(|(released, _)| {
                            bytes[*record..*record + ALLOCATION_LEAF_RECORD_BYTES] == released[..]
                        })
                    })
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !leaves_with_the_released_records.is_empty(),
        "流 {name} 里第二批那一版的分配记录叶写着那两条已释放的记录"
    );
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &leaves_with_the_released_records,
        &|bytes| {
            for record in records_of_an_allocation_leaf(bytes) {
                for (released, allocated) in &record_replacements {
                    if bytes[record..record + ALLOCATION_LEAF_RECORD_BYTES] == released[..] {
                        bytes[record..record + ALLOCATION_LEAF_RECORD_BYTES]
                            .copy_from_slice(allocated);
                    }
                }
            }
        },
    );

    // 记账：第二批那一版两块盘的 defer 待释放各减一个单元
    let unit_bytes = flow
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == quarantined.locations[0].device
                && write.offset == quarantined.locations[0].slot.to_device_offset()
        })
        .expect("流里写过被隔离的那个数据单元")
        .length_in_bytes();
    let devices_of_the_unit: Vec<u32> = quarantined
        .locations
        .iter()
        .map(|location| location.device.0)
        .collect();
    let accounting_leaf = last_unit_copies(&flow, name, &is_an_accounting_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &accounting_leaf, &|bytes| {
        let entry_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(ACCOUNTING_KEY_BYTES),
        ));
        for device in &devices_of_the_unit {
            let rows: Vec<usize> = (0..entry_count)
                .map(|index| {
                    index_node_entries_start(ACCOUNTING_KEY_BYTES) + index * ACCOUNTING_ROW_BYTES
                })
                .filter(|row| {
                    read_u16_at(bytes, *row) == STATISTIC_DEFER_QUEUE_BYTES
                        && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET) == *device
                })
                .collect();
            assert_eq!(rows.len(), 1, "记账叶里盘 {device} 的 defer 待释放恰好一行");
            let value = read_u64_at(bytes, rows[0] + ACCOUNTING_ROW_VALUE_OFFSET);
            assert!(
                value >= unit_bytes,
                "盘 {device} 的 defer 待释放 {value} 字节里算着被放掉的那个单元（{unit_bytes} 字节）"
            );
            bytes[rows[0] + ACCOUNTING_ROW_VALUE_OFFSET..rows[0] + ACCOUNTING_ROW_VALUE_OFFSET + 8]
                .copy_from_slice(&(value - unit_bytes).to_le_bytes());
        }
    });

    flow
}

/// 豁免那一形：被隔离的单元两块盘那两条记录都带隔离记号，整组豁免，第二批那一版上 I-3.11 照样成立（不豁免的话已分配多一个单元、红）。
/// 两份副本都是原样：只认记号、不读副本。CPU 一个状态都不红 I-3.11，GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_records_carry_the_mark_on_both_devices_is_exempted(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_records_carry_the_mark_on_both_devices_is_exempted";
    let flow =
        record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined(
            "crash-judge-gpu-quarantine-records-carry-the-mark",
            QuarantineMarkOfTheRecords::OnBothDevices,
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.11")
            && !comparison.cpu_checker_red_invariants.contains("I-3.11"),
        "流 {} 上 CPU 池级 checker 判过 I-3.11、一个状态都不红（被隔离的单元豁免了；成立过的 {:?}，红过的 {:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
}

/// 不豁免那一形：两条记录都不带隔离记号（与写者泄漏一个单元逐字节同形），不豁免，CPU 判 I-3.11 红，GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_records_carry_no_mark_is_not_exempted(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_records_carry_no_mark_is_not_exempted",
        &record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined(
            "crash-judge-gpu-quarantine-records-carry-no-mark",
            QuarantineMarkOfTheRecords::Absent,
        ),
        "I-3.11",
    );
}

/// 分片的段（C790）：新池新建文件那条流照甲二展开，挑状态最多的那一段只判它域里后一半（片起点 = 段的态数的一半）。判表按片起点加片内序号拆出的
/// 持久掩码与不分片的判表在同一个段内编号上拆出的逐个相同（CPU 一侧，不要卡）；GPU 探针从片起点的各位起加片内序号、逐位进位拆出的掩码
/// 与 CPU 判表逐序号相同、一个都不判不了；GPU 判器判这张表的每个状态，对拍字与 CPU 判器逐状态相同。
// crash-case-check:not-a-crash-case 只判一张判表里一段分片之后的那几十个状态（新池新建文件那条流），单跑几秒
#[test]
fn the_gpu_judge_decodes_and_judges_a_sliced_segment_from_its_slice_start_like_the_cpu() {
    const TEST_NAME: &str =
        "the_gpu_judge_decodes_and_judges_a_sliced_segment_from_its_slice_start_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow =
        record_new_pool_file_creation("crash-judge-gpu-sliced-segment", "new-pool-sliced-segment");
    let expansion =
        singlefs_verification::crash::in_place_subsets_with_copy_on_write_none_or_all_expansion;
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let whole = build_judge_tables(&judge_flow).expect("建得出表");
    let (sliced_segment, states_of_the_segment) = whole
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| (segment_index, segment.states.end - segment.states.start))
        .max_by_key(|(_segment_index, states)| *states)
        .expect("流里至少一段");
    assert!(
        states_of_the_segment >= 2,
        "挑的那一段至少两个态，切得出后一半"
    );
    let slice_start = states_of_the_segment / 2;
    let slice_states = states_of_the_segment - slice_start;
    let mut segment_slices = singlefs_verification::crash::no_segment_slices(flow.segments.len());
    segment_slices[sliced_segment] =
        Some(singlefs_verification::crash::CrashPointReplaySegmentSlice {
            start_within_the_domain: singlefs_verification::crash::StateCount1280Bits::from_u64(
                slice_start,
            ),
            state_count: slice_states,
        });
    let sliced = singlefs_verification::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices(
        &judge_flow,
        &[],
        &segment_slices,
    )
    .expect("分片的判表建得出");
    let start_of_the_slice_in_the_flow = sliced.segments[sliced_segment].states.start;
    let start_of_the_segment_in_the_whole_flow = whole.segments[sliced_segment].states.start;
    for ordinal_within_the_slice in 0..slice_states {
        assert_eq!(
            sliced.persisted_writes_of_state(
                start_of_the_slice_in_the_flow + ordinal_within_the_slice
            ),
            whole.persisted_writes_of_state(
                start_of_the_segment_in_the_whole_flow + slice_start + ordinal_within_the_slice
            ),
            "片内第 {ordinal_within_the_slice} 个是段内编号 {} 的那个态",
            slice_start + ordinal_within_the_slice
        );
    }
    let Some(packed) = packed_tables_through_the_library(TEST_NAME, &sliced) else {
        return;
    };
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, 1))
    else {
        return;
    };
    let ordinals_of_the_slice =
        start_of_the_slice_in_the_flow..start_of_the_slice_in_the_flow + slice_states;
    let probe_words = singlefs_verification::crash_judge_dispatch::persisted_masks_on_the_card(
        lease.card(),
        &packed,
        ordinals_of_the_slice.clone(),
    );
    drop(lease);
    let enumerated_writes = usize::try_from(sliced.enumerated_write_count).expect("装得进 usize");
    for (offset, words) in probe_words.iter().enumerate() {
        let ordinal = ordinals_of_the_slice.start + u64::try_from(offset).expect("装得进 u64");
        let (persisted, undecidable) =
            singlefs_verification::crash_judge_gpu::persisted_set_of_the_probe_words(
                words,
                enumerated_writes,
            );
        assert_eq!(undecidable, 0, "序号 {ordinal}：探针拆得了");
        assert_eq!(
            persisted,
            sliced.persisted_writes_of_state(ordinal),
            "序号 {ordinal}：GPU 从片起点的各位起拆出的持久掩码与 CPU 相同"
        );
    }
    let Some(gpu) =
        singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool(
            cards, TEST_NAME, &sliced,
        )
    else {
        return;
    };
    // CPU 参照一侧：不分片的那条流先入崩溃点重放枚举那一类的库、再从库里读回，判它的每个状态（CPU 参照也从库读）。
    // 这张表的序号换到不分片的编号上：切的那一段之前的段两边一样；那一段里片内第 k 个是段内编号 片起点 + k 的那个态（上面逐个核过持久掩码）；
    // 之后的段整体往后挪片起点那么多个
    assert_eq!(
        start_of_the_slice_in_the_flow, start_of_the_segment_in_the_whole_flow,
        "切的那一段之前的段两边一样，那一段在两张表里起在同一个序号"
    );
    let ordinal_in_the_whole_flow = |ordinal: u64| {
        if ordinal < start_of_the_slice_in_the_flow {
            ordinal
        } else {
            ordinal + slice_start
        }
    };
    let Some(cpu_by_ordinal) = cpu_cross_check_words_read_back_from_the_library(
        &format!("{TEST_NAME}-cpu-reference"),
        &flow,
        &expansion,
        0..whole.state_count,
    ) else {
        return;
    };
    let mismatches: Vec<String> = gpu
        .iter()
        .enumerate()
        .filter_map(|(ordinal, gpu_words)| {
            let ordinal = u64::try_from(ordinal).expect("装得进 u64");
            let whole_ordinal = ordinal_in_the_whole_flow(ordinal);
            let (cpu_persisted, cpu_cross_check_words) = &cpu_by_ordinal[&whole_ordinal];
            assert_eq!(
                *cpu_persisted,
                sliced.persisted_writes_of_state(ordinal),
                "序号 {ordinal}：CPU 从库里读回判的不分片序号 {whole_ordinal} 与这张表的那一个是同一个状态"
            );
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (!cross_check_words_agree_in_the_cross_check(
                &gpu_cross_check_words,
                cpu_cross_check_words,
            ))
            .then(|| {
                format!(
                    "序号 {ordinal}：CPU {cpu_cross_check_words:?}；GPU {gpu_cross_check_words:?}"
                )
            })
        })
        .collect();
    assert_eq!(
        u64::try_from(gpu.len()).expect("装得进 u64"),
        sliced.state_count,
        "GPU 判了这张表的每个状态"
    );
    assert!(
        mismatches.is_empty(),
        "分片之后的 {} 个状态里 {} 个 GPU 与 CPU 的对拍字不同：\n{}",
        sliced.state_count,
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 里程碑四快照、搬迁与 discard 那几条流发布用的实例代号：都在建池那个进程里接着发。
pub(super) const INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS: InstanceGeneration =
    InstanceGeneration(1);
