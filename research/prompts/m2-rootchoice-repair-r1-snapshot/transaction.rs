//! 事务层（里程碑步 3 / 步 4 / 步 5）：一个共享的提交状态机 + 一个封闭的提交步骤枚举
//! （D17（实现分层与第三方管道） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
//! 三条路径都从同一个枚举走：取号 = 逐盘一次系统配置槽写 + 一道屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）；
//! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
//! 第一个事务（D16（发布语义） 已定项 7）= 单元写 × 8 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换。
//! 覆盖写（里程碑「第二个事务」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、六个提交内生块与树表 COW 出新版本，
//! 被换下的八个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
//! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。

use std::collections::BTreeMap;

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
    ALLOCATION_RECORD_KEY_BYTES, DATA_UNIT_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES,
    FIRST_TRANSACTION_TXG, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, INSTANCE_ROW_BYTES,
    JOURNAL_NAMED_ENTRIES_PER_RECORD, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use crate::allocator::{
    AllocationRecord, Placement, PlacementRefusal, PoolAllocator, UnitFootprint,
};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::inode_tree::{
    write_records_into_leaf_containers, InodeLeafContainer, InodeLeafContainerIndexInTree,
    InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
};
use crate::journal::{back_chain_of, record_offset, JournalRecord, NamedUnit};
use crate::make_filesystem::{
    location_entries, MakeFilesystemParameters, MKFS_INSTANCE_GENERATION, TREE_TABLE_KEY_WIDTH,
};
use crate::pointer::{
    slot_shared_by_both_location_entries, BirthSequence, DataPointer,
    LocationEntriesOnDifferentSlots, LocationEntry, NodePointer, PointerHead,
};
use crate::records::{
    build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
    mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key, node_key_tail,
    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
    ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, STATISTIC_ALLOCATED_BYTES,
    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
    STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
    STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
    STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
};
use crate::recovery::{highest_root_instance, verified_system_configuration_slots};
use crate::root_record::RootRecord;
use crate::root_ring::{slot_offset, target_for_publish};
use crate::system_configuration::{
    SystemConfiguration, SystemImmutableConfiguration, SystemMutableConfiguration,
    SystemRuntimeConfiguration, SystemRuntimeQuantities,
};
use crate::unit::{
    build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
    index_node_entry_capacity, parse_index_node, unit_filesystem_identifier, DataUnitIdentity,
    PackedIdentity, WriteOrder, PACKED_TYPE_INSTANCE_TABLE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED,
};
use crate::write_accounting::{WritesByStructureKind, WrittenStructureKind};
use crate::write_request_split::split_sequential_write_into_one_unit_transactions;

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
        /// 这个单元的角色：只供按结构种类记账（增补 1），不影响写到哪、怎么写。
        identity: TransactionUnit,
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
    /// 每盘一次系统配置槽原地覆写：世代号 = 这块盘两槽里自证过的最大世代号 + 1，槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16，逐盘计）。
    RotateSystemConfigurationSlots {
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
    /// 这个写入口交给设备、设备报了成功的写，按结构种类累计（每块盘一次写调用算一次）；一次发布的账是发布前后两次快照之差。
    writes_by_structure_kind: WritesByStructureKind,
    /// 落盘阶段中途失败的发布各自已记的写，一次失败一份，按失败的先后排（增补 2 第 20b 行）。成功发布的账是两次快照之差、只在成功路径上取，
    /// 失败那次落盘的写不属于任何一次成功发布 ⇒ 不交出去就与设备一层的合计对不上。另立一份、不并进任何一次成功发布的账。
    writes_of_failed_publishes: Vec<WritesByStructureKind>,
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
            writes_by_structure_kind: WritesByStructureKind::NOTHING_WRITTEN,
            writes_of_failed_publishes: Vec::new(),
        }
    }
}

impl<Device: BlockDevice> PoolWriter<'_, Device> {
    pub fn perform(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
        if !matches!(step, CommitStep::Barrier) {
            self.has_writes_since_barrier = true;
        }
        match step {
            CommitStep::WriteUnitToEveryDevice {
                slot,
                unit,
                identity,
            } => {
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(slot.to_device_offset(), unit, WriteDurability::Plain)?;
                    let kind = WrittenStructureKind::of_unit(identity);
                    self.writes_by_structure_kind.count_write_call(kind, unit);
                }
            }
            CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {
                let offset = record_offset(counter, self.parameters.geometry.journal_ring_bytes);
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(offset, record, WriteDurability::Plain)?;
                    self.writes_by_structure_kind
                        .count_write_call(WrittenStructureKind::JournalRecord, record);
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
                self.writes_by_structure_kind
                    .count_write_call(WrittenStructureKind::RootSlot, root_slot);
            }
            CommitStep::RotateSystemConfigurationSlots {
                journal_tail,
                journal_instance,
            } => {
                for index in 0..self.devices.len() {
                    self.write_system_configuration_slot(index, journal_tail, journal_instance)?;
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

    /// 一块盘写一次系统配置槽：世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2。
    fn write_system_configuration_slot(
        &mut self,
        index: usize,
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    ) -> Result<(), BlockDeviceError> {
        let spacing = u64::from(self.parameters.geometry.fixed_structure_slot_spacing);
        let identity = self.devices[index].0;
        let slot_generation = verified_system_configuration_slots(
            &*self.devices,
            identity,
            spacing,
            &self.parameters.filesystem_identifier,
        )
        .iter()
        .map(|system_configuration| system_configuration.quantities.slot_generation)
        .max()
        .unwrap_or(0)
            + 1;
        let system_configuration = SystemConfiguration {
            immutable: SystemImmutableConfiguration {
                filesystem_identifier: self.parameters.filesystem_identifier,
                this_device: identity,
                device_count: u32::try_from(self.devices.len()).expect("设备数"),
                region_devices: self.parameters.region_devices,
                sizes: self.parameters.geometry,
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation,
                journal_tail,
                journal_instance,
            },
        };
        self.has_writes_since_barrier = true;
        let slot_bytes = system_configuration.to_slot();
        self.devices[index].1.write_at(
            DeviceOffsetInBytes(
                (slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,
            ),
            &slot_bytes,
            WriteDurability::Plain,
        )?;
        self.writes_by_structure_kind
            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);
        Ok(())
    }

    /// 落盘阶段中途失败的那次发布已记的写：落盘开始时的快照到此刻的差，记成失败账里的一份（增补 2 第 20b 行）。
    /// 只有落盘那几步（写单元、屏障、journal 记录、根槽、系统配置槽）里失败的发布才记，落盘阶段一次失败记一份，哪怕这一阶段
    /// 一个写都没记上就失败（那一份是空的）——份数就是这个写入口上落盘阶段失败过几次发布；准入、释放判定、分配、装单元这些
    /// 落盘之前的步骤里失败的发布一个写都没发，不记。
    fn count_failed_publish(&mut self, writes_before_this_publish: &WritesByStructureKind) {
        let written_before_the_failure = self
            .writes_by_structure_kind
            .since(writes_before_this_publish);
        self.writes_of_failed_publishes
            .push(written_before_the_failure);
    }

    /// 这个写入口上落盘阶段中途失败的发布各自已记的写，按失败的先后排。合计对账时它与每一次成功发布的账相加，才等于设备一层记下的写。
    #[must_use]
    pub fn writes_of_failed_publishes(&self) -> &[WritesByStructureKind] {
        &self.writes_of_failed_publishes
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
            if let Err(rollback_error) =
                self.write_system_configuration_slot(index, 0, previous_instance)
            {
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
    /// 第一块盘的取号写就报错了：没有写出过带新号的系统配置，没有要回卷的。
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

/// 每块盘两槽里全部自证过（fsid 与本池相同）的系统配置中最大的实例代号；一份都没有时是 mkfs 的 0。取号回卷时写回的旧号也是它。
fn highest_system_configuration_instance<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> InstanceGeneration {
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let filesystem_identifier = pool.parameters.filesystem_identifier;
    pool.devices
        .iter()
        .flat_map(|(identity, _)| {
            verified_system_configuration_slots(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            )
        })
        .map(|system_configuration| system_configuration.quantities.journal_instance)
        .max()
        .unwrap_or(MKFS_INSTANCE_GENERATION)
}

/// 取号会取到的号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的实例代号) + 1（只读盘、不写）：可写挂载要在取号之前
/// 按它判这次要写的行区间（没有文件版本的一版上要写行就在任何写之前拒绝），`acquire_instance` 取的也是它。
#[must_use]
pub fn instance_generation_to_acquire<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> InstanceGeneration {
    let highest_root = highest_root_instance(
        &*pool.devices,
        &pool.parameters.region_devices,
        &pool.parameters.geometry,
        &pool.parameters.filesystem_identifier,
    )
    .unwrap_or(MKFS_INSTANCE_GENERATION);
    InstanceGeneration(
        highest_system_configuration_instance(pool)
            .max(highest_root)
            .0
            + 1,
    )
}

/// 取号（D23（journal 的角色与格式） 已定项 16、D18（块里携带什么信息） 已定项 11；2026-09-14 用户定案，
/// C322（取号那一步的屏障怎么放没有条款） 三轮三方）：新号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的
/// 实例代号) + 1；逐盘写一次系统配置槽（世代号逐盘 +1）；两写之后一道屏障，屏障完成才把新号交出去，于是本实例的第一个
/// 非系统配置写一定排在它之后。首次挂载路径上它就是暖机开场那道：暖机那道屏障前面没有写，写入口不再发第二次，录制流与设备
/// 收到的 FLUSH 数都与只发一道时相同。任一块盘的取号写或那道屏障报错 ⇒ 已写出的那几份回卷成旧代号（取号之前全部自证过的
/// 槽中最大的实例代号），报失败。⚠️ D18（块里携带什么信息） 已定项 11 的「重试到 T_retry 用尽才算失败」这里没有做：
/// 块设备报的第一个错就判失败（写路径上的重试全仓都还没有）。
pub fn acquire_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
) -> Result<InstanceGeneration, AcquisitionFailed> {
    let instance = instance_generation_to_acquire(pool);
    write_acquired_instance(pool, instance)
}

/// 按调用方判定时算出的号取号没做成。
#[derive(Debug)]
pub enum ExpectedInstanceAcquisitionFailed {
    /// 写之前重算出的号与调用方判定时算出的号不同（两次读系统配置之间一次瞬时读错就够：读错的槽当作没有）：一个字节都没写。
    InstanceGenerationChangedBeforeWrite {
        expected: InstanceGeneration,
        recomputed: InstanceGeneration,
    },
    Acquisition(AcquisitionFailed),
}

/// 取号，号用调用方判定时算出的那一个（`instance_generation_to_acquire`）：写之前再算一遍，对不上就不写、报错返回——可写挂载在取号之前
/// 按这个号判要写的行区间，号在判定与取号之间变了，判定就作废（m2-emptypool-nonempty-r1 云端攻方腿 Z2：两块盘第 2 次读系统配置槽 0
/// 各报一次瞬时读错，判定看到号 1 放行、取号读到号 2 写进两盘）。
///
/// # Errors
/// 重算的号不同 ⇒ `InstanceGenerationChangedBeforeWrite`；取号写或屏障报错 ⇒ `Acquisition`（与 `acquire_instance` 同）。
pub fn acquire_expected_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    expected: InstanceGeneration,
) -> Result<InstanceGeneration, ExpectedInstanceAcquisitionFailed> {
    let recomputed = instance_generation_to_acquire(pool);
    if recomputed != expected {
        return Err(
            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                expected,
                recomputed,
            },
        );
    }
    write_acquired_instance(pool, recomputed)
        .map_err(ExpectedInstanceAcquisitionFailed::Acquisition)
}

/// 取号的写那一半：逐盘写一次系统配置槽，两写之后一道屏障；任一报错就把已写出的回卷成旧代号。
fn write_acquired_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    instance: InstanceGeneration,
) -> Result<InstanceGeneration, AcquisitionFailed> {
    let previous_instance = highest_system_configuration_instance(pool);
    let mut written = Vec::new();
    for index in 0..pool.devices.len() {
        if let Err(cause) = pool.write_system_configuration_slot(index, 0, instance) {
            return Err(pool.roll_back_acquisition(&written, previous_instance, cause));
        }
        written.push(index);
    }
    if let Err(cause) = pool.perform(CommitStep::Barrier) {
        return Err(pool.roll_back_acquisition(&written, previous_instance, cause));
    }
    Ok(instance)
}

/// 暖机写出的东西：两代根、两条空记录、最后一条记录的字节（下一条的反向链要罩它）、每次空发布按结构种类的写。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmUpOutput {
    pub roots: Vec<RootRecord>,
    pub records: Vec<JournalRecord>,
    pub last_record_bytes: Vec<u8>,
    /// 与 `roots` 同序，一次空发布一份。
    pub writes: Vec<WritesByStructureKind>,
}

