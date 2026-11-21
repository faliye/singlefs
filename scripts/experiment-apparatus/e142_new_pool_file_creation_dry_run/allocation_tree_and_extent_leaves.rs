//! 分配记录树的位置、层级与节点数、整棵树的拼装、extent 上层叶条目与事务输出的计数。

use e7_index_bench::{u16_from_u64, usize_from_u64};
use super::*;

// ───────────────────────── 分配记录树：按位置寻址的形状与编码（D8（核心索引结构） 已定项 14，第七节 B1/B3/B13） ─────────────────────────

/// 层级 L（0 = 叶）的一个节点罩多少槽：叶罩 W，往上每升一层乘一次扇出（D8 已定项 14「层级 L 的节点罩 W × 169^L 个槽」）。
pub(super) fn allocation_record_tree_span_at_level(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 根层 R（D8 已定项 14）：取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。
/// `device_slots` 按 δ 甲读法传「设备字节 ÷ 16384」（`DEVICE_SLOTS`），第八节 G3 的 δ乙 变体在调用处另传单元区槽数。
pub(super) fn allocation_record_tree_root_level(device_slots: &[u64]) -> u32 {
    let mut level = 1u32;
    loop {
        let span = allocation_record_tree_span_at_level(level - 1);
        let total: u64 = device_slots.iter().map(|&slots| slots.div_ceil(span)).sum();
        if total <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return level;
        }
        level += 1;
    }
}

/// 给定根层 R，一块盘在根节点里占几格（第七节 B3「root_cells_per_device」）。
pub(super) fn allocation_record_tree_cells_per_device(slots: u64, root_level: u32) -> u64 {
    slots.div_ceil(allocation_record_tree_span_at_level(root_level - 1))
}

/// 新池新建文件在给定几何上要写的分配记录树节点数（第七节 B13「new_pool_file_creation_allocation_nodes」）：
/// 这个实验的全部记录都落在同一片叶（新池新建文件的落点很少）⇒ 每盘一片叶、每盘每个中间层（层级 1..R−1）一个节点、一个根。
pub(super) fn allocation_record_tree_new_pool_file_creation_node_count(device_count: u64, root_level: u32) -> u64 {
    device_count + device_count * u64::from(root_level - 1) + 1
}

/// 位置 key：与 `AllocationRecord::key_bytes()` 同一种编码（设备 4 + 槽号 6）——不是分配记录本身，
/// 是「这一段的起点」，供分配记录树内部 / 根节点的格 key 与 smallest_key / largest_key 共用。
pub(super) fn allocation_position_key(device: u32, slot: u64) -> Vec<u8> {
    AllocationRecord { device: DeviceIdentity(device), slot: SlotNumber(slot), span_slots: 0, generation: CheckpointTxg(0), is_released: false }.key_bytes()
}

/// 分配记录树内部 / 根节点一条内部条目（D8 已定项 14 第 395 行，唯一写法：稀疏，96 字节）：
/// 只给有孩子的格建条目 = 10 字节位置 key（ι甲：这个孩子按位置规定罩的那一段的起点）+ `NodePointer` 86 字节；
/// 没有孩子的格不占条目（不再写 96 字节全 0 的空格）。
fn allocation_internal_cell(device: u32, slot: u64, pointer: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES));
    writer.put(&allocation_position_key(device, slot));
    pointer.write_to(&mut writer);
    writer.bytes
}

/// 分配记录树内部 / 根节点一整层的格（层级 1 一个节点的 169 格，或根的 `cells_per_device * devices` 格）
/// 编成最终写进节点的条目序列（D8 已定项 14 第 395 行，唯一写法：稀疏，按 key 升序）。`slots` 是这一层全部
/// 位置，按位置升序给（位置升序本身就是 key 升序，ι甲：key = 位置起点），每项＝(设备, 这一格罩的起点位置,
/// 这一格的孩子指针（没有孩子就是 `None`，被这里过滤掉、不占条目）)。
fn assemble_allocation_cells(slots: &[(u32, u64, Option<NodePointer>)]) -> Vec<Vec<u8>> {
    slots.iter().filter_map(|&(device, position, child)| child.map(|pointer| allocation_internal_cell(device, position, pointer))).collect()
}

