//! 写池的句柄 `PoolWriter`：经它走的每一个提交步骤、系统配置槽写与回退，连同它带着的几样设置
//! （journal 记录装得下的点名项数、inode 号水位从哪来）。

use super::*;

/// 一条 journal 记录装几个点名项，只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入）。
/// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）在产品路径上要一次发布点名 65 项以上才走得到——树表 0 条的一版上写行要实例表长到
/// 64 片（两万三千多行），带文件的一版要五十几片 inode 叶容器；压小这个数就能用几个单元造出跨记录的发布。产品路径恒 `FromTheRecordFormat`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRecordNamedEntryCapacity {
    /// 记录格式装得下的：(4096 − 496) ÷ 56 = 64 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）。
    FromTheRecordFormat,
    /// 压到这么多项（1..=64）。
    CappedForTests { named_entries_per_record: usize },
}

impl JournalRecordNamedEntryCapacity {
    /// 一条记录装几个点名项。
    #[must_use]
    pub fn named_entries_per_record(self) -> usize {
        match self {
            JournalRecordNamedEntryCapacity::FromTheRecordFormat => {
                usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64")
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

/// 只供测试压回去的 inode 号水位：这次发布从这个号起发、记账里写它加这次新建的个数（`.claude/kb/checks-owed.md` C479：
/// 产品路径上号只从记账里的水位发、水位只增，「复用一个已发过的号」这条写路径造不出来；要它造出来，池级 checker 的 I-9.6
/// （水位大于两处最大号）才有对象可判）。至少是第一个 inode 号（0 是无效 inode 号，D8（核心索引结构） 已定项 6）；
/// 是不是真的「压回去」（小于上一版的水位）要对着上一版判，在发布路径读它的那一处断言（[`InodeNumberWatermarkSource::watermark_this_publish_starts_from`]）。
/// 只住内存，不落盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InodeNumberWatermarkPressedBackForTests(u64);

impl InodeNumberWatermarkPressedBackForTests {
    /// 小于第一个 inode 号（`FIRST_INODE_NUMBER`）交回 `None`。
    #[must_use]
    pub fn new(next_inode_number: u64) -> Option<Self> {
        (next_inode_number >= FIRST_INODE_NUMBER).then_some(Self(next_inode_number))
    }

    #[must_use]
    pub const fn next_inode_number(self) -> u64 {
        self.0
    }
}

/// 一次发布从哪个 inode 号水位起算（D5（快照 / 空间记账机制） 已定项 4 第 12 项「inode 号水位」）：产品路径恒
/// `FromThePreviousVersionsAccounting`；只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）换成压回去的那个号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InodeNumberWatermarkSource {
    /// 上一版记账树里真写下的那一行（[`TransactionOutput::inode_number_watermark`]）。
    FromThePreviousVersionsAccounting,
    PressedBackForTests(InodeNumberWatermarkPressedBackForTests),
}

impl InodeNumberWatermarkSource {
    /// 走的是哪一臂（五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn branch_name(self) -> &'static str {
        match self {
            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => {
                "inode_number_watermark=from_the_previous_versions_accounting"
            }
            InodeNumberWatermarkSource::PressedBackForTests(_) => {
                "inode_number_watermark=pressed_back_for_tests"
            }
        }
    }

    /// 这次发布从哪个号起发：产品路径就是上一版记账里的水位，开关装着时是压回去的那个号。
    ///
    /// # Panics
    /// 开关压的号不小于上一版的水位：那不是「压回一个已发过的号」，开关给错了。在发布路径读它的两处（建 inode 的起号、
    /// 记账里写的水位）都在任何落盘动作之前。
    #[must_use]
    pub fn watermark_this_publish_starts_from(
        self,
        inode_number_watermark_of_the_previous_version: u64,
    ) -> u64 {
        match self {
            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => {
                inode_number_watermark_of_the_previous_version
            }
            InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => {
                assert!(
                    pressed_back.next_inode_number()
                        < inode_number_watermark_of_the_previous_version,
                    "开关把 inode 号水位压到 {}，上一版的水位是 {inode_number_watermark_of_the_previous_version}：\
                     要小于它才是压回一个已发过的号",
                    pressed_back.next_inode_number()
                );
                pressed_back.next_inode_number()
            }
        }
    }

    /// 没有上一版的那次发布（新池新建文件）从哪个号起发：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
    /// 这一次没有上一版的水位可压，产品路径恒走 `FromThePreviousVersionsAccounting` 那一臂。
    ///
    /// # Panics
    /// 开关装着（`PressedBackForTests`）：这次发布不读上一版的水位，压的号用不上，装着它走到这里是开关给错了（C479 的开关只给有上一版的发布）。
    /// 在发布路径读它的那一处在任何落盘动作之前。
    #[must_use]
    pub fn watermark_of_a_publish_without_a_previous_version(self) -> u64 {
        match self {
            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => FIRST_INODE_NUMBER + 1,
            InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => panic!(
                "开关把 inode 号水位压到 {}，而这次发布没有上一版（新池新建文件，水位恒 {}）：\
                 开关只给有上一版的发布，装着它走到这里是开关给错了",
                pressed_back.next_inode_number(),
                FIRST_INODE_NUMBER + 1
            ),
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
    pub(super) writes_by_structure_kind: WritesByStructureKind,
    /// 落盘阶段中途失败的发布各自已记的写，一次失败一份，按失败的先后排（增补 2 第 20b 行）。成功发布的账是两次快照之差、只在成功路径上取，
    /// 失败那次落盘的写不属于任何一次成功发布 ⇒ 不交出去就与设备一层的合计对不上。另立一份、不并进任何一次成功发布的账。
    writes_of_failed_publishes: Vec<WritesByStructureKind>,
    /// 记账树与中央映射树的节点容量，只供测试的开关（`CodeTwoTreeNodeCapacities`）；新开的写入口按产品路径起步。
    pub(super) code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
    /// 一条 journal 记录装几个点名项，只供测试的开关（[`JournalRecordNamedEntryCapacity`]）；新开的写入口按记录格式起步。
    pub(super) journal_record_named_entry_capacity: JournalRecordNamedEntryCapacity,
    /// 墓碑树与谱系树的三个容量（一片墓碑容器、一片谱系容器装满几条，一个码 2 节点装满几条），只供测试的开关
    /// （D18（块里携带什么信息） 已定项 24 第 3 条：旋钮不落盘）；新开的写入口按格式起步。
    pub(super) container_tree_capacities: ContainerTreeCapacities,
    /// 这次发布从哪个 inode 号水位起算，只供测试的开关（[`InodeNumberWatermarkSource`]）；新开的写入口按上一版的记账起步。
    pub(super) inode_number_watermark_source: InodeNumberWatermarkSource,
    /// 被杀的用户可见单元走释放判定的哪一臂，只供测试的开关（[`SnapshotLifetimeJudgement`]）；新开的写入口照判定。
    pub(super) snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
    /// 这个写入口上读新槽残留的账（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R2：残留那一读出错记数、
    /// 不拒写，读到的字节不进新单元）。只记，发布照成；新开的写入口从 0 起。
    residue_reads: Cell<ResidueReadsOfTheWriter>,
    /// 这个写入口上读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」，
    /// [`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。只在内存里、不落盘；新开的写入口从 0 起。
    pub(super) old_data_unit_reads_back_with_one_copy_failing: Cell<u64>,
    /// 同一个计数里每一次读回的那个旧单元在上一版里的映射 key，按读的先后（一次一把；同一个单元读两次记两把）。会话拿它另发一批搬迁，
    /// 把这次发布之后仍被活快照钉着（换下来进 deadlist、映射条目留存）的那几个搬到新槽（用户 2026-11-04 定「另发一批搬迁修好」，
    /// [`relocations_of_old_data_units_kept_for_the_snapshots`]）。只在内存里；新开的写入口从空起。
    pub(super) mapping_keys_of_old_data_units_read_back_with_one_copy_failing:
        RefCell<Vec<Vec<u8>>>,
    /// 这个写入口上写系统配置槽之前算 F 生效值（[`Self::write_system_configuration_slot`]）读到只坏一份的那几组位置条目，按读的先后；
    /// 每次发布都走这一读，读的是最新那条根的实例表（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。写入口手里没有分配器：
    /// 有分配器的调用方接着把它们记进这次挂载别的读那一份账（[`Self::count_the_reads_before_the_system_configuration_writes`]）。只在内存里；新开的写入口从空起。
    locations_read_with_one_copy_failing_before_the_system_configuration_writes:
        RefCell<Vec<[LocationEntry; 2]>>,
    /// 这个写入口上最近一次分配记录树重写集的固定点走了几轮（[`PoolWriter::rounds_of_the_last_allocation_record_tree_settling`]）。只在内存里、不落盘；新开的写入口从 0 起。
    pub(super) rounds_of_the_last_allocation_record_tree_settling: Cell<usize>,
}

/// 一个写入口上读新槽残留的账（[`PoolWriter::residue_reads`]）：发了几次、读不出几次。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResidueReadsOfTheWriter {
    pub issued: u64,
    pub failed: u64,
}

impl ResidueReadsOfTheWriter {
    /// 一次都没读。
    pub const NONE: Self = ResidueReadsOfTheWriter {
        issued: 0,
        failed: 0,
    };
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
            container_tree_capacities: ContainerTreeCapacities::FROM_THE_FORMAT,
            inode_number_watermark_source:
                InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
            snapshot_lifetime_judgement: SnapshotLifetimeJudgement::FollowTheJudgement,
            residue_reads: Cell::new(ResidueReadsOfTheWriter::NONE),
            old_data_unit_reads_back_with_one_copy_failing: Cell::new(0),
            mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell::new(Vec::new()),
            locations_read_with_one_copy_failing_before_the_system_configuration_writes:
                RefCell::new(Vec::new()),
            rounds_of_the_last_allocation_record_tree_settling: Cell::new(0),
        }
    }
}

impl<Device: BlockDevice> PoolWriter<'_, Device> {
    /// 这个写入口上最近一次定下来的分配记录树重写集固定点走了几轮（[`settle_the_allocation_record_tree`]、
    /// [`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`]）；还没有定下来过是 0，轮数用完的那一次走错误成员，不记在这里。
    #[must_use]
    pub fn rounds_of_the_last_allocation_record_tree_settling(&self) -> usize {
        self.rounds_of_the_last_allocation_record_tree_settling
            .get()
    }