/// mkfs 之后环里一条记录都没有：接着它数的 jsn 从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条，与可写挂载「环里没有记录时从 1 起」同一个起点）。
const LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM: u64 = 0;

/// 暖机（D16（发布语义） 已定项 8）：mkfs 之后第一次可写挂载的两次空发布，checkpoint_txg 走 1、2；环在 mkfs 之后是空的，
/// jsn 从 1 起。其余见 `warm_up_after_journal_counter`。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn warm_up<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
) -> Result<WarmUpOutput, BlockDeviceError> {
    warm_up_after_journal_counter(
        pool,
        genesis,
        instance,
        LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM,
    )
}

/// 暖机的两次空发布，接在环里 jsn 计数器为 `last_journal_counter` 的那条记录之后：checkpoint_txg 照格式常量走 1、2
/// （`WARM_UP_EMPTY_PUBLISHES`，D16（发布语义） 已定项 8），jsn 与系统配置里的 tail 按记录号接着数、不取 txg
/// （D23（journal 的角色与格式） 已定项 18：tail 存 jsn 的 48 位计数器；已定项 14 第 3 条：计数器全池接着走；C366（暖机路径把 txg 写进计数器与 tail））。
/// 空记录不点名任何单元、事务号 0、提交标记 1、新根段照 mkfs 的根（D16（发布语义） 已定项 9 / D23（journal 的角色与格式） 已定项 19），
/// 根记录只改 checkpoint_txg 与实例代号。今天只有 `warm_up` 调它，环是空的、两个量按构造相等；给不相等的起点才分得出它们。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn warm_up_after_journal_counter<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
    last_journal_counter: u64,
) -> Result<WarmUpOutput, BlockDeviceError> {
    let mut roots = Vec::new();
    let mut records = Vec::new();
    let mut writes = Vec::new();
    let mut previous_record_bytes: Option<Vec<u8>> = None;
    let mut previous_counter = last_journal_counter;
    for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {
        let output = publish_without_units(
            pool,
            genesis,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(txg_number),
                counter: previous_counter + 1,
                instance,
                back_chain: previous_record_bytes.as_deref().map_or(0, back_chain_of),
                rollback_floor: genesis.rollback_floor,
            },
        )?;
        previous_counter = output.record.counter;
        roots.push(output.root);
        records.push(output.record);
        writes.push(output.writes);
        previous_record_bytes = Some(output.record_bytes);
    }
    Ok(WarmUpOutput {
        roots,
        records,
        last_record_bytes: previous_record_bytes.expect("暖机至少一次"),
        writes,
    })
}

/// 一次零单元发布的参数（D16（发布语义） 已定项 9：树表 0 条时空发布写零个单元）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroUnitPublishPlan {
    pub txg: CheckpointTxg,
    /// jsn 计数器，全池接着走（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    pub counter: u64,
    pub instance: InstanceGeneration,
    /// 反向链：本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub rollback_floor: CheckpointTxg,
}

/// 一次零单元发布写出的东西：根、记录、记录的字节（下一条的反向链要罩它）、按结构种类的写。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroUnitPublishOutput {
    pub root: RootRecord,
    pub record: JournalRecord,
    pub record_bytes: Vec<u8>,
    pub writes: WritesByStructureKind,
}

/// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号与回退下界。
/// 第一次可写挂载的暖机与「只做过 mkfs 的池」上的可写挂载都走它（第一个事务的字节不变）。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn publish_without_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> Result<ZeroUnitPublishOutput, BlockDeviceError> {
    let record = JournalRecord {
        instance: plan.instance,
        counter: plan.counter,
        checkpoint_txg: plan.txg,
        transaction: 0,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
        new_tree_table: previous_root.tree_table,
        new_mapping_root: previous_root.mapping_root,
        new_tree_identifier_watermark: previous_root.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named: Vec::new(),
    };
    let record_bytes = record.to_bytes();
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance: plan.instance,
        checkpoint_txg: plan.txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: previous_root.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: previous_root.instance_table,
        mapping_root: previous_root.mapping_root,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：四步收在一个闭包里，失败在这里记账再把错原样交回。
    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: plan.counter,
            record: &record_bytes,
        })?;
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {
            checkpoint_txg: plan.txg,
            root_slot: &root_slot,
        })?;
        writer.perform(CommitStep::RotateSystemConfigurationSlots {
            journal_tail: plan.counter,
            journal_instance: plan.instance,
        })
    };
    if let Err(cause) = persist(pool) {
        pool.count_failed_publish(&writes_before_this_publish);
        return Err(cause);
    }
    Ok(ZeroUnitPublishOutput {
        root,
        record,
        record_bytes,
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_publish),
    })
}

/// 池里现行的那一版：还没发布过文件版本（树表 0 条，零单元发布写出的根），或带文件的一版。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有几个这样的值、不进集合，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
pub enum PoolVersion {
    WithoutFile(ZeroUnitPublishOutput),
    WithFile(TransactionOutput),
}

impl PoolVersion {
    #[must_use]
    pub fn root(&self) -> &RootRecord {
        match self {
            PoolVersion::WithoutFile(version) => &version.root,
            PoolVersion::WithFile(version) => &version.root,
        }
    }
    #[must_use]
    pub fn record(&self) -> &JournalRecord {
        match self {
            PoolVersion::WithoutFile(version) => &version.record,
            PoolVersion::WithFile(version) => &version.record,
        }
    }
    #[must_use]
    pub fn record_bytes(&self) -> &[u8] {
        match self {
            PoolVersion::WithoutFile(version) => &version.record_bytes,
            PoolVersion::WithFile(version) => &version.record_bytes,
        }
    }
    /// 带文件的那一版；还没发布过文件版本时 `None`。
    #[must_use]
    pub fn file_version(&self) -> Option<&TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) => Some(version),
        }
    }
    #[must_use]
    pub fn into_file_version(self) -> Option<TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) => Some(version),
        }
    }
}

