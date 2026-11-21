//! 里程碑三二·1 那一批：每条不变量一条只改坏它管的那个字段的流，与逐条钉 GPU 与 CPU 逐状态对拍的用例。

use super::*;

// ───────────── 里程碑三二·1 那一批：每条不变量一条只改坏它管的字段的流（GPU 与 CPU 逐状态逐位对拍，CPU 要在这条流上红那一条）─────────────

/// 系统配置槽：整槽校验和字段在 155、罩 4096 字节。
pub(super) const SYSTEM_CONFIGURATION_CHECKSUM_FIELD: usize = 155;

/// journal 记录头里新根段的起点（树表指针 86、映射根指针 86、树 ID 水位 8、回退下界 F 8）。
pub(super) const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;

/// 根记录里实例表指针的起点（86 字节的码 2 指针，出生树在指针偏移 34）。
pub(super) const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;

/// 码 2 / 码 3 指针头里出生树的偏移。
pub(super) const POINTER_BIRTH_TREE_OFFSET: usize = 34;

/// journal 记录头 496 字节（D23（journal 的角色与格式） 已定项 4），点名项从这里起、每项 56 字节（诞生代号在点名项偏移 37）。
pub(super) const JOURNAL_RECORD_HEADER_BYTES: usize = 496;

/// 点名项宽（D23（journal 的角色与格式） 已定项 17）。
pub(super) const JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD: usize = 56;

/// 发布的根（`judged_root_index` 那次根槽写）的 (实例代号, checkpoint_txg)：根记录偏移 24、28。
pub(super) fn publish_root_identity(flow: &RecordedFlow) -> (u32, u64) {
    let bytes = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节");
    (
        u32::from_le_bytes(bytes[24..28].try_into().expect("4")),
        u64::from_le_bytes(bytes[28..36].try_into().expect("8")),
    )
}

/// 改一次根槽写的字节，再重封根记录的整槽校验和（罩整个槽宽）。
pub(super) fn rewrite_root_record_and_reseal(
    flow: &mut RecordedFlow,
    index: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    assert_eq!(
        flow.writes[index].kind,
        StepKind::RootRecordFua,
        "改的是根槽写"
    );
    let WrittenContents::Bytes(bytes) = &mut flow.writes[index].contents else {
        panic!("根槽写是普通写");
    };
    rewrite(bytes);
    reseal_root_record(bytes);
}

/// 改一次 journal 记录写的字节，再重封载荷校验和与头校验和；同盘同实例计数器大 1 的那一条的反向链跟着改、一路传下去。
pub(super) fn rewrite_journal_record_and_reseal(
    flow: &mut RecordedFlow,
    index: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    assert_eq!(
        flow.writes[index].kind,
        StepKind::JournalRecord,
        "改的是 journal 记录写"
    );
    let WrittenContents::Bytes(bytes) = &mut flow.writes[index].contents else {
        panic!("记录写是普通写");
    };
    rewrite(bytes);
    reseal_journal_record(bytes);
    carry_the_back_chain_forward_from(flow, vec![index]);
}

/// 发布那次的每一份 journal 记录（每块盘一份：实例与 checkpoint_txg 都等于发布的根）各改一次、重封：重放与逐盘判定读到的都是改过的那一份。
pub(super) fn rewrite_every_journal_record_of_the_publish(
    flow: &mut RecordedFlow,
    rewrite: &dyn Fn(&mut [u8]),
) {
    let (instance, checkpoint_txg) = publish_root_identity(flow);
    let records: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[16..20].try_into().expect("4")) == instance
                        && u64::from_le_bytes(bytes[26..34].try_into().expect("8"))
                            == checkpoint_txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !records.is_empty(),
        "流 {} 里有发布那次（实例 {instance}、txg {checkpoint_txg}）的 journal 记录",
        flow.name
    );
    for index in records {
        rewrite_journal_record_and_reseal(flow, index, rewrite);
    }
}

/// 改流里最后一次系统配置槽写（它在那块盘上世代号最大，是那块盘择到的那一槽）的字节、重封整槽校验和。
fn rewrite_the_last_system_configuration_slot_and_reseal(
    flow: &mut RecordedFlow,
    rewrite: &dyn Fn(&mut Vec<u8>),
) {
    rewrite_and_reseal(
        flow,
        StepKind::SystemConfigurationSlot,
        0,
        4096,
        SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
        rewrite,
    );
}

