//! 事务层（里程碑步 3 / 步 4 / 步 5）：一个共享的提交状态机 + 一个封闭的提交步骤枚举
//! （D17（实现分层与第三方管道） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
//! 三条路径都从同一个枚举走：取号 = 逐盘一次系统配置槽写 + 一道屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）；
//! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
//! 第一个事务（D16（发布语义） 已定项 7）= 单元写 × 8 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换。
//! 覆盖写（里程碑「第二个事务」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、六个提交内生块与树表 COW 出新版本，
//! 被换下的八个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
//! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, EXTENT_KEY_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
    INSTANCE_ROW_BYTES, JOURNAL_NAMED_ENTRIES_PER_RECORD, MAPPING_ENTRY_BYTES, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
};

use crate::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber, TreeIdentifier,
};
use crate::admission::{
    admission_reading_before_a_publish, admit_on_every_device, demand_of_the_roles_on_each_device,
    AdmissionRefusedOnSomeDevices, BytesOnOneDevice, SpaceAdmission,
};
use crate::allocation_record_tree::{
    build_allocation_record_tree_node, children_of, nodes_holding_records,
    nodes_whose_contents_changed, records_of_each_leaf, AllocationRecordTreeGeometry,
    AllocationRecordTreeNode, AllocationRecordTreeNodeContents, AllocationRecordTreeNodeOrigin,
    AllocationRecordTreeNodePosition, AllocationRecordTreePlan, AllocationRecordTreeVersion,
};
use crate::allocator::{
    AllocationRecord, Placement, PlacementRefusal, PoolAllocator, UnitFootprint,
};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::checksum::crc32_castagnoli;
use crate::code_two_tree::{
    build_internal_entry, internal_entry_width_in_bytes, leaf_position_routed_to,
    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
    CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents, CodeTwoTreeNodeOrigin,
    CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation, CodeTwoTreeRefusal,
    CodeTwoTreeShape, CodeTwoTreeVersion,
};
use crate::extent_tree::{
    build_lower_internal_node, build_lower_leaf, build_upper_internal_node, build_upper_leaf,
    lower_children_of_a_file_without_holes, lower_segment_nodes_of_a_file_without_holes,
    upper_path_of_inode, upper_root_level_for, ExtentLowerNodePosition, ExtentTreeNodeIdentity,
    ExtentTreeVersion, ExtentUpperLeafEntry, ExtentUpperLeafTarget, ExtentUpperNodePosition,
};
use crate::inode_tree::{
    write_records_into_leaf_containers, InodeLeafContainer, InodeLeafContainerIndexInTree,
    InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
};
use crate::instance_table::{
    instance_table_page_records, instance_table_pages_for_rows, instance_table_rows_of_each_page,
    InstanceRow, InstanceTableChainRecord, InstanceTablePageIndex,
};
use crate::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish, NamedUnit,
};
use crate::make_filesystem::{
    location_entries, MakeFilesystemParameters, MKFS_INSTANCE_GENERATION, TREE_TABLE_KEY_WIDTH,
};
use crate::pointer::{
    slot_shared_by_both_location_entries, BirthSequence, DataPointer,
    LocationEntriesOnDifferentSlots, LocationEntry, NodePointer, PointerHead,
};
use crate::records::{
    build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
    mapping_key_for_data, mapping_key_for_node, node_key_tail, parse_mapping_entry,
    AccountingEntry, InodeRecord, TreeTableEntry, ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
    STATISTIC_ALLOCATED_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
    STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
    STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
    STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
};
use crate::recovery::{
    highest_root_instance, tree_table_entry_count, verified_system_configuration_slots, PoolReader,
    RecoveryFailure,
};
use crate::rollback_witness::RollbackWitnessTable;
use crate::root_record::RootRecord;
use crate::root_ring::{slot_offset, target_for_publish};
use crate::system_configuration::{
    SystemConfiguration, SystemImmutableConfiguration, SystemMutableConfiguration,
    SystemRuntimeConfiguration, SystemRuntimeQuantities,
};
use crate::unit::{
    build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
    parse_index_node, unit_filesystem_identifier, DataUnitIdentity, WriteOrder, UNIT_CLASS_DATA,
    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use crate::write_accounting::{WritesByStructureKind, WrittenStructureKind};
use crate::write_request_split::{
    split_sequential_write_into_one_unit_transactions, OneUnitTransaction,
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

/// 一条 journal 记录装几个点名项，只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入）。
/// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）在产品路径上要一次发布点名 68 项以上才走得到——树表 0 条的一版上写行要实例表长到
/// 67 片（两万四千多行），带文件的一版要六十几片 inode 叶容器；压小这个数就能用几个单元造出跨记录的发布。产品路径恒 `FromTheRecordFormat`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRecordNamedEntryCapacity {
    /// 记录格式装得下的：(4096 − 311) ÷ 56 = 67 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）。
    FromTheRecordFormat,
    /// 压到这么多项（1..=67）。
    CappedForTests { named_entries_per_record: usize },
}

impl JournalRecordNamedEntryCapacity {
    /// 一条记录装几个点名项。
    #[must_use]
    pub fn named_entries_per_record(self) -> usize {
        match self {
            JournalRecordNamedEntryCapacity::FromTheRecordFormat => {
                usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67")
            }
            JournalRecordNamedEntryCapacity::CappedForTests {
                named_entries_per_record,
            } => named_entries_per_record,
        }
    }

    /// 走的是哪一臂（五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn branch_name(self) -> &'static str {
        match self {
            JournalRecordNamedEntryCapacity::FromTheRecordFormat => {
                "journal_record_named_entries=from_the_record_format"
            }
            JournalRecordNamedEntryCapacity::CappedForTests { .. } => {
                "journal_record_named_entries=capped_for_tests"
            }
        }
    }
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
    /// 记账树与中央映射树的节点容量，只供测试的开关（`CodeTwoTreeNodeCapacities`）；新开的写入口按产品路径起步。
    code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
    /// 一条 journal 记录装几个点名项，只供测试的开关（[`JournalRecordNamedEntryCapacity`]）；新开的写入口按记录格式起步。
    journal_record_named_entry_capacity: JournalRecordNamedEntryCapacity,
    /// 这个写入口之后的每一次系统配置写带哪一张回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：
    /// `None` 照抄那块盘上择到的那一槽里的（新开的写入口按它起步，不读盘）；可写挂载与回退按删除规则与这次的回退定好之后装上（`mount`）。
    rollback_witness_to_write: Option<RollbackWitnessTable>,
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
            code_two_tree_node_capacities: CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            journal_record_named_entry_capacity:
                JournalRecordNamedEntryCapacity::FromTheRecordFormat,
            rollback_witness_to_write: None,
        }
    }
}

impl<Device: BlockDevice> PoolWriter<'_, Device> {
    /// 装上节点容量这个只供测试的开关（`CodeTwoTreeNodeCapacities`）。产品路径一处都不调它。
    ///
    /// # Panics
    /// 压的容量大于格式算出来的（节点装不下，走到 `build_index_node` 的断言），或叶小于 1 条、内部节点小于 2 个孩子
    /// （切出来的两半要都不空、根分裂出来的新根要装得下两个孩子）：开关给错了。
    pub fn set_code_two_tree_node_capacities(&mut self, capacities: CodeTwoTreeNodeCapacities) {
        for tree in [
            MultiLevelCodeTwoTree::Accounting,
            MultiLevelCodeTwoTree::CentralMapping,
        ] {
            let capped = capacities.of_tree(tree);
            let of_the_format = tree.node_capacity_of_the_node_format();
            assert!(
                (1..=of_the_format.leaf_entries).contains(&capped.leaf_entries)
                    && (2..=of_the_format.internal_entries).contains(&capped.internal_entries),
                "{tree:?} 的容量压到 {capped:?}：叶要在 1..={}、内部节点要在 2..={} 之内",
                of_the_format.leaf_entries,
                of_the_format.internal_entries
            );
        }
        self.code_two_tree_node_capacities = capacities;
    }

    /// 这会儿装着的节点容量：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn code_two_tree_node_capacities(&self) -> CodeTwoTreeNodeCapacities {
        self.code_two_tree_node_capacities
    }

    /// 装上「一条 journal 记录装几个点名项」这个只供测试的开关（[`JournalRecordNamedEntryCapacity`]）。产品路径一处都不调它。
    ///
    /// # Panics
    /// 压的项数是 0 或大于记录格式装得下的 67 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）：开关给错了。
    pub fn set_journal_record_named_entry_capacity(
        &mut self,
        capacity: JournalRecordNamedEntryCapacity,
    ) {
        let of_the_format = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
        assert!(
            (1..=of_the_format).contains(&capacity.named_entries_per_record()),
            "一条记录装的点名项压到 {capacity:?}：要在 1..={of_the_format} 之内"
        );
        self.journal_record_named_entry_capacity = capacity;
    }

    /// 这会儿装着的「一条记录装几个点名项」：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn journal_record_named_entry_capacity(&self) -> JournalRecordNamedEntryCapacity {
        self.journal_record_named_entry_capacity
    }

    /// 从下一次系统配置写起每一次都带这一张回退见证表（挂载按删除规则与这次的回退定好之后装上）。
    pub(crate) fn write_rollback_witness_from_now_on(&mut self, table: RollbackWitnessTable) {
        self.rollback_witness_to_write = Some(table);
    }

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
                let target = target_for_publish(
                    checkpoint_txg,
                    self.parameters.geometry.root_ring_slots_per_region,
                );
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
        let verified_on_this_device = verified_system_configuration_slots(
            &*self.devices,
            identity,
            spacing,
            &self.parameters.filesystem_identifier,
        );
        let slot_generation = verified_on_this_device
            .iter()
            .map(|system_configuration| system_configuration.quantities.slot_generation)
            .max()
            .unwrap_or(0)
            + 1;
        // 回退见证表每一次系统配置写都整张带着（D23（journal 的角色与格式） 已定项 14「回退见证」）：挂载定了这次要写哪一张就写那一张，
        // 没定就照抄这块盘上择到的那一槽（两槽里自证过、世代号最大的）里的——与算世代号读的是同两槽，不多读一次盘。
        let rollback_witness = match self.rollback_witness_to_write {
            Some(table) => table,
            None => verified_on_this_device
                .iter()
                .max_by_key(|system_configuration| system_configuration.quantities.slot_generation)
                .map_or(RollbackWitnessTable::EMPTY, |system_configuration| {
                    system_configuration.rollback_witness
                }),
        };
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
            rollback_witness,
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

/// 暖机那两次空发布里有一次没做成：块设备的错，连同失败之前已经落盘的那几次各自的写（按先后）。
/// 失败那一次落盘阶段已记的写照旧进调用方那个写入口的失败账（`PoolWriter::writes_of_failed_publishes`）；已经落盘的那几次的账
/// 只在这里——它们的输出随错一起丢掉，不交出来，这段暖机里设备一层数到的写与程序交得出的账就对不上（增补 2 收口表第 58 行）。
#[derive(Debug)]
pub struct WarmUpFailed {
    pub cause: BlockDeviceError,
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
}

/// mkfs 之后环里一条记录都没有：接着它数的 jsn 从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条，与可写挂载「环里没有记录时从 1 起」同一个起点）。
const LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM: u64 = 0;

/// 暖机（D16（发布语义） 已定项 8）：mkfs 之后第一次可写挂载的两次空发布，checkpoint_txg 走 1、2；环在 mkfs 之后是空的，
/// jsn 从 1 起。其余见 `warm_up_after_journal_counter`。
///
/// # Errors
/// 块设备报的错，连同已经落盘的那几次的账（[`WarmUpFailed`]）。
pub fn warm_up<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
) -> Result<WarmUpOutput, WarmUpFailed> {
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
/// 块设备报的错，连同已经落盘的那几次的账（[`WarmUpFailed`]）。
pub fn warm_up_after_journal_counter<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
    last_journal_counter: u64,
) -> Result<WarmUpOutput, WarmUpFailed> {
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
                // mkfs 同一个进程里那条流：mkfs 之后根环里只有 mkfs 的第 0 代根，暖机根照抄它（D16（发布语义） 已定项 9）。
                tree_identifier_watermark: genesis.tree_identifier_watermark,
            },
        )
        .map_err(|cause| WarmUpFailed {
            cause,
            writes_of_persisted_publishes: writes.clone(),
        })?;
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
    /// 这次写进根记录与记录新根段的树 ID 水位：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)
    /// （D8（核心索引结构） 已定项 8 ②），零单元发布一个号都不发 ⇒ 就是根环里的 max。本实例的第一次发布由挂载按环算
    /// （回退到的那一版的水位可以低于环里被抛弃的根带的）；之后接着本会话现行那一版的根照抄。
    pub tree_identifier_watermark: u64,
}

/// 树表 0 条的一版上一次发布写出的东西：根、记录、记录的字节（下一条的反向链要罩它）、按结构种类的写。
/// 零单元发布（暖机）与写行那次发布（重写实例表一个单元）都交回它——两者写出的可观测态只差根记录里的实例表指针，
/// 而这一版没有记账树、分配记录树与映射树，没有别的内存态要带到下一次发布（`PoolVersion::WithoutFile` 装的就是它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionWithoutFilePublishOutput {
    pub root: RootRecord,
    /// 这次发布的末条记录（下一条的反向链要罩它的头）。
    pub record: JournalRecord,
    pub record_bytes: Vec<u8>,
    /// 末条之前的那几条：写行那次发布点名项多于一条记录装得下时末条再跨记录（D23（journal 的角色与格式） 已定项 17）；
    /// 零单元发布与只有一条记录的写行是空的。
    pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
    /// 这次发布真正写出的角色，按写出的次序（= 点名项的次序：实例表链各片尾片先，再分配记录树重写的节点先叶后根）；零单元发布是空的。
    pub rewritten: Vec<TransactionUnit>,
    pub writes: WritesByStructureKind,
}

/// 一次发布要交给设备的全部字节，按 D16（发布语义） 已定项 7 的持久顺序排好：这次重写的单元 → 屏障 → journal 记录 → 屏障 →
/// 根槽 FUA → 系统配置槽轮换。三条发布路径（带单元的、零单元的、树表 0 条上写行的）都先装成它、再交给
/// [`persist_publish_writes`] 落盘；落盘中途失败的那一次原样冻结着它，重发时照它逐字节再发一遍
/// （D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：checkpoint_txg、计数器、本次发布内序号、记录标志、
/// 单元的位置与字节都不变）。系统配置槽不在里面：它的世代号按写那一刻盘上自证过的槽现算（D22（单元原子性怎么合成） 已定项 16，逐盘计），
/// 内容是 tail、实例代号与写入口手里的回退见证表。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishWrites {
    /// 这次重写的单元，按写的次序（bump 次序）。零单元发布是空的。
    pub units: Vec<PublishedUnit>,
    /// 这次发布的 journal 记录，按计数器升序。
    pub records: Vec<JournalRecordWrite>,
    pub checkpoint_txg: CheckpointTxg,
    /// 根槽 FUA 写的整槽字节。
    pub root_slot: Vec<u8>,
    /// 系统配置里的 tail：这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
    pub journal_tail: u64,
    pub journal_instance: InstanceGeneration,
}

/// 一条要写进 journal 环的记录：计数器（定落在环里哪一槽）与整条 4096 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalRecordWrite {
    pub counter: u64,
    pub bytes: Vec<u8>,
}

/// 把一次发布的字节按持久顺序交给设备（D16（发布语义） 已定项 7）。三条发布路径与重发冻结的那一次共用这一处——
/// 各抄一份会分叉，而「根槽在系统配置槽之前」正是崩溃窗口那几格的前提。
///
/// # Errors
/// 块设备报的第一个错原样交回，之后的步骤一步都不发。
fn persist_publish_writes<Device: BlockDevice>(
    writer: &mut PoolWriter<'_, Device>,
    writes: &PublishWrites,
) -> Result<(), BlockDeviceError> {
    for unit in &writes.units {
        writer.perform(CommitStep::WriteUnitToEveryDevice {
            slot: unit.slot,
            unit: &unit.bytes,
            identity: unit.identity,
        })?;
    }
    writer.perform(CommitStep::Barrier)?;
    for record in &writes.records {
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: record.counter,
            record: &record.bytes,
        })?;
    }
    writer.perform(CommitStep::Barrier)?;
    persist_the_root_then_rotate_the_system_configuration(
        writer,
        writes.checkpoint_txg,
        &writes.root_slot,
        writes.journal_tail,
        writes.journal_instance,
    )
}

/// 落盘阶段中途失败、冻结着等原样重发的那一次发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：发布不接受失败，
/// 失败的那次冻结，下一次发布之前逐字节原样重发它，重发成功才建下一次发布）。住在分配器上
/// （`PoolAllocator::frozen_publish`）：每一次发布都要交分配器进来，冻结着的时候发布路径在任何读写之前拒绝
/// （`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），只有 [`resend_the_frozen_publish`] 清得掉它。
/// 于是同一 (实例代号, checkpoint_txg) 不会先后写出两次不同的发布、留下两条带末条标志的记录。
///
/// 零单元发布（`publish_without_units`）不经分配器、不冻结：它的调用方（暖机、可写挂载取号之后那一串）一失败就整个挂载返回错误，
/// 下一次发布在下一次挂载里、换了实例代号。
#[derive(Clone, Debug)]
pub struct FrozenPublish {
    writes: PublishWrites,
    /// 重发成功之后交出去的那一版；它的写账在重发那一刻现记（`writes` 字段），这里是空账。
    version: PoolVersion,
    /// 这次发布成立之后的分配器（释放、取落点、记根都做完的那一份）。冻结期间调用方手里的分配器回到发布之前的样子，
    /// 重发成功时整个换成它——冻结期间对那个分配器做的别的改动随之丢掉（发布路径冻结期间一个都不许进来）。
    allocator_after_the_publish: PoolAllocator,
}

impl FrozenPublish {
    #[must_use]
    pub fn checkpoint_txg(&self) -> CheckpointTxg {
        self.writes.checkpoint_txg
    }

    #[must_use]
    pub fn instance(&self) -> InstanceGeneration {
        self.writes.journal_instance
    }

    /// 重发时要逐字节再发一遍的那些字节。
    #[must_use]
    pub fn writes(&self) -> &PublishWrites {
        &self.writes
    }
}

/// 发布路径的第一道：分配器上冻结着一次没重发的发布，就在任何读写之前拒绝（D23（journal 的角色与格式） 已定项 14
/// 「这一版的失败处置」：重发成功才建下一次发布）。
///
/// # Errors
/// `PublishFrozenAfterAWriteFailureIsNotResentYet`，带着冻结的那一次的 checkpoint_txg 与实例代号。
fn refuse_while_a_publish_is_frozen(allocator: &PoolAllocator) -> Result<(), PublishError> {
    match allocator.frozen_publish() {
        None => Ok(()),
        Some(frozen) => Err(
            PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: frozen.checkpoint_txg(),
                instance: frozen.instance(),
            },
        ),
    }
}

/// 把装好、还没落盘的一次发布落盘（经分配器的两条发布路径共用：带文件的一版、树表 0 条的一版上写行）：
/// 成功就把这次落盘的写账填进交回的那一版；落盘那几步里失败，这次已记的写进写入口的失败账（增补 2 第 20b 行），
/// 分配器换回发布之前的样子，这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
///
/// # Errors
/// 块设备报的错（`PublishError::BlockDevice`），这时这次发布已冻结。
fn persist_the_publish_or_freeze_it<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    allocator_before_this_publish: PoolAllocator,
    writes: PublishWrites,
    version: PoolVersion,
) -> Result<PoolVersion, PublishError> {
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    if let Err(cause) = persist_publish_writes(pool, &writes) {
        pool.count_failed_publish(&writes_before_this_publish);
        let allocator_after_the_publish =
            std::mem::replace(allocator, allocator_before_this_publish);
        allocator.freeze_publish(FrozenPublish {
            writes,
            version,
            allocator_after_the_publish,
        });
        return Err(PublishError::BlockDevice(cause));
    }
    Ok(version.with_writes(
        pool.writes_by_structure_kind
            .since(&writes_before_this_publish),
    ))
}

/// 把分配器上冻结着的那次发布逐字节原样重发（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：单元、journal 记录、
/// 根槽照冻结时装好的字节再发一遍（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置都不变），系统配置槽照常轮换。
/// 成功就把分配器换成这次发布成立之后的那一份、交回那一版（写账是重发这一遍真写出去的）；没有冻结着的发布就什么都不写、交回 `None`。
///
/// # Errors
/// 重发那几步里又有一步报错（`PublishError::BlockDevice`）：这一遍已记的写进失败账，那次发布照旧冻结着。
pub fn resend_the_frozen_publish<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
) -> Result<Option<PoolVersion>, PublishError> {
    let Some(frozen) = allocator.take_frozen_publish() else {
        return Ok(None);
    };
    let writes_before_the_resend = pool.writes_by_structure_kind.clone();
    if let Err(cause) = persist_publish_writes(pool, &frozen.writes) {
        pool.count_failed_publish(&writes_before_the_resend);
        allocator.freeze_publish(frozen);
        return Err(PublishError::BlockDevice(cause));
    }
    let FrozenPublish {
        writes: _,
        version,
        allocator_after_the_publish,
    } = frozen;
    *allocator = allocator_after_the_publish;
    Ok(Some(
        version.with_writes(
            pool.writes_by_structure_kind
                .since(&writes_before_the_resend),
        ),
    ))
}

/// 发布的最后两步（D16（发布语义） 已定项 7 的持久顺序）：根槽 FUA 写 → 系统配置槽轮换。
/// 三条发布路径（带单元的、零单元的、树表 0 条上只写实例表的）共用这一处，不各抄一份——
/// 抄出来的三份会分叉，而「根槽在系统配置槽之前」正是崩溃窗口那几格的前提。
fn persist_the_root_then_rotate_the_system_configuration<Device: BlockDevice>(
    writer: &mut PoolWriter<'_, Device>,
    checkpoint_txg: CheckpointTxg,
    root_slot: &[u8],
    journal_tail: u64,
    journal_instance: InstanceGeneration,
) -> Result<(), BlockDeviceError> {
    writer.perform(CommitStep::WriteRootRecordForceUnitAccess {
        checkpoint_txg,
        root_slot,
    })?;
    writer.perform(CommitStep::RotateSystemConfigurationSlots {
        journal_tail,
        journal_instance,
    })
}

/// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号、回退下界
/// 与树 ID 水位（取计划里给的，D8（核心索引结构） 已定项 8 ②）。
/// 第一次可写挂载的暖机与「只做过 mkfs 的池」上的可写挂载都走它（第一个事务的字节不变）。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn publish_without_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> Result<VersionWithoutFilePublishOutput, BlockDeviceError> {
    let (writes, output) = prepare_the_publish_without_units(pool, previous_root, plan);
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：失败在这里记账再把错原样交回。零单元发布不冻结（见 `FrozenPublish`）。
    if let Err(cause) = persist_publish_writes(pool, &writes) {
        pool.count_failed_publish(&writes_before_this_publish);
        return Err(cause);
    }
    let VersionWithoutFilePublishOutput {
        root,
        record,
        record_bytes,
        earlier_records_of_this_publish,
        rewritten,
        writes: _nothing_written_before_persisting,
    } = output;
    Ok(VersionWithoutFilePublishOutput {
        root,
        record,
        record_bytes,
        earlier_records_of_this_publish,
        rewritten,
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_publish),
    })
}

/// 零单元发布落盘之前的全部：装好记录与根，交回要交给设备的字节与这一版（写账是空的）。发布路径（[`publish_without_units`]）
/// 与可写挂载取号之前的预演（`mount`）读的是同一份。
#[must_use]
pub fn prepare_the_publish_without_units<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> (PublishWrites, VersionWithoutFilePublishOutput) {
    let record = JournalRecord {
        instance: plan.instance,
        counter: plan.counter,
        checkpoint_txg: plan.txg,
        transaction: 0,
        is_commit: true,
        // 空发布记录也写 1（D23（journal 的角色与格式） 已定项 4）：它一条就是一次发布。
        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
        // 它一条就是一次发布，也就是这次发布的末条（D23（journal 的角色与格式） 已定项 17：每次只有一条记录的发布，
        // 含空发布记录，那一条记录标志位 0 也写 1）。
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
        new_tree_table: previous_root.tree_table,
        new_mapping_root: previous_root.mapping_root,
        new_tree_identifier_watermark: plan.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named: Vec::new(),
    };
    let record_bytes = record.to_bytes();
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance: plan.instance,
        checkpoint_txg: plan.txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: previous_root.instance_table,
        mapping_root: previous_root.mapping_root,
        // 零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针，账也因此一条都没变。
        allocation_record_tree_root: previous_root.allocation_record_tree_root,
    };
    // 零单元：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换（单元那一段是空的，头一道屏障前面没有写，写入口不发它）。
    let writes = PublishWrites {
        units: Vec::new(),
        records: vec![JournalRecordWrite {
            counter: plan.counter,
            bytes: record_bytes.clone(),
        }],
        checkpoint_txg: plan.txg,
        root_slot: root.to_slot(pool.root_slot_bytes()),
        journal_tail: plan.counter,
        journal_instance: plan.instance,
    };
    (
        writes,
        VersionWithoutFilePublishOutput {
            root,
            record,
            record_bytes,
            earlier_records_of_this_publish: Vec::new(),
            rewritten: Vec::new(),
            writes: WritesByStructureKind::NOTHING_WRITTEN,
        },
    )
}

/// 写行那次发布的参数，树表 0 条的一版上（D18（块里携带什么信息） 已定项 11「每次可写挂载都写行」）：身份字段同零单元发布，
/// 另带这次整条实例表链怎么重写（这次之后整张表的行——调用方在上一版那张表后面接上这次的行——与被换下的那条旧链）。
#[derive(Clone, Copy, Debug)]
pub struct InstanceTableOnlyPublishPlan<'records> {
    pub txg: CheckpointTxg,
    /// jsn 计数器，全池接着走（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    pub counter: u64,
    pub instance: InstanceGeneration,
    /// 反向链：本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub rollback_floor: CheckpointTxg,
    pub instance_table: &'records InstanceTableRewrite,
    /// 这次写进根记录与记录新根段的树 ID 水位，同 `ZeroUnitPublishPlan::tree_identifier_watermark`：写行那次发布一个树 ID 都不发，
    /// 就是根环里全部根记录的 max（D8（核心索引结构） 已定项 8 ②）——回退到树表 0 条的一版时它高于那一版自己带的。
    pub tree_identifier_watermark: u64,
}

