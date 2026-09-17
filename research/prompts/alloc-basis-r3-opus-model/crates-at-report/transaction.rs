//! 事务层（里程碑步 3 / 步 4 / 步 5）：一个共享的提交状态机 + 一个封闭的提交步骤枚举
//! （D17（实现分层与第三方管道） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
//! 三条路径都从同一个枚举走：取号 = 逐盘一次超级块槽写 + 一道屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）；
//! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
//! 第一个事务（D16（发布语义） 已定项 7）= 单元写 × 8 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
//! 覆盖写（里程碑「第二个事务」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、六个提交内生块与树表 COW 出新版本，
//! 被换下的八个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
//! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。

use std::collections::BTreeMap;

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
    ALLOCATION_RECORD_KEY_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES, FIRST_TRANSACTION_TXG,
    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, MAPPING_ENTRY_BYTES,
    MAPPING_KEY_BYTES, SLOT_BYTES, SUPERBLOCK_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use crate::allocator::{AllocationRecord, Placement, PoolAllocator, UnitFootprint};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::journal::{back_chain_of, record_offset, JournalRecord, NamedUnit};
use crate::make_filesystem::{
    location_entries, MakeFilesystemParameters, MKFS_INSTANCE_GENERATION, TREE_TABLE_KEY_WIDTH,
};
use crate::pointer::{BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead};
use crate::records::{
    build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
    mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key, node_key_tail,
    parse_inode_internal_entry, parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
    ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, STATISTIC_ALLOCATED_BYTES,
    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
    STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
    STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
    STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
};
use crate::recovery::{highest_root_instance, verified_superblock_slots};
use crate::root_record::RootRecord;
use crate::root_ring::{slot_offset, target_for_publish};
use crate::superblock::Superblock;
use crate::unit::{
    build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
    index_node_entry_capacity, parse_index_node, unit_filesystem_identifier, DataUnitIdentity,
    PackedIdentity, WriteOrder, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, UNIT_CLASS_DATA,
    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};

/// 第一个文件的 inode 号（里程碑步 3 预想）。
pub const FIRST_INODE_NUMBER: u64 = 1;
/// 第一个事务的事务号；0 保留给不承载事务的记录（D23（journal 的角色与格式） 已定项 19 ①）。
pub const FIRST_TRANSACTION_NUMBER: u64 = 1;
/// 无归属的树 ID（树表单元自己、mkfs 的固定单元）。
pub const TREE_IDENTIFIER_NONE: u64 = 0;

/// 提交步骤的封闭枚举：五种步骤种类（D17（实现分层与第三方管道） 已定项 2），`match` 不写通配臂——少一个臂编译不过。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitStep<'publish> {
    WriteUnitToEveryDevice {
        slot: SlotNumber,
        unit: &'publish [u8],
    },
    WriteJournalRecordToEveryDevice {
        counter: u64,
        record: &'publish [u8],
    },
    /// 根槽 FUA 写：落区域 `txg mod 3` 的槽 `(txg div 3) mod 8`，写在那个区域归属的盘上。
    WriteRootRecordForceUnitAccess {
        checkpoint_txg: CheckpointTxg,
        root_slot: &'publish [u8],
    },
    /// 每盘一次超级块槽原地覆写：世代号 = 这块盘两槽里自证过的最大世代号 + 1，槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16，逐盘计）。
    RotateSuperblockSlots {
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    },
    Barrier,
}

/// 一个池的写入口：几块盘 + mkfs 参数（几何与区域归属都从这里取）。
pub struct PoolWriter<'pool, Device: BlockDevice> {
    pub parameters: &'pool MakeFilesystemParameters,
    pub devices: &'pool mut [(DeviceIdentity, Device)],
    /// 上一道屏障之后这个写入口发没发过写。没发过就不再发屏障：两道屏障之间没有写，后一道什么也不多保证，
    /// 设备却会多收到一次 FLUSH。新开的写入口不知道之前发生过什么，按「发过」起步。
    has_writes_since_barrier: bool,
}

impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
    #[must_use]
    pub fn new(
        parameters: &'pool MakeFilesystemParameters,
        devices: &'pool mut [(DeviceIdentity, Device)],
    ) -> Self {
        Self {
            parameters,
            devices,
            has_writes_since_barrier: true,
        }
    }
}

impl<Device: BlockDevice> PoolWriter<'_, Device> {
    pub fn perform(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
        if !matches!(step, CommitStep::Barrier) {
            self.has_writes_since_barrier = true;
        }
        match step {
            CommitStep::WriteUnitToEveryDevice { slot, unit } => {
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(slot.to_device_offset(), unit, WriteDurability::Plain)?;
                }
            }
            CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {
                let offset = record_offset(counter, self.parameters.geometry.journal_ring_bytes);
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(offset, record, WriteDurability::Plain)?;
                }
            }
            CommitStep::WriteRootRecordForceUnitAccess {
                checkpoint_txg,
                root_slot,
            } => {
                let target = target_for_publish(checkpoint_txg);
                let region_device =
                    self.parameters.region_devices[usize::try_from(target.region).expect("区域号")];
                let (_, device) = self
                    .devices
                    .iter_mut()
                    .find(|(identity, _)| *identity == region_device)
                    .expect("mkfs 的 check_geometry 核过区域归属");
                device.write_at(
                    slot_offset(
                        target,
                        self.parameters.geometry.fixed_structure_slot_spacing,
                    ),
                    root_slot,
                    WriteDurability::ForceUnitAccess,
                )?;
            }
            CommitStep::RotateSuperblockSlots {
                journal_tail,
                journal_instance,
            } => {
                for index in 0..self.devices.len() {
                    self.write_superblock_slot(index, journal_tail, journal_instance)?;
                }
            }
            CommitStep::Barrier => {
                if self.has_writes_since_barrier {
                    for (_, device) in self.devices.iter_mut() {
                        device.barrier()?;
                    }
                    self.has_writes_since_barrier = false;
                }
            }
        }
        Ok(())
    }

    /// 一块盘写一次超级块槽：世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2。
    fn write_superblock_slot(
        &mut self,
        index: usize,
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    ) -> Result<(), BlockDeviceError> {
        let spacing = u64::from(self.parameters.geometry.fixed_structure_slot_spacing);
        let identity = self.devices[index].0;
        let slot_generation = verified_superblock_slots(
            &*self.devices,
            identity,
            spacing,
            &self.parameters.filesystem_identifier,
        )
        .iter()
        .map(|superblock| superblock.slot_generation)
        .max()
        .unwrap_or(0)
            + 1;
        let superblock = Superblock {
            filesystem_identifier: self.parameters.filesystem_identifier,
            this_device: identity,
            device_count: u32::try_from(self.devices.len()).expect("设备数"),
            slot_generation,
            region_devices: self.parameters.region_devices,
            geometry: self.parameters.geometry,
            journal_tail,
            journal_instance,
        };
        self.has_writes_since_barrier = true;
        self.devices[index].1.write_at(
            DeviceOffsetInBytes((slot_generation % SUPERBLOCK_SLOTS_PER_DEVICE) * spacing),
            &superblock.to_slot(),
            WriteDurability::Plain,
        )
    }

    /// 取号全或无失败：已写出的那几份回卷成旧代号（D18（块里携带什么信息） 已定项 11），回卷写发生在任何单元之前。
    fn roll_back_acquisition(
        &mut self,
        written: &[usize],
        previous_instance: InstanceGeneration,
        cause: BlockDeviceError,
    ) -> AcquisitionFailed {
        if written.is_empty() {
            return AcquisitionFailed {
                cause,
                rollback: AcquisitionRollback::NothingWritten,
            };
        }
        for &index in written {
            if let Err(rollback_error) = self.write_superblock_slot(index, 0, previous_instance) {
                return AcquisitionFailed {
                    cause,
                    rollback: AcquisitionRollback::RollbackFailed(rollback_error),
                };
            }
        }
        AcquisitionFailed {
            cause,
            rollback: AcquisitionRollback::RolledBack,
        }
    }

    #[must_use]
    pub fn location_entries(
        &self,
        slot: SlotNumber,
        unit: &[u8],
    ) -> [crate::pointer::LocationEntry; 2] {
        let identities: Vec<DeviceIdentity> =
            self.devices.iter().map(|(identity, _)| *identity).collect();
        location_entries(&identities, slot, unit)
    }

    fn root_slot_bytes(&self) -> usize {
        usize::try_from(self.parameters.geometry.physical_block_size).expect("根槽宽")
    }
}