/// 在流尾另起一段，放这几次写（它们在最后全部持久的那个状态里都落了）。
pub(super) fn append_a_segment_of_writes(flow: &mut RecordedFlow, writes: Vec<RetainedWrite>) {
    let first = flow.writes.len();
    let count = writes.len();
    flow.writes.extend(writes);
    flow.segments.push((first..first + count).collect());
}

/// I-1.12（指针出生树等于被指单元头）：发布的根记录里实例表指针的出生树加 1、重封根记录——实例表那一片单元头里的出生树是 0，指针不再与它相等。
fn record_new_pool_with_an_instance_table_pointer_naming_another_birth_tree(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-instance-table-pointer-birth-tree-plus-one");
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let offset = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + POINTER_BIRTH_TREE_OFFSET;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    flow
}

/// I-1.12（指针出生树等于被指单元头）数据单元那一支：发布那片 extent 树上段叶第一条内联的数据指针（条目偏移 25，标签 2）出生树加 1、
/// 重封到根——数据单元五元组里的树（头偏移 43）不再等于它。
fn record_new_pool_with_a_data_pointer_naming_another_birth_tree(tag: &str) -> RecordedFlow {
    let extent_upper_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 24
            && index_node_entry_width(bytes) == 113
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-data-pointer-birth-tree-plus-one",
        &extent_upper_leaf,
        false,
        &|bytes| {
            let first_entry = 115 + 2 * 24;
            assert_eq!(
                bytes[first_entry + 24],
                2,
                "上段叶第一条内联一个数据指针（标签 2）"
            );
            let birth_tree = first_entry + 25 + POINTER_BIRTH_TREE_OFFSET;
            bytes[birth_tree] = bytes[birth_tree].wrapping_add(1);
        },
    )
}

/// I-8.10（记录新根段与同代根相同）：发布那条记录新根段里的树 ID 水位加 1、重封——与同 (实例, txg) 的根里那 8 字节不再逐字节相同。
fn record_new_pool_with_a_last_record_carrying_another_watermark_than_its_root(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-last-record-watermark-plus-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let watermark = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86;
        bytes[watermark] = bytes[watermark].wrapping_add(1);
    });
    flow
}

/// I-8.11（点名项与被指单元头相同）：发布那条记录第一个点名项的诞生代号加 1、重封——它点名的单元整单元校验和照样对得上，头里的诞生代号不再等于它。
fn record_new_pool_with_a_named_entry_naming_another_birth_txg(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-named-entry-birth-txg-plus-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        assert!(
            u32::from_le_bytes(bytes[12..16].try_into().expect("4")) >= 1,
            "发布那条记录至少点名一个单元"
        );
        let birth_txg = JOURNAL_RECORD_HEADER_BYTES + 37;
        bytes[birth_txg] = bytes[birth_txg].wrapping_add(1);
    });
    flow
}

/// I-4.9（非最新候选根引用的单元不晚于它）：暖机最后那条根的树表指针与映射根指针换成发布的根的、重封——发布的根落了的状态上，
/// 这条更早的候选根引用的树表与映射是发布那一代诞生的（经「别的版本先走过」那一支判到）。
fn record_new_pool_with_an_older_root_pointing_at_the_trees_of_the_publish(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-warm-up-root-points-at-the-publish-trees");
    let publish_root = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    let warm_up_root = flow.writes[..flow.judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let (_, publish_txg) = publish_root_identity(&flow);
    rewrite_root_record_and_reseal(&mut flow, warm_up_root, &|bytes| {
        let warm_up_txg = u64::from_le_bytes(bytes[28..36].try_into().expect("8"));
        assert!(warm_up_txg < publish_txg, "暖机那条根比发布的根早");
        bytes[36..122].copy_from_slice(&publish_root[36..122]);
        bytes[256..342].copy_from_slice(&publish_root[256..342]);
    });
    flow
}

/// I-9.25（树表条目的 previous_snapshot_txg 取值）：发布那张树表里第一条不是 inode 树的条目，previous_snapshot_txg（条目偏移 100）写成 1、重封到根。
fn record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_tree(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-previous-snapshot-txg-on-a-non-inode-tree",
        &tree_table,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let entry = (0..count)
                .map(|entry| 131 + entry * 200)
                .find(|offset| {
                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) != 2
                })
                .expect("树表里有不是 inode 树的条目");
            bytes[entry + 100] = 1;
        },
    )
}

