//! 指针、日志与代号字段的坏镜像表，以及出生身份、引用区间与释放代号各条。

use super::*;

const SYSTEM_CONFIGURATION_DEVICE_COUNT_OFFSET: usize = 143;

/// 记账树根兼叶（50250）里条目区的起点与每条宽（`set_accounting` 同一个读法：115 + 2 × 22 起，每条 34）、代段在行里的偏移
/// （统计量 2 + 树 8 + 设备 4 之后）。
const ACCOUNTING_LEAF_ENTRY_START: usize = 159;

const ACCOUNTING_LEAF_ENTRY_BYTES: usize = 34;

const ACCOUNTING_ROW_GENERATION_OFFSET: usize = 14;

const INODE_RECORD_CHANGE_COUNT_OFFSET: usize = 88;

/// 写完新池新建文件那份镜像上，`parent` 单元里指着 `child` 的那条指针（两条位置条目依次是盘 0、盘 1 那一份）的 extent 偏移 0 → 1，
/// 重封、补到根。指针按两条位置条目的字节找，`parent` 里恰好一处。
fn set_the_extent_offset_of_the_pointer_from_parent_to_child(
    image: &mut MemoryPool,
    parent: u64,
    child: u64,
) {
    let child_length = UNITS
        .iter()
        .find(|(start, _)| *start == child)
        .expect("登记过的单元")
        .1;
    let locations: Vec<u8> = DEVICES
        .iter()
        .flat_map(|device| {
            let child_checksum =
                crc32_castagnoli(&read(image, *device, child * SLOT, child_length));
            location_pattern(*device, child, child_checksum)
        })
        .collect();
    assert_eq!(locations.len(), 2 * POINTER_LOCATION_BYTES);
    mutate_unit(
        image,
        parent,
        |bytes| {
            let found: Vec<usize> = bytes
                .windows(locations.len())
                .enumerate()
                .filter(|(_, window)| *window == locations.as_slice())
                .map(|(position, _)| position)
                .collect();
            assert_eq!(
                found.len(),
                1,
                "单元 {parent} 里指着 {child} 的指针恰好一条：{found:?}"
            );
            let field =
                found[0] - POINTER_FIRST_LOCATION_OFFSET + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
            assert_eq!(
                bytes[field..field + 2],
                [0, 0],
                "指着 {child} 的那条指针 extent 偏移本来是 0"
            );
            bytes[field] = 1;
        },
        true,
    );
}

/// 树表条目里 `previous_snapshot_txg` 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86
/// （D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86;

/// 新池新建文件那一版树表的条目数（十一条，见 `LIVELIST_TREE_TABLE_ENTRY_INDEX` 那一行）。
pub(super) const TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE: usize = 11;

/// 新池新建文件那一版树表里 inode 树那一条（种类 2）的下标：恰好一条。
fn inode_tree_entry_index_in(tree_table: &[u8]) -> usize {
    let indexes: Vec<usize> = (0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE)
        .filter(|index| {
            let kind_offset = tree_table_entry_offset(*index) + TREE_TABLE_ENTRY_KIND_OFFSET;
            u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]])
                == TREE_KIND_INODE
        })
        .collect();
    assert_eq!(indexes.len(), 1, "树表里 inode 树恰好一条：{indexes:?}");
    indexes[0]
}

/// inode 记录里对象出生代那 8 字节的偏移（D8（核心索引结构） 已定项 6：偏移 8）。
pub(super) const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;

/// 记录新根段（记录头 99 起，D23（journal 的角色与格式） 已定项 4）里 I-8.10 逐字节比的另外三段，各取一个改了不牵动别的判定的字段。
#[derive(Clone, Copy)]
enum NewRootSegmentField {
    /// 树表指针（新根段 [0..86)）第一条位置条目的整单元校验和（4 字节）。
    TreeTablePointerFirstChecksum,
    /// 中央映射树根指针（新根段 [86..172)）第一条位置条目的整单元校验和（4 字节）。
    MappingRootPointerFirstChecksum,
    /// 树 ID 水位（新根段 [172..180)，8 字节）。
    TreeIdentifierWatermark,
}

pub(super) const JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET: usize = 99;

const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER_OFFSET: usize = 86;

const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK_OFFSET: usize = 172;