/// 写行那次发布，上一版树表 0 条：**重写实例表链与分配记录树**（链有几片就几个实例表单元，加分配记录树这次内容变了的节点），
/// 落盘顺序同别的发布（D16（发布语义） 已定项 7）——单元写 → 屏障 → journal 记录（点名这几个单元）→ 屏障 → 根槽 FUA → 系统配置槽轮换。
///
/// 为什么是这两样：树表 0 条 ⇒ 这一版没有记账树，D16（发布语义） 已定项 9 那五样（记账行、记账树节点、映射条目、
/// 树表单元、树表条目）一样都不写；而 D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行、写行 COW 重写整条链 ⇒ 实例表自己那几个单元
/// 必写。**分配记录树是第二样**：写行换下上一版那条实例表链，这条释放要有地方记，不然重开之后只能从根记录那两条指针重建账、
/// 那几片就成了空闲槽，而根环里 txg 更低的候选根还指着它们（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪））。
/// 它的根指针住**根记录**新加的那一项，不进树表——进树表 `tree_table_has_no_entries` 当场翻面，
/// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变。这棵树照 D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
/// 树号 0（这一版还没登记过任何树），节点豁免映射；重写哪几个节点照带文件那一路走到固定点
/// （[`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`]），没变的节点照抄上一版的。
/// 落点照 D3（空间分配） 已定项 5 从聚簇段 bump、按已定项 10 ⑤ 各自那一档取（实例表各片最前、尾片先，分配记录树节点在后、先叶后根），
/// 与这几个角色在别的发布路径上走的是同一条规则。
///
/// 被换下的（上一版的整条实例表链、上一版那棵分配记录树这次重写的节点）在同一次发布里释放（释放先于分配，D3（空间分配） 已定项 7）：
/// 两样都豁免映射（实例表见 D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12；这一版的分配记录树根住根记录），
/// 实例表链的各片取计划带着的那条旧链（`instance_table_chain_to_release`），分配记录树的节点取分配器记着的那一版那棵树的指针，
/// 三样逐盘核过（`placement_to_release_after_checking_every_device`）。上一版是 mkfs 的第 0 代（分配记录树根指针全零）时只释放实例表。
///
/// 新根照上一版的根，只换 checkpoint_txg、实例代号、回退下界、实例表指针、分配记录树根指针与树 ID 水位（取计划里给的，
/// D8（核心索引结构） 已定项 8 ②）：树表、映射树根照抄（这一版的树表仍是 mkfs 那片 0 条的）。
///
/// # Errors
/// 分配器上冻结着一次没重发的发布（`PublishFrozenAfterAWriteFailureIsNotResentYet`）、
/// 被换下的任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、落点取不到（`PlacementRefused`）、
/// 分配记录树重写集的固定点没定下来（`AllocationRecordTreeRewriteSetDidNotSettle`）、块设备报错。
/// 前几样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子、这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
///
/// # Panics
/// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
pub fn publish_instance_table_on_version_without_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous_root: &RootRecord,
    plan: InstanceTableOnlyPublishPlan<'_>,
) -> Result<VersionWithoutFilePublishOutput, PublishError> {
    refuse_while_a_publish_is_frozen(allocator)?;
    // 失败就换回去：取落点会动分配器（bump 指针、位图、记录），中途报错不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let built =
        prepare_the_row_publish_on_a_version_without_file(pool, allocator, previous_root, plan);
    let (writes, output) = match built {
        Ok(built) => built,
        Err(refusal) => {
            *allocator = allocator_before_this_publish;
            return Err(refusal);
        }
    };
    persist_the_publish_or_freeze_it(
        pool,
        allocator,
        allocator_before_this_publish,
        writes,
        PoolVersion::WithoutFile(output),
    )
    .map(|version| {
        version.into_version_without_file().expect(
            "交给落盘的是树表 0 条的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）",
        )
    })
}

/// 树表 0 条那一版写行时这一次的样子：分配记录树这次之后的计划、这次重写的角色（实例表各片尾片先，再分配记录树重写的节点，先叶后根）、
/// 要释放的落点（整条旧链在前，再是分配记录树换下的节点）。
struct SettledRowPublishOnAVersionWithoutFile {
    allocation_record_tree: AllocationRecordTreePlan,
    rewritten_roles: Vec<TransactionUnit>,
    release: Vec<Placement>,
}

/// 树表 0 条那一版写行时，分配记录树重写哪几个节点（固定点，同带文件那一路的 [`settle_the_allocation_record_tree`]）：上一版的树是分配器记着的
/// 那一棵（`PoolAllocator::allocation_record_tree_of_the_version_without_file`；mkfs 的第 0 代没有树，节点全新），换下的节点逐盘核三样
/// （`placement_to_release_after_checking_every_device`，只查不改）。在分配器的拷贝上走，不动 `allocator`。
///
/// # Errors
/// 旧链或换下的节点三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、拷贝上取不到落点（`PlacementRefused`）、
/// 轮数用完（`AllocationRecordTreeRewriteSetDidNotSettle`）。
///
/// # Panics
/// 上一版根记录那一项不是全零，而分配器没记着那一版的分配记录树，或记着的那一棵的根不是那一项指的：分配器的这一项由同一进程的写行
/// 或挂载时整棵读回来记下（`mount`），与根记录说的是同一版。
fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
    previous_root: &RootRecord,
    instance_table: &InstanceTableRewrite,
    txg: CheckpointTxg,
    allocator: &PoolAllocator,
) -> Result<SettledRowPublishOnAVersionWithoutFile, PublishError> {
    let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let previous_tree = if previous_root.allocation_record_tree_root == NodePointer::empty_root() {
        None
    } else {
        let tree = allocator
            .allocation_record_tree_of_the_version_without_file()
            .expect("根记录那一项不是全零时，分配器记着那一版的分配记录树（同一进程的写行或挂载时读回来的）");
        assert_eq!(
            tree.version.root_pointer(),
            previous_root.allocation_record_tree_root,
            "分配器记着的那一棵就是根记录那一项指的那一棵"
        );
        Some(tree)
    };
    let previous_nodes: BTreeSet<AllocationRecordTreeNode> = previous_tree
        .map(|tree| tree.version.node_set())
        .unwrap_or_default();
    let previous_records: &[AllocationRecord] =
        previous_tree.map_or(&[], |tree| tree.records.as_slice());
    let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;
    let instance_table_roles =
        instance_table_page_roles_in_bump_order(instance_table.pages_after_this_publish());
    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
    // 迭代上界与跨轮携带的同 `settle_the_allocation_record_tree`。
    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
            rewritten.intersection(&nodes_after).copied().collect();
        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
            &previous_nodes,
            &nodes_after,
            &rewritten_among_the_nodes_after,
        );
        let rewritten_by_the_plan: BTreeSet<AllocationRecordTreeNode> =
            allocation_plan.rewritten_nodes().into_iter().collect();
        let mut release = chain.clone();
        for node in &allocation_plan.replaced_previous_nodes {
            let pointer = previous_tree
                .and_then(|tree| tree.version.pointer_of(*node))
                .expect("换下的节点在上一版那棵树里");
            release.push(placement_to_release_after_checking_every_device(
                role_of_allocation_record_tree_node(*node),
                &pointer.locations,
                allocator,
            )?);
        }
        let rewritten_roles: Vec<TransactionUnit> = instance_table_roles
            .iter()
            .copied()
            .chain(
                allocation_plan
                    .rewritten_nodes()
                    .into_iter()
                    .map(role_of_allocation_record_tree_node),
            )
            .collect();
        let mut rehearsal = allocator.clone();
        for placement in &release {
            rehearsal.release(*placement, txg);
        }
        // 释放本身弄脏的叶先并进重写集再取落点（同 `settle_the_allocation_record_tree`）。
        let changed_by_the_release =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
            rewritten.extend(changed_by_the_release);
            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
            continue;
        }
        for identity in &rewritten_roles {
            allocate_placement_for_role(&mut rehearsal, *identity, txg)?;
        }
        let changed =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        let holding_records = nodes_holding_records(&geometry, rehearsal.records());
        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
            return Ok(SettledRowPublishOnAVersionWithoutFile {
                allocation_record_tree: allocation_plan,
                rewritten_roles,
                release,
            });
        }
        rewritten.extend(changed);
        nodes_after = holding_records;
    }
    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
    })
}

/// 这次重写出去的实例表链上的一片：角色、单元字节、指着它的指针（第 0 片的进根记录，第 k 片的进第 k − 1 片的链指针记录）。
struct InstanceTablePageUnit {
    role: TransactionUnit,
    bytes: Vec<u8>,
    pointer: NodePointer,
}

/// 装一条实例表链（D18（块里携带什么信息） 已定项 11）：行按链上的次序一片写满 369 行再开下一片、最后一片装剩下的
/// （`instance_table_rows_of_each_page`，用户 2026-09-24 定案），每片末尾一条链指针记录——最后一片写「无下一片」，别的片写下一片的指针。
/// **尾片先装、先发出生序号**（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：第 k 片当第 k + 1 片的父，照先叶后根——第 k 片的
/// 链指针记录要第 k + 1 片的落点、整单元校验和与出生序号，第 k + 1 片装好了它才装得出来）。落点由调用方先按同一个次序取好
/// （`instance_table_page_roles_in_bump_order`），这里按角色查。交回的各片按链上的次序，第 0 片在前。
/// 带文件的一版上写行（`publish_admitted`）与树表 0 条的一版上写行（`prepare_the_row_publish_on_a_version_without_file`）共用这一处：
/// 两处各装一份，片的切法与发号次序会分叉。
///
/// # Panics
/// 某一片的角色没取落点（`slot_of` 里的断言）：调用方按同一张行表的片数取的落点。
fn build_instance_table_chain(
    rows: &[InstanceRow],
    slot_of: &dyn Fn(TransactionUnit) -> SlotNumber,
    txg: CheckpointTxg,
    write_order: WriteOrder,
    filesystem_identifier: &[u8; 16],
    device_identities: &[DeviceIdentity],
    sequences: &mut BirthSequenceAllocator,
) -> Vec<InstanceTablePageUnit> {
    let rows_of_each_page = instance_table_rows_of_each_page(rows);
    let mut pages_tail_first: Vec<InstanceTablePageUnit> =
        Vec::with_capacity(rows_of_each_page.len());
    // 迭代次数的上界是片数；跨轮带的只有后一片的指针（前一片的链指针记录要它），没有提前出口。
    let mut pointer_to_the_next_page: Option<NodePointer> = None;
    for (position, page_rows) in rows_of_each_page.iter().enumerate().rev() {
        let page = InstanceTablePageIndex(u64::try_from(position).expect("片序号"));
        let role = TransactionUnit::of_instance_table_page(page);
        let chain = match pointer_to_the_next_page {
            None => InstanceTableChainRecord::LastPage,
            Some(next_page) => InstanceTableChainRecord::NextPage(next_page),
        };
        let birth_sequence = sequences.next(
            TreeIdentifier(TREE_IDENTIFIER_NONE),
            txg,
            write_order.instance,
        );
        let bytes = build_packed_unit(
            page.packed_identity(),
            u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
            &instance_table_page_records(page_rows, &chain),
            txg,
            filesystem_identifier,
            write_order,
            birth_sequence,
        );
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                birth_txg: txg,
            },
            locations: location_entries(device_identities, slot_of(role), &bytes),
            instance: write_order.instance,
            birth_sequence,
        };
        pointer_to_the_next_page = Some(pointer);
        pages_tail_first.push(InstanceTablePageUnit {
            role,
            bytes,
            pointer,
        });
    }
    pages_tail_first.reverse();
    pages_tail_first
}

/// 树表 0 条那一版上写行那次发布，落盘之前的全部：走分配记录树重写集的固定点、释放、取落点、装单元，交回装好、还没落盘的字节与这一版
/// （写账是空的，落盘由调用方做）。只动 `allocator`：失败时它停在中途，由调用方换回去——发布路径换回进来时那一份，
/// 可写挂载取号之前的预演拿的本来就是拷贝（`mount`，D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取：预演里写行那次报错，
/// 同样判这次不能可写）。两处走的是同一段代码。
///
/// # Errors
/// 旧链或换下的分配记录树节点三样核不过、落点取不到、固定点没定下来（见 [`publish_instance_table_on_version_without_file`]）。
///
/// # Panics
/// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous_root: &RootRecord,
    plan: InstanceTableOnlyPublishPlan<'_>,
) -> Result<(PublishWrites, VersionWithoutFilePublishOutput), PublishError> {
    assert_eq!(
        plan.instance_table.replaced_chain.first(),
        Some(&previous_root.instance_table),
        "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
    );
    let txg = plan.txg;
    let instance = plan.instance;
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let write_order = WriteOrder {
        instance,
        // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①：空发布与写行的记录写事务号 0）。
        transaction: 0,
    };
    let settled = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
        previous_root,
        plan.instance_table,
        txg,
        allocator,
    )?;
    let previous_tree_pointers = allocator
        .allocation_record_tree_of_the_version_without_file()
        .filter(|_| previous_root.allocation_record_tree_root != NodePointer::empty_root())
        .map(|tree| tree.version.clone());
    for released in settled.release.iter().copied() {
        allocator.release(released, txg);
    }
    // 实例表链的各片最前、尾片先（D3（空间分配） 已定项 10 ⑤），分配记录树的节点在它们之后按位置先叶后根取：
    // 这些节点自己的分配记录也要进这棵树，所以落点都取完才装节点。
    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
    for identity in &settled.rewritten_roles {
        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
        slots.insert(*identity, placement.slot);
    }
    // 落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去）。
    allocator.record_root_written_by_this_process(txg);
    // 这一次发布的提交内生块（实例表链的各片、分配记录树的节点）都归树 0（这一版还没登记过任何树），在 (txg, 实例) 上连着发出生序号：
    // 实例表各片按 bump 次序（尾片先）在前，分配记录树的节点在后（先叶后根）。
    let mut sequences = BirthSequenceAllocator::default();
    let device_identities: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let instance_table_pages = build_instance_table_chain(
        &plan.instance_table.rows,
        &|identity| slots[&identity],
        txg,
        write_order,
        filesystem_identifier,
        &device_identities,
        &mut sequences,
    );
    let instance_table_pointer = instance_table_pages
        .first()
        .expect("一条链至少一片：0 行也写出空的那一片")
        .pointer;
    // 分配记录树：树号 0（这一版还没登记过任何树，写 13 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
    // 由根记录独占持有，与树表单元、实例表单元同一个身份（D22（单元原子性怎么合成） 已定项 7 / 已定项 12）。
    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let allocation_built = build_allocation_record_tree(
        &settled.allocation_record_tree,
        &allocation_geometry,
        &allocation_records,
        &|node| {
            previous_tree_pointers
                .as_ref()
                .and_then(|version| version.pointer_of(node))
        },
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        &MultiLevelTreeBuildContext {
            txg,
            filesystem_identifier,
            instance,
            device_identities: &device_identities,
        },
        &|identity| slots[&identity],
        &mut sequences,
    );
    let allocation_record_tree_root = allocation_built.version.root_pointer();
    // 这一版的分配记录树换成了刚装的这一棵：下一次发布（再写一次行，或在这一版上发第一个文件版本）要照抄或换下它的节点，
    // 而那时手里只有这个分配器——记的要是上一版那一棵（这次刚换下几个节点的那一棵），新写的节点就永远没人释放，
    // I-3.1（已分配统计对得上） 在抬 F 之后当场红。
    allocator.note_allocation_record_tree_of_the_version_without_file(
        crate::allocator::AllocationRecordTreeOfTheVersionWithoutFile {
            version: allocation_built.version.clone(),
            records: allocation_records,
        },
    );
    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树重写的节点。
    let named_roles: Vec<TransactionUnit> = settled.rewritten_roles.clone();
    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
        match identity {
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_) | TransactionUnit::AllocationTree => {
                let unit = allocation_built
                    .rewritten_units
                    .iter()
                    .find(|unit| unit.identity == identity)
                    .expect("点名的分配记录树节点是这次重写的");
                NamedUnit {
                    locations: pool.location_entries(unit.slot, &unit.bytes),
                    unit_class: identity.unit_class(),
                    // 树 ID 0：这一版还没登记过任何树，这棵树由根记录独占持有。
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    birth_txg: txg,
                    key_tail: node_key_tail(
                        instance,
                        allocation_built.birth_sequences_of_rewritten_nodes[&identity],
                    ),
                }
            }
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                let page = instance_table_pages
                    .iter()
                    .find(|page| page.role == identity)
                    .expect("每个取了落点的实例表角色都装了一片");
                NamedUnit {
                    locations: page.pointer.locations,
                    unit_class: identity.unit_class(),
                    // 实例表单元不属于任何一棵树（树 0）；这一版还没发过树 ID，不按 `TransactionUnit::tree` 取。
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    birth_txg: txg,
                    key_tail: node_key_tail(instance, page.pointer.birth_sequence),
                }
            }
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => unreachable!(
                "树表 0 条的一版上写行只点名实例表链各片与分配记录树节点（`named_roles` 就是这两样）"
            ),
        }
    };
    // 这次发布切成几条记录（D23（journal 的角色与格式） 已定项 17「末条再跨记录」，与带文件那一路同一个切法：
    // `roles_named_by_each_record_of_the_publish`）：点名项多于一条记录装得下的（实例表长到 67 片起）就装满一条再开下一条。
    // 写行那次发布不承载事务，这几条都是事务号 0、同属一个事务（`transaction_offset_of_each_record_of_the_publish` 全是 0）：
    // 提交标记只在它的最后一条上（已定项 7），「本次发布末条」标志只在真正的最后一条上（已定项 17），本次发布内序号依次 1..N（已定项 4），
    // 反向链第一条接 `plan.back_chain`、之后每条接这次发布里前一条的头（已定项 8），每条都带整次发布的新根段（已定项 15）。
    let named_entry_capacity = pool.journal_record_named_entry_capacity();
    let roles_named_by_each_record =
        roles_named_by_each_record_of_the_publish(&named_roles, named_entry_capacity);
    let transaction_offset_of_each_record =
        transaction_offset_of_each_record_of_the_publish(&named_roles, named_entry_capacity);
    let mut written_records: Vec<WrittenJournalRecord> =
        Vec::with_capacity(roles_named_by_each_record.len());
    // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
    for (record_offset_in_this_publish, roles_of_this_record) in
        roles_named_by_each_record.iter().enumerate()
    {
        let back_chain = match written_records.last() {
            None => plan.back_chain,
            Some(previous_record_of_this_publish) => {
                back_chain_of(&previous_record_of_this_publish.bytes)
            }
        };
        let transaction_offset = transaction_offset_of_each_record[record_offset_in_this_publish];
        let transaction_of_the_next_record = transaction_offset_of_each_record
            .get(record_offset_in_this_publish + 1)
            .copied();
        let record = JournalRecord {
            instance,
            counter: plan.counter + u64::try_from(record_offset_in_this_publish).expect("记录序号"),
            checkpoint_txg: txg,
            // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①）：事务号 0 加这条属于的事务序号（恒 0）。
            transaction: transaction_offset,
            is_commit: transaction_of_the_next_record != Some(transaction_offset),
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(
                record_offset_in_this_publish,
            ),
            place_in_publish: if transaction_of_the_next_record.is_none() {
                JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            },
            back_chain,
            filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
            new_tree_table: previous_root.tree_table,
            new_mapping_root: previous_root.mapping_root,
            new_tree_identifier_watermark: plan.tree_identifier_watermark,
            new_rollback_floor: plan.rollback_floor,
            named: roles_of_this_record
                .iter()
                .copied()
                .map(named_unit_of)
                .collect(),
        };
        let bytes = record.to_bytes();
        written_records.push(WrittenJournalRecord { record, bytes });
    }
    let last_counter_of_this_publish =
        plan.counter + u64::try_from(written_records.len() - 1).expect("一次发布至少一条记录");
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance,
        checkpoint_txg: txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: previous_root.mapping_root,
        allocation_record_tree_root,
    };
    // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树重写的节点（先叶后根）最后。
    let units: Vec<PublishedUnit> = settled
        .rewritten_roles
        .iter()
        .filter(|identity| identity.is_a_page_of_the_instance_table())
        .map(|identity| {
            let page = instance_table_pages
                .iter()
                .find(|page| page.role == *identity)
                .expect("每个取了落点的实例表角色都装了一片");
            PublishedUnit {
                slot: slots[identity],
                identity: *identity,
                bytes: page.bytes.clone(),
            }
        })
        .chain(allocation_built.rewritten_units.iter().cloned())
        .collect();
    let writes = PublishWrites {
        units,
        records: written_records
            .iter()
            .map(|written_record| JournalRecordWrite {
                counter: written_record.record.counter,
                bytes: written_record.bytes.clone(),
            })
            .collect(),
        checkpoint_txg: txg,
        root_slot: root.to_slot(pool.root_slot_bytes()),
        // 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
        journal_tail: last_counter_of_this_publish,
        journal_instance: instance,
    };
    let WrittenJournalRecord {
        record,
        bytes: record_bytes,
    } = written_records
        .pop()
        .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
    Ok((
        writes,
        VersionWithoutFilePublishOutput {
            root,
            record,
            record_bytes,
            earlier_records_of_this_publish: written_records,
            rewritten: settled.rewritten_roles,
            writes: WritesByStructureKind::NOTHING_WRITTEN,
        },
    ))
}

/// 一个角色这次发布的落点：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）。
/// 取不到时带上是哪个角色、分配器给的原因。
///
/// # Errors
/// `PlacementRefused`。
fn allocate_placement_for_role(
    allocator: &mut PoolAllocator,
    identity: TransactionUnit,
    txg: CheckpointTxg,
) -> Result<Placement, PublishError> {
    let placement = match identity.placement() {
        PlacementRule::UserData => allocator.try_allocate_user_data(txg),
        PlacementRule::CommitGenerated(footprint) => {
            allocator.try_allocate_commit_generated(footprint, txg)
        }
    };
    placement.map_err(|refusal| PublishError::PlacementRefused {
        unit: identity,
        refusal,
    })
}

/// 池里现行的那一版：还没发布过文件版本（树表 0 条，零单元发布或写行那次发布写出的根），或带文件的一版。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有几个这样的值、不进集合，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
pub enum PoolVersion {
    WithoutFile(VersionWithoutFilePublishOutput),
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
    /// 树表 0 条的那一版；带文件的一版时 `None`。
    #[must_use]
    pub fn into_version_without_file(self) -> Option<VersionWithoutFilePublishOutput> {
        match self {
            PoolVersion::WithoutFile(version) => Some(version),
            PoolVersion::WithFile(_) => None,
        }
    }
    /// 这一版的写账：这次发布交给设备、设备报了成功的写（落盘或重发那一遍之后现记）。
    #[must_use]
    pub fn writes(&self) -> &WritesByStructureKind {
        match self {
            PoolVersion::WithoutFile(version) => &version.writes,
            PoolVersion::WithFile(version) => &version.writes,
        }
    }
    /// 换上写账、别的一个字段都不动（成员不换）。
    #[must_use]
    fn with_writes(mut self, writes: WritesByStructureKind) -> Self {
        match &mut self {
            PoolVersion::WithoutFile(version) => version.writes = writes,
            PoolVersion::WithFile(version) => version.writes = writes,
        }
        self
    }
}

/// 第一个事务写出的八个单元，声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
/// 也是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）与字节表七的 t1..t8。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionUnit {
    /// 文件的一个数据单元（码 1）。带的是它在文件里的单元序号：一个文件跨多个单元时（里程碑「第二个事务」并行线一）
    /// 每个单元一个角色，落点、映射 key、点名项与按结构种类记的账都按角色走，共用一个角色就分不开两个单元。
    /// 只有一个单元的文件（第一个事务那一档）就是 [`DataUnitIndexInFile::FIRST`]。
    Data(DataUnitIndexInFile),
    /// 第一个文件那一棵 extent 树下段里的一个节点（D8（核心索引结构） 已定项 14：下段一个文件一棵、按数据单元号按位置寻址），
    /// 带它在下段里的位置（层级、同层序号）；下段的根也是这一族（最高那一层的第 0 个）。只有一个数据单元的文件不建下段（内联），没有这一族。
    ExtentLowerNode(ExtentLowerNodePosition),
    /// extent 树上段里根之下的一个节点（按 inode 号按位置寻址），带它在上段里的位置。上段只有一层时没有这一族（根兼叶是 `ExtentRoot`）。
    ExtentUpperNodeBelowTheRoot(ExtentUpperNodePosition),
    /// extent 树的根：上段的根（树表条目里 extent 树那一条指着它）。
    ExtentRoot,
    /// inode 树的一片叶容器（码 3 打包记录类型 2，一容器 233 条 140 字节记录，D8（核心索引结构） 已定项 6）。
    /// 带的是它在树里的叶序，不是容器号：一棵树可以有好几片，角色按叶序分开
    /// （落点、映射 key、点名项、按结构种类记的账都按角色走，共用一个角色就分不开两片叶）。
    InodeLeafContainer(InodeLeafContainerIndexInTree),
    InodeRoot,
    /// 分配记录树里根之下的一个节点（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址），带它的位置（层级、盘、同盘同层序号）。
    /// 位置由槽号算出、不随版本挪动：上一版同一个位置上的节点照抄进来时角色不变。
    AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition),
    /// 分配记录树的根（罩整个 key 空间；带文件的一版里树表条目指着它，树表 0 条的一版里根记录那一项指着它）。
    AllocationTree,
    /// 记账树里根之下的一个节点（D8（核心索引结构） 已定项 11：多层码 2 树）：带它在这一版树里的位置（层级、同层从左数第几个）。
    /// 根兼叶那一档没有这个角色——记账树只有一个节点时它就是 `AccountingTree`。位置是这一版的：上一版的节点照抄进来时
    /// 位置可以变（它左边的叶切开了），角色跟着这一版的位置走（`code_two_tree::CodeTwoTreePlan::origins` 记着它从哪来）。
    AccountingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
    /// 记账树的根（只有一个节点时它就是根兼叶）。
    AccountingTree,
    /// 中央映射树里根之下的一个节点，同 `AccountingTreeNodeBelowTheRoot`。映射树的节点都豁免映射
    /// （D19（块指针的结构与宽度预算） 已定项 8「映射树自己的节点」）：父条目里的位置条目是权威。
    MappingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
    /// 中央映射树的根（只有一个节点时它就是根兼叶），根指针住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
    MappingTree,
    TreeTable,
    /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
    /// 不在第一个事务的八个角色里，第二个事务起才进发布路径。它是链上第 0 片——根记录直接持有的那一片；
    /// 多于一片时第 1 片起是 [`TransactionUnit::InstanceTablePageAfterTheFirst`]。
    InstanceTable,
    /// 实例表链上第 1 片起的一片（码 3 打包记录类型 4，身份 (0, 4, 片序号, 0)，D18（块里携带什么信息） 已定项 11）：它的指针不在根记录里，
    /// 在上一片末尾那条链指针记录里。带的是片序号，恒 ≥ 1（第 0 片是 [`TransactionUnit::InstanceTable`]，
    /// 两者由 [`TransactionUnit::of_instance_table_page`] 按片序号分）。表的行多于一片装得下的 369 行时写行那次发布才写出它们，
    /// 在 bump 次序里尾片先（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案）。
    InstanceTablePageAfterTheFirst(InstanceTablePageIndex),
}

/// 多层码 2 树里哪一棵：记账树或中央映射树（D8（核心索引结构） 已定项 11 的两棵照分裂做的树）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultiLevelCodeTwoTree {
    Accounting,
    CentralMapping,
}

impl MultiLevelCodeTwoTree {
    /// 这棵树在这一版里某个位置上的节点的角色：根是 `AccountingTree` / `MappingTree`，根之下各按位置。
    #[must_use]
    pub fn role_of_node(
        self,
        position: CodeTwoTreeNodePosition,
        shape: &CodeTwoTreeShape,
    ) -> TransactionUnit {
        match (self, shape.is_root(position)) {
            (MultiLevelCodeTwoTree::Accounting, true) => TransactionUnit::AccountingTree,
            (MultiLevelCodeTwoTree::Accounting, false) => {
                TransactionUnit::AccountingTreeNodeBelowTheRoot(position)
            }
            (MultiLevelCodeTwoTree::CentralMapping, true) => TransactionUnit::MappingTree,
            (MultiLevelCodeTwoTree::CentralMapping, false) => {
                TransactionUnit::MappingTreeNodeBelowTheRoot(position)
            }
        }
    }

    /// 叶条目的字段表宽：记账条目 34、映射条目 55。
    #[must_use]
    pub fn leaf_entry_width_in_bytes(self) -> usize {
        match self {
            MultiLevelCodeTwoTree::Accounting => {
                usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34")
            }
            MultiLevelCodeTwoTree::CentralMapping => {
                usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
            }
        }
    }

    #[must_use]
    pub const fn key_field_widths(self) -> CodeTwoKeyFieldWidths {
        match self {
            MultiLevelCodeTwoTree::Accounting => CodeTwoKeyFieldWidths::ACCOUNTING,
            MultiLevelCodeTwoTree::CentralMapping => CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
        }
    }

