//! 被崩溃恢复抛弃的根与实例表链（I-9.29、I-3.14）：最新根的实例表解不开、链比 GPU 走的片数长交 CPU。

use super::*;

/// 被崩溃恢复抛弃的 C 那一段历史录成的流，与录制时 C 那次发布在录制流里占的那一截（[`record_a_root_abandoned_by_a_crash_recovery`] 交回）。
pub(super) struct FlowWithARootAbandonedByACrashRecovery {
    pub(super) flow: RecordedFlow,
    /// `flow.writes` 每次写在录制流（扣掉 mkfs 那几步）里的下标。
    stream_indexes: Vec<usize>,
    /// 录制流（扣掉 mkfs 那几步）里 C 那次发布之前的步数与到它为止的步数：C 那次发布的写，录制流下标落在这两个数之间。
    operations_before_the_abandoned_root: usize,
    operations_through_the_abandoned_root: usize,
    abandoned_root_checkpoint_txg: u64,
}

/// A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成名叫 `name` 的流（每一版的内容都登记进 `versions`），
/// 判的根是录制流里最后一次根槽写。
pub(super) fn record_a_root_abandoned_by_a_crash_recovery(
    tag: &str,
    name: &'static str,
) -> FlowWithARootAbandonedByACrashRecovery {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    let second_txg = second.root.checkpoint_txg.0;
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(second_txg),
        content: second_content(),
    });
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in second_txg + 1..=pool.output.root.checkpoint_txg.0 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let operations_before_the_third = pool.stream.operation_count() - pool.mkfs_operation_count;
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    let operations_through_the_third = pool.stream.operation_count() - pool.mkfs_operation_count;
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: third.root.checkpoint_txg,
        content: third_content(),
    });
    pool.output = third.clone();
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    assert_eq!(
        (
            abandoning.output.instance,
            abandoning.output.effective_root.checkpoint_txg.0 + 1,
            abandoning.output.row_publish.root().checkpoint_txg.0,
        ),
        (
            InstanceGeneration(3),
            third.root.checkpoint_txg.0,
            third.root.checkpoint_txg.0 + 1
        ),
        "恢复落到 C 之前那条根，实例 3 写行接在 C 的 txg 之后"
    );
    for txg in
        abandoning.output.row_publish.root().checkpoint_txg.0..=pool.output.root.checkpoint_txg.0
    {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    FlowWithARootAbandonedByACrashRecovery {
        flow: RecordedFlow {
            name,
            base: pool.memory_pool_after_mkfs(),
            writes,
            segments,
            judged_root_index,
            versions,
        },
        stream_indexes,
        operations_before_the_abandoned_root: operations_before_the_third,
        operations_through_the_abandoned_root: operations_through_the_third,
        abandoned_root_checkpoint_txg: third.root.checkpoint_txg.0,
    }
}

/// I-9.29（同号只许一个出生代） 判不了那一形（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
/// X 的 harness 坏镜像 `an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back`
/// 同一段历史，录成流）：A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成流之后 C 那片 inode 叶容器两份里第一个文件那条记录的对象出生代加一、
/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再照 `change` 改最新根（实例 3）指着的实例表（[`ChangeOfTheNewestInstanceTable`]）。
/// 交回流，与改实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
pub(super) fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
    tag: &str,
    change: ChangeOfTheNewestInstanceTable,
) -> (RecordedFlow, MemoryPool) {
    let FlowWithARootAbandonedByACrashRecovery {
        mut flow,
        stream_indexes,
        operations_before_the_abandoned_root: operations_before_the_third,
        operations_through_the_abandoned_root: operations_through_the_third,
        abandoned_root_checkpoint_txg: _,
    } = record_a_root_abandoned_by_a_crash_recovery(tag, change.flow_name());
    let inode_leaf_of_the_third: Vec<usize> = stream_indexes
        .iter()
        .enumerate()
        .filter(|(write_index, stream_index)| {
            (operations_before_the_third..operations_through_the_third).contains(*stream_index)
                && flow.writes[*write_index].kind == StepKind::UnitWrite
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
                })
        })
        .map(|(write_index, _)| write_index)
        .collect();
    assert_eq!(
        inode_leaf_of_the_third.len(),
        2,
        "C 那次发布写一片 inode 叶容器、两份副本"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &inode_leaf_of_the_third, &|bytes| {
        let object_birth =
            offset_of_the_inode_record(bytes, FIRST_FILE.0) + INODE_RECORD_OBJECT_BIRTH_OFFSET;
        let another_object_birth = read_u64_at(bytes, object_birth) + 1;
        bytes[object_birth..object_birth + 8].copy_from_slice(&another_object_birth.to_le_bytes());
    });
    let mut image_with_the_newest_instance_table_readable = flow.base.clone();
    image_with_the_newest_instance_table_readable.apply_writes(&flow.writes);
    let is_an_instance_table_unit = |bytes: &[u8]| {
        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
    };
    let newest_instance_table = last_unit_copies(
        &flow,
        "实例 3 写行那次写的实例表",
        &is_an_instance_table_unit,
        false,
    );
    for write_index in &newest_instance_table {
        assert!(
            stream_indexes[*write_index] >= operations_through_the_third,
            "最后写的那张实例表是实例 3 写行那次写的"
        );
    }
    match change {
        ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing => {
            for write_index in newest_instance_table {
                let WrittenContents::Bytes(bytes) = &mut flow.writes[write_index].contents else {
                    panic!("单元写是普通写");
                };
                let flipped = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
                bytes[flipped] ^= 0xff;
            }
        }
        ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &newest_instance_table,
                &insert_a_record_of_an_unregistered_kind_before_the_chain_record,
            );
        }
        ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage => {
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &newest_instance_table,
                &insert_a_copy_of_the_chain_record_before_the_chain_record,
            );
        }
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetNotZero => {
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &newest_instance_table,
                &set_the_flags_of_the_first_row_to_a_value_that_is_not_zero,
            );
        }
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetNotZeroAndItsPublishedTxgRaisedToTheNewestRoot => {
            let newest_root_txg = flow
                .versions
                .iter()
                .map(|version| version.checkpoint_txg.0)
                .max()
                .expect("流里至少一版");
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &newest_instance_table, &|bytes| {
                set_the_flags_of_the_first_row_to_a_value_that_is_not_zero(bytes);
                let first_row_published_txg =
                    PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET;
                bytes[first_row_published_txg..first_row_published_txg + 8]
                    .copy_from_slice(&newest_root_txg.to_le_bytes());
            });
        }
        ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
            let (_, other_rows) = rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                &flow,
                &newest_instance_table,
            );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, Vec::new()],
                PagesAfterTheFirstInTheFlow::Intact,
            );
            let first_page_copies = last_unit_copies(
                &flow,
                "改造成两片之后的第 0 片",
                &is_an_instance_table_unit,
                false,
            );
            let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &first_page_copies, &|bytes| {
                point_the_first_location_of_the_chain_record_at_slot(bytes, first_page_slot);
            });
        }
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, rows_of_the_abandoned_instance],
                PagesAfterTheFirstInTheFlow::Intact,
            );
            let first_page_copies = last_unit_copies(
                &flow,
                "改造成两片之后的第 0 片",
                &is_an_instance_table_unit,
                false,
            );
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &first_page_copies,
                &set_the_first_mac_byte_of_the_chain_record_pointer,
            );
        }
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, rows_of_the_abandoned_instance],
                PagesAfterTheFirstInTheFlow::Intact,
            );
            let first_page_copies = last_unit_copies(
                &flow,
                "改造成两片之后的第 0 片",
                &is_an_instance_table_unit,
                false,
            );
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &first_page_copies,
                &set_the_extent_offset_of_the_chain_record_pointer,
            );
        }
        ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            assert_eq!(rows_of_the_abandoned_instance.len(), 1, "实例 2 那一行恰好一行");
            let mut records = other_rows;
            records.push(chain_record_saying_a_next_page_without_a_pointer());
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &newest_instance_table, &|bytes| {
                replace_the_records_of_an_instance_table_page(bytes, &records);
            });
        }
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, rows_of_the_abandoned_instance],
                PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing,
            );
        }
        ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
            pages,
        ) => {
            let (_, other_rows) = rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                &flow,
                &newest_instance_table,
            );
            let mut rows_of_each_page = vec![other_rows];
            rows_of_each_page.resize(pages.count(), Vec::new());
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &rows_of_each_page,
                PagesAfterTheFirstInTheFlow::Intact,
            );
        }
    }
    (flow, image_with_the_newest_instance_table_readable)
}