    /// 这个写入口上读新槽残留发了几次、读不出几次（判决 m4-partial-write-r1 第二节 R2「残留读出错记数、不拒写」）。
    #[must_use]
    pub fn residue_reads(&self) -> ResidueReadsOfTheWriter {
        self.residue_reads.get()
    }

    /// 记一次读新槽残留：`read_back` 是那一读读没读回字节。
    pub(super) fn note_a_residue_read(&self, read_back: bool) {
        let before = self.residue_reads.get();
        self.residue_reads.set(ResidueReadsOfTheWriter {
            issued: before.issued + 1,
            failed: before.failed + u64::from(!read_back),
        });
    }

    /// 这个写入口上读—改—写读回旧数据单元时只坏一份的次数（部分写、截断与变长的尾单元；D4（校验和位置） 已定项 4「读到只坏一份时计数」）。
    /// 读回的那个单元在同一次发布里整个重写到新槽、两份新写，旧落点照释放之前读盘核隔离（[`copies_failing_the_release_checksum_check`]）。
    #[must_use]
    pub fn old_data_unit_reads_back_with_one_copy_failing(&self) -> u64 {
        self.old_data_unit_reads_back_with_one_copy_failing.get()
    }

    /// 把写系统配置槽之前算 F 生效值读到只坏一份的那几组位置条目记进这次挂载别的读那一份账（[`PoolAllocator::count_the_other_unit_reads_with_one_copy_failing`]），
    /// 写入口上的清空。下一次用户改动发成之后另发的那一批按它们换落点，发布本身已发成的那次改动的那一批就算下一批。
    pub(crate) fn count_the_reads_before_the_system_configuration_writes(
        &self,
        allocator: &PoolAllocator,
    ) {
        allocator.count_the_other_unit_reads_with_one_copy_failing(
            self.locations_read_with_one_copy_failing_before_the_system_configuration_writes
                .take(),
        );
    }