    /// 从盘上读这棵树时对它的期望（`code_two_tree::read_code_two_tree`）：这一版里它的树 ID 由调用方给（从水位发的号）。
    #[must_use]
    pub fn read_expectation(self, tree_identifier: TreeIdentifier) -> CodeTwoTreeReadExpectation {
        CodeTwoTreeReadExpectation {
            tree: tree_identifier,
            field_widths: self.key_field_widths(),
            internal_entry_name: match self {
                MultiLevelCodeTwoTree::Accounting => "记账树内部条目",
                MultiLevelCodeTwoTree::CentralMapping => "中央映射树内部条目",
            },
        }
    }

    /// 格式算出来的节点容量：记账树叶 477、内部 150，中央映射树叶 294、内部 143。
    #[must_use]
    pub fn node_capacity_of_the_node_format(self) -> CodeTwoTreeNodeCapacity {
        CodeTwoTreeNodeCapacity::of_the_node_format(
            self.key_field_widths().key_width_in_bytes(),
            self.leaf_entry_width_in_bytes(),
        )
    }
}

/// 记账树与中央映射树的节点容量从哪来：产品路径按节点格式算；只供测试的开关把两棵树各自压小
/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入——格式算出来的容量下，记账树要 80 块盘才分裂，
/// 中央映射树今天走得到的条目数装不满一个节点，分裂、收缩、降高这几条写序不压小就进不去）。只住写入口的内存：重开之后按产品路径起步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTwoTreeNodeCapacities {
    FromTheNodeFormat,
    CappedForTests {
        accounting: CodeTwoTreeNodeCapacity,
        central_mapping: CodeTwoTreeNodeCapacity,
    },
}

impl CodeTwoTreeNodeCapacities {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            CodeTwoTreeNodeCapacities::FromTheNodeFormat => {
                "code_two_tree_node_capacities_from_the_node_format"
            }
            CodeTwoTreeNodeCapacities::CappedForTests { .. } => {
                "code_two_tree_node_capacities_capped_for_tests"
            }
        }
    }

    /// 这棵树这会儿一个节点最多装几条。
    #[must_use]
    pub fn of_tree(self, tree: MultiLevelCodeTwoTree) -> CodeTwoTreeNodeCapacity {
        match (self, tree) {
            (CodeTwoTreeNodeCapacities::FromTheNodeFormat, _) => {
                tree.node_capacity_of_the_node_format()
            }
            (
                CodeTwoTreeNodeCapacities::CappedForTests { accounting, .. },
                MultiLevelCodeTwoTree::Accounting,
            ) => accounting,
            (
                CodeTwoTreeNodeCapacities::CappedForTests {
                    central_mapping, ..
                },
                MultiLevelCodeTwoTree::CentralMapping,
            ) => central_mapping,
        }
    }
}

impl TransactionUnit {
    pub const IN_BUMP_ORDER: [TransactionUnit; 8] = [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
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
    /// 数据单元同一个写法：第一个单元是字节表的 `t1`，第二个起写成 `t1+单元序号`。
    #[must_use]
    pub fn tag(self) -> String {
        match self {
            TransactionUnit::Data(DataUnitIndexInFile::FIRST) => "t1".to_string(),
            TransactionUnit::Data(index) => format!("t1+{}", index.0),
            // 字节表七只登记了 extent 树根兼叶那一档的 t2；下段与上段根之下的节点写成「t2下@层级.序号」「t2上@层级.序号」，
            // 只出现在报错与用例消息里。
            TransactionUnit::ExtentLowerNode(position) => {
                format!("t2下@{}.{}", position.level, position.index)
            }
            TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
                format!("t2上@{}.{}", position.level, position.index)
            }
            TransactionUnit::ExtentRoot => "t2".to_string(),
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST) => {
                "t3".to_string()
            }
            TransactionUnit::InodeLeafContainer(index) => format!("t3+{}", index.0),
            TransactionUnit::InodeRoot => "t4".to_string(),
            // 分配记录树根之下的节点写成「t5@层级.盘.同盘同层序号」，只出现在报错与用例消息里。
            TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => format!(
                "t5@{}.{}.{}",
                position.level, position.device.0, position.index_in_device
            ),
            TransactionUnit::AllocationTree => "t5".to_string(),
            // 字节表七只登记了根兼叶那一档的 t6 / t7；根之下的节点写成「t6@层级.同层序号」，只出现在报错与用例消息里。
            TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
                format!("t6@{}.{}", position.level, position.index_in_level)
            }
            TransactionUnit::AccountingTree => "t6".to_string(),
            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
                format!("t7@{}.{}", position.level, position.index_in_level)
            }
            TransactionUnit::MappingTree => "t7".to_string(),
            TransactionUnit::TreeTable => "t8".to_string(),
            TransactionUnit::InstanceTable => "ti".to_string(),
            TransactionUnit::InstanceTablePageAfterTheFirst(page) => format!("ti+{}", page.0),
        }
    }

    /// 实例表链上第 `page` 片的角色：第 0 片是根记录持有的 [`TransactionUnit::InstanceTable`]，第 1 片起是
    /// [`TransactionUnit::InstanceTablePageAfterTheFirst`]。
    #[must_use]
    pub fn of_instance_table_page(page: InstanceTablePageIndex) -> Self {
        if page == InstanceTablePageIndex::FIRST {
            TransactionUnit::InstanceTable
        } else {
            TransactionUnit::InstanceTablePageAfterTheFirst(page)
        }
    }

    /// 这个角色是不是实例表链上的一片（第 0 片或第 1 片起）。
    #[must_use]
    pub const fn is_a_page_of_the_instance_table(self) -> bool {
        match self {
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                true
            }
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => false,
        }
    }

    #[must_use]
    pub const fn unit_class(self) -> u8 {
        match self {
            TransactionUnit::Data(_) => UNIT_CLASS_DATA,
            TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => UNIT_CLASS_PACKED,
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => UNIT_CLASS_INDEX_NODE,
        }
    }

    /// 点名项里的归属树：这一版那几棵树各自的号（`trees`，第一个文件版本那次从水位发出来、之后照抄）；
    /// 树表单元与实例表单元不属于任何一棵树（写 0）。
    #[must_use]
    pub const fn tree(self, trees: &FileVersionTreeIdentifiers) -> TreeIdentifier {
        match self {
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot => trees.extent,
            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => trees.inode,
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree => trees.allocation_records,
            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => trees.accounting,
            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                trees.central_mapping
            }
            TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                TreeIdentifier(TREE_IDENTIFIER_NONE)
            }
        }
    }

    /// 落点怎么取：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）。
    #[must_use]
    pub const fn placement(self) -> PlacementRule {
        match self {
            TransactionUnit::Data(_) => PlacementRule::UserData,
            TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
            }
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
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

/// 带文件的一版那八棵树各自的树 ID，按 D8（核心索引结构） 已定项 11 的次序：extent、inode、分配记录、记账、中央映射、
/// livelist、稀疏旁表、deadlist。
///
/// 第一个文件版本那次发布从它要建在上面的那一版的树 ID 水位起，按这个次序连号发这八个号
/// （[`FileVersionTreeIdentifiers::issued_from_watermark`]）；之后每一版照抄，再不发号。
/// mkfs 那条流上那一版的水位是 mkfs 种下的 11，发出来就是已定项 11 登记的 11..18；回退到树表 0 条的一版之后再发，
/// 那一版的水位带着回退之前根环里的 max（D8（核心索引结构） 已定项 8 ②），发出来的号高于此前发过的每一个——号永不重发
/// （里程碑「第二个事务」增补 2 收口表第 ④ 行；C511（回退到无文件那一版之后诞生代怎么接））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileVersionTreeIdentifiers {
    pub extent: TreeIdentifier,
    pub inode: TreeIdentifier,
    pub allocation_records: TreeIdentifier,
    pub accounting: TreeIdentifier,
    pub central_mapping: TreeIdentifier,
    pub livelist: TreeIdentifier,
    pub sparse_side_table: TreeIdentifier,
    pub deadlist: TreeIdentifier,
}

/// 从水位起连号发八个树 ID 时，水位装不下：盘上读来的 8 字节水位离 `u64::MAX` 不到八个号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
    pub tree_identifier_watermark: u64,
}

impl FileVersionTreeIdentifiers {
    /// 八棵树在发号次序里各自离水位几个号：mkfs 水位 11 时发出来的就是 D8（核心索引结构） 已定项 11 登记的常量，
    /// 偏移从那几个常量现算，不另抄一份次序。
    const OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER: [u64; 8] = [
        TREE_IDENTIFIER_EXTENT - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_INODE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_ALLOCATION_RECORDS - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_ACCOUNTING - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_CENTRAL_MAPPING - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_LIVELIST - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_SPARSE_SIDE_TABLE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_DEADLIST - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    ];

    /// 从 `tree_identifier_watermark`（下一个可用号）起按 D8（核心索引结构） 已定项 11 的次序连号发八个号，
    /// 连同发完之后的下一个可用号（最大那个号 + 1）。
    ///
    /// # Errors
    /// 水位加八个号越过 `u64::MAX`（水位是盘上读来的 8 字节）⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
    pub fn issued_from_watermark(
        tree_identifier_watermark: u64,
    ) -> Result<(Self, u64), TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees> {
        let next_available_after_this_issue = tree_identifier_watermark
            .checked_add(
                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            )
            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
                tree_identifier_watermark,
            })?;
        let issued =
            Self::OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER.map(|offset_from_the_watermark| {
                TreeIdentifier(tree_identifier_watermark + offset_from_the_watermark)
            });
        let [extent, inode, allocation_records, accounting, central_mapping, livelist, sparse_side_table, deadlist] =
            issued;
        let trees = FileVersionTreeIdentifiers {
            extent,
            inode,
            allocation_records,
            accounting,
            central_mapping,
            livelist,
            sparse_side_table,
            deadlist,
        };
        assert_eq!(
            trees.highest().0 + 1,
            next_available_after_this_issue,
            "八个号连号发、最大的是 deadlist：发完之后的下一个可用号就是 mkfs 那条流上第一次发布之后的水位 19 平移过来"
        );
        Ok((trees, next_available_after_this_issue))
    }

    /// 八个号按发号次序。
    #[must_use]
    pub const fn in_issue_order(&self) -> [TreeIdentifier; 8] {
        [
            self.extent,
            self.inode,
            self.allocation_records,
            self.accounting,
            self.central_mapping,
            self.livelist,
            self.sparse_side_table,
            self.deadlist,
        ]
    }

    /// 这八个号里最大的那一个。
    #[must_use]
    pub fn highest(&self) -> TreeIdentifier {
        self.in_issue_order()
            .into_iter()
            .max()
            .expect("八个号，数组非空")
    }
}

/// 一个写出去的单元：落点、身份、字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedUnit {
    pub slot: SlotNumber,
    pub identity: TransactionUnit,
    pub bytes: Vec<u8>,
}

/// 一次发布写出的一条 journal 记录与它的 4096 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenJournalRecord {
    pub record: JournalRecord,
    pub bytes: Vec<u8>,
}

/// 第一个事务写出的东西，留给验收与探针用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionOutput {
    pub root: RootRecord,
    /// 这次发布的末条记录：带「本次发布末条」标志的那一条（D23（journal 的角色与格式） 已定项 17），
    /// 也是所选根覆盖到的最后一条（已定项 14 注 1）——下一次发布的 jsn 与反向链接在它后面。只有一条记录的发布就是那一条。
    pub record: JournalRecord,
    pub record_bytes: Vec<u8>,
    /// 这次发布末条之前的那几条，按 jsn 升序：一次发布切成 N 个事务时前 N − 1 个事务各一条、各只点名自己那个数据单元，
    /// 最后一个事务装不下一条记录时再跨出去的那几条（末条除外）也在这里（D23（journal 的角色与格式） 已定项 17）。
    /// 只有一条记录的发布、从盘上重建出来的版本都为空。
    pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
    /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（文件 / 固定点角色按 bump 次序——每个数据单元、extent 根、
    /// 每片 inode 叶容器、inode 根、四个固定点单元——实例表链的各片在末尾（按链上的次序，第 0 片在前）、mkfs 之后第一次重写之前不在）。
    pub units: Vec<PublishedUnit>,
    /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
    pub rewritten: Vec<TransactionUnit>,
    /// 这一版文件每个数据单元的指针，第 i 项是文件第 i 个单元（extent 根兼叶里的记录按同一次序排）。
    pub data_pointers: Vec<DataPointer>,
    pub mapping_keys: Vec<Vec<u8>>,
    /// 这一版的全部分配记录，按 (设备, 槽号) 升序（它们分装在分配记录树的各片叶里）。
    pub allocation_records: Vec<AllocationRecord>,
    /// 这一版分配记录树的每个节点与它的指针（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）；节点的字节住 `units` 里分配记录树那一族角色。
    pub allocation_record_tree: AllocationRecordTreeVersion,
    /// 这一版 extent 树上段与第一个文件下段的每个节点与它的指针（D8（核心索引结构） 已定项 14：两段按位置寻址）；
    /// 节点的字节住 `units` 里 extent 树那一族角色。
    pub extent_tree: ExtentTreeVersion,
    pub accounting_entries: Vec<AccountingEntry>,
    /// 这一版记账树的形状与每个节点的指针（D8（核心索引结构） 已定项 11：多层码 2 树）；节点的字节住 `units` 里记账树那一族角色。
    pub accounting_tree: CodeTwoTreeVersion,
    /// 这一版中央映射树的形状与每个节点的指针，同上；根指针同时住根记录。
    pub central_mapping_tree: CodeTwoTreeVersion,
    pub tree_table_entries: Vec<TreeTableEntry>,
    /// 这一版那八棵树各自的号：第一个文件版本那次从水位发出来，之后每一版照抄（从盘上重建的版本按树表条目的种类与根记录里
    /// 中央映射树根指针的出生树读回来）。
    pub tree_identifiers: FileVersionTreeIdentifiers,
    /// 这一版第一个文件那条 inode 记录（覆盖写要接着它的对象出生代）。
    pub inode_record: InodeRecord,
    /// 这一版 inode 树里的全部叶容器，左起按 key 序：每片的身份、它装的记录、这一版它的指针
    /// （这次重写的是新落点，没重写的照抄上一版）。下一次发布按它算记录落在哪一片、要不要分裂。
    pub inode_leaf_containers: Vec<InodeLeafContainerVersion>,
    /// 进映射的单元各自的映射 key（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树；映射树与树表豁免）；
    /// 下一次覆盖写按它经映射取落点释放（D19（块指针的结构与宽度预算） 已定项 5 第 1 条）。
    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；第一个事务为空）：逻辑上释放了的都在，
    /// 其中有一份核出对不上、那块盘上的记录留在已分配的也在（那几份另列在下一个字段）。
    pub released: Vec<Placement>,
    /// 这次发布释放的落点里，释放之前读盘核校验和核出对不上（读不出也算）、隔离了的那几份（D19（块指针的结构与宽度预算） 已定项 5
    /// 硬规则 1「记进隔离并计数报出」的这一次那一份）：隔离 = 那一份在它那块盘上的分配记录留在已分配（用户 2026-09-24 定案），
    /// 落盘即跨重挂。从盘上重建的版本不是这个进程发布的，是空的。
    pub quarantined_after_release_checksum_mismatch:
        Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，第一个事务恒 0。
    pub key_order_mismatches: u64,
    /// 这次发布交给设备的写，按结构种类（增补 1）；从盘上重建的版本不是这个进程写出的，是空账。
    pub writes: WritesByStructureKind,
    /// 这个实例到这次发布为止用过的最大事务号（空发布写 0、不推进它）。下一次发布取它加一，
    /// 而不是取上一条记录上的事务号加一（D23（journal 的角色与格式） 已定项 7：事务号按实例计数、从 1 起）。
    pub highest_transaction_number_in_this_instance: u64,
}

/// 释放之前按位置项读盘核校验和、这个单元有一份核出对不上（读不出也算）而隔离的一份：哪个角色、哪块盘、那一份的落点
/// （槽取位置项里的，跨度取角色的）、那一份自己读出来的样子。隔离就是那块盘上这个落点的分配记录留在已分配；
/// 一个单元只要有一份对不上，每块盘上那一份都隔离（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定：各盘的账保持对称）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyQuarantinedAfterReleaseChecksumMismatch {
    pub unit: TransactionUnit,
    pub device: DeviceIdentity,
    pub placement: Placement,
    pub reading: QuarantinedCopyReading,
}

/// 隔离的那一份自己读出来是什么样子（硬规则 1 的读盘核，D19（块指针的结构与宽度预算） 已定项 5）。计数报出时分得开
/// 「真坏的份数」与「为了两块盘的账对称一起留下的好份」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarantinedCopyReading {
    /// 第一次没对上（读不出或对不上），重读那一次读出来了、整单元 CRC-32C 与位置项里的仍对不上。
    ChecksumMismatch,
    /// 第一次没对上（读不出或对不上），重读那一次读不出（只重读一次，用户 2026-09-24 定）。
    UnreadableAfterOneReread,
    /// 这一份对得上，同一个单元别的盘上那一份对不上或读不出：两块盘一起留在已分配（用户 2026-09-25 定）。
    IntactButAnotherCopyFailed,
}

/// 按 key 空间定形状的两棵派生树这一版的高（每个都从根节点头现读，[`TransactionOutput::position_addressed_tree_heights_read_from_the_root_node_headers`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionAddressedTreeHeights {
    pub allocation_record_tree: u64,
    pub extent_tree_upper_segment: u64,
    /// 第一个文件的下段；只有一个数据单元（内联）时 0。
    pub extent_tree_lower_segment_of_the_file: u64,
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

    /// 一棵多层码 2 树的高：从它这一版根节点的码 2 头里现读层级 + 1（D8（核心索引结构） 已定项 11 ⑤；
    /// D28（挂载期承诺量） 已定项 4：ckpt_cost 里每棵码 2 树的高从根节点头现读，不用内存里另存一份）。
    ///
    /// # Panics
    /// 这一版里这棵树的根单元解不开：它是这个进程装出来的，或从盘上重建时校验过的（`recovery::rebuild_version`）。
    #[must_use]
    pub fn height_read_from_the_root_node_header(&self, tree: MultiLevelCodeTwoTree) -> u64 {
        let root_role = match tree {
            MultiLevelCodeTwoTree::Accounting => TransactionUnit::AccountingTree,
            MultiLevelCodeTwoTree::CentralMapping => TransactionUnit::MappingTree,
        };
        let root_header = parse_index_node(&self.unit(root_role).bytes)
            .expect("根单元是这个进程装的，或重建时解过、核过的");
        u64::from(root_header.level) + 1
    }

    /// 按 key 空间定形状的两棵派生树的高，同样从根节点的码 2 头里现读层级 + 1（D8（核心索引结构） 已定项 14 末句：
    /// 树高照已定项 11「树高 = 根节点头层级 + 1」读；D28（挂载期承诺量） 已定项 4 的 ckpt_cost 读它）：
    /// 分配记录树读它的根；extent 树读上段的根，与第一个文件下段的根（内联、没有下段时是 0）。
    ///
    /// # Panics
    /// 这一版里那个根单元解不开：它是这个进程装的，或从盘上重建时校验过的（`recovery::rebuild_version`）。
    #[must_use]
    pub fn position_addressed_tree_heights_read_from_the_root_node_headers(
        &self,
    ) -> PositionAddressedTreeHeights {
        let height_of = |root_role: TransactionUnit| {
            let root_header = parse_index_node(&self.unit(root_role).bytes)
                .expect("根单元是这个进程装的，或重建时解过、核过的");
            u64::from(root_header.level) + 1
        };
        PositionAddressedTreeHeights {
            allocation_record_tree: height_of(TransactionUnit::AllocationTree),
            extent_tree_upper_segment: height_of(TransactionUnit::ExtentRoot),
            extent_tree_lower_segment_of_the_file: self
                .extent_tree
                .lower_nodes
                .last()
                .map_or(0, |(position, _)| {
                    height_of(TransactionUnit::ExtentLowerNode(*position))
                }),
        }
    }

    /// 这一版一棵多层码 2 树（形状与指针）。
    #[must_use]
    pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreeVersion {
        match tree {
            MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
            MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
        }
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

/// 第一个文件版本在树表 0 条的那一版上换下的落点：mkfs 那片树表单元，加那一版写行时写下的那棵分配记录树的每个节点
/// （C512（树表 0 条的一版上被换下的单元记在哪）：它的根住根记录那一项，重开时由 `mount` 整棵读进分配器）——那棵树的树号是 0，
/// 这一版的分配记录树换成第一个文件版本发出来的号，节点一个都不照抄（D18（块里携带什么信息） 已定项 7 的树 ID 进块头）。
/// 都只在「这一角色这次重写、那一片还登记着、还没释放过」时进来。
fn format_time_tree_table_to_release(
    allocator: &PoolAllocator,
    rewritten: &[TransactionUnit],
) -> Vec<Placement> {
    let still_allocated = |placement: &Placement| {
        allocator
            .devices
            .first()
            .and_then(|device_map| allocator.record_for(device_map.device, placement.slot))
            .is_some_and(|record| !record.is_released)
    };
    let mut released = Vec::new();
    if rewritten.contains(&TransactionUnit::TreeTable) {
        if let Some(tree_table) = allocator.format_time_tree_table().filter(&still_allocated) {
            released.push(tree_table);
        }
    }
    if rewritten.contains(&TransactionUnit::AllocationTree) {
        if let Some(tree) = allocator.allocation_record_tree_of_the_version_without_file() {
            released.extend(
                tree.version
                    .nodes
                    .iter()
                    .map(|(node, pointer)| Placement {
                        slot: slot_shared_by_both_location_entries(&pointer.locations).expect(
                            "树表 0 条那一版的分配记录树：本进程写的两盘同槽，`mount` 记它之前逐节点判过两条位置条目同槽",
                        ),
                        span: role_of_allocation_record_tree_node(*node).span_slots(),
                    })
                    .filter(&still_allocated),
            );
        }
    }
    released
}

/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：`roles` 是这次重写的角色，
/// 上一版里这些角色的落点从上一版的映射节点按 key 查出来，查不到就报「不在映射」、一个落点都不释放；
/// 这次重写文件内容时，上一版比这一版多出来的那几个数据单元一并释放（文件变短），这一版新多出来的数据单元与叶容器在上一版里没有落点、跳过。
/// 映射树与树表豁免映射，各从根记录里指着它们的那条指针取
/// （D19（块指针的结构与宽度预算） 已定项 11：映射树的根住根记录）。两盘同槽（D2（RAID 条带策略） 已定项 10）。
/// 实例表链不在这里释放：整条旧链由 [`instance_table_chain_to_release`] 按计划带着的那条链逐片释放（第 1 片起的指针只在
/// 上一片的链指针记录里，不在上一版的内存态里）。
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
    let mut placements = Vec::new();
    for identity in roles_replaced_via_mapping(previous, roles) {
        let locations = match identity {
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => {
                if !has_a_placement_in_the_previous_version(previous, identity) {
                    continue;
                }
                match mapping_locations_of_a_mapped_unit(previous, identity) {
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
            // 映射树根之下的节点同样豁免映射：父条目里的位置条目是权威（D19（块指针的结构与宽度预算） 已定项 8），
            // 上一版记着每个节点的那条指针。
            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
                previous.central_mapping_tree.pointer_of(position).locations
            }
            TransactionUnit::MappingTree => previous.root.mapping_root.locations,
            TransactionUnit::TreeTable => previous.root.tree_table.locations,
            // 整条旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                continue
            }
        };
        placements.push(placement_to_release_after_checking_every_device(
            identity, &locations, allocator,
        )?);
    }
    Ok(placements)
}

/// 整条实例表链重写时被换下的那条旧链，逐片的落点（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）：
/// `replaced_chain` 是那条旧链每一片的指针，按片序号（第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条）。
/// 实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），不经映射、不做释放之前的读盘核；每一片照样按
/// `placement_to_release_after_checking_every_device` 核三样（两条位置条目同槽、池里每块盘在册、没释放过、跨度对得上）。
/// 只查不改：带文件的一版上写行（`publish_version`）与树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）都调它；
/// 可写挂载取号之前的预演（`mount`）走的是这两处落盘之前那一段，判的是同一件事。
///
/// # Errors
/// 某一片的三样核不过 ⇒ `ReleaseTargetLocationsOnDifferentSlots` / `ReleaseTargetNotAllocated` /
/// `ReleaseTargetAlreadyReleased` / `ReleaseSpanMismatch`，带着是第几片（角色）。
pub fn instance_table_chain_to_release(
    replaced_chain: &[NodePointer],
    allocator: &PoolAllocator,
) -> Result<Vec<Placement>, PublishError> {
    replaced_chain
        .iter()
        .enumerate()
        .map(|(position, page_pointer)| {
            placement_to_release_after_checking_every_device(
                TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
                    u64::try_from(position).expect("片序号"),
                )),
                &page_pointer.locations,
                allocator,
            )
        })
        .collect()
}

/// 这次发布经映射换下的角色：这次重写的角色，加上文件内容重写时上一版比这一版多出来的那几个数据单元（文件变短）——
/// 它们这次没有对应的新角色，却同样被换下，不释放它们，它们就一直占着槽、再也没有树引用。释放判定路径
/// （`placements_to_release_via_mapping`）与释放之前读盘核校验和（`copies_failing_the_release_checksum_check`）读的是这同一张清单
/// （D19（块指针的结构与宽度预算） 已定项 5 第 1 条与硬规则 1：经映射释放的每一个都核）。文件内容这次没重写（写行、暖机）时不加。
fn roles_replaced_via_mapping(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
) -> Vec<TransactionUnit> {
    let previous_data_units = previous.data_pointers.len();
    let rewritten_data_units = roles
        .iter()
        .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
        .count();
    let previous_data_units_without_a_successor: Vec<TransactionUnit> = if rewritten_data_units == 0
    {
        Vec::new()
    } else {
        (rewritten_data_units..previous_data_units)
            .map(|position| {
                TransactionUnit::Data(DataUnitIndexInFile(
                    u64::try_from(position).expect("单元序号"),
                ))
            })
            .collect()
    };
    roles
        .iter()
        .copied()
        .chain(previous_data_units_without_a_successor)
        .collect()
}

/// 这个角色在上一版里有没有落点。这次发布新建的叶容器（末尾分裂出来的右半）在上一版里没有落点 ⇒ 没有东西要释放、要核；
/// 数据单元同一条：文件变长时新多出来的那几个单元在上一版里没有落点。
/// 判据取上一版的容器数、单元数，不取「映射里查不到」：查不到要报 `ReleaseNotInMapping`，那是另一件事。
fn has_a_placement_in_the_previous_version(
    previous: &TransactionOutput,
    identity: TransactionUnit,
) -> bool {
    match identity {
        TransactionUnit::InodeLeafContainer(index) => {
            index.position() < previous.inode_leaf_containers.len()
        }
        TransactionUnit::Data(index) => {
            index.0 < u64::try_from(previous.data_pointers.len()).expect("单元数")
        }
        // 按位置寻址的两棵树的节点：上一版有没有那个位置（这次新长出来的节点在上一版里没有落点）。
        TransactionUnit::ExtentLowerNode(position) => {
            previous.extent_tree.lower_pointer_of(position).is_some()
        }
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
            previous.extent_tree.upper_pointer_of(position).is_some()
                && position.level != previous.extent_tree.upper_root().level
        }
        TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => previous
            .allocation_record_tree
            .pointer_of(AllocationRecordTreeNode::BelowTheRoot(position))
            .is_some(),
        // 多层码 2 树的节点按上一版的位置点名（`previous_roles_replaced_by_this_publish` 从计划里的「被换下的节点」取），恒在上一版里。
        TransactionUnit::ExtentRoot
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable => true,
        // 第 1 片起的实例表页在不在上一版里，上一版的内存态答不出（指针只在上一片的链指针记录里）；两个调用方都只拿进映射的角色问，
        // 旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。这一臂只为穷举。
        TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
    }
}

