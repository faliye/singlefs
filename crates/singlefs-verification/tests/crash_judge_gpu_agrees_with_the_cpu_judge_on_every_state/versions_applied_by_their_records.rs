//! 由记录造出来的两类版本取带末条标志那条记录的新根段，树表指针整条全零照 core 跟过去。

use super::*;

// ─── 由记录造出来的两类版本取带末条标志那条记录的新根段（FL）、树表指针全零照 core 跟过去（Q1、Q3） ───

/// 新根段七段在记录与根记录里的偏移（新根段从记录偏移 99 起；D23（journal 的角色与格式） 已定项 15 / 已定项 4 的字段表，与 walk.rs 的
/// `NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT` 同一张表）：(新根段里的起点, 根记录里的起点, 字节数)。
const NEW_ROOT_SEGMENT_FIELDS_IN_THE_ROOT_RECORD: [(usize, usize, usize); 7] = [
    (0, 36, 86),
    (86, 256, 86),
    (172, 122, 8),
    (180, 130, 8),
    (188, 170, 86),
    (274, 342, 86),
    (360, 20, 4),
];

/// journal 记录头里实例代号（4 字节）、计数器（6 字节）、`checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 17 的字段表）。
pub(super) const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;

pub(super) const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;

pub(super) const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;

/// 码 2 节点头里诞生代号那 8 字节的偏移扣掉 2k（D18（块里携带什么信息） 已定项 7：诞生代号 52 + 2k）。
const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;

/// 流里 (实例 1, `checkpoint_txg`) 那次发布的每一份 journal 记录写：(写下标, 计数器)。
pub(super) fn journal_record_writes_of_the_publish(
    flow: &RecordedFlow,
    checkpoint_txg: u64,
) -> Vec<(usize, u64)> {
    flow.writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord)
        .filter_map(|(index, write)| {
            let bytes = write.bytes()?;
            let instance = u32::from_le_bytes(
                bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
                    .try_into()
                    .expect("4"),
            );
            let txg = u64::from_le_bytes(
                bytes[JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET
                    ..JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                    .try_into()
                    .expect("8"),
            );
            let mut counter = [0u8; 8];
            counter[..6].copy_from_slice(
                &bytes[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6],
            );
            (instance == 1 && txg == checkpoint_txg).then_some((index, u64::from_le_bytes(counter)))
        })
        .collect()
}

/// 三个数据单元那条流里第一个文件顺序写那次发布（实例 1、txg 4，切成三条记录）的**第一条**记录（两块盘各一份）新根段七段换成
/// txg 3 那条根的（写侧违反 D23（journal 的角色与格式） 已定项 15「每条记录都带同一份新根段」的样子），重封、同实例后面的记录重链。
fn write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish(
    flow: &mut RecordedFlow,
) {
    let root_of_the_first_file: Vec<u8> = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .filter_map(RetainedWrite::bytes)
        .find(|bytes| {
            u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 3
        })
        .expect("流里有第一个文件那条根 (1, 3) 的根槽写")
        .to_vec();
    let records = journal_record_writes_of_the_publish(flow, 4);
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    assert_eq!(
        records.len(),
        6,
        "流 {}：顺序写那次发布切成三条记录、两块盘各一份",
        flow.name
    );
    for (index, _) in records
        .iter()
        .filter(|(_, counter)| *counter == first_counter)
    {
        rewrite_journal_record_and_reseal(flow, *index, &|bytes| {
            for (in_the_segment, in_the_root, length) in NEW_ROOT_SEGMENT_FIELDS_IN_THE_ROOT_RECORD
            {
                let segment = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + in_the_segment;
                bytes[segment..segment + length]
                    .copy_from_slice(&root_of_the_first_file[in_the_root..in_the_root + length]);
            }
        });
    }
}

/// 一片分配记录叶由 txg 4 那次发布写出（三个数据单元那条流上是顺序写那次，两次单条覆盖写那条流上是第一次覆盖写）、
/// 且里面有一对改得动的记录（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]）。
pub(super) fn is_an_allocation_leaf_born_at_txg_four_that_can_be_widened(bytes: &[u8]) -> bool {
    if unit_class(bytes) != 2 || index_node_key_width(bytes) != 10 || bytes[50] != 0 {
        return false;
    }
    let birth_txg_offset = INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS + 2 * 10;
    u64::from_le_bytes(
        bytes[birth_txg_offset..birth_txg_offset + 8]
            .try_into()
            .expect("8"),
    ) == 4
        && widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
            &mut bytes.to_vec(),
        ) == 1
}

