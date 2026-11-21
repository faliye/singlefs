//! 根环（I-7 类）与树 ID 诞生：I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 只取一圈窗口里的根。

use super::*;

// ───────────── I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取两个一圈窗口里的根 ─────────────

/// 根记录里 flags（卸载记号在位 0）、checkpoint_txg、回退下界 F 的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
const ROOT_RECORD_FLAGS_OFFSET: usize = 20;

pub(super) const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;

const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: usize = 130;

/// 池级 checker 违例位里这一条的那一位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
pub(super) fn invariant_bit(invariant: &str) -> u128 {
    let index = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|candidate| *candidate == invariant)
        .expect("这一条在池级 checker 的清单里");
    1u128 << index
}

/// 流尾另起一段写的那条根带不带卸载记号（根记录 flags 位 0）。
#[derive(Clone, Copy, Debug)]
enum UnmountMarkerOfTheAppendedRoot {
    Written,
    NotWritten,
}

/// 流尾另起一段写一条根：抄第 `source_root_write` 次写（根槽写）的字节，checkpoint_txg 改成 `checkpoint_txg`、回退下界 F 改成
/// `rollback_floor`、flags 位 0 照 `unmount_marker`，实例代号与各条指针照抄，重封整槽校验和，写到这个 txg 按发布规则算出的那个根槽上
/// （`singlefs_core::root_ring::target_for_publish`，I-7.17 不因为它红）。那个槽里原来的根被它盖掉。
fn append_a_root_at_a_later_txg(
    flow: &mut RecordedFlow,
    source_root_write: usize,
    checkpoint_txg: u64,
    rollback_floor: u64,
    unmount_marker: UnmountMarkerOfTheAppendedRoot,
) {
    let source = flow.writes[source_root_write].clone();
    assert_eq!(source.kind, StepKind::RootRecordFua, "抄的是根槽写");
    let WrittenContents::Bytes(mut bytes) = source.contents else {
        panic!("根槽写是普通写");
    };
    bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
        .copy_from_slice(&checkpoint_txg.to_le_bytes());
    bytes[ROOT_RECORD_ROLLBACK_FLOOR_OFFSET..ROOT_RECORD_ROLLBACK_FLOOR_OFFSET + 8]
        .copy_from_slice(&rollback_floor.to_le_bytes());
    let flags: u32 = match unmount_marker {
        UnmountMarkerOfTheAppendedRoot::Written => 1,
        UnmountMarkerOfTheAppendedRoot::NotWritten => 0,
    };
    bytes[ROOT_RECORD_FLAGS_OFFSET..ROOT_RECORD_FLAGS_OFFSET + 4]
        .copy_from_slice(&flags.to_le_bytes());
    reseal_root_record(&mut bytes);
    let target = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(checkpoint_txg),
        geometry().root_ring_slots_per_region,
    );
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset =
        singlefs_core::root_ring::slot_offset(target, geometry().fixed_structure_slot_spacing);
    append_a_segment_of_writes(
        flow,
        vec![RetainedWrite {
            device,
            kind: StepKind::RootRecordFua,
            is_force_unit_access: source.is_force_unit_access,
            offset,
            contents: WrittenContents::Bytes(bytes),
        }],
    );
}

/// 根环槽数 3S（三个区域、每区 S 槽）。
fn root_ring_slot_count() -> u64 {
    singlefs_format::ROOT_RING_REGIONS * geometry().root_ring_slots_per_region.count()
}

/// 树表单元：码 2、key 宽 8、条目宽 200。
pub(super) fn is_a_tree_table_unit(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && index_node_key_width(bytes) == 8
        && index_node_entry_width(bytes) == 200
}

/// 树表单元里 inode 树（种类 2）那几条条目的诞生 txg（条目偏移 108）加 1。
fn add_one_to_the_birth_txg_of_the_inode_tree_entries(bytes: &mut [u8]) {
    for entry in 0..(bytes.len() - 131) / 200 {
        let offset = 131 + entry * 200;
        if u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 2 {
            bytes[offset + 108] = bytes[offset + 108].wrapping_add(1);
        }
    }
}