/// 第一个事务写出的八个单元，声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
/// 也是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）与字节表七的 t1..t8。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionUnit {
    Data,
    ExtentRoot,
    /// inode 树的一片叶容器（码 3 打包记录类型 2，一容器 233 条 140 字节记录，D8（核心索引结构） 已定项 6）。
    /// 带的是它在树里的叶序，不是容器号：一棵树可以有好几片，角色按叶序分开
    /// （落点、映射 key、点名项、按结构种类记的账都按角色走，共用一个角色就分不开两片叶）。
    InodeLeafContainer(InodeLeafContainerIndexInTree),
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
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
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

    /// 字节表七的步号。字节表七只登记了一片叶容器那一档的 `t3`（第一个事务的 inode 树只有一片叶）；
    /// 第二片起写成 `t3+叶序`，那是这一版给多容器起的名字，只出现在报错与用例消息里，不进字节表、不进段序列登记表。
    #[must_use]
    pub fn tag(self) -> String {
        match self {
            TransactionUnit::Data => "t1".to_string(),
            TransactionUnit::ExtentRoot => "t2".to_string(),
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST) => {
                "t3".to_string()
            }
            TransactionUnit::InodeLeafContainer(index) => format!("t3+{}", index.0),
            TransactionUnit::InodeRoot => "t4".to_string(),
            TransactionUnit::AllocationTree => "t5".to_string(),
            TransactionUnit::AccountingTree => "t6".to_string(),
            TransactionUnit::MappingTree => "t7".to_string(),
            TransactionUnit::TreeTable => "t8".to_string(),
            TransactionUnit::InstanceTable => "ti".to_string(),
        }
    }

    #[must_use]
    pub const fn unit_class(self) -> u8 {
        match self {
            TransactionUnit::Data => UNIT_CLASS_DATA,
            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InstanceTable => {
                UNIT_CLASS_PACKED
            }
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
            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => {
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
            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InstanceTable => {
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
    /// 这一版第一个文件那条 inode 记录（覆盖写要接着它的对象出生代）。
    pub inode_record: InodeRecord,
    /// 这一版 inode 树里的全部叶容器，左起按 key 序：每片的身份、它装的记录、这一版它的指针
    /// （这次重写的是新落点，没重写的照抄上一版）。下一次发布按它算记录落在哪一片、要不要分裂。
    pub inode_leaf_containers: Vec<InodeLeafContainerVersion>,
    /// 六个进映射的单元各自的映射 key（映射树与树表豁免）；下一次覆盖写按它经映射取落点释放（D19（块指针的结构与宽度预算） 已定项 5 第 1 条）。
    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；第一个事务为空）。
    pub released: Vec<Placement>,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，第一个事务恒 0。
    pub key_order_mismatches: u64,
    /// 这次发布交给设备的写，按结构种类（增补 1）；从盘上重建的版本不是这个进程写出的，是空账。
    pub writes: WritesByStructureKind,
    /// 这个实例到这次发布为止用过的最大事务号（空发布写 0、不推进它）。下一次发布取它加一，
    /// 而不是取上一条记录上的事务号加一（D23（journal 的角色与格式） 已定项 7：事务号按实例计数、从 1 起）。
    pub highest_transaction_number_in_this_instance: u64,
}

/// 一版里的一片 inode 叶容器：装了什么、这一版它的指针在哪。字节不放这里，放 `TransactionOutput::units`
/// 里那个角色的单元（重写的是新装的，照抄的是上一版那一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InodeLeafContainerVersion {
    pub contents: InodeLeafContainer,
    pub pointer: NodePointer,
}

impl TransactionOutput {
    /// 这一版 inode 树里的叶容器内容，交给 `crate::inode_tree` 算下一次发布的落法。
    #[must_use]
    pub fn inode_leaf_container_contents(&self) -> Vec<InodeLeafContainer> {
        self.inode_leaf_containers
            .iter()
            .map(|container| container.contents.clone())
            .collect()
    }

    /// 记账里这一版的 inode 号水位（下一个可用号，D5（快照 / 空间记账机制） 已定项 4 第 12 项）。
    /// 读的是这一版真写进记账树的那一行，不重算——下一次发布接着它发号，I-9.6（水位大于两处最大号） 在盘上判它与树里最大 key 的关系。
    ///
    /// # Panics
    /// 这一版的记账行里没有 inode 号水位那一行。两条来路各自保证它在：这个进程发布出来的版本由 `publish_version` 写它
    /// （池级三行之一，每次发布都重写）；从盘上重建出来的版本由 `recovery::rebuild_version` 在交出这一版之前判过
    /// （盘上的统计量标签可以是任何值，那一判返回 `InodeNumberWatermarkRowMissingFromTheAccountingTree`）。
    #[must_use]
    pub fn inode_number_watermark(&self) -> u64 {
        self.accounting_entries
            .iter()
            .find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
            .expect(
                "inode 号水位那一行在：发布路径每次都写它（D5 已定项 8 的池级三行之一），\
                 从盘上重建的版本在 rebuild_version 的边界判上验过（盘上的标签可以是任何值）",
            )
            .value
    }

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
/// **那三样对池里每块盘各核一遍**：`PoolAllocator::release` 对每块盘都要求一条对得上的记录，而两块盘的分配记录树
/// 对不对称是盘上读来的、不是不变量（panic 面普查 R10；此前只核 `locations[0].device` 那一块）。
///
/// # Errors
/// 查不到 key、或上一版的映射节点解不开 ⇒ `ReleaseNotInMapping`；映射节点里有条目窄于字段表 ⇒
/// `MappingEntryNarrowerThanItsFieldTable`；查出来的两条位置条目槽号不等 ⇒ `ReleaseTargetLocationsOnDifferentSlots`；
/// 槽在某块盘的分配记录里没条目 ⇒ `ReleaseTargetNotAllocated`；某块盘的条目已是已释放 ⇒ `ReleaseTargetAlreadyReleased`；
/// 某块盘的记录跨度与种类不符 ⇒ `ReleaseSpanMismatch`。三样都带着是哪块盘。
pub fn placements_to_release_via_mapping(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    roles: &[TransactionUnit],
) -> Result<Vec<Placement>, PublishError> {
    let mapping_node_bytes = &previous.unit(TransactionUnit::MappingTree).bytes;
    let mut placements = Vec::new();
    for identity in roles.iter().copied() {
        // 这次发布新建的叶容器（末尾分裂出来的右半）在上一版里没有落点 ⇒ 没有东西要释放。
        // 判据取上一版的容器数，不取「映射里查不到」：查不到要报 `ReleaseNotInMapping`，那是另一件事。
        if let TransactionUnit::InodeLeafContainer(index) = identity {
            if index.position() >= previous.inode_leaf_containers.len() {
                continue;
            }
        }
        let locations = match identity {
            TransactionUnit::Data
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTree => {
                let (_, key) = previous
                    .mapped_units
                    .iter()
                    .find(|(mapped, _)| *mapped == identity)
                    .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树）");
                match mapping_locations_for_key(mapping_node_bytes, key) {
                    MappingLookup::Found(locations) => locations,
                    MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
                        return Err(PublishError::ReleaseNotInMapping { unit: identity })
                    }
                    MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
                        return Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
                            unit: identity,
                            entry_bytes,
                            field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
                        })
                    }
                }
            }
            TransactionUnit::MappingTree => previous.root.mapping_root.locations,
            TransactionUnit::TreeTable => previous.root.tree_table.locations,
            TransactionUnit::InstanceTable => previous.root.instance_table.locations,
        };
        let slot = slot_shared_by_both_location_entries(&locations).map_err(|disagreement| {
            PublishError::ReleaseTargetLocationsOnDifferentSlots {
                unit: identity,
                disagreement,
            }
        })?;
        // **池里每块盘都核一遍，不只核 `locations[0].device` 那一块**：`PoolAllocator::release` 对每块盘都要求
        // 一条在册、未释放、跨度对得上的记录，而两块盘的分配记录树对不对称是盘上读来的、不是不变量
        // （panic 面普查 R10：盘 0 有记录、盘 1 没有 / 已释放 / 跨度不同时，此前这三样在 `release` 的
        // `expect` 与两条断言上 panic）。这道判与那三样逐条对齐：少判一样，那一样就还在断言上。
        // 跨度取核过的那个值，每块盘都要与种类该有的跨度相同，所以逐盘核完取哪一块都一样。
        for device in allocator.devices.iter().map(|device_map| device_map.device) {
            let record = allocator.record_for(device, slot).ok_or(
                PublishError::ReleaseTargetNotAllocated {
                    unit: identity,
                    device,
                    slot,
                },
            )?;
            if record.is_released {
                return Err(PublishError::ReleaseTargetAlreadyReleased {
                    unit: identity,
                    device,
                    slot,
                });
            }
            let recorded_span = u64::from(record.span_slots);
            if recorded_span != identity.span_slots() {
                return Err(PublishError::ReleaseSpanMismatch {
                    unit: identity,
                    device,
                    slot,
                    recorded_span,
                    expected_span: identity.span_slots(),
                });
            }
        }
        placements.push(Placement {
            slot,
            // 逐盘核过「记录的跨度 = 这个种类该有的跨度」，两者相等，取种类那一个。
            span: identity.span_slots(),
        });
    }
    Ok(placements)
}

/// 在一个映射节点里按 key 查落点的结果。三个成员按**调用方要做的决定**分：交出落点、报「不在映射」、报「条目切不动」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingLookup {
    /// 这把 key 的两条位置条目。
    Found([LocationEntry; 2]),
    /// 节点解不开，或者解得开、里面没有这把 key。两样让调用方做同一个决定（报「不在映射」、一个落点都不释放），
    /// 所以不分成两个成员（`code-discipline.md`「错误」：成员按调用方要做的决定分，不按底层原因逐个展开）。
    NodeMalformedOrNoEntryWithThisKey,
    /// 节点自述的条目宽窄于映射条目的字段表（55）：盘上读来的宽度，不是不变量（panic 面普查 R2）。
    EntryNarrowerThanItsFieldTable { entry_bytes: usize },
}

/// 在一个映射节点里按 key 查落点。
#[must_use]
pub fn mapping_locations_for_key(mapping_node_bytes: &[u8], key: &[u8]) -> MappingLookup {
    let Ok(node) = parse_index_node(mapping_node_bytes) else {
        return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
    };
    let mut found = None;
    for entry in &node.entries {
        let Some((candidate, locations)) = parse_mapping_entry(entry) else {
            return MappingLookup::EntryNarrowerThanItsFieldTable {
                entry_bytes: entry.len(),
            };
        };
        if candidate == key {
            found = Some(locations);
        }
    }
    match found {
        Some(locations) => MappingLookup::Found(locations),
        None => MappingLookup::NodeMalformedOrNoEntryWithThisKey,
    }
}