/// 分配记录树写出来的物理节点与它的根指针（这个装置的固定几何：两块盘，R 恒为 2）。
pub(super) struct AllocationTreeBuild {
    /// 按 D3（空间分配） 已定项 10 ⑤ 的 bump 次序：每盘一片叶（设备升序）、每盘一个层级 1 节点（设备升序）、一个根。
    pub(super) units: Vec<(SlotNumber, TransactionUnit, Vec<u8>)>,
    /// 根指针自己就带 `birth_sequence`（NodePointer 的字段），不用另存一份。
    pub(super) root_pointer: NodePointer,
    pub(super) leaf_pointers: Vec<(u32, NodePointer)>,
    pub(super) internal_pointers: Vec<(u32, NodePointer)>,
}

/// 建出分配记录树的全部物理节点。这个装置的两条参数（两块 4 GiB 盘、一块 4 GiB 盘）根都在层级 2——
/// `assert_eq!` 钉住这条假设：变了要先看第五节 5.2 δ 那一格，不能悄悄按旧代码继续跑。
/// 每盘的记录全部落在同一片叶、落在这片叶所在的同一个层级 1 节点，也用 `assert_eq!` 钉住（第七节 B3）。
pub(super) fn build_allocation_tree(parameters: &PoolParameters, records: &[AllocationRecord], txg: CheckpointTxg, instance: InstanceGeneration, sequences: &mut BirthSequenceAllocator) -> AllocationTreeBuild {
    let devices = parameters.devices();
    let device_slots: Vec<u64> = devices.iter().map(|_| parameters.device_slots()).collect();
    let root_level = allocation_record_tree_root_level(&device_slots);
    assert_eq!(root_level, 2, "这个装置的两个几何（每盘 4 GiB 的主点、每盘 8 GiB 的 P8G）根恒在层级 2（第七节 B3）");
    let cells_per_device = allocation_record_tree_cells_per_device(parameters.device_slots(), root_level);
    let level_one_span = allocation_record_tree_span_at_level(root_level - 1);

    let mut units = Vec::new();
    let mut leaf_pointers = Vec::new();
    // 第一段：每盘一片叶（bump 次序：树内先叶后根、同层按 key 升序 ⇒ 设备升序）。
    for device in &devices {
        let device_records: Vec<AllocationRecord> = records.iter().filter(|record| record.device == *device).copied().collect();
        let leaf_indices: std::collections::BTreeSet<u64> = device_records.iter().map(|record| record.slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS).collect();
        assert_eq!(leaf_indices.len(), 1, "这个装置的场景里一块盘的全部记录落在同一片叶（第七节 B3「leaves」）");
        let leaf_index = *leaf_indices.iter().next().expect("刚断言过恰好一个");
        let span_start = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        let smallest_key = allocation_position_key(device.0, span_start);
        let largest_key = allocation_position_key(device.0, span_start + ALLOCATION_RECORD_TREE_LEAF_SLOTS - 1);
        let mut sorted_records = device_records;
        sorted_records.sort_by_key(AllocationRecord::sort_key);
        let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
        let unit = build_index_node(
            TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            0,
            ALLOCATION_KEY_BYTES,
            &smallest_key,
            &largest_key,
            txg,
            &parameters.fsid,
            instance,
            sequence,
            u16_from_u64(ALLOCATION_RECORD_BYTES),
            &sorted_records.iter().map(AllocationRecord::to_bytes).collect::<Vec<_>>(),
        );
        let slot = SlotNumber(if device.0 == 0 { SLOT_ALLOCATION_LEAF_DEVICE_0 } else { SLOT_ALLOCATION_LEAF_DEVICE_1 });
        let pointer = NodePointer { head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), birth_txg: txg }, locations: parameters.location_entries(slot, &unit), instance, birth_sequence: sequence };
        units.push((slot, TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: device.0 }), unit));
        leaf_pointers.push((device.0, pointer));
        let _ = leaf_index; // 供下一段算层级 1 的格号用（重新从 span_start 反推，省一个字段）
    }

    // 第二段：每盘一个层级 1 节点。
    let mut internal_pointers = Vec::new();
    let mut root_children: Vec<(u32, u64, NodePointer)> = Vec::new(); // (device, level_one_index, 指向层级 1 节点的指针)
    for (device, leaf_pointer) in &leaf_pointers {
        let device_records: Vec<&AllocationRecord> = records.iter().filter(|record| record.device.0 == *device).collect();
        let leaf_index = device_records[0].slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        assert!(device_records.iter().all(|record| record.slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS == leaf_index), "同一盘的记录都在同一片叶");
        let level_one_index = leaf_index / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
        let cell_within_level_one = leaf_index % ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
        // 5.2 α：这个层级 1 节点罩的 169 个位置，每个位置的起点槽 = 节点自己的起点 + 位置序号 × 叶宽。
        let level_one_slots: Vec<(u32, u64, Option<NodePointer>)> = (0..ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
            .map(|position| {
                let position_start = level_one_index * level_one_span + position * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                let child = if position == cell_within_level_one { Some(*leaf_pointer) } else { None };
                (*device, position_start, child)
            })
            .collect();
        let cells = assemble_allocation_cells(&level_one_slots);
        let smallest_key = allocation_position_key(*device, level_one_index * level_one_span);
        let largest_key = allocation_position_key(*device, level_one_index * level_one_span + level_one_span - 1);
        let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
        let unit = build_index_node(
            TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            1,
            ALLOCATION_KEY_BYTES,
            &smallest_key,
            &largest_key,
            txg,
            &parameters.fsid,
            instance,
            sequence,
            u16_from_u64(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES),
            &cells,
        );
        let slot = SlotNumber(if *device == 0 { SLOT_ALLOCATION_INTERNAL_DEVICE_0 } else { SLOT_ALLOCATION_INTERNAL_DEVICE_1 });
        let pointer = NodePointer { head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), birth_txg: txg }, locations: parameters.location_entries(slot, &unit), instance, birth_sequence: sequence };
        units.push((slot, TransactionUnit::Allocation(AllocationNodeRole::Internal { device: *device, level: 1 }), unit));
        internal_pointers.push((*device, pointer));
        root_children.push((*device, level_one_index, internal_pointers.last().expect("刚 push 过").1));
    }

    // 第三段：根（单一节点，按盘分流，D8 已定项 14「根罩整个 key 空间、按盘分流」）。
    let root_children_by_position: std::collections::BTreeMap<(u32, u64), NodePointer> =
        root_children.iter().map(|(device, level_one_index, pointer)| ((*device, *level_one_index), *pointer)).collect();
    let mut root_slots: Vec<(u32, u64, Option<NodePointer>)> = Vec::new();
    for device in &devices {
        for level_one_index in 0..cells_per_device {
            let position_start = level_one_index * level_one_span;
            let child = root_children_by_position.get(&(device.0, level_one_index)).copied();
            root_slots.push((device.0, position_start, child));
        }
    }
    let root_cells = assemble_allocation_cells(&root_slots);
    // D8 已定项 14 第 395 行（唯一写法）：根罩整个 key 空间，smallest_key = (0, 0)，largest_key = (0xFFFFFFFF, 2^48 − 1)——
    // 与盘数、盘大小无关（第七节 A6）。
    let root_smallest_key = allocation_position_key(0, 0);
    let root_largest_key = allocation_position_key(u32::MAX, (1u64 << 48) - 1);
    let root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
    let root_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
        u8::try_from(root_level).expect("根层级 1 字节装得下"),
        ALLOCATION_KEY_BYTES,
        &root_smallest_key,
        &root_largest_key,
        txg,
        &parameters.fsid,
        instance,
        root_sequence,
        u16_from_u64(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES),
        &root_cells,
    );
    let root_slot = SlotNumber(SLOT_ALLOCATION_ROOT);
    let root_pointer = NodePointer { head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), birth_txg: txg }, locations: parameters.location_entries(root_slot, &root_unit), instance, birth_sequence: root_sequence };
    units.push((root_slot, TransactionUnit::Allocation(AllocationNodeRole::Root), root_unit));

    AllocationTreeBuild { units, root_pointer, leaf_pointers, internal_pointers }
}