/// 回退流上回退那次发布 D（txg 9）的树表里 inode 树条目的诞生 txg 加 1、重封到根（同 `rollback-tree-table-birth-txg-plus-one`），
/// 流尾再在 txg 30 写一条 D 的根、回退下界 F 抬到 9（落在 txg 6 那个槽上）。这一条落了的状态上候选集只剩 D 与它（别的根都在 F 之下），
/// 两条共用 D 那片树表，I-9.14 比不出来；I-9.28 照比根环里每一条读得出的根，A、B、C 那几片记的诞生 txg 与 D 的不同，红。
fn record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK: u64 = 30;
    const FLOOR_AT_THE_ROLLBACK: u64 = 9;
    let mut flow = record_rollback_with_a_resealed_field(
        tag,
        "rollback-birth-txg-differs-only-outside-the-candidate-set",
        &is_a_tree_table_unit,
        &add_one_to_the_birth_txg_of_the_inode_tree_entries,
    );
    assert_eq!(
        publish_root_identity(&flow),
        (2, FLOOR_AT_THE_ROLLBACK),
        "被判的根是回退那次发布 D（实例 2、txg 9）"
    );
    let rollback_root = flow.judged_root_index;
    append_a_root_at_a_later_txg(
        &mut flow,
        rollback_root,
        TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK,
        FLOOR_AT_THE_ROLLBACK,
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK),
        content: file_content(),
    });
    flow
}

/// I-9.28 比的是根环里每一条读得出、自证过的根，不只是回退候选集（I-9.14 只比候选集）：有一个状态上 CPU 红 I-9.28 而 I-9.14 不红，
/// GPU 每个状态逐位相同。GPU 那一条只比候选根时，这个状态上它不红、与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg(
) {
    let flow = record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
        "crash-judge-gpu-i928-outside-candidates",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::CountStatesWhere(
            StateCondition::CpuCheckerRedOnOneAndNotTheOther {
                red: "I-9.28",
                not_red: "I-9.14",
            },
        )],
    ) else {
        return;
    };
    assert!(
        comparison.examined[0].states_counted > 0,
        "流 {} 上要有一个状态 CPU 红 I-9.28 而 I-9.14 不红（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 回退流上覆盖写 C（txg 8）那条根指着的树表两份副本各改坏末字节：C 那条根落了的状态上它读得出、自证过，树表指针非零而树表读不出，
/// I-9.28 少它那一份证据（CPU 报「部分没判」，GPU 与不适用一样不置违例位）；读得出的那几片记的诞生 txg 都相同，不红。
fn record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_OVERWRITE_BEFORE_THE_ROLLBACK: u64 = 8;
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-tree-table-of-c-unreadable";
    let root_of_the_overwrite = flow
        .writes
        .iter()
        .rposition(|write| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u64::from_le_bytes(
                        bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
                            ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                            .try_into()
                            .expect("8"),
                    ) == TXG_OF_THE_OVERWRITE_BEFORE_THE_ROLLBACK
                })
        })
        .expect("回退流上有覆盖写 C（txg 8）的根槽写");
    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, root_of_the_overwrite);
    flow
}

