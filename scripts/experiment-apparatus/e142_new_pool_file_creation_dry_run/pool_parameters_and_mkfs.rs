//! 栅栏策略、池参数、出生序号分配器、mkfs 与取得实例、事务里的单元。

use e7_index_bench::{u16_from_u64, usize_from_u64};
use super::*;

// ───────────────────────── mkfs 与新池新建文件 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum BarrierPolicy {
    /// D16 已定项 7：单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置槽 → 屏障（末一道另由 `TrailingPublishBarrier` 管）。
    Settled,
    /// 阳性对照：一道屏障都不放。
    None,
}

/// 发布尾那道屏障（E142 第十九次跑臂 N19C 的 ⑧；D16（发布语义） 已定项 7 条文「COW 单元 / 节点 → 屏障 →
/// journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」）。只在 `BarrierPolicy::Settled` 时才发，
/// 与另两道同一条件；`Omitted` 是取样点 G11、G12 那一格（登记 R44：⑧ 关时就是臂 N19）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TrailingPublishBarrier {
    Issued,
    Omitted,
}

#[derive(Clone, Debug)]
pub(super) struct PoolParameters {
    pub(super) fsid: [u8; 16],
    pub(super) device_count: usize,
    pub(super) region_devices: [DeviceIdentity; 3],
    pub(super) barriers: BarrierPolicy,
    trailing_publish_barrier: TrailingPublishBarrier,
    /// 写者开关（E142 第十四次跑，重跑登记第五节 5.1）：臂 F（真实基线，`crates/` 今天的写法）恒 `0x01`；
    /// 臂 H（对照）恒 `0x00`。本实验每次发布只有一条记录，这条记录也就是它自己发布的末条，
    /// 已定项 17 因此要求它写 1——这个字段只用来在两条臂之间切换，不代表「这条记录是不是末条」有别的取法。
    pub(super) last_of_publish_flag: u8,
    /// 每盘字节数（mkfs 参数，D23 已定项 19 ③ 只给下界）。主点 `DEVICE_BYTES`（4 GiB）；第二十二次跑第二段的 P8G 取 8 GiB。
    /// 只管分配记录树的形状与单元区大小（记账的空闲字节、全空聚簇段数、空闲 run 数），写路别的字节不看它。
    pub(super) device_bytes: u64,
}

impl PoolParameters {
    pub(super) fn settled_two_devices() -> Self {
        Self {
            fsid: FIXED_FSID,
            device_count: 2,
            region_devices: [DeviceIdentity(RING_REGION_DEVICES[0]), DeviceIdentity(RING_REGION_DEVICES[1]), DeviceIdentity(RING_REGION_DEVICES[2])],
            barriers: BarrierPolicy::Settled,
            trailing_publish_barrier: TrailingPublishBarrier::Issued,
            last_of_publish_flag: 0x01,
            device_bytes: DEVICE_BYTES,
        }
    }
    /// 第二十二次跑第二段的第二个几何（第二十二次登记第一节「第 2 行」的读法）：两块盘，每盘字节数换掉，别的字段与 `settled_two_devices` 相同。
    pub(super) fn settled_two_devices_of_device_bytes(device_bytes: u64) -> Self {
        Self { device_bytes, ..Self::settled_two_devices() }
    }
    /// 分配记录树按位置寻址的 key 空间（D8 已定项 14 δ甲）：设备字节 ÷ 16384 向下取整。
    pub(super) fn device_slots(&self) -> u64 {
        slots_of_device_bytes(self.device_bytes)
    }
    /// 单元区大小（D5 已定项 7）：起始槽号到盘尾的整槽数。
    pub(super) fn unit_area_slots(&self) -> u64 {
        unit_area_slots_of_device_bytes(self.device_bytes)
    }
    /// 取样点 G11（登记 R19C-4 ⑩）：主几何，只把发布尾那道屏障关掉，别的字段与 `settled_two_devices` 相同。
    pub(super) fn settled_two_devices_without_trailing_publish_barrier() -> Self {
        Self { trailing_publish_barrier: TrailingPublishBarrier::Omitted, ..Self::settled_two_devices() }
    }
    pub(super) fn control_one_device_no_barriers() -> Self {
        Self {
            fsid: FIXED_FSID,
            device_count: 1,
            region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)],
            barriers: BarrierPolicy::None,
            trailing_publish_barrier: TrailingPublishBarrier::Issued,
            last_of_publish_flag: 0x01,
            device_bytes: DEVICE_BYTES,
        }
    }
    /// E142 第十七次跑第八节 G6：同一个一盘几何，只把 D16（发布语义） 已定项 7 的两道屏障放回去
    /// （与 `control_one_device_no_barriers` 相比只改 `barriers` 这一个字段，用来判「屏障放不放」这一个旋钮的方向）。
    pub(super) fn one_device_with_barriers() -> Self {
        Self {
            fsid: FIXED_FSID,
            device_count: 1,
            region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)],
            barriers: BarrierPolicy::Settled,
            trailing_publish_barrier: TrailingPublishBarrier::Issued,
            last_of_publish_flag: 0x01,
            device_bytes: DEVICE_BYTES,
        }
    }
    /// 发布尾那道屏障发不发（⑧）：两个开关都开才发。
    pub(super) fn issues_trailing_publish_barrier(&self) -> bool {
        match (self.barriers, self.trailing_publish_barrier) {
            (BarrierPolicy::Settled, TrailingPublishBarrier::Issued) => true,
            (BarrierPolicy::Settled, TrailingPublishBarrier::Omitted) | (BarrierPolicy::None, TrailingPublishBarrier::Issued | TrailingPublishBarrier::Omitted) => false,
        }
    }
    /// 臂 H：把写者开关扳到 `0x00`（E142 第十四次跑第五节 5.1）；不改别的字段。
    pub(super) fn with_last_of_publish_flag(mut self, flag: u8) -> Self {
        self.last_of_publish_flag = flag;
        self
    }
    /// 一个单元的两条位置条目：两盘各一份（D2 已定项 10）；只有一块盘时两条都指盘 0（对照臂）。
    pub(super) fn location_entries(&self, slot: SlotNumber, unit: &[u8]) -> [LocationEntry; 2] {
        let unit_checksum = castagnoli_crc32(unit);
        let second_device = DeviceIdentity(u32::try_from(self.device_count - 1).expect("设备数"));
        [LocationEntry { device: DeviceIdentity(0), slot, unit_checksum }, LocationEntry { device: second_device, slot, unit_checksum }]
    }
    pub(super) fn devices(&self) -> Vec<DeviceIdentity> {
        (0..self.device_count).map(|index| DeviceIdentity(u32::try_from(index).expect("设备数"))).collect()
    }
}