/// 最新根（实例 3）指着的那张实例表在流里怎么改（两份副本一起改）。
#[derive(Clone, Copy, Debug)]
pub(super) enum ChangeOfTheNewestInstanceTable {
    /// 两份各翻一个载荷字节、不重封：位置条目里的整单元校验和对不上，两份都读不出（N5-d）。
    /// 实例 3 的根落了、C 写回了的状态上，候选集滤不掉被抛弃的 C：CPU 报 I-9.29 部分没判、不红，GPU 扫描内核那张表不比、不置位
    /// （照旧比的话 C 那条记录与现行线同号不同出生代，GPU 红 I-9.29、CPU 不红）。
    BothCopiesFlippedWithoutResealing,
    /// 链指针记录之前插一条没登记种类（kind 2）的记录、重封到根（攻方 N5-a，`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」）：
    /// I-3.14 不成立，这一片解不开（D18（块里携带什么信息） 已定项 11「kind 不是 0 或 1 ⇒ 这一片解不开」）：
    /// 主走读不收这一片的行、走不到末片，候选集滤不掉被抛弃的 C，CPU 报 I-9.29 部分没判、红 I-3.14，GPU 那张表不比、置 I-3.14 那一位。
    RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
    /// 链指针记录之前插一条一模一样的链指针记录（kind 1、无下一片）、重封到根：I-3.14「链指针记录只在一片末尾」不成立，处置同上一形。
    SecondChainRecordInsertedInTheMiddleOfThePage,
    /// 第一行（kind 0）的 flags 写成非 0（1）、重封到根：I-3.14「行的 flags 为 0」不成立（D18（块里携带什么信息） 已定项 11「第一版恒 0、
    /// 非 0 拒收」），处置同上两形。
    FirstRowFlagsSetNotZero,
    /// 同上一形，第一行（实例 1 那一行）的 T 另抬到流里最新那条根的 txg：只读不判的读法（池级 checker 的
    /// `instance_table_rows_of_root_without_judging`、GPU 扫描内核的 `r_instance_rows`）拒收这一片，I-3.12 两边都不判这张表；
    /// GPU 那条读法不判 flags、照收这一行时，指着这张表的实例 3 那几条根上 GPU 红 I-3.12 而 CPU 不红。记录核对器那条读法
    /// （`InstanceTableOfRootRecord::read`、GPU 恢复内核的 `load_instance_table_of_the_effective_root`）同样拒收：实例 2 那一行照在，
    /// GPU 照收这张表时按它把 C 那几次发布判成被抛弃、不罩，CPU 读不出这张表、照罩。
    FirstRowFlagsSetNotZeroAndItsPublishedTxgRaisedToTheNewestRoot,
    /// 拆成两片（第 0 片放实例 2 之外的行，第 1 片只有末片的链指针记录），第 0 片链指针记录里盘 0 那条位置条目的槽写成第 0 片自己的槽、
    /// 整单元校验和仍是第 1 片的，盘 1 那条完好（攻方 H-b，`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）。
    /// 读第 1 片时盘 0 那一份对不上、读回的是盘 1 那一份：「走过」按读回那一份的落点认，第 1 片没走过，主走读走到末片、门开，
    /// 候选集里没有实例 2 那一行、滤不掉 C，CPU 红 I-9.29。GPU 按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，GPU 不红而 CPU 红。
    SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
    /// 拆成两片：第 0 片放实例 2 之外的行、链指针记录指第 1 片；第 1 片放实例 2 那一行与末片的链指针记录，两份完好。第 0 片链指针记录里
    /// 那条指针 MAC 的头一个字节写 1、位置条目不动，重封到根（I-3.14 第五形：有下一片而位置指针头部有一段不是第一版的取值
    /// ）。CPU 三个读法都当第 0 片解不开：主走读红 I-3.14、行不收、链不往下走（那条指针 I-2.4 不判）、门关；
    /// 只读不判的读法与记录核对器读恢复那一版的实例表（`InstanceTableOfRootRecord::read`）读不出；引用集合那一遍记数不全。
    /// GPU 照走第 1 片时：走读内核不红 I-3.14、红 I-2.4，恢复内核读出实例 2 那一行、把 C 那几次发布判成被抛弃，都与 CPU 不一致。
    SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero,
    /// 同上一形，MAC 不动，第 0 片链指针记录里那条指针的 extent 偏移写 1（GPU 两处第五形判法
    /// 只删 extent 偏移那一截时 MAC 那一条流照绿）。CPU 三个读法与 GPU 三个内核该判的同上一形：走读内核与扫描内核经
    /// `w_chain_record_kind` 的 `pointer_extent_offset_is_zero`，恢复内核经 `next.extent_offset_zero`。
    SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero,
    /// 唯一那一片里去掉被抛弃的实例 2 那一行，链指针记录的「有无下一片」写 1、指针照旧全零（不合法的链指针记录）、重封到根
    /// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条，GPU 关门那一侧「链指针记录不合法」一形）：
    /// 主走读没走到末片，门关，CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比。门要是开了，候选集里没有实例 2 那一行、滤不掉 C，
    /// 比就红——GPU 走读内核把「链指针记录不合法」也当走到末片时，这条流上 GPU 红 I-9.29 而 CPU 不红。
    ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance,
    /// 拆成两片：第 0 片放实例 2 之外的行、链指针记录指第 1 片；第 1 片放实例 2 那一行与末片的链指针记录，两份各翻一个载荷字节、
    /// 不重封（第 1 片两份都读不出）。主走读读到第 0 片、第 1 片读不出，没走到末片，门关：
    /// CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比（门要是开了，候选集里没有实例 2 那一行，比就红）。
    SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable,
    /// 拆成 `pages` 片：第 0 片放实例 2 之外的行，其余各片只有链指针记录（攻方 H-c 那一形，`research/prompts/m3-batch25-code-r1-opus-output.md`
    /// 第 2.1 节）。主走读走到末片、门开，候选集里没有实例 2 那一行、滤不掉 C，C 与现行线同号不同出生代，CPU 红 I-9.29。
    SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(InstanceTableChainPages),
}