/// A 那条记录（计数器 3，新池新建文件那次发布的末条）新根段里 `field` 加 1，两盘各一份、头校验和重封；A 的根不动。
fn add_one_to_a_new_root_segment_field_of_the_first_file_record(
    image: &mut MemoryPool,
    field: NewRootSegmentField,
) {
    let offset =
        singlefs_core::journal::record_offset(3, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
    for device in DEVICES {
        let mut record = read(image, device, offset, 4096);
        assert_eq!(
            get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
            3,
            "计数器 3 是 A 那条记录"
        );
        match field {
            NewRootSegmentField::TreeTablePointerFirstChecksum => {
                let checksum_offset = JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                    + POINTER_FIRST_LOCATION_OFFSET
                    + LOCATION_CHECKSUM_OFFSET;
                let checksum = get_u32(&record, checksum_offset);
                set_u32(&mut record, checksum_offset, checksum.wrapping_add(1));
            }
            NewRootSegmentField::MappingRootPointerFirstChecksum => {
                let checksum_offset = JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                    + NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER_OFFSET
                    + POINTER_FIRST_LOCATION_OFFSET
                    + LOCATION_CHECKSUM_OFFSET;
                let checksum = get_u32(&record, checksum_offset);
                set_u32(&mut record, checksum_offset, checksum.wrapping_add(1));
            }
            NewRootSegmentField::TreeIdentifierWatermark => {
                let watermark_offset = JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                    + NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK_OFFSET;
                let watermark = get_u64(&record, watermark_offset);
                set_u64(&mut record, watermark_offset, watermark + 1);
            }
        }
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(image, device, offset, &record);
    }
}

/// 二·1 那一批新立的几条里，坏镜像建在**写完新池新建文件**那份干净镜像上的：每份改一处、只红它自己那一条
/// （`research/prompts/m3-inv-b-r1-main-verification.md` 第二节、`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7、11 行）。
pub(super) fn known_bad_images_of_pointer_journal_and_generation_fields(
) -> Vec<(&'static str, Mutation)> {
    let mut images: Vec<(&'static str, Mutation)> = vec![
        // I-1.12：A 的 extent 树根里那条数据指针的出生树（指针偏移 34）改成另一棵发过号的树，重封、补到根：单元头里的树照旧是 extent 树。
        (
            POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                let data_pointer_birth_tree = EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET - 8;
                mutate_unit(
                    image,
                    EXTENT_ROOT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, data_pointer_birth_tree),
                            TREE_IDENTIFIER_EXTENT,
                            "数据指针的出生树本来是 extent 树"
                        );
                        set_u64(
                            bytes,
                            data_pointer_birth_tree,
                            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
                        );
                    },
                    true,
                );
            }),
        ),
        // I-8.10：A 那条记录（计数器 3，环里最后一条、带末条标志）的新根段里 F（偏移 99 + 180）0 → 1，两盘各一份、头校验和重封；
        // A 的根（txg 3）照旧带 F 0。
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                const NEW_ROOT_SEGMENT_ROLLBACK_FLOOR_OFFSET: usize = 99 + 180;
                let offset = singlefs_core::journal::record_offset(
                    3,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0;
                for device in DEVICES {
                    let mut record = read(image, device, offset, 4096);
                    assert_eq!(
                        get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
                        3,
                        "计数器 3 是 A 那条记录"
                    );
                    assert_eq!(
                        get_u64(&record, NEW_ROOT_SEGMENT_ROLLBACK_FLOOR_OFFSET),
                        0,
                        "A 那条记录的新根段 F 本来是 0"
                    );
                    set_u64(&mut record, NEW_ROOT_SEGMENT_ROLLBACK_FLOOR_OFFSET, 1);
                    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
                    record[46..78].copy_from_slice(&digest);
                    write(image, device, offset, &record);
                }
            }),
        ),
        // I-9.25：A 的树表（50252）第 0 条（extent 树，不是 inode 树）的 previous_snapshot_txg（条目偏移 100）0 → 3，重封、补到根。
        (
            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let field = tree_table_entry_offset(0)
                            + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET;
                        assert_eq!(
                            get_u64(bytes, field),
                            0,
                            "extent 树那条的 previous_snapshot_txg 本来是 0"
                        );
                        set_u64(bytes, field, 3);
                    },
                    true,
                );
            }),
        ),
        // I-3.13：A 的记账树（50250，根兼叶）第 1 条（不是首末条，头里的 key 区间不跟着变）的代段 3 → 2，重封、补到根。
        (
            ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| {
                        let count =
                            usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
                        assert!(count >= 3, "记账树至少三行，第 1 条不是首末条");
                        let generation = ACCOUNTING_LEAF_ENTRY_START
                            + ACCOUNTING_LEAF_ENTRY_BYTES
                            + ACCOUNTING_ROW_GENERATION_OFFSET;
                        assert_eq!(get_u64(bytes, generation), 3, "A 那一版记账行的代段是 3");
                        set_u64(bytes, generation, 2);
                    },
                    true,
                );
            }),
        ),
        // I-9.26：A 的 inode 叶容器（50242）第 0 条记录的改动计数 3 → 4（大于容器诞生代号 3），重封、补到根。
        (
            INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_LEAF,
                    |bytes| {
                        let field =
                            INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_CHANGE_COUNT_OFFSET;
                        assert_eq!(get_u64(bytes, field), 3, "改动计数本来是 3");
                        set_u64(bytes, field, 4);
                    },
                    true,
                );
            }),
        ),
        // I-9.25 inode 树那一半（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 2 条）：A 的树表（50252）里
        // inode 树那一条（种类 2）的 previous_snapshot_txg 0 → 4，比引用这张树表的根（txg 3）大，重封、补到根。
        (
            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let field = tree_table_entry_offset(inode_tree_entry_index_in(bytes))
                            + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET;
                        assert_eq!(
                            get_u64(bytes, field),
                            0,
                            "inode 树那条的 previous_snapshot_txg 本来是 0"
                        );
                        set_u64(bytes, field, 4);
                    },
                    true,
                );
            }),
        ),
        // I-9.26 下界（盲区第 3 条）：A 的 inode 叶容器（50242）第 0 条记录的改动计数改成对象出生代减 1（上界照旧成立），重封、补到根。
        (
            INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_LEAF,
                    |bytes| {
                        let object_birth = get_u64(
                            bytes,
                            INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_OBJECT_BIRTH_OFFSET,
                        );
                        assert!(object_birth >= 1, "对象出生代 {object_birth} 至少是 1");
                        set_u64(
                            bytes,
                            INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_CHANGE_COUNT_OFFSET,
                            object_birth - 1,
                        );
                    },
                    true,
                );
            }),
        ),
        // I-7.17 只错槽那一格（盲区第 6 条）：txg 1 那条根（区域 1 槽 0，盘 1）整槽原样拷到同一区域的槽 3（那一槽空着）：
        // 区域对、槽不对。
        (
            ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
            Box::new(|image: &mut MemoryPool| {
                let region_one_start = 64 * SLOT + 3 * (1 << 20);
                let root_of_txg_one = read(image, 1, region_one_start, 512);
                assert_eq!(
                    get_u64(&root_of_txg_one, 28),
                    1,
                    "区域 1 槽 0 坐着 txg 1 的根"
                );
                let target = region_one_start + 3 * 4096;
                assert_ne!(
                    &read(image, 1, target, 512)[..4],
                    b"SFSR",
                    "区域 1 槽 3 空着"
                );
                write(image, 1, target, &root_of_txg_one);
            }),
        ),
        // I-8.10 新根段另外三段各一份（盲区第 5 条）：A 那条记录（计数器 3）新根段里树表指针第一条位置条目的校验和、映射根指针第一条
        // 位置条目的校验和、树 ID 水位各加 1，两盘各一份、头校验和重封；A 的根照旧。
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                add_one_to_a_new_root_segment_field_of_the_first_file_record(
                    image,
                    NewRootSegmentField::TreeTablePointerFirstChecksum,
                );
            }),
        ),
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                add_one_to_a_new_root_segment_field_of_the_first_file_record(
                    image,
                    NewRootSegmentField::MappingRootPointerFirstChecksum,
                );
            }),
        ),
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                add_one_to_a_new_root_segment_field_of_the_first_file_record(
                    image,
                    NewRootSegmentField::TreeIdentifierWatermark,
                );
            }),
        ),
        // I-2.4 指针 extent 偏移在走读的另外三个入口上各一份（盲区第 1 条）：码 2 节点指针（树表里 extent 树那条的根指针，指着 50240）、
        // inode 内部条目的子指针（inode 树根 50244 里指着叶容器 50242 的那条）、extent 记录的数据指针（extent 树根里指着数据单元 50180 的那条），
        // extent 偏移 0 → 1，重封、补到根。
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                set_the_extent_offset_of_the_pointer_from_parent_to_child(
                    image,
                    TREE_TABLE,
                    EXTENT_ROOT,
                );
            }),
        ),
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                set_the_extent_offset_of_the_pointer_from_parent_to_child(
                    image, INODE_ROOT, INODE_LEAF,
                );
            }),
        ),
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                set_the_extent_offset_of_the_pointer_from_parent_to_child(
                    image,
                    EXTENT_ROOT,
                    DATA_UNIT,
                );
            }),
        ),
        // I-7.17：txg 1 那条根（区域 1 槽 0，盘 1）整槽原样拷到区域 2 槽 3（盘 0，那一槽空着）：按 txg 算它只该住区域 1 槽 0。
        (
            ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
            Box::new(|image: &mut MemoryPool| {
                let root_of_txg_one = read(image, 1, 64 * SLOT + 3 * (1 << 20), 512);
                assert_eq!(
                    get_u64(&root_of_txg_one, 28),
                    1,
                    "区域 1 槽 0 坐着 txg 1 的根"
                );
                let target = 64 * SLOT + 2 * 3 * (1 << 20) + 3 * 4096;
                assert_ne!(
                    &read(image, 0, target, 512)[..4],
                    b"SFSR",
                    "区域 2 槽 3 空着"
                );
                write(image, 0, target, &root_of_txg_one);
            }),
        ),
        // I-7.15：盘 1 两个系统配置槽的设备数 2 → 3，整槽校验和重封：盘 0 择到的说 2、盘 1 择到的说 3。
        (
            DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                for slot_offset in [0, system_configuration_slot_one_offset(image, 1)] {
                    mutate_system_configuration_slot(image, 1, slot_offset, |bytes| {
                        assert_eq!(
                            get_u32(bytes, SYSTEM_CONFIGURATION_DEVICE_COUNT_OFFSET),
                            2,
                            "两块盘的池"
                        );
                        set_u32(bytes, SYSTEM_CONFIGURATION_DEVICE_COUNT_OFFSET, 3);
                    });
                }
            }),
        ),
        // I-2.4 指针 extent 偏移那一格（`research/prompts/m3-inv-a-r1-opus-output.md` A6 的造法）：每条自证过的根里实例表指针的 extent 偏移 0 → 1，
        // 根槽自证校验和重封；实例表指针也在 I-8.10 比的新根段七段里，同 (实例, txg) 的记录跟着改、反向链接着重算
        // （`set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots`）。
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                let mut rewritten_roots = 0;
                for (device, offset) in root_slot_offsets_of_both_devices() {
                    let mut bytes = read(image, device, offset, 512);
                    if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
                        continue;
                    }
                    let field = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                        + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
                    assert_eq!(
                        bytes[field..field + 2],
                        [0, 0],
                        "实例表指针的 extent 偏移本来是 0"
                    );
                    bytes[field] = 1;
                    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                    bytes[138..170].copy_from_slice(&digest);
                    write(image, device, offset, &bytes);
                    rewritten_roots += 1;
                }
                assert!(rewritten_roots > 0, "根环里至少改了一条根");
                set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots(
                    image,
                );
            }),
        ),
        // I-9.27（判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1）：A 的树表（50252）第 0 条（extent 树，种类 1）的
        // 头 ID（条目偏移 116）从同一张表里 inode 树条目的树 ID 改成 0，重封、补到根。头 ID 别的判定都不读，只红 I-9.27。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let entry = tree_table_entry_offset(0);
                        assert_eq!(
                            u16::from_le_bytes([
                                bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                                bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1]
                            ]),
                            TREE_KIND_EXTENT,
                            "A 的树表第 0 条是 extent 树"
                        );
                        let inode_tree_entry =
                            tree_table_entry_offset(inode_tree_entry_index_in(bytes));
                        let field = entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
                        assert_eq!(
                            get_u64(bytes, field),
                            get_u64(bytes, inode_tree_entry),
                            "extent 树那条的头 ID 本来等于 inode 树条目的树 ID"
                        );
                        set_u64(bytes, field, 0);
                    },
                    true,
                );
            }),
        ),
        // I-9.27 种类 2 那一支（判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K1 证据缺口「种类 2 自等」）：
        // A 的树表里 inode 树条目（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1，重封、补到根。种类 1、9、10 比的是 inode 树条目的树 ID、
        // 不读它的头 ID，只红 I-9.27。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_INODE,
                    |own_tree_identifier, _inode_tree_identifier| own_tree_identifier,
                    |own_tree_identifier, _inode_tree_identifier| own_tree_identifier + 1,
                );
            }),
        ),
        // I-9.27 写 0 那一支（同一节证据缺口「写 0 的七种」）：A 的树表里分配记录树条目（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_ALLOCATION,
                    |_own_tree_identifier, _inode_tree_identifier| 0,
                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                );
            }),
        ),
        // I-9.27 种类 9 那一格（同一节证据缺口「种类 9 / 10」）：A 的树表里 `deleted_inodes` 条目的头 ID 从 inode 树条目的树 ID 改成 0。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_DELETED_INODES,
                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                    |_own_tree_identifier, _inode_tree_identifier| 0,
                );
            }),
        ),
        // I-9.27 种类 10 那一格：A 的树表里墓碑树条目的头 ID 从 inode 树条目的树 ID 改成 0。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_TOMBSTONE,
                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                    |_own_tree_identifier, _inode_tree_identifier| 0,
                );
            }),
        ),
    ];
    // I-9.27 写 0 那一支的其余六种（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K3-3）：照种类 3 那张，
    // A 的树表里那一种条目的头 ID 从 0 改成 inode 树条目的树 ID，每种一张，接在上面那批后面。
    images.extend(
        TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE.map(
            |kind| -> (&'static str, Mutation) {
                (
                    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
                    Box::new(move |image: &mut MemoryPool| {
                        rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                            image,
                            kind,
                            |_own_tree_identifier, _inode_tree_identifier| 0,
                            |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                        );
                    }),
                )
            },
        ),
    );
    images
}