/// 发布能出的错：分配器给不出落点（哪一个单元、为什么）、两棵单节点树装不下、内容装不下、释放判定路径对不上，或底层块设备错。
#[derive(Debug)]
pub enum PublishError {
    /// 分配器拒了这个单元的落点，`refusal` 原样带着分配器的原因：每块盘上都没有合政策的落点（容量不够）是 `NoFreeSlotOnAnyDevice`；
    /// 小盘写满、各盘的落点不一致是第一版不支持的池形状（D2（RAID 条带策略） 已定项 2 ⚠️、已定项 10；D3（空间分配） 已定项 8），
    /// 不是容量不够。此前这四种一律报成 `NoSpaceFor`（C368（分配器落点只看盘 0，盘不等大时断言失败） 仍欠的那一半；
    /// 增补 3 第 2 件代码三方第一轮判决第三节第 2 条：胶水把它们都当成单元区墙）。
    PlacementRefused {
        unit: TransactionUnit,
        refusal: PlacementRefusal,
    },
    /// 分配记录树第一版只有一个节点，这次发布之后的记录数装不下（每次发布每盘加 8 条、释放只改写不删）。
    AllocationRecordsExceedOneNode {
        records: usize,
        capacity: usize,
    },
    /// 记账树第一版只有一个节点，这次发布要写的记账行装不下：行数 = 池级 3 行 + 每块盘 6 行，只随盘数变（477 条的节点在第 80 块盘时装不下）。
    AccountingEntriesExceedOneNode {
        entries: usize,
        capacity: usize,
    },
    /// 释放判定路径在上一版的映射里查不到这个单元（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：不按提示释放）。
    ReleaseNotInMapping {
        unit: TransactionUnit,
    },
    /// 映射查出来的落点在**这块盘**的分配记录里没有条目。`device` 带着是哪块盘：`PoolAllocator::release` 对池里
    /// 每块盘都要求一条对得上的记录，而两块盘的分配记录树对不对称是**盘上读来的**、不是不变量（panic 面普查 R10）。
    ReleaseTargetNotAllocated {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 映射查出来的落点在**这块盘**上已经是已释放的（同一个落点释放两次）。
    ReleaseTargetAlreadyReleased {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 要释放的这个单元，上一版那条指针的两条位置条目落在不同的槽：槽号是**盘上读来的 6 字节**、可以是任何值，
    /// 而第一版的布局是两盘同槽（一个单元整个落在一列上、两盘各一份，D2（RAID 条带策略） 已定项 10 每个副本一条位置条目）
    /// ⇒ 这是一份坏镜像，不是我们的不变量被破坏，报错、不断言（panic 面普查 R5）。拒的是「盘上这条指针的两条位置条目不同槽」，
    /// 与 `PlacementRefusal` 那两条「各盘算出来的落点不同、第一版不支持」不是同一件事（那两条说的是写侧算出来的落点）。
    /// 与释放判定路径的其它几种一样：一个落点都不释放，在动分配器之前返回，盘上逐字节不变。
    ReleaseTargetLocationsOnDifferentSlots {
        unit: TransactionUnit,
        disagreement: LocationEntriesOnDifferentSlots,
    },
    /// **这块盘**的分配记录里的跨度与这个单元种类该有的跨度不符（映射给槽、记录给跨度，两边对不上）。
    ReleaseSpanMismatch {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
        recorded_span: u64,
        expected_span: u64,
    },
    /// 上一版的映射节点里有一条条目窄于映射条目的字段表（55）：条目宽是映射树根自述的一个**盘上字段**，
    /// `parse_index_node` 只判了它 ≥ key 宽 27，切到偏移 55 之前要判一次（panic 面普查 R2）。
    /// 与 `ReleaseNotInMapping` 分开报：那一条说的是「节点好、里面没有这把 key」，这一条说的是「节点里的条目切不动」。
    MappingEntryNarrowerThanItsFieldTable {
        unit: TransactionUnit,
        entry_bytes: usize,
        field_table_bytes: usize,
    },
    /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）。
    ContentExceedsDataUnit {
        bytes: usize,
        capacity: usize,
    },
    /// 这次顺序写请求按切分纪律切出不止一个数据单元，而多单元写要的条款仓里还没定：在任何落盘动作之前返回，
    /// 盘上逐字节不变。`data_units` 是切分算出的单元数（= 事务数 = 记录数），`undecided` 逐条说明哪条条款没定。
    MoreThanOneDataUnitNeedsUndecidedClauses {
        data_units: u64,
        undecided: &'static [UndecidedClauseBlockingMoreThanOneDataUnit],
    },
    /// 这次发布要往 inode 树里写的记录，落法要的条款仓里还没定（[`InodeTreeWriteRefusal`] 的三格）：
    /// 在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
    InodeTreeWriteRefused(InodeTreeWriteRefusal),
    /// 这次发布要点名的单元多于一条 journal 记录装得下的 67 项（D23（journal 的角色与格式） 已定项 12 的 4 KiB 定长记录 +
    /// 已定项 17 的 56 字节点名项）。装不下就要写成多条记录，而**一次发布写多条记录时共享的提交内生块在哪一条记录里点名**
    /// 仓里还没定——与并行线一被拒的那一条是同一格
    /// （[`UndecidedClauseBlockingMoreThanOneDataUnit::RecordThatNamesTheSharedCommitGeneratedUnits`]）⇒
    /// 在任何落盘动作之前返回，盘上逐字节不变。
    MoreNamedUnitsThanOneJournalRecordHolds {
        named_units: usize,
        capacity: usize,
        undecided: UndecidedClauseBlockingMoreThanOneDataUnit,
    },
    /// `publish_first_file` 写死 txg 3、jsn 3，而上一版的记录不是 txg 2、jsn 2（`None` = 那几个字节解不出本池的记录）：接上去会盖在别的发布上。
    FirstFileVersionNotRightAfterTheSecondWarmUp {
        previous_record: Option<(CheckpointTxg, u64)>,
    },
    BlockDevice(BlockDeviceError),
}

impl From<BlockDeviceError> for PublishError {
    fn from(error: BlockDeviceError) -> Self {
        PublishError::BlockDevice(error)
    }
}

impl From<InodeTreeWriteRefusal> for PublishError {
    fn from(refusal: InodeTreeWriteRefusal) -> Self {
        PublishError::InodeTreeWriteRefused(refusal)
    }
}

/// 一次写请求要跨多个数据单元时，仓里还没定、因此这一版不支持的条款。两条今天都没定，一起从
/// `publish_sequential_write` 交回；定下来一条就从 [`UndecidedClauseBlockingMoreThanOneDataUnit::STILL_UNDECIDED_TODAY`]
/// 里删一条，两条都删完之前多单元的顺序写一律在任何落盘动作之前被拒
/// （`.claude/agents/implementation-writer.md`：条款没写、要做设计判断的地方停在那一处，不自己定）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndecidedClauseBlockingMoreThanOneDataUnit {
    /// extent 叶记录 key 的 offset 段取文件字节偏移还是单元序号。D8（核心索引结构） 已定项 3 只写
    /// key = `(locality_id, inode, offset)`，没写 offset 的单位；`.claude/kb/layout/01-first-txn.md`
    /// 「四·二、extent 树」第一个事务那一格的值是 `(0, 1, 0)`，两种读法都是 0、分不开。第二个数据单元起
    /// 两种读法给出不同的 24 字节 key（字节偏移 32634 对单元序号 1）⇒ 不定案就写不出那 24 字节。
    /// 里程碑「第二个事务」并行线一逐字标「**预想** offset 字段是文件字节偏移……开工前核」。
    ExtentLeafKeyOffsetUnit,
    /// 一次发布里 N 个事务 N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、
    /// 记账树、中央映射树、树表单元）在哪一条记录的点名项里出现。D23（journal 的角色与格式） 已定项 17 定的是点名项
    /// 那 56 字节的字段表，不定哪些单元进哪一条记录；C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案
    /// 「一条记录装一个事务连同它的元数据」在共享的元数据上有两种读法（每条记录各点名一遍、只在最后一条点名）。
    /// 里程碑「第二个事务」并行线一验收标准逐字「这次发布共享的提交内生块在哪条记录里点名，开工前定（**预想**：只在最后一条）」。
    ///
    /// 并行线三（多个 inode）从另一边走到同一格：一次发布重写的单元多到一条 4 KiB 记录的 67 个点名项装不下时
    /// （[`PublishError::MoreNamedUnitsThanOneJournalRecordHolds`]），同样要先定这一条才写得出那几条记录。
    RecordThatNamesTheSharedCommitGeneratedUnits,
}

impl UndecidedClauseBlockingMoreThanOneDataUnit {
    /// 今天挡着多单元顺序写的全部条款。
    pub const STILL_UNDECIDED_TODAY: [UndecidedClauseBlockingMoreThanOneDataUnit; 2] = [
        UndecidedClauseBlockingMoreThanOneDataUnit::ExtentLeafKeyOffsetUnit,
        UndecidedClauseBlockingMoreThanOneDataUnit::RecordThatNamesTheSharedCommitGeneratedUnits,
    ];

    /// 这条没定的东西登记在哪：交回给人看的那一行。
    #[must_use]
    pub const fn registered_at(self) -> &'static str {
        match self {
            UndecidedClauseBlockingMoreThanOneDataUnit::ExtentLeafKeyOffsetUnit => {
                "D8（核心索引结构） 已定项 3 + .claude/kb/layout/01-first-txn.md「四·二、extent 树」：extent 叶记录 key 的 offset 段取文件字节偏移还是单元序号"
            }
            UndecidedClauseBlockingMoreThanOneDataUnit::RecordThatNamesTheSharedCommitGeneratedUnits => {
                "D23（journal 的角色与格式） 已定项 17 + C310（事务切分纪律与记录数口径打架）：一次发布里 N 条记录时共享的提交内生块在哪一条记录里点名"
            }
        }
    }
}

/// 出生序号：同一 (树, txg, 实例) 里每写出一个码 2 / 码 3 单元加 1，从 0 起（D19（块指针的结构与宽度预算） 已定项 9）。
/// 作用域是一次 checkpoint：`publish_admitted` 每次发布建一个，这次装的每个对象（`build_file_version_units`）与固定点结构都从它取号。
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

/// 这次发布要写的文件版本：数据单元与 extent 树根重写，这个对象的 inode 记录跟着换一版（它落在 inode 树的哪一片叶、
/// 要不要分裂由 `crate::inode_tree` 算）。没有它的发布（写行、暖机）这两个角色照抄上一版。
#[derive(Clone, Copy, Debug)]
pub struct FileVersionPlan<'content> {
    pub content: &'content [u8],
    pub write_time_seconds: u64,
    /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
    pub inode_object_birth: CheckpointTxg,
    /// 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）。
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
    /// 这个实例在这次发布之前用过的最大事务号。`publish_version` 拿它与 `transaction` 取大的存进
    /// `TransactionOutput`，下一次发布从那里加一。
    ///
    /// 为什么不直接取上一条记录的事务号加一：空发布在记录上写 0（已定项 19 ①），取「上一条记录 + 1」会让计数
    /// 退回 1，同一个实例的事务号重号，而 D23（journal 的角色与格式） 已定项 7 逐字要求「事务号按实例计数、从 1 起」，
    /// 并且实例表行的 W 能当精确前缀正是靠这条纪律。
    pub highest_transaction_number_before_this_publish: u64,
    pub instance: InstanceGeneration,
    /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub file: Option<FileVersionPlan<'content>>,
    /// 这次发布往 inode 树里新建的 inode，按 inode 号严格升序（建 N 个文件那一路，里程碑「第二个事务」并行线三）。
    /// 号由调用方按记账里的水位发（D5（快照 / 空间记账机制） 已定项 4 第 12 项），落在哪一片叶、要不要在末尾分裂
    /// 由 `crate::inode_tree` 在任何落盘动作之前算。别的发布给空的：文件版本那次只换它自己那条记录，写行与暖机不碰 inode 树。
    pub new_inode_records: &'content [InodeRecord],
    pub instance_table: InstanceTablePlan,
    /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
    pub tree_birth_txg: CheckpointTxg,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
}

/// 一次发布重写哪些角色，只由三件事定：这次写不写文件内容（数据单元 + extent 树根）、重写几片 inode 叶容器、
/// 实例表是重写还是照抄。计划的身份字段（新实例代号、表的字节、txg）都不进来——可写挂载要在取号之前算写行那次发布的准入，
/// 而那时新实例代号还没取（增补 2 第 20a 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishShape {
    pub rewrites_file_content: bool,
    /// 这次重写几片 inode 叶容器；0 = 这次一个字节都不碰 inode 树（连根都不重写）。
    pub rewritten_inode_leaf_containers: usize,
    pub rewrites_instance_table: bool,
}