impl ChangeOfTheNewestInstanceTable {
    pub(super) const fn flow_name(self) -> &'static str {
        match self {
            Self::BothCopiesFlippedWithoutResealing => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-unreadable"
            }
            Self::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-unregistered-record-kind"
            }
            Self::SecondChainRecordInsertedInTheMiddleOfThePage => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-a-chain-record-in-the-middle"
            }
            Self::FirstRowFlagsSetNotZero => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-row-flags-not-zero"
            }
            Self::FirstRowFlagsSetNotZeroAndItsPublishedTxgRaisedToTheNewestRoot => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-row-flags-not-zero-and-its-txg-not-below-the-root"
            }
            Self::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page"
            }
            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-named-through-a-pointer-whose-mac-is-not-zero"
            }
            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-named-through-a-pointer-whose-extent-offset-is-not-zero"
            }
            Self::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-invalid-chain-record"
            }
            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-unreadable"
            }
            Self::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
                InstanceTableChainPages::AsManyAsTheGpuWalks,
            ) => "abandoned-root-naming-another-object-birth-newest-instance-table-of-256-pages",
            Self::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
                InstanceTableChainPages::OneMoreThanTheGpuWalks,
            ) => "abandoned-root-naming-another-object-birth-newest-instance-table-of-257-pages",
        }
    }
}

/// GPU 走读内核沿实例表链最多走几片（`crash_judge_checker_walk_of_units_allocation_and_extent_trees.wgsl` 的 `INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`；CPU 不设上限）。
const GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: usize = 256;

/// GPU 走读内核走满 [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 片、链还没完时交回的判不了原因字
/// （`crash_judge_checker_walk_of_units_allocation_and_extent_trees.wgsl` 的 `UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`：「条目太多」一类 3、站点号 61）。
pub(super) const GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 =
    3 | (61 << 8);

/// GPU 扫描内核沿实例表链读满 [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 片、链还没完时交回的判不了原因字（`crash_judge_checker_scan.wgsl`，
/// 「条目太多」一类 3）：候选根的引用集合那一处（`UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED`，站点号 37）、
/// 一条根自己那张实例表的全部行那一处（`UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED`，I-7.9 与 I-3.12 读它，站点号 38）。
pub(super) const GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 =
    3 | (37 << 8);

pub(super) const GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 =
    3 | (38 << 8);

/// 走读内核与扫描内核各写一份实例表链的片数上限（两个内核各自编译，共用段 `crash_judge_checker_common.wgsl` 里没有它）：两份都与
/// [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 相同（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第三节第 2 条
/// 「上限用命名常量」）。只读两个内核的源码，不用卡。
#[test]
fn the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages()
{
    let declaration = format!(
        "const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = {GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED}u;"
    );
    for (kernel, source_files) in [
        (
            "走读内核",
            &singlefs_verification::crash_judge_gpu::GPU_JUDGE_CHECKER_SOURCE_FILES[..],
        ),
        (
            "扫描内核",
            &singlefs_verification::crash_judge_gpu::GPU_JUDGE_CHECKER_SCAN_SOURCE_FILES[..],
        ),
    ] {
        let source = singlefs_verification::crash_judge_gpu::joined_wgsl_source(source_files);
        assert_eq!(
            source.matches(declaration.as_str()).count(),
            1,
            "{kernel}的源码（{}）里恰好一处 `{declaration}`",
            source_files
                .iter()
                .map(|(file_name, _source)| *file_name)
                .collect::<Vec<_>>()
                .join("、")
        );
    }
}

/// 改造出来的实例表链有几片。
#[derive(Clone, Copy, Debug)]
pub(super) enum InstanceTableChainPages {
    /// 恰好是 GPU 走读内核走得完的片数：两边都走到末片、都比。
    AsManyAsTheGpuWalks,
    /// 比 GPU 走读内核走得完的多一片：CPU 走到末片、比；GPU 走满上限链还没完，记判不了交 CPU。
    OneMoreThanTheGpuWalks,
}

impl InstanceTableChainPages {
    pub(super) const fn count(self) -> usize {
        match self {
            Self::AsManyAsTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED,
            Self::OneMoreThanTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED + 1,
        }
    }
}

/// 被崩溃恢复抛弃的 C 那一版的实例代号（流里 C 由实例 2 发布，恢复之后的实例 3 在它的实例表里给实例 2 写一行）。
const INSTANCE_OF_THE_ABANDONED_ROOT: u32 = 2;

/// 实例表一行里实例代号（4 字节）的偏移；第 0 字节是种类。
const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;

const INSTANCE_TABLE_RECORD_KIND_ROW: u8 = 0;

/// 链指针记录里「有无下一片」那一字节的偏移，与之后 86 字节节点指针的偏移。
const INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET: usize = 1;

const INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET: usize = 2;

/// 打包记录单元头里容器号（8 字节）的偏移：实例表那一片的容器号就是它在链上的片号。
pub(super) const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;

/// 节点指针里两条位置条目的起点（盘 4、槽 6、整单元校验和 4）。
pub(super) const POINTER_LOCATION_ENTRY_OFFSETS: [usize; 2] = [50, 64];

/// 一个槽的字节数（实例表一片占两槽）。
pub(super) const SLOT_BYTES: u64 = 16384;

/// 改造实例表链时第 1 片起写在哪：从这个槽起每片两槽（4 GiB 的盘上流里没写过、基镜像里全零的那一截，写之前逐片核）。
const FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST: u64 = 200_000;

/// 实例表一片里的记录，按记录序。
pub(super) fn records_of_an_instance_table_page(bytes: &[u8]) -> Vec<Vec<u8>> {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    (0..record_count)
        .map(|record_index| {
            let start = PACKED_UNIT_RECORDS_START + record_index * INSTANCE_TABLE_RECORD_BYTES;
            bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec()
        })
        .collect()
}

/// 实例表一片的载荷换成 `records`：记录数与声明长度跟着改，载荷里记录之后的字节写 0（重封归调用方）。
pub(super) fn replace_the_records_of_an_instance_table_page(bytes: &mut [u8], records: &[Vec<u8>]) {
    bytes[PACKED_UNIT_RECORDS_START..].fill(0);
    for (record_index, record) in records.iter().enumerate() {
        let start = PACKED_UNIT_RECORDS_START + record_index * INSTANCE_TABLE_RECORD_BYTES;
        bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(record);
    }
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(records.len())
            .expect("一片装得下")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &u16::try_from(records.len() * INSTANCE_TABLE_RECORD_BYTES)
            .expect("一片装得下")
            .to_le_bytes(),
    );
}