/// I-9.27 头 ID 写 0 的七种（种类 3、4、6、7、8、11、12）里种类 3 之外的六种；A 的树表里每种恰好一条。
const TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE: [u16; 6] = [
    TREE_KIND_ACCOUNTING,
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_LINEAGE,
    TREE_KIND_LOGGED_OPERATIONS,
];

/// A 的树表（50252）里种类是 `kind` 的那一条（恰好一条）：头 ID（条目偏移 116）先核等于 `head_identifier_before(那条的树 ID, inode 树条目的树 ID)`，
/// 再改成 `head_identifier_after(…)`，重封、补到根。别的字段一个都不动。
fn rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    head_identifier_before: impl Fn(u64, u64) -> u64,
    head_identifier_after: impl Fn(u64, u64) -> u64,
) {
    mutate_unit(
        image,
        TREE_TABLE,
        |bytes| {
            let entries_of_the_kind: Vec<usize> = (0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE)
                .map(tree_table_entry_offset)
                .filter(|entry| {
                    u16::from_le_bytes([
                        bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                        bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
                    ]) == kind
                })
                .collect();
            let [entry] = entries_of_the_kind.as_slice() else {
                panic!("A 的树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
            };
            let own_tree_identifier = get_u64(bytes, *entry);
            let inode_tree_identifier = get_u64(
                bytes,
                tree_table_entry_offset(inode_tree_entry_index_in(bytes)),
            );
            let field = entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
            assert_eq!(
                get_u64(bytes, field),
                head_identifier_before(own_tree_identifier, inode_tree_identifier),
                "种类 {kind} 那条的头 ID 本来的取值"
            );
            set_u64(
                bytes,
                field,
                head_identifier_after(own_tree_identifier, inode_tree_identifier),
            );
        },
        true,
    );
}

/// 记录新根段里实例表指针的起点：新根段 99 起，树表指针 86、映射根指针 86、树 ID 水位 8、F 8 之后（D23（journal 的角色与格式） 已定项 15）。
const JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 99 + 188;

/// 环里从计数器 1 起读到第一格没有 magic 的那条为止：同 (实例, txg) 的根读得出（根槽 magic 在、实例表指针 extent 偏移已改成 1）的记录，
/// 新根段里实例表指针的 extent 偏移也 0 → 1；每一条的反向链按前一条（改后的）记录头重算，头校验和重封，两盘各一份。
/// 记录不经 `JournalRecord::parse` 改：实现的读者拒 extent 偏移非 0 的指针，解不出这样的记录。
/// 这份镜像上只有实例 1 写过记录（反向链在实例内逐条接），迭代上界是 [`RECORDS_READ_BACK_AT_MOST`]。
fn set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots(
    image: &mut MemoryPool,
) {
    let roots_with_the_extent_offset_set: std::collections::BTreeSet<(u32, u64)> =
        root_slot_offsets_of_both_devices()
            .into_iter()
            .map(|(device, offset)| read(image, device, offset, 512))
            .filter(|bytes| bytes[..4] == singlefs_core::root_record::ROOT_MAGIC)
            .map(|bytes| (get_u32(&bytes, 24), get_u64(&bytes, 28)))
            .collect();
    for device in DEVICES {
        let mut previous_record: Option<Vec<u8>> = None;
        for counter in 1..=RECORDS_READ_BACK_AT_MOST {
            let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
            let mut bytes = read(image, device, offset, 4096);
            if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
                break;
            }
            let identity = (
                get_u32(&bytes, JOURNAL_RECORD_INSTANCE_OFFSET),
                get_u64(&bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
            );
            if roots_with_the_extent_offset_set.contains(&identity) {
                let field = JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
                    + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
                bytes[field] = 1;
            }
            if let Some(previous) = &previous_record {
                set_u32(
                    &mut bytes,
                    JOURNAL_RECORD_BACK_CHAIN_OFFSET,
                    back_chain_of(previous),
                );
            }
            let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
            bytes[46..78].copy_from_slice(&digest);
            write(image, device, offset, &bytes);
            previous_record = Some(bytes);
        }
    }
}

/// 上面那批坏镜像各自**只**红在自己那一条上；干净镜像上那几条要真被评估过且成立（不是「不适用」）。
#[test]
fn each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant() {
    let clean = build_pool("known-bad-pointer-journal-and-generation").memory_pool();
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    for (invariant, _) in known_bad_images_of_pointer_journal_and_generation_fields() {
        assert_eq!(
            verdict(&clean, invariant),
            InvariantVerdict::Holds,
            "干净镜像上 {invariant} 要真被评估过且成立：{verdicts_on_the_clean_image:?}"
        );
    }
    // 逐张判完再断言：哪几张（下标从 0 数，按清单次序）判红的不是只有它自己那一条，一次全列出来。
    let images_not_reddening_only_their_own_invariant: Vec<String> =
        known_bad_images_of_pointer_journal_and_generation_fields()
            .into_iter()
            .enumerate()
            .filter_map(|(image_index, (invariant, mutation))| {
                let mut image = clean.clone();
                mutation(&mut image);
                let verdicts = check_pool_image(&image);
                let violated = violated_invariants(&verdicts);
                (violated != [invariant]).then(|| {
                    format!(
                        "第 {image_index} 张坏镜像判红的该只有 {invariant}，实际 {violated:?}：{verdicts:?}"
                    )
                })
            })
            .collect();
    assert_eq!(
        images_not_reddening_only_their_own_invariant,
        Vec::<String>::new(),
        "每张坏镜像判红的只有它自己那一条"
    );
}

/// 根记录里实例表指针（170 起）头部出生 txg 的偏移（指针偏移 42）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_BIRTH_TXG_OFFSET: usize =
    ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + 42;

/// I-1.2（块头写序已发布） 状态收窄（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）：实例表单元按例外只免谓词那一半，
/// 指着它的指针与它头里的出生身份仍要相等。新池新建文件那条根（txg 3）的实例表指针出生 txg 加 1、重封根槽：实例表本身一个字节不动 ⇒
/// **只**红 I-1.2。
#[test]
fn an_instance_table_pointer_whose_birth_txg_differs_from_the_instance_table_header_reddens_only_the_birth_identity_invariant(
) {
    let clean = build_pool("known-bad-instance-table-pointer-birth").memory_pool();
    assert_eq!(
        verdict(&clean, "I-1.2"),
        InvariantVerdict::Holds,
        "干净镜像上 I-1.2 真被评估过且成立"
    );
    let mut image = clean.clone();
    let (device, offset) = NEW_POOL_FILE_CREATION_ROOT_SLOT;
    let mut root = read(&image, device, offset, 512);
    assert_eq!(get_u64(&root, 28), 3, "这一槽是新池新建文件那条根（txg 3）");
    let birth_txg = get_u64(&root, ROOT_RECORD_INSTANCE_TABLE_POINTER_BIRTH_TXG_OFFSET);
    set_u64(
        &mut root,
        ROOT_RECORD_INSTANCE_TABLE_POINTER_BIRTH_TXG_OFFSET,
        birth_txg + 1,
    );
    let digest = wide_checksum_with_field_zeroed(&root, 512, 138);
    root[138..170].copy_from_slice(&digest);
    write(&mut image, device, offset, &root);
    // 记录新根段里那份实例表指针跟着改（I-8.10 七段里有它）：盘上留下的是「写者从一开始就写出了这样的根」的样子。
    copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.2"],
        "判红的该只有 I-1.2：{verdicts:?}"
    );
}