// ───────────────────────── extent 树上段叶（D8（核心索引结构） 已定项 14，ζ：根就是上段叶） ─────────────────────────

/// 标签字节（D8 已定项 14「extent 树」段）：0 全零占位（只有读者认、写者不写）、1 = 下段根指针、
/// 2 = 那一个数据单元的数据指针（内联）。新池新建文件只有一个数据单元的文件 ⇒ 只写得到标签 2。
pub(super) const EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT: u8 = 0;
pub(super) const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
pub(super) const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;

/// extent 树上段叶条目 113（D8 已定项 14）：key 24 `(locality 0, inode, 第三分量)` + 标签 1 + 载荷 88。
/// 第三分量按 5.2 η 甲恒 0（key 前 24 字节与旧的 `build_extent_record` 同一种布局，只是第三分量不再叫「offset」）。
pub(super) fn build_extent_upper_leaf_entry(inode: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES));
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(0);
    writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, "extent 上段叶条目");
    writer.bytes
}

pub(super) fn parse_extent_upper_leaf_entry(bytes: &[u8]) -> ([u8; 24], u8, DataPointer) {
    let mut reader = ByteReader::new(bytes);
    let key: [u8; 24] = reader.take(24).try_into().expect("切了 24 字节");
    let tag = reader.get_u8();
    (key, tag, DataPointer::read_from(&mut reader))
}