/// 一个进映射的角色在上一版映射节点里的那条映射条目（按上一版记的 key 查）。
///
/// # Panics
/// `identity` 不是进映射的角色（映射树、树表、实例表豁免映射）：调用方按角色分过。
fn mapping_locations_of_a_mapped_unit(
    previous: &TransactionOutput,
    identity: TransactionUnit,
) -> MappingLookup {
    let (_, key) = previous
        .mapped_units
        .iter()
        .find(|(mapped, _)| *mapped == identity)
        .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");
    central_mapping_locations_in_the_version(previous, key)
}

/// 在一版的中央映射树里按 key 查落点：按这一版的形状从根沿分隔 key 走到那片叶（D8（核心索引结构） 已定项 11：
/// 最后一个分隔 key ≤ key 的那个孩子），叶的字节取 `units` 里它那个角色的单元、在叶里按 key 查（`mapping_locations_for_key`）。
/// 查的是叶的字节，不是内存里记的 key：叶被改过就照实查不到。
#[must_use]
pub fn central_mapping_locations_in_the_version(
    version: &TransactionOutput,
    key: &[u8],
) -> MappingLookup {
    let shape = &version.central_mapping_tree.shape;
    let Some(leaf) = leaf_position_routed_to(
        shape,
        &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
    ) else {
        return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
    };
    let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, shape);
    mapping_locations_for_key(&version.unit(leaf_role).bytes, key)
}

/// 经映射释放的角色（数据、extent 根、inode 叶容器与根、分配记录树、记账树）：这次重写的角色里走得到映射条目的那几个，按角色次序。
/// 映射树、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），硬规则 1 的读盘核挂在「经映射核到那条映射条目之后」，
/// 它们不在其内；这次新多出来、上一版没有落点的角色也不在其内。
fn mapped_roles_released_by_this_publish(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
) -> Vec<TransactionUnit> {
    roles_replaced_via_mapping(previous, roles)
        .into_iter()
        .filter(|identity| match identity {
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => true,
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
        })
        .filter(|identity| has_a_placement_in_the_previous_version(previous, *identity))
        .collect()
}

/// 一个经映射释放的角色在上一版映射里的那两条位置项。
///
/// # Panics
/// 在上一版的映射里查不到：调用方先走 `placements_to_release_via_mapping`、它按同一个上一版、同一串角色查过，
/// 查不到时它已经报了 `ReleaseNotInMapping` 或 `MappingEntryNarrowerThanItsFieldTable`，走不到这里。
fn mapping_locations_of_a_released_unit(
    previous: &TransactionOutput,
    identity: TransactionUnit,
) -> [LocationEntry; 2] {
    match mapping_locations_of_a_mapped_unit(previous, identity) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => panic!(
            "{identity:?} 在上一版的映射里查不到：placements_to_release_via_mapping 刚按同一个上一版查过，查不到时它已经报错返回"
        ),
    }
}

/// 经映射释放的每个单元，映射条目的两条位置项各指池里的一块盘、两条指的不是同一块（D19（块指针的结构与宽度预算） 已定项 5
/// 「硬规则 1 的读盘核读不出、核出对不上时怎么办」：位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏，
/// 在任何写之前拒绝，盘上不变；用户 2026-09-25 定）。只读内存里的上一版、不读盘：发布路径在读盘核之前判，
/// 可写挂载在取号之前的预演里按写行那一次换下的角色判（`mount` 的 `dry_run_of_the_publishes_after_acquisition`），
/// 两处判的是同一件事。
///
/// # Errors
/// 位置项指的盘不在 `pool_devices` 里 ⇒ `MappingEntryLocationOnADeviceOutsideThePool`；
/// 两条位置项指同一块盘 ⇒ `MappingEntryLocationsOnTheSameDevice`。按角色次序报第一个。
///
/// # Panics
/// 同 [`mapping_locations_of_a_released_unit`]。
pub fn refuse_mapping_entries_that_do_not_name_two_pool_devices(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
    pool_devices: &[DeviceIdentity],
) -> Result<(), PublishError> {
    for identity in mapped_roles_released_by_this_publish(previous, roles) {
        let locations = mapping_locations_of_a_released_unit(previous, identity);
        if let Some(outside) = locations
            .iter()
            .find(|location| !pool_devices.contains(&location.device))
        {
            return Err(PublishError::MappingEntryLocationOnADeviceOutsideThePool {
                unit: identity,
                device: outside.device,
                slot: outside.slot,
            });
        }
        if locations[0].device == locations[1].device {
            return Err(PublishError::MappingEntryLocationsOnTheSameDevice {
                unit: identity,
                device: locations[0].device,
                slot: locations[0].slot,
            });
        }
    }
    Ok(())
}

/// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，用户 2026-09-23 定案）：
/// 这次重写的角色里经映射释放的那几个（[`mapped_roles_released_by_this_publish`]），映射条目的每条位置项指的那一份
/// 整单元读出来算 CRC-32C、与位置项里的比（写的时候位置项的校验和就是整单元 CRC-32C，`make_filesystem::location_entries`）。
/// **读盘本身失败、或读出来核出对不上，都先重读一次，两次都没对上才按对不上处置**（读不出那一半用户 2026-09-24 定案，C394 三问的第一问；
/// 对不上那一半主 agent 2026-09-25 定，D19（块指针的结构与宽度预算） 已定项 5）：只重读一次，不退避、不多次重试。
/// **任一份核出对不上（读不出也算），这个单元在每块盘上的分配记录都留在「已分配」**、不改成已释放——另一块盘上那一份对得上也一起留，
/// 各盘的账保持对称（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定）：交回的是这些单元在池里每块盘上的那一份，
/// 按角色次序、位置项的次序（设备身份升序），每一份带着它自己读出来的样子（[`QuarantinedCopyReading`]）；发布照常释放（映射条目去掉），
/// 只是这些份的记录留在已分配（`PoolAllocator::release_leaving_the_record_allocated_on`），发布照成。
///
/// 只读，在动分配器、发任何一个写之前调；调用方先走 `placements_to_release_via_mapping`、它核过了才走到这里。
///
/// # Errors
/// 映射条目的位置项指池外的盘、或两条指同一块盘（[`refuse_mapping_entries_that_do_not_name_two_pool_devices`]）：
/// 在读任何一份之前返回。
///
/// # Panics
/// 同 [`mapping_locations_of_a_released_unit`]。
pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
    reader: &Reader,
) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
    let pool_devices = reader.device_identities();
    refuse_mapping_entries_that_do_not_name_two_pool_devices(previous, roles, &pool_devices)?;
    let mut quarantined = Vec::new();
    for identity in mapped_roles_released_by_this_publish(previous, roles) {
        let locations = mapping_locations_of_a_released_unit(previous, identity);
        let unit_bytes = usize::try_from(identity.span_slots() * SLOT_BYTES)
            .expect("一个单元两槽以内，字节数装得进 usize");
        let reading_of_each_copy: Vec<(LocationEntry, QuarantinedCopyReading)> = locations
            .iter()
            .map(|location| {
                let read_the_copy = || {
                    reader.read(
                        location.device,
                        location.slot.to_device_offset(),
                        unit_bytes,
                    )
                };
                let check_the_copy_against_its_location_entry = |copy: Option<Vec<u8>>| match copy {
                    None => CopyCheck::Unreadable,
                    Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {
                        CopyCheck::ChecksumMismatch
                    }
                    Some(_) => CopyCheck::Intact,
                };
                // 第一次读不出、或读出来核出对不上，都先重读一次（只一次，D19（块指针的结构与宽度预算） 已定项 5
                // 「硬规则 1 的读盘核读不出、核出对不上时怎么办」）：一次瞬时坏读不许让单元永久隔离。
                // 重读那一次对得上就算这一份对得上；两次都没对上，按重读那一次的样子报。
                let reading = match check_the_copy_against_its_location_entry(read_the_copy()) {
                    CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                    CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {
                        match check_the_copy_against_its_location_entry(read_the_copy()) {
                            CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                            CopyCheck::Unreadable => {
                                QuarantinedCopyReading::UnreadableAfterOneReread
                            }
                            CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,
                        }
                    }
                };
                (*location, reading)
            })
            .collect();
        let some_copy_failed = reading_of_each_copy
            .iter()
            .any(|(_, reading)| match reading {
                QuarantinedCopyReading::ChecksumMismatch
                | QuarantinedCopyReading::UnreadableAfterOneReread => true,
                QuarantinedCopyReading::IntactButAnotherCopyFailed => false,
            });
        if !some_copy_failed {
            continue;
        }
        // 两条位置项各指池里一块不同的盘（上面判过；第一版池里恰好两块盘，mkfs 断言过），两份都留：按位置项的次序（设备身份升序）各交一份。
        for (location, reading) in reading_of_each_copy {
            quarantined.push(CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: identity,
                device: location.device,
                placement: Placement {
                    slot: location.slot,
                    span: identity.span_slots(),
                },
                reading,
            });
        }
    }
    Ok(quarantined)
}

/// 释放之前读盘核一份的一次结果（[`copies_failing_the_release_checksum_check`]）：读不出、读出来整单元 CRC-32C 与位置项里的对不上、对得上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CopyCheck {
    Unreadable,
    ChecksumMismatch,
    Intact,
}

/// 一个被换下的单元的落点：两条位置条目同槽，池里每块盘各有一条在册、未释放、跨度对得上的记录；有一样不对就报错、不交回落点。
///
/// **池里每块盘都核一遍，不只核 `locations[0].device` 那一块**：`PoolAllocator::release` 对每块盘都要求
/// 一条在册、未释放、跨度对得上的记录，而两块盘的分配记录树对不对称是盘上读来的、不是不变量
/// （panic 面普查 R10：盘 0 有记录、盘 1 没有 / 已释放 / 跨度不同时，此前这三样在 `release` 的
/// `expect` 与两条断言上 panic）。这道判与那三样逐条对齐：少判一样，那一样就还在断言上。
/// 跨度取核过的那个值，每块盘都要与种类该有的跨度相同，所以逐盘核完取哪一块都一样。
///
/// 两处调它，两处判的是同一件事、不各写一份：带文件的一版按映射（或根记录）查出位置条目之后
/// （`placements_to_release_via_mapping`）；树表 0 条的一版上写行时按根记录里那条实例表指针
/// （`publish_instance_table_on_version_without_file`——那一版没有上一版的内存态，也没有映射树）。
///
/// # Errors
/// `ReleaseTargetLocationsOnDifferentSlots`、`ReleaseTargetNotAllocated`、`ReleaseTargetAlreadyReleased`、`ReleaseSpanMismatch`。
fn placement_to_release_after_checking_every_device(
    identity: TransactionUnit,
    locations: &[LocationEntry; 2],
    allocator: &PoolAllocator,
) -> Result<Placement, PublishError> {
    let slot = slot_shared_by_both_location_entries(locations).map_err(|disagreement| {
        PublishError::ReleaseTargetLocationsOnDifferentSlots {
            unit: identity,
            disagreement,
        }
    })?;
    for device in allocator.devices.iter().map(|device_map| device_map.device) {
        let record =
            allocator
                .record_for(device, slot)
                .ok_or(PublishError::ReleaseTargetNotAllocated {
                    unit: identity,
                    device,
                    slot,
                })?;
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
    Ok(Placement {
        slot,
        span: identity.span_slots(),
    })
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

/// 发布能出的错：分配器给不出落点（哪一个单元、为什么）、多层码 2 树的形状算不出来、内容装不下、释放判定路径对不上，或底层块设备错。
/// 按 key 空间定形状的分配记录树与 extent 树不分裂、没有装不下这一格（D8（核心索引结构） 已定项 14），此前那两个成员
/// （分配记录装不进一个节点、extent 树要长内部节点）随之去掉。
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
    /// 空间准入不够（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md`
    /// 第四节第 2 条把它接进发布路径）：这次发布的普通分配（用户数据单元、extent 树与 inode 树的节点，`admission::space_budget_of_role`）
    /// 在至少一块盘上多于可用(d)，带着不够的每一块。在算定这次发布的样子之后、读盘核被换下的单元与动分配器之前返回，
    /// 一个写都没发、盘上逐字节不变。准入不够时先推空发布抬 F 再判（D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））
    /// 没有实现：调用方收到的就是这一次的拒绝。
    SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices),
    /// 记账树或中央映射树这次之后的形状算不出来（`code_two_tree::plan_the_tree_after_this_publish`）：要长到 257 层、
    /// 码 2 头的层级 1 字节写不下（多层之后映射树容量准入剩下的唯一一条，D19（块指针的结构与宽度预算） 已定项 5），
    /// 或上一版（从盘上重建的）树按分隔 key 走不到它自己叶里的一把 key。在动分配器、发任何一个写之前返回，盘上逐字节不变。
    /// 两棵树装不下一个节点时不再报错、照 D8（核心索引结构） 已定项 11 分裂（里程碑「第二个事务」增补 2 收口表第 28 行）。
    MultiLevelCodeTwoTreeRefused {
        tree: MultiLevelCodeTwoTree,
        refusal: CodeTwoTreeRefusal,
    },
    /// 释放判定路径在上一版的映射里查不到这个单元（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：不按提示释放）。
    ReleaseNotInMapping { unit: TransactionUnit },
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
    /// 经映射释放的这个单元，映射条目的一条位置项指的盘不在这个池里（D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、
    /// 核出对不上时怎么办」：当映射条目损坏，用户 2026-09-25 定）。映射条目是盘上读来的字节，坏镜像、外来镜像才有。
    /// 在动分配器、读盘核任何一份、发任何一个写之前返回，盘上逐字节不变。
    MappingEntryLocationOnADeviceOutsideThePool {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 经映射释放的这个单元，映射条目的两条位置项指同一块盘（同上一条：当映射条目损坏）——另一块盘上那一份从来没被核过就会被释放。
    /// 在动分配器、读盘核任何一份、发任何一个写之前返回，盘上逐字节不变。`slot` 是第一条位置项的槽。
    MappingEntryLocationsOnTheSameDevice {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）：`publish_first_file` 与 `publish_overwrite` 按一个数据单元写
    /// （它们的契约），多单元的内容走 `publish_sequential_write`。
    ContentExceedsDataUnit { bytes: usize, capacity: usize },
    /// 分配记录树这次重写哪几个节点的固定点在 `rounds` 轮里没有定下来（`settle_the_allocation_record_tree`）。产品路径上走不到这里
    /// （释放不在这次发布里空出槽、多一个角色只把取落点那一串拉长，重写集与节点都单调）；只供测试的复用窗口置 0 开关下单调性推不出来，
    /// 条款没写那一格怎么办 ⇒ 第一版不支持：在任何落盘动作之前返回，盘上逐字节不变。
    AllocationRecordTreeRewriteSetDidNotSettle { rounds: usize },
    /// 这次发布要往 inode 树里写的记录，落法要的条款仓里还没定（[`InodeTreeWriteRefusal`] 的三格）：
    /// 在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
    InodeTreeWriteRefused(InodeTreeWriteRefusal),
    /// `publish_first_file` 要建在上面的那一版的根与交给它的上一条记录说的不是同一版：记录解不出本池的记录（`None`），
    /// 或记录的 checkpoint_txg 与那条根的不同。新根的 txg 从那条根接着算、jsn 从那条记录接着算，两者对不上时接上去会盖在别的发布上。
    FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
        version_to_build_on: CheckpointTxg,
        previous_record: Option<(CheckpointTxg, u64)>,
    },
    /// `publish_first_file` 要建在上面的那一版已经有过文件版本（它的树表不是 0 条，`tree_table_entries` 是条数）：
    /// 这条路径按「树还没建起来」写——`previous` 传 `None`、八棵树从水位重新发号、inode 树与 extent 树从头建、
    /// 树表条目的诞生 txg 与 inode 1 的对象出生代都取这次发布的 txg。接在已经有文件的一版后面写出来的那条根，
    /// 会与同一条时间线上的旧根在 I-9.14（树表条目的诞生 txg 跨根不变） 与 I-9.10（对象出生代与 inode 记录相符） 上对不上
    /// （2026-09-23 崩溃注入快档打中）。判的是树表条数，不是水位（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：
    /// 回退到树表 0 条的一版时水位带着根环里的 max，已经不是 mkfs 的 11）。
    /// 同一个文件再写一版走 `publish_overwrite`。一个字节都不写。
    FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: usize },
    /// `publish_first_file` 要判「那一版有没有过文件版本」得读它的树表单元，而那一片两份都读不出或解不开
    /// （`failure` 原样带着恢复路径那一格的原因）：判不了就不写，一个字节都不写。
    TreeTableOfTheVersionToBuildOnUnreadable { failure: RecoveryFailure },
    /// `publish_first_file` 从那一版的树 ID 水位起连号发八棵树的号，而水位（盘上读来的 8 字节）加八个号越过 `u64::MAX`：
    /// 发不出号就不写，一个字节都不写。
    TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(
        TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees,
    ),
    /// 分配器上冻结着一次落盘中途失败、还没原样重发的发布（`FrozenPublish`，D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：
    /// 重发成功才建下一次发布）：先 `resend_the_frozen_publish`，成功之后在它交回的那一版上接着发。在任何读写之前返回，盘上逐字节不变。
    /// 带的是冻结的那一次的 checkpoint_txg 与实例代号。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 块设备报的错。落盘阶段报的（带文件的一版、树表 0 条的一版上写行）：这次发布已冻结在分配器上等原样重发。
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
/// 要不要分裂由 `crate::inode_tree` 算）。没有它的发布（写行、暖机）这两类角色照抄上一版。
///
/// `content` 是整份文件内容（顺序写从偏移 0 写到末尾）：按切分纪律切成几个一单元事务就写几个数据单元
/// （`crate::write_request_split`，D16（发布语义） 已定项 5 末段），事务号从 [`PublishPlan::transaction`] 起连号。
#[derive(Clone, Copy, Debug)]
pub struct FileVersionPlan<'content> {
    pub content: &'content [u8],
    pub write_time_seconds: u64,
    /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
    pub inode_object_birth: CheckpointTxg,
    /// 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）。
    pub change_count: u64,
}

/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者整条链重写（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。
#[derive(Clone, Debug)]
pub enum InstanceTablePlan {
    Carry(NodePointer),
    Rewrite(InstanceTableRewrite),
}

/// 整条实例表链重写：这次之后整张表的行，与被这次换下的那条旧链。
#[derive(Clone, Debug)]
pub struct InstanceTableRewrite {
    /// 这次之后整张表的行，按链上的次序（上一版那张表的行在前、这次写的接在后面）。怎么分到各片由
    /// `instance_table_rows_of_each_page` 定（一片写满 369 行再开下一片）。
    pub rows: Vec<InstanceRow>,
    /// 被这次重写换下的那条旧链每一片的指针，按片序号：第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条
    /// （`recovery::instance_table_chain_of_root` 读出来的 `page_pointers`）。发布逐片释放它们。
    /// 第 1 片起的指针不在上一版的内存态（`TransactionOutput`）里，所以由拼这张计划的调用方带进来：它拼行读的就是这同一条链，
    /// 接行与释放读的是同一份输入。
    pub replaced_chain: Vec<NodePointer>,
}

impl InstanceTableRewrite {
    /// 这次之后这条链有几片（D28（挂载期承诺量） 已定项 3 的 max(1, ⌈行数 ÷ 369⌉)）。
    #[must_use]
    pub fn pages_after_this_publish(&self) -> usize {
        instance_table_pages_for_rows(self.rows.len())
    }
}

/// 重写一条 `pages` 片的实例表链时各片的角色，按 bump 次序：尾片先（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：
/// 第 k 片当第 k + 1 片的父，照先叶后根），第 0 片最后。这个次序同时是各片取落点与发出生序号的次序（D19（块指针的结构与宽度预算） 已定项 9）。
#[must_use]
pub fn instance_table_page_roles_in_bump_order(pages: usize) -> Vec<TransactionUnit> {
    (0..pages)
        .rev()
        .map(|position| {
            TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
                u64::try_from(position).expect("片序号"),
            ))
        })
        .collect()
}

/// 一次发布的全部参数：哪些角色重写、身份字段取什么。第一个事务、覆盖写、写行、暖机都是它的一种取值，走同一条发布路径
/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
#[derive(Clone, Debug)]
pub struct PublishPlan<'content> {
    pub txg: CheckpointTxg,
    /// 这次发布第一条记录的 jsn 计数器（记录落在环里的槽位），全池接着走、换实例不归零（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    /// 一次发布切成 N 条记录时（文件版本有 N 个数据单元）它们连号：`counter .. counter + N`。
    pub counter: u64,
    /// 这次发布第一个事务的事务号，按实例计数从 1 起，空发布写 0（D23（journal 的角色与格式） 已定项 7 / 已定项 19 ①）。
    /// 文件版本有 N 个数据单元时这次发布是 N 个事务（切分纪律一事务一单元），事务号 `transaction .. transaction + N` 连号，
    /// 一条记录一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）。
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
    /// 这次发布写进根记录与记录新根段的树 ID 水位（D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)）。
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
}

/// 一次发布重写哪些角色，只由三件事定：这次写不写文件内容（数据单元 + extent 树根）、重写几片 inode 叶容器、
/// 实例表是重写还是照抄。计划的身份字段（新实例代号、表的字节、txg）都不进来——可写挂载要在取号之前算写行那次发布的准入，
/// 而那时新实例代号还没取（增补 2 第 20a 行）。
///
/// 映射条目那一条准入要的是**这次之后 inode 树一共几片叶容器**（映射节点每次发布整片重写，照抄的那几片也各占一条），
/// 形状答不了它——「重写几片」与「一共几片」是两个数。那个数各条路径自己给：发布路径从 `PublishPlan::resolve`
/// 算出来的树取；取号之前那一串走的是同一条发布路径（`mount` 的 `dry_run_of_the_publishes_after_acquisition`），也从那里取。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishShape {
    /// 这次重写几个数据单元；0 = 这次不写文件内容（数据单元与 extent 树根照抄上一版）。
    pub rewritten_data_units: usize,
    /// 这次重写几片 inode 叶容器；0 = 这次一个字节都不碰 inode 树（连根都不重写）。
    pub rewritten_inode_leaf_containers: usize,
    /// 这次重写的实例表链有几片；0 = 实例表照抄。
    pub rewritten_instance_table_pages: usize,
}

impl PublishShape {
    /// 写行那次发布、实例表链这次之后只有一片时的形状（行数不超过 369，第一个事务之后的多数挂载都是它）：
    /// 多于一片的见 [`PublishShape::row_publish_rewriting_instance_table_pages`]。
    pub const ROW_PUBLISH: PublishShape =
        PublishShape::row_publish_rewriting_instance_table_pages(1);

    /// 写行那次发布的形状：不写文件内容、不碰 inode 树、实例表整条链重写成 `instance_table_pages` 片
    /// （`publish_rows_on_file_version` 交给 `publish_version` 的那张计划同形）。
    #[must_use]
    pub const fn row_publish_rewriting_instance_table_pages(instance_table_pages: usize) -> Self {
        PublishShape {
            rewritten_data_units: 0,
            rewritten_inode_leaf_containers: 0,
            rewritten_instance_table_pages: instance_table_pages,
        }
    }

    /// 暖机那几次空发布的形状：不写文件内容、不碰 inode 树、实例表照抄（`publish_empty_after` 交给 `publish_version` 的那张计划同形，
    /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写四个固定点单元（D16（发布语义） 已定项 9）。
    pub const EMPTY_PUBLISH: PublishShape = PublishShape {
        rewritten_data_units: 0,
        rewritten_inode_leaf_containers: 0,
        rewritten_instance_table_pages: 0,
    };

    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点），
    /// 多于一片时尾片先（[`instance_table_page_roles_in_bump_order`]）。
    ///
    /// ⚠️ **叶容器在这里按 0 起的连号排，不是它们在树里的真实叶序**：形状只答「几个角色」，
    /// 答不了「改的是哪几片」——那要看上一版的树长什么样。
    /// 真实的角色清单由 [`PublishPlan::rewritten_roles`] 给。
    #[must_use]
    pub fn rewritten_roles(self) -> Vec<TransactionUnit> {
        let mut roles = Vec::new();
        for position in 0..self.rewritten_data_units {
            roles.push(TransactionUnit::Data(DataUnitIndexInFile(
                u64::try_from(position).expect("单元序号"),
            )));
        }
        roles.extend(instance_table_page_roles_in_bump_order(
            self.rewritten_instance_table_pages,
        ));
        if self.rewritten_data_units > 0 {
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
    /// 这次之后 extent 树长什么样（D8（核心索引结构） 已定项 14 的两段）。
    pub extent_tree: ExtentTreePlan,
    /// 这次之后分配记录树长什么样（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。重写哪几个节点由发布路径在分配器的拷贝上
    /// 走到固定点定下来（`settle_the_allocation_record_tree`），交给 `PublishPlan::resolve` 的就是那一份。
    pub allocation_record_tree: AllocationRecordTreePlan,
    /// 这次之后记账树长什么样（D8（核心索引结构） 已定项 11 的分裂与收缩）。
    pub accounting_tree: CodeTwoTreePlan,
    /// 这次之后中央映射树长什么样，同上；按下面那一份映射 key 算。
    pub central_mapping_tree: CodeTwoTreePlan,
    /// 这一版每个进映射的单元与它的映射 key，按 bump 次序（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、
    /// 记账树的每个节点）：这次重写的按这次要发的出生身份现算（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9），
    /// 照抄的取上一版的。装映射节点那一段（`publish_admitted`）核装出来的与它逐项相等。
    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
    pub rewritten_roles: Vec<TransactionUnit>,
}

impl ResolvedPublish {
    /// 这次发布的形状（准入按它数角色）。
    #[must_use]
    pub fn shape(&self) -> PublishShape {
        PublishShape {
            rewritten_data_units: self
                .rewritten_roles
                .iter()
                .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
                .count(),
            rewritten_inode_leaf_containers: self.inode_tree.rewritten.len(),
            rewritten_instance_table_pages: self
                .rewritten_roles
                .iter()
                .filter(|identity| identity.is_a_page_of_the_instance_table())
                .count(),
        }
    }

    /// 一棵多层码 2 树这次之后的计划。
    #[must_use]
    pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreePlan {
        match tree {
            MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
            MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
        }
    }

    /// 这次发布换下的上一版角色：这次重写的角色里不属于那四棵多节点树（extent 树、分配记录树、记账树、中央映射树）的那些
    /// （同一个角色上一版那一份被换下），加 extent 树与分配记录树按计划换下的节点、两棵多层码 2 树里上一版被换下的节点
    /// （都按上一版的位置点名；这一版照抄进来的节点不被换下）。
    /// 释放判定路径与释放之前读盘核校验和读的都是这一张（`placements_to_release_via_mapping`、`copies_failing_the_release_checksum_check`）。
    #[must_use]
    pub fn previous_roles_replaced_by_this_publish(
        &self,
        previous: &TransactionOutput,
    ) -> Vec<TransactionUnit> {
        // 按 bump 次序：数据单元与实例表、extent 树换下的节点、inode 树、分配记录树换下的节点、记账树换下的节点、
        // 中央映射树换下的节点、树表最末——与这次重写的角色同一个次序，释放清单与读盘核校验和的次序都照它。
        let is_a_node_of_a_tree_with_its_own_replacement_list = |identity: TransactionUnit| {
            multi_level_tree_of_role(identity).is_some()
                || allocation_record_tree_node_of_role(identity).is_some()
                || matches!(
                    identity,
                    TransactionUnit::ExtentLowerNode(_)
                        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
                        | TransactionUnit::ExtentRoot
                )
        };
        let rewritten_before_the_extent_tree =
            self.rewritten_roles.iter().copied().filter(|identity| {
                matches!(
                    identity,
                    TransactionUnit::Data(_)
                        | TransactionUnit::InstanceTable
                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
                )
            });
        let mut replaced: Vec<TransactionUnit> = rewritten_before_the_extent_tree.collect();
        replaced.extend(self.extent_tree.replaced_previous_roles.iter().copied());
        replaced.extend(self.rewritten_roles.iter().copied().filter(|identity| {
            !is_a_node_of_a_tree_with_its_own_replacement_list(*identity)
                && !matches!(
                    identity,
                    TransactionUnit::Data(_)
                        | TransactionUnit::InstanceTable
                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
                        | TransactionUnit::TreeTable
                )
        }));
        replaced.extend(
            self.allocation_record_tree
                .replaced_previous_nodes
                .iter()
                .map(|node| role_of_allocation_record_tree_node(*node)),
        );
        for tree in [
            MultiLevelCodeTwoTree::Accounting,
            MultiLevelCodeTwoTree::CentralMapping,
        ] {
            let previous_shape = &previous.multi_level_tree(tree).shape;
            replaced.extend(
                self.multi_level_tree(tree)
                    .replaced_previous_nodes
                    .iter()
                    .map(|position| tree.role_of_node(*position, previous_shape)),
            );
        }
        if self.rewritten_roles.contains(&TransactionUnit::TreeTable) {
            replaced.push(TransactionUnit::TreeTable);
        }
        replaced
    }
}

/// 一个角色属于哪棵多层码 2 树（记账树、中央映射树的根与根之下的节点）；别的角色交回 `None`。
#[must_use]
pub const fn multi_level_tree_of_role(identity: TransactionUnit) -> Option<MultiLevelCodeTwoTree> {
    match identity {
        TransactionUnit::AccountingTreeNodeBelowTheRoot(_) | TransactionUnit::AccountingTree => {
            Some(MultiLevelCodeTwoTree::Accounting)
        }
        TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
            Some(MultiLevelCodeTwoTree::CentralMapping)
        }
        TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
    }
}

/// 分配记录树的一个节点在一版里的角色：根是 `AllocationTree`，根之下按位置（位置不随版本挪动）。
#[must_use]
pub const fn role_of_allocation_record_tree_node(
    node: AllocationRecordTreeNode,
) -> TransactionUnit {
    match node {
        AllocationRecordTreeNode::Root => TransactionUnit::AllocationTree,
        AllocationRecordTreeNode::BelowTheRoot(position) => {
            TransactionUnit::AllocationTreeNodeBelowTheRoot(position)
        }
    }
}

/// 一个角色是不是分配记录树的节点、是哪一个；别的角色交回 `None`。
#[must_use]
pub const fn allocation_record_tree_node_of_role(
    identity: TransactionUnit,
) -> Option<AllocationRecordTreeNode> {
    match identity {
        TransactionUnit::AllocationTree => Some(AllocationRecordTreeNode::Root),
        TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => {
            Some(AllocationRecordTreeNode::BelowTheRoot(position))
        }
        TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
    }
}

/// extent 树上段一个位置在一版里的角色：这一版上段的根（最高那一层 `upper_root_level` 的第 0 个）是 `ExtentRoot`，别的按位置。
#[must_use]
pub fn role_of_extent_upper_node(
    position: ExtentUpperNodePosition,
    upper_root_level: u8,
) -> TransactionUnit {
    if position.level == upper_root_level {
        TransactionUnit::ExtentRoot
    } else {
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position)
    }
}