/// I-9.27（树表条目的头 ID 取值）：发布那张树表里 extent 树那条（种类 1）的头 ID（条目偏移 116）写成 0、重封到根——
/// 它该等于同一张表里 inode 树条目的树 ID。
fn record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-extent-tree-entry-head-identifier-zero",
        &tree_table,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let entry = (0..count)
                .map(|entry| 131 + entry * 200)
                .find(|offset| {
                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 1
                })
                .expect("树表里有 extent 树那条");
            assert_ne!(
                bytes[entry + 116..entry + 124],
                [0u8; 8],
                "extent 树那条的头 ID 本来不是 0"
            );
            bytes[entry + 116..entry + 124].fill(0);
        },
    )
}

/// I-9.27 种类 2 那一支：发布那张树表里 inode 树那条（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1、重封到根。
fn record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-tree-entry-head-identifier-off-its-own-tree-identifier",
        &tree_table,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, 2);
            let own_tree_identifier =
                u64::from_le_bytes(bytes[entry..entry + 8].try_into().expect("8"));
            assert_eq!(
                u64::from_le_bytes(bytes[entry + 116..entry + 124].try_into().expect("8")),
                own_tree_identifier,
                "inode 树那条的头 ID 本来等于它自己的树 ID"
            );
            bytes[entry + 116..entry + 124]
                .copy_from_slice(&(own_tree_identifier + 1).to_le_bytes());
        },
    )
}

/// I-9.27 写 0 那一支：发布那张树表里分配记录树那条（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID、重封到根。
fn record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-allocation-record-tree-entry-head-identifier-non-zero",
        &tree_table,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, 3);
            let inode_tree_entry = only_tree_table_entry_of_kind(bytes, 2);
            assert_eq!(
                bytes[entry + 116..entry + 124],
                [0u8; 8],
                "分配记录树那条的头 ID 本来是 0"
            );
            let inode_tree_identifier = bytes[inode_tree_entry..inode_tree_entry + 8].to_vec();
            bytes[entry + 116..entry + 124].copy_from_slice(&inode_tree_identifier);
        },
    )
}

/// 一张树表单元（码 2、条目宽 200）里种类是 `kind` 的那一条（恰好一条）在单元里的偏移。
pub(super) fn only_tree_table_entry_of_kind(bytes: &[u8], kind: u16) -> usize {
    let header_end = 86 + 2 * usize::from(bytes[51]);
    let count = usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ));
    let entries_of_the_kind: Vec<usize> = (0..count)
        .map(|entry| 131 + entry * 200)
        .filter(|offset| {
            u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == kind
        })
        .collect();
    let [entry] = entries_of_the_kind.as_slice() else {
        panic!("树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
    };
    *entry
}

/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：新池新建文件那一版唯一的数据单元（两份副本，单元区起始槽号 + 4）挪到再往后两槽
/// （下一个 32 KiB 对齐的槽对，这条流上空着）——两次单元写的偏移、流里每一处指着它的位置条目（extent 上段叶内联的数据指针、
/// 中央映射叶、journal 记录）的槽号一起改，单元重封到根、记录重封。树表与诞生 txg 一个字节不动，这一版照样被认出，
/// 每块盘上落点最小的数据单元在 U + 6、不是 U + 4。
fn record_new_pool_with_the_first_data_unit_two_slots_further(tag: &str) -> RecordedFlow {
    const SLOTS_MOVED: u64 = 2;
    let mut flow = record_new_pool_file_creation(tag, "new-pool-first-data-unit-two-slots-further");
    let data_unit_copies: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        data_unit_copies.len(),
        2,
        "新池新建文件只写一个数据单元，两块盘各一份"
    );
    // (旧的位置条目 14 字节, 新的槽号 6 字节)
    let renamed_locations: Vec<(Vec<u8>, [u8; 6])> = data_unit_copies
        .iter()
        .map(|index| {
            let write = &flow.writes[*index];
            let slot = write.offset.0 / 16384;
            let mut location = Vec::with_capacity(14);
            location.extend_from_slice(&write.device.0.to_le_bytes());
            location.extend_from_slice(&slot.to_le_bytes()[..6]);
            location.extend_from_slice(&crc32c(write.bytes().expect("单元写带字节")).to_le_bytes());
            let mut new_slot = [0u8; 6];
            new_slot.copy_from_slice(&(slot + SLOTS_MOVED).to_le_bytes()[..6]);
            (location, new_slot)
        })
        .collect();
    for index in &data_unit_copies {
        flow.writes[*index].offset.0 += SLOTS_MOVED * 16384;
    }
    let old_locations: Vec<Vec<u8>> = renamed_locations
        .iter()
        .map(|(location, _)| location.clone())
        .collect();
    let names_the_data_unit = |bytes: &[u8]| {
        old_locations.iter().any(|location| {
            bytes
                .windows(14)
                .any(|window| window == location.as_slice())
        })
    };
    let rename = move |bytes: &mut [u8]| {
        for (location, new_slot) in &renamed_locations {
            let mut position = 0;
            while position + 14 <= bytes.len() {
                if bytes[position..position + 14] == location[..] {
                    bytes[position + 4..position + 10].copy_from_slice(new_slot);
                    position += 14;
                } else {
                    position += 1;
                }
            }
        }
    };
    let unit_referrers: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write
                    .bytes()
                    .is_some_and(|bytes| unit_class(bytes) != 1 && names_the_data_unit(bytes))
        })
        .map(|(index, _)| index)
        .collect();
    let journal_referrers: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord && write.bytes().is_some_and(names_the_data_unit)
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !unit_referrers.is_empty(),
        "流里有单元指着那个数据单元（extent 上段叶、中央映射叶）"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &unit_referrers, &rename);
    for index in journal_referrers {
        rewrite_journal_record_and_reseal(&mut flow, index, &rename);
    }
    flow
}