impl PublishShape {
    /// 写行那次发布的形状：不写文件内容、不碰 inode 树、重写实例表（`publish_rows_on_file_version` 交给 `publish_version` 的那张计划同形）。
    pub const ROW_PUBLISH: PublishShape = PublishShape {
        rewrites_file_content: false,
        rewritten_inode_leaf_containers: 0,
        rewrites_instance_table: true,
    };

    /// 暖机那几次空发布的形状：不写文件内容、不碰 inode 树、实例表照抄（`publish_empty_after` 交给 `publish_version` 的那张计划同形，
    /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写四个固定点单元（D16（发布语义） 已定项 9）。
    pub const EMPTY_PUBLISH: PublishShape = PublishShape {
        rewrites_file_content: false,
        rewritten_inode_leaf_containers: 0,
        rewrites_instance_table: false,
    };

    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。
    ///
    /// ⚠️ **叶容器在这里按 0 起的连号排，不是它们在树里的真实叶序**：形状只答「几个角色」（准入按条数算，
    /// `publish_sequence_admission` 在取号之前要的就是这个数），答不了「改的是哪几片」——那要看上一版的树长什么样。
    /// 真实的角色清单由 [`PublishPlan::rewritten_roles`] 给。
    #[must_use]
    pub fn rewritten_roles(self) -> Vec<TransactionUnit> {
        let mut roles = Vec::new();
        if self.rewrites_file_content {
            roles.push(TransactionUnit::Data);
        }
        if self.rewrites_instance_table {
            roles.push(TransactionUnit::InstanceTable);
        }
        if self.rewrites_file_content {
            roles.push(TransactionUnit::ExtentRoot);
        }
        for position in 0..self.rewritten_inode_leaf_containers {
            roles.push(TransactionUnit::InodeLeafContainer(
                InodeLeafContainerIndexInTree::of_position(position),
            ));
        }
        if self.rewritten_inode_leaf_containers > 0 {
            roles.push(TransactionUnit::InodeRoot);
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

/// 一次发布在动分配器、动盘之前先算完的东西：这次之后 inode 树是哪几片叶容器、这次重写哪些角色（按 bump 次序）。
/// 只读——算它不发一个写、不动分配器，所以条款没写的那几格（`crate::inode_tree`）在这里交回时盘上逐字节不变。
#[derive(Clone, Debug)]
pub struct ResolvedPublish {
    pub inode_tree: InodeLeafContainersAfterThisPublish,
    pub rewritten_roles: Vec<TransactionUnit>,
}

impl ResolvedPublish {
    /// 这次发布的形状（准入按它数角色）。
    #[must_use]
    pub fn shape(&self) -> PublishShape {
        PublishShape {
            rewrites_file_content: self.rewritten_roles.contains(&TransactionUnit::Data),
            rewritten_inode_leaf_containers: self.inode_tree.rewritten.len(),
            rewrites_instance_table: self
                .rewritten_roles
                .contains(&TransactionUnit::InstanceTable),
        }
    }
}

impl PublishPlan<'_> {
    /// 这次发布的 inode 记录写入：文件版本那条（有文件版本时）在前，新建的那些在后。
    /// 号天然严格升序——第一个文件恒是 inode 1，新建的号从记账水位起发、都大于树里每一个 key。
    fn inode_record_writes(&self) -> Vec<InodeRecord> {
        let mut records = Vec::new();
        if let Some(file) = &self.file {
            records.push(inode_record_of_file_version(file));
        }
        records.extend_from_slice(self.new_inode_records);
        records
    }

    /// 把这张计划算成「这次之后 inode 树是什么样、这次重写哪些角色」。
    ///
    /// # Errors
    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）。
    pub fn resolve(
        &self,
        previous: Option<&TransactionOutput>,
    ) -> Result<ResolvedPublish, PublishError> {
        let containers_before = previous
            .map(TransactionOutput::inode_leaf_container_contents)
            .unwrap_or_default();
        let inode_tree = write_records_into_leaf_containers(
            &containers_before,
            &self.inode_record_writes(),
            self.txg,
            TreeIdentifier(TREE_IDENTIFIER_INODE),
        )?;
        let mut rewritten_roles = Vec::new();
        if self.file.is_some() {
            rewritten_roles.push(TransactionUnit::Data);
        }
        if matches!(self.instance_table, InstanceTablePlan::Rewrite(_)) {
            rewritten_roles.push(TransactionUnit::InstanceTable);
        }
        if self.file.is_some() {
            rewritten_roles.push(TransactionUnit::ExtentRoot);
        }
        for index in &inode_tree.rewritten {
            rewritten_roles.push(TransactionUnit::InodeLeafContainer(*index));
        }
        if !inode_tree.rewritten.is_empty() {
            rewritten_roles.push(TransactionUnit::InodeRoot);
        }
        rewritten_roles.extend([
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]);
        Ok(ResolvedPublish {
            inode_tree,
            rewritten_roles,
        })
    }
}

/// 文件版本那条 inode 记录（D8（核心索引结构） 已定项 6 的字段表）：size = 这次写的内容长度，
/// blocks = 一个 32 KiB 数据单元占的 512 字节块数——这一版一个文件对象恒一个数据单元（多单元那一档
/// 由 `publish_sequential_write` 在落盘之前拒掉）。
fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {
    InodeRecord {
        inode: FIRST_INODE_NUMBER,
        object_birth: file.inode_object_birth,
        size: u64::try_from(file.content.len()).expect("文件长度"),
        occupied_blocks_of_512_bytes: DATA_UNIT_BYTES / 512,
        change_count: file.change_count,
        write_time_seconds: file.write_time_seconds,
    }
}

/// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
///
/// # Errors
/// txg 与 jsn 写死 3（`FIRST_TRANSACTION_TXG`），只接得上 txg 2、jsn 2 的那一版（两次暖机之后）：`previous_record_bytes` 解不出本池的记录，
/// 或它的 checkpoint_txg、jsn 不是 2 ⇒ `FirstFileVersionNotRightAfterTheSecondWarmUp`，一个字节都不写（m2-emptypool-nonempty-r1 云端攻方腿
/// Z3-A：接在 txg 4 的暖机根后面照写 txg 3，盖在暖机根上、冷恢复读不到）。检查的是记录不是 `genesis`：mkfs 同一个进程里的第一个事务
/// 传的是 mkfs 的第 0 代根，它只供照抄实例表指针。其余同 `publish_version`。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    genesis: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
    let previous_record = JournalRecord::parse(
        previous_record_bytes,
        unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
    )
    .map(|record| (record.checkpoint_txg, record.counter));
    let follows_directly = previous_record.is_some_and(|(previous_txg, previous_counter)| {
        previous_txg.0 + 1 == FIRST_TRANSACTION_TXG && previous_counter + 1 == FIRST_TRANSACTION_TXG
    });
    if !follows_directly {
        return Err(PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record });
    }
    let (_, previous_counter) =
        previous_record.expect("follows_directly 为真时上一条记录解得出（is_some_and）");
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous_counter + 1,
            transaction: FIRST_TRANSACTION_NUMBER,
            // 这个实例的第一个事务：在它之前只有暖机那两次空发布，事务号都是 0。
            highest_transaction_number_before_this_publish: 0,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: txg,
                // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88 的字段表定义）：
                // 第一个事务这次发布的 txg，暖机之后是 3（`FIRST_TRANSACTION_TXG`，上面那句 `let txg` 取的也是它）。
                // 写 1 是 2026-09-18 之前的老样子（暖机把第一个事务从 txg 1 推到 3 时这一格没跟着改，增补 2 第 11 行）。
                // 这里不写 `txg.0`：`crates/mutations.tsv` 第 22 行按那串字面锚在 `publish_overwrite` 上，同一份文件里出现两次它就腐化。
                change_count: FIRST_TRANSACTION_TXG,
            }),
            // 第一个事务只建第一个文件那一个 inode：树是空的，这条记录建第一片容器（D8（核心索引结构） 已定项 6）。
            new_inode_records: &[],
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
/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、落点被拒（`PlacementRefused`，带分配器的原因）、装不下、
/// 块设备报错，都原样交回。
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
            transaction: previous.highest_transaction_number_in_this_instance + 1,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: previous.inode_record.object_birth,
                change_count: txg.0,
            }),
            // 覆盖写不建新 inode：只把第一个文件那条记录换一版，它落在原来那一片叶里（容器身份不变）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 顺序写一次写请求（里程碑「第二个事务」并行线一的写路径）：按切分纪律把请求切成若干一单元事务
/// （`crate::write_request_split`：D16（发布语义） 已定项 5 末段 + D23（journal 的角色与格式） 已定项 7 +
/// C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），N 个事务在同一次发布里写出 N 个数据单元、N 条记录。
///
/// 切出一个事务的那一档接在 `publish_overwrite` 上，走的是同一条发布路径、写出的字节一个不变
/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」：这里不另开一条发布路径）。
///
/// # Errors
/// 切出的事务多于一个 ⇒ `MoreThanOneDataUnitNeedsUndecidedClauses`：多单元写要的两条条款仓里还没定
/// （[`UndecidedClauseBlockingMoreThanOneDataUnit`]），在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
/// 一个事务的那一档：其余的错与 `publish_overwrite` 相同。
///
/// **同一份输入算两遍**：这里按内容长度切分判「几个单元」，`publish_version` 在任何写之前按同一个内容长度
/// 再判一次「装不装得进一个数据单元」（`ContentExceedsDataUnit`）；两道都在落盘之前，哪一道先拦都不落一个字节。
pub fn publish_sequential_write<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let content_length_in_bytes = u64::try_from(file.content.len()).expect("内容长度");
    let transactions = split_sequential_write_into_one_unit_transactions(
        content_length_in_bytes,
        previous.highest_transaction_number_in_this_instance + 1,
    );
    let data_units = u64::try_from(transactions.len()).expect("单元数");
    if data_units > 1 {
        return Err(PublishError::MoreThanOneDataUnitNeedsUndecidedClauses {
            data_units,
            undecided: &UndecidedClauseBlockingMoreThanOneDataUnit::STILL_UNDECIDED_TODAY,
        });
    }
    let only_transaction = transactions
        .first()
        .copied()
        .expect("切分至少给一个事务（长度 0 的请求也写一个声明长度 0 的数据单元）");
    assert_eq!(
        only_transaction.payload_of(file.content).len(),
        file.content.len(),
        "一个事务的那一档：这个事务写的就是整份内容"
    );
    publish_overwrite(pool, allocator, previous, file, instance)
}