/// 一版 extent 树里全部节点的角色，按 bump 次序（下段先叶后根，再上段先叶后根、上段根最末，D3（空间分配） 已定项 10 ⑤「树内先叶后根」：
/// 上段叶条目要下段根这一版的指针，下段排在前面）。
#[must_use]
pub fn extent_tree_roles_in_bump_order(version: &ExtentTreeVersion) -> Vec<TransactionUnit> {
    let lower = version
        .lower_nodes
        .iter()
        .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position));
    let upper_root_level = version.upper_root().level;
    let upper = version
        .upper_nodes
        .iter()
        .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level));
    lower.chain(upper).collect()
}

/// 这次发布之后 extent 树一个节点从哪来：照抄上一版同一个位置上的节点，或这次重写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtentTreeNodeOrigin {
    CarriedFromThePreviousVersion,
    RewrittenThisPublish,
}

/// 这次发布之后 extent 树长什么样（第一版只有第一个文件有内容，D8（核心索引结构） 已定项 14 的两段）：
/// 写文件内容的发布重写第一个文件的整个下段（每个数据单元都是新的，下段每片叶都变）与上段里从它那片叶到根那一串，
/// 上段别的节点照抄；不写文件内容的发布整棵照抄。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtentTreePlan {
    /// 第一个文件下段这一版的节点，bump 次序（先叶后根）；只有一个数据单元（内联）时为空。
    pub lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
    /// 上段这一版的节点，bump 次序（先叶后根，根在最末）。
    pub upper_nodes: Vec<(ExtentUpperNodePosition, ExtentTreeNodeOrigin)>,
    /// 上一版被这次换下的 extent 树节点的角色（按上一版的位置与根层级取名），按上一版的 bump 次序。
    pub replaced_previous_roles: Vec<TransactionUnit>,
}

impl ExtentTreePlan {
    /// 这一版上段根的层级。
    ///
    /// # Panics
    /// 上段一个节点都没有：规划恒交出第一个文件那片叶到根那一串。
    #[must_use]
    pub fn upper_root_level(&self) -> u8 {
        self.upper_nodes
            .last()
            .expect("上段恒有第一个文件那片叶到根那一串")
            .0
            .level
    }

    /// 这次重写的 extent 树角色，按 bump 次序（= extent 树里出生序号的发号次序）。
    #[must_use]
    pub fn rewritten_roles(&self) -> Vec<TransactionUnit> {
        let upper_root_level = self.upper_root_level();
        self.lower_nodes
            .iter()
            .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
            .chain(
                self.upper_nodes
                    .iter()
                    .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
                    .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
            )
            .collect()
    }

    /// 这一版全部 extent 树角色，按 bump 次序。
    #[must_use]
    pub fn roles(&self) -> Vec<TransactionUnit> {
        let upper_root_level = self.upper_root_level();
        self.lower_nodes
            .iter()
            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
            .chain(
                self.upper_nodes
                    .iter()
                    .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
            )
            .collect()
    }
}

/// 这次发布之后的 extent 树。写文件内容的发布：第一个文件有 `data_units` 个单元（从 0 起连号、没有洞）——一个就内联、不建下段，
/// 两个起建下段（整段重写）；上段从第一个文件那片叶到根那一串重写，上段别的节点照抄。不写文件内容的发布（`data_units` 为 `None`）整棵照抄。
///
/// # Panics
/// 写文件内容的发布没有上一版、却也不是第一个文件版本以外的东西：没有上一版时上段只有这一串，不会走到照抄那一支。
#[must_use]
pub fn plan_the_extent_tree_after_this_publish(
    previous: Option<&ExtentTreeVersion>,
    data_units_of_the_file_written_this_publish: Option<u64>,
) -> ExtentTreePlan {
    let Some(data_units) = data_units_of_the_file_written_this_publish else {
        let carried = previous.expect("不写文件内容的发布接在带文件的一版之后：extent 树整棵照抄");
        return ExtentTreePlan {
            lower_nodes: carried
                .lower_nodes
                .iter()
                .map(|(position, _)| {
                    (
                        *position,
                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                    )
                })
                .collect(),
            upper_nodes: carried
                .upper_nodes
                .iter()
                .map(|(position, _)| {
                    (
                        *position,
                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                    )
                })
                .collect(),
            replaced_previous_roles: Vec::new(),
        };
    };
    let previous_upper_root_level = previous.map(|version| version.upper_root().level);
    let upper_root_level =
        upper_root_level_for(FIRST_INODE_NUMBER).max(previous_upper_root_level.unwrap_or(0));
    let path_of_the_file: BTreeSet<ExtentUpperNodePosition> =
        upper_path_of_inode(FIRST_INODE_NUMBER, upper_root_level)
            .into_iter()
            .collect();
    let previous_upper: BTreeSet<ExtentUpperNodePosition> = previous
        .map(|version| {
            version
                .upper_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect()
        })
        .unwrap_or_default();
    let upper_nodes: Vec<(ExtentUpperNodePosition, ExtentTreeNodeOrigin)> = path_of_the_file
        .union(&previous_upper)
        .map(|position| {
            let origin = if path_of_the_file.contains(position) {
                ExtentTreeNodeOrigin::RewrittenThisPublish
            } else {
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
            };
            (*position, origin)
        })
        .collect();
    let lower_nodes = lower_segment_nodes_of_a_file_without_holes(data_units)
        .into_iter()
        .map(|position| (position, ExtentTreeNodeOrigin::RewrittenThisPublish))
        .collect();
    // 换下的：上一版第一个文件的整个下段（这次每个数据单元都换了），与上一版上段里这次重写的那几个（按上一版的根层级取名）。
    let replaced_previous_roles = match (previous, previous_upper_root_level) {
        (Some(version), Some(previous_root_level)) => version
            .lower_nodes
            .iter()
            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
            .chain(
                version
                    .upper_nodes
                    .iter()
                    .filter(|(position, _)| path_of_the_file.contains(position))
                    .map(|(position, _)| role_of_extent_upper_node(*position, previous_root_level)),
            )
            .collect(),
        (None, _) | (Some(_), None) => Vec::new(),
    };
    ExtentTreePlan {
        lower_nodes,
        upper_nodes,
        replaced_previous_roles,
    }
}

/// 记账树里不带设备维的三行（D5（快照 / 空间记账机制） 已定项 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PoolWideAccountingRow {
    InodeNumberWatermark,
    PendingDeleteBytes,
    CommittedReservationBytes,
}

impl PoolWideAccountingRow {
    const ALL: [Self; 3] = [
        PoolWideAccountingRow::InodeNumberWatermark,
        PoolWideAccountingRow::PendingDeleteBytes,
        PoolWideAccountingRow::CommittedReservationBytes,
    ];

    const fn statistic(self) -> u16 {
        match self {
            PoolWideAccountingRow::InodeNumberWatermark => STATISTIC_INODE_WATERMARK,
            PoolWideAccountingRow::PendingDeleteBytes => STATISTIC_PENDING_DELETE_BYTES,
            PoolWideAccountingRow::CommittedReservationBytes => {
                STATISTIC_COMMITTED_RESERVATION_BYTES
            }
        }
    }

    /// 这一行记在哪棵树上：inode 号水位记在 inode 树上，另两行不属于任何一棵树。
    const fn tree(self, inode_tree: TreeIdentifier) -> TreeIdentifier {
        match self {
            PoolWideAccountingRow::InodeNumberWatermark => inode_tree,
            PoolWideAccountingRow::PendingDeleteBytes
            | PoolWideAccountingRow::CommittedReservationBytes => {
                TreeIdentifier(TREE_IDENTIFIER_NONE)
            }
        }
    }
}

/// 记账树里每块盘各一行的六项（D5（快照 / 空间记账机制） 已定项 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PerDeviceAccountingRow {
    AllocatedBytes,
    FreeBytes,
    UnreclaimableBytes,
    DeferQueueBytes,
    FragmentationRuns,
    EmptyClusterSegments,
}

impl PerDeviceAccountingRow {
    const ALL: [Self; 6] = [
        PerDeviceAccountingRow::AllocatedBytes,
        PerDeviceAccountingRow::FreeBytes,
        PerDeviceAccountingRow::UnreclaimableBytes,
        PerDeviceAccountingRow::DeferQueueBytes,
        PerDeviceAccountingRow::FragmentationRuns,
        PerDeviceAccountingRow::EmptyClusterSegments,
    ];

    const fn statistic(self) -> u16 {
        match self {
            PerDeviceAccountingRow::AllocatedBytes => STATISTIC_ALLOCATED_BYTES,
            PerDeviceAccountingRow::FreeBytes => STATISTIC_FREE_BYTES,
            PerDeviceAccountingRow::UnreclaimableBytes => STATISTIC_UNRECLAIMABLE_BYTES,
            PerDeviceAccountingRow::DeferQueueBytes => STATISTIC_DEFER_QUEUE_BYTES,
            PerDeviceAccountingRow::FragmentationRuns => STATISTIC_FRAGMENTATION_RUNS,
            PerDeviceAccountingRow::EmptyClusterSegments => STATISTIC_EMPTY_CLUSTER_SEGMENTS,
        }
    }
}

/// 一次发布写的记账行（池级三行加每块盘六行，代 = 这次的 txg、seq 一律 1，D8（核心索引结构） 已定项 10），按 key 升序。
/// 准入之前算记账树形状（只要 key，值给 0）与装记账树（值从分配器取）读的是这一份行表。
fn accounting_entries_of_this_publish(
    txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    device_identities: &[DeviceIdentity],
    pool_wide_value: &dyn Fn(PoolWideAccountingRow) -> u64,
    per_device_value: &dyn Fn(PerDeviceAccountingRow, DeviceIdentity) -> u64,
) -> Vec<AccountingEntry> {
    let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL
        .into_iter()
        .map(|row| AccountingEntry {
            statistic: row.statistic(),
            tree: row.tree(inode_tree),
            device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
            generation: txg,
            value: pool_wide_value(row),
            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
        })
        .collect();
    for device in device_identities {
        for row in PerDeviceAccountingRow::ALL {
            entries.push(AccountingEntry {
                statistic: row.statistic(),
                tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                device: *device,
                generation: txg,
                value: per_device_value(row, *device),
                sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
            });
        }
    }
    entries.sort_by_key(AccountingEntry::sort_key);
    entries
}

/// 一个这次发布里新写出的码 2 / 码 3 单元的映射 key（D19（块指针的结构与宽度预算） 已定项 6）：类标签、出生 (树, txg)、实例代号、出生序号。
/// key 不看落点：这次要写哪几个单元、各自的出生序号在取落点之前就定了（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9）。
fn mapping_key_of_a_node_born_in_this_publish(
    unit_class: u8,
    tree: TreeIdentifier,
    txg: CheckpointTxg,
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    mapping_key_for_node(
        unit_class,
        NodePointer {
            head: PointerHead {
                birth_tree: tree,
                birth_txg: txg,
            },
            locations: NodePointer::empty_root().locations,
            instance,
            birth_sequence,
        },
    )
}

/// 从上一版取一个照抄进来的角色的映射 key。
///
/// # Panics
/// 上一版里没有这个角色的映射 key：照抄进来的角色恒是上一版里进映射的那一个（上一版装映射节点时逐个记下了）。
fn mapping_key_carried_from(previous: &TransactionOutput, identity: TransactionUnit) -> Vec<u8> {
    previous
        .mapped_units
        .iter()
        .find(|(mapped, _)| *mapped == identity)
        .map(|(_, key)| key.clone())
        .expect("照抄进来的角色在上一版里进映射、有一把 key")
}

/// 按 key 空间定形状的两棵树这次之后的样子（`resolve` 给映射排 key 要它们的节点清单）。
struct MappedPositionAddressedTrees<'plans> {
    extent: &'plans ExtentTreePlan,
    allocation_records: &'plans AllocationRecordTreePlan,
}

/// 这张计划要写的记账树与中央映射树，在一切落盘动作之前算完：记账树按这一版的记账 key，中央映射树按这一版每个进映射单元的 key。
struct MultiLevelTreesOfThisPublish {
    accounting_tree: CodeTwoTreePlan,
    central_mapping_tree: CodeTwoTreePlan,
    mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
}

impl PublishPlan<'_> {
    /// 这次发布写的文件内容按切分纪律切出来的一单元事务，按单元序号升序（= extent key 升序，D16（发布语义） 已定项 5 末段）；
    /// 没有文件版本的发布（写行、暖机、建 inode）为空。事务号从 `transaction` 起连号。
    /// 角色清单、数据单元的装法、记录的切法都读这一份，不各算一遍（`crate::write_request_split` 是切分唯一的实现）。
    #[must_use]
    pub fn file_content_transactions(&self) -> Vec<OneUnitTransaction> {
        match &self.file {
            Some(file) => split_sequential_write_into_one_unit_transactions(
                u64::try_from(file.content.len()).expect("内容长度"),
                self.transaction,
            ),
            None => Vec::new(),
        }
    }

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

    /// 把这张计划算成「这次之后 extent 树、inode 树、记账树、中央映射树各长什么样、这次重写哪些角色」。`trees` 是这一版那八棵树的号
    /// （接在上一版之后就是上一版的，第一个文件版本是这次发出来的），新分裂出来的叶容器的出生树取其中的 inode 树。
    /// `device_identities` 是记账行按盘展开的那几块盘（与装记账行时的分配器同一批），`capacities` 是两棵多层码 2 树的节点容量
    /// （写入口上那个只供测试的开关，产品路径按节点格式算）。`allocation_record_tree` 是这次之后分配记录树的样子：它重写哪几个节点
    /// 要在分配器的拷贝上走到固定点才知道（取落点会改记录），由调用方定下来交进来（`settle_the_allocation_record_tree`），
    /// 这里按它排角色、发映射 key。
    ///
    /// # Errors
    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`。
    pub fn resolve(
        &self,
        previous: Option<&TransactionOutput>,
        trees: &FileVersionTreeIdentifiers,
        device_identities: &[DeviceIdentity],
        capacities: CodeTwoTreeNodeCapacities,
        allocation_record_tree: AllocationRecordTreePlan,
    ) -> Result<ResolvedPublish, PublishError> {
        let containers_before = previous
            .map(TransactionOutput::inode_leaf_container_contents)
            .unwrap_or_default();
        let inode_tree = write_records_into_leaf_containers(
            &containers_before,
            &self.inode_record_writes(),
            self.txg,
            trees.inode,
        )?;
        let file_content_transactions = self.file_content_transactions();
        let extent_tree = plan_the_extent_tree_after_this_publish(
            previous.map(|version| &version.extent_tree),
            self.file
                .as_ref()
                .map(|_| u64::try_from(file_content_transactions.len()).expect("这次写的单元数")),
        );
        let mut rewritten_roles: Vec<TransactionUnit> = file_content_transactions
            .iter()
            .map(|transaction| TransactionUnit::Data(transaction.unit_index_in_file))
            .collect();
        if let InstanceTablePlan::Rewrite(rewrite) = &self.instance_table {
            rewritten_roles.extend(instance_table_page_roles_in_bump_order(
                rewrite.pages_after_this_publish(),
            ));
        }
        rewritten_roles.extend(extent_tree.rewritten_roles());
        for index in &inode_tree.rewritten {
            rewritten_roles.push(TransactionUnit::InodeLeafContainer(*index));
        }
        if !inode_tree.rewritten.is_empty() {
            rewritten_roles.push(TransactionUnit::InodeRoot);
        }
        rewritten_roles.extend(
            allocation_record_tree
                .rewritten_nodes()
                .into_iter()
                .map(role_of_allocation_record_tree_node),
        );
        let MultiLevelTreesOfThisPublish {
            accounting_tree,
            central_mapping_tree,
            mapped_units,
        } = self.plan_the_multi_level_trees(
            previous,
            trees,
            &MappedPositionAddressedTrees {
                extent: &extent_tree,
                allocation_records: &allocation_record_tree,
            },
            &inode_tree,
            device_identities,
            capacities,
        )?;
        // 树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤），记账树（树 14）在前、中央映射树倒数第二、树表最末。
        for (tree, plan) in [
            (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
            (MultiLevelCodeTwoTree::CentralMapping, &central_mapping_tree),
        ] {
            rewritten_roles.extend(
                plan.rewritten_positions()
                    .into_iter()
                    .map(|position| tree.role_of_node(position, &plan.shape)),
            );
        }
        rewritten_roles.push(TransactionUnit::TreeTable);
        Ok(ResolvedPublish {
            inode_tree,
            extent_tree,
            allocation_record_tree,
            accounting_tree,
            central_mapping_tree,
            mapped_units,
            rewritten_roles,
        })
    }

    /// 记账树与中央映射树这次之后的形状，与这一版每个进映射单元的映射 key（`resolve` 的后一半）。
    fn plan_the_multi_level_trees(
        &self,
        previous: Option<&TransactionOutput>,
        trees: &FileVersionTreeIdentifiers,
        position_addressed_trees: &MappedPositionAddressedTrees<'_>,
        inode_tree: &InodeLeafContainersAfterThisPublish,
        device_identities: &[DeviceIdentity],
        capacities: CodeTwoTreeNodeCapacities,
    ) -> Result<MultiLevelTreesOfThisPublish, PublishError> {
        let txg = self.txg;
        let instance = self.instance;
        let empty_shape = CodeTwoTreeShape::default();
        let previous_shape_of = |tree: MultiLevelCodeTwoTree| match previous {
            Some(previous_version) => &previous_version.multi_level_tree(tree).shape,
            None => &empty_shape,
        };
        let refused = |tree: MultiLevelCodeTwoTree| {
            move |refusal: CodeTwoTreeRefusal| PublishError::MultiLevelCodeTwoTreeRefused {
                tree,
                refusal,
            }
        };
        // 记账树：每次发布整批重写（代 = 这次的 txg），key 只看统计量、树、盘与代，不看值。
        let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(
            txg,
            trees.inode,
            device_identities,
            &|_| 0,
            &|_, _| 0,
        )
        .iter()
        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
        .collect();
        let accounting_tree = plan_the_tree_after_this_publish(
            previous_shape_of(MultiLevelCodeTwoTree::Accounting),
            &accounting_keys,
            capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
        )
        .map_err(refused(MultiLevelCodeTwoTree::Accounting))?;

        // 这一版每个进映射的单元的 key：重写的现算，照抄的取上一版的。出生序号在各自那棵树里按 bump 次序从 0 发（数据单元不发，
        // 码 1 的 key 带写序）：extent 树下段先叶后根、再上段先叶后根，inode 树先叶后根，分配记录树按位置先叶后根。
        let node_key = |unit_class: u8, tree: TreeIdentifier, birth_sequence: usize| {
            mapping_key_of_a_node_born_in_this_publish(
                unit_class,
                tree,
                txg,
                instance,
                BirthSequence(
                    u32::try_from(birth_sequence).expect("一棵树一次发布的单元数装得进 u32"),
                ),
            )
        };
        let carried = |identity: TransactionUnit| {
            mapping_key_carried_from(
                previous.expect("照抄的角色只出现在接着上一版的发布里"),
                identity,
            )
        };
        let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = Vec::new();
        match &self.file {
            Some(_) => {
                for transaction in self.file_content_transactions() {
                    mapped_units.push((
                        TransactionUnit::Data(transaction.unit_index_in_file),
                        mapping_key_for_data(
                            PointerHead {
                                birth_tree: trees.extent,
                                birth_txg: txg,
                            },
                            WriteOrder {
                                instance,
                                transaction: transaction.transaction_number,
                            },
                        ),
                    ));
                }
            }
            None => {
                let carried_version =
                    previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄");
                for position in 0..carried_version.data_pointers.len() {
                    let identity = TransactionUnit::Data(DataUnitIndexInFile(
                        u64::try_from(position).expect("单元序号"),
                    ));
                    mapped_units.push((identity, carried(identity)));
                }
            }
        }
        // extent 树的每个节点（下段与上段）都进映射（码 2 节点，D19（块指针的结构与宽度预算） 已定项 8）：重写的按 bump 次序发出生序号，
        // 照抄的取上一版那个角色的 key（上段的根层级这次没变时角色名照旧；变了的只有这次重写的那一串）。
        let extent_plan = position_addressed_trees.extent;
        let previous_upper_root_level =
            previous.map(|previous_version| previous_version.extent_tree.upper_root().level);
        let mut extent_birth_sequence = 0;
        let upper_root_level = extent_plan.upper_root_level();
        let extent_nodes = extent_plan
            .lower_nodes
            .iter()
            .map(|(position, origin)| {
                (
                    TransactionUnit::ExtentLowerNode(*position),
                    TransactionUnit::ExtentLowerNode(*position),
                    *origin,
                )
            })
            .chain(extent_plan.upper_nodes.iter().map(|(position, origin)| {
                (
                    role_of_extent_upper_node(*position, upper_root_level),
                    role_of_extent_upper_node(
                        *position,
                        previous_upper_root_level.unwrap_or(upper_root_level),
                    ),
                    *origin,
                )
            }));
        for (identity, identity_in_the_previous_version, origin) in extent_nodes {
            match origin {
                ExtentTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(UNIT_CLASS_INDEX_NODE, trees.extent, extent_birth_sequence),
                    ));
                    extent_birth_sequence += 1;
                }
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                    mapped_units.push((identity, carried(identity_in_the_previous_version)));
                }
            }
        }
        let mut inode_tree_birth_sequence = 0;
        for position in 0..inode_tree.containers.len() {
            let index = InodeLeafContainerIndexInTree::of_position(position);
            let identity = TransactionUnit::InodeLeafContainer(index);
            if inode_tree.rewritten.contains(&index) {
                mapped_units.push((
                    identity,
                    node_key(UNIT_CLASS_PACKED, trees.inode, inode_tree_birth_sequence),
                ));
                inode_tree_birth_sequence += 1;
            } else {
                mapped_units.push((identity, carried(identity)));
            }
        }
        mapped_units.push(if inode_tree.rewritten.is_empty() {
            (
                TransactionUnit::InodeRoot,
                carried(TransactionUnit::InodeRoot),
            )
        } else {
            (
                TransactionUnit::InodeRoot,
                node_key(
                    UNIT_CLASS_INDEX_NODE,
                    trees.inode,
                    inode_tree_birth_sequence,
                ),
            )
        });
        // 分配记录树的每个节点都进映射：重写的按位置先叶后根发出生序号，照抄的取上一版同一个位置那个角色的 key（位置不随版本挪动）。
        let mut allocation_birth_sequence = 0;
        for (node, origin) in &position_addressed_trees.allocation_records.nodes {
            let identity = role_of_allocation_record_tree_node(*node);
            match origin {
                AllocationRecordTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(
                            UNIT_CLASS_INDEX_NODE,
                            trees.allocation_records,
                            allocation_birth_sequence,
                        ),
                    ));
                    allocation_birth_sequence += 1;
                }
                AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion => {
                    mapped_units.push((identity, carried(identity)));
                }
            }
        }
        let previous_accounting_shape = previous_shape_of(MultiLevelCodeTwoTree::Accounting);
        let mut accounting_birth_sequence = 0;
        for (node, origin) in accounting_tree
            .shape
            .nodes()
            .iter()
            .zip(&accounting_tree.origins)
        {
            let identity = MultiLevelCodeTwoTree::Accounting
                .role_of_node(node.position, &accounting_tree.shape);
            match origin {
                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(
                            UNIT_CLASS_INDEX_NODE,
                            trees.accounting,
                            accounting_birth_sequence,
                        ),
                    ));
                    accounting_birth_sequence += 1;
                }
                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
                    let previous_identity = MultiLevelCodeTwoTree::Accounting
                        .role_of_node(*previous_position, previous_accounting_shape);
                    mapped_units.push((identity, carried(previous_identity)));
                }
            }
        }
        let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units
            .iter()
            .map(|(_, key)| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
            .collect();
        assert_eq!(
            mapping_keys.len(),
            mapped_units.len(),
            "进映射的单元各一把 key，互不相撞（D19（块指针的结构与宽度预算） 已定项 6：出生身份各不相同）"
        );
        let central_mapping_tree = plan_the_tree_after_this_publish(
            previous_shape_of(MultiLevelCodeTwoTree::CentralMapping),
            &mapping_keys,
            capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
        )
        .map_err(refused(MultiLevelCodeTwoTree::CentralMapping))?;
        Ok(MultiLevelTreesOfThisPublish {
            accounting_tree,
            central_mapping_tree,
            mapped_units,
        })
    }
}

/// 文件版本那条 inode 记录（D8（核心索引结构） 已定项 6 的字段表）：size = 这次写的内容长度；
/// blocks 由 size 现算（⌈size ÷ 512⌉，[`InodeRecord::to_bytes`]），不按这个对象分到几个数据单元填。
fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {
    InodeRecord {
        inode: FIRST_INODE_NUMBER,
        object_birth: file.inode_object_birth,
        size: u64::try_from(file.content.len()).expect("文件长度"),
        change_count: file.change_count,
        write_time_seconds: file.write_time_seconds,
    }
}

/// 一个池里的第一个文件版本（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
/// 建在树表 0 条的那一版上（`version_to_build_on`）：照抄它的实例表指针、换下它指着的那片 mkfs 树表。
///
/// txg 与 jsn 都从它接着算——txg = 那一版的 txg + 1、jsn = 上一条记录的 jsn + 1，**不写死 3**：
/// `FIRST_TRANSACTION_TXG` 只管 mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两次 → 第一个事务），不管任何池的第一个文件版本
/// （2026-09-23 用户定案）。只做过 mkfs 的池重开一次可写挂载之后再写文件时，那一版已经推到 txg 4，这里接着写 txg 5。
///
/// 这一版的八棵树从 `version_to_build_on` 的树 ID 水位起连号发（[`FileVersionTreeIdentifiers::issued_from_watermark`]），
/// 新水位 = max(那一版的水位, 发出的最高号 + 1)（D8（核心索引结构） 已定项 8 ②）。那一版的水位就是根环里全部根记录的 max：
/// 它是这个会话里的现行那一版，挂载时本实例的第一次发布取过环里的 max（`mount` 的 `tree_identifier_watermark_of_the_ring`），
/// 之后每次发布只照抄或推高它，而环里后来写进去的根都是这个会话自己写的。mkfs 同一个进程里那条流上环里只有 mkfs 的根与暖机根，
/// 都带 mkfs 种下的 11。
///
/// # Errors
/// `version_to_build_on` 那一版的树表读不出、解不开 ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`；
/// 那一版的树表不是 0 条（已经有过文件版本）⇒ `FirstFileVersionOnAVersionThatAlreadyHasAFile`（同一个文件再写一版走
/// `publish_overwrite`）。`previous_record_bytes` 解不出本池的记录，或它的 checkpoint_txg 与 `version_to_build_on` 那条根的不同
/// （两者要说同一版，新根才恒落在它上面一格）⇒ `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`
/// （m2-emptypool-nonempty-r1 云端攻方腿 Z3-A：拿 txg 4 的暖机根配 txg 2 的记录，新根盖在已有的根上、冷恢复读不到）。
/// 那一版的水位离 `u64::MAX` 不到八个号 ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
/// 这几样都在任何写之前返回，一个字节都不写。其余同 `publish_version`。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    version_to_build_on: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    // 下面补记零单元根会动分配器：冻结着一次没重发的发布时连它也不许做，第一道就拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
    refuse_while_a_publish_is_frozen(allocator)?;
    // 「树还没建起来」看那一版的树表有几条（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步），不看水位：
    // 回退到树表 0 条的一版时水位带着根环里的 max（D8（核心索引结构） 已定项 8 ②），那一版没有树、水位却早已不是 mkfs 的 11。
    let tree_table_entries = tree_table_entry_count(&*pool.devices, version_to_build_on)
        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;
    if tree_table_entries != 0 {
        return Err(
            PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries },
        );
    }
    let previous_record = JournalRecord::parse(
        previous_record_bytes,
        unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
    )
    .map(|record| (record.checkpoint_txg, record.counter));
    let follows_directly = previous_record
        .is_some_and(|(previous_txg, _)| previous_txg == version_to_build_on.checkpoint_txg);
    if !follows_directly {
        return Err(
            PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
                version_to_build_on: version_to_build_on.checkpoint_txg,
                previous_record,
            },
        );
    }
    let (_, previous_counter) =
        previous_record.expect("follows_directly 为真时上一条记录解得出（is_some_and）");
    // 八棵树从那一版的水位（下一个可用号）起连号发：水位是 mkfs 的 11 时就是 D8（核心索引结构） 已定项 11 那几个常量，
    // 回退到树表 0 条的一版之后水位带着环里的 max，号高于此前发过的每一个，不重发。
    let (tree_identifiers, next_available_after_this_issue) =
        FileVersionTreeIdentifiers::issued_from_watermark(
            version_to_build_on.tree_identifier_watermark,
        )
        .map_err(PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees)?;
    let first_file_version_txg = CheckpointTxg(version_to_build_on.checkpoint_txg.0 + 1);
    // 那一版之前调用方直接发的零单元发布没经分配器：先把它们写下的根补记进分配器那张根环表，这次的根才接得上
    // （`PoolAllocator::record_zero_unit_roots_leading_to`；补记的根都已在盘上，这次发布失败也不用退）。
    allocator.record_zero_unit_roots_leading_to(version_to_build_on.checkpoint_txg);
    publish_version_of_trees_holding_one_data_unit(
        pool,
        allocator,
        PublishPlan {
            txg: first_file_version_txg,
            counter: previous_counter + 1,
            transaction: FIRST_TRANSACTION_NUMBER,
            // 这个实例的第一个事务：在它之前只有暖机与写行那几次发布，事务号都是 0。
            highest_transaction_number_before_this_publish: 0,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: first_file_version_txg,
                // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88 的字段表定义）：
                // 就是这次发布的 txg（mkfs 同一个进程里那条流上是 3）。写 1 是 2026-09-18 之前的老样子
                // （暖机把第一个事务从 txg 1 推到 3 时这一格没跟着改，增补 2 第 11 行）。
                // 这里不写 `txg.0`：`crates/mutations.tsv` 第 22 行按那串字面锚在 `publish_overwrite` 上，同一份文件里出现两次它就腐化。
                change_count: first_file_version_txg.0,
            }),
            // 第一个事务只建第一个文件那一个 inode：树是空的，这条记录建第一片容器（D8（核心索引结构） 已定项 6）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),
            tree_birth_txg: first_file_version_txg,
            // D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)；
            // 前一项就是那一版的水位（见文档注释），后一项恒更大。
            tree_identifier_watermark: version_to_build_on
                .tree_identifier_watermark
                .max(next_available_after_this_issue),
            // F 照抄建在上面的那一版（D16（发布语义） 已定项 1：F 只升不降）。mkfs 同一个进程里那条流上它恒是 0，
            // 重开之后写行那次发布带的是恢复后生效的 F，这里接着带它。
            rollback_floor: version_to_build_on.rollback_floor,
        },
        None,
        tree_identifiers,
    )
}