/// I-3.13（记账行代段等于这一版的 txg）：发布那片记账叶第二条的代段（key 偏移 14）加 1、重封到根。取中间那条：
/// 首末两条的 key 是叶头里的区间，改了它们 I-1.1 先红。
fn record_new_pool_with_an_accounting_row_of_another_generation(tag: &str) -> RecordedFlow {
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-accounting-row-generation-plus-one",
        &accounting_leaf,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            assert!(count >= 3, "记账叶至少三条，改中间那条（有 {count} 条）");
            let generation = 159 + 34 + 14;
            bytes[generation] = bytes[generation].wrapping_add(1);
        },
    )
}

/// I-9.26（inode 改动计数落在出生代与容器诞生代号之间）：发布那片 inode 叶第一条记录的改动计数（偏移 88）最高字节写成 0x7F、重封到根——
/// 远大于容器头的诞生代号。
pub(super) fn record_new_pool_with_an_inode_change_count_past_its_container_birth(
    tag: &str,
) -> RecordedFlow {
    let inode_leaf = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 2;
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-change-count-past-the-container-birth",
        &inode_leaf,
        false,
        &|bytes| {
            bytes[136 + 88 + 7] = 0x7F;
        },
    )
}

/// I-7.16（系统配置槽世代号与槽号的关系）：最后一次系统配置槽写的世代号（偏移 147）加 2、重封——奇偶照旧，与同盘另一槽相差 3。
fn record_new_pool_with_system_configuration_generations_three_apart(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-generation-plus-two");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[147] = bytes[147].wrapping_add(2);
    });
    flow
}

/// I-7.17（根槽位置等于 txg 算出的位置）：发布的根原样再写一份到同一区域的下一个槽（流尾另起一段）——那个槽按 txg 算不该住它。
fn record_new_pool_with_a_root_copied_into_the_next_slot(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-publish-root-copied-into-the-next-slot");
    let publish_root = flow.writes[flow.judged_root_index].clone();
    let next_slot = singlefs_core::address::DeviceOffsetInBytes(
        publish_root.offset.0 + u64::from(geometry().fixed_structure_slot_spacing),
    );
    let next_slot_in_the_base = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        publish_root.device,
        next_slot,
        usize::try_from(publish_root.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到下一个根槽");
    assert!(
        next_slot_in_the_base.iter().all(|byte| *byte == 0),
        "下一个根槽在基镜像里空着（发布的根在区域里不是最后一槽）"
    );
    assert!(
        flow.writes
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root.device, next_slot)),
        "流里没有别的写落在下一个根槽上"
    );
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device: publish_root.device,
            kind: publish_root.kind,
            is_force_unit_access: publish_root.is_force_unit_access,
            offset: next_slot,
            contents: publish_root.contents.clone(),
        }],
    );
    flow
}

/// I-7.15（各盘所选系统配置槽设备数相同）：最后一次系统配置槽写的设备数（偏移 143）改成 3、重封——那块盘择到的这一槽与别的盘不同。
pub(super) fn record_new_pool_with_a_system_configuration_of_another_device_count(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-device-count-three");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[143..147].copy_from_slice(&3u32.to_le_bytes());
    });
    flow
}