/// 建 N 个 inode（里程碑「第二个事务」并行线三：多个文件、元数据按「个」量、没有目录）：一次发布把 N 条 inode 记录
/// 写进 inode 树——号从记账里的 inode 号水位起连着发（D5（快照 / 空间记账机制） 已定项 4 第 12 项），
/// 落在最右那片叶、满 233 条就在末尾分裂（D8（核心索引结构） 已定项 6，`crate::inode_tree`），
/// 这次发布之后水位加 N；数据单元与 extent 树不动（新建的 inode 一个数据单元都不占，`blocks` 写 0）。
///
/// 这次发布是**一个事务**：D16（发布语义） 已定项 5 的切分纪律逐字「一个事务最多写一个单元的用户数据」，
/// 建 inode 一个用户数据单元都不写 ⇒ 不切；一个事务一条 journal 记录（D23（journal 的角色与格式） 已定项 7）。
/// 「N 个创建要不要各算一个事务」仓里没有条款，这一版按「不切」走（报告里交主 agent 定）。
///
/// # Errors
/// 记录落法要的条款没定（中间插入、容器数超过一个根装得下的）⇒ `InodeTreeWriteRefused`；
/// 这次要点名的单元超过一条 journal 记录装得下的 67 项 ⇒ `MoreNamedUnitsThanOneJournalRecordHolds`；
/// 其余与 `publish_overwrite` 相同。三样都在任何落盘动作之前返回，盘上逐字节不变。
pub fn publish_new_inodes<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    new_inode_count: u64,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
    let first_new_inode_number = previous.inode_number_watermark();
    // 改动计数与事务号都先取成局部量：`crates/mutations.tsv` 有几条变异按 `change_count: txg.0,` 与
    // `transaction: previous.highest_transaction_number_in_this_instance + 1,` 这两串字面锚在 `publish_overwrite` 上，
    // 同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_first_file` 里那句注释是同一件事）。
    // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）；建出来那次就是这次发布。
    let change_count = txg.0;
    let transaction_number = previous.highest_transaction_number_in_this_instance + 1;
    let new_inode_records: Vec<InodeRecord> = (0..new_inode_count)
        .map(|offset_from_the_watermark| InodeRecord {
            inode: first_new_inode_number + offset_from_the_watermark,
            // 对象出生代 = 创建那次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 8）。
            object_birth: txg,
            size: 0,
            // 建出来还没写过数据：一个数据单元都不占。
            occupied_blocks_of_512_bytes: 0,
            change_count,
            write_time_seconds,
        })
        .collect();
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: transaction_number,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            // 文件内容不动：数据单元与 extent 树根照抄上一版。
            file: None,
            new_inode_records: &new_inode_records,
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 发布一版：先做准入（内容装得进一个数据单元，再加 `publish_admission` 的两条），再释放上一版被换下的角色的落点、分配、装单元、
/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
/// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
/// 落盘那几步里失败的，这次已记的写进写入口的失败账（增补 2 第 20b 行，`PoolWriter::writes_of_failed_publishes`）。
///
/// # Errors
/// `InodeTreeWriteRefused`、`MoreNamedUnitsThanOneJournalRecordHolds`、`ContentExceedsDataUnit`、
/// `AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、释放判定路径的四种错、`PlacementRefused`、块设备错。
pub fn publish_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    // 先算这次之后 inode 树是什么样、这次重写哪些角色：只读，条款没写的那几格在这里交回，盘上逐字节不变。
    let resolved = plan.resolve(previous)?;
    let rewritten = resolved.rewritten_roles.clone();
    // 点名项一条记录装 67 个（D23（journal 的角色与格式） 已定项 12 / 已定项 17）：这次重写的角色多于它就要写成多条记录，
    // 而多条记录时共享的提交内生块在哪一条里点名还没定 ⇒ 在动分配器之前拒掉。
    let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
    if rewritten.len() > named_unit_capacity {
        return Err(PublishError::MoreNamedUnitsThanOneJournalRecordHolds {
            named_units: rewritten.len(),
            capacity: named_unit_capacity,
            undecided:
                UndecidedClauseBlockingMoreThanOneDataUnit::RecordThatNamesTheSharedCommitGeneratedUnits,
        });
    }
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
    publish_admission(allocator, &rewritten)?;
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let outcome = publish_admitted(pool, allocator, &plan, previous, &resolved, &release);
    if outcome.is_err() {
        *allocator = allocator_before_this_publish;
    }
    outcome
}

/// 一次发布的准入里与这次写什么内容无关的那两条：分配记录树与记账树第一版各只有一个节点（分裂不做），这次发布之后都要装得下。
/// 只读——不动分配器、不发一个写，算不过时盘上逐字节不变。两处调它：发布路径在动分配器之前（`publish_version`）；
/// 可写挂载在**取号之前**按这次挂载要发的那几次（写行 + 暖机）算一遍（`publish_sequence_admission`，增补 2 第 20a 行：
/// 算不过就不许先把实例代号烧掉——取号是两次系统配置槽写加一道屏障，之后再拒绝，池此后每试一次可写挂载就多烧一个代号）。
/// 两处读的是同一个内存里的分配器，取号不碰它，所以两次必定同答案；发布路径那一遍仍留着，它是动分配器之前的最后一道。
///
/// # Errors
/// `AllocationRecordsExceedOneNode`（这次之后的分配记录条数越过一个节点）、`AccountingEntriesExceedOneNode`（记账行数越过一个节点）。
pub fn publish_admission(
    allocator: &PoolAllocator,
    rewritten: &[TransactionUnit],
) -> Result<(), PublishError> {
    admission_of_one_publish(allocator, allocator.records().len(), rewritten)
}

/// 接连几次发布的准入，在第一次动分配器之前一次算完：按次序逐次走 `publish_admission` 那两条，前面几次要新增的分配记录
/// 算进后面几次的基数。基数只加不减，理由与单次那条同一句——释放只改写记录、不加，回收要 F 抬到释放代之上（`reclaim_released_up_to`），
/// 而这一串（写行 + 暖机）里不抬 F。
///
/// # Errors
/// `PublishSequenceRefusal`：第几次算不过（从 0 数），连它的 `PublishError` 一起交回。
pub fn publish_sequence_admission(
    allocator: &PoolAllocator,
    shapes: &[PublishShape],
) -> Result<(), PublishSequenceRefusal> {
    let mut records_before_this_publish = allocator.records().len();
    for (publish_index, shape) in shapes.iter().enumerate() {
        let rewritten = shape.rewritten_roles();
        admission_of_one_publish(allocator, records_before_this_publish, &rewritten).map_err(
            |cause| PublishSequenceRefusal {
                publish_index,
                cause,
            },
        )?;
        records_before_this_publish += rewritten.len() * allocator.devices.len();
    }
    Ok(())
}

/// 一串发布的准入里第几次算不过（从 0 数）、为什么。
#[derive(Debug)]
pub struct PublishSequenceRefusal {
    pub publish_index: usize,
    pub cause: PublishError,
}

/// 一次发布的那两条准入，分配记录的基数由调用方给：发布路径给的是分配器此刻的记录数，取号之前那一串给的是把前面几次
/// 要新增的算进去之后的数。
fn admission_of_one_publish(
    allocator: &PoolAllocator,
    records_before_this_publish: usize,
    rewritten: &[TransactionUnit],
) -> Result<(), PublishError> {
    // 分配记录树第一版只有一个节点：这次发布之后装不下就报错，不许走到 build_index_node 的断言
    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
    let records_after_this_publish =
        records_before_this_publish + rewritten.len() * allocator.devices.len();
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
    // 记账树第一版也只有一个节点：行数只随盘数变（代码三方第二轮攻方腿 Y3：约 80 块盘）。
    // 装行那一段按同一个 `allocator.devices` 装，并断言行数与这里算的相等。
    let accounting_entries_of_this_publish = accounting_entry_count(allocator.devices.len());
    let accounting_node_capacity = index_node_entry_capacity(
        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    );
    if accounting_entries_of_this_publish > accounting_node_capacity {
        return Err(PublishError::AccountingEntriesExceedOneNode {
            entries: accounting_entries_of_this_publish,
            capacity: accounting_node_capacity,
        });
    }
    Ok(())
}

/// 记账树里不带设备维的行：inode 号水位、待删占用、已承诺预留（D5（快照 / 空间记账机制） 已定项 8）。
const POOL_WIDE_ACCOUNTING_ENTRIES: usize = 3;
/// 记账树里每块盘各一行的：已分配、空闲、不可回收、defer 待释放、碎片段数、全空聚簇段数。
const ACCOUNTING_ENTRIES_PER_DEVICE: usize = 6;

/// 一次发布写的记账行数：池级行加每块盘各一组。准入与装行共用这一处。
fn accounting_entry_count(device_count: usize) -> usize {
    POOL_WIDE_ACCOUNTING_ENTRIES + ACCOUNTING_ENTRIES_PER_DEVICE * device_count
}

/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
    previous.unit(identity).clone()
}

/// 一个文件对象这次发布的两个内容角色（数据单元、extent 根）：单元字节、指针、出生序号。
/// 重写时由 `build_file_version_units` 装，照抄时从上一版取（`carried_file_version_units`）。
/// 这个对象的 inode 记录不在这里——它进 inode 树，由 `build_inode_tree_units` 按叶容器装。
struct FileVersionUnits {
    data_unit: Vec<u8>,
    data_pointer: DataPointer,
    extent_unit: Vec<u8>,
    extent_sequence: BirthSequence,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数；照抄的一版恒 0。
    key_order_mismatches: u64,
}

/// 装文件对象的单元时整个 checkpoint 共用的身份字段。
struct FileVersionCheckpoint<'checkpoint> {
    txg: CheckpointTxg,
    write_order: WriteOrder,
    filesystem_identifier: &'checkpoint [u8; 16],
    /// 位置条目按设备身份升序（I-2.5）。
    device_identities: &'checkpoint [DeviceIdentity],
}

/// 一个文件对象带位置条目的那个单元这次取到的落点。
#[derive(Clone, Copy, Debug)]
struct FileVersionSlots {
    data: SlotNumber,
}