/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里接在上一次发布之后再发布一版同一个文件——txg、jsn、事务号各加一，
/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的八个落点经映射释放（进 defer 队列）。
/// 契约是一个数据单元：多个数据单元的内容走 `publish_sequential_write`。
///
/// # Errors
/// 内容装不进一个数据单元 ⇒ `ContentExceedsDataUnit`（在动分配器、动盘之前）；
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
    publish_version_holding_one_data_unit(
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
            // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max（本会话每次发布只照抄或推高它，见 `publish_first_file`）。
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 顺序写一次写请求（里程碑「第二个事务」并行线一）：从文件偏移 0 写整份内容，按切分纪律切成 N 个一单元事务
/// （`crate::write_request_split`：D16（发布语义） 已定项 5 末段 + D23（journal 的角色与格式） 已定项 7 +
/// C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），N 个事务在同一次发布里写出 N 个数据单元、N 条记录：
/// 前 N − 1 条各只点名自己那个数据单元，这次发布共享的提交内生块只在最后一条点名（D23（journal 的角色与格式） 已定项 17，
/// C491（多条记录时共享内生块在哪条点名没定） 2026-09-23 定）；extent 叶记录 key 的 offset 段是文件字节偏移
/// （D8（核心索引结构） 已定项 3，C490（extent 叶 key 的 offset 段没定单位） 2026-09-23 定）。
///
/// 走的是 `publish_version` 那一条发布路径（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」：这里不另开一条）：
/// 只要一个数据单元的内容，写出的字节与同内容的 `publish_overwrite` 一个不差。上一版比这一版多出来的数据单元
/// 在同一次发布里释放（文件变短的那一格，`placements_to_release_via_mapping`）。
///
/// # Errors
/// 切出的单元多于一片 extent 叶装得下的记录数 ⇒ `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`
/// （extent 内部条目的格式没有条款），在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
/// 最后一个事务要点名的项多于一条记录装得下的，末条再跨记录（D23（journal 的角色与格式） 已定项 17），不拒。其余与 `publish_overwrite` 相同
/// （不报 `ContentExceedsDataUnit`：多个数据单元正是这条路径要写的）。
pub fn publish_sequential_write<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
    // 改动计数与第一个事务号先取成局部量、字段也换了次序：`crates/mutations.tsv` 有几条变异按 `publish_overwrite` 里
    // `change_count: txg.0,`、`transaction: previous.highest_transaction_number_in_this_instance + 1,` 与那几行字段的原样
    // 锚着，同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_new_inodes` 里那句注释是同一件事）。
    let change_count = txg.0;
    let first_transaction_of_this_request =
        previous.highest_transaction_number_in_this_instance + 1;
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: first_transaction_of_this_request,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            file: Some(FileVersionPlan {
                content: file.content,
                // 对象出生代不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
                inode_object_birth: previous.inode_record.object_birth,
                write_time_seconds: file.write_time_seconds,
                change_count,
            }),
            back_chain: back_chain_of(&previous.record_bytes),
            // 顺序写不建新 inode：只把第一个文件那条记录换一版，它落在原来那一片叶里（容器身份不变）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max。
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 建 N 个 inode（里程碑「第二个事务」并行线三：多个文件、元数据按「个」量、没有目录）：一次发布把 N 条 inode 记录
/// 写进 inode 树——号从记账里的 inode 号水位起连着发（D5（快照 / 空间记账机制） 已定项 4 第 12 项），
/// 落在最右那片叶、满 233 条就在末尾分裂（D8（核心索引结构） 已定项 6，`crate::inode_tree`），
/// 这次发布之后水位加 N；数据单元与 extent 树不动（新建的 inode 长度 0，`blocks` = ⌈0 ÷ 512⌉ = 0）。
///
/// 这次发布是**一个事务**：D16（发布语义） 已定项 5 的切分纪律逐字「一个事务最多写一个单元的用户数据」，
/// 建 inode 一个用户数据单元都不写 ⇒ 不切；一个事务一条 journal 记录（D23（journal 的角色与格式） 已定项 7）。
/// 「N 个创建要不要各算一个事务」仓里没有条款，这一版按「不切」走（报告里交主 agent 定）。
///
/// # Errors
/// 记录落法要的条款没定（中间插入、容器数超过一个根装得下的）⇒ `InodeTreeWriteRefused`；
/// 其余与 `publish_overwrite` 相同。两样都在任何落盘动作之前返回，盘上逐字节不变。
/// 这次要点名的单元超过一条 journal 记录装得下的 67 项时不拒：这个事务的记录再跨几条，只有真正的最后一条带
/// 「本次发布末条」标志与提交标记（D23（journal 的角色与格式） 已定项 17 / 已定项 7）。
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
            // 建出来还没写过数据：长度 0，blocks 跟着是 0。
            size: 0,
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

/// `publish_first_file` 与 `publish_overwrite` 的契约是一个数据单元（多个数据单元的内容走 `publish_sequential_write`）：
/// 内容装不进一个数据单元是调用方能恢复的失败，不是不变量被破坏——报错，在动分配器、动盘之前返回，不走到 `build_data_unit` 的断言。
///
/// # Errors
/// `ContentExceedsDataUnit`。
fn refuse_a_file_that_does_not_fit_one_data_unit(
    plan: &PublishPlan<'_>,
) -> Result<(), PublishError> {
    if let Some(file) = &plan.file {
        let data_unit_capacity = data_unit_payload_capacity();
        if file.content.len() > data_unit_capacity {
            return Err(PublishError::ContentExceedsDataUnit {
                bytes: file.content.len(),
                capacity: data_unit_capacity,
            });
        }
    }
    Ok(())
}

/// `publish_overwrite` 那一路：先按一个数据单元的契约判内容，再走 `publish_version`。
fn publish_version_holding_one_data_unit<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    refuse_a_file_that_does_not_fit_one_data_unit(&plan)?;
    publish_version(pool, allocator, plan, previous)
}

/// `publish_first_file` 那一路：先按一个数据单元的契约判内容，再走 `publish_version_of_trees`（八棵树的号由它给）。
fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
) -> Result<TransactionOutput, PublishError> {
    refuse_a_file_that_does_not_fit_one_data_unit(&plan)?;
    publish_version_of_trees(pool, allocator, plan, previous, trees)
}

/// 发布一版：先做准入（extent 树装得下这个文件的数据单元），
/// 再释放上一版被换下的角色的落点、分配、装单元、
/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
/// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
/// 落盘那几步里失败的，这次已记的写进写入口的失败账（增补 2 第 20b 行，`PoolWriter::writes_of_failed_publishes`）。
///
/// 这一版那八棵树的号：接在上一版之后照抄上一版的（树 ID 只在第一个文件版本那次发）。没有上一版的只有第一个文件版本，
/// 这里按 mkfs 那条流的水位 11 发（D8（核心索引结构） 已定项 11 登记的那几个常量）；从别的水位发号的第一个文件版本
/// （回退到树表 0 条的一版之后）走 `publish_first_file`，它按要建在上面的那一版的水位发。
///
/// # Errors
/// `InodeTreeWriteRefused`、`ContentExceedsDataUnit`、`MultiLevelCodeTwoTreeRefused`、
/// `AllocationRecordTreeRewriteSetDidNotSettle`、释放判定路径的四种错、`PlacementRefused`、块设备错。
///
/// # Panics
/// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 19：调用方把一个水位早已推高的池当成了 mkfs 那条流，
/// 按 11 发号会重发已经发过的号（D8（核心索引结构） 已定项 8 ②）。
pub fn publish_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    let trees = match previous {
        Some(previous_version) => previous_version.tree_identifiers,
        None => {
            assert_eq!(
                plan.tree_identifier_watermark, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
                "没有上一版的发布在这里按 mkfs 那条流的水位 11 发号：计划写的新水位必须就是那条流的 19，别的水位走 publish_first_file"
            );
            FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
                .expect("11 加八个号装得下")
                .0
        }
    };
    publish_version_of_trees(pool, allocator, plan, previous, trees)
}

/// `publish_version` 的本体，这一版那八棵树的号由调用方给：`publish_first_file` 给它这次从水位发出来的，
/// `publish_version` 给上一版的（或 mkfs 那条流的）。
fn publish_version_of_trees<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
) -> Result<TransactionOutput, PublishError> {
    // 第一道：冻结着一次没重发的发布就在任何读写之前拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
    refuse_while_a_publish_is_frozen(allocator)?;
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let built = prepare_the_version_publish(
        pool,
        allocator,
        &plan,
        previous,
        trees,
        ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
    );
    let (writes, output) = match built {
        Ok(built) => built,
        Err(refusal) => {
            *allocator = allocator_before_this_publish;
            return Err(refusal);
        }
    };
    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。
    persist_the_publish_or_freeze_it(
        pool,
        allocator,
        allocator_before_this_publish,
        writes,
        PoolVersion::WithFile(output),
    )
    .map(|version| {
        version
            .into_file_version()
            .expect("交给落盘的是带文件的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）")
    })
}

/// 释放之前读不读盘核被换下的每一份的校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseChecksumCheck {
    /// 发布路径：读盘核，核出对不上（读不出也算）的单元在每块盘上的分配记录留在已分配。
    ReadEveryReplacedCopyBeforeReleasingIt,
    /// 可写挂载取号之前的预演（`mount`）：不读盘、照常释放——预演只要这一串取落点的次序与结局；
    /// 两边只在「有一份核出对不上、那一槽又在这一串里被回收」时分叉，那一格见 `mount::establish_instance` 的注释。
    SkippedByTheDryRunBeforeAcquisition,
}

/// 带文件的一版的一次发布，落盘之前的全部：算这次之后每棵树的样子（分配记录树重写哪几个节点走到固定点，
/// [`settle_the_allocation_record_tree`]）、释放判定、释放之前读盘核（`release_checksum_check` 说核不核）、释放、取落点、装单元，
/// 交回装好、还没落盘的字节与这一版（写账是空的）。只动 `allocator`：失败时它停在中途，由调用方换回去——发布路径换回进来时那一份，
/// 可写挂载取号之前的预演拿的本来就是拷贝。两处走的是同一段代码，预演取到的落点因此就是真发时取到的（`mount::establish_instance` 断言）。
///
/// # Errors
/// `InodeTreeWriteRefused`、`MultiLevelCodeTwoTreeRefused`、释放判定路径的几种错、`PlacementRefused`。
pub fn prepare_the_version_publish<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
    release_checksum_check: ReleaseChecksumCheck,
) -> Result<(PublishWrites, TransactionOutput), PublishError> {
    // 先算这次之后每棵树是什么样、这次重写哪些角色、换下哪些：只读（固定点在分配器的拷贝上走），条款没写的那几格与多层码 2 树的拒绝、
    // 释放判定的错、取不到落点都在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    let settled = settle_the_allocation_record_tree(
        plan,
        previous,
        &trees,
        &device_identities_of_the_accounting_rows,
        pool.code_two_tree_node_capacities(),
        allocator,
    )?;
    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）：读数取分配器此刻的计数——这次的释放与取落点都还没做——，
    // 需求只算这次的普通分配（`admission::space_budget_of_role`）；一个普通分配都没有的发布（空发布、写行）不判，
    // 它们的空间在保留池与切换预留里（`admission` 的模块文档）。在读盘核与动分配器之前拒，盘上逐字节不变。
    // 只供测试的开关关掉准入时（`SpaceAdmission::SkippedByTheTestOnlySwitch`）不判，留给落点那一道在任何写之前拒。
    let demand = demand_of_the_roles_on_each_device(
        &settled.resolved.rewritten_roles,
        &device_identities_of_the_accounting_rows,
    );
    let demand_is_judged = match allocator.space_admission() {
        SpaceAdmission::JudgedByTheFormula => demand
            .iter()
            .any(|demand_on_device| demand_on_device.bytes != BytesOnOneDevice::ZERO),
        SpaceAdmission::SkippedByTheTestOnlySwitch => false,
    };
    if demand_is_judged {
        admit_on_every_device(
            &admission_reading_before_a_publish(allocator, previous),
            &demand,
        )
        .map_err(PublishError::SpaceAdmissionRefused)?;
    }
    // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
    // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
    let quarantine = match (previous, release_checksum_check) {
        (Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {
            copies_failing_the_release_checksum_check(
                previous_version,
                &settled.previous_roles_replaced,
                &*pool.devices,
            )?
        }
        (Some(previous_version), ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition) => {
            // 预演不读盘，但映射条目的位置项指池外的盘、两条指同一块盘那一判不读盘，照样判（发布路径在读盘核之前判的同一件事）。
            refuse_mapping_entries_that_do_not_name_two_pool_devices(
                previous_version,
                &settled.previous_roles_replaced,
                &device_identities_of_the_accounting_rows,
            )?;
            Vec::new()
        }
        (None, _) => Vec::new(),
    };
    publish_admitted(
        pool,
        allocator,
        plan,
        previous,
        &settled.resolved,
        &settled.release,
        &quarantine,
        trees,
    )
}

/// 这次发布算定了的样子：每棵树这次之后的形状与重写的角色、这次换下的上一版角色、要释放的落点（按 bump 次序）。
struct SettledPublish {
    resolved: ResolvedPublish,
    previous_roles_replaced: Vec<TransactionUnit>,
    release: Vec<Placement>,
}

/// 这次发布换下的上一版角色与要释放的落点（释放判定路径，只查不改，D19（块指针的结构与宽度预算） 已定项 5 第 1 条）：
/// 整条实例表旧链排最前，再是经映射查到的那几个（第一个文件版本没有上一版的内存态：换下的是 mkfs 那片树表与树表 0 条那一版的分配记录树）。
///
/// # Errors
/// 释放判定路径的几种错（`ReleaseNotInMapping` 一族、`ReleaseTarget*`、`ReleaseSpanMismatch`）。
fn placements_released_by_this_publish(
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    resolved: &ResolvedPublish,
    allocator: &PoolAllocator,
) -> Result<(Vec<TransactionUnit>, Vec<Placement>), PublishError> {
    // 换下的是上一版的角色：多层码 2 树与按位置寻址的两棵树的节点按计划里「上一版被换下的节点」点名，别的角色同一个角色上一版那一份。
    let previous_roles_replaced = match previous {
        Some(previous_version) => {
            resolved.previous_roles_replaced_by_this_publish(previous_version)
        }
        None => Vec::new(),
    };
    let release = match previous {
        Some(previous_version) => placements_to_release_via_mapping(
            previous_version,
            allocator,
            &previous_roles_replaced,
        )?,
        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
        None => format_time_tree_table_to_release(allocator, &resolved.rewritten_roles),
    };
    // 整条实例表链重写（写行、回退那一次）：被换下的那条旧链逐片释放，指针取计划带着的那条链（第 1 片起的不在上一版的内存态里）。
    // 旧链排在别的角色前面，与实例表在 bump 次序里排最前同一个次序。
    let release = match &plan.instance_table {
        InstanceTablePlan::Rewrite(rewrite) => {
            if let Some(previous_version) = previous {
                assert_eq!(
                    rewrite.replaced_chain.first(),
                    Some(&previous_version.root.instance_table),
                    "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
                );
            }
            let mut chain_then_the_rest =
                instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;
            chain_then_the_rest.extend(release);
            chain_then_the_rest
        }
        InstanceTablePlan::Carry(_) => release,
    };
    Ok((previous_roles_replaced, release))
}

/// 分配记录树这次重写哪几个节点、这一版有哪几个节点（D8（核心索引结构） 已定项 14：按位置寻址，只重写内容变了的叶与它们的祖先，
/// 没有记录的一段不写节点）。它们要在取落点之前定下来（角色清单、出生序号、映射 key、中央映射树的形状都要它），而取落点又会改记录——
/// 这次写的每个单元、连分配记录树自己的节点，每盘各记一条；换下的每一份把它那条改成已释放（D3（空间分配） 已定项 7；这就是释放链）。
/// 所以走一个固定点：从「重写集空、节点照上一版」起，按它算这次发布的全部角色，在分配器的**拷贝**上把释放与取落点照发布路径的次序
/// 走一遍，按走出来的记录算有记录的节点与内容变了的节点（[`nodes_whose_contents_changed`]）；节点对得上、变了的都在重写集里（或本来就是新节点）
/// 就定下，否则节点换成走出来的那一批、变了的并进重写集再走一遍。多罩的节点内容没变、照样重写一份（合法的 COW）。
/// 真发时走的是同一串释放与取落点，得出同一份记录（释放之前读盘核出对不上的那几份记录留在已分配，只会让变了的节点更少）。
/// 没有上一版（第一个文件版本）时这棵树的树号换成这次发出来的（树表 0 条那一版的分配记录树是树 0），节点全部重写、不照抄。
///
/// **走得完**：产品路径上释放只改写记录、不在这次发布里空出槽（回收要等根记下之后），取落点时分配记录树的节点之后只剩单槽的提交内生块、
/// 按同一串槽位往后取——多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减，都以池里的节点数为上界。
/// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性推不出来：
/// 给 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮，走不完交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
///
/// # Errors
/// 同 [`PublishPlan::resolve`]、[`placements_released_by_this_publish`]，与拷贝上取不到落点（`PlacementRefused`）——
/// 真发时同一个落点同样取不到；轮数用完 ⇒ `AllocationRecordTreeRewriteSetDidNotSettle`。都在任何落盘动作之前交回。
fn settle_the_allocation_record_tree(
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: &FileVersionTreeIdentifiers,
    device_identities: &[DeviceIdentity],
    capacities: CodeTwoTreeNodeCapacities,
    allocator: &PoolAllocator,
) -> Result<SettledPublish, PublishError> {
    let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let (previous_nodes, previous_records): (
        BTreeSet<AllocationRecordTreeNode>,
        &[AllocationRecord],
    ) = match previous {
        Some(previous_version) => (
            previous_version.allocation_record_tree.node_set(),
            &previous_version.allocation_records,
        ),
        None => (BTreeSet::new(), &[]),
    };
    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
    // 迭代上界是 ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT 轮；跨轮携带的是重写集（只增不减）与这一版的节点（换成上一轮走出来的那一批）。
    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
            rewritten.intersection(&nodes_after).copied().collect();
        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
            &previous_nodes,
            &nodes_after,
            &rewritten_among_the_nodes_after,
        );
        let rewritten_by_the_plan: BTreeSet<AllocationRecordTreeNode> =
            allocation_plan.rewritten_nodes().into_iter().collect();
        let resolved = plan.resolve(
            previous,
            trees,
            device_identities,
            capacities,
            allocation_plan,
        )?;
        let (previous_roles_replaced, release) =
            placements_released_by_this_publish(plan, previous, &resolved, allocator)?;
        let mut rehearsal = allocator.clone();
        for placement in &release {
            rehearsal.release(*placement, plan.txg);
        }
        // 释放本身弄脏的叶（连祖先与根）这一轮的计划还没罩住：先并进重写集再取落点。这样取落点被拒时交回的角色
        // 是罩住它们的那一版 bump 次序里第一个取不到的（分配记录树的节点排在前面），不是只罩了一部分的那一版里的。
        // 这一版的节点也先并进释放之后装着记录的那几个（上一版没有树时释放之前就有的记录也在内：mkfs 的两个单元）；
        // 释放只改写记录、不删，这几个节点取完落点之后仍装着记录，下面「装着记录的节点等于这一版的节点」那一判不会因此来回。
        let changed_by_the_release =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
            rewritten.extend(changed_by_the_release);
            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
            continue;
        }
        for identity in &resolved.rewritten_roles {
            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;
        }
        let changed =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        let holding_records = nodes_holding_records(&geometry, rehearsal.records());
        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
            return Ok(SettledPublish {
                resolved,
                previous_roles_replaced,
                release,
            });
        }
        rewritten.extend(changed);
        nodes_after = holding_records;
    }
    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
    })
}

/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。产品路径上三四轮就定（第一轮重写集是空的、
/// 释放弄脏的叶在取落点之前先并进来；下一轮罩住这次写的那几片叶与它们的祖先、再一轮罩住释放链弄脏的叶）；
/// 只供测试的复用窗口置 0 开关下单调性推不出来，这个数给它一个边。
pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;

/// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
/// 2026-09-23 定）：这次写了 N 个数据单元（N ≥ 2）⇒ N 个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），
/// 第 k 个事务（k < N − 1）一条记录、只点名文件第 k 个数据单元；这次发布共享的提交内生块（数据单元之外的每个重写角色）
/// **只在最后一个事务的记录里点名**，与第 N − 1 个数据单元一起，前面几条不重复点名它们。写了一个数据单元或一个都没写 ⇒
/// 一个事务点名全部重写角色（第一个事务今天的形态）。
///
/// 最后一个事务要点名的项多于一条记录装得下的 67 项（[`JOURNAL_NAMED_ENTRIES_PER_RECORD`]）时**末条再跨记录**
/// （已定项 17，用户 2026-09-24 定）：按 bump 次序装满一条再开下一条，这些记录都属于最后一个事务
/// （[`transaction_offset_of_each_record_of_the_publish`]）；只有真正的最后一条带「本次发布末条」标志。
///
/// 每一项按 `rewritten` 里的次序（bump 次序）列角色；返回至少一项，每项至多 67 个角色。落盘写出的记录读的就是它。
#[must_use]
pub fn roles_named_by_each_record_of_the_publish(
    rewritten: &[TransactionUnit],
    named_entry_capacity: JournalRecordNamedEntryCapacity,
) -> Vec<Vec<TransactionUnit>> {
    let data_roles: Vec<TransactionUnit> = rewritten
        .iter()
        .copied()
        .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
        .collect();
    let data_roles_named_before_the_last_record = &data_roles[..data_roles.len().saturating_sub(1)];
    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =
        data_roles_named_before_the_last_record
            .iter()
            .map(|data_role| vec![*data_role])
            .collect();
    roles_named_by_each_record.push(
        rewritten
            .iter()
            .copied()
            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))
            .collect(),
    );
    // 末条再跨记录（已定项 17）：最后一个事务那一项装不下一条记录时切成几条，装满一条再开下一条。
    let roles_named_by_the_last_transaction = roles_named_by_each_record
        .pop()
        .expect("上一句刚推进去最后一个事务那一项");
    if roles_named_by_the_last_transaction.is_empty() {
        // 一个角色都不点名的发布照样写一条记录（它一条就是一次发布）。
        roles_named_by_each_record.push(roles_named_by_the_last_transaction);
    } else {
        let named_unit_capacity = named_entry_capacity.named_entries_per_record();
        roles_named_by_each_record.extend(
            roles_named_by_the_last_transaction
                .chunks(named_unit_capacity)
                .map(<[TransactionUnit]>::to_vec),
        );
    }
    roles_named_by_each_record
}

/// 一次发布里从 0 数第 k 条记录属于这次发布从 0 数的第几个事务（与 [`roles_named_by_each_record_of_the_publish`] 同一种切法）：
/// 前 N − 1 条（N = 这次写的数据单元数，N ≥ 2）各是自己那个数据单元的事务，第 k 条就是第 k 个事务；
/// 其余的——最后一个事务那条记录与它装不下再跨出去的几条（D23（journal 的角色与格式） 已定项 17）——都属于最后一个事务，
/// 共享它的事务号（已定项 7「同一事务的全部记录共享它」）。
#[must_use]
pub fn transaction_offset_of_each_record_of_the_publish(
    rewritten: &[TransactionUnit],
    named_entry_capacity: JournalRecordNamedEntryCapacity,
) -> Vec<u64> {
    let transactions_named_one_record_each = rewritten
        .iter()
        .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
        .count()
        .saturating_sub(1);
    (0..roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity).len())
        .map(|record_offset_in_this_publish| {
            u64::try_from(record_offset_in_this_publish.min(transactions_named_one_record_each))
                .expect("一次发布的事务数装得进 u64")
        })
        .collect()
}

/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
    previous.unit(identity).clone()
}

/// 一个文件对象这次发布的数据单元：单元字节与指针。重写时由 `build_file_version_units` 装，照抄时从上一版取
/// （`carried_file_version_units`）。extent 树的节点不在这里——它们按这些指针由 `build_extent_tree` 装；
/// 这个对象的 inode 记录也不在这里——它进 inode 树，由 `build_inode_tree_units` 按叶容器装。
struct FileVersionUnits {
    /// 文件的数据单元，第 i 项是文件第 i 个单元（与 `data_pointers` 同序）。
    data_units: Vec<Vec<u8>>,
    /// 指向每个数据单元的指针，第 i 项是文件第 i 个单元；extent 树下段的叶记录（或内联的那一条）按同一个次序装（key 升序）。
    data_pointers: Vec<DataPointer>,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数；照抄的一版恒 0。
    key_order_mismatches: u64,
}

/// 装文件对象的单元时整个 checkpoint 共用的身份字段。
struct FileVersionCheckpoint<'checkpoint> {
    txg: CheckpointTxg,
    /// 提交内生块（码 3 容器）头里的写序：这次发布末条记录那个事务（共享的提交内生块只在末条点名，
    /// D23（journal 的角色与格式） 已定项 17）。数据单元各带自己那个事务的号，不取它（`build_file_version_units`）。
    write_order: WriteOrder,
    /// 这一版那八棵树各自的号（单元头与指针头部的出生树按它写）。
    trees: FileVersionTreeIdentifiers,
    filesystem_identifier: &'checkpoint [u8; 16],
    /// 位置条目按设备身份升序（I-2.5）。
    device_identities: &'checkpoint [DeviceIdentity],
}

/// 一个文件对象带位置条目的那几个数据单元这次取到的落点，第 i 项是文件第 i 个单元。
#[derive(Clone, Debug)]
struct FileVersionSlots {
    data: Vec<SlotNumber>,
}

/// 装一个文件对象的数据单元（字节表二）：切分纪律切出的每个一单元事务装一个数据单元（写序带那个事务的号，
/// 锚点偏移与 extent key 的 offset 段都是这个单元第一个字节的文件偏移，D8（核心索引结构） 已定项 3、D9（加密） 已定项 6）。
/// 数据单元是码 1、不发出生序号；extent 树的节点按这里交出的指针另装（`build_extent_tree`）。
///
/// # Panics
/// 事务数与落点数不等：角色清单按同一张切分排，不等说明调用方给错了输入。
fn build_file_version_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    file: &FileVersionPlan<'_>,
    transactions: &[OneUnitTransaction],
    slots: &FileVersionSlots,
) -> FileVersionUnits {
    let txg = checkpoint.txg;
    let instance = checkpoint.write_order.instance;
    let filesystem_identifier = checkpoint.filesystem_identifier;
    assert_eq!(
        transactions.len(),
        slots.data.len(),
        "每个一单元事务一个数据单元落点：角色清单与切分读的是同一份内容"
    );
    let mut data_units = Vec::with_capacity(transactions.len());
    let mut data_pointers = Vec::with_capacity(transactions.len());
    let mut extent_keys: Vec<[u8; 24]> = Vec::with_capacity(transactions.len());
    // 迭代次数的上界是这次的单元数；跨轮携带的只有往三个表里追加的那一项，没有提前出口。
    for (transaction, slot) in transactions.iter().zip(&slots.data) {
        let write_order = WriteOrder {
            instance,
            transaction: transaction.transaction_number,
        };
        // t1（字节表二）：五元组的锚点偏移是这个单元第一个字节在文件里的偏移。
        let data_identity = DataUnitIdentity {
            tree: checkpoint.trees.extent,
            object: FIRST_INODE_NUMBER,
            object_birth: file.inode_object_birth,
            anchor_offset: transaction.payload_start.0,
        };
        let data_unit = build_data_unit(
            data_identity,
            txg,
            filesystem_identifier,
            write_order,
            transaction.payload_of(file.content),
        );
        let data_pointer = DataPointer {
            head: PointerHead {
                birth_tree: checkpoint.trees.extent,
                birth_txg: txg,
            },
            locations: location_entries(checkpoint.device_identities, *slot, &data_unit),
            write_order,
        };
        let extent_record = build_extent_record(
            FIRST_INODE_NUMBER,
            transaction.payload_start.0,
            data_pointer,
        );
        extent_keys.push(
            extent_record[..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
                .try_into()
                .expect("24"),
        );
        data_units.push(data_unit);
        data_pointers.push(data_pointer);
    }
    let key_order_mismatches = count_key_order_mismatches(&extent_keys);
    FileVersionUnits {
        data_units,
        data_pointers,
        key_order_mismatches,
    }
}

/// 这一版 extent 树装好的样子：每个节点这一版的指针、全部节点的单元（bump 次序：下段先叶后根、上段先叶后根），与这次重写的节点各自的出生序号。
struct BuiltExtentTree {
    version: ExtentTreeVersion,
    units: Vec<PublishedUnit>,
    birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
}

/// 按计划装 extent 树这一版的全部节点（D8（核心索引结构） 已定项 14 的两段）：重写的节点按 bump 次序逐个装、发出生序号、取这次的落点——
/// 下段先叶后根（叶装它罩的那几个单元的 extent 叶记录，内部节点装「孩子那一段的起点 key + 孩子这一版的指针」），再上段先叶后根
/// （第一个文件那一条叶条目：一个数据单元就内联它的数据指针（标签 2），多个就放下段根这一版的指针（标签 1）；同一片叶里别的 inode 的条目照抄上一版那片叶的）。
/// 照抄的节点字节、落点、指针全取上一版同一个位置上的。`data_pointers` 是第一个文件这一版每个数据单元的指针（第 i 项是单元 i）。
///
/// # Panics
/// 重写的节点没拿到落点、照抄的节点不在上一版里、上一版那片上段叶解不开：三样都是发布路径自己的不变量
/// （角色清单按同一份计划排、上一版的字节是这个进程装的或重建时核过的）。
fn build_extent_tree(
    plan: &ExtentTreePlan,
    previous: Option<&TransactionOutput>,
    data_pointers: &[DataPointer],
    tree: TreeIdentifier,
    context: &MultiLevelTreeBuildContext<'_>,
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> BuiltExtentTree {
    let identity = ExtentTreeNodeIdentity {
        tree,
        birth_txg: context.txg,
        filesystem_identifier: context.filesystem_identifier,
        instance: context.instance,
    };
    let data_units = u64::try_from(data_pointers.len()).expect("单元数");
    let mut units: Vec<PublishedUnit> = Vec::new();
    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
    let mut rewritten_node = |role: TransactionUnit,
                              build: &dyn Fn(BirthSequence) -> Vec<u8>,
                              units_so_far: &mut Vec<PublishedUnit>|
     -> NodePointer {
        let birth_sequence = sequences.next(tree, context.txg, context.instance);
        let bytes = build(birth_sequence);
        let slot = *slots
            .get(&role)
            .expect("这次重写的 extent 树节点都在角色清单里、取了落点");
        birth_sequences_of_rewritten_nodes.insert(role, birth_sequence);
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: tree,
                birth_txg: context.txg,
            },
            locations: location_entries(context.device_identities, slot, &bytes),
            instance: context.instance,
            birth_sequence,
        };
        units_so_far.push(PublishedUnit {
            slot,
            identity: role,
            bytes,
        });
        pointer
    };
    let carried_node = |role_in_the_previous_version: TransactionUnit,
                        role: TransactionUnit,
                        units_so_far: &mut Vec<PublishedUnit>| {
        let previous_version = previous.expect("照抄的 extent 树节点只出现在接着上一版的发布里");
        let carried = previous_version.unit(role_in_the_previous_version);
        units_so_far.push(PublishedUnit {
            slot: carried.slot,
            identity: role,
            bytes: carried.bytes.clone(),
        });
    };
    let mut lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)> = Vec::new();
    // 迭代上界是下段的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
    for (position, origin) in &plan.lower_nodes {
        let role = TransactionUnit::ExtentLowerNode(*position);
        let pointer = match origin {
            ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => rewritten_node(
                role,
                &|birth_sequence| {
                    build_lower_leaf(
                        &identity,
                        FIRST_INODE_NUMBER,
                        *position,
                        data_pointers,
                        birth_sequence,
                    )
                },
                &mut units,
            ),
            ExtentTreeNodeOrigin::RewrittenThisPublish => {
                let children: Vec<(ExtentLowerNodePosition, NodePointer)> =
                    lower_children_of_a_file_without_holes(*position, data_units)
                        .into_iter()
                        .map(|child| {
                            let (_, pointer) = lower_nodes
                                .iter()
                                .find(|(built, _)| *built == child)
                                .expect("孩子在 bump 次序里排在父节点前面、先装好了");
                            (child, *pointer)
                        })
                        .collect();
                rewritten_node(
                    role,
                    &|birth_sequence| {
                        build_lower_internal_node(
                            &identity,
                            FIRST_INODE_NUMBER,
                            *position,
                            &children,
                            birth_sequence,
                        )
                    },
                    &mut units,
                )
            }
            ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                carried_node(role, role, &mut units);
                previous
                    .and_then(|previous_version| {
                        previous_version.extent_tree.lower_pointer_of(*position)
                    })
                    .expect("照抄的下段节点在上一版里")
            }
        };
        lower_nodes.push((*position, pointer));
    }
    let upper_root_level = plan.upper_root_level();
    let previous_upper_root_level =
        previous.map(|previous_version| previous_version.extent_tree.upper_root().level);
    let mut upper_nodes: Vec<(ExtentUpperNodePosition, NodePointer)> = Vec::new();
    // 迭代上界是上段的节点数；跨轮携带的同下段。
    for (position, origin) in &plan.upper_nodes {
        let role = role_of_extent_upper_node(*position, upper_root_level);
        let role_in_the_previous_version = previous_upper_root_level
            .map(|previous_root_level| role_of_extent_upper_node(*position, previous_root_level));
        let pointer = match origin {
            ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => {
                let target = match lower_nodes.last() {
                    None => ExtentUpperLeafTarget::InlineDataUnit(
                        *data_pointers
                            .first()
                            .expect("写文件内容的发布至少一个数据单元（长度 0 的内容也写一个）"),
                    ),
                    Some((_, lower_root_pointer)) => {
                        ExtentUpperLeafTarget::LowerSegmentRoot(*lower_root_pointer)
                    }
                };
                // 同一片叶里别的 inode 的条目照抄上一版那片叶的（第一版只有第一个文件有内容，今天这一串是空的）。
                let mut entries: Vec<ExtentUpperLeafEntry> = role_in_the_previous_version
                    .filter(|_| {
                        previous.is_some_and(|previous_version| {
                            previous_version
                                .extent_tree
                                .upper_pointer_of(*position)
                                .is_some()
                        })
                    })
                    .map(|previous_role| {
                        let previous_leaf = parse_index_node(
                            &previous.expect("上一版有这片叶").unit(previous_role).bytes,
                        )
                        .expect("上一版的上段叶是这个进程装的，或重建时核过的");
                        previous_leaf
                            .entries
                            .iter()
                            .map(|entry| {
                                ExtentUpperLeafEntry::parse(entry)
                                    .expect("上一版的上段叶条目是这个进程装的，或重建时核过的")
                            })
                            .filter(|entry| entry.inode != FIRST_INODE_NUMBER)
                            .collect()
                    })
                    .unwrap_or_default();
                entries.push(ExtentUpperLeafEntry {
                    inode: FIRST_INODE_NUMBER,
                    target,
                });
                entries.sort_by_key(|entry| entry.inode);
                rewritten_node(
                    role,
                    &|birth_sequence| {
                        build_upper_leaf(&identity, *position, &entries, birth_sequence)
                    },
                    &mut units,
                )
            }
            ExtentTreeNodeOrigin::RewrittenThisPublish => {
                let children: Vec<(ExtentUpperNodePosition, NodePointer)> = upper_nodes
                    .iter()
                    .filter(|(child, _)| {
                        child.level + 1 == position.level && child.parent() == *position
                    })
                    .copied()
                    .collect();
                rewritten_node(
                    role,
                    &|birth_sequence| {
                        build_upper_internal_node(&identity, *position, &children, birth_sequence)
                    },
                    &mut units,
                )
            }
            ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                carried_node(
                    role_in_the_previous_version.expect("照抄的上段节点只出现在接着上一版的发布里"),
                    role,
                    &mut units,
                );
                previous
                    .and_then(|previous_version| {
                        previous_version.extent_tree.upper_pointer_of(*position)
                    })
                    .expect("照抄的上段节点在上一版里")
            }
        };
        upper_nodes.push((*position, pointer));
    }
    BuiltExtentTree {
        version: ExtentTreeVersion {
            upper_nodes,
            lower_nodes,
        },
        units,
        birth_sequences_of_rewritten_nodes,
    }
}

/// 这一版分配记录树装好的样子：每个节点这一版的指针（bump 次序）、这次重写的节点的单元（bump 次序），与它们各自的出生序号。
struct BuiltAllocationRecordTree {
    version: AllocationRecordTreeVersion,
    rewritten_units: Vec<PublishedUnit>,
    birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
}