/// I-4.1（重放后树可遍历）：发布那条记录新根段里树表指针两条位置条目的整单元校验和各翻一遍、重封——点名单元照样验得过，
/// 发布的根没落而记录落了的状态上重放施加它，施加出来的那一版树表两份都对不上校验和、走不下去。
pub(super) fn record_new_pool_with_a_replayed_version_whose_tree_table_cannot_be_read(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-replayed-tree-table-checksums-flipped");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        for location_checksum in [
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 60,
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 74,
        ] {
            for byte in &mut bytes[location_checksum..location_checksum + 4] {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// I-8.1（环几何够大）三个字段那一支：最后一次系统配置槽写的安全系数 F（偏移 357）改成 2、重封——整池不作保，只红 I-8.1。
pub(super) fn record_new_pool_with_a_safety_factor_of_two(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-safety-factor-2");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[357..361].copy_from_slice(&2u32.to_le_bytes());
    });
    flow
}

/// 两支整池不作保都中：最后一次系统配置槽写的槽距改成 512（读者不收）、F 改成 2，重封——走 I-7.13 那一支，只红 I-7.13、I-8.1 报不适用。
pub(super) fn record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-system-configuration-spacing-512-and-safety-factor-2",
    );
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[429..433].copy_from_slice(&512u32.to_le_bytes());
        bytes[357..361].copy_from_slice(&2u32.to_le_bytes());
    });
    flow
}

/// 系统配置槽里区域数 R 与每区槽数 S 那两字节的偏移（`.claude/kb/layout/01-first-txn.md` 一：F 357 + 4 → R 361，S 紧随其后）。
const SYSTEM_CONFIGURATION_REGIONS_OFFSET: usize = 361;

const SYSTEM_CONFIGURATION_SLOTS_PER_REGION_OFFSET: usize = 362;

/// I-7.13（系统配置池级字段在读者收的范围里）、C663（系统配置槽越界与不收之值两读者分类不同）：最后一次系统配置槽写的每区槽数 S 改成 17（区间 [4, 16] 之外）、重封——S 越界照 D22（单元原子性怎么合成） 已定项 2
/// 整池拒，CPU 只红 I-7.13；GPU 走读内核读这一槽同判带不收之值。
pub(super) fn record_new_pool_with_seventeen_slots_per_region(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-slots-per-region-17");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        assert_eq!(
            bytes[SYSTEM_CONFIGURATION_SLOTS_PER_REGION_OFFSET], 8,
            "mkfs 写下的每区槽数是 8"
        );
        bytes[SYSTEM_CONFIGURATION_SLOTS_PER_REGION_OFFSET] = 17;
    });
    flow
}

/// I-7.13（系统配置池级字段在读者收的范围里）、C663（系统配置槽越界与不收之值两读者分类不同）：最后一次系统配置槽写的区域数 R 改成 2（与第一版常量 3 不等）、重封——
/// R 读出不收之值整池拒，CPU 只红 I-7.13；GPU 走读内核读这一槽同判带不收之值。
pub(super) fn record_new_pool_with_two_regions(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-system-configuration-regions-2");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        assert_eq!(
            bytes[SYSTEM_CONFIGURATION_REGIONS_OFFSET], 3,
            "mkfs 写下的区域数是 3"
        );
        bytes[SYSTEM_CONFIGURATION_REGIONS_OFFSET] = 2;
    });
    flow
}

/// I-2.4 系统配置槽尾（C728）：最后一次系统配置槽写的末字节（偏移 4095，字段表之后那一段 [489, 4096) 的最后一个）写 1、重封整槽校验和——
/// core 不读这一段，两遍恢复照常；CPU 池级 checker 对每个可择的槽判 [489, 4096) 全 0，那一槽落了的状态上红 I-2.4。
/// 取末字节：GPU 走读内核那一段先逐字节读到字边界、再按字读，末字节落在按字读那一截里（根槽尾那条流的偏移 457 落在逐字节那一截里）。
pub(super) fn record_new_pool_with_a_non_zero_last_byte_in_a_system_configuration_slot(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-last-slot-byte-one");
    let last_byte_of_the_slot =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES - 1).expect("4095");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        assert_eq!(bytes[last_byte_of_the_slot], 0, "第一版字段表之后写 0");
        bytes[last_byte_of_the_slot] = 1;
    });
    flow
}

/// I-8.5（tail 槽有效集非空）：流尾另起一段，两块盘的两个系统配置槽各写一份全零——它们都落了的状态上一个自证过的槽都没有。
pub(super) fn record_new_pool_with_every_system_configuration_slot_zeroed_at_the_end(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-every-system-configuration-slot-zeroed");
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let zeroed_slots: Vec<RetainedWrite> = [0u32, 1]
        .into_iter()
        .flat_map(|device| {
            [0, spacing].into_iter().map(move |offset| RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(device),
                kind: StepKind::SystemConfigurationSlot,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(offset),
                contents: WrittenContents::Bytes(vec![0u8; 4096]),
            })
        })
        .collect();
    append_a_segment_of_writes(&mut flow, zeroed_slots);
    flow
}