/// 发布 B（txg 4）写出的 extent 树根与中央映射根（落点由 `overwrite_in_one_instance.rs` 的验收钉住，`UNITS_AFTER_OVERWRITE` 里 B 写的那几槽）。
const EXTENT_ROOT_AFTER_OVERWRITE: u64 = 50253;

const MAPPING_ROOT_AFTER_OVERWRITE: u64 = 50263;

/// 发布 B（txg 4）写的数据单元（`DATA_UNIT_AFTER_OVERWRITE`，诞生代号 4）诞生代号往小改成前一版 A 的 txg 3，引用它的三处一起改成 3
/// （B 的 extent 树根里的数据指针、B 的中央映射里那条码 1 key、B 的分配记录树两片叶里那两条未释放记录的分配代），各自重封、沿引用链补到 B 的根：
/// 指针、映射、分配代与单元头都对得上，按已发布谓词也是已发布的（3 ≤ 挂载根 txg 4）。不取新池新建文件那一版（txg 3）：那一版是从 mkfs
/// 水位第一次建树的那一版，I-5.5（从 mkfs 水位建树那一版的 t1 落点） 按分配代认这一版新写的落点，分配代往小改它跟着红。
fn lower_the_birth_of_the_data_unit_of_the_overwrite_to_the_txg_of_the_file_creation_publish_everywhere_it_is_named(
    image: &mut MemoryPool,
) {
    mutate_unit_of(
        image,
        &UNITS_AFTER_OVERWRITE,
        DATA_UNIT_AFTER_OVERWRITE,
        |bytes| {
            assert_eq!(
                get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                4,
                "B 的数据单元诞生代号本来是 4"
            );
            set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 3);
        },
        true,
    );
    mutate_unit_of(
        image,
        &UNITS_AFTER_OVERWRITE,
        EXTENT_ROOT_AFTER_OVERWRITE,
        |bytes| {
            assert_eq!(
                get_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET),
                4,
                "B 的 extent 记录里那条数据指针的诞生代号本来是 4"
            );
            set_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, 3);
        },
        true,
    );
    mutate_unit_of(
        image,
        &UNITS_AFTER_OVERWRITE,
        MAPPING_ROOT_AFTER_OVERWRITE,
        |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 4, 3),
        true,
    );
    assert_eq!(
        mutate_allocation_leaves(
            image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| rewrite_unreleased_allocation_generation(
                bytes,
                DATA_UNIT_AFTER_OVERWRITE,
                4,
                3
            )
        ),
        2,
        "B 的数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 4"
    );
}