/// 最新那张实例表（单片）里的行分成两份：被抛弃的实例 2 那几行，与其余各行（都按原来的次序）。核过它是单片、最后一条是末片的链指针记录。
fn rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
    flow: &RecordedFlow,
    newest_instance_table: &[usize],
) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let page = flow.writes[newest_instance_table[0]]
        .bytes()
        .expect("单元写带字节");
    let mut records = records_of_an_instance_table_page(page);
    let chain_record = records.pop().expect("实例表一片里至少有链指针记录");
    assert_eq!(
        chain_record,
        chain_record_of_the_last_page(),
        "最新那张实例表只有一片：链指针记录说没有下一片、指针全零"
    );
    assert!(
        records
            .iter()
            .all(|record| record[0] == INSTANCE_TABLE_RECORD_KIND_ROW),
        "链指针记录之前都是行"
    );
    records.into_iter().partition(|row| {
        read_u32_at(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == INSTANCE_OF_THE_ABANDONED_ROOT
    })
}

/// 末片的链指针记录：种类 1、没有下一片、指针全零。
pub(super) fn chain_record_of_the_last_page() -> Vec<u8> {
    let mut record = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
    record[0] = INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD;
    record
}

/// 不合法的链指针记录：说有下一片，指针却全零（D18（块里携带什么信息） 已定项 11 的有无下一片与指针配不上）。
fn chain_record_saying_a_next_page_without_a_pointer() -> Vec<u8> {
    let mut record = chain_record_of_the_last_page();
    record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
    record
}

/// 照 `pointer`（一条实例表指针）的头与两条位置条目的盘，两条位置条目的槽换成 `slot`、整单元校验和换成 `whole_unit_checksum`。
pub(super) fn pointer_naming_a_page_at(
    pointer: &[u8],
    slot: u64,
    whole_unit_checksum: u32,
) -> Vec<u8> {
    let mut pointer = pointer.to_vec();
    for location in POINTER_LOCATION_ENTRY_OFFSETS {
        pointer[location + 4..location + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
        pointer[location + 10..location + 14].copy_from_slice(&whole_unit_checksum.to_le_bytes());
    }
    pointer
}

/// 指下一片的链指针记录：照 `pointer_to_the_first_page`（根记录里指第 0 片的那条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC。
pub(super) fn chain_record_naming_the_next_page(
    pointer_to_the_first_page: &[u8],
    next_page_slot: u64,
    next_page_whole_unit_checksum: u32,
) -> Vec<u8> {
    let pointer = pointer_naming_a_page_at(
        pointer_to_the_first_page,
        next_page_slot,
        next_page_whole_unit_checksum,
    );
    let mut record = chain_record_of_the_last_page();
    record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
    record[INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
        ..INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET + NODE_POINTER_BYTES]
        .copy_from_slice(&pointer);
    record
}

/// 节点指针第 `location` 条位置条目里的盘与槽。
pub(super) fn device_and_slot_of_a_pointer_location(pointer: &[u8], location: usize) -> (u32, u64) {
    let mut slot = [0u8; 8];
    slot[..6].copy_from_slice(&pointer[location + 4..location + 10]);
    (read_u32_at(pointer, location), u64::from_le_bytes(slot))
}

/// 改造出来的实例表链上第 1 片起各片怎么写进流。
#[derive(Clone, Copy, Debug)]
pub(super) enum PagesAfterTheFirstInTheFlow {
    /// 照造出来的字节写，两份都读得出。
    Intact,
    /// 两份各翻一个载荷字节、不重封（前一片链指针记录里的整单元 CRC 按翻之前算）：两份都读不出。
    BothCopiesFlippedWithoutResealing,
}

/// 照 `template`（实例表的一片）的头造一段链：第 i 片（片号 `first_page_index + i`）放 `rows_of_each_page[i]` 再补链指针记录
/// （末片说没有下一片，其余指下一片：照 `pointer_head`（一条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC），容器号写片号、重封；
/// 片号 k 的那一片在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] + 2k 槽。交回各片的 (槽, 字节)，按片号升序。
pub(super) fn instance_table_pages_from(
    template: &[u8],
    pointer_head: &[u8],
    first_page_index: usize,
    rows_of_each_page: &[Vec<Vec<u8>>],
) -> Vec<(u64, Vec<u8>)> {
    let mut pages_from_the_last: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut next_page: Option<(u64, u32)> = None;
    for (index_in_the_run, rows) in rows_of_each_page.iter().enumerate().rev() {
        let page_index =
            u64::try_from(first_page_index + index_in_the_run).expect("片号装得进 u64");
        let mut records = rows.clone();
        records.push(match next_page {
            Some((slot, whole_unit_checksum)) => {
                chain_record_naming_the_next_page(pointer_head, slot, whole_unit_checksum)
            }
            None => chain_record_of_the_last_page(),
        });
        let mut page = template.to_vec();
        replace_the_records_of_an_instance_table_page(&mut page, &records);
        page[PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
            .copy_from_slice(&page_index.to_le_bytes());
        reseal_both_checksums_of_the_unit_bytes(&mut page);
        let slot = FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST + 2 * page_index;
        next_page = Some((slot, crc32c(&page)));
        pages_from_the_last.push((slot, page));
    }
    pages_from_the_last.reverse();
    pages_from_the_last
}

/// `pages`（[`instance_table_pages_from`] 交回的 (槽, 字节)）在 `devices` 两块盘上各写一份的单元写，照 `how` 写：核过每一片的槽在这块盘上
/// 流里没写过、基镜像里全零。
pub(super) fn writes_of_instance_table_pages(
    flow: &RecordedFlow,
    devices: [singlefs_core::address::DeviceIdentity; 2],
    pages: Vec<(u64, Vec<u8>)>,
    how: PagesAfterTheFirstInTheFlow,
) -> Vec<RetainedWrite> {
    let mut page_writes: Vec<RetainedWrite> = Vec::new();
    for (slot, page) in pages {
        let page_offset = slot * SLOT_BYTES;
        let page_end = page_offset + u64::try_from(page.len()).expect("单元长装得进 u64");
        for device in devices {
            assert!(
                flow.writes.iter().all(|write| write.device != device
                    || write.offset.0 + write.length_in_bytes() <= page_offset
                    || write.offset.0 >= page_end),
                "实例表那一片写的槽 {slot} 在盘 {} 上流里没写过",
                device.0
            );
            assert!(
                singlefs_core::recovery::PoolReader::read(
                    &flow.base,
                    device,
                    singlefs_core::address::DeviceOffsetInBytes(page_offset),
                    page.len(),
                )
                .expect("基镜像里读得到这一截")
                .iter()
                .all(|byte| *byte == 0),
                "实例表那一片写的槽 {slot} 在盘 {} 的基镜像里全零",
                device.0
            );
            let mut written = page.clone();
            match how {
                PagesAfterTheFirstInTheFlow::Intact => {}
                PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing => {
                    let flipped = written.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
                    written[flipped] ^= 0xff;
                }
            }
            page_writes.push(RetainedWrite {
                device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(page_offset),
                contents: WrittenContents::Bytes(written),
            });
        }
    }
    page_writes
}

/// 一条实例表指针两条位置条目的盘。
pub(super) fn devices_of_an_instance_table_pointer(
    pointer: &[u8],
) -> [singlefs_core::address::DeviceIdentity; 2] {
    POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
        singlefs_core::address::DeviceIdentity(
            device_and_slot_of_a_pointer_location(pointer, location).0,
        )
    })
}