/// 装一个文件对象的两个内容单元（字节表二、四·二）。出生序号从调用方传进来的发号器取，发号器的作用域是一次 checkpoint、
/// 不是这一次调用（D19（块指针的结构与宽度预算） 已定项 9：同一棵树内每写出一个码 2 或码 3 单元加 1，作用域换到下一个 checkpoint 时清零）：
/// 同一个 checkpoint 里装第二个对象时序号接着数，不在同一个 (树, txg, 实例) 上从 0 重数、撞出重复的映射 key（里程碑「第二个事务」增补 2 第 19 行）。
fn build_file_version_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    file: &FileVersionPlan<'_>,
    slots: FileVersionSlots,
    sequences: &mut BirthSequenceAllocator,
) -> FileVersionUnits {
    let txg = checkpoint.txg;
    let write_order = checkpoint.write_order;
    let instance = write_order.instance;
    let filesystem_identifier = checkpoint.filesystem_identifier;
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
        locations: location_entries(checkpoint.device_identities, slots.data, &data_unit),
        write_order,
    };
    let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
    let extent_key: [u8; 24] = extent_record[..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
        .try_into()
        .expect("24");
    let key_order_mismatches = count_key_order_mismatches(&[extent_key]);

    // t2 extent 树根兼叶（字节表四·二）。
    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
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

    FileVersionUnits {
        data_unit,
        data_pointer,
        extent_unit,
        extent_sequence,
        key_order_mismatches,
    }
}

/// 这一版 inode 树的全部单元：每片叶容器（重写的是这次装的、没重写的照抄上一版），加上重写了的根。
struct InodeTreeUnits {
    /// 左起按 key 序，每片一项。
    leaf_containers: Vec<InodeLeafContainerUnit>,
    /// 这次有叶容器被重写 ⇒ 根跟着重写（COW 叶 + 全部祖先，D23（journal 的角色与格式） 已定项 1 甲）；
    /// 一片都没重写（写行、暖机）⇒ `None`，根指针照抄上一版的树表条目、连出生序号都不发。
    rewritten_root: Option<InodeTreeRootUnit>,
}

/// 这次重写出来的 inode 树根。
struct InodeTreeRootUnit {
    bytes: Vec<u8>,
    birth_sequence: BirthSequence,
}

/// 这一版 inode 树里的一片叶容器：装了什么、这一版的指针、这一版的字节，以及它这次是不是重写的。
struct InodeLeafContainerUnit {
    index: InodeLeafContainerIndexInTree,
    contents: InodeLeafContainer,
    pointer: NodePointer,
    bytes: Vec<u8>,
    is_rewritten_this_publish: bool,
}

/// 装这次发布的 inode 树（字节表四）：先按叶序给这次重写的每一片发出生序号、装码 3 容器，再装码 2 根
/// （树内先叶后根，D3（空间分配） 已定项 10 ⑤ 的 bump 次序与 D19（块指针的结构与宽度预算） 已定项 9 的发号次序）。
/// 没重写的那几片一个字节都不动：字节、落点、指针、出生序号全取上一版（分裂时的左半走的就是这一支）。
///
/// # Panics
/// 这次重写的某一片没拿到落点，或某一片既不在这次的重写清单里、上一版里也没有——两样都说明角色清单
/// （`PublishPlan::rewritten_roles`）与这里算的树对不上，那是发布路径自己的不变量。
fn build_inode_tree_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    inode_tree: &InodeLeafContainersAfterThisPublish,
    previous_containers: &[InodeLeafContainerVersion],
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> InodeTreeUnits {
    let txg = checkpoint.txg;
    let instance = checkpoint.write_order.instance;
    let inode_tree_identifier = TreeIdentifier(TREE_IDENTIFIER_INODE);
    let mut leaf_containers = Vec::with_capacity(inode_tree.containers.len());
    for (position, contents) in inode_tree.containers.iter().enumerate() {
        let index = InodeLeafContainerIndexInTree::of_position(position);
        let role = TransactionUnit::InodeLeafContainer(index);
        if inode_tree.rewritten.contains(&index) {
            let birth_sequence = sequences.next(inode_tree_identifier, txg, instance);
            let bytes = build_packed_unit(
                contents.identity,
                u16::try_from(INODE_RECORD_BYTES).expect("140"),
                &contents
                    .records
                    .iter()
                    .map(InodeRecord::to_bytes)
                    .collect::<Vec<_>>(),
                txg,
                checkpoint.filesystem_identifier,
                checkpoint.write_order,
                birth_sequence,
            );
            let pointer = NodePointer {
                head: PointerHead {
                    birth_tree: inode_tree_identifier,
                    birth_txg: txg,
                },
                locations: location_entries(
                    checkpoint.device_identities,
                    *slots.get(&role).expect("这次重写的叶容器都取了落点"),
                    &bytes,
                ),
                instance,
                birth_sequence,
            };
            leaf_containers.push(InodeLeafContainerUnit {
                index,
                contents: contents.clone(),
                pointer,
                bytes,
                is_rewritten_this_publish: true,
            });
        } else {
            let carried = previous_containers
                .get(position)
                .expect("没重写的叶容器一定在上一版里（新建的那几片恒在重写清单里）");
            leaf_containers.push(InodeLeafContainerUnit {
                index,
                contents: contents.clone(),
                pointer: carried.pointer,
                bytes: Vec::new(),
                is_rewritten_this_publish: false,
            });
        }
    }
    if inode_tree.rewritten.is_empty() {
        return InodeTreeUnits {
            leaf_containers,
            rewritten_root: None,
        };
    }
    // t4 inode 树根：层级 1，每片叶一条 120 字节条目（按分隔 key 升序）。
    // key 区间 = 第一条与最后一条**条目的 key**（码 2 节点的 key 区间恒是这样，I-1.1（key 区间罩住条目）：
    // 条目的 key 打头，内部节点条目打头的是分隔 key），不是树里最小 / 最大的 inode 号——最右那片装到第二条记录起，
    // 树里的最大 key 就大于最后一条分隔 key 了。
    let root_sequence = sequences.next(inode_tree_identifier, txg, instance);
    let smallest_key = leaf_containers
        .first()
        .expect("记录数为 0 的类型 2 容器不落盘（D8 已定项 6）：树里至少有一片叶")
        .contents
        .separator_key()
        .to_le_bytes();
    let largest_key = leaf_containers
        .last()
        .expect("同上：至少有一片叶")
        .contents
        .separator_key()
        .to_le_bytes();
    let root_unit = build_index_node(
        inode_tree_identifier,
        1,
        8,
        &smallest_key,
        &largest_key,
        txg,
        checkpoint.filesystem_identifier,
        instance,
        root_sequence,
        u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
        &leaf_containers
            .iter()
            .map(|container| {
                build_inode_internal_entry(
                    container.contents.separator_key(),
                    container.contents.identity,
                    container.pointer,
                )
            })
            .collect::<Vec<_>>(),
    );
    InodeTreeUnits {
        leaf_containers,
        rewritten_root: Some(InodeTreeRootUnit {
            bytes: root_unit,
            birth_sequence: root_sequence,
        }),
    }
}