/// 改法 FL 下一次挂载那一支（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告的 s3 形）：
/// 三个数据单元那条流上，顺序写那次发布（txg 4，三条记录）写的分配记录叶里两条记录罩住同一个槽（重封到根与三条记录），
/// 再把第一条记录的新根段写成 txg 3 那条根的。三条记录都落了、根槽没落的状态上，CPU 认下一次挂载先施加的那一版取重放第一次施加的发布的末条，
/// 红 I-5.4；GPU 扫描内核 `record_row_of_the_version_the_next_mount_applies_first` 读走读内核同一份重放交来的那一条、红在同样的状态上
/// （取第一条就走 txg 3 的树、不红）。
fn record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-stale-first-record-over-overlapping-allocation-records";
    let mut flow = record_three_unit_file_with_a_resealed_field(
        tag,
        name,
        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
        &|bytes| {
            assert_eq!(
                widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
                1,
                "挑叶时这一片改得动"
            );
        },
    );
    write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish(&mut flow);
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot",
        &record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
            "crash-judge-gpu-fl-next-mount-version",
        ),
        "I-5.4",
    );
}

/// 三个数据单元的第一个文件那段历史（`pool_with_a_three_unit_file`：新池新建文件 txg 3、顺序写三个数据单元那次发布 txg 4、三条记录）
/// 录到 txg 4 的根槽写之前停——崩在那次根槽 FUA 之前，三条记录两块盘都落了。被判的根是 (1, 3) 那一次根槽写；版本是 txg 3、txg 4 两版
/// （txg 4 那一版只在三条记录里，记录全落的状态上恢复由记录施加它）。
pub(super) fn record_three_unit_file_up_to_the_root_of_its_three_record_publish(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let pool = pool_with_a_three_unit_file(tag);
    let operations = pool.retained_operations();
    let classifier = geometry();
    let operations_after_mkfs = &operations[pool.mkfs_operation_count..];
    let root_of_the_three_unit_publish = operations_after_mkfs
        .iter()
        .position(|retained| {
            classifier.classify(&retained.operation) == StepKind::RootRecordFua
                && retained.contents.as_ref().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 4
                })
        })
        .expect("流里有顺序写那次发布 (1, 4) 的根槽写");
    let (writes, segments, _stream_indexes_before_the_crash) =
        writes_and_segments_with_stream_indexes(
            &operations_after_mkfs[..root_of_the_three_unit_publish],
            &classifier,
        );
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        judged_root_index: writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("顺序写之前有新建文件那次发布 (1, 3) 的根槽写"),
        writes,
        segments,
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: content_of_three_data_units(),
            },
        ],
    }
}

/// [`record_three_unit_file_up_to_the_root_of_its_three_record_publish`] 那一段上，`rewrite_the_records_before_the_mount` 改那次发布的记录。
/// 再从这一段写全落的那张崩溃镜像起新进程（另一条录制流）录可写挂载（施加那三条记录、取号 2、写行、暖机两次）与实例 2 的一次覆盖写，
/// 两段写接成一条流。txg 4 那一版只在它的三条记录里：挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。挂载读到的就是这条流里那三条记录（
/// P3、R2：录完整段历史再从流里拿掉 txg 4 的根槽写，录制时挂载读到的是 txg 4 的根、什么都没施加，
/// 写出的实例表行（W）与之后的发布、根槽之后的系统配置轮换都与合法历史不同）。
pub(super) fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
    tag: &str,
    name: &'static str,
    rewrite_the_records_before_the_mount: &dyn Fn(&mut RecordedFlow),
) -> RecordedFlow {
    let mut flow = record_three_unit_file_up_to_the_root_of_its_three_record_publish(tag, name);
    rewrite_the_records_before_the_mount(&mut flow);

    let classifier = geometry();
    let stream_of_the_new_process = SharedStream::retaining_contents();
    let every_write_landed = vec![true; flow.writes.len()];
    let mut devices = common::crash_state_devices(
        &flow.base,
        &flow.writes,
        &every_write_landed,
        &stream_of_the_new_process,
    );
    let mut mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices)
        .expect("崩在 txg 4 根槽之前的镜像上新进程可写挂载：施加那三条记录、取号 2");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(4), 3),
        "流 {name}：择 (1, 3) 的根，施加 txg 4 的三条记录，现行版是 (1, 4)"
    );
    let current = mounted
        .current
        .into_file_version()
        .expect("施加了 txg 4，现行那一版带文件");
    let overwrite = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &third_content(),
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写")
    };
    let (
        writes_of_the_new_process,
        segments_of_the_new_process,
        _stream_indexes_of_the_new_process,
    ) = writes_and_segments_with_stream_indexes(
        &stream_of_the_new_process.retained_operations(),
        &classifier,
    );
    let first_write_of_the_new_process = flow.writes.len();
    flow.writes.extend(writes_of_the_new_process);
    flow.segments
        .extend(segments_of_the_new_process.into_iter().map(|segment| {
            segment
                .into_iter()
                .map(|index| first_write_of_the_new_process + index)
                .collect()
        }));
    flow.judged_root_index = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("覆盖写那次发布有根槽写");
    let overwrite_txg = overwrite.root.checkpoint_txg.0;
    flow.versions = vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: content_of_three_data_units(),
        },
    ];
    for txg in 5..overwrite_txg {
        flow.versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: content_of_three_data_units(),
        });
    }
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(overwrite_txg),
        content: third_content(),
    });
    flow
}