/// 最新根（实例 3）指着的那张单片实例表（`first_page_copies` 是写它的两份副本）改造成一条 `rows_of_each_page.len()` 片的链：
/// 第 k 片放 `rows_of_each_page[k]` 再补链指针记录（末片说没有下一片，其余指下一片）。第 0 片就地改、重封到根；第 1 片起照第 0 片的头造
/// （[`instance_table_pages_from`]），两份写在与第 0 片同一对盘上，照 `pages_after_the_first` 写进流（[`writes_of_instance_table_pages`]）；
/// 这些写另起一段、插在写第 0 片那一段之前（第 0 片落了的状态上它们都已落）。
fn rebuild_the_newest_instance_table_as_a_chain(
    flow: &mut RecordedFlow,
    first_page_copies: &[usize],
    rows_of_each_page: &[Vec<Vec<u8>>],
    pages_after_the_first: PagesAfterTheFirstInTheFlow,
) {
    let first_page = flow.writes[first_page_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
    let pointer_to_the_first_page = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .filter_map(RetainedWrite::bytes)
        .map(|root| {
            root[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
                .to_vec()
        })
        .find(|pointer| {
            device_and_slot_of_a_pointer_location(pointer, POINTER_LOCATION_ENTRY_OFFSETS[0]).1
                == first_page_slot
        })
        .expect("流里有根记录指着最新那张实例表");
    let built_pages_after_the_first = instance_table_pages_from(
        &first_page,
        &pointer_to_the_first_page,
        1,
        &rows_of_each_page[1..],
    );
    let mut first_page_records = rows_of_each_page[0].clone();
    first_page_records.push(match built_pages_after_the_first.first() {
        Some((slot, page)) => {
            chain_record_naming_the_next_page(&pointer_to_the_first_page, *slot, crc32c(page))
        }
        None => chain_record_of_the_last_page(),
    });
    corrupt_units_and_reseal_up_to_the_root(flow, first_page_copies, &|bytes| {
        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
    });
    let page_writes = writes_of_instance_table_pages(
        flow,
        devices_of_an_instance_table_pointer(&pointer_to_the_first_page),
        built_pages_after_the_first,
        pages_after_the_first,
    );
    let first_page_write = *first_page_copies.iter().min().expect("第 0 片至少一份");
    insert_a_segment_of_writes_before_the_segment_of(flow, first_page_write, page_writes);
}

/// `writes` 另起一段，插在 `write_index` 那次写所在那一段之前（那一段之前的各段之后）：之后各次写的下标与 `judged_root_index` 跟着挪。
pub(super) fn insert_a_segment_of_writes_before_the_segment_of(
    flow: &mut RecordedFlow,
    write_index: usize,
    writes: Vec<RetainedWrite>,
) {
    let segment_of_the_write = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&write_index))
        .expect("那次写在某一段里");
    let insertion = *flow.segments[segment_of_the_write]
        .iter()
        .min()
        .expect("段里至少一次写");
    assert!(
        flow.segments[..segment_of_the_write]
            .iter()
            .flatten()
            .all(|index| *index < insertion)
            && flow.segments[segment_of_the_write..]
                .iter()
                .flatten()
                .all(|index| *index >= insertion),
        "段按写序排：插进来的写在那一段之前、之前各段之后"
    );
    let inserted = writes.len();
    flow.writes.splice(insertion..insertion, writes);
    for segment in &mut flow.segments {
        for index in segment.iter_mut() {
            if *index >= insertion {
                *index += inserted;
            }
        }
    }
    flow.segments.insert(
        segment_of_the_write,
        (insertion..insertion + inserted).collect(),
    );
    if flow.judged_root_index >= insertion {
        flow.judged_root_index += inserted;
    }
}

/// 实例表一条记录的字节数（D18（块里携带什么信息） 已定项 11）；第 0 字节是种类：1 链指针记录，2 没登记。
pub(super) const INSTANCE_TABLE_RECORD_BYTES: usize = 88;

const INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD: u8 = 1;

const INSTANCE_TABLE_RECORD_KIND_UNREGISTERED: u8 = 2;

/// 实例表一片里链指针记录（最后一条）往后挪一条，原位插一条全零、种类 2 的记录，记录数与声明长度各加一条（重封归调用方）。
fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [u8]) {
    let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
    record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
    insert_a_record_before_the_chain_record(bytes, &record_of_an_unregistered_kind);
}

/// 实例表一片里链指针记录之前插一条它自己的拷贝：片中间多一条链指针记录（重封归调用方）。
fn insert_a_copy_of_the_chain_record_before_the_chain_record(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    let chain_record =
        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
    insert_a_record_before_the_chain_record(bytes, &chain_record);
}

/// 实例表行记录（kind 0）里 flags 字节的偏移（kind 1 + 实例代号 4 + T 8 + W 8，D18（块里携带什么信息） 已定项 11）
/// 与坏流往那一格写的非 0 取值（同一项「第一版恒 0、非 0 拒收」）。
const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;

const INSTANCE_TABLE_ROW_FLAGS_NOT_ZERO: u8 = 1;

/// 实例表一片末尾那条指下一片的链指针记录里，第 0 条位置条目的槽换成 `slot`（盘与整单元校验和不动；重封归调用方）。
fn point_the_first_location_of_the_chain_record_at_slot(bytes: &mut [u8], slot: u64) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        (
            bytes[chain_record_start],
            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
        "最后一条是指下一片的链指针记录"
    );
    let first_location_slot = chain_record_start
        + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
        + POINTER_LOCATION_ENTRY_OFFSETS[0]
        + 4;
    bytes[first_location_slot..first_location_slot + 6].copy_from_slice(&slot.to_le_bytes()[..6]);
}

/// 指针头部 MAC 16 字节的头一个字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表：MAC 从偏移 0 起）。
const POINTER_MAC_FIRST_BYTE_OFFSET: usize = 0;

