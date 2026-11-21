//! 几个子模块的测试共用的合成输入：内存里的盘与单元、十二槽的根环、树表与快照世界、journal 记录与读判定的小函数。

use super::*;

/// 环长 12 的几何（R = 3、S = 4）：钉槽与根环那几条用例用。
pub(super) fn geometry_of_a_ring_of_twelve() -> PoolGeometry {
    PoolGeometry {
        filesystem_identifier: [0u8; 16],
        physical_block_size: 512,
        journal_ring_start_slot: 1024,
        journal_ring_bytes: 1 << 20,
        declared_in_flight_record_limit: 85,
        declared_worst_case_occupancy_bytes: 85 * 4096,
        declared_safety_factor: 3,
        declared_minimum_input_output_bytes: 512,
        regions: 3,
        slots_per_region: 4,
        prime_step: 3,
        chunk_bytes: 1 << 20,
        base_slot: 64,
        region_devices: [0, 1, 0],
        unit_area_start_slot: 1088,
        slot_spacing: 4096,
    }
}

/// 根环里一条自证过的根（I-7.18、I-7.19 那一判只读实例、txg、F、t、p；记录字节这几条用例不读）。
pub(super) fn ring_root_of(
    instance: u32,
    checkpoint_txg: u64,
    rollback_floor: u64,
    reclaim_threshold: u64,
    pinned_ring_slot: u8,
) -> crate::RootView {
    crate::RootView {
        instance,
        checkpoint_txg,
        tree_identifier_watermark: 0,
        rollback_floor,
        unmount_marker: crate::UnmountMarkerView::NotWrittenByTheUnmountSequence,
        reclaim_threshold,
        pinned_ring_slot,
        record_bytes: Vec::new(),
        first_non_zero_byte_past_the_record: None,
    }
}

/// C717（回退下界上限的单窗口盲带）那一对历史的用户可见状态（两腿模型 `research/prompts/m3-close-b19-checker-r1-opus-model/k8/k8_scripted_pair.py`
/// 的 A–E）：每个状态一个标记字节，inode 树与 extent 树的根指针都写成这个字节。
pub(super) fn user_visible_state_marked(marker: u8) -> UserVisibleTreeRootPointers {
    UserVisibleTreeRootPointers {
        inode_tree: Some(vec![marker; 86]),
        extent_tree: Some(vec![marker; 86]),
        snapshot_tree_roots: Vec::new(),
    }
}

/// 一张码 2 树表节点：两条条目，树 ID 1 是 inode 树、树 ID 2 是 extent 树，两条根指针都写成 `marker` 这个字节；头校验和与载荷校验和都封好。
/// 偏移照 `crate::index_node_view`：key 宽 8、条目数在 98、条目宽在 100、明文头到 102、29 字节预留之后从 131 起是条目。
pub(super) fn tree_table_node_with_both_user_visible_tree_roots_marked(marker: u8) -> Vec<u8> {
    let key_width = 8usize;
    let header_end = 86 + 2 * key_width;
    let entries_start = header_end + 29;
    let entry_width = tree_table_entry_bytes();
    let mut unit = vec![0u8; node_bytes()];
    unit[..4].copy_from_slice(b"SFSU");
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&UNIT_FORMAT_VERSION_THIS_CHECKER_READS.to_le_bytes());
    unit[6] = 2;
    unit[51] = u8::try_from(key_width).expect("8");
    unit[8..10].copy_from_slice(
        &u16::try_from(2 * entry_width)
            .expect("两条条目")
            .to_le_bytes(),
    );
    unit[header_end - 4..header_end - 2].copy_from_slice(&2u16.to_le_bytes());
    unit[header_end - 2..header_end]
        .copy_from_slice(&u16::try_from(entry_width).expect("条目宽").to_le_bytes());
    for (index, (tree_identifier, kind)) in [(1u64, TREE_KIND_INODE), (2u64, TREE_KIND_EXTENT)]
        .into_iter()
        .enumerate()
    {
        let entry_start = entries_start + index * entry_width;
        unit[entry_start..entry_start + 8].copy_from_slice(&tree_identifier.to_le_bytes());
        unit[entry_start + 10..entry_start + 12].copy_from_slice(&kind.to_le_bytes());
        unit[entry_start + 14..entry_start + 100].fill(marker);
    }
    let payload_checksum = crc32_castagnoli_table(&unit[header_end..]);
    unit[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = crc32_castagnoli_table(&unit[..header_end]);
    unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
    let view = index_node_view(&unit).expect("手造的树表节点解得开");
    assert_eq!(
        user_visible_tree_root_pointers_of_the_tree_table_entries(&view.entries),
        Some(user_visible_state_marked(marker)),
        "手造的树表读出来是这个状态"
    );
    unit
}