/// 取号全或无失败之后回卷写的结局（D18（块里携带什么信息） 已定项 11：先把已经写出的那几份回卷成旧代号，回卷不成才只读挂载）。
#[derive(Debug)]
pub enum AcquisitionRollback {
    /// 第一块盘的取号写就报错了：没有写出过带新号的超级块，没有要回卷的。
    NothingWritten,
    /// 已写出的那几份都回卷成了旧代号；这次挂载只读，盘上若还留着新号，下一次取号跳过它。
    RolledBack,
    /// 回卷写也报错：只读挂载。
    RollbackFailed(BlockDeviceError),
}

/// 取号没做成。拿不到实例代号，调用方就发不出任何带新号的写——那道屏障报错之后不许只重发屏障就继续
/// （D23（journal 的角色与格式） 已定项 16）。
#[derive(Debug)]
pub struct AcquisitionFailed {
    pub cause: BlockDeviceError,
    pub rollback: AcquisitionRollback,
}

/// 取号（D23（journal 的角色与格式） 已定项 16、D18（块里携带什么信息） 已定项 11；2026-09-14 用户定案，
/// C322（取号那一步的屏障怎么放没有条款） 三轮三方）：新号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的
/// 实例代号) + 1；逐盘写一次超级块槽（世代号逐盘 +1）；两写之后一道屏障，屏障完成才把新号交出去，于是本实例的第一个
/// 非超级块写一定排在它之后。首次挂载路径上它就是暖机开场那道：暖机那道屏障前面没有写，写入口不再发第二次，录制流与设备
/// 收到的 FLUSH 数都与只发一道时相同。任一块盘的取号写或那道屏障报错 ⇒ 已写出的那几份回卷成旧代号（取号之前全部自证过的
/// 槽中最大的实例代号），报失败。⚠️ D18（块里携带什么信息） 已定项 11 的「重试到 T_retry 用尽才算失败」这里没有做：
/// 块设备报的第一个错就判失败（写路径上的重试全仓都还没有）。
pub fn acquire_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
) -> Result<InstanceGeneration, AcquisitionFailed> {
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let filesystem_identifier = pool.parameters.filesystem_identifier;
    let identities: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let previous_instance = identities
        .iter()
        .flat_map(|identity| {
            verified_superblock_slots(&*pool.devices, *identity, spacing, &filesystem_identifier)
        })
        .map(|superblock| superblock.journal_instance)
        .max()
        .unwrap_or(MKFS_INSTANCE_GENERATION);
    let highest_root = highest_root_instance(
        &*pool.devices,
        &pool.parameters.region_devices,
        &pool.parameters.geometry,
        &filesystem_identifier,
    )
    .unwrap_or(MKFS_INSTANCE_GENERATION);
    let instance = InstanceGeneration(previous_instance.max(highest_root).0 + 1);
    let mut written = Vec::new();
    for index in 0..pool.devices.len() {
        if let Err(cause) = pool.write_superblock_slot(index, 0, instance) {
            return Err(pool.roll_back_acquisition(&written, previous_instance, cause));
        }
        written.push(index);
    }
    if let Err(cause) = pool.perform(CommitStep::Barrier) {
        return Err(pool.roll_back_acquisition(&written, previous_instance, cause));
    }
    Ok(instance)
}

/// 暖机写出的东西：两代根、两条空记录、最后一条记录的字节（下一条的反向链要罩它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmUpOutput {
    pub roots: Vec<RootRecord>,
    pub records: Vec<JournalRecord>,
    pub last_record_bytes: Vec<u8>,
}

/// 暖机（D16（发布语义） 已定项 8）：两次空发布，checkpoint_txg 走 1、2；空记录不点名任何单元、事务号 0、提交标记 1、
/// 新根段照 mkfs 的根（D16（发布语义） 已定项 9 / D23（journal 的角色与格式） 已定项 19），根记录只改 checkpoint_txg 与实例代号。
pub fn warm_up<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
) -> Result<WarmUpOutput, BlockDeviceError> {
    let mut roots = Vec::new();
    let mut records = Vec::new();
    let mut previous_record_bytes: Option<Vec<u8>> = None;
    for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {
        let txg = CheckpointTxg(txg_number);
        pool.perform(CommitStep::Barrier)?;
        let record = JournalRecord {
            instance,
            counter: txg_number,
            checkpoint_txg: txg,
            transaction: 0,
            is_commit: true,
            back_chain: previous_record_bytes.as_deref().map_or(0, back_chain_of),
            filesystem_identifier: unit_filesystem_identifier(
                &pool.parameters.filesystem_identifier,
            ),
            new_tree_table: genesis.tree_table,
            new_mapping_root: genesis.mapping_root,
            new_tree_identifier_watermark: genesis.tree_identifier_watermark,
            new_rollback_floor: genesis.rollback_floor,
            named: Vec::new(),
        };
        let record_bytes = record.to_bytes();
        pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: txg_number,
            record: &record_bytes,
        })?;
        pool.perform(CommitStep::Barrier)?;
        let root = RootRecord {
            filesystem_identifier: genesis.filesystem_identifier,
            instance,
            checkpoint_txg: txg,
            tree_table: genesis.tree_table,
            tree_identifier_watermark: genesis.tree_identifier_watermark,
            rollback_floor: genesis.rollback_floor,
            instance_table: genesis.instance_table,
            mapping_root: genesis.mapping_root,
        };
        let root_slot = root.to_slot(pool.root_slot_bytes());
        pool.perform(CommitStep::WriteRootRecordForceUnitAccess {
            checkpoint_txg: txg,
            root_slot: &root_slot,
        })?;
        pool.perform(CommitStep::RotateSuperblockSlots {
            journal_tail: txg_number,
            journal_instance: instance,
        })?;
        roots.push(root);
        records.push(record);
        previous_record_bytes = Some(record_bytes);
    }
    Ok(WarmUpOutput {
        roots,
        records,
        last_record_bytes: previous_record_bytes.expect("暖机至少一次"),
    })
}