/// 照抄上一版的两个文件内容角色：数据指针与两个单元的字节都取上一版。
fn carried_file_version_units(carried: &TransactionOutput) -> FileVersionUnits {
    FileVersionUnits {
        data_unit: carried.unit(TransactionUnit::Data).bytes.clone(),
        data_pointer: carried.data_pointer,
        extent_unit: carried.unit(TransactionUnit::ExtentRoot).bytes.clone(),
        extent_sequence: carried
            .tree_root_pointer(TREE_IDENTIFIER_EXTENT)
            .birth_sequence,
        key_order_mismatches: 0,
    }
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
    resolved: &ResolvedPublish,
    release: &[Placement],
) -> Result<TransactionOutput, PublishError> {
    let rewritten = &resolved.rewritten_roles;
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
            PlacementRule::UserData => allocator.try_allocate_user_data(txg),
            PlacementRule::CommitGenerated(footprint) => {
                allocator.try_allocate_commit_generated(footprint, txg)
            }
        };
        let placement = placement.map_err(|refusal| PublishError::PlacementRefused {
            unit: *identity,
            refusal,
        })?;
        slots.insert(*identity, placement.slot);
    }
    let slot_of = |identity: TransactionUnit| slots[&identity];
    let device_identities: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let checkpoint = FileVersionCheckpoint {
        txg,
        write_order,
        filesystem_identifier,
        device_identities: &device_identities,
    };

    // 文件内容角色：有新版本就装两个单元，没有就照抄上一版的指针与字节。
    let carried_file = match &plan.file {
        Some(_) => None,
        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
    };
    let FileVersionUnits {
        data_unit,
        data_pointer,
        extent_unit,
        extent_sequence,
        key_order_mismatches,
    } = match (&plan.file, carried_file) {
        (Some(file), _) => build_file_version_units(
            &checkpoint,
            file,
            FileVersionSlots {
                data: slot_of(TransactionUnit::Data),
            },
            &mut sequences,
        ),
        (None, Some(carried)) => carried_file_version_units(carried),
        (None, None) => {
            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
        }
    };

    // t3 inode 树（字节表四）：这次重写的叶容器与根都在这里装；没重写的叶容器取上一版的指针，字节下面照抄。
    let previous_inode_leaf_containers: &[InodeLeafContainerVersion] = match previous {
        Some(previous_version) => &previous_version.inode_leaf_containers,
        None => &[],
    };
    let inode_tree_units = build_inode_tree_units(
        &checkpoint,
        &resolved.inode_tree,
        previous_inode_leaf_containers,
        &slots,
        &mut sequences,
    );
    // 第一个文件那条 inode 记录：树里 inode 号 1 的那一条。每条发布路径要么这次写了它（文件版本那一次），
    // 要么接在写过它的上一版后面（写行、暖机、建 inode），所以它恒在树里。
    let inode_record = inode_tree_units
        .leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == FIRST_INODE_NUMBER)
        .copied()
        .expect("inode 树里恒有第一个文件那条记录（inode 号 1）");

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
    // inode 号水位 = 下一个可用号（D5（快照 / 空间记账机制） 已定项 4 第 12 项）：每次发布重写这一行，
    // 这次建了几个 inode 就在上一版的水位上加几。树里最大的 key 恒小于它（I-9.6（水位大于两处最大号））——
    // 这里按「已经发出去的号」算，不按「树里现在有什么」算：号一旦发出去就不再复用（D8（核心索引结构） 已定项 6）。
    let new_inodes_of_this_publish =
        u64::try_from(plan.new_inode_records.len()).expect("这次新建的 inode 数");
    let inode_number_watermark = match previous {
        Some(previous_version) => previous_version.inode_number_watermark(),
        // 第一个事务：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
        None => FIRST_INODE_NUMBER + 1,
    } + new_inodes_of_this_publish;
    let mut accounting_entries = vec![
        pool_wide(
            STATISTIC_INODE_WATERMARK,
            TREE_IDENTIFIER_INODE,
            inode_number_watermark,
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
    assert_eq!(
        accounting_entries.len(),
        accounting_entry_count(allocator.devices.len()),
        "准入按 accounting_entry_count 判过装不装得下：这里装的行数要与它相等，改了行的构成要一起改那两个常量"
    );
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
    // 文件内容角色的指针：重写的按这次的落点算，照抄的取上一版。
    let extent_pointer = match carried_file {
        None => node_pointer(
            TREE_IDENTIFIER_EXTENT,
            TransactionUnit::ExtentRoot,
            &extent_unit,
            extent_sequence,
        ),
        Some(carried) => carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT),
    };
    // inode 树根的指针：这次重写了就按这次的落点算，一片叶都没重写就取上一版树表里那一条。
    let inode_root_pointer = match (&inode_tree_units.rewritten_root, previous) {
        (Some(root), _) => node_pointer(
            TREE_IDENTIFIER_INODE,
            TransactionUnit::InodeRoot,
            &root.bytes,
            root.birth_sequence,
        ),
        (None, Some(carried)) => carried.tree_root_pointer(TREE_IDENTIFIER_INODE),
        (None, None) => {
            unreachable!("没有上一版的发布（第一个文件版本）恒要写 inode 记录 ⇒ 恒重写 inode 树")
        }
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

    // t7 中央映射树（字节表三·二）：码 1 一条 + 码 2 / 码 3 五条（inode 叶容器多一片就多一条）；
    // 映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // 照抄的角色 key 照旧、位置照旧；重写的按这次的指针算。
    let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> = vec![
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
    ];
    for container in &inode_tree_units.leaf_containers {
        mapped_units_with_locations.push((
            TransactionUnit::InodeLeafContainer(container.index),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
            container.pointer.locations,
        ));
    }
    mapped_units_with_locations.extend([
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
    ]);
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

    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（按 bump 次序：数据、extent 根、每片 inode 叶容器、
    // inode 根、四个固定点单元，实例表单元在末尾）。
    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
        slot: slot_of(identity),
        identity,
        bytes,
    };
    let mut units: Vec<PublishedUnit> = Vec::new();
    units.push(match carried_file {
        None => rewritten_unit(TransactionUnit::Data, data_unit.clone()),
        Some(carried) => carried_unit(carried, TransactionUnit::Data),
    });
    units.push(match carried_file {
        None => rewritten_unit(TransactionUnit::ExtentRoot, extent_unit.clone()),
        Some(carried) => carried_unit(carried, TransactionUnit::ExtentRoot),
    });
    for container in &inode_tree_units.leaf_containers {
        let identity = TransactionUnit::InodeLeafContainer(container.index);
        units.push(if container.is_rewritten_this_publish {
            rewritten_unit(identity, container.bytes.clone())
        } else {
            carried_unit(
                previous.expect("没重写的叶容器只会出现在接着上一版的发布里"),
                identity,
            )
        });
    }
    units.push(match &inode_tree_units.rewritten_root {
        Some(root) => rewritten_unit(TransactionUnit::InodeRoot, root.bytes.clone()),
        None => carried_unit(
            previous.expect("没重写 inode 根的发布恒接在上一版之后"),
            TransactionUnit::InodeRoot,
        ),
    });
    units.push(rewritten_unit(
        TransactionUnit::AllocationTree,
        allocation_unit.clone(),
    ));
    units.push(rewritten_unit(
        TransactionUnit::AccountingTree,
        accounting_unit.clone(),
    ));
    units.push(rewritten_unit(
        TransactionUnit::MappingTree,
        mapping_unit.clone(),
    ));
    units.push(rewritten_unit(
        TransactionUnit::TreeTable,
        tree_table_unit.clone(),
    ));
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
            TransactionUnit::InodeLeafContainer(index) => node_key_tail(
                instance,
                inode_tree_units
                    .leaf_containers
                    .get(index.position())
                    .expect("点名的叶容器是这次重写的那几片之一")
                    .pointer
                    .birth_sequence,
            ),
            TransactionUnit::InodeRoot => node_key_tail(
                instance,
                inode_tree_units
                    .rewritten_root
                    .as_ref()
                    .expect("点名 inode 根的发布一定重写了它")
                    .birth_sequence,
            ),
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

    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换。
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：六步收在一个闭包里，失败在这里记账再把错原样交回——
    // 账是两次快照之差、只在成功路径上取，失败那次落盘的写不交出去就不属于任何一次发布，与设备一层的合计对不上。
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
        for unit in &written_units {
            writer.perform(CommitStep::WriteUnitToEveryDevice {
                slot: unit.slot,
                unit: &unit.bytes,
                identity: unit.identity,
            })?;
        }
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: plan.counter,
            record: &record_bytes,
        })?;
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {
            checkpoint_txg: txg,
            root_slot: &root_slot,
        })?;
        writer.perform(CommitStep::RotateSystemConfigurationSlots {
            journal_tail: plan.counter,
            journal_instance: instance,
        })
    };
    if let Err(cause) = persist(pool) {
        pool.count_failed_publish(&writes_before_this_publish);
        return Err(PublishError::BlockDevice(cause));
    }

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
        inode_leaf_containers: inode_tree_units
            .leaf_containers
            .into_iter()
            .map(|container| InodeLeafContainerVersion {
                contents: container.contents,
                pointer: container.pointer,
            })
            .collect(),
        mapped_units,
        released: release.to_vec(),
        key_order_mismatches,
        highest_transaction_number_in_this_instance: plan
            .highest_transaction_number_before_this_publish
            .max(plan.transaction),
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_publish),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::INODE_LEAF_RECORDS;

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

    /// 出生序号的作用域是一次 checkpoint（D19（块指针的结构与宽度预算） 已定项 9）：同一个 checkpoint 里连着装两个文件对象
    /// 与一棵两片叶的 inode 树，后装的接着先装的序号数，写进单元头里的也是接着数的号，映射 key 不撞；
    /// 发号器挪回「每次装对象都重建」时第二个对象从 0 重数，这条用例红（里程碑「第二个事务」增补 2 第 19 行）。
    #[test]
    fn second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero(
    ) {
        let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
        let instance = InstanceGeneration(1);
        let filesystem_identifier = [0x5a; 16];
        let device_identities = [DeviceIdentity(0), DeviceIdentity(1)];
        let checkpoint = FileVersionCheckpoint {
            txg,
            write_order: WriteOrder {
                instance,
                transaction: FIRST_TRANSACTION_NUMBER,
            },
            filesystem_identifier: &filesystem_identifier,
            device_identities: &device_identities,
        };
        let content = [7u8; 3000];
        let file = FileVersionPlan {
            content: &content,
            write_time_seconds: 1_788_000_000,
            inode_object_birth: txg,
            // 第一个事务那次发布的取值：改动计数 = 这次发布的 checkpoint_txg（增补 2 第 11 行）。
            change_count: FIRST_TRANSACTION_TXG,
        };
        let mut checkpoint_sequences = BirthSequenceAllocator::default();
        let first = build_file_version_units(
            &checkpoint,
            &file,
            FileVersionSlots {
                data: SlotNumber(50176),
            },
            &mut checkpoint_sequences,
        );
        let second = build_file_version_units(
            &checkpoint,
            &file,
            FileVersionSlots {
                data: SlotNumber(50178),
            },
            &mut checkpoint_sequences,
        );
        assert_eq!(
            (first.extent_sequence, second.extent_sequence),
            (BirthSequence(0), BirthSequence(1)),
            "同一个 checkpoint 的第二个对象接着数：extent 树 0、1"
        );
        assert_eq!(
            parse_index_node(&second.extent_unit)
                .expect("刚装的 extent 根解得开")
                .birth_sequence,
            BirthSequence(1),
            "单元头里写的就是接着数的号"
        );
        // inode 树这一半：两片叶容器加一个根，按叶序先叶后根发号，接着上面两个对象已经用掉的号数
        // （inode 树那个计数器上面一个都没用过，所以从 0 起）。
        let mut records = Vec::new();
        // 这里不写 `change_count: txg.0,`：`crates/mutations.tsv` 按那串字面锚在 `publish_overwrite` 上，
        // 同一份文件里出现两次它就腐化（门禁 59 号）。
        let change_count = txg.0;
        for inode in 1..=(INODE_LEAF_RECORDS + 1) {
            records.push(InodeRecord {
                inode,
                object_birth: txg,
                size: 0,
                occupied_blocks_of_512_bytes: 0,
                change_count,
                write_time_seconds: 1_788_000_000,
            });
        }
        let inode_tree = write_records_into_leaf_containers(
            &[],
            &records,
            txg,
            TreeIdentifier(TREE_IDENTIFIER_INODE),
        )
        .expect("234 条记录分两片容器");
        let mut slots = BTreeMap::new();
        slots.insert(
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(0)),
            SlotNumber(50242),
        );
        slots.insert(
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1)),
            SlotNumber(50250),
        );
        let inode_tree_units = build_inode_tree_units(
            &checkpoint,
            &inode_tree,
            &[],
            &slots,
            &mut checkpoint_sequences,
        );
        assert_eq!(
            inode_tree_units
                .leaf_containers
                .iter()
                .map(|container| container.pointer.birth_sequence)
                .collect::<Vec<_>>(),
            vec![BirthSequence(0), BirthSequence(1)],
            "两片叶按叶序发号"
        );
        assert_eq!(
            inode_tree_units
                .rewritten_root
                .as_ref()
                .expect("这次重写了根")
                .birth_sequence,
            BirthSequence(2),
            "树内先叶后根：根排在两片叶之后"
        );
        assert_ne!(
            mapping_key_for_node(
                UNIT_CLASS_PACKED,
                inode_tree_units.leaf_containers[0].pointer
            ),
            mapping_key_for_node(
                UNIT_CLASS_PACKED,
                inode_tree_units.leaf_containers[1].pointer
            ),
            "两片叶容器的映射 key 不撞"
        );
        assert_eq!(
            checkpoint_sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance),
            BirthSequence(3),
            "装完两片叶与一个根之后，同一个 checkpoint 里 inode 树的下一个号是 3"
        );
        assert_eq!(
            checkpoint_sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance),
            BirthSequence(2),
            "extent 树那个计数器各数各的：两个对象用掉 0、1，下一个是 2"
        );
    }

    #[test]
    fn every_transaction_unit_names_its_class_tree_and_placement_rule() {
        for identity in TransactionUnit::IN_BUMP_ORDER {
            assert_eq!(identity.tag().len(), 2);
        }
        assert_eq!(
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST)
                .placement(),
            PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned),
            "码 3 容器按数据单元那一档"
        );
        assert_eq!(
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1))
                .tag(),
            "t3+1",
            "第二片叶容器的步号：字节表七只登记了一片那一档的 t3"
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