/// 出生序号（C291：分配规则只在实验源码里）——装置预想：按 (树, txg, 实例) 从 0 计，码 2 与码 3 共用一个计数。
#[derive(Default)]
pub(super) struct BirthSequenceAllocator {
    counters: BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>,
}

impl BirthSequenceAllocator {
    pub(super) fn next(&mut self, tree: TreeIdentifier, txg: CheckpointTxg, instance: InstanceGeneration) -> BirthSequence {
        let counter = self.counters.entry((tree, txg, instance)).or_insert(0);
        let sequence = BirthSequence(*counter);
        *counter += 1;
        sequence
    }
}

pub(super) fn write_unit_to_every_device(pool: &mut RecordingPool, parameters: &PoolParameters, slot: SlotNumber, unit: &[u8]) {
    for device in parameters.devices() {
        pool.write(device, slot.device_offset(), unit, StepKind::UnitWrite);
    }
}

#[derive(Clone, Debug)]
pub(super) struct MkfsOutput {
    pub(super) root: RootRecord,
    pub(super) instance_table_unit: Vec<u8>,
    pub(super) tree_table_genesis_unit: Vec<u8>,
}

pub(super) fn mkfs(parameters: &PoolParameters) -> (RecordingPool, MkfsOutput) {
    let mut pool = RecordingPool { pool: Pool { devices: vec![SparseDevice::default(); parameters.device_count], device_bytes: parameters.device_bytes }, operations: Vec::new() };
    // D22（单元原子性怎么合成） 已定项 8 第 3 条（追平 C484（mkfs 不清根环，同 fsid 重来旧根还择得中）与 W1）：
    // 写任何元数据之前，每块盘先把 [0, 根环基址) 整段写 0（第一版 [0, 1 MiB) = 64 × 16 KiB），再把根环三个区域与 journal 环
    // 各整段写 0；每块盘上五段各一次调用，区域归属只定根种在哪块盘、不定清哪块盘——两块盘各清全部五段，共 10 步；
    // 段序列与步数以 `.claude/kb/layout/01-first-txn.md` 八「mkfs 种根」那一行为准（`14+1+1+1+4`、25 次操作（21 次写、
    // 两道池屏障按设备记成 4 步）、16402 个崩溃状态，种类 `[zero_fill×10,unit_write×4,barrier×2]|...`）；清零几段之间不加屏障。
    // 条款没定一块盘五段的先后；[0, 根环基址) 那一段放在最前，只记一步、不带字节、不改镜像（`record_zero_fill` 的约定）。
    for device in parameters.devices() {
        pool.record_zero_fill(device, DeviceOffset(0)); // [0, 根环基址)
        for region_index in 0..RING_REGIONS {
            pool.record_zero_fill(device, ring_region_offset(region_index));
        }
        pool.record_zero_fill(device, DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES));
    }
    // D23（journal 的角色与格式） 已定项 16：mkfs 写实例代号 0，单元写序 (0, 0)，第 0 代根实例代号 0。
    let instance = InstanceGeneration(MKFS_INSTANCE_GENERATION);
    let genesis = CheckpointTxg(0);
    let genesis_write_order = WriteOrder { instance, transaction: TransactionNumber(0) };
    let mut sequences = BirthSequenceAllocator::default();

    let instance_table_identity = PackedIdentity { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), record_type: PACKED_TYPE_INSTANCE_TABLE, container: 0, container_birth: genesis };
    let instance_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), genesis, instance);
    let instance_table_unit = build_packed_unit(
        instance_table_identity,
        u16_from_u64(INSTANCE_ROW_BYTES),
        &[build_instance_table_chain_record()],
        genesis,
        &parameters.fsid,
        genesis_write_order,
        instance_table_sequence,
    );
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), genesis, instance);
    let tree_table_genesis_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &[0u8; 8],
        &[0u8; 8],
        genesis,
        &parameters.fsid,
        instance,
        tree_table_sequence,
        u16_from_u64(TREE_TABLE_ENTRY_BYTES),
        &[],
    );
    write_unit_to_every_device(&mut pool, parameters, SlotNumber(SLOT_INSTANCE_TABLE), &instance_table_unit);
    write_unit_to_every_device(&mut pool, parameters, SlotNumber(SLOT_TREE_TABLE_GENESIS), &tree_table_genesis_unit);
    pool.barrier();

    let root = RootRecord {
        fsid: parameters.fsid,
        unmount_marker: false, // mkfs 第 0 代不是正常卸载写的根（D22 已定项 7）
        instance,
        checkpoint_txg: genesis,
        tree_table: NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: genesis },
            locations: parameters.location_entries(SlotNumber(SLOT_TREE_TABLE_GENESIS), &tree_table_genesis_unit),
            instance,
            birth_sequence: tree_table_sequence,
        },
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: genesis },
            locations: parameters.location_entries(SlotNumber(SLOT_INSTANCE_TABLE), &instance_table_unit),
            instance,
            birth_sequence: instance_table_sequence,
        },
        // 中央映射树的根住根记录（D19 已定项 11）；mkfs 那一刻还没有映射树。
        mapping_root: NodePointer::empty_root(),
        // C512（树表 0 条的一版上被换下的实例表记在哪没有条款）：mkfs 第 0 代写全零。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let root_slot = root.to_slot();
    // D22 已定项 8：第 0 代根种进全部区域，各自槽 0，FUA。
    for region in 0..RING_REGIONS {
        pool.write(parameters.region_devices[usize_from_u64(region)], ring_slot_offset(region, 0), &root_slot, StepKind::RootRecordFua);
    }
    // D22（单元原子性怎么合成） 已定项 16：每盘恒 2 个槽，世代号从 1 起；mkfs 把两个槽都种上世代号 1。
    for device in parameters.devices() {
        let system_configuration = SystemConfiguration {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
            rollback_floor: CheckpointTxg(0), // mkfs 没有抬 F（D16 已定项 1）
        };
        for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            pool.write(device, DeviceOffset(slot_offset), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
        }
    }
    pool.barrier();
    (pool, MkfsOutput { root, instance_table_unit, tree_table_genesis_unit })
}