/// 第一个事务写出的八个单元，声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
/// 也是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）与字节表七的 t1..t8。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionUnit {
    Data,
    ExtentRoot,
    InodeLeaf,
    InodeRoot,
    AllocationTree,
    AccountingTree,
    MappingTree,
    TreeTable,
    /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
    /// 不在第一个事务的八个角色里，第二个事务起才进发布路径。
    InstanceTable,
}

impl TransactionUnit {
    pub const IN_BUMP_ORDER: [TransactionUnit; 8] = [
        TransactionUnit::Data,
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeaf,
        TransactionUnit::InodeRoot,
        TransactionUnit::AllocationTree,
        TransactionUnit::AccountingTree,
        TransactionUnit::MappingTree,
        TransactionUnit::TreeTable,
    ];

    /// 落点跨几个槽：用户数据与 32768 对齐的提交内生块 2 槽，16 KiB 节点 1 槽。
    #[must_use]
    pub fn span_slots(self) -> u64 {
        match self.placement() {
            PlacementRule::UserData => 2,
            PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned) => 2,
            PlacementRule::CommitGenerated(UnitFootprint::OneSlot) => 1,
        }
    }

    /// 字节表七的步号。
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            TransactionUnit::Data => "t1",
            TransactionUnit::ExtentRoot => "t2",
            TransactionUnit::InodeLeaf => "t3",
            TransactionUnit::InodeRoot => "t4",
            TransactionUnit::AllocationTree => "t5",
            TransactionUnit::AccountingTree => "t6",
            TransactionUnit::MappingTree => "t7",
            TransactionUnit::TreeTable => "t8",
            TransactionUnit::InstanceTable => "ti",
        }
    }

    #[must_use]
    pub const fn unit_class(self) -> u8 {
        match self {
            TransactionUnit::Data => UNIT_CLASS_DATA,
            TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => UNIT_CLASS_PACKED,
            TransactionUnit::ExtentRoot
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTree
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => UNIT_CLASS_INDEX_NODE,
        }
    }

    /// 点名项里的归属树；树表单元不属于任何一棵树（写 0）。
    #[must_use]
    pub const fn tree(self) -> TreeIdentifier {
        match self {
            TransactionUnit::Data | TransactionUnit::ExtentRoot => {
                TreeIdentifier(TREE_IDENTIFIER_EXTENT)
            }
            TransactionUnit::InodeLeaf | TransactionUnit::InodeRoot => {
                TreeIdentifier(TREE_IDENTIFIER_INODE)
            }
            TransactionUnit::AllocationTree => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
            TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
            TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
            TransactionUnit::TreeTable | TransactionUnit::InstanceTable => {
                TreeIdentifier(TREE_IDENTIFIER_NONE)
            }
        }
    }

    /// 落点怎么取：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）。
    #[must_use]
    pub const fn placement(self) -> PlacementRule {
        match self {
            TransactionUnit::Data => PlacementRule::UserData,
            TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
                PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
            }
            TransactionUnit::ExtentRoot
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTree
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => PlacementRule::CommitGenerated(UnitFootprint::OneSlot),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementRule {
    UserData,
    CommitGenerated(UnitFootprint),
}

/// 一个写出去的单元：落点、身份、字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedUnit {
    pub slot: SlotNumber,
    pub identity: TransactionUnit,
    pub bytes: Vec<u8>,
}

/// 第一个事务写出的东西，留给验收与探针用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionOutput {
    pub root: RootRecord,
    pub record: JournalRecord,
    pub record_bytes: Vec<u8>,
    /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾、mkfs 之后第一次重写之前不在）。
    pub units: Vec<PublishedUnit>,
    /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
    pub rewritten: Vec<TransactionUnit>,
    pub data_pointer: DataPointer,
    pub mapping_keys: Vec<Vec<u8>>,
    pub allocation_records: Vec<AllocationRecord>,
    pub accounting_entries: Vec<AccountingEntry>,
    pub tree_table_entries: Vec<TreeTableEntry>,
    /// 这次发布写出的 inode 记录（覆盖写要接着它的对象出生代）。
    pub inode_record: InodeRecord,
    /// 六个进映射的单元各自的映射 key（映射树与树表豁免）；下一次覆盖写按它经映射取落点释放（D19（块指针的结构与宽度预算） 已定项 5 第 1 条）。
    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；第一个事务为空）。
    pub released: Vec<Placement>,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，第一个事务恒 0。
    pub key_order_mismatches: u64,
}

impl TransactionOutput {
    #[must_use]
    pub fn unit(&self, identity: TransactionUnit) -> &PublishedUnit {
        self.units
            .iter()
            .find(|unit| unit.identity == identity)
            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")
    }

    /// 树表里某棵树的根指针（照抄没重写的角色时用）。
    #[must_use]
    pub fn tree_root_pointer(&self, tree: u64) -> NodePointer {
        self.tree_table_entries
            .iter()
            .find(|entry| entry.tree == TreeIdentifier(tree))
            .expect("树表里每棵登记的树一条")
            .root
    }

    /// 树表条目的诞生 txg（七条同一个数：树建起来那次发布）。
    #[must_use]
    pub fn tree_birth_txg(&self) -> CheckpointTxg {
        self.tree_table_entries
            .first()
            .expect("树表第 1 版起恒有七条")
            .birth_txg
    }

    /// 这一版全部角色的落点，按写者自己记的槽号（位置提示那一路）。释放不走它、走 `placements_to_release_via_mapping`；
    /// 留着给验收拿两条路互相对。
    #[must_use]
    pub fn placements(&self) -> Vec<Placement> {
        self.units
            .iter()
            .map(|unit| Placement {
                slot: unit.slot,
                span: unit.identity.span_slots(),
            })
            .collect()
    }
}