    /// 同 [`Self::old_data_unit_reads_back_with_one_copy_failing`] 那几次读回的旧单元在上一版里的映射 key，按读的先后，一次一把。
    /// 有活快照时读回的旧单元换下来进 deadlist、不释放（判决 m4-partial-write-r1 R7），坏的那一份留在旧落点上；会话按这几把 key
    /// 另发一批搬迁（[`relocations_of_old_data_units_kept_for_the_snapshots`]）。
    #[must_use]
    pub fn mapping_keys_of_old_data_units_read_back_with_one_copy_failing(&self) -> Vec<Vec<u8>> {
        self.mapping_keys_of_old_data_units_read_back_with_one_copy_failing
            .borrow()
            .clone()
    }

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
    /// 压的项数是 0 或大于记录格式装得下的 64 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）：开关给错了。
    pub fn set_journal_record_named_entry_capacity(
        &mut self,
        capacity: JournalRecordNamedEntryCapacity,
    ) {
        let of_the_format = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64");
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

    /// 装上墓碑树与谱系树的容量这个只供测试的开关（[`ContainerTreeCapacities`]，三样的上下界在各自的构造函数里判过）。产品路径一处都不调它。
    pub fn set_container_tree_capacities(&mut self, capacities: ContainerTreeCapacities) {
        self.container_tree_capacities = capacities;
    }

    /// 这会儿装着的墓碑树与谱系树的容量：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn container_tree_capacities(&self) -> ContainerTreeCapacities {
        self.container_tree_capacities
    }