/// C395（代字段只有验收断言盯着） 第 10 项「单元头与点名项的诞生代号」往小那一格：I-3.5（引用区间的精确性） (a)「后一版引用、前一版不引用的单元
/// 诞生代号严格大于前一版的 txg」只红它自己的坏镜像。发布 B（txg 4）新写的数据单元诞生代号连同指针、映射 key、分配代一起改成前一版 A 的 txg 3
/// （[`lower_the_birth_of_the_data_unit_of_the_overwrite_to_the_txg_of_the_file_creation_publish_everywhere_it_is_named`]）：**只**红 I-3.5，说明点名那个数据单元；
/// 别的每一条与干净镜像上的判定逐条相同。只改单元头里那一处（`checker_judges_reference_intervals_and_every_new_root_segment_field.rs` 那一份）
/// 时 I-1.2、I-1.11、I-3.10、I-3.15 跟着红，判别力算不到 I-3.5 头上。
#[test]
fn a_new_data_unit_born_in_the_previous_version_everywhere_it_is_named_reddens_only_the_reference_interval_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-data-unit-birth-lowered-everywhere");
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "干净镜像上 I-3.5 真被评估过且成立"
    );
    let mut image = clean.clone();
    lower_the_birth_of_the_data_unit_of_the_overwrite_to_the_txg_of_the_file_creation_publish_everywhere_it_is_named(
        &mut image,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![REFERENCE_INTERVALS_ARE_EXACT],
        "只红 I-3.5：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, REFERENCE_INTERVALS_ARE_EXACT) else {
        panic!("上面判过 I-3.5 红");
    };
    assert!(
        detail.contains(&format!("槽 {DATA_UNIT_AFTER_OVERWRITE}"))
            && detail.contains("诞生代号 3 不大于前一版的 txg"),
        "违例点名 B 的数据单元、诞生代号 3 不大于前一版的 txg：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &clean_verdicts, &[REFERENCE_INTERVALS_ARE_EXACT]),
        Vec::new(),
        "除 I-3.5 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// 树表里 extent 树那条条目根指针（条目偏移 14 起 86 字节）里出生 txg 那 8 字节（指针偏移 42）的偏移。
const TREE_TABLE_ENTRY_ROOT_POINTER_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 42;

/// extent 树上段叶第一条条目内联的那条数据指针里写序实例代号那 4 字节的偏移（指针偏移 78，D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
const EXTENT_RECORD_DATA_POINTER_INSTANCE_OFFSET: usize = 163 + 24 + 1 + 78;

/// C395（代字段只有验收断言盯着） 第 20 项「父指针缓存的出生 txg、写序与子单元头是否一致」码 2 那一格：树表里 extent 树根指针的出生 txg
/// 3 → 2，extent 树根（码 2）一个字节不动，树表重封、沿引用链补到根：指针一侧只错出生 txg ⇒ **只**红 I-1.2（块头写序已发布），
/// 说明点名 extent 树根与指针记的诞生代号 2；别的每一条与干净镜像上的判定逐条相同。
#[test]
fn an_extent_root_pointer_whose_birth_txg_alone_differs_from_the_node_header_reddens_only_the_birth_identity_invariant(
) {
    let clean = build_pool("known-bad-extent-root-pointer-birth-txg-alone").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-1.2"),
        InvariantVerdict::Holds,
        "干净镜像上 I-1.2 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            let birth_txg =
                tree_table_entry_offset(0) + TREE_TABLE_ENTRY_ROOT_POINTER_BIRTH_TXG_OFFSET;
            assert_eq!(
                get_u64(bytes, birth_txg),
                3,
                "extent 树根指针的出生 txg 本来是 3"
            );
            set_u64(bytes, birth_txg, 2);
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec!["I-1.2"],
        "只红 I-1.2：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-1.2") else {
        panic!("上面判过 I-1.2 红");
    };
    assert!(
        detail.contains("诞生代号 3") && detail.contains("与引用它的指针记的（诞生代号 2、"),
        "违例点名头里的诞生代号 3 与指针记的 2：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &clean_verdicts, &["I-1.2"]),
        Vec::new(),
        "除 I-1.2 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// C395（代字段只有验收断言盯着） 第 20 项码 1 那一格：extent 树根里那条数据指针的写序实例代号 1 → 2，数据单元一个字节不动，
/// extent 树根重封、沿引用链补到根：指针一侧只错写序实例代号 ⇒ 红 I-1.2（块头写序已发布），说明点名头里的写序实例 1 与指针记的实例 2；
/// 同时红 I-1.13（扫描重建与遍历方向的索引相同）：「每个 inode 的数据单元集合」那张表扫描重建一侧照单元头记、遍历方向照指针记，码 1 指针
/// 只错一段也对不上——码 1 那一格分得开 I-1.2 与 I-1.13 的镜像不存在，只红 I-1.2 的那一份在码 2 那一格（上一条用例）。别的每一条与干净镜像上的判定逐条相同。
#[test]
fn a_data_pointer_whose_write_order_instance_alone_differs_from_the_data_unit_header_reddens_the_birth_identity_and_scan_rebuild_invariants(
) {
    let clean = build_pool("known-bad-data-pointer-write-order-instance-alone").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        EXTENT_ROOT,
        |bytes| {
            let instance = EXTENT_RECORD_DATA_POINTER_INSTANCE_OFFSET;
            assert_eq!(
                u32::from_le_bytes(bytes[instance..instance + 4].try_into().expect("4 字节")),
                1,
                "数据指针的写序实例代号本来是 1"
            );
            bytes[instance..instance + 4].copy_from_slice(&2u32.to_le_bytes());
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec!["I-1.2", SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX],
        "红的恰是 I-1.2 与 I-1.13：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-1.2") else {
        panic!("上面判过 I-1.2 红");
    };
    assert!(
        detail.contains("写序实例 1") && detail.contains("实例 2、"),
        "违例点名头里的写序实例 1 与指针记的实例 2：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &["I-1.2", SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX]
        ),
        Vec::new(),
        "除 I-1.2 与 I-1.13 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// 翻一个单元两份各一个载荷字节，不重封、不补引用链：位置条目校验和对不上，只读不判那一遍读不出它。
pub(super) fn break_both_copies_of_the_unit(image: &mut MemoryPool, slot: u64, length: usize) {
    for device in DEVICES {
        let mut bytes = read(image, device, slot * SLOT, length);
        bytes[length - 100] ^= 0xff;
        write(image, device, slot * SLOT, &bytes);
    }
}

/// I-3.9（释放代落在停止引用它的那一格区间里） 守卫收窄的正例（攻方 `research/prompts/m3-inv-b-r1-opus-model/proto.diff` 的组合坏镜像）：
/// 发布 B 之后那份镜像上 B 的释放代 4 改成 3，再把 A 那棵账的一片叶两份都翻坏——走不完的是 A（txg 3），它不比还引用那几个落点的最后一条根
/// （L = A 自己）晚，那几条记录照判 ⇒ I-3.9 判红。整条按「任一候选根走不完就不适用」的守卫在这里放过释放代。
#[test]
fn a_wrong_release_generation_is_still_reddened_when_only_a_root_not_later_than_the_last_one_referencing_it_is_incomplete(
) {
    let mut image =
        image_after_the_overwrite("release-generation-behind-an-earlier-incomplete-root");
    assert_eq!(
        mutate_allocation_leaves(
            &mut image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| rewrite_released_generations(bytes, 4, 3),
        ),
        24,
        "B 的账里释放代 4 的记录两片叶合起来 24 条"
    );
    break_both_copies_of_the_unit(&mut image, ALLOCATION_LEAVES[0], 16384);
    match verdict(&image, "I-3.9") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("释放代 3"),
            "红在释放代 3 那几条上：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("A 走不完而它不比 L 晚：I-3.9 照判、该红，实际 {other:?}")
        }
    }
}

/// I-3.9 守卫收窄的对照格（攻方造出来的那一份）：步 5 那段历史抬 F 到 11（空发布 txg 15、16 不换用户可见树），再覆盖写一次（txg 17）
/// 释放 txg 14 那一版的用户可见单元（释放代 17，真 L = 16）；把 txg 16 那条根自己的树表两份都翻坏，那几个落点藏在读不出的树表下面，
/// 算出来的 L = 15、T = 16。比 L 晚的 txg 16 走不完 ⇒ 那几条跳过，I-3.9 报部分没判、点出跳过的条数（C701（别的判法静默跳过读不出的部分）
/// 之前报不适用）、不红；不设防的判法在这里误红。
#[test]
fn release_generations_hidden_behind_a_later_incomplete_root_are_skipped_and_the_release_generation_invariant_is_partially_judged(
) {
    let mut pool =
        pool_before_raising_the_floor("release-generation-behind-a-later-incomplete-root");
    raise_the_floor(&mut pool, CheckpointTxg(11), CheckpointTxg(0));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(16),
        "抬 F 那串空发布最后一次是 txg 16"
    );
    let tree_table_of_the_last_raise = pool.output.unit(TransactionUnit::TreeTable).slot.0;
    overwrite_and_advance(&mut pool, &content_seeded(1200, 3), InstanceGeneration(2));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(17),
        "抬 F 之后那次覆盖写是 txg 17"
    );
    let mut image = pool.memory_pool();
    assert!(
        !matches!(verdict(&image, "I-3.9"), InvariantVerdict::Violated(_)),
        "翻坏之前 I-3.9 不红"
    );
    break_both_copies_of_the_unit(&mut image, tree_table_of_the_last_raise, 16384);
    match verdict(&image, "I-3.9") {
        InvariantVerdict::PartiallyJudged(detail) => assert!(
            detail.contains("走不完") && detail.contains("128 条"),
            "部分没判的说明要说比 L 晚的候选根走不完、点出跳过的 128 条：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("落点藏在比 L 晚、走不完的根后面：I-3.9 该报部分没判，实际 {other:?}")
        }
    }
}