/// D23（journal 的角色与格式） 已定项 16：第一次可写挂载取 max(系统配置, 根环) + 1 = 1，
/// **并写进每一份系统配置（一次系统配置槽写，世代号 +1）之后才动单元**——所以它自成一段，排在暖机之前。
/// 段的收尾靠暖机第一次空发布开头那道屏障（D16 已定项 7 的形态，不另加屏障：这是最少屏障的写法）。
pub(super) fn acquire_instance(pool: &mut RecordingPool, parameters: &PoolParameters) -> InstanceGeneration {
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let slot_index = (SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION % 2) as usize;
    for device in parameters.devices() {
        let system_configuration = SystemConfiguration {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
            rollback_floor: CheckpointTxg(0), // 取号没有抬 F（D16 已定项 1）
        };
        pool.write(device, DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[slot_index]), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
    }
    instance
}

/// 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）拆成的物理节点角色：两片叶（层级 0，一盘一片）、
/// 两个层级 1 节点（一盘一个，主几何下只需要这一层中间节点）、一个根（层级 2，单一节点、按盘分流）。
/// 这个装置的固定几何（两块 4 GiB 盘）只走得到这三层；`Internal` 带层级号是为了让第八节 G3 的形状函数
/// 能对更大的盘（如 256 GiB，根在层级 3）说话，即便这个枚举本身在这个场景里只用得到 `level == 1`。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) enum AllocationNodeRole {
    Leaf { device: u32 },
    Internal { device: u32, level: u32 },
    Root,
}