/// 新池新建文件写出的东西，留给探针与断言用。
#[derive(Clone, Debug)]
pub(super) struct TransactionOutput {
    pub(super) root: RootRecord,
    pub(super) record: JournalRecord,
    pub(super) record_bytes: Vec<u8>,
    pub(super) units_by_slot: Vec<(SlotNumber, TransactionUnit, Vec<u8>)>,
    pub(super) data_pointer: DataPointer,
    pub(super) mapping_keys: Vec<Vec<u8>>,
    pub(super) allocation_records: Vec<AllocationRecord>,
    pub(super) accounting_entries: Vec<AccountingEntry>,
    pub(super) index_node_header_widths: Vec<(&'static str, usize)>,
}

/// 单元区里的空闲槽数（D5 已定项 7：容量 = 单元区大小，固定结构既不在容量里也不算已分配）。
pub(super) fn free_slot_count(occupied: &std::collections::BTreeSet<u64>, unit_area_slots: u64) -> u64 {
    unit_area_slots - occupied.len() as u64
}

/// 空闲 run 数（统计量第 10 项）：单元区里极长的连续空闲槽段有几段。
pub(super) fn free_run_count(occupied: &std::collections::BTreeSet<u64>, unit_area_slots: u64) -> u64 {
    let mut runs = 0u64;
    let mut previous_was_free = false;
    for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + unit_area_slots {
        let is_free = !occupied.contains(&slot);
        if is_free && !previous_was_free {
            runs += 1;
        }
        previous_was_free = is_free;
    }
    runs
}

/// 全空聚簇段数（统计量第 11 项，D3 已定项 10 ①）：段内 64 槽都没有未释放分配记录的段数。
pub(super) fn empty_cluster_segment_count(occupied: &std::collections::BTreeSet<u64>, unit_area_slots: u64) -> u64 {
    (0..unit_area_slots / CLUSTER_SEGMENT_SLOTS)
        .filter(|segment_index| {
            let start = UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS;
            (start..start + CLUSTER_SEGMENT_SLOTS).all(|slot| !occupied.contains(&slot))
        })
        .count() as u64
}

/// 点名项 key 尾段的两种形态（D23 已定项 17）：码 1 写 10 字节写序；码 2 / 码 3 写实例代号 4 + 出生序号 4 + 补零 2。
pub(super) fn data_key_tail(write_order: WriteOrder) -> [u8; 10] {
    let mut tail = [0u8; 10];
    tail[..4].copy_from_slice(&write_order.instance.0.to_le_bytes());
    tail[4..].copy_from_slice(&write_order.transaction.0.to_le_bytes()[..6]);
    tail
}

pub(super) fn node_key_tail(instance: InstanceGeneration, birth_sequence: BirthSequence) -> [u8; 10] {
    let mut tail = [0u8; 10];
    tail[..4].copy_from_slice(&instance.0.to_le_bytes());
    tail[4..8].copy_from_slice(&birth_sequence.0.to_le_bytes());
    tail
}

#[cfg(test)]
#[path = "allocation_tree_and_extent_leaves_tests.rs"]
mod tests;