/// I-7.5（根槽按判定宽度对齐）：最后一次系统配置槽写的 io_min（偏移 425）改成 12288、重封——判定宽度变成 12288，根环起点 1 MiB 不是它的整数倍。
pub(super) fn record_new_pool_with_a_minimum_input_output_the_root_slots_are_not_aligned_to(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-system-configuration-io-min-12288");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[425..429].copy_from_slice(&12288u32.to_le_bytes());
    });
    flow
}

/// I-3.12（实例表行 T 低于引用它的根的 txg）：回退流上最后一版实例表第一行的 T（行偏移 5，8 字节）最高字节加 1、重封到根——T 不小于引用它的根的 txg。
fn record_rollback_with_an_instance_table_row_not_below_its_root(tag: &str) -> RecordedFlow {
    let instance_table = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 4;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-instance-row-published-txg-past-its-root",
        &instance_table,
        &|bytes| {
            assert_eq!(bytes[136], 0, "实例表第一条是行（kind 0）");
            bytes[136 + 12] = bytes[136 + 12].wrapping_add(1);
        },
    )
}

/// I-8.12（同实例记录的 txg 不减）：回退流上最后一次 journal 记录写的 checkpoint_txg（偏移 26）改成 1、重封——同盘同实例更早的记录 txg 更大。
fn record_rollback_with_a_journal_record_txg_going_back(tag: &str) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-last-journal-record-txg-one";
    let last_record = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("流里有 journal 记录写");
    let (device, instance) = {
        let write = &flow.writes[last_record];
        let bytes = write.bytes().expect("记录写带字节");
        (
            write.device,
            u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
        )
    };
    assert!(
        flow.writes[..last_record].iter().any(|write| {
            write.kind == StepKind::JournalRecord
                && write.device == device
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[16..20].try_into().expect("4")) == instance
                        && u64::from_le_bytes(bytes[26..34].try_into().expect("8")) > 1
                })
        }),
        "同盘同实例有更早的、txg 大于 1 的记录"
    );
    rewrite_journal_record_and_reseal(&mut flow, last_record, &|bytes| {
        bytes[26..34].copy_from_slice(&1u64.to_le_bytes());
    });
    flow
}

// ───────────── I-3.5 的出现规则、恢复施加一条记录取新根段的七段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段 ─────────────

/// journal 记录新根段里实例表指针的段内偏移（D23（journal 的角色与格式） 已定项 15：树表指针 86 + 映射根指针 86 + 水位 8 + F 8）。
pub(super) const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: usize = 188;

/// 新根段里分配记录树根指针的段内偏移（实例表指针之后 86 字节）。
const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: usize = 274;

/// 新根段末尾卸载记号 4 字节的段内偏移（只有位 0，与根记录 flags 同一个编码）。
const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: usize = 360;

/// 码 2 / 码 3 指针里两条位置条目各自的整单元校验和的偏移（位置条目 50 与 64 起，盘 4 + 槽 6 之后）。
pub(super) const POINTER_LOCATION_CHECKSUM_OFFSETS: [usize; 2] = [60, 74];

/// 指针头里 extent 偏移 2 字节的偏移（MAC / nonce 28、算法类型 1、压缩码 1、压缩长度 2 之后）。
pub(super) const POINTER_EXTENT_OFFSET_OFFSET: usize = 32;

/// 指针头里算法类型 1 字节的偏移（MAC / nonce 28 之后）。
pub(super) const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;

/// 码 2 / 码 3 指针宽。
pub(super) const NODE_POINTER_BYTES: usize = 86;

/// 数据单元头里诞生代号的偏移（码 1 写序五元组：树 43、inode 51、对象出生代 59、偏移 67、诞生代号 75）。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;