/// 第一个文件版本换下 mkfs 那片树表单元时要释放的落点：树表这一角色在这次重写、mkfs 的树表单元还登记着、还没释放过。
fn format_time_tree_table_to_release(
    allocator: &PoolAllocator,
    rewritten: &[TransactionUnit],
) -> Vec<Placement> {
    if !rewritten.contains(&TransactionUnit::TreeTable) {
        return Vec::new();
    }
    let Some(tree_table) = allocator.format_time_tree_table() else {
        return Vec::new();
    };
    let still_allocated = allocator
        .devices
        .first()
        .and_then(|device_map| allocator.record_for(device_map.device, tree_table.slot))
        .is_some_and(|record| !record.is_released);
    if still_allocated {
        vec![tree_table]
    } else {
        Vec::new()
    }
}

/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：上一版八个单元的落点从上一版的
/// 映射节点按 key 查出来，查不到就报「不在映射」、一个落点都不释放；映射树与树表豁免映射，各从根记录里指着它们的那条指针取
/// （D19（块指针的结构与宽度预算） 已定项 11：映射树的根住根记录）。两盘同槽（D2（RAID 条带策略） 已定项 10）。
/// 映射只给槽号；跨度取分配记录里的，并与单元种类该有的跨度互核——查出来的槽在册、没释放过、跨度对得上，三样有一样不对就报错、
/// 一个落点都不释放（第二轮攻方腿：此前查得到 key 就直接交给 `release`，落点指错时在断言上 panic）。
///
/// # Errors
/// 查不到 key ⇒ `ReleaseNotInMapping`；槽在分配记录里没条目 ⇒ `ReleaseTargetNotAllocated`；条目已是已释放 ⇒ `ReleaseTargetAlreadyReleased`；
/// 记录的跨度与种类不符 ⇒ `ReleaseSpanMismatch`。
pub fn placements_to_release_via_mapping(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    roles: &[TransactionUnit],
) -> Result<Vec<Placement>, PublishError> {
    let mapping_node_bytes = &previous.unit(TransactionUnit::MappingTree).bytes;
    let mut placements = Vec::new();
    for identity in roles.iter().copied() {
        let locations = match identity {
            TransactionUnit::Data
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeaf
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTree => {
                let (_, key) = previous
                    .mapped_units
                    .iter()
                    .find(|(mapped, _)| *mapped == identity)
                    .expect("六个进映射的单元每个一把 key");
                mapping_locations_for_key(mapping_node_bytes, key)
                    .ok_or(PublishError::ReleaseNotInMapping { unit: identity })?
            }
            TransactionUnit::MappingTree => previous.root.mapping_root.locations,
            TransactionUnit::TreeTable => previous.root.tree_table.locations,
            TransactionUnit::InstanceTable => previous.root.instance_table.locations,
        };
        assert_eq!(
            locations[0].slot, locations[1].slot,
            "两盘同槽（D2（RAID 条带策略） 已定项 10）"
        );
        let slot = locations[0].slot;
        let record = allocator.record_for(locations[0].device, slot).ok_or(
            PublishError::ReleaseTargetNotAllocated {
                unit: identity,
                slot,
            },
        )?;
        if record.is_released {
            return Err(PublishError::ReleaseTargetAlreadyReleased {
                unit: identity,
                slot,
            });
        }
        let recorded_span = u64::from(record.span_slots);
        if recorded_span != identity.span_slots() {
            return Err(PublishError::ReleaseSpanMismatch {
                unit: identity,
                slot,
                recorded_span,
                expected_span: identity.span_slots(),
            });
        }
        placements.push(Placement {
            slot,
            span: recorded_span,
        });
    }
    Ok(placements)
}

/// 在一个映射节点里按 key 查落点；解不开节点或没有这把 key 都是 `None`。
#[must_use]
pub fn mapping_locations_for_key(
    mapping_node_bytes: &[u8],
    key: &[u8],
) -> Option<[LocationEntry; 2]> {
    let node = parse_index_node(mapping_node_bytes).ok()?;
    node.entries
        .iter()
        .map(|entry| parse_mapping_entry(entry))
        .find(|(candidate, _)| candidate == key)
        .map(|(_, locations)| locations)
}

/// 发布能出的错：单元区放不下（哪一个单元没拿到落点），或底层块设备错。
#[derive(Debug)]
pub enum PublishError {
    NoSpaceFor {
        unit: TransactionUnit,
    },
    /// 分配记录树第一版只有一个节点，这次发布之后的记录数装不下（每次发布每盘加 8 条、释放只改写不删）。
    AllocationRecordsExceedOneNode {
        records: usize,
        capacity: usize,
    },
    /// 释放判定路径在上一版的映射里查不到这个单元（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：不按提示释放）。
    ReleaseNotInMapping {
        unit: TransactionUnit,
    },
    /// 映射查出来的落点在分配记录里没有条目。
    ReleaseTargetNotAllocated {
        unit: TransactionUnit,
        slot: SlotNumber,
    },
    /// 映射查出来的落点已经是已释放的（同一个落点释放两次）。
    ReleaseTargetAlreadyReleased {
        unit: TransactionUnit,
        slot: SlotNumber,
    },
    /// 分配记录里的跨度与这个单元种类该有的跨度不符（映射给槽、记录给跨度，两边对不上）。
    ReleaseSpanMismatch {
        unit: TransactionUnit,
        slot: SlotNumber,
        recorded_span: u64,
        expected_span: u64,
    },
    /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）。
    ContentExceedsDataUnit {
        bytes: usize,
        capacity: usize,
    },
    BlockDevice(BlockDeviceError),
}

impl From<BlockDeviceError> for PublishError {
    fn from(error: BlockDeviceError) -> Self {
        PublishError::BlockDevice(error)
    }
}

/// 出生序号：同一 (树, txg, 实例) 里每写出一个码 2 / 码 3 单元加 1，从 0 起（D19（块指针的结构与宽度预算） 已定项 9）。
#[derive(Default)]
pub struct BirthSequenceAllocator {
    counters: BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>,
}

impl BirthSequenceAllocator {
    pub fn next(
        &mut self,
        tree: TreeIdentifier,
        txg: CheckpointTxg,
        instance: InstanceGeneration,
    ) -> BirthSequence {
        let counter = self.counters.entry((tree, txg, instance)).or_insert(0);
        let sequence = BirthSequence(*counter);
        *counter += 1;
        sequence
    }
}

/// 同一请求里的数据单元按取号序排，相邻两把 key 反序的次数（D16（发布语义） 已定项 5 的切分纪律；C319 的计数）。
#[must_use]
pub fn count_key_order_mismatches(extent_keys_in_issue_order: &[[u8; 24]]) -> u64 {
    let field_wise = |key: &[u8; 24]| {
        (
            u64::from_le_bytes(key[0..8].try_into().expect("8")),
            u64::from_le_bytes(key[8..16].try_into().expect("8")),
            u64::from_le_bytes(key[16..24].try_into().expect("8")),
        )
    };
    u64::try_from(
        extent_keys_in_issue_order
            .windows(2)
            .filter(|pair| field_wise(&pair[0]) >= field_wise(&pair[1]))
            .count(),
    )
    .expect("次数")
}