/// 一条只带键、反向链与链值的扫描记录（别的字段这几条用例不读）。
pub(super) fn scanned_record_of(
    instance: u32,
    back_chain: u32,
    chain_value_the_next_record_must_carry: u32,
) -> ScannedJournalRecord {
    ScannedJournalRecord {
        instance,
        checkpoint_txg: 0,
        new_root_segment: Vec::new(),
        transaction: 0,
        commit_marker: CommitMarker::Present,
        ordinal_within_publish: 1,
        record_flags_byte: RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH,
        slot: 0,
        back_chain,
        chain_value_the_next_record_must_carry,
        named_entry_locations: Vec::new(),
        named_entries: Vec::new(),
    }
}

/// 盘 0、盘 1 的槽 1（偏移一个槽宽）上坐着同一份单元字节，按读的长度截（数据单元宽的读拿到单元后面补零）；别处读不出。
/// 别的版本先走过的单元被再引用的那几条用例（`records/2026-10-24-singlefs-checker-审计复核.md` 的 C01，审计局部对照那一形）用它。
pub(super) struct ImageHoldingOneUnitAtSlotOne {
    pub(super) unit_padded_to_a_data_unit: Vec<u8>,
}

impl ImageHoldingOneUnitAtSlotOne {
    pub(super) fn of(unit: &[u8]) -> Self {
        let mut unit_padded_to_a_data_unit = unit.to_vec();
        unit_padded_to_a_data_unit.resize(data_unit_bytes(), 0);
        Self {
            unit_padded_to_a_data_unit,
        }
    }
}

impl ImageReader for ImageHoldingOneUnitAtSlotOne {
    fn devices(&self) -> Vec<u32> {
        vec![0, 1]
    }
    fn device_bytes(&self, _device: u32) -> Option<u64> {
        Some(1 << 30)
    }
    fn read(&self, _device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        (offset == SLOT_BYTES && length <= self.unit_padded_to_a_data_unit.len())
            .then(|| self.unit_padded_to_a_data_unit[..length].to_vec())
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(vec![1])
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

/// 一个零条目的码 2 节点：树 ID `tree`、key 宽 8、条目宽 8，头里出生身份全 0，头校验和与载荷校验和都封好（fsid 0）。
pub(super) fn index_node_of_tree(tree: u64) -> Vec<u8> {
    let mut unit = vec![0u8; node_bytes()];
    unit[..4].copy_from_slice(b"SFSU");
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&UNIT_FORMAT_VERSION_THIS_CHECKER_READS.to_le_bytes());
    unit[6] = 2;
    unit[42..50].copy_from_slice(&tree.to_le_bytes());
    unit[51] = 8;
    unit[100..102].copy_from_slice(&8u16.to_le_bytes());
    let payload_checksum = crc32_castagnoli_table(&unit[102..]);
    unit[92..96].copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = crc32_castagnoli_table(&unit[..102]);
    unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
    assert!(
        index_node_view(&unit).is_ok(),
        "手造的节点解得开：{:?}",
        index_node_view(&unit)
    );
    unit
}

/// 盘 0、盘 1 上按槽号各坐着同一份单元字节（按读的长度截，数据单元宽的读拿到单元后面补零）；别处读不出。
/// 两版共享的单元下面还要挂一层单元的那几条用例用它（[`ImageHoldingOneUnitAtSlotOne`] 只放得下一个）。
pub(super) struct ImageHoldingUnitsAtSlots {
    unit_padded_to_a_data_unit_by_slot: BTreeMap<u64, Vec<u8>>,
}

impl ImageHoldingUnitsAtSlots {
    pub(super) fn of(units_by_slot: &[(u64, &[u8])]) -> Self {
        Self {
            unit_padded_to_a_data_unit_by_slot: units_by_slot
                .iter()
                .map(|(slot, unit)| {
                    let mut unit_padded_to_a_data_unit = unit.to_vec();
                    unit_padded_to_a_data_unit.resize(data_unit_bytes(), 0);
                    (*slot, unit_padded_to_a_data_unit)
                })
                .collect(),
        }
    }