/// 实例表一片末尾那条指下一片的链指针记录里，位置指针 MAC 的头一个字节写 1（I-3.14 第五形：有下一片而位置指针头部不是第一版的取值；
/// 位置条目不动，指的仍是下一片；重封归调用方）。
fn set_the_first_mac_byte_of_the_chain_record_pointer(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        (
            bytes[chain_record_start],
            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
        "最后一条是指下一片的链指针记录"
    );
    let first_mac_byte = chain_record_start
        + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
        + POINTER_MAC_FIRST_BYTE_OFFSET;
    assert_eq!(
        bytes[first_mac_byte], 0,
        "照第 0 片指针的头造的链指针 MAC 全零"
    );
    bytes[first_mac_byte] = 1;
}

/// 实例表一片末尾那条指下一片的链指针记录里，位置指针的 extent 偏移写 1（I-3.14 第五形 extent 偏移那一段；
/// MAC 与位置条目不动，指的仍是下一片；重封归调用方）。
fn set_the_extent_offset_of_the_chain_record_pointer(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        (
            bytes[chain_record_start],
            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
        "最后一条是指下一片的链指针记录"
    );
    set_the_extent_offset_of_the_pointer_at(
        bytes,
        chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET,
    );
}

/// 实例表一片的第一条（实例 3 写行那一片里是一行）flags 写成非 0（1；重封归调用方）。
fn set_the_flags_of_the_first_row_to_a_value_that_is_not_zero(bytes: &mut [u8]) {
    let first_row_flags = PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_FLAGS_OFFSET;
    assert_eq!(
        (bytes[PACKED_UNIT_RECORDS_START], bytes[first_row_flags]),
        (0, 0),
        "实例表这一片的第一条是行、flags 是 0"
    );
    bytes[first_row_flags] = INSTANCE_TABLE_ROW_FLAGS_NOT_ZERO;
}

/// 实例表一片里链指针记录（最后一条）往后挪一条，原位放 `record`，记录数与声明长度各加一条（重封归调用方）。
fn insert_a_record_before_the_chain_record(bytes: &mut [u8], record: &[u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        bytes[chain_record_start], INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD,
        "实例表一片的最后一条是链指针记录"
    );
    let chain_record =
        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
        .copy_from_slice(record);
    bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
        ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
        .copy_from_slice(&chain_record);
    let new_record_count = record_count + 1;
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count)
            .expect("一片装得下")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
            .expect("一片装得下")
            .to_le_bytes(),
    );
}

/// 实例表一行里本实例最后一次发布的 `checkpoint_txg`（T，8 字节）的偏移（种类 1 字节、实例代号 4 字节之后）。
const INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET: usize = 5;

/// journal 记录新根段里实例表指针（86 字节）离新根段起点的偏移（walk.rs 的 `NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER`）。
pub(super) const JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 188;

/// C 自己那张实例表里改了 T 的那一行的实例代号：实例 2 写行那次写的表里只有实例 1 那一行。
const INSTANCE_OF_THE_ROW_MOVED_TO_THE_ABANDONED_ROOT_TXG: u32 = 1;

/// O-1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 K1 J1 那一行、第三节第 2 条）：被崩溃恢复抛弃的 C
/// （[`record_a_root_abandoned_by_a_crash_recovery`]；实例 3 的根落了之后 C 在回退候选集外）自己那张实例表换成一条 257 片的新链。
/// C 的根记录原来指着实例 2 写行那次写的单片实例表（实例 2 的几条根共用），改指新链的第 0 片：第 0 片放那张表的行，实例 1 那一行的 T 改成
/// C 自己的 txg（I-3.12「T < 这条根的 checkpoint_txg」不成立），其余各片只有链指针记录（[`instance_table_pages_from`]，头照那张表）；
/// C 那次发布的根槽写（发布时一次、恢复之后原样写回一次）与 journal 记录新根段里的实例表指针都改指第 0 片、重封；新链各片两份另起一段，
/// 插在 C 那次发布的记录与根槽写之前（引用它们的写落了的状态上它们都已落）。CPU 一侧先钉流造对了：整条落盘的镜像上 I-3.12 红。
fn record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
    tag: &str,
) -> RecordedFlow {
    let FlowWithARootAbandonedByACrashRecovery {
        mut flow,
        stream_indexes,
        operations_before_the_abandoned_root,
        operations_through_the_abandoned_root,
        abandoned_root_checkpoint_txg,
    } = record_a_root_abandoned_by_a_crash_recovery(
        tag,
        "abandoned-root-own-instance-table-of-257-pages-with-a-row-at-its-own-txg",
    );
    let root_slot_write_of_the_publish: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            (operations_before_the_abandoned_root..operations_through_the_abandoned_root)
                .contains(&stream_indexes[*write_index])
                && flow.writes[*write_index].kind == StepKind::RootRecordFua
        })
        .collect();
    assert_eq!(
        root_slot_write_of_the_publish.len(),
        1,
        "C 那次发布写一次根槽"
    );
    let root_of_the_abandoned = flow.writes[root_slot_write_of_the_publish[0]]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    assert_eq!(
        read_u64_at(&root_of_the_abandoned, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
        abandoned_root_checkpoint_txg,
        "那次根槽写写的是 C"
    );
    let instance_of_the_abandoned = read_u32_at(&root_of_the_abandoned, 24);
    let root_slot_writes_of_the_abandoned: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::RootRecordFua
                && flow.writes[*write_index].bytes() == Some(root_of_the_abandoned.as_slice())
        })
        .collect();
    assert_eq!(
        root_slot_writes_of_the_abandoned.len(),
        2,
        "C 的根槽写两次：发布时一次，恢复之后原样写回一次"
    );
    let journal_records_of_the_abandoned: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::JournalRecord
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET) == instance_of_the_abandoned
                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
                            == abandoned_root_checkpoint_txg
                })
        })
        .collect();
    assert!(
        !journal_records_of_the_abandoned.is_empty(),
        "C 那次发布写了 journal 记录"
    );
    let pointer_to_the_shared_table = root_of_the_abandoned
        [ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
            ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
        .to_vec();
    let journal_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
            + NODE_POINTER_BYTES;
    for record in &journal_records_of_the_abandoned {
        assert_eq!(
            &flow.writes[*record].bytes().expect("记录写带字节")[journal_pointer_field.clone()],
            pointer_to_the_shared_table.as_slice(),
            "C 那次发布的记录新根段里的实例表指针与 C 的根记录相同"
        );
    }
    let (shared_table_device, shared_table_slot) = device_and_slot_of_a_pointer_location(
        &pointer_to_the_shared_table,
        POINTER_LOCATION_ENTRY_OFFSETS[0],
    );
    let shared_table = flow.writes[..root_slot_write_of_the_publish[0]]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device.0 == shared_table_device
                && write.offset.0 / SLOT_BYTES == shared_table_slot
        })
        .and_then(RetainedWrite::bytes)
        .expect("C 之前流里写过 C 指着的那张实例表")
        .to_vec();
    assert!(
        unit_class(&shared_table) == 3
            && packed_record_type(&shared_table) == PACKED_RECORD_TYPE_INSTANCE_TABLE,
        "C 的根记录指着一片实例表"
    );
    let mut rows = records_of_an_instance_table_page(&shared_table);
    assert_eq!(
        rows.pop(),
        Some(chain_record_of_the_last_page()),
        "C 指着的那张实例表只有一片"
    );
    let mut moved_rows = 0usize;
    for row in &mut rows {
        if row[0] == INSTANCE_TABLE_RECORD_KIND_ROW
            && read_u32_at(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET)
                == INSTANCE_OF_THE_ROW_MOVED_TO_THE_ABANDONED_ROOT_TXG
        {
            row[INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET
                ..INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET + 8]
                .copy_from_slice(&abandoned_root_checkpoint_txg.to_le_bytes());
            moved_rows += 1;
        }
    }
    assert_eq!(moved_rows, 1, "那张表里实例 1 恰好一行");
    let mut rows_of_each_page = vec![rows];
    rows_of_each_page.resize(
        InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),
        Vec::new(),
    );
    let pages = instance_table_pages_from(
        &shared_table,
        &pointer_to_the_shared_table,
        0,
        &rows_of_each_page,
    );
    let pointer_to_the_new_first_page = pointer_naming_a_page_at(
        &pointer_to_the_shared_table,
        pages[0].0,
        crc32c(&pages[0].1),
    );
    for root_slot_write in &root_slot_writes_of_the_abandoned {
        rewrite_root_record_and_reseal(&mut flow, *root_slot_write, &|bytes| {
            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
                .copy_from_slice(&pointer_to_the_new_first_page);
        });
    }
    for record in &journal_records_of_the_abandoned {
        rewrite_journal_record_and_reseal(&mut flow, *record, &|bytes| {
            bytes[journal_pointer_field.clone()].copy_from_slice(&pointer_to_the_new_first_page);
        });
    }
    let page_writes = writes_of_instance_table_pages(
        &flow,
        devices_of_an_instance_table_pointer(&pointer_to_the_shared_table),
        pages,
        PagesAfterTheFirstInTheFlow::Intact,
    );
    let first_write_naming_the_new_chain = *root_slot_writes_of_the_abandoned
        .iter()
        .chain(&journal_records_of_the_abandoned)
        .min()
        .expect("至少一次写");
    insert_a_segment_of_writes_before_the_segment_of(
        &mut flow,
        first_write_naming_the_new_chain,
        page_writes,
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_every_write, "I-3.12"),
            InvariantVerdict::Violated(_)
        ),
        "流 {}：CPU 沿 C 自己那张 257 片的实例表读到末片，实例 1 那一行的 T 等于 C 的 txg，I-3.12 红：{:?}",
        flow.name,
        pool_checker_verdict_on_the_image(&image_with_every_write, "I-3.12")
    );
    flow
}