    /// 装上「这次发布从哪个 inode 号水位起算」这个只供测试的开关（[`InodeNumberWatermarkSource`]）。产品路径一处都不调它。
    /// 装着时这个写入口发的每一次有上一版的发布都从压回去的号起算（建 inode 从它起发号，记账里写它加这次新建的个数）；
    /// 没有上一版的那次（新池新建文件，水位恒 2）不读上一版的水位：开关装着走到那一次当场 panic
    /// （[`InodeNumberWatermarkSource::watermark_of_a_publish_without_a_previous_version`]），在任何落盘动作之前。
    pub fn set_inode_number_watermark_source(&mut self, source: InodeNumberWatermarkSource) {
        self.inode_number_watermark_source = source;
    }

    /// 这会儿装着的 inode 号水位来源：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn inode_number_watermark_source(&self) -> InodeNumberWatermarkSource {
        self.inode_number_watermark_source
    }

    /// 装上「被杀的用户可见单元走释放判定的哪一臂」这个只供测试的开关（[`SnapshotLifetimeJudgement`]，判决
    /// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 末句「只供测试的开关：强制立即释放 / 进 deadlist」）。产品路径一处都不调它。
    /// 装着 [`SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests`] 时，活头没打过快照而这次发布换下了用户可见单元，发布在任何写之前 panic
    /// （纪元 0 的条目哪个快照都放不掉，判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J4）。
    pub fn set_snapshot_lifetime_judgement(&mut self, judgement: SnapshotLifetimeJudgement) {
        self.snapshot_lifetime_judgement = judgement;
    }

    /// 这会儿装着的释放判定那一臂：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn snapshot_lifetime_judgement(&self) -> SnapshotLifetimeJudgement {
        self.snapshot_lifetime_judgement
    }

    pub fn perform_commit_step(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
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
                    self.write_system_configuration_slot(
                        index,
                        journal_tail,
                        journal_instance,
                        RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor,
                    )?;
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

    /// 离线出路（α'，`system_configuration_repair`）重写一块盘不自证的那一槽：同 [`Self::write_system_configuration_slot`]，F 带整池的生效值。
    /// 这块盘有一槽自证过时，世代号 = 它 + 1，落的正是另一槽（槽世代号与槽号的关系，D22（单元原子性怎么合成） 已定项 16）；两槽都不自证时
    /// 头一次写落槽 1（世代号 1），再调一次落槽 0（世代号 2）。
    pub(crate) fn write_system_configuration_slot_carrying_the_pool_effective_floor(
        &mut self,
        index: usize,
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    ) -> Result<(), BlockDeviceError> {
        self.write_system_configuration_slot(
            index,
            journal_tail,
            journal_instance,
            RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor,
        )
    }

    /// 一块盘写一次系统配置槽：世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2；
    /// 回退下界 F 那 8 字节按 `rollback_floor_to_write` 取（[`RollbackFloorOfASystemConfigurationWrite`]）。
    /// 算世代号、覆写哪一槽那一读是这一刻读盘（[`BlockDevice::read_at_straight_from_the_disk`]，[`PoolReaderStraightFromTheDisk`]）：
    /// 两槽轮换覆写的要是这一刻盘上较旧的那一槽，可写挂载里这次挂载记下的那一份不替它读（代码轮
    /// `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L1 ②：记下的那一份可以是一次读回的坏字节，拿它算会覆写这块盘最新的那一槽）。
    pub(super) fn write_system_configuration_slot(
        &mut self,
        index: usize,
        journal_tail: u64,
        journal_instance: InstanceGeneration,
        rollback_floor_to_write: RollbackFloorOfASystemConfigurationWrite,
    ) -> Result<(), BlockDeviceError> {
        let spacing = u64::from(self.parameters.geometry.fixed_structure_slot_spacing);
        let identity = self.devices[index].0;
        let verified_on_this_device = verified_system_configuration_slots(
            &PoolReaderStraightFromTheDisk::over(&*self.devices),
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
        // 这一写带的 F 在写之前按盘上现状现算整池的生效值（D16（发布语义） 已定项 1「生效」）：非抬 F 的写就带它，
        // 抬 F 先写系统配置那一步带新 F 与它的大者——F 的生效值只增不减（「抬 F 那一串」那一行）。
        // 读不出的根槽与最新那条根读不出的实例表各重读一次（实审 A3b 报告 Q7），仍读不出照原来的退路（当没有根、不按表滤），不拒：
        // 这个写入口手里没有分配器上那张根环表，分不出哪个槽是这个进程知道住着根的（D16（发布语义） 已定项 1「根槽这一次读坏」那一行
        // 要按它分两类），而这一写是发布末尾的轮换、取号或先写 F，报错的去处只有块设备错。退路不让 F 回落：生效值取系统配置里读得出的
        // F 与根上带的 F 的大者，要被覆写的那一槽在这里先读进来，根上的 F 不高于它那块盘系统配置里的（I-7.12（系统配置 F 不低于同盘根上的 F）），
        // 只剩「那块盘两槽系统配置与带最高 F 的根同时读不出」这一种叠加故障算得低（推的，没造过）。
        let devices_counting_unit_reads_with_one_copy_failing =
            PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*self.devices);
        let pool_effective_floor = effective_rollback_floor_rereading_unreadable_reads_once(
            &devices_counting_unit_reads_with_one_copy_failing,
            &self.parameters.region_devices,
            &self.parameters.geometry,
            &self.parameters.filesystem_identifier,
        );
        self.locations_read_with_one_copy_failing_before_the_system_configuration_writes
            .borrow_mut()
            .extend(
                devices_counting_unit_reads_with_one_copy_failing
                    .locations_of_the_units_read_with_one_copy_failing(),
            );
        let rollback_floor = match rollback_floor_to_write {
            RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor => pool_effective_floor,
            RollbackFloorOfASystemConfigurationWrite::RaisedFloor(raised_floor) => {
                raised_floor.max(pool_effective_floor)
            }
        };
        let system_configuration = SystemConfiguration {
            immutable: SystemImmutableConfiguration {
                filesystem_identifier: self.parameters.filesystem_identifier,
                this_device: identity,
                device_count: u32::try_from(self.devices.len()).expect("设备数"),
                region_devices: self.parameters.region_devices,
                sizes: self.parameters.geometry,
                journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation,
                journal_tail,
                journal_instance,
                rollback_floor,
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
    pub(super) fn count_failed_publish(
        &mut self,
        writes_before_this_publish: &WritesByStructureKind,
    ) {
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

    /// 取号全或无失败：已写出的那几块盘回卷成旧代号（D18（块里携带什么信息） 已定项 11），回卷写发生在任何单元之前。
    /// `written` 是写出过取号写的盘（盘表下标），每块盘只出现一次：取号写两轮（甲'）之后每块盘只回卷一次，回卷写覆写这块盘此刻较旧的那一槽，
    /// 另一槽留着新号（下一次取号跳过它）；同一块盘传两次会在一段里把两槽都覆写，同段断电两槽一起坏（第二轮判决 T1 打中的那一形）。
    /// 回卷写带的 tail 是取号那一写带的同一个见证值（`witnessed_journal_tail`，取号那一刻读的那一次）：回卷是「像没取过号」，
    /// 取号之前盘上的见证值是它，不退回 0（C554 乙-配置续，用户 2026-09-27 定）。
    pub(super) fn roll_back_acquisition(
        &mut self,
        written: &[usize],
        previous_instance: InstanceGeneration,
        witnessed_journal_tail: u64,
        cause: BlockDeviceError,
    ) -> InstanceAcquisitionFailed {
        if written.is_empty() {
            return AcquisitionFailed {
                cause,
                rollback: AcquisitionRollback::NothingWritten,
            }
            .into();
        }
        // 回卷写之前一道池屏障（代码审阅第 19 条）：回卷写覆写的是这块盘上取号之前最新的那一槽，另一槽是刚写的取号写、还可能没持久；
        // 两槽轮换要「覆写旧槽时新槽已持久」。这道屏障报错就不写回卷：回卷不成，只读挂载（D18（块里携带什么信息） 已定项 11），
        // 盘上可能留着新号，下一次取号跳过它。
        if let Err(barrier_error) = self.perform_commit_step(CommitStep::Barrier) {
            return AcquisitionFailed {
                cause,
                rollback: AcquisitionRollback::RollbackFailed(barrier_error),
            }
            .into();
        }
        for &index in written {
            if let Err(rollback_error) = self.write_system_configuration_slot(
                index,
                witnessed_journal_tail,
                previous_instance,
                RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor,
            ) {
                return AcquisitionFailed {
                    cause,
                    rollback: AcquisitionRollback::RollbackFailed(rollback_error),
                }
                .into();
            }
        }
        AcquisitionFailed {
            cause,
            rollback: AcquisitionRollback::RolledBack,
        }
        .into()
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

    pub(super) fn root_slot_bytes(&self) -> usize {
        usize::try_from(self.parameters.geometry.physical_block_size).expect("根槽宽")
    }
}

#[cfg(test)]
mod tests;