/// 第一个文件：内容与写入时间（时间是参数，不取系统时钟，同参数两次发布逐字节相同）。
#[derive(Clone, Copy, Debug)]
pub struct FirstFile<'content> {
    pub content: &'content [u8],
    pub write_time_seconds: u64,
}

/// 这次发布要写的文件版本：四个文件角色（数据、extent 根、inode 叶、inode 根）全部重写。没有它的发布（写行、暖机）文件角色照抄上一版。
#[derive(Clone, Copy, Debug)]
pub struct FileVersionPlan<'content> {
    pub content: &'content [u8],
    pub write_time_seconds: u64,
    /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
    pub inode_object_birth: CheckpointTxg,
    /// 改动计数（D8（核心索引结构） 已定项 6 偏移 88）。
    pub change_count: u64,
}

/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者重写成给定的记录（行记录在前、链指针记录最末，D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Debug)]
pub enum InstanceTablePlan {
    Carry(NodePointer),
    Rewrite(Vec<Vec<u8>>),
}

/// 一次发布的全部参数：哪些角色重写、身份字段取什么。第一个事务、覆盖写、写行、暖机都是它的一种取值，走同一条发布路径
/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
#[derive(Clone, Debug)]
pub struct PublishPlan<'content> {
    pub txg: CheckpointTxg,
    /// jsn 计数器（记录落在环里的槽位），全池接着走、换实例不归零（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    pub counter: u64,
    /// 事务号，按实例计数从 1 起，空发布写 0（D23（journal 的角色与格式） 已定项 7 / 已定项 19 ①）。
    pub transaction: u64,
    pub instance: InstanceGeneration,
    /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub file: Option<FileVersionPlan<'content>>,
    pub instance_table: InstanceTablePlan,
    /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
    pub tree_birth_txg: CheckpointTxg,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
}

impl PublishPlan<'_> {
    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。
    #[must_use]
    pub fn rewritten_roles(&self) -> Vec<TransactionUnit> {
        let mut roles = Vec::new();
        if self.file.is_some() {
            roles.push(TransactionUnit::Data);
        }
        if matches!(self.instance_table, InstanceTablePlan::Rewrite(_)) {
            roles.push(TransactionUnit::InstanceTable);
        }
        if self.file.is_some() {
            roles.extend([
                TransactionUnit::ExtentRoot,
                TransactionUnit::InodeLeaf,
                TransactionUnit::InodeRoot,
            ]);
        }
        roles.extend([
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]);
        roles
    }
}

/// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    genesis: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: FIRST_TRANSACTION_TXG,
            transaction: FIRST_TRANSACTION_NUMBER,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: txg,
                change_count: 1,
            }),
            instance_table: InstanceTablePlan::Carry(genesis.instance_table),
            tree_birth_txg: txg,
            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            rollback_floor: CheckpointTxg(0),
        },
        None,
    )
}

/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里接在上一次发布之后再发布一版同一个文件——txg、jsn、事务号各加一，
/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的八个落点经映射释放（进 defer 队列）。
///
/// # Errors
/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、空间不够、装不下、块设备报错，都原样交回。
pub fn publish_overwrite<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: previous.record.transaction + 1,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: previous.inode_record.object_birth,
                change_count: txg.0,
            }),
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 发布一版：先做准入（内容装得进一个数据单元、分配记录树装得下这次的记录），再释放上一版被换下的角色的落点、分配、装单元、
/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）。
///
/// # Errors
/// `ContentExceedsDataUnit`、`AllocationRecordsExceedOneNode`、释放判定路径的四种错、`NoSpaceFor`、块设备错。
pub fn publish_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    let rewritten = plan.rewritten_roles();
    // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
    let release = match previous {
        Some(previous_version) => {
            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?
        }
        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
        None => format_time_tree_table_to_release(allocator, &rewritten),
    };
    // 用户给的内容装不进一个数据单元是调用方能恢复的失败，不是不变量被破坏：报错，不走到 build_data_unit 的断言。
    if let Some(file) = &plan.file {
        let data_unit_capacity = data_unit_payload_capacity();
        if file.content.len() > data_unit_capacity {
            return Err(PublishError::ContentExceedsDataUnit {
                bytes: file.content.len(),
                capacity: data_unit_capacity,
            });
        }
    }
    // 分配记录树第一版只有一个节点：这次发布之后装不下就在动分配器之前报错，不许走到 build_index_node 的断言
    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
    let records_after_this_publish =
        allocator.records().len() + rewritten.len() * allocator.devices.len();
    let allocation_node_capacity = index_node_entry_capacity(
        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
        usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    );
    if records_after_this_publish > allocation_node_capacity {
        return Err(PublishError::AllocationRecordsExceedOneNode {
            records: records_after_this_publish,
            capacity: allocation_node_capacity,
        });
    }
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let outcome = publish_admitted(pool, allocator, &plan, previous, &rewritten, &release);
    if outcome.is_err() {
        *allocator = allocator_before_this_publish;
    }
    outcome
}

/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
    previous.unit(identity).clone()
}