    /// 指向盘 0、盘 1 槽 `slot` 的节点指针（86 字节），出生 txg 0、出生树 `birth_tree`，两条位置条目的整单元校验和按读 `unit_bytes`
    /// 那么长算。
    pub(super) fn node_pointer_to_slot(
        &self,
        slot: u64,
        unit_bytes: usize,
        birth_tree: u64,
    ) -> Vec<u8> {
        let checksum =
            crc32_castagnoli_table(&self.unit_padded_to_a_data_unit_by_slot[&slot][..unit_bytes]);
        let mut pointer = vec![0u8; node_pointer_bytes()];
        pointer[34..42].copy_from_slice(&birth_tree.to_le_bytes());
        for (device, location_offset) in [(0u32, 50usize), (1u32, 64usize)] {
            pointer[location_offset..location_offset + 4].copy_from_slice(&device.to_le_bytes());
            pointer[location_offset + 4..location_offset + 10]
                .copy_from_slice(&slot.to_le_bytes()[..6]);
            pointer[location_offset + 10..location_offset + 14]
                .copy_from_slice(&checksum.to_le_bytes());
        }
        pointer
    }
}

impl ImageReader for ImageHoldingUnitsAtSlots {
    fn devices(&self) -> Vec<u32> {
        vec![0, 1]
    }
    fn device_bytes(&self, _device: u32) -> Option<u64> {
        Some(1 << 30)
    }
    fn read(&self, _device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        if !offset.is_multiple_of(SLOT_BYTES) {
            return None;
        }
        self.unit_padded_to_a_data_unit_by_slot
            .get(&(offset / SLOT_BYTES))
            .filter(|unit| length <= unit.len())
            .map(|unit| unit[..length].to_vec())
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(
            self.unit_padded_to_a_data_unit_by_slot
                .keys()
                .copied()
                .collect(),
        )
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

// ---- C701（别的判法静默跳过读不出的部分）剩下那几处 ----

/// 一条判定在 `judgements` 收完之后的结论。
pub(super) fn verdict_of(judgements: Judgements, invariant: &str) -> InvariantVerdict {
    judgements
        .into_report()
        .into_iter()
        .find(|(reported, _)| *reported == invariant)
        .map(|(_, verdict)| verdict)
        .expect("判过的不变量都在报告里")
}

/// 默认 768 MiB 环的单元区起始槽号（I-5.5 那几条合成用例的 U）。
pub(super) const UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING: u64 = 50176;

/// 盘 0 上一条分配记录。
pub(super) fn allocation_record_on_device_zero(
    slot: u64,
    span_slots: u64,
    generation: u64,
    is_released: bool,
) -> AllocationRecordView {
    AllocationRecordView {
        device: 0,
        slot,
        span_slots,
        generation,
        is_released,
        quarantine_mark: QuarantineMarkInTheSpanField::Absent,
    }
}

/// 一条字段表宽（200 字节）的树表条目，只填 I-9.27 读的三样：树 ID（偏移 0）、种类（偏移 10）、头 ID（偏移 116）。
pub(super) fn tree_table_entry_with_head_identifier(
    tree: u64,
    kind: u16,
    head_identifier: u64,
) -> Vec<u8> {
    let mut entry = vec![0u8; tree_table_entry_bytes()];
    entry[..8].copy_from_slice(&tree.to_le_bytes());
    entry[TREE_TABLE_ENTRY_KIND_OFFSET..TREE_TABLE_ENTRY_KIND_OFFSET + 2]
        .copy_from_slice(&kind.to_le_bytes());
    entry[TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET..TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8]
        .copy_from_slice(&head_identifier.to_le_bytes());
    entry
}

// ───── 只读快照与 deadlist（I-3.5 ③ ④ 与逆向、I-3.6、I-3.7）与「非空」的比较─────

/// 合成世界里活头 inode 树的树 ID（新池新建文件那一版的号）。
pub(super) const HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD: u64 = 65537;
/// 合成世界里两个快照的码 14 树 ID 与 S.txg：S1 在 txg 4 建、S2 在 txg 6 建。
pub(super) const FIRST_SNAPSHOT_TREE: u64 = 70000;
pub(super) const FIRST_SNAPSHOT_TXG: u64 = 4;
pub(super) const SECOND_SNAPSHOT_TREE: u64 = 70002;
pub(super) const SECOND_SNAPSHOT_TXG: u64 = 6;

/// 一条字段表宽（200 字节）的树表条目，填快照那一遍读的几样：树 ID、种类、flags、根指针（出生 txg 与第 0 条位置条目的设备号非零）、
/// `previous_snapshot_txg`、诞生 txg、头 ID。
#[allow(
    clippy::too_many_arguments,
    reason = "一条树表条目的七个字段各是一个参数：都要写出来，收成结构体只是换个地方逐个填"
)]
fn tree_table_entry_of_the_snapshot_scan(
    tree: u64,
    kind: u16,
    flags: u16,
    root_pointer_birth_txg: u64,
    root_pointer_device: u32,
    previous_snapshot_txg: u64,
    birth_txg: u64,
    head_identifier: u64,
) -> Vec<u8> {
    let mut entry = tree_table_entry_with_head_identifier(tree, kind, head_identifier);
    entry[TREE_TABLE_ENTRY_FLAGS_OFFSET..TREE_TABLE_ENTRY_FLAGS_OFFSET + 2]
        .copy_from_slice(&flags.to_le_bytes());
    entry[14 + 42..14 + 50].copy_from_slice(&root_pointer_birth_txg.to_le_bytes());
    entry[14 + 50..14 + 54].copy_from_slice(&root_pointer_device.to_le_bytes());
    entry[TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET
        ..TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET + 8]
        .copy_from_slice(&previous_snapshot_txg.to_le_bytes());
    entry[TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET..TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET + 8]
        .copy_from_slice(&birth_txg.to_le_bytes());
    entry
}

/// 合成的一张树表：活头 extent 树、inode 树（根指针出生 txg 3），再接 `snapshot_pairs` 里每个快照的一对（码 14 / 码 15，根指针抄活头的）：
/// (码 14 树 ID, S.txg, 前驱, flags)。
pub(super) fn synthetic_tree_table_with_snapshots(
    snapshot_pairs: &[(u64, u64, u64, u16)],
) -> Vec<Vec<u8>> {
    let mut entries = vec![
        tree_table_entry_of_the_snapshot_scan(
            65536,
            TREE_KIND_EXTENT,
            0,
            3,
            1,
            0,
            3,
            HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        ),
        tree_table_entry_of_the_snapshot_scan(
            HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
            TREE_KIND_INODE,
            0,
            3,
            2,
            snapshot_pairs
                .last()
                .map_or(0, |(_, snapshot_txg, _, _)| *snapshot_txg),
            3,
            HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        ),
    ];
    for (tree, snapshot_txg, previous_snapshot_txg, flags) in snapshot_pairs {
        entries.push(tree_table_entry_of_the_snapshot_scan(
            *tree,
            TREE_KIND_SNAPSHOT_INODE_ROOT,
            *flags,
            3,
            2,
            *previous_snapshot_txg,
            *snapshot_txg,
            HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        ));
        entries.push(tree_table_entry_of_the_snapshot_scan(
            tree + 1,
            TREE_KIND_SNAPSHOT_EXTENT_ROOT,
            0,
            3,
            1,
            0,
            *snapshot_txg,
            *tree,
        ));
    }
    entries
}

/// 一个没登记的种类码：种类码登记表（D8（核心索引结构） 已定项 9）到 15 为止。
pub(super) const TREE_KIND_NOT_REGISTERED: u16 = 16;