/// I-3.5（引用区间的精确性）的出现规则：新池新建文件那条流上，发布那一版（txg 3）新写的数据单元头里的诞生代号 3 → 2、重封到根——
/// 发布那一版引用、暖机最后一版（txg 2）不引用的这个单元诞生代号不大于 2，它在 extent 树里，而 extent 树的根指针生在 txg 3、不是整棵继承下来的。
/// 指针、映射 key 与分配代还记 3，I-1.2、I-1.11、I-3.10 跟着红（与 `checker_known_bad_images.rs` 里 I-3.5 那一格同一个改法）。
pub(super) fn record_new_pool_with_a_data_unit_born_no_later_than_the_previous_version(
    tag: &str,
) -> RecordedFlow {
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-data-unit-birth-txg-three-to-two",
        &data_unit,
        false,
        &|bytes| {
            let birth_txg = u64::from_le_bytes(
                bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
                    .try_into()
                    .expect("8"),
            );
            assert_eq!(birth_txg, 3, "新建的文件那个数据单元生在发布那一版 txg 3");
            bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
                .copy_from_slice(&2u64.to_le_bytes());
        },
    )
}

/// 发布那几条 journal 记录新根段里的卸载记号写成 0b10（位 0 之外有位为 1）、重封：core 的读者当记录损坏（重放不施加它），
/// CPU 池级 checker 照同一口径不收它（RR1）。
pub(super) fn record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-unmount-marker-bit-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let marker = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_UNMOUNT_MARKER;
        assert_eq!(
            bytes[marker..marker + 4],
            [0u8; 4],
            "建文件那次发布不是卸载那一串，卸载记号是 0"
        );
        bytes[marker..marker + 4].copy_from_slice(&0b10u32.to_le_bytes());
    });
    flow
}

/// 发布那几条 journal 记录新根段里的卸载记号写成 1（只有位 0）、重封：记录照收、照施加，新根段与同代的根（flags 0）不逐字节相同，
/// CPU 池级 checker 红 I-8.10（第七段，卸载记号）。
pub(super) fn record_new_pool_with_a_last_record_carrying_the_unmount_marker(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-last-record-unmount-marker-set");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let marker = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_UNMOUNT_MARKER;
        bytes[marker..marker + 4].copy_from_slice(&1u32.to_le_bytes());
    });
    flow
}

/// 发布那几条 journal 记录新根段里实例表指针的两条位置条目校验和各取反、重封：记录照收、照施加，施加出来那一版的实例表指针
/// 与所选根（暖机最后那条根）的不同、两份都读不对。core 的恢复从记录取实例表（已定项 15）走到文件之前就读不出；
/// CPU 池级 checker 走施加出来的那一版时实例表读不出（I-4.1），根也落了的状态上新根段与同代的根不逐字节相同（I-8.10 第五段）。
pub(super) fn record_new_pool_with_a_last_record_whose_instance_table_pointer_reads_wrong(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-instance-table-checksums-flipped");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let pointer = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
        for location_checksum in POINTER_LOCATION_CHECKSUM_OFFSETS {
            for byte in &mut bytes[pointer + location_checksum..pointer + location_checksum + 4] {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// 发布那几条 journal 记录新根段里实例表指针头的 extent 偏移写成 1、重封：core 的读者当记录损坏（指针头不是第一版的取值，
/// `NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from`），重放不施加它；CPU 池级 checker 不判记录里的指针头、
/// 照收，根也落了的状态上红 I-8.10（第五段）。
pub(super) fn record_new_pool_with_a_last_record_whose_instance_table_pointer_carries_an_extent_offset(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-instance-table-extent-offset-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let extent_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER
            + POINTER_EXTENT_OFFSET_OFFSET;
        bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    });
    flow
}

/// 发布那几条 journal 记录新根段里分配记录树根指针（建文件那一版本来全零）换成外来的字节：同一条记录的实例表指针、
/// 两条位置条目校验和各取反，重封。施加出来那一版的分配记录树根与它施加在其上的那条根（全零）不同：CPU 池级 checker 走那一版时
/// 从记录取它、两份都读不对（I-4.1），根也落了的状态上新根段与同代的根不逐字节相同（I-8.10 第六段）。
pub(super) fn record_new_pool_with_a_last_record_whose_allocation_record_tree_root_is_foreign(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-allocation-root-foreign-bytes");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let instance_table =
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
        let allocation_root =
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
        assert_eq!(
            bytes[allocation_root..allocation_root + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "带文件的一版分配记录树根恒全零"
        );
        let foreign = bytes[instance_table..instance_table + NODE_POINTER_BYTES].to_vec();
        bytes[allocation_root..allocation_root + NODE_POINTER_BYTES].copy_from_slice(&foreign);
        for location_checksum in POINTER_LOCATION_CHECKSUM_OFFSETS {
            for byte in &mut bytes
                [allocation_root + location_checksum..allocation_root + location_checksum + 4]
            {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// I-1.12：发布的根记录里实例表指针的出生树与实例表那一片单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_tree() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_tree",
        &record_new_pool_with_an_instance_table_pointer_naming_another_birth_tree("crash-judge-gpu-i112"),
        "I-1.12",
    );
}

/// I-1.12 数据单元那一支：extent 树里内联的数据指针的出生树与数据单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_another_birth_tree_than_its_unit() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_another_birth_tree_than_its_unit",
        &record_new_pool_with_a_data_pointer_naming_another_birth_tree("crash-judge-gpu-i112-data"),
        "I-1.12",
    );
}

/// I-8.10：发布那条记录的新根段与同代的根不逐字节相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_of_a_publish_carries_another_watermark_than_its_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_last_record_of_a_publish_carries_another_watermark_than_its_root",
        &record_new_pool_with_a_last_record_carrying_another_watermark_than_its_root("crash-judge-gpu-i810"),
        "I-8.10",
    );
}