/// I-9.28 的四种判定在 GPU 上只有违例置位：有一条读得出、自证过的根树表读不出，而读得出的那几片都对得上时，CPU 一个状态都不红它，
/// GPU 每个状态逐位相同。GPU 把读不出树表的根当成违例时这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant(
) {
    let flow = record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
        "crash-judge-gpu-i928-unreadable-tree-table",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-9.28"),
        "流 {} 上 CPU 一个状态都不红 I-9.28（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 覆盖写那一版的文件长度：一个数据单元装得下（32768 − 134），内容跨过单元的第二槽起点（单元偏移 16384）。
const FORGED_HEADER_FILE_BYTES: usize = 20_000;

/// 伪头里的树 ID：2⁴⁰，远高于这条流的水位。
const FORGED_HEADER_TREE_IDENTIFIER: u64 = 1 << 40;

/// 伪头的 key 宽（码 2 头偏移 51）。
const FORGED_HEADER_KEY_WIDTH: u8 = 8;

/// 伪头里的诞生代号：1，不超过根环里最大的 checkpoint_txg（扫描方向第一道排除挡不住它）。
const FORGED_HEADER_BIRTH_TXG: u64 = 1;

/// 伪头带的「别的池」的 fsid 低 8 字节：写序实例按 fsid 认不出，扫描方向第二道排除也挡不住它。
const FORGED_HEADER_FOREIGN_FILESYSTEM_IDENTIFIER_LOW: u64 = 0x0123_4567_89ab_cdef;

/// 覆盖写的内容：按位置取的字节，内容偏移 16384 − 134 起放一个头校验和自洽的码 2 伪头（D18（块里携带什么信息） 已定项 18 的偏移：
/// 树 ID 42、key 宽 51、诞生代号 52 + 2k、fsid 60 + 2k、明文头末尾 86 + 2k，头校验和在 10）。
fn content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit() -> Vec<u8> {
    let mut content: Vec<u8> = (0..FORGED_HEADER_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 5) % 251).expect("小于 251"))
        .collect();
    let key_span = 2 * usize::from(FORGED_HEADER_KEY_WIDTH);
    let header_end = 86 + key_span;
    let mut header = vec![0u8; header_end];
    header[..4].copy_from_slice(&singlefs_core::unit::UNIT_MAGIC);
    header[6] = singlefs_core::unit::UNIT_CLASS_INDEX_NODE;
    header[42..50].copy_from_slice(&FORGED_HEADER_TREE_IDENTIFIER.to_le_bytes());
    header[51] = FORGED_HEADER_KEY_WIDTH;
    header[52 + key_span..60 + key_span].copy_from_slice(&FORGED_HEADER_BIRTH_TXG.to_le_bytes());
    header[60 + key_span..68 + key_span]
        .copy_from_slice(&FORGED_HEADER_FOREIGN_FILESYSTEM_IDENTIFIER_LOW.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&header, header_end, 10);
    header[10..42].copy_from_slice(&digest);
    let forged_header_content_offset =
        usize::try_from(singlefs_format::SLOT_BYTES - singlefs_format::DATA_UNIT_PAYLOAD_OFFSET)
            .expect("小于一个槽");
    content[forged_header_content_offset..forged_header_content_offset + header_end]
        .copy_from_slice(&header);
    content
}

/// 新池新建文件之后覆盖写一次，内容在它那个数据单元第二槽的起点上带一个码 2 伪头（[`content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit`]）。
fn record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
    tag: &str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let content = content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit();
    pool.output = overwrite_step(&mut recorder, &mut pool, &content, InstanceGeneration(1));
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let second_slot = usize::try_from(singlefs_format::SLOT_BYTES).expect("16384");
    assert!(
        writes.iter().any(|write| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| {
                    unit_class(bytes) == 1
                        && bytes.len() > second_slot + 52
                        && bytes[second_slot..second_slot + 4] == singlefs_core::unit::UNIT_MAGIC
                        && bytes[second_slot + 6] == singlefs_core::unit::UNIT_CLASS_INDEX_NODE
                })
        }),
        "覆盖写那一版的文件装在一个数据单元里，伪头落在它第二槽的起点上"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        name: "new-pool-overwrite-forging-an-index-node-header-on-the-second-slot",
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content,
            },
        ],
    }
}

/// I-7.8 扫描方向的认领规则（D18（块里携带什么信息） 已定项 7，walk.rs 的 `scanned_tree_identifiers`）：覆盖写的数据单元第二槽起点上
/// 带一个头校验和自洽、树 ID 2⁴⁰ 的码 2 伪头。GPU 的落点表里有这个第二槽（覆盖写那次写碰过的每个 16 KiB 槽各是一个落点），
/// CPU 的崩溃镜像只把那次写的起点槽当候选；数据单元落了的状态上，GPU 照认领规则让起点槽上的数据单元认领两槽、不读第二槽的头，
/// 与 CPU 一样不数它，CPU 一个状态都不红 I-7.8、GPU 每个状态逐位相同。GPU 不按整单元跳时把伪头数进「出现过的」树 ID、红 I-7.8。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted(
) {
    let flow = record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
        "crash-judge-gpu-i78-claim",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.8"),
        "流 {} 上 CPU 一个状态都不红 I-7.8（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 新池新建文件那条流上，流尾在 txg 27 = 3 + 3S 写一条抄发布那条根 (1, 3)、回退下界 F 从 0 抬到 5 的根（落在 txg 3 那个槽上、把它盖掉）。
/// 之后根环里是 txg 0、1、2 的根与这一条，它是镜像最新的根：锚在它上的一圈窗口（txg > 3）里 txg 比它小的根一条都没有，
/// 两支都不判它（准入那一支上限无从算起；带卸载记号那一支没有「之前的根」）。不取窗口时 txg 1、2 的根进来，F 5 高过它们，红。
fn record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
    tag: &str,
    name: &'static str,
    unmount_marker: UnmountMarkerOfTheAppendedRoot,
) -> RecordedFlow {
    const FLOOR_RAISED_ONE_REVOLUTION_PAST_THE_PUBLISH: u64 = 5;
    let mut flow = record_new_pool_file_creation(tag, name);
    let (instance, publish_txg) = publish_root_identity(&flow);
    assert_eq!(
        (instance, publish_txg),
        (1, 3),
        "被判的根是新建文件那次发布"
    );
    let txg_one_revolution_past_the_publish = publish_txg + root_ring_slot_count();
    let publish_root = flow.judged_root_index;
    append_a_root_at_a_later_txg(
        &mut flow,
        publish_root,
        txg_one_revolution_past_the_publish,
        FLOOR_RAISED_ONE_REVOLUTION_PAST_THE_PUBLISH,
        unmount_marker,
    );
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(txg_one_revolution_past_the_publish),
        content: file_content(),
    });
    flow
}