/// O-1：回退候选集外的根（被崩溃恢复抛弃的 C）自己那张实例表链 257 片
/// （[`record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks`]）。CPU 读链不设片数上限，读到末片、判 I-3.12 红；
/// GPU 走满上限链还没完，池级 checker 记判不了（实例表链过长那三种原因字之一），不当读不出、跳过 I-3.12。逐状态：池级 checker 判了的，
/// 违例位与 CPU 逐位相同；判不了的，原因字只能是实例表链过长，且至少一个这样的状态上 CPU 红 I-3.12；恢复内核判不了的，原因只能是
/// 沿链读满 256 片还有下一片（[`GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES`]，恢复读到 C 的状态）；判不了就回落 CPU
/// （`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
/// 记哪一个原因字是推的（读码，没跑过）：走读内核把根环里每条自证过的根都另起一份走读走一遍（最新根、候选根、被抛弃的根、F 之下的根各一个阶段，
/// `crash_judge_checker_walk_driver_pool_judgements_and_entry.wgsl` 的 `walk_and_hand_off`），每次先走它的实例表链，链 257 片先在那里记 61，扫描内核不跑；扫描内核那两处（37、38）
/// 是走读内核哪天不再走某一类根时的接手，这条流上落不到。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates(
) {
    let test_name = "the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates";
    let flow =
        record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
            "crash-judge-gpu-i312-abandoned-root-instance-table-of-257-pages",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let chain_limit_reasons = vec![
        GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED,
        GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
        GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
    ];
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::LeftToTheCpuOnlyFor {
            reasons_left_to_the_cpu: chain_limit_reasons,
            // 恢复读到 C（链 257 片）的状态上恢复内核沿链读满 256 片还有下一片，按设计交 CPU（E9 那一趟这条流 56 个状态记 4）
            recovery_reasons_left_to_the_cpu:
                GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES
                    .to_vec(),
            reasons_one_state_must_carry: Vec::new(),
            invariant_red_on_a_state_left_to_the_cpu: Some("I-3.12"),
        }],
    ) else {
        return;
    };
    let left_to_the_cpu = &comparison.examined[0];
    eprintln!(
        "CRASH_JUDGE_GPU_CHAIN_LIMIT flow={} states={} left_to_the_cpu_by_a_chain_limit={}",
        flow.name, comparison.states, left_to_the_cpu.states_left_to_the_cpu
    );
    assert!(
        left_to_the_cpu.states_left_to_the_cpu > 0,
        "流 {}：C 的实例表链 257 片、C 的根读得出的状态上 GPU 池级 checker 要记判不了（实例表链过长）",
        flow.name
    );
    assert!(
        left_to_the_cpu.states_left_to_the_cpu_and_red_on_the_invariant > 0,
        "流 {}：GPU 因链过长判不了的状态里，至少一个 CPU 红 I-3.12（CPU 读到末片、判）",
        flow.name
    );
    assert!(
        left_to_the_cpu.disagreements.is_empty(),
        "流 {}：{} 处不一致（前几个）：\n{}",
        flow.name,
        left_to_the_cpu.disagreements.state_count(),
        left_to_the_cpu
            .disagreements
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 池级 checker（CPU，旧入口）在一份镜像上判 `invariant` 的结果：镜像先经对拍那一类的库（写进去再读回：
/// CPU 参照也从库读，`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`），判的是读回的那一份。
///
/// # Panics
/// 只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）：取卡之前的前提要从库里读回才判得了，这一趟不判。
// cpu-reference-for-gpu-comparison
pub(super) fn pool_checker_verdict_on_the_image(
    image: &MemoryPool,
    invariant: &str,
) -> InvariantVerdict {
    let read_back = cpu_reference_table_read_back_from_the_pairing_library(
        &RecordedGivenStatesTable {
            table_name: "pool-checker-precondition".to_string(),
            base: image.clone(),
            writes: Vec::new(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: vec![Vec::new()],
        },
    )
    .unwrap_or_else(|| {
        panic!(
            "只录那一趟：取卡之前判 {invariant} 的前提要从库里读回那张镜像才判得了；先跑录入段，再不设选段或设 judge 跑判段"
        )
    });
    check_pool_image(&read_back.base)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .map(|(_, verdict)| verdict)
        .unwrap_or_else(|| panic!("池级 checker 报了 {invariant}"))
}

/// I-9.29 判不了那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`] 取
/// [`ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing`]）：
/// CPU 一侧先钉流造对了——翻坏实例表之前整条落盘的镜像上 C 被滤掉、I-9.29 真被评估过且成立；翻坏之后整条落盘的镜像上 I-9.29 报部分没判。
/// 流上 C 还是最新根的那几个状态，C 的实例表读得出、C 进候选集，同号不同出生代照比，CPU 红 I-9.29；GPU 每个状态逐位相同
/// （实例 3 的根落了、C 写回了的状态上 GPU 照旧比的话红 I-9.29 而 CPU 不红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back(
) {
    let (flow, image_with_the_newest_instance_table_readable) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            "crash-judge-gpu-i929-newest-instance-table-unreadable",
            ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing,
        );
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
        InvariantVerdict::Holds,
        "最新根的实例表读得出：被抛弃的 C 按它滤掉、不进候选集，现行线上各版同号同出生代，I-9.29 真被评估过且成立"
    );
    let mut image_with_the_newest_instance_table_unreadable = flow.base.clone();
    image_with_the_newest_instance_table_unreadable.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_unreadable, "I-9.29") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "最新根的实例表两份都翻坏、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
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