/// 按计划装分配记录树这一版的全部节点（D8（核心索引结构） 已定项 14）：重写的节点按 bump 次序（按位置先叶后根）逐个装——叶装它罩的那一段里的全部记录
/// （按 key 升序），内部节点装它每个孩子「那一段的起点 key + 孩子这一版的指针」；头里的 key 区间是这个节点按位置规定罩的那一段。
/// 出生序号按同一个次序发，落点取这次分配的。照抄的节点只取指针（`previous_pointer_of`）：带文件的一版上它的字节由调用方从上一版照抄进 `units`，
/// 树表 0 条的一版上不写照抄的节点。`records` 是这次发布取完落点之后分配器里的全部记录。
///
/// # Panics
/// 重写的节点没拿到落点，或照抄的节点在上一版里没有指针：角色清单按同一份计划排，上一版的节点是这个进程记的或重建时读回来的。
#[allow(
    clippy::too_many_arguments,
    reason = "装一棵树要的：计划、几何、记录、上一版节点的指针、树号、身份字段、落点、发号器，各自独立"
)]
fn build_allocation_record_tree(
    plan: &AllocationRecordTreePlan,
    geometry: &AllocationRecordTreeGeometry,
    records: &[AllocationRecord],
    previous_pointer_of: &dyn Fn(AllocationRecordTreeNode) -> Option<NodePointer>,
    tree: TreeIdentifier,
    context: &MultiLevelTreeBuildContext<'_>,
    slot_of_rewritten_role: &dyn Fn(TransactionUnit) -> SlotNumber,
    sequences: &mut BirthSequenceAllocator,
) -> BuiltAllocationRecordTree {
    let records_by_leaf = records_of_each_leaf(geometry, records);
    let node_set = plan.node_set();
    let mut pointers: BTreeMap<AllocationRecordTreeNode, NodePointer> = BTreeMap::new();
    let mut rewritten_units = Vec::new();
    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
    // 迭代上界是这一版的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
    for (node, origin) in &plan.nodes {
        let role = role_of_allocation_record_tree_node(*node);
        let pointer = match origin {
            AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion => {
                previous_pointer_of(*node).expect("照抄的分配记录树节点在上一版里有指针")
            }
            AllocationRecordTreeNodeOrigin::RewrittenThisPublish => {
                let birth_sequence = sequences.next(tree, context.txg, context.instance);
                let contents = match geometry.level_of(*node) {
                    0 => AllocationRecordTreeNodeContents::Leaf(
                        records_by_leaf
                            .get(node)
                            .map(Vec::as_slice)
                            .expect("这一版的叶都有记录（规划按有记录的叶排）"),
                    ),
                    _ => AllocationRecordTreeNodeContents::Internal(
                        children_of(geometry, &node_set, *node)
                            .into_iter()
                            .map(|child| {
                                (
                                    child,
                                    *pointers
                                        .get(&child)
                                        .expect("孩子在 bump 次序里排在父节点前面、先装好了"),
                                )
                            })
                            .collect(),
                    ),
                };
                let bytes = build_allocation_record_tree_node(
                    geometry,
                    *node,
                    &contents,
                    tree,
                    context.txg,
                    context.filesystem_identifier,
                    context.instance,
                    birth_sequence,
                );
                let slot = slot_of_rewritten_role(role);
                birth_sequences_of_rewritten_nodes.insert(role, birth_sequence);
                let pointer = NodePointer {
                    head: PointerHead {
                        birth_tree: tree,
                        birth_txg: context.txg,
                    },
                    locations: location_entries(context.device_identities, slot, &bytes),
                    instance: context.instance,
                    birth_sequence,
                };
                rewritten_units.push(PublishedUnit {
                    slot,
                    identity: role,
                    bytes,
                });
                pointer
            }
        };
        pointers.insert(*node, pointer);
    }
    BuiltAllocationRecordTree {
        version: AllocationRecordTreeVersion {
            nodes: plan
                .nodes
                .iter()
                .map(|(node, _)| (*node, pointers[node]))
                .collect(),
        },
        rewritten_units,
        birth_sequences_of_rewritten_nodes,
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
    let inode_tree_identifier = checkpoint.trees.inode;
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

/// 照抄上一版的数据单元：数据指针与每个数据单元的字节都取上一版（extent 树的节点照抄在 `build_extent_tree` 里）。
fn carried_file_version_units(carried: &TransactionOutput) -> FileVersionUnits {
    FileVersionUnits {
        data_units: (0..carried.data_pointers.len())
            .map(|position| {
                carried
                    .unit(TransactionUnit::Data(DataUnitIndexInFile(
                        u64::try_from(position).expect("单元序号"),
                    )))
                    .bytes
                    .clone()
            })
            .collect(),
        data_pointers: carried.data_pointers.clone(),
        key_order_mismatches: 0,
    }
}

/// 准入之后的那一段：释放、分配、装单元，交回装好、还没落盘的字节（按持久顺序排好的 [`PublishWrites`]）与这一版
/// （写账是空的，落盘由调用方做：`persist_the_publish_or_freeze_it`）。失败时分配器由调用方退回，这里不管。
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "一次发布就是一件能单独验证的事：九个角色的装法与一条持久顺序，拆开只会把顺序藏进几个函数；\
              参数各是一样东西：写入口、分配器、计划、上一版、算好的树、要释放的、要隔离的、树号"
)]
fn publish_admitted<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    resolved: &ResolvedPublish,
    release: &[Placement],
    quarantine: &[CopyQuarantinedAfterReleaseChecksumMismatch],
    trees: FileVersionTreeIdentifiers,
) -> Result<(PublishWrites, TransactionOutput), PublishError> {
    let rewritten = &resolved.rewritten_roles;
    let txg = plan.txg;
    let instance = plan.instance;
    // 这次发布切成几条记录、每条点名哪几个角色、属于第几个事务（D23（journal 的角色与格式） 已定项 17：最后一个事务装不下一条
    // 记录时末条再跨记录）。
    let named_entry_capacity = pool.journal_record_named_entry_capacity();
    let roles_named_by_each_record =
        roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity);
    let transaction_offset_of_each_record =
        transaction_offset_of_each_record_of_the_publish(rewritten, named_entry_capacity);
    let records_in_this_publish =
        u64::try_from(roles_named_by_each_record.len()).expect("记录条数");
    // 一个数据单元一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）：第 k 个事务是第 `plan.transaction + k` 号，
    // 共享的提交内生块在最后一个事务的记录里点名 ⇒ 它们头里的写序取最后那个事务的号（数据单元各带自己那个事务的号）。
    let last_transaction_of_this_publish = plan.transaction
        + transaction_offset_of_each_record
            .last()
            .copied()
            .expect("一次发布至少一条记录");
    let write_order = WriteOrder {
        instance,
        transaction: last_transaction_of_this_publish,
    };
    let file_content_transactions = plan.file_content_transactions();
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let mut sequences = BirthSequenceAllocator::default();

    // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
    // 释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录
    // 留在已分配、不改成已释放（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案），核得上的那一份照常释放。
    for placement in release {
        let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
            .iter()
            .filter(|copy| copy.placement == *placement)
            .map(|copy| copy.device)
            .collect();
        allocator.release_leaving_the_record_allocated_on(
            *placement,
            txg,
            &devices_whose_copy_failed_the_checksum,
        );
    }
    // 第一个文件版本把树表 0 条那一版的分配记录树整棵换下了（上面释放的就是它的节点，`format_time_tree_table_to_release`）：
    // 分配器不再记着它，之后的发布按这一版自己的分配记录树走。
    if previous.is_none() {
        allocator.forget_the_allocation_record_tree_of_the_version_without_file();
    }

    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
    for identity in rewritten {
        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
        slots.insert(*identity, placement.slot);
    }
    // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，
    // 环里最旧有效根往前挪了就按谓词回收，记账行按回收之后的数写（与这条根一起说同一件事）。失败时整个分配器由调用方换回去。
    allocator.record_root_written_by_this_process(txg);
    let slot_of = |identity: TransactionUnit| slots[&identity];
    let device_identities: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let checkpoint = FileVersionCheckpoint {
        txg,
        write_order,
        trees,
        filesystem_identifier,
        device_identities: &device_identities,
    };

    // 文件内容角色：有新版本就按切分装每个数据单元，没有就照抄上一版的指针与字节。
    let carried_file = match &plan.file {
        Some(_) => None,
        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
    };
    let FileVersionUnits {
        data_units,
        data_pointers,
        key_order_mismatches,
    } = match (&plan.file, carried_file) {
        (Some(file), _) => build_file_version_units(
            &checkpoint,
            file,
            &file_content_transactions,
            &FileVersionSlots {
                data: file_content_transactions
                    .iter()
                    .map(|transaction| {
                        slot_of(TransactionUnit::Data(transaction.unit_index_in_file))
                    })
                    .collect(),
            },
        ),
        (None, Some(carried)) => carried_file_version_units(carried),
        (None, None) => {
            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
        }
    };
    let multi_level_tree_context = MultiLevelTreeBuildContext {
        txg,
        filesystem_identifier,
        instance,
        device_identities: &device_identities,
    };
    // t2 extent 树（D8（核心索引结构） 已定项 14 的两段）：写文件内容的发布重写下段整段与上段那一串，别的照抄。
    let extent_built = build_extent_tree(
        &resolved.extent_tree,
        previous,
        &data_pointers,
        trees.extent,
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );

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

    // 实例表链（码 3 打包记录类型 4）：重写时每一片的容器身份是 (0, 4, 片序号, 0)（第 0 片照 mkfs），归树 0、在树表之前发号
    // （mkfs 也是先实例表后树表）；多于一片时尾片先装、先发号（`build_instance_table_chain`）。
    let instance_table_rewrite: Vec<InstanceTablePageUnit> = match &plan.instance_table {
        InstanceTablePlan::Rewrite(rewrite) => build_instance_table_chain(
            &rewrite.rows,
            &slot_of,
            txg,
            write_order,
            filesystem_identifier,
            &device_identities,
            &mut sequences,
        ),
        InstanceTablePlan::Carry(_) => Vec::new(),
    };
    let instance_table_pointer = match &plan.instance_table {
        InstanceTablePlan::Carry(pointer) => *pointer,
        InstanceTablePlan::Rewrite(_) => {
            instance_table_rewrite
                .first()
                .expect("一条链至少一片：0 行也写出空的那一片")
                .pointer
        }
    };

    // t5 分配记录树（字节表五；D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，
    // 每盘各一条，按 (设备, 槽号) 升序分装进各自那片叶。重写哪几个节点在准入之前走到固定点定下来（`settle_the_allocation_record_tree`）：
    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配，只会让变了的更少）。
    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let rewritten_allocation_nodes: BTreeSet<AllocationRecordTreeNode> = resolved
        .allocation_record_tree
        .rewritten_nodes()
        .into_iter()
        .collect();
    let previous_allocation_records: &[AllocationRecord] = match previous {
        Some(previous_version) => &previous_version.allocation_records,
        None => &[],
    };
    assert!(
        nodes_whose_contents_changed(
            &allocation_geometry,
            previous_allocation_records,
            &allocation_records
        )
        .is_subset(&rewritten_allocation_nodes)
            && nodes_holding_records(&allocation_geometry, &allocation_records)
                == resolved.allocation_record_tree.node_set(),
        "准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树：两边走的是同一串释放与取落点"
    );
    let allocation_built = build_allocation_record_tree(
        &resolved.allocation_record_tree,
        &allocation_geometry,
        &allocation_records,
        &|node| {
            previous.and_then(|previous_version| {
                previous_version.allocation_record_tree.pointer_of(node)
            })
        },
        trees.allocation_records,
        &multi_level_tree_context,
        &slot_of,
        &mut sequences,
    );

    // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
    // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
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
    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    let accounting_entries = accounting_entries_of_this_publish(
        txg,
        trees.inode,
        &device_identities_of_the_accounting_rows,
        &|row| match row {
            PoolWideAccountingRow::InodeNumberWatermark => inode_number_watermark,
            // 这两行与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。
            PoolWideAccountingRow::PendingDeleteBytes => {
                crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0
            }
            PoolWideAccountingRow::CommittedReservationBytes => {
                crate::admission::COMMITTED_RESERVATION_OF_THE_FIRST_VERSION.0
            }
        },
        &|row, device| {
            let device_map = allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == device)
                .expect("行按分配器里的那几块盘展开");
            match row {
                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,
                // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
                PerDeviceAccountingRow::FreeBytes => device_map.free_slots() * SLOT_BYTES,
                PerDeviceAccountingRow::UnreclaimableBytes => {
                    crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0
                }
                // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
                PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,
                PerDeviceAccountingRow::FragmentationRuns => device_map.free_runs(),
                PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),
            }
        },
    );
    // 记账树按计划装（D8（核心索引结构） 已定项 11）：行装不下一个节点时从中间切，节点按先叶后根发出生序号、取这次的落点。
    let accounting_built = build_multi_level_tree(
        MultiLevelCodeTwoTree::Accounting,
        trees.accounting,
        &resolved.accounting_tree,
        previous,
        &accounting_entries
            .iter()
            .map(|entry| {
                (
                    CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING),
                    entry.to_bytes(),
                )
            })
            .collect(),
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );

    let node_pointer = |tree: TreeIdentifier,
                        identity: TransactionUnit,
                        unit: &[u8],
                        sequence: BirthSequence| NodePointer {
        head: PointerHead {
            birth_tree: tree,
            birth_txg: txg,
        },
        locations: pool.location_entries(slot_of(identity), unit),
        instance,
        birth_sequence: sequence,
    };
    // extent 树根（上段的根）的指针：装树时已经按这次的落点算好（照抄的取上一版）。
    let extent_pointer = *extent_built
        .version
        .upper_nodes
        .last()
        .map(|(_, pointer)| pointer)
        .expect("上段恒有第一个文件那片叶到根那一串");
    // inode 树根的指针：这次重写了就按这次的落点算，一片叶都没重写就取上一版树表里那一条。
    let inode_root_pointer = match (&inode_tree_units.rewritten_root, previous) {
        (Some(root), _) => node_pointer(
            trees.inode,
            TransactionUnit::InodeRoot,
            &root.bytes,
            root.birth_sequence,
        ),
        (None, Some(carried)) => carried.tree_root_pointer(trees.inode.0),
        (None, None) => {
            unreachable!("没有上一版的发布（第一个文件版本）恒要写 inode 记录 ⇒ 恒重写 inode 树")
        }
    };
    let allocation_pointer = allocation_built.version.root_pointer();
    let accounting_pointer = accounting_built.version.root_pointer();

    // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 的 extent 树每个节点、每片 inode 叶容器、inode 根、
    // 分配记录树每个节点、记账树的每个节点各一条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // 照抄的角色 key 照旧、位置照旧；重写的按这次的指针算。数据单元的码 1 key 带写序（事务号），
    // 一事务一单元 ⇒ 同一个文件的各个单元 key 不撞（D19（块指针的结构与宽度预算） 已定项 6 压在切分纪律上）。
    let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> =
        data_pointers
            .iter()
            .enumerate()
            .map(|(position, pointer)| {
                (
                    TransactionUnit::Data(DataUnitIndexInFile(
                        u64::try_from(position).expect("单元序号"),
                    )),
                    mapping_key_for_data(pointer.head, pointer.write_order),
                    pointer.locations,
                )
            })
            .collect();
    for (role, pointer) in extent_tree_roles_in_bump_order(&extent_built.version)
        .into_iter()
        .zip(
            extent_built
                .version
                .lower_nodes
                .iter()
                .map(|(_, pointer)| pointer)
                .chain(
                    extent_built
                        .version
                        .upper_nodes
                        .iter()
                        .map(|(_, pointer)| pointer),
                ),
        )
    {
        mapped_units_with_locations.push((
            role,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    for container in &inode_tree_units.leaf_containers {
        mapped_units_with_locations.push((
            TransactionUnit::InodeLeafContainer(container.index),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
            container.pointer.locations,
        ));
    }
    mapped_units_with_locations.push((
        TransactionUnit::InodeRoot,
        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
        inode_root_pointer.locations,
    ));
    for (node, pointer) in &allocation_built.version.nodes {
        mapped_units_with_locations.push((
            role_of_allocation_record_tree_node(*node),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    for (node, pointer) in accounting_built
        .version
        .shape
        .nodes()
        .iter()
        .zip(&accounting_built.version.pointers)
    {
        mapped_units_with_locations.push((
            MultiLevelCodeTwoTree::Accounting
                .role_of_node(node.position, &accounting_built.version.shape),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations
        .iter()
        .map(|(identity, key, _)| (*identity, key.clone()))
        .collect();
    assert_eq!(
        mapped_units, resolved.mapped_units,
        "中央映射树的形状按 resolve 现算的那一份映射 key 算过：这里装出来的 key 要逐项相等（出生序号按 bump 次序发，D19 已定项 9）"
    );
    let mapping_entries_by_key: BTreeMap<CodeTwoTreeKey, Vec<u8>> = mapped_units_with_locations
        .iter()
        .map(|(_, key, locations)| {
            (
                CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
                build_mapping_entry(key, *locations),
            )
        })
        .collect();
    let mapping_keys: Vec<Vec<u8>> = mapping_entries_by_key
        .keys()
        .map(|key| key.bytes().to_vec())
        .collect();
    let mapping_built = build_multi_level_tree(
        MultiLevelCodeTwoTree::CentralMapping,
        trees.central_mapping,
        &resolved.central_mapping_tree,
        previous,
        &mapping_entries_by_key,
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );
    let mapping_pointer = mapping_built.version.root_pointer();

    // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
    // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
    let table_entry =
        |kind: u16, tree: TreeIdentifier, root: NodePointer, head_identifier: u64| TreeTableEntry {
            kind,
            tree,
            root,
            birth_txg: plan.tree_birth_txg,
            head_identifier,
        };
    let tree_table_entries = vec![
        table_entry(
            TREE_KIND_EXTENT,
            trees.extent,
            extent_pointer,
            trees.inode.0,
        ),
        table_entry(
            TREE_KIND_INODE,
            trees.inode,
            inode_root_pointer,
            trees.inode.0,
        ),
        table_entry(
            TREE_KIND_ALLOCATION,
            trees.allocation_records,
            allocation_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_ACCOUNTING,
            trees.accounting,
            accounting_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_LIVELIST,
            trees.livelist,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_SPARSE_SIDE_TABLE,
            trees.sparse_side_table,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_DEADLIST,
            trees.deadlist,
            NodePointer::empty_root(),
            0,
        ),
    ];
    assert!(
        tree_table_entries
            .windows(2)
            .all(|pair| pair[0].tree < pair[1].tree),
        "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：八个号连号发、中央映射树不进树表，剩下七条仍按发号次序升序"
    );
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &trees.extent.0.to_le_bytes(),
        &trees.deadlist.0.to_le_bytes(),
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
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        TransactionUnit::TreeTable,
        &tree_table_unit,
        tree_table_sequence,
    );

    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（按 bump 次序：每个数据单元、extent 根、每片 inode 叶容器、
    // inode 根、分配记录树、记账树与中央映射树的每个节点（树内先叶后根）、树表，实例表链的各片在末尾）。
    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
        slot: slot_of(identity),
        identity,
        bytes,
    };
    let mut units: Vec<PublishedUnit> = Vec::new();
    for (position, data_unit) in data_units.iter().enumerate() {
        let identity = TransactionUnit::Data(DataUnitIndexInFile(
            u64::try_from(position).expect("单元序号"),
        ));
        units.push(match carried_file {
            None => rewritten_unit(identity, data_unit.clone()),
            Some(carried) => carried_unit(carried, identity),
        });
    }
    units.extend(extent_built.units.iter().cloned());
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
    // 分配记录树按 bump 次序：重写的是这次装的，照抄的从上一版同一个位置那个角色拷。
    for (node, _) in &allocation_built.version.nodes {
        let identity = role_of_allocation_record_tree_node(*node);
        units.push(
            match allocation_built
                .rewritten_units
                .iter()
                .find(|unit| unit.identity == identity)
            {
                Some(allocation_record_tree_node_unit) => allocation_record_tree_node_unit.clone(),
                None => carried_unit(
                    previous.expect("照抄的分配记录树节点只出现在接着上一版的发布里"),
                    identity,
                ),
            },
        );
    }
    units.extend(accounting_built.units.iter().cloned());
    units.extend(mapping_built.units.iter().cloned());
    units.push(rewritten_unit(
        TransactionUnit::TreeTable,
        tree_table_unit.clone(),
    ));
    // 实例表链上的各片按链上的次序（第 0 片在前）：重写的是这次装的，照抄的是上一版带着的那几片。
    match (&plan.instance_table, previous) {
        (InstanceTablePlan::Rewrite(_), _) => {
            for page in &instance_table_rewrite {
                units.push(rewritten_unit(page.role, page.bytes.clone()));
            }
        }
        (InstanceTablePlan::Carry(_), Some(previous_version)) => {
            units.extend(
                previous_version
                    .units
                    .iter()
                    .filter(|unit| unit.identity.is_a_page_of_the_instance_table())
                    .cloned(),
            );
        }
        (InstanceTablePlan::Carry(_), None) => {}
    }

    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
        match identity {
            TransactionUnit::Data(index) => data_key_tail(
                data_pointers
                    .get(usize::try_from(index.0).expect("单元序号"))
                    .expect("点名的数据单元是这次装的那几个之一")
                    .write_order,
            ),
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot => node_key_tail(
                instance,
                *extent_built
                    .birth_sequences_of_rewritten_nodes
                    .get(&identity)
                    .expect("点名的 extent 树节点是这次重写的"),
            ),
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
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree => node_key_tail(
                instance,
                *allocation_built
                    .birth_sequences_of_rewritten_nodes
                    .get(&identity)
                    .expect("点名的分配记录树节点是这次重写的"),
            ),
            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => node_key_tail(
                instance,
                accounting_built.birth_sequence_of_a_rewritten_node(identity),
            ),
            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                node_key_tail(
                    instance,
                    mapping_built.birth_sequence_of_a_rewritten_node(identity),
                )
            }
            TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                node_key_tail(
                    instance,
                    instance_table_rewrite
                        .iter()
                        .find(|page| page.role == identity)
                        .expect("点名实例表那一片的发布一定重写了整条链")
                        .pointer
                        .birth_sequence,
                )
            }
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
    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
        let unit = units
            .iter()
            .find(|unit| unit.identity == identity)
            .expect("点名的角色都是这次重写、装过单元的");
        NamedUnit {
            locations: pool.location_entries(unit.slot, &unit.bytes),
            unit_class: identity.unit_class(),
            birth_tree: identity.tree(&trees),
            birth_txg: txg,
            key_tail: key_tail_of(identity),
        }
    };
    // 这次发布的记录（D23（journal 的角色与格式） 已定项 7 / 已定项 17）：第 k 条的 jsn 计数器是 `plan.counter + k`、
    // 事务号是 `plan.transaction` 加它属于的那个事务的序号（前 N − 1 条一条一个事务；最后一个事务装不下一条记录时再跨几条，
    // 那几条共享它的事务号）。提交标记只在一个事务的最后一条记录上（已定项 7；I-8.8（前缀里的事务不被切开） ③），
    // 「本次发布末条」标志只在这次发布真正的最后一条上（已定项 17）——恢复按它认发布边界（已定项 14 第六条）。
    // 每条都带整次发布的新根段（已定项 15）。反向链：第一条接 `plan.back_chain`（上一条记录的头），之后每条接这次发布里前一条的头（已定项 8）。
    let mut written_records: Vec<WrittenJournalRecord> =
        Vec::with_capacity(roles_named_by_each_record.len());
    // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
    for (record_offset_in_this_publish, named_roles) in
        roles_named_by_each_record.iter().enumerate()
    {
        let offset = u64::try_from(record_offset_in_this_publish).expect("记录序号");
        let back_chain = match written_records.last() {
            None => plan.back_chain,
            Some(previous_record_of_this_publish) => {
                back_chain_of(&previous_record_of_this_publish.bytes)
            }
        };
        let transaction_offset = transaction_offset_of_each_record[record_offset_in_this_publish];
        let transaction_of_the_next_record = transaction_offset_of_each_record
            .get(record_offset_in_this_publish + 1)
            .copied();
        let is_the_last_record_of_its_transaction =
            transaction_of_the_next_record != Some(transaction_offset);
        let place_in_publish = if transaction_of_the_next_record.is_none() {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        };
        let record = JournalRecord {
            instance,
            counter: plan.counter + offset,
            checkpoint_txg: txg,
            transaction: plan.transaction + transaction_offset,
            is_commit: is_the_last_record_of_its_transaction,
            // 本次发布内序号依次是 1..N（D23（journal 的角色与格式） 已定项 4）：所选根那条记录读不出时，
            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(
                record_offset_in_this_publish,
            ),
            place_in_publish,
            back_chain,
            filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
            new_tree_table: tree_table_pointer,
            new_mapping_root: mapping_pointer,
            new_tree_identifier_watermark: plan.tree_identifier_watermark,
            new_rollback_floor: plan.rollback_floor,
            named: named_roles.iter().copied().map(named_unit_of).collect(),
        };
        let bytes = record.to_bytes();
        written_records.push(WrittenJournalRecord { record, bytes });
    }
    let last_counter_of_this_publish = plan.counter + (records_in_this_publish - 1);
    let root = RootRecord {
        filesystem_identifier: *filesystem_identifier,
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: mapping_pointer,
        // 带文件的一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8）：根记录这一项恒全零，
        // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换。
    // 这里只装、不落盘：调用方拿它落盘，中途失败时这次已记的写进失败账（增补 2 第 20b 行）、这次发布冻结等原样重发。
    let writes = PublishWrites {
        units: written_units.into_iter().cloned().collect(),
        records: written_records
            .iter()
            .map(|written_record| JournalRecordWrite {
                counter: written_record.record.counter,
                bytes: written_record.bytes.clone(),
            })
            .collect(),
        checkpoint_txg: txg,
        root_slot: root.to_slot(pool.root_slot_bytes()),
        // 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
        journal_tail: last_counter_of_this_publish,
        journal_instance: instance,
    };

    let WrittenJournalRecord {
        record,
        bytes: record_bytes,
    } = written_records
        .pop()
        .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
    let output = TransactionOutput {
        root,
        record,
        record_bytes,
        earlier_records_of_this_publish: written_records,
        units,
        rewritten: rewritten.to_vec(),
        data_pointers,
        mapping_keys,
        allocation_records,
        allocation_record_tree: allocation_built.version,
        extent_tree: extent_built.version,
        accounting_entries,
        accounting_tree: accounting_built.version,
        central_mapping_tree: mapping_built.version,
        tree_table_entries,
        tree_identifiers: trees,
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
        quarantined_after_release_checksum_mismatch: quarantine.to_vec(),
        key_order_mismatches,
        highest_transaction_number_in_this_instance: plan
            .highest_transaction_number_before_this_publish
            .max(last_transaction_of_this_publish),
        // 落盘之后由 `persist_the_publish_or_freeze_it` 换成这次真写出去的账。
        writes: WritesByStructureKind::NOTHING_WRITTEN,
    };
    Ok((writes, output))
}

/// 装一棵多层码 2 树时整次发布共用的身份字段。
struct MultiLevelTreeBuildContext<'build> {
    txg: CheckpointTxg,
    filesystem_identifier: &'build [u8; 16],
    instance: InstanceGeneration,
    /// 位置条目按设备身份升序（I-2.5）。
    device_identities: &'build [DeviceIdentity],
}

/// 一棵多层码 2 树这一版装好的样子。
struct BuiltMultiLevelTree {
    version: CodeTwoTreeVersion,
    /// 这一版的全部节点单元，按 bump 次序（先叶后根、同层按 key 升序），根在最末。
    units: Vec<PublishedUnit>,
    /// 这次重写的节点各自的出生序号（点名项的 key 尾段要它）。
    birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
}

impl BuiltMultiLevelTree {
    /// # Panics
    /// 这个角色这次没重写：点名项只点名这次重写的角色。
    fn birth_sequence_of_a_rewritten_node(&self, identity: TransactionUnit) -> BirthSequence {
        *self
            .birth_sequences_of_rewritten_nodes
            .get(&identity)
            .expect("点名的是这次重写的节点")
    }
}

/// 按计划装一棵多层码 2 树这一版的全部节点（D8（核心索引结构） 已定项 11）：重写的节点按 bump 次序逐个装——先叶后根、
/// 同层按 key 升序（D3（空间分配） 已定项 10 ⑤）、出生序号按同一个次序发（D19（块指针的结构与宽度预算） 已定项 9）、
/// 落点取这次分配的；叶装这一版那几把 key 的完整条目，内部节点装「分隔 key + 孩子这一版的指针」（孩子先装好，指针才写得出），
/// 头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），层级取计划里的位置。照抄的节点字节、落点、指针全取
/// 上一版那个位置上的，角色换成这一版的位置。
///
/// # Panics
/// 计划里叶的 key 与 `leaf_entry_bytes_by_key` 的 key 不是同一个集合（计划按 `resolve` 现算的 key 算，条目按这次装出来的，
/// 两边出生序号的发号次序不同步就对不上）；照抄的节点不在上一版里；重写的节点没拿到落点——三样都是发布路径自己的不变量。
#[allow(
    clippy::too_many_arguments,
    reason = "装一棵树要的八样：哪一棵、它的号、计划、上一版、条目、身份字段、落点、发号器，各自独立"
)]
fn build_multi_level_tree(
    tree: MultiLevelCodeTwoTree,
    tree_identifier: TreeIdentifier,
    plan: &CodeTwoTreePlan,
    previous: Option<&TransactionOutput>,
    leaf_entry_bytes_by_key: &BTreeMap<CodeTwoTreeKey, Vec<u8>>,
    context: &MultiLevelTreeBuildContext<'_>,
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> BuiltMultiLevelTree {
    let planned_keys: Vec<&CodeTwoTreeKey> = plan.shape.keys_in_order();
    assert!(
        planned_keys.len() == leaf_entry_bytes_by_key.len()
            && planned_keys
                .iter()
                .zip(leaf_entry_bytes_by_key.keys())
                .all(|(planned, built)| *planned == built),
        "{tree:?}：计划里叶的 key 与这次装出来的条目的 key 不是同一个集合"
    );
    let key_width = tree.key_field_widths().key_width_in_bytes();
    let key_ranges = crate::code_two_tree::key_ranges_in_bump_order(&plan.shape);
    let mut pointers_by_position: BTreeMap<CodeTwoTreeNodePosition, NodePointer> = BTreeMap::new();
    let mut pointers: Vec<NodePointer> = Vec::with_capacity(plan.shape.nodes().len());
    let mut units: Vec<PublishedUnit> = Vec::with_capacity(plan.shape.nodes().len());
    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
    // 迭代上界是这一版的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
    for ((node, origin), (smallest_key, largest_key)) in
        plan.shape.nodes().iter().zip(&plan.origins).zip(key_ranges)
    {
        let identity = tree.role_of_node(node.position, &plan.shape);
        let (pointer, unit) = match origin {
            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
                let previous_version = previous.expect("照抄的节点只出现在接着上一版的发布里");
                let previous_tree = previous_version.multi_level_tree(tree);
                let carried = previous_version
                    .unit(tree.role_of_node(*previous_position, &previous_tree.shape));
                (
                    previous_tree.pointer_of(*previous_position),
                    PublishedUnit {
                        slot: carried.slot,
                        identity,
                        bytes: carried.bytes.clone(),
                    },
                )
            }
            CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
                let (entry_width, entries): (usize, Vec<Vec<u8>>) = match &node.contents {
                    CodeTwoTreeNodeContents::Leaf { keys } => (
                        tree.leaf_entry_width_in_bytes(),
                        keys.iter()
                            .map(|key| leaf_entry_bytes_by_key[key].clone())
                            .collect(),
                    ),
                    CodeTwoTreeNodeContents::Internal { children } => (
                        internal_entry_width_in_bytes(key_width),
                        children
                            .iter()
                            .map(|child| {
                                build_internal_entry(
                                    child.separator_key.bytes(),
                                    pointers_by_position[&child.child],
                                )
                            })
                            .collect(),
                    ),
                };
                let birth_sequence = sequences.next(tree_identifier, context.txg, context.instance);
                let bytes = build_index_node(
                    tree_identifier,
                    node.position.level,
                    key_width,
                    &smallest_key,
                    &largest_key,
                    context.txg,
                    context.filesystem_identifier,
                    context.instance,
                    birth_sequence,
                    u16::try_from(entry_width).expect("条目宽 2 字节"),
                    &entries,
                );
                let slot = *slots
                    .get(&identity)
                    .expect("这次重写的节点都在角色清单里、取了落点");
                birth_sequences_of_rewritten_nodes.insert(identity, birth_sequence);
                (
                    NodePointer {
                        head: PointerHead {
                            birth_tree: tree_identifier,
                            birth_txg: context.txg,
                        },
                        locations: location_entries(context.device_identities, slot, &bytes),
                        instance: context.instance,
                        birth_sequence,
                    },
                    PublishedUnit {
                        slot,
                        identity,
                        bytes,
                    },
                )
            }
        };
        pointers_by_position.insert(node.position, pointer);
        pointers.push(pointer);
        units.push(unit);
    }
    BuiltMultiLevelTree {
        version: CodeTwoTreeVersion {
            shape: plan.shape.clone(),
            pointers,
        },
        units,
        birth_sequences_of_rewritten_nodes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::{FIRST_TRANSACTION_TXG, INODE_LEAF_RECORDS};

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
            trees: FileVersionTreeIdentifiers::issued_from_watermark(
                TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            )
            .expect("11 加八个号装得下")
            .0,
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
        let one_unit_transactions = split_sequential_write_into_one_unit_transactions(
            u64::try_from(content.len()).expect("内容长度"),
            FIRST_TRANSACTION_NUMBER,
        );
        let context = MultiLevelTreeBuildContext {
            txg,
            filesystem_identifier: &filesystem_identifier,
            instance,
            device_identities: &device_identities,
        };
        let extent_plan = plan_the_extent_tree_after_this_publish(None, Some(1));
        let extent_tree_of =
            |data_slot: u64, extent_root_slot: u64, sequences: &mut BirthSequenceAllocator| {
                let units = build_file_version_units(
                    &checkpoint,
                    &file,
                    &one_unit_transactions,
                    &FileVersionSlots {
                        data: vec![SlotNumber(data_slot)],
                    },
                );
                let mut extent_slots = BTreeMap::new();
                extent_slots.insert(TransactionUnit::ExtentRoot, SlotNumber(extent_root_slot));
                build_extent_tree(
                    &extent_plan,
                    None,
                    &units.data_pointers,
                    checkpoint.trees.extent,
                    &context,
                    &extent_slots,
                    sequences,
                )
            };
        let first = extent_tree_of(50176, 50240, &mut checkpoint_sequences);
        let second = extent_tree_of(50178, 50241, &mut checkpoint_sequences);
        assert_eq!(
            (
                first.birth_sequences_of_rewritten_nodes[&TransactionUnit::ExtentRoot],
                second.birth_sequences_of_rewritten_nodes[&TransactionUnit::ExtentRoot]
            ),
            (BirthSequence(0), BirthSequence(1)),
            "同一个 checkpoint 的第二个对象接着数：extent 树 0、1"
        );
        assert_eq!(
            parse_index_node(&second.units[0].bytes)
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
            TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
                level: 0,
                device: DeviceIdentity(1),
                index_in_device: 61,
            })
            .tag(),
            "t5@0.1.61",
            "分配记录树根之下的节点按位置写步号"
        );
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
        assert_eq!(
            TransactionUnit::Data(DataUnitIndexInFile::FIRST).placement(),
            PlacementRule::UserData
        );
        let (trees_issued_from_the_make_filesystem_watermark, _) =
            FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
                .expect("11 加八个号装得下");
        assert_eq!(
            TransactionUnit::TreeTable.tree(&trees_issued_from_the_make_filesystem_watermark),
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

    /// 一次发布的重写角色：`data_units` 个数据单元、extent 根、`leaf_containers` 片 inode 叶容器、inode 根与四个固定点单元，按 bump 次序。
    fn rewritten_roles(data_units: u64, leaf_containers: usize) -> Vec<TransactionUnit> {
        (0..data_units)
            .map(|index| TransactionUnit::Data(DataUnitIndexInFile(index)))
            .chain([TransactionUnit::ExtentRoot])
            .chain((0..leaf_containers).map(|position| {
                TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                    position,
                ))
            }))
            .chain([
                TransactionUnit::InodeRoot,
                TransactionUnit::AllocationTree,
                TransactionUnit::AccountingTree,
                TransactionUnit::MappingTree,
                TransactionUnit::TreeTable,
            ])
            .collect()
    }

    /// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）：最后一个事务要点名的项多于一条记录装得下的 67 项时，按 bump 次序
    /// 装满一条再开下一条，这几条都属于最后一个事务；前 N − 1 个数据单元各一条、各是自己那个事务。
    /// 两个数据单元 + extent 根 + 130 片叶容器 + 5 个角色 = 138 个：第一条只点名数据单元 0（事务 0），
    /// 最后一个事务要点名 137 项 ⇒ 67 + 67 + 3 三条（事务都是 1）。不写数据单元的 68 个 ⇒ 67 + 1 两条（事务都是 0）。
    /// 装得下的照旧：三个数据单元 ⇒ 三条三个事务，末条点名数据单元 2 与 6 个共享角色。
    #[test]
    fn the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction(
    ) {
        let spilling_with_data = rewritten_roles(2, 130);
        let records_with_data = roles_named_by_each_record_of_the_publish(
            &spilling_with_data,
            JournalRecordNamedEntryCapacity::FromTheRecordFormat,
        );
        assert_eq!(
            records_with_data.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![1, 67, 67, 3],
            "数据单元 0 一条；最后一个事务 137 项装满两条再开第三条"
        );
        assert_eq!(
            records_with_data[0],
            vec![TransactionUnit::Data(DataUnitIndexInFile(0))]
        );
        assert_eq!(
            records_with_data.concat(),
            spilling_with_data,
            "几条合起来按 bump 次序恰好点名每个重写角色一次"
        );
        assert_eq!(
            transaction_offset_of_each_record_of_the_publish(
                &spilling_with_data,
                JournalRecordNamedEntryCapacity::FromTheRecordFormat
            ),
            vec![0, 1, 1, 1],
            "跨出去的两条属于最后一个事务"
        );

        let spilling_without_data = rewritten_roles(0, 62);
        assert_eq!(spilling_without_data.len(), 68);
        let records_without_data = roles_named_by_each_record_of_the_publish(
            &spilling_without_data,
            JournalRecordNamedEntryCapacity::FromTheRecordFormat,
        );
        assert_eq!(
            records_without_data
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            vec![67, 1]
        );
        assert_eq!(
            transaction_offset_of_each_record_of_the_publish(
                &spilling_without_data,
                JournalRecordNamedEntryCapacity::FromTheRecordFormat
            ),
            vec![0, 0],
            "一个事务跨两条"
        );

        let fitting = rewritten_roles(3, 1);
        assert_eq!(
            roles_named_by_each_record_of_the_publish(
                &fitting,
                JournalRecordNamedEntryCapacity::FromTheRecordFormat
            )
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>(),
            vec![1, 1, 8]
        );
        assert_eq!(
            transaction_offset_of_each_record_of_the_publish(
                &fitting,
                JournalRecordNamedEntryCapacity::FromTheRecordFormat
            ),
            vec![0, 1, 2]
        );

        assert_eq!(
            roles_named_by_each_record_of_the_publish(
                &[],
                JournalRecordNamedEntryCapacity::FromTheRecordFormat
            ),
            vec![Vec::<TransactionUnit>::new()],
            "一个角色都不重写的发布照样一条记录"
        );
        assert_eq!(
            transaction_offset_of_each_record_of_the_publish(
                &[],
                JournalRecordNamedEntryCapacity::FromTheRecordFormat
            ),
            vec![0]
        );
    }
}