/// I-8.11：点名项的诞生代号与它点名的单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_named_entry_names_another_birth_txg_than_its_unit() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_named_entry_names_another_birth_txg_than_its_unit",
        &record_new_pool_with_a_named_entry_naming_another_birth_txg("crash-judge-gpu-i811"),
        "I-8.11",
    );
}

/// I-4.9：更早的候选根引用的单元晚于它诞生。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_older_candidate_root_references_units_born_after_it() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_older_candidate_root_references_units_born_after_it",
        &record_new_pool_with_an_older_root_pointing_at_the_trees_of_the_publish("crash-judge-gpu-i49"),
        "I-4.9",
    );
}

/// I-9.25：不是 inode 树的树表条目带了 previous_snapshot_txg。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inode_tree_carries_a_previous_snapshot_txg(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inode_tree_carries_a_previous_snapshot_txg",
        &record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_tree("crash-judge-gpu-i925"),
        "I-9.25",
    );
}

/// I-9.27：extent 树那条树表条目的头 ID 写成 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree",
        &record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero("crash-judge-gpu-i927"),
        "I-9.27",
    );
}

/// I-9.27 种类 2 那一支：inode 树那条树表条目的头 ID 不等于它自己的树 ID。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier",
        &record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier("crash-judge-gpu-i927-inode"),
        "I-9.27",
    );
}

/// I-9.27 写 0 那一支：分配记录树那条树表条目的头 ID 不是 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier",
        &record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry("crash-judge-gpu-i927-allocation"),
        "I-9.27",
    );
}

/// I-5.5：从 mkfs 水位建树的那一版，数据单元不落在单元区起始槽号 + 4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four",
        &record_new_pool_with_the_first_data_unit_two_slots_further("crash-judge-gpu-i55"),
        "I-5.5",
    );
}

/// I-3.13：记账行的代段不是这一版的 txg。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version",
        &record_new_pool_with_an_accounting_row_of_another_generation("crash-judge-gpu-i313"),
        "I-3.13",
    );
}

/// I-9.26：inode 记录的改动计数越过所在容器的诞生代号。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth",
        &record_new_pool_with_an_inode_change_count_past_its_container_birth("crash-judge-gpu-i926"),
        "I-9.26",
    );
}

/// I-3.12：实例表的一行的 T 不小于引用这张表的根的 txg（回退流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_instance_table_row_is_not_below_the_root_referencing_it(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_instance_table_row_is_not_below_the_root_referencing_it",
        &record_rollback_with_an_instance_table_row_not_below_its_root("crash-judge-gpu-i312"),
        "I-3.12",
    );
}

/// I-7.16：同一块盘两个系统配置槽的世代号相差 3。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_system_configuration_slots_are_three_generations_apart(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_system_configuration_slots_are_three_generations_apart",
        &record_new_pool_with_system_configuration_generations_three_apart("crash-judge-gpu-i716"),
        "I-7.16",
    );
}

/// I-7.17：一条根住在按它的 txg 算不该住的槽里。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_lives_in_another_slot_than_its_txg_names() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_lives_in_another_slot_than_its_txg_names",
        &record_new_pool_with_a_root_copied_into_the_next_slot("crash-judge-gpu-i717"),
        "I-7.17",
    );
}

/// I-8.12：同一块盘同一实例的 journal 记录 txg 往回走（回退流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_journal_record_txg_goes_back_within_an_instance() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_journal_record_txg_goes_back_within_an_instance",
        &record_rollback_with_a_journal_record_txg_going_back("crash-judge-gpu-i812"),
        "I-8.12",
    );
}