/// 同步助手 `common::copy_each_root_into_the_new_root_segment_of_its_journal_records` 里「前一条被改过、又与这一条同实例，
/// 这一条的反向链按前一条改后的记录头重算」那一支（I-8.6（反向链算法）只链本实例内逻辑前一条）。发布 B 之后的干净镜像上实例 1 有
/// A 的记录（计数器 3，txg 3）与 B 的记录（计数器 4，txg 4）；改的是较旧那条根——A 的根（txg 3）树 ID 水位压到 mkfs 时的水位、重封自证校验和。
/// 同步之后计数器 3 两盘各一份的新根段跟着压（I-8.10（记录新根段与同代根相同）），计数器 4 两盘各一份的新根段不动、反向链按改后的计数器 3 重算。
/// B 的根是最新那条、水位照旧盖得住全池出现过的树 ID（I-7.8 取根环全部根的 max，另判最新那条根），所以判决与干净镜像逐项相同；
/// 那一支不重算，计数器 4 的反向链接不上改后的计数器 3，红 I-8.6。
#[test]
fn rewriting_an_older_root_rechains_the_later_record_of_its_instance_and_every_verdict_stays_as_on_the_clean_image(
) {
    let clean = image_after_the_overwrite("older-root-rewritten-later-record-rechained");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    let mut image = clean.clone();
    let (device, offset) = root_slot_of(CheckpointTxg(3));
    let mut bytes = read(&image, device, offset, 512);
    assert_eq!(get_u64(&bytes, 28), 3, "那一槽是 A 的根（txg 3）");
    assert!(
        get_u64(&bytes, ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET)
            > singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        "A 发过树，它的水位高于 mkfs 时的水位：压下去才是真改了"
    );
    set_u64(
        &mut bytes,
        ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET,
        singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    );
    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(&mut image, device, offset, &bytes);
    let records_rewritten =
        copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "改了较旧那条根、同步之后一条都不红（不重算后一条的反向链就红 I-8.6）：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &[]),
        [],
        "每一条的判定与干净镜像逐项相同"
    );
    assert_eq!(
        records_rewritten, 4,
        "计数器 3 两盘各一份改新根段，计数器 4 两盘各一份重算反向链"
    );
}