/// I-7.9 准入抬 F 那一支只取两个一圈窗口里的根：窗口里一条之前的根都没有，这条根不判，
/// CPU 一个状态都不判 I-7.9（不红、也不报成立），GPU 每个状态逐位相同。GPU 那一支不取窗口时红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged(
) {
    let flow = record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
        "crash-judge-gpu-i79-admission-window",
        "new-pool-floor-raise-one-revolution-past-every-earlier-root",
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.9")
            && !comparison.cpu_checker_held_invariants.contains("I-7.9"),
        "流 {} 上 CPU 一个状态都不判 I-7.9（红过的：{:?}；成立过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants,
        comparison.cpu_checker_held_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// I-7.9 带卸载记号那一支同样只取两个一圈窗口里的根：同一张根环、那条根带卸载记号，窗口里 txg 比它小的根一条都没有，CPU 一个状态都不红 I-7.9，
/// GPU 每个状态逐位相同。GPU 那一支不取窗口时拿 txg 2 那条根比、F 5 高过它，红。CPU 那一支的窗口由同一批的兄弟实现员补，
/// 合入之前这条在 CPU 一侧照旧红（不取窗口）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened(
) {
    let flow = record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
        "crash-judge-gpu-i79-unmount-window",
        "new-pool-unmount-marked-floor-raise-one-revolution-past-every-earlier-root",
        UnmountMarkerOfTheAppendedRoot::Written,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.9"),
        "流 {} 上 CPU 一个状态都不红 I-7.9（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 回退流（根环里 txg 0..9，S = 8、3S = 24）上流尾先写一段：txg 29 一条抄回退那次发布 D 的根、F 从 0 抬到 6（落在 txg 5 那个槽上）；
/// 再写一段：txg 27 一条同样的根、F 6（落在 txg 3 那个槽上），这时抬 F 的是 txg 27 那一条（txg 29 那一条的前一条变成它、F 不再高）。
/// 两段都落了的状态上，锚在被判那条根（27）上的窗口是 txg > 3、锚在镜像最新那条根（29）上的是 txg > 5：B 那条根 (1, 4) 只在前一个里。
/// 只取锚在镜像最新根上的窗口时之前的有效根是 6、7、8、9，上限 6，F 6 在窗口里成立；去掉最旧那条之后非空有效根去重只剩 8、9 两条，
/// 窗口定不住上限，照 C717（回退下界上限的单窗口盲带）丁按 journal 重建抬 F 那一刻的根环（txg 3、5 那两槽认回 txg 3、5）：
/// 非空有效根 3、4、8、9 去重之后不足 4 条（txg 3 与 9 是同一个状态：D 回退到的就是 (1, 3)），上限取最旧有效根 0，F 6 红。
/// 只有前一段落了的状态上抬 F 的是 txg 29 那一条，
/// 窗口同样定不住、重建的环同样只计入三条，同样红（批 19 验证段 CPU 探针量的：两种状态都红 I-7.9，另红 I-7.12、I-3.13）。
fn record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_NEWEST_ROOT: u64 = 29;
    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR: u64 = 27;
    const FLOOR_RAISED_PAST_THE_ROLLBACK: u64 = 6;
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-floor-raise-whose-image-window-drops-an-older-root";
    assert_eq!(
        publish_root_identity(&flow),
        (2, 9),
        "被判的根是回退那次发布 D"
    );
    assert_eq!(
        TXG_OF_THE_NEWEST_ROOT - root_ring_slot_count(),
        5,
        "锚在最新那条根上的窗口是 txg > 5"
    );
    let rollback_root = flow.judged_root_index;
    for checkpoint_txg in [TXG_OF_THE_NEWEST_ROOT, TXG_OF_THE_ROOT_RAISING_THE_FLOOR] {
        append_a_root_at_a_later_txg(
            &mut flow,
            rollback_root,
            checkpoint_txg,
            FLOOR_RAISED_PAST_THE_ROLLBACK,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        );
        flow.versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            content: file_content(),
        });
    }
    flow
}