/// 准入之后的那一段：释放、分配、装单元、落盘。失败时分配器由调用方退回，这里不管。
#[allow(
    clippy::too_many_lines,
    reason = "一次发布就是一件能单独验证的事：九个角色的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
)]
fn publish_admitted<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    rewritten: &[TransactionUnit],
    release: &[Placement],
) -> Result<TransactionOutput, PublishError> {
    let txg = plan.txg;
    let instance = plan.instance;
    let write_order = WriteOrder {
        instance,
        transaction: plan.transaction,
    };
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let mut sequences = BirthSequenceAllocator::default();

    // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
    for placement in release {
        allocator.release(*placement, txg);
    }

    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
    for identity in rewritten {
        let placement = match identity.placement() {
            PlacementRule::UserData => allocator.allocate_user_data(txg),
            PlacementRule::CommitGenerated(footprint) => {
                allocator.allocate_commit_generated(footprint, txg)
            }
        };
        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: *identity })?;
        slots.insert(*identity, placement.slot);
    }
    let slot_of = |identity: TransactionUnit| slots[&identity];

    // 文件角色：有新版本就装四个单元，没有就照抄上一版的指针与字节。
    let carried_file = match &plan.file {
        Some(_) => None,
        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
    };
    let (
        data_unit,
        data_pointer,
        extent_unit,
        key_order_mismatches,
        inode_leaf_unit,
        inode_leaf_pointer,
        inode_leaf_sequence,
        inode_record,
        inode_root_unit,
        extent_sequence,
        inode_root_sequence,
    ) = match (&plan.file, carried_file) {
        (Some(file), _) => {
            // t1 数据单元（字节表二）。
            let data_identity = DataUnitIdentity {
                tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                object: FIRST_INODE_NUMBER,
                object_birth: file.inode_object_birth,
                anchor_offset: 0,
            };
            let data_unit = build_data_unit(
                data_identity,
                txg,
                filesystem_identifier,
                write_order,
                file.content,
            );
            let data_pointer = DataPointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                    birth_txg: txg,
                },
                locations: pool.location_entries(slot_of(TransactionUnit::Data), &data_unit),
                write_order,
            };
            let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
            let extent_key: [u8; 24] = extent_record
                [..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
                .try_into()
                .expect("24");
            let key_order_mismatches = count_key_order_mismatches(&[extent_key]);

            // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
            // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
            let inode_leaf_identity = PackedIdentity {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
                record_type: PACKED_TYPE_INODE,
                container: FIRST_INODE_NUMBER,
                container_birth: plan.tree_birth_txg,
            };
            let inode_leaf_sequence =
                sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
            let inode_record = InodeRecord {
                inode: FIRST_INODE_NUMBER,
                object_birth: file.inode_object_birth,
                size: u64::try_from(file.content.len()).expect("文件长度"),
                change_count: file.change_count,
                write_time_seconds: file.write_time_seconds,
            };
            let inode_leaf_unit = build_packed_unit(
                inode_leaf_identity,
                u16::try_from(INODE_RECORD_BYTES).expect("140"),
                &[inode_record.to_bytes()],
                txg,
                filesystem_identifier,
                write_order,
                inode_leaf_sequence,
            );
            let inode_leaf_pointer = NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
                    birth_txg: txg,
                },
                locations: pool
                    .location_entries(slot_of(TransactionUnit::InodeLeaf), &inode_leaf_unit),
                instance,
                birth_sequence: inode_leaf_sequence,
            };

            // t2 extent 树根兼叶（字节表四·二）。
            let extent_sequence =
                sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
            let extent_unit = build_index_node(
                TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                0,
                usize::try_from(EXTENT_KEY_BYTES).expect("24"),
                &extent_key,
                &extent_key,
                txg,
                filesystem_identifier,
                instance,
                extent_sequence,
                u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
                std::slice::from_ref(&extent_record),
            );

            // t4 inode 树根：层级 1，key 区间 [ino, ino]，一条 120 字节条目。
            let inode_root_sequence =
                sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
            let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
            let inode_root_unit = build_index_node(
                TreeIdentifier(TREE_IDENTIFIER_INODE),
                1,
                8,
                &inode_key,
                &inode_key,
                txg,
                filesystem_identifier,
                instance,
                inode_root_sequence,
                u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
                &[build_inode_internal_entry(
                    FIRST_INODE_NUMBER,
                    inode_leaf_identity,
                    inode_leaf_pointer,
                )],
            );
            (
                data_unit,
                data_pointer,
                extent_unit,
                key_order_mismatches,
                inode_leaf_unit,
                inode_leaf_pointer,
                inode_leaf_sequence,
                inode_record,
                inode_root_unit,
                extent_sequence,
                inode_root_sequence,
            )
        }
        (None, Some(carried)) => {
            // 照抄：数据指针、inode 记录、四个单元的字节都取上一版；inode 叶的指针从上一版 inode 根的那条条目解出来。
            let inode_root_bytes = &carried.unit(TransactionUnit::InodeRoot).bytes;
            let inode_root_node = parse_index_node(inode_root_bytes)
                .expect("上一版的 inode 根是这次或上次发布装出来的，解得开");
            let (_, _inode_leaf_identity, inode_leaf_pointer) = parse_inode_internal_entry(
                inode_root_node
                    .entries
                    .first()
                    .expect("第一版 inode 树根恒有一条条目"),
            );
            let extent_unit = carried.unit(TransactionUnit::ExtentRoot).bytes.clone();
            let extent_pointer_previous = carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT);
            let inode_root_pointer_previous = carried.tree_root_pointer(TREE_IDENTIFIER_INODE);
            (
                carried.unit(TransactionUnit::Data).bytes.clone(),
                carried.data_pointer,
                extent_unit,
                0,
                carried.unit(TransactionUnit::InodeLeaf).bytes.clone(),
                inode_leaf_pointer,
                inode_leaf_pointer.birth_sequence,
                carried.inode_record,
                carried.unit(TransactionUnit::InodeRoot).bytes.clone(),
                extent_pointer_previous.birth_sequence,
                inode_root_pointer_previous.birth_sequence,
            )
        }
        (None, None) => {
            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
        }
    };
    // 实例表单元（码 3 打包记录类型 4）：重写时容器身份照 mkfs（容器 0、出生代 0），归树 0、在树表之前发号（mkfs 也是先实例表后树表）。
    let instance_table_rewrite = match &plan.instance_table {
        InstanceTablePlan::Rewrite(records) => {
            let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
            let unit = build_packed_unit(
                PackedIdentity {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    record_type: PACKED_TYPE_INSTANCE_TABLE,
                    container: 0,
                    container_birth: CheckpointTxg(0),
                },
                u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
                records,
                txg,
                filesystem_identifier,
                write_order,
                sequence,
            );
            let pointer = NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    birth_txg: txg,
                },
                locations: pool.location_entries(slot_of(TransactionUnit::InstanceTable), &unit),
                instance,
                birth_sequence: sequence,
            };
            Some((unit, pointer, sequence))
        }
        InstanceTablePlan::Carry(_) => None,
    };
    let instance_table_pointer = match (&plan.instance_table, &instance_table_rewrite) {
        (InstanceTablePlan::Carry(pointer), _) => *pointer,
        (InstanceTablePlan::Rewrite(_), Some((_, pointer, _))) => *pointer,
        (InstanceTablePlan::Rewrite(_), None) => unreachable!("重写时上面一定装了单元"),
    };

    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_sequence = sequences.next(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
        txg,
        instance,
    );
    let allocation_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
        0,
        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
        &allocation_records[0].key_bytes(),
        &allocation_records[allocation_records.len() - 1].key_bytes(),
        txg,
        filesystem_identifier,
        instance,
        allocation_sequence,
        u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
        &allocation_records
            .iter()
            .map(AllocationRecord::to_bytes)
            .collect::<Vec<_>>(),
    );

    // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
    // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
    let pool_wide = |statistic: u16, tree: u64, value: u64| AccountingEntry {
        statistic,
        tree: TreeIdentifier(tree),
        device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
        generation: txg,
        value,
        sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
    };
    let mut accounting_entries = vec![
        pool_wide(
            STATISTIC_INODE_WATERMARK,
            TREE_IDENTIFIER_INODE,
            FIRST_INODE_NUMBER + 1,
        ),
        pool_wide(STATISTIC_PENDING_DELETE_BYTES, TREE_IDENTIFIER_NONE, 0),
        pool_wide(
            STATISTIC_COMMITTED_RESERVATION_BYTES,
            TREE_IDENTIFIER_NONE,
            0,
        ),
    ];
    for device_map in &allocator.devices {
        let per_device = |statistic: u16, value: u64| AccountingEntry {
            statistic,
            tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            device: device_map.device,
            generation: txg,
            value,
            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
        };
        accounting_entries.push(per_device(
            STATISTIC_ALLOCATED_BYTES,
            device_map.allocated_slots() * SLOT_BYTES,
        ));
        // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
        accounting_entries.push(per_device(
            STATISTIC_FREE_BYTES,
            device_map.free_slots() * SLOT_BYTES,
        ));
        accounting_entries.push(per_device(STATISTIC_UNRECLAIMABLE_BYTES, 0));
        // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
        accounting_entries.push(per_device(
            STATISTIC_DEFER_QUEUE_BYTES,
            device_map.deferred_slots() * SLOT_BYTES,
        ));
        accounting_entries.push(per_device(
            STATISTIC_FRAGMENTATION_RUNS,
            device_map.free_runs(),
        ));
        accounting_entries.push(per_device(
            STATISTIC_EMPTY_CLUSTER_SEGMENTS,
            device_map.empty_segments(),
        ));
    }
    accounting_entries.sort_by_key(AccountingEntry::sort_key);
    let accounting_sequence =
        sequences.next(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), txg, instance);
    let accounting_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
        0,
        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        &accounting_entries[0].key_bytes(),
        &accounting_entries[accounting_entries.len() - 1].key_bytes(),
        txg,
        filesystem_identifier,
        instance,
        accounting_sequence,
        u16::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
        &accounting_entries
            .iter()
            .map(AccountingEntry::to_bytes)
            .collect::<Vec<_>>(),
    );

    let node_pointer =
        |tree: u64, identity: TransactionUnit, unit: &[u8], sequence: BirthSequence| NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(tree),
                birth_txg: txg,
            },
            locations: pool.location_entries(slot_of(identity), unit),
            instance,
            birth_sequence: sequence,
        };
    // 文件角色的指针：重写的按这次的落点算，照抄的取上一版。
    let (extent_pointer, inode_root_pointer) = match carried_file {
        None => (
            node_pointer(
                TREE_IDENTIFIER_EXTENT,
                TransactionUnit::ExtentRoot,
                &extent_unit,
                extent_sequence,
            ),
            node_pointer(
                TREE_IDENTIFIER_INODE,
                TransactionUnit::InodeRoot,
                &inode_root_unit,
                inode_root_sequence,
            ),
        ),
        Some(carried) => (
            carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT),
            carried.tree_root_pointer(TREE_IDENTIFIER_INODE),
        ),
    };
    let allocation_pointer = node_pointer(
        TREE_IDENTIFIER_ALLOCATION_RECORDS,
        TransactionUnit::AllocationTree,
        &allocation_unit,
        allocation_sequence,
    );
    let accounting_pointer = node_pointer(
        TREE_IDENTIFIER_ACCOUNTING,
        TransactionUnit::AccountingTree,
        &accounting_unit,
        accounting_sequence,
    );

    // t7 中央映射树（字节表三·二）：码 1 一条 + 码 2 / 码 3 五条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // 照抄的文件角色 key 照旧、位置照旧；重写的按这次的指针算。
    let mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> = vec![
        (
            TransactionUnit::Data,
            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
            data_pointer.locations,
        ),
        (
            TransactionUnit::ExtentRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
            extent_pointer.locations,
        ),
        (
            TransactionUnit::InodeLeaf,
            mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer),
            inode_leaf_pointer.locations,
        ),
        (
            TransactionUnit::InodeRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
            inode_root_pointer.locations,
        ),
        (
            TransactionUnit::AllocationTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
            allocation_pointer.locations,
        ),
        (
            TransactionUnit::AccountingTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
            accounting_pointer.locations,
        ),
    ];
    let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations
        .iter()
        .map(|(identity, key, _)| (*identity, key.clone()))
        .collect();
    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = mapped_units_with_locations
        .into_iter()
        .map(|(_, key, locations)| (key, locations))
        .collect();
    mapping_entries.sort_by_key(|(key, _)| mapping_key_sort_key(key));
    let mapping_sequence = sequences.next(
        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
        txg,
        instance,
    );
    let mapping_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
        0,
        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
        &mapping_entries[0].0,
        &mapping_entries[mapping_entries.len() - 1].0,
        txg,
        filesystem_identifier,
        instance,
        mapping_sequence,
        u16::try_from(MAPPING_ENTRY_BYTES).expect("55"),
        &mapping_entries
            .iter()
            .map(|(key, locations)| build_mapping_entry(key, *locations))
            .collect::<Vec<_>>(),
    );
    let mapping_pointer = node_pointer(
        TREE_IDENTIFIER_CENTRAL_MAPPING,
        TransactionUnit::MappingTree,
        &mapping_unit,
        mapping_sequence,
    );

    // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
    // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
    let table_entry =
        |kind: u16, tree: u64, root: NodePointer, head_identifier: u64| TreeTableEntry {
            kind,
            tree: TreeIdentifier(tree),
            root,
            birth_txg: plan.tree_birth_txg,
            head_identifier,
        };
    let tree_table_entries = vec![
        table_entry(
            TREE_KIND_EXTENT,
            TREE_IDENTIFIER_EXTENT,
            extent_pointer,
            TREE_IDENTIFIER_INODE,
        ),
        table_entry(
            TREE_KIND_INODE,
            TREE_IDENTIFIER_INODE,
            inode_root_pointer,
            TREE_IDENTIFIER_INODE,
        ),
        table_entry(
            TREE_KIND_ALLOCATION,
            TREE_IDENTIFIER_ALLOCATION_RECORDS,
            allocation_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_ACCOUNTING,
            TREE_IDENTIFIER_ACCOUNTING,
            accounting_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_LIVELIST,
            TREE_IDENTIFIER_LIVELIST,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_SPARSE_SIDE_TABLE,
            TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_DEADLIST,
            TREE_IDENTIFIER_DEADLIST,
            NodePointer::empty_root(),
            0,
        ),
    ];
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
        &TREE_IDENTIFIER_DEADLIST.to_le_bytes(),
        txg,
        filesystem_identifier,
        instance,
        tree_table_sequence,
        u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
        &tree_table_entries
            .iter()
            .map(TreeTableEntry::to_bytes)
            .collect::<Vec<_>>(),
    );
    let tree_table_pointer = node_pointer(
        TREE_IDENTIFIER_NONE,
        TransactionUnit::TreeTable,
        &tree_table_unit,
        tree_table_sequence,
    );

    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾）。
    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
        slot: slot_of(identity),
        identity,
        bytes,
    };
    let mut units: Vec<PublishedUnit> = Vec::new();
    for identity in TransactionUnit::IN_BUMP_ORDER {
        let unit = match identity {
            TransactionUnit::Data => match carried_file {
                None => rewritten_unit(identity, data_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::ExtentRoot => match carried_file {
                None => rewritten_unit(identity, extent_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::InodeLeaf => match carried_file {
                None => rewritten_unit(identity, inode_leaf_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::InodeRoot => match carried_file {
                None => rewritten_unit(identity, inode_root_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::AllocationTree => rewritten_unit(identity, allocation_unit.clone()),
            TransactionUnit::AccountingTree => rewritten_unit(identity, accounting_unit.clone()),
            TransactionUnit::MappingTree => rewritten_unit(identity, mapping_unit.clone()),
            TransactionUnit::TreeTable => rewritten_unit(identity, tree_table_unit.clone()),
            TransactionUnit::InstanceTable => {
                unreachable!("IN_BUMP_ORDER 只有八个文件 / 固定点角色")
            }
        };
        units.push(unit);
    }
    match (&instance_table_rewrite, previous) {
        (Some((unit, _, _)), _) => {
            units.push(rewritten_unit(TransactionUnit::InstanceTable, unit.clone()))
        }
        (None, Some(previous_version)) => {
            if let Some(carried) = previous_version
                .units
                .iter()
                .find(|unit| unit.identity == TransactionUnit::InstanceTable)
            {
                units.push(carried.clone());
            }
        }
        (None, None) => {}
    }

    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
        match identity {
            TransactionUnit::Data => data_key_tail(data_pointer.write_order),
            TransactionUnit::ExtentRoot => node_key_tail(instance, extent_sequence),
            TransactionUnit::InodeLeaf => node_key_tail(instance, inode_leaf_sequence),
            TransactionUnit::InodeRoot => node_key_tail(instance, inode_root_sequence),
            TransactionUnit::AllocationTree => node_key_tail(instance, allocation_sequence),
            TransactionUnit::AccountingTree => node_key_tail(instance, accounting_sequence),
            TransactionUnit::MappingTree => node_key_tail(instance, mapping_sequence),
            TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
            TransactionUnit::InstanceTable => node_key_tail(
                instance,
                instance_table_rewrite
                    .as_ref()
                    .map(|(_, _, sequence)| *sequence)
                    .expect("点名实例表单元的发布一定重写了它"),
            ),
        }
    };
    let written_units: Vec<&PublishedUnit> = rewritten
        .iter()
        .map(|identity| {
            units
                .iter()
                .find(|unit| unit.identity == *identity)
                .expect("重写的角色都装了单元")
        })
        .collect();
    let named: Vec<NamedUnit> = written_units
        .iter()
        .map(|unit| NamedUnit {
            locations: pool.location_entries(unit.slot, &unit.bytes),
            unit_class: unit.identity.unit_class(),
            birth_tree: unit.identity.tree(),
            birth_txg: txg,
            key_tail: key_tail_of(unit.identity),
        })
        .collect();
    let record = JournalRecord {
        instance,
        counter: plan.counter,
        checkpoint_txg: txg,
        transaction: plan.transaction,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
        new_tree_table: tree_table_pointer,
        new_mapping_root: mapping_pointer,
        new_tree_identifier_watermark: plan.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named,
    };
    let record_bytes = record.to_bytes();
    let root = RootRecord {
        filesystem_identifier: *filesystem_identifier,
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: mapping_pointer,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());

    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
    for unit in &written_units {
        pool.perform(CommitStep::WriteUnitToEveryDevice {
            slot: unit.slot,
            unit: &unit.bytes,
        })?;
    }
    pool.perform(CommitStep::Barrier)?;
    pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
        counter: plan.counter,
        record: &record_bytes,
    })?;
    pool.perform(CommitStep::Barrier)?;
    pool.perform(CommitStep::WriteRootRecordForceUnitAccess {
        checkpoint_txg: txg,
        root_slot: &root_slot,
    })?;
    pool.perform(CommitStep::RotateSuperblockSlots {
        journal_tail: plan.counter,
        journal_instance: instance,
    })?;

    Ok(TransactionOutput {
        root,
        record,
        record_bytes,
        units,
        rewritten: rewritten.to_vec(),
        data_pointer,
        mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
        allocation_records,
        accounting_entries,
        tree_table_entries,
        inode_record,
        mapped_units,
        released: release.to_vec(),
        key_order_mismatches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversing_eight_extent_keys_counts_seven_mismatches_and_one_key_counts_zero() {
        let key = |offset: u64| {
            let mut key = [0u8; 24];
            key[8..16].copy_from_slice(&1u64.to_le_bytes());
            key[16..24].copy_from_slice(&offset.to_le_bytes());
            key
        };
        let ascending: Vec<[u8; 24]> = (0..8u64).map(|index| key(index * 32768)).collect();
        assert_eq!(count_key_order_mismatches(&ascending), 0);
        let mut reversed = ascending.clone();
        reversed.reverse();
        assert_eq!(
            count_key_order_mismatches(&reversed),
            7,
            "7 对相邻 key 全反"
        );
        assert_eq!(
            count_key_order_mismatches(&ascending[..1]),
            0,
            "第一个事务只有一个数据单元"
        );
        assert_eq!(
            count_key_order_mismatches(&[key(256 << 8), key(1)]),
            1,
            "按字段比不按字节比：小端字节序会把 65536 看成比 1 小"
        );
    }

    #[test]
    fn birth_sequences_count_per_tree_and_per_checkpoint_from_zero() {
        let mut sequences = BirthSequenceAllocator::default();
        let inode = TreeIdentifier(TREE_IDENTIFIER_INODE);
        assert_eq!(
            sequences.next(inode, CheckpointTxg(3), InstanceGeneration(1)),
            BirthSequence(0),
            "叶先发号"
        );
        assert_eq!(
            sequences.next(
                TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                CheckpointTxg(3),
                InstanceGeneration(1)
            ),
            BirthSequence(0)
        );
        assert_eq!(
            sequences.next(inode, CheckpointTxg(3), InstanceGeneration(1)),
            BirthSequence(1),
            "同树同 txg 的根排第二"
        );
        assert_eq!(
            sequences.next(inode, CheckpointTxg(4), InstanceGeneration(1)),
            BirthSequence(0),
            "换 txg 从 0 起"
        );
    }

    #[test]
    fn every_transaction_unit_names_its_class_tree_and_placement_rule() {
        for identity in TransactionUnit::IN_BUMP_ORDER {
            assert_eq!(identity.tag().len(), 2);
        }
        assert_eq!(
            TransactionUnit::InodeLeaf.placement(),
            PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned),
            "码 3 容器按数据单元那一档"
        );
        assert_eq!(TransactionUnit::Data.placement(), PlacementRule::UserData);
        assert_eq!(
            TransactionUnit::TreeTable.tree(),
            TreeIdentifier(0),
            "树表单元不属于任何一棵树"
        );
        assert_eq!(
            TransactionUnit::IN_BUMP_ORDER
                .iter()
                .map(|unit| unit.tag())
                .collect::<Vec<_>>(),
            ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8"]
        );
    }
}