/// I-3.14（实例表每片记录形态合法）不成立的三形各一条流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`]
/// 取 `change`）共用的判法。CPU 一侧先钉流造对了——整条落盘的镜像上最新根那一片解不开（D18（块里携带什么信息） 已定项 11
/// 「kind 不是 0 或 1 ⇒ 这一片解不开」）：主走读不收它的行、走不到末片，I-3.14 红，I-9.29 报部分没判。
/// 流上 C 还是最新根的那几个状态 C 的实例表读得出、C 进候选集，CPU 红 I-9.29；实例 3 的根落了的状态 CPU 红 I-3.14；GPU 每个状态逐位相同
/// （GPU 照收这一片的行、走到末片时，实例 3 的根落了、C 写回了的状态上 GPU 比那张表而红 I-9.29，CPU 不比）。
/// `invariants_the_cpu_never_reddens` 里的每一条，整条落盘的镜像上不是违例、流上一个状态都不红（CPU 一侧钉流造对了的另一半）。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
    test_name: &str,
    tag: &str,
    change: ChangeOfTheNewestInstanceTable,
    invariants_the_cpu_never_reddens: &[&str],
) {
    let (flow, _) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            tag, change,
        );
    let mut image_with_the_page_that_does_not_decode = flow.base.clone();
    image_with_the_page_that_does_not_decode.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, "I-3.14"),
            InvariantVerdict::Violated(_)
        ),
        "{change:?}：最新根的实例表那一片记录形态不合法，I-3.14 红"
    );
    match pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, "I-9.29") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "{change:?}：部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "{change:?}：最新根的实例表那一片解不开、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
    for invariant in invariants_the_cpu_never_reddens {
        let verdict =
            pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, invariant);
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "{change:?}：整条落盘的镜像上 {invariant} 不是违例，实际 {verdict:?}"
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-9.29", "I-3.14"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    for invariant in invariants_the_cpu_never_reddens {
        assert!(
            !comparison.cpu_checker_red_invariants.contains(*invariant),
            "流 {} 上 CPU 一个状态都不红 {invariant}（红过的：{:?}）",
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

/// 攻方 N5-a 那一形的流（[`ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord`]；判决
/// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条「GPU 同改，对拍补 N5-a」），判法见
/// [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode`]。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode",
        "crash-judge-gpu-i929-newest-instance-table-unregistered-record-kind",
        ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
        &[],
    );
}

/// 片中间多一条链指针记录那一形的流（[`ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage`]），判法同上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-chain-record-in-the-middle",
        ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage,
        &[],
    );
}

/// 行的 flags 不是 0 那一形的流（[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetNotZero`]），判法同上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-row-flags-not-zero",
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetNotZero,
        &[],
    );
}

/// I-3.14 第五形的流（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero`]
/// ），判法同上，另钉 CPU 一个状态都不红 I-2.4：那条指针不跟随、它的头部不判。三个内核都要把这一形当解不开——
/// 走读内核与扫描内核经 `crash_judge_checker_common.wgsl` 的 `w_chain_record_kind`，恢复内核在 `load_instance_table_of_the_effective_root` 里判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-chain-pointer-mac-not-zero",
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero,
        &["I-2.4"],
    );
}

/// I-3.14 第五形 extent 偏移那一段的流（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero`]
/// ），判法与另钉的同上一条：GPU 两处判法只删 extent 偏移那一截（`w_chain_record_kind` 的
/// `pointer_extent_offset_is_zero`、恢复内核的 `next.extent_offset_zero`）时 MAC 那一条照绿，这一条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_extent_offset_is_not_zero_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_extent_offset_is_not_zero_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-chain-pointer-extent-offset-not-zero",
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero,
        &["I-2.4"],
    );
}

/// 行的 flags 不是 0、那一行的 T 抬到最新根的 txg 那一形的流
/// （[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetNotZeroAndItsPublishedTxgRaisedToTheNewestRoot`]），判法同上，
/// 另钉 CPU 一个状态都不红 I-3.12：只读不判的读法拒收这一片，指着它的根不判。GPU 扫描内核 `r_instance_rows` 不判 flags 时照收这一行、
/// 红 I-3.12；GPU 恢复内核 `load_instance_table_of_the_effective_root` 不判 flags 时照收这张表、按实例 2 那一行不罩 C 那几次发布，
/// 两处都与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant",
        "crash-judge-gpu-i312-newest-instance-table-row-flags-not-zero-txg-not-below-the-root",
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetNotZeroAndItsPublishedTxgRaisedToTheNewestRoot,
        &["I-3.12"],
    );
}

/// 攻方 H-b 那一形的流（[`ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage`]）：
/// CPU 一侧先钉流造对了——整条落盘的镜像上主走读读回第 1 片盘 1 那一份、
/// 按它的落点认「走过」、走到末片，门开，候选集里没有实例 2 那一行、滤不掉 C，I-9.29 红（不是部分没判）。
/// GPU 每个状态逐位相同（GPU 走读内核按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，实例 3 的根落了、C 写回了的状态上
/// GPU 不红 I-9.29 而 CPU 红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back(
) {
    let (flow, image_with_the_newest_instance_table_readable) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            "crash-judge-gpu-i929-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page",
            ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
        );
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
        InvariantVerdict::Holds,
        "改实例表之前：被抛弃的 C 按最新根的实例表滤掉，I-9.29 真被评估过且成立"
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    let verdict = pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29");
    assert!(
        matches!(verdict, InvariantVerdict::Violated(_)),
        "流 {}：主走读按读回那一份认「走过」、走到第 1 片（末片），候选集里没有实例 2 那一行，I-9.29 红，实际 {verdict:?}",
        flow.name
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
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