/// I-7.9 准入那一支窗口定不住上限时照丁按 journal 重建抬 F 那一刻的根环判（C717（回退下界上限的单窗口盲带））：
/// 这条流上锚在镜像最新根上的窗口里 F 6 成立，而重建的环上限 0，CPU 判红 I-7.9，GPU 每个状态逐位相同。
/// 丁之前这条流守的是「之前的根只取锚在镜像最新根上的窗口」（CPU 成立、GPU 取锚在被判那条根上的窗口时红）；丁之后两种窗口在这条流上都红，
/// 那一判改由 [`the_gpu_judge_holds_like_the_cpu_a_floor_raise_bounded_by_the_journal_while_the_window_anchored_at_the_raising_root_reddens_it`] 守。
#[test]
fn the_gpu_judge_reddens_like_the_cpu_a_floor_raise_whose_window_cannot_bound_the_ceiling_against_the_ring_rebuilt_from_the_journal(
) {
    let flow = record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
        "crash-judge-gpu-i79-image-window",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_reddens_like_the_cpu_a_floor_raise_whose_window_cannot_bound_the_ceiling_against_the_ring_rebuilt_from_the_journal",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-7.9")
            && !comparison.cpu_checker_held_invariants.contains("I-7.9"),
        "流 {} 上 CPU 按 journal 重建的环判红 I-7.9、一个状态都不判它成立（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 同一个实例里回退流那几次覆盖写的内容：按 `fill_seed` 各不相同，长度都在一个数据单元里。
fn content_of_an_overwrite_before_the_rollback_inside_one_instance(fill_seed: usize) -> Vec<u8> {
    (0..2000 + fill_seed * 100)
        .map(|index| u8::try_from((index * 7 + fill_seed * 13 + 1) % 251).expect("小于 256"))
        .collect()
}