/// 新池新建文件写出的十二个单元各自的身份。它是封闭枚举：每一处 match 都穷举，
/// 漏一种编译不过。
/// 步骤种类（`StepKind`）说的是「这一步是哪一类写」，这个说的是「写的是哪个单元」——两件事分开。
/// 分配记录树的变体是 `Allocation(AllocationNodeRole)`。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) enum TransactionUnit {
    Data,
    ExtentRoot,
    InodeLeaf,
    InodeRoot,
    Allocation(AllocationNodeRole),
    AccountingTree,
    MappingTree,
    TreeTable,
}

impl TransactionUnit {
    /// E142 第十一次跑量 1/3/4 用的可读标签（`tag()` 的 t1..t12 是既有产物格式，这里不改它）。
    /// 这个装置的场景只跑到两块盘（设备号 0 / 1），别的设备号在这里 panic——不是这一格没考虑到，
    /// 是这个实验的几何写死两盘，撞见别的设备号说明调用方传错了参数。
    pub(super) fn descriptive_tag(self) -> &'static str {
        match self {
            TransactionUnit::Data => "data_unit",
            TransactionUnit::ExtentRoot => "extent_root",
            TransactionUnit::InodeLeaf => "inode_leaf",
            TransactionUnit::InodeRoot => "inode_root",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 0 }) => "allocation_leaf_of_device_0",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 1 }) => "allocation_leaf_of_device_1",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 0, .. }) => "allocation_internal_of_device_0",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 1, .. }) => "allocation_internal_of_device_1",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device, .. }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Root) => "allocation_root",
            TransactionUnit::AccountingTree => "accounting_root",
            TransactionUnit::MappingTree => "mapping_root",
            TransactionUnit::TreeTable => "tree_table",
        }
    }
    /// 字节表七里的步号，E142 第十五次跑起从 t1..t8 延伸到 t1..t12（第七节 B2）：
    /// t5/t6 两片叶、t7/t8 两个层级 1 节点、t9 根，t10..t12 顺延给记账/映射/树表。
    pub(super) fn tag(self) -> &'static str {
        match self {
            TransactionUnit::Data => "t1",
            TransactionUnit::ExtentRoot => "t2",
            TransactionUnit::InodeLeaf => "t3",
            TransactionUnit::InodeRoot => "t4",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 0 }) => "t5",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 1 }) => "t6",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 0, .. }) => "t7",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 1, .. }) => "t8",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device, .. }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Root) => "t9",
            TransactionUnit::AccountingTree => "t10",
            TransactionUnit::MappingTree => "t11",
            TransactionUnit::TreeTable => "t12",
        }
    }
    /// D18（块里携带什么信息） 已定项 11 的类码，加上码 2 的 key 宽（码 1 与码 3 没有 key 区间，写 0）。
    /// D3（空间分配） 已定项 11：分配记录树 key 宽 10 对全部层级成立，不分叶或内部（位置 key 与分配记录 key 同构）。
    pub(super) fn class_and_key_width(self) -> (u8, usize) {
        match self {
            TransactionUnit::Data => (UNIT_CLASS_DATA, 0),
            TransactionUnit::InodeLeaf => (UNIT_CLASS_PACKED, 0),
            TransactionUnit::ExtentRoot => (UNIT_CLASS_INDEX_NODE, 24),
            TransactionUnit::InodeRoot => (UNIT_CLASS_INDEX_NODE, 8),
            TransactionUnit::Allocation(_) => (UNIT_CLASS_INDEX_NODE, ALLOCATION_KEY_BYTES),
            TransactionUnit::AccountingTree => (UNIT_CLASS_INDEX_NODE, 22),
            TransactionUnit::MappingTree => (UNIT_CLASS_INDEX_NODE, usize_from_u64(MAPPING_KEY_BYTES)),
            TransactionUnit::TreeTable => (UNIT_CLASS_INDEX_NODE, TREE_TABLE_KEY_WIDTH),
        }
    }
    /// 点名项里的归属树；树表单元不属于任何一棵树（写 0）。
    pub(super) fn tree(self) -> TreeIdentifier {
        match self {
            TransactionUnit::Data | TransactionUnit::ExtentRoot => TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            TransactionUnit::InodeLeaf | TransactionUnit::InodeRoot => TreeIdentifier(TREE_IDENTIFIER_INODE),
            TransactionUnit::Allocation(_) => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
            TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_MAPPING),
            TransactionUnit::TreeTable => TreeIdentifier(TREE_IDENTIFIER_NONE),
        }
    }
}

#[cfg(test)]
#[path = "pool_parameters_and_mkfs_tests.rs"]
mod tests;