/// 在流尾另起一段，把 `checkpoint_txg` 那次发布（实例 1）的每一份 journal 记录写（两块盘各一份）抄一遍、`rewrite` 改过、重封，
/// 同盘里计数器接着的那一条的反向链按前一条改后的头重算：这一段全落的状态上盘面就是「整段历史写完之后那几条记录被改成这样」。
/// 流里原来那几次记录写不动，挂载录制时读到的仍是它们。
fn append_the_records_of_the_publish_rewritten(
    flow: &mut RecordedFlow,
    checkpoint_txg: u64,
    rewrite: &dyn Fn(&mut [u8]),
) {
    let mut records = journal_record_writes_of_the_publish(flow, checkpoint_txg);
    records.sort_by_key(|(index, counter)| (flow.writes[*index].device, *counter));
    let mut rewritten: Vec<RetainedWrite> = Vec::new();
    let mut previous_on_the_device: Option<(DeviceIdentity, u64, u32)> = None;
    for (index, counter) in records {
        let mut write = flow.writes[index].clone();
        let device = write.device;
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("记录写是普通写");
        };
        rewrite(bytes);
        if let Some((_, _, chain)) =
            previous_on_the_device.filter(|(previous_device, previous_counter, _)| {
                *previous_device == device && previous_counter + 1 == counter
            })
        {
            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
        }
        reseal_journal_record(bytes);
        previous_on_the_device = Some((device, counter, journal_chain_value(bytes)));
        rewritten.push(write);
    }
    assert!(
        !rewritten.is_empty(),
        "流 {} 里有 txg {checkpoint_txg} 那次发布的 journal 记录",
        flow.name
    );
    append_a_segment_of_writes(flow, rewritten);
}

/// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：txg 4 只在三条记录里、第一条的新根段写成 txg 3 那条根的。
/// 挂载之后的状态上 txg 4 是由记录施加出来的那一版：CPU 取末条那一条走它自己的树，GPU 走读内核拼 `R_DATA` 那一段同取末条、
/// 每个状态逐位相同（取先到的那一条就走 txg 3 的树，txg 4 自己的单元没走到，I-3.1、I-3.5 在 GPU 一侧红出来）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root(
) {
    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
        "crash-judge-gpu-fl-version-applied-only-by-records",
        "three-unit-file-applied-only-by-its-records-stale-first-record",
        &write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root",
    ) else {
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
}

/// Q3 甲与 Q1 甲下一次挂载那一支：新池新建文件那次发布的根记录与它的记录（两块盘各一份）里
/// 树表指针整条写零、重封。根落了的状态上最新根照 core 跟过去读不出（CPU 红 I-2.1、I-4.8、I-7.2、I-7.4），记录落了、根没落的状态上
/// 重放施加出来的那一版走不完（CPU 红 I-4.1）；GPU 走读内核 `w_walk_tree_table_and_mapping` 同判、每个状态逐位相同。
fn record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-publish-tree-table-pointer-all-zero");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        bytes
            [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
            .fill(0);
    });
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        bytes[36..36 + NODE_POINTER_BYTES].fill(0);
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero(
) {
    let flow = record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
        "crash-judge-gpu-all-zero-tree-table-pointer",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-2.1", "I-7.2", "I-4.1"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// Q1 甲由记录施加出来的那一版那一支：同上那条挂载之后的流（挂载读的是干净的三条记录），覆盖写之后在流尾另起一段，把 txg 4 的三条记录
/// （两块盘各一份）抄一遍、新根段里的树表指针都写零、重封、重链（挂载之后改记录，`checker_known_bad_images.rs` 同一造法）。
/// 那一段落了的状态上 txg 4 是由记录施加出来的那一版，CPU 照 core 跟过去、这一版走不完（红 I-2.1、I-4.8、I-7.4）；GPU 走读内核同判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer(
) {
    let mut flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
        "crash-judge-gpu-all-zero-tree-table-pointer-applied-only-by-records",
        "three-unit-file-applied-only-by-its-records-all-zero-tree-table-pointer",
        &|_records_left_as_written| {},
    );
    append_the_records_of_the_publish_rewritten(&mut flow, 4, &|bytes| {
        bytes
            [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
            .fill(0);
    });
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-2.1", "I-4.8", "I-7.4"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