/// 新池新建文件 (1, 3) 之后可写挂载（实例 2：txg 4、5），实例 2 里覆盖写五次（txg 6–10，内容各不相同、各是一个非空状态），
/// 再在同一个实例里回退到 txg 7 那一版（回退 D 是 (2, 11)，树取 txg 7 的）。流尾先写一段：txg 32 一条抄 D 的根、F 从 0 抬到 8
/// （落在 txg 8 那个槽上）；再写一段：txg 30 一条同样的根、F 8（落在 txg 6 那个槽上），这时抬 F 的是 txg 30 那一条。
/// 两段都落了的状态上（3S = 24）锚在镜像最新根（32）上的窗口是 txg > 8，之前的有效根 9、10、11 是三个不同的状态，上限取最旧有效根 9，
/// F 8 在窗口里成立；去掉最旧那条只剩两条，窗口定不住，丁按 journal 重建抬 F 那一刻的环（txg 6、8 那两槽认回 txg 6、8）：
/// 从新到旧去重计入 11、10、9、8，上限 8，F 8 成立。锚在被判那条根（30）上的窗口是 txg > 6，多出 txg 7 那条根：
/// 它与 D 是同一个状态、不计入，计入的仍是三条，上限取最旧有效根 7，F 8 红（不进丁：窗口里就红了）。
/// 只有前一段落了的状态上抬 F 的是 txg 32 那一条、它就是最新的根，两个窗口相同（txg > 8），丁重建的环相同，成立。
/// 窗口定得住上限（去掉最旧那条仍计入 4 条）的流上，窗口外的旧根改不了上限的下沿（`walk.rs` 的
/// `window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise`），两个窗口判法相同、分不出来，这一判只能放在丁成立的流上。
fn record_rollback_inside_one_instance_with_a_floor_raise_bounded_by_the_journal(
    tag: &str,
) -> RecordedFlow {
    const OVERWRITES_INSIDE_THE_SECOND_INSTANCE: usize = 5;
    const TXG_OF_THE_ROLLBACK_TARGET: u64 = 7;
    const TXG_OF_THE_NEWEST_ROOT: u64 = 32;
    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR: u64 = 30;
    const FLOOR_RAISED_TO_THE_CEILING_OF_THE_RING_AT_THE_RAISE: u64 = 8;
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("新建文件之后重开，现行那一版带文件");
    for txg in 4..=5 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: file_content(),
        });
    }
    let mut content_of_the_rollback_target = Vec::new();
    for (checkpoint_txg, fill_seed) in (6..).zip(7..7 + OVERWRITES_INSIDE_THE_SECOND_INSTANCE) {
        let content = content_of_an_overwrite_before_the_rollback_inside_one_instance(fill_seed);
        let overwritten = overwrite_step(&mut recorder, &mut pool, &content, InstanceGeneration(2));
        pool.output = overwritten;
        if checkpoint_txg == TXG_OF_THE_ROLLBACK_TARGET {
            content_of_the_rollback_target = content.clone();
        }
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            content,
        });
    }
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_ROLLBACK_TARGET),
        },
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(11),
        content: content_of_the_rollback_target.clone(),
    });
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let root_identities: Vec<(u32, u64)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| {
            let bytes = write.bytes().expect("根槽写带字节");
            (
                u32::from_le_bytes(bytes[24..28].try_into().expect("4")),
                u64::from_le_bytes(bytes[28..36].try_into().expect("8")),
            )
        })
        .collect();
    assert_eq!(
        root_identities,
        [(1, 1), (1, 2), (1, 3)]
            .into_iter()
            .chain((4..=11).map(|txg| (2, txg)))
            .collect::<Vec<_>>(),
        "取号、暖机、新建文件 (1, 3)，重开写下 txg 4、5，五次覆盖写 txg 6–10，回退 D 是 (2, 11)"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    let mut flow = RecordedFlow {
        name: "rollback-inside-one-instance-floor-raise-bounded-by-the-journal",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    };
    assert_eq!(
        TXG_OF_THE_NEWEST_ROOT - root_ring_slot_count(),
        FLOOR_RAISED_TO_THE_CEILING_OF_THE_RING_AT_THE_RAISE,
        "最新那条根落在 txg 8 那个槽上，锚在它上的窗口是 txg > 8"
    );
    let rollback_root = flow.judged_root_index;
    for checkpoint_txg in [TXG_OF_THE_NEWEST_ROOT, TXG_OF_THE_ROOT_RAISING_THE_FLOOR] {
        append_a_root_at_a_later_txg(
            &mut flow,
            rollback_root,
            checkpoint_txg,
            FLOOR_RAISED_TO_THE_CEILING_OF_THE_RING_AT_THE_RAISE,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        );
        flow.versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            content: content_of_the_rollback_target.clone(),
        });
    }
    flow
}

/// I-7.9 准入那一支的「之前的根」只取锚在镜像最新根上的一圈窗口里的（不另取锚在被判那条根上的那一个）：这条流上窗口定不住上限、
/// 丁按 journal 重建的环判成立，CPU 判过 I-7.9 而成立、一个状态都不红它，GPU 每个状态逐位相同。
/// GPU 取锚在被判那条根上的窗口时把 txg 7 那条根算进来，窗口里上限 7、F 8 红（批 19 验证段 CPU 探针把池级 checker 的窗口照这样换掉量的：
/// 两段都落了的那个状态红 I-7.9，只落前一段的成立；F 抬到 9 时两种窗口都红、丁重建的环上限 8）。
#[test]
fn the_gpu_judge_holds_like_the_cpu_a_floor_raise_bounded_by_the_journal_while_the_window_anchored_at_the_raising_root_reddens_it(
) {
    let flow = record_rollback_inside_one_instance_with_a_floor_raise_bounded_by_the_journal(
        "crash-judge-gpu-i79-journal-bound",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_holds_like_the_cpu_a_floor_raise_bounded_by_the_journal_while_the_window_anchored_at_the_raising_root_reddens_it",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-7.9")
            && !comparison.cpu_checker_red_invariants.contains("I-7.9"),
        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
