//! 整个池的分配器 `PoolAllocator`：各盘的落点、提交内生块的回退、释放、隔离与照记录重建。

use super::*;

/// 树表 0 条那一版的分配记录树（它的根住根记录那一项，树号 0，节点豁免映射）：每个节点与它的指针，与这一版的全部记录
/// （按 (设备, 槽号) 升序）——下一次发布按记录判哪几片叶变了、按指针照抄或换下节点。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationRecordTreeOfTheVersionWithoutFile {
    pub version: crate::allocation_record_tree::AllocationRecordTreeVersion,
    pub records: Vec<AllocationRecord>,
}

/// 一块盘上的一个落点：被抛弃的根引用着的单元按盘记（两盘各一份，D2（RAID 条带策略） 已定项 10）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementOnDevice {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    pub span: u64,
}

impl PlacementOnDevice {
    fn covers(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {
        self.device == device && slot.0 >= self.slot.0 && slot.0 < self.slot.0 + self.span
    }
}

/// 这次可写挂载里建上一版与读—改—写读回之外的读读到只坏一份的账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），记账的读在
/// [`PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`] 的文档串里逐类列。读的是现行那一版的单元，下一次用户改动之后那一批换落点；
/// 读的是别的版本的单元（根环里别的根的账、回退目标、被抛弃根）照样计数，那一批在现行那一版里认不出它们，不搬。
/// 挂载这一串读到的那几组也在这份账上等下一次用户改动（[`PoolAllocator::keep_the_locations_read_with_one_copy_failing_uncounted`]），只是不进这个数：
/// 挂载读到的一份坏副本，数据单元与映射树叶一样，挂载那个数计一次，之后再读、或另发那一批释放之前读盘核，这个数都不加。
#[derive(Debug, Default)]
pub(super) struct OtherUnitReadsWithOneCopyFailing {
    /// 读到只坏一份的次数：一组位置条目等着下一次用户改动的这一程里只算一次。
    reads: u64,
    /// 读到只坏一份、还在等下一次用户改动处置的那几组位置条目（一个单元一组）。
    locations_waiting_for_the_next_user_change: Vec<[LocationEntry; 2]>,
}

/// 池级分配器：每块盘按同一条规则各自取，各盘取得一致才给号（盘可以不等大，D2（RAID 条带策略） 已定项 2）。
#[derive(Clone, Debug)]
pub struct PoolAllocator {
    pub devices: Vec<DeviceFreeMap>,
    /// 开放聚簇段的起点与 bump 游标（只在内存）。`None` = 这会儿没有段可 bump（还没开过，或上一次落点走了回落）。
    pub(super) open_segment: Option<SlotNumber>,
    pub(super) bump_cursor: u64,
    /// 这次挂载开过的聚簇段的起点，开过就一直算数（只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来）。
    /// 按已定项 10 ②「不在任何开放的聚簇段里」，用户数据的候选集排除它们全部、不只排除当前那一个——回落把 `open_segment` 置空之后
    /// 那一段仍装着这次提交的内生块，D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」照样压着它。
    /// 段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块。
    pub(super) cluster_segments: BTreeSet<SlotNumber>,
    pub(super) records: Vec<AllocationRecord>,
    /// 已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。
    pub(super) reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,
    /// mkfs 写出的第 0 版树表单元的落点：第一个文件版本重写树表时把它释放（COW 换下的单元进 defer 队列，D3（空间分配） 已定项 7）；
    /// 重开之后上一版从盘上重建、释放经映射与根记录走，这里留空。
    format_time_tree_table: Option<Placement>,
    /// 树表 0 条的那一版写行时写下的那棵分配记录树（根记录那一项指着它的根，C512（树表 0 条的一版上被换下的实例表记在哪没有条款））：
    /// 每个节点与它的指针，连同那一版的全部记录。这一版上再发一次时（再写一次行，或发第一个文件版本）要照抄或换下它的节点、
    /// 按它的记录判哪几片叶变了，而那一版没有上一版的内存态可查——与 `format_time_tree_table` 同一个用处。
    /// mkfs 的第 0 代与带文件的一版都留空。
    allocation_record_tree_of_the_version_without_file:
        Option<AllocationRecordTreeOfTheVersionWithoutFile>,
    /// 复用窗口，只供测试的开关（`ReuseWindow`）。只住内存：重开之后按产品路径起步（`rebuild_from_records` 走 `new`）。
    reuse_window: ReuseWindow,
    /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（两盘同槽的一个落点算一条，就是 `reclaim_released_up_to` 交回的条数）：
    /// 开关走没走到，读这个数就看得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    placements_reclaimed_on_release_by_the_forced_zero_reuse_window: u64,
    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBack`，D3（空间分配） 已定项 8 ②）的次数，
    /// 只增、只住内存、`new` 与 `rebuild_from_records` 起步是 0。只读，产品路径不按它分支：它是回落那一支的观测
    /// （`.claude/rules/fs-design.md` 五条硬要求第 4 条「分支必须可观测」）。harness 的看门用例在「mkfs 之后连续可写挂载、再写第一个文件」
    /// 那一族上断言它是 0：I-5.5 的开放段认法（池级 checker 拿树表单元所在的段当开放段）假定这一族不走回落，哪天走进回落，用例红、
    /// 回来重看那一格。
    commit_generated_placements_that_fell_back: u64,
    /// 分配器眼里的根环（`RootRingOccupancy`）：挂载时装（`mount::rebuilt_allocator`），mkfs 同一个进程里由建分配器的一方装
    /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`）。`None`：建分配器的一方没装（`new` 与 `rebuild_from_records`
    /// 都不装）。没装的分配器不许记根：`record_root_written_by_this_process` 断言（C551），只做不发布的事：分配，
    /// 以及调用方自己给根环每区槽数的准入读数 `admission::admission_reading_before_a_publish_on_a_ring_of`——
    /// `admission::admission_reading_before_a_publish` 从这张表取每区槽数，没装就断言。
    pub(super) root_ring: Option<RootRingOccupancy>,
    /// 落盘中途失败、冻结着等原样重发的那一次发布（`transaction::FrozenPublish`，D23（journal 的角色与格式） 已定项 14
    /// 「这一版的失败处置」）。住在分配器上是因为每一次发布都要交分配器进来：冻结着的时候发布路径在任何读写之前拒绝，
    /// 只有 `transaction::resend_the_frozen_publish` 清得掉它。只住内存：重开之后走恢复、换实例代号，冻结的那一次不带过去。
    frozen_publish: Option<Box<FrozenPublish>>,
    /// 这次挂载的 rows0（D28（挂载期承诺量） 已定项 3）：挂载时读到的实例表行数加写行那次要写的行数，挂载期承诺量里实例切换预留
    /// 那一项按它算（`admission::admission_reading_with_the_switch_reserve_of`：可写挂载自己判时按它，发布路径按它加一）。挂载期间常量、只住内存、每次挂载重算：
    /// 可写挂载在取号之前记上（`mount::establish_instance`），`new` 与 `rebuild_from_records` 起步是 0——mkfs 同一个进程里
    /// mkfs 的实例表一行都没有、取号 1 不写行（D18（块里携带什么信息） 已定项 11），rows0 本来就是 0。
    instance_rows_after_this_mounts_row_publish: u64,
    /// 判不判空间准入，只供测试的开关（`admission::SpaceAdmission`）。只住内存：`new` 与 `rebuild_from_records` 起步是判；
    /// 可写挂载按调用方给的装（`mount::mount_writable_with_space_admission`）。
    space_admission: SpaceAdmission,
    /// 删除那几次发布的两个只供测试的开关（`unlink::DeletionSwitches`）。只住内存：`new` 与 `rebuild_from_records` 起步是产品值；
    /// 可写挂载按调用方给的装（`mount::mount_writable_with_test_only_switches_and_deletion_switches`），挂载续做与会话照它。
    deletion_switches: DeletionSwitches,
    /// 整理在分配器这一侧的状态（`compaction_plan::CompactionSwitchesAndTheExcludedRegion`：只供测试的开关、活意图的 R 排他、
    /// 发布路径隔离过还没交给会话的槽）。R 排他就是分配器的一条合取（D26（后台整理与放置回收） 已定项 1 第 3 条）；不合格段表、常驻隔离集与观测
    /// 住会话（`compaction_plan::CompactionOfThisMount`）。`new` 与 `rebuild_from_records` 起步是产品值、没有活意图。
    pub(super) compaction: CompactionSwitchesAndTheExcludedRegion,
    /// 上一次记根时回收用的门槛与还钉着的槽里按（钉值, 槽）最小的那一槽（[`ReclaimFloorWhenTheRootWasRecorded`]，C773）。只住内存，`new` 与
    /// `rebuild_from_records` 起步是 `None`；随分配器整个换回发布之前（失败的发布）一起退回。
    reclaim_floor_when_the_latest_root_was_recorded: Option<ReclaimFloorWhenTheRootWasRecorded>,
    /// 别的读读到只坏一份的账（[`OtherUnitReadsWithOneCopyFailing`]）。一次挂载一份，各份拷贝共用：发布被拒时分配器换回发布之前那一份、
    /// 预演在拷贝上走，读是真发到盘上的，账不随换回退掉。`new` 与 `rebuild_from_records` 起步是空的。
    other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,
}

impl PoolAllocator {
    #[must_use]
    pub fn new(devices: Vec<DeviceFreeMap>) -> Self {
        Self {
            devices,
            open_segment: None,
            bump_cursor: 0,
            cluster_segments: BTreeSet::new(),
            records: Vec::new(),
            reclaimed: BTreeSet::new(),
            format_time_tree_table: None,
            allocation_record_tree_of_the_version_without_file: None,
            reuse_window: ReuseWindow::GatedByTheRollbackFloor,
            placements_reclaimed_on_release_by_the_forced_zero_reuse_window: 0,
            commit_generated_placements_that_fell_back: 0,
            root_ring: None,
            frozen_publish: None,
            instance_rows_after_this_mounts_row_publish: 0,
            space_admission: SpaceAdmission::JudgedByTheFormula,
            deletion_switches: DeletionSwitches::PRODUCT,
            compaction: CompactionSwitchesAndTheExcludedRegion::of_a_new_mount(),
            reclaim_floor_when_the_latest_root_was_recorded: None,
            other_unit_reads_with_one_copy_failing: Arc::default(),
        }
    }

    /// 别的读读到只坏一份的次数（[`OtherUnitReadsWithOneCopyFailing`]）。
    ///
    /// # Panics
    /// 账的锁中毒：持锁的几处只做加法与表操作，不 panic。
    #[must_use]
    pub fn other_unit_reads_with_one_copy_failing(&self) -> u64 {
        self.other_unit_reads_with_one_copy_failing
            .lock()
            .expect("账的锁没有中毒：持锁的几处只做加法与表操作")
            .reads
    }

    /// 经数只坏一份的那一层做一次读（`read`），读完（读成与否）把数到的那几组记进账（[`Self::count_the_other_unit_reads_with_one_copy_failing`]）。
    /// 走它的读：整理的回退门与后台入口读根环、准入算上限、取号之前取树表 0 条那一版的单元清单、会话改动入口的落后支、
    /// 挂着之后收盘表那一核的落后支（`mount::caller_inputs_agreeing_with_the_disk`：抬 F、准入抬 F、一次准入里再推一串、推环、正常卸载、管理员回退共用）、
    /// 读文件入口的落后支（`mounted_session::refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount`）、回退读现行那一版与目标那一版、回退的释放之前读盘核、清扫的判与发。
    /// 读完另经 [`Self::count_the_other_unit_reads_with_one_copy_failing`] 进账、不走这一层的读：抬 F 读实例表与重算影子账、发 discard 之前读实例表、
    /// 重建分配器读最新那条根的实例表与各根的账（`mount::rebuilt_allocator`）、可写挂载取号之前逐盘核的落后支（`mount::devices_passing_the_check_before_the_acquisition`）、释放之前读盘核、搬留存单元读源、另发那一批认位置沿链读实例表、经会话读文件、写系统配置槽之前算 F 生效值
    /// （经写入口转交，`transaction::PoolWriter::count_the_reads_before_the_system_configuration_writes`）。
    /// 建上一版那一读与读—改—写读回各有自己的数，不进这份账的次数。没接这份账的是可写挂载之外的读：只读挂载、冷恢复、离线修系统配置。
    /// 账各份分配器拷贝共用（读是真发到盘上的），所以 `&self` 够了。
    pub(crate) fn reading_and_counting_the_other_unit_reads_with_one_copy_failing<
        'reader,
        Reader: PoolReader + ?Sized,
        ReadOutput,
    >(
        &self,
        reader: &'reader Reader,
        read: impl FnOnce(&PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader>) -> ReadOutput,
    ) -> ReadOutput {
        let counting_reader =
            PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(reader);
        let read_output = read(&counting_reader);
        self.count_the_other_unit_reads_with_one_copy_failing(
            counting_reader.locations_of_the_units_read_with_one_copy_failing(),
        );
        read_output
    }

    /// 记下这一读读到只坏一份的那几组位置条目：已经在等下一次用户改动的不再加次数，别的加一次、接在后面。
    ///
    /// # Panics
    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）；次数越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
    pub(crate) fn count_the_other_unit_reads_with_one_copy_failing(
        &self,
        locations: Vec<[LocationEntry; 2]>,
    ) {
        let mut account = self
            .other_unit_reads_with_one_copy_failing
            .lock()
            .expect("账的锁没有中毒：持锁的几处只做加法与表操作");
        // 迭代上界是这一读报的组数；跨轮携带的只有账本身。
        for read_locations in locations {
            if !account
                .locations_waiting_for_the_next_user_change
                .contains(&read_locations)
            {
                account.reads = account
                    .reads
                    .checked_add(1)
                    .expect("一次挂载里只坏一份的读次数装得进 u64");
                account
                    .locations_waiting_for_the_next_user_change
                    .push(read_locations);
            }
        }
    }

    /// 把几组位置条目记成等着下一次用户改动，不加次数：建上一版读到的那几组（次数另有一个数，
    /// [`crate::compaction_plan::CompactionOfThisMount::unit_reads_with_one_copy_failing_while_mounting`]）。已经在等的不再记。
    ///
    /// # Panics
    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
    pub(crate) fn keep_the_locations_read_with_one_copy_failing_uncounted(
        &self,
        locations: &[[LocationEntry; 2]],
    ) {
        let mut account = self
            .other_unit_reads_with_one_copy_failing
            .lock()
            .expect("账的锁没有中毒：持锁的几处只做加法与表操作");
        // 迭代上界是交进来的组数；跨轮携带的只有账本身。
        for read_locations in locations {
            if !account
                .locations_waiting_for_the_next_user_change
                .contains(read_locations)
            {
                account
                    .locations_waiting_for_the_next_user_change
                    .push(*read_locations);
            }
        }
    }

    /// 等着下一次用户改动的那几组位置条目，按记下的先后。
    ///
    /// # Panics
    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
    #[must_use]
    pub(crate) fn locations_read_with_one_copy_failing_waiting_for_the_next_user_change(
        &self,
    ) -> Vec<[LocationEntry; 2]> {
        self.other_unit_reads_with_one_copy_failing
            .lock()
            .expect("账的锁没有中毒：持锁的几处只做加法与表操作")
            .locations_waiting_for_the_next_user_change
            .clone()
    }

    /// 那几组位置条目里去掉 `is_handled` 判真的那几组（另发的那一批试过一次）。次数不动。
    ///
    /// # Panics
    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
    pub(crate) fn forget_the_locations_read_with_one_copy_failing(
        &self,
        is_handled: impl Fn(&[LocationEntry; 2]) -> bool,
    ) {
        self.other_unit_reads_with_one_copy_failing
            .lock()
            .expect("账的锁没有中毒：持锁的几处只做加法与表操作")
            .locations_waiting_for_the_next_user_change
            .retain(|locations| !is_handled(locations));
    }

    /// 装上整理的只供测试的开关（`compaction_plan::CompactionSwitches`）。产品路径一处都不调它。
    pub fn set_compaction_switches(
        &mut self,
        switches: crate::compaction_plan::CompactionSwitches,
    ) {
        self.compaction.switches = switches;
    }

    /// 整理在分配器这一侧的状态（开关、活意图的 R 排他、发布路径隔离过还没交给会话的槽）。
    #[must_use]
    pub fn compaction(&self) -> &CompactionSwitchesAndTheExcludedRegion {
        &self.compaction
    }

    /// 同 [`Self::compaction`]，可改：会话签发、删、丢、暂停意图时装上与撤掉 R 排他，挂载按盘上的活意图装上，发布路径记隔离都经它。
    pub fn compaction_mut(&mut self) -> &mut CompactionSwitchesAndTheExcludedRegion {
        &mut self.compaction
    }

    /// 落点此刻要排除的 R（活整理意图且排他时；D3（空间分配） 已定项 7 登记表第 5 行）。
    #[must_use]
    pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
        self.compaction.region_excluded_from_placement()
    }

    /// 只供测试的目的地政策（`compaction_plan::RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests`）：每块盘取单元区里
    /// 起点最小、整跨度没挡、不在活 R 里的落点，各盘一致才分配，记一条分配代为 `generation` 的记录。
    ///
    /// # Errors
    /// 同 [`Self::try_allocate_commit_generated`]：拒绝时分配器一样都没动。
    pub fn try_allocate_at_the_lowest_unblocked_slot_for_tests(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        let excluded_region = self.region_excluded_from_placement();
        let slot_per_device: Vec<(DeviceIdentity, Option<SlotNumber>)> = self
            .devices
            .iter()
            .map(|device| {
                (
                    device.device,
                    device.lowest_commit_generated_fallback_slot_outside_the_region(
                        footprint,
                        excluded_region,
                    ),
                )
            })
            .collect();
        let slot = match agreement_across_devices(&slot_per_device) {
            DeviceAgreement::Agreed(slot) => slot,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(slot_per_device) => {
                return Err(
                    PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                        slot_per_device,
                    },
                );
            }
        };
        let placement = Placement {
            slot,
            span: footprint.slots(),
        };
        self.record(placement, generation);
        Ok(placement)
    }

    /// 发布返回之后每块盘要发 discard 的段起点（[`DeviceFreeMap::take_the_segments_to_discard`]）：按盘，每块盘自己的那几段。
    pub fn take_the_segments_to_discard_on_every_device(
        &mut self,
    ) -> Vec<(DeviceIdentity, Vec<SlotNumber>)> {
        self.devices
            .iter_mut()
            .map(|device_map| (device_map.device, device_map.take_the_segments_to_discard()))
            .collect()
    }

    /// 装上删除那几次发布的只供测试的开关（`unlink::DeletionSwitches`）。产品路径一处都不调它。
    pub fn set_deletion_switches(&mut self, deletion_switches: DeletionSwitches) {
        self.deletion_switches = deletion_switches;
    }

    /// 这会儿装着的删除开关：运行时看得出走的是哪一条分支（`UnlinkBatchSize::name`、`TombstoneContainerCapacity::name`）。
    #[must_use]
    pub fn deletion_switches(&self) -> DeletionSwitches {
        self.deletion_switches
    }

    /// 装上判不判空间准入这个只供测试的开关（`admission::SpaceAdmission`）。产品路径一处都不调它。
    pub fn set_space_admission(&mut self, space_admission: SpaceAdmission) {
        self.space_admission = space_admission;
    }

    /// 这会儿装着的空间准入开关：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn space_admission(&self) -> SpaceAdmission {
        self.space_admission
    }

    /// 记上这次挂载的 rows0（字段 `instance_rows_after_this_mounts_row_publish` 的注释）。可写挂载在取号之前调一次。
    pub fn record_instance_rows_after_this_mounts_row_publish(&mut self, rows: u64) {
        self.instance_rows_after_this_mounts_row_publish = rows;
    }

    /// 这次挂载的 rows0；没挂载过的（mkfs 同一个进程）是 0。
    #[must_use]
    pub fn instance_rows_after_this_mounts_row_publish(&self) -> u64 {
        self.instance_rows_after_this_mounts_row_publish
    }

    /// 冻结着的那一次发布；没有时 `None`。
    #[must_use]
    pub fn frozen_publish(&self) -> Option<&FrozenPublish> {
        self.frozen_publish.as_deref()
    }

    /// 把落盘中途失败的那一次发布冻结在这里（`transaction::persist_the_publish_or_freeze_it`）。
    ///
    /// # Panics
    /// 已经冻结着一次：发布路径的第一道就拒（`transaction::refuse_while_a_publish_is_frozen`），冻结着的时候走不到落盘。
    pub(crate) fn freeze_publish(&mut self, frozen: FrozenPublish) {
        assert!(
            self.frozen_publish.is_none(),
            "冻结着一次发布时发布路径的第一道就拒，走不到第二次落盘失败"
        );
        self.frozen_publish = Some(Box::new(frozen));
    }

    /// 取走冻结着的那一次发布（重发时）。
    pub(crate) fn take_frozen_publish(&mut self) -> Option<FrozenPublish> {
        self.frozen_publish.take().map(|frozen| *frozen)
    }

    /// 装上复用窗口这个只供测试的开关（`ReuseWindow`）。产品路径一处都不调它。
    pub fn set_reuse_window(&mut self, reuse_window: ReuseWindow) {
        self.reuse_window = reuse_window;
    }

    /// 这会儿装着的复用窗口：运行时看得出走的是哪一条分支。
    #[must_use]
    pub fn reuse_window(&self) -> ReuseWindow {
        self.reuse_window
    }

    /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数；开关没装上时恒 0。
    #[must_use]
    pub fn placements_reclaimed_on_release_by_the_forced_zero_reuse_window(&self) -> u64 {
        self.placements_reclaimed_on_release_by_the_forced_zero_reuse_window
    }

    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBack`）的次数：每取一个回落的落点加一。
    /// 可写挂载交回的分配器（`mount::Mounted::allocator`）带着挂载里写行、暖机、抬 F 与续做那几次发布数下的；重开之后从 0 数起。
    #[must_use]
    pub fn commit_generated_placements_that_fell_back(&self) -> u64 {
        self.commit_generated_placements_that_fell_back
    }

    /// 重开时从盘上那棵分配记录树重建分配器（D23（journal 的角色与格式） 已定项 14：defer 队列、分配器游标、记账的现行值从所选根那棵账重新载入）：
    /// 每条记录把自己那块盘上的槽标成占着，已释放的再进 defer 队列；空闲计数随 `mark_allocated` 减、与写路径同一条事件路径。
    /// 开放段与 bump 游标只在内存、盘上没有（分配记录树不记它们），重开后按「没有开放段」起步：下一个提交内生块从最低的全空段开——
    /// 上一段里没用完的槽先留着（C146（无空段时的回落政策全仓无定义） 之外的另一格，里程碑步 3 的决策点）。
    #[must_use]
    pub fn rebuild_from_records(
        devices: Vec<DeviceFreeMap>,
        records: Vec<AllocationRecord>,
    ) -> Self {
        let mut allocator = Self::new(devices);
        for record in &records {
            let device_map = allocator
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("分配记录的盘在池里：盘上读来的记录在 recovery::allocation_records_fit_the_pool_geometry 逐条比过 PoolReader::device_identities");
            let span = u64::from(record.span_slots);
            device_map.mark_allocated(record.slot, span);
            if record.is_released {
                device_map.mark_released(record.slot, span);
            }
        }
        allocator.records = records;
        // mkfs 那片第 0 版树表单元还没被换下（分配代 0、跨度 1、未释放；m1 实例表跨度 2，字节表一）时，重开之后第一个文件版本
        // 照样要把它释放——不然 mkfs 与第一个文件版本不在同一个进程里那一格永远漏一槽。
        allocator.format_time_tree_table = allocator
            .records
            .iter()
            .find(|record| {
                record.generation == CheckpointTxg(0)
                    && record.span_slots == 1
                    && !record.is_released
            })
            .map(|record| Placement {
                slot: record.slot,
                span: u64::from(record.span_slots),
            });
        allocator
    }

    /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：28 条里 m1 / m2 各两条，分配代 0）。
    /// 单元区之外的固定结构（系统配置、根环、journal 环）不写分配记录、分配器不下探（D3（空间分配） 已定项 10 ④）。
    pub fn mark_format_time_units(&mut self, instance_table: Placement, tree_table: Placement) {
        self.record(instance_table, CheckpointTxg(0));
        self.record(tree_table, CheckpointTxg(0));
        self.format_time_tree_table = Some(tree_table);
    }

    /// mkfs 写出的第 0 版树表单元（还没被第一个文件版本换下时 `Some`）。
    #[must_use]
    pub fn format_time_tree_table(&self) -> Option<Placement> {
        self.format_time_tree_table
    }

    /// 记下树表 0 条那一版的分配记录树（写行那次发布刚写出来的，或重开一个「树表 0 条、写过行」的池时从盘上整棵读回来的）：
    /// 下一次发布要照抄或换下它的节点，而这一版没有上一版的内存态可查。
    pub fn note_allocation_record_tree_of_the_version_without_file(
        &mut self,
        tree: AllocationRecordTreeOfTheVersionWithoutFile,
    ) {
        self.allocation_record_tree_of_the_version_without_file = Some(tree);
    }

    /// 树表 0 条那一版的分配记录树（这一版写过行、而它还没被第一个文件版本换下时 `Some`）。
    #[must_use]
    pub fn allocation_record_tree_of_the_version_without_file(
        &self,
    ) -> Option<&AllocationRecordTreeOfTheVersionWithoutFile> {
        self.allocation_record_tree_of_the_version_without_file
            .as_ref()
    }

    /// 第一个文件版本把树表 0 条那一版的分配记录树整棵换下之后（树号从 0 换成那一版发出来的号，节点一个都不照抄）清掉它。
    pub fn forget_the_allocation_record_tree_of_the_version_without_file(&mut self) {
        self.allocation_record_tree_of_the_version_without_file = None;
    }

    #[must_use]
    pub fn records(&self) -> &[AllocationRecord] {
        &self.records
    }

    /// 某块盘上某个槽的分配记录；没分配过就 `None`。释放判定路径拿它核「映射查出来的落点真的在册、跨度对得上」。
    #[must_use]
    pub fn record_for(
        &self,
        device: DeviceIdentity,
        slot: SlotNumber,
    ) -> Option<&AllocationRecord> {
        self.records
            .iter()
            .find(|record| record.device == device && record.slot == slot)
    }

    /// 记下一次分配：每块盘上一条落点 (设备, 槽, 跨度) 的记录，再在位图上标占。同一块盘上与 [槽, 槽 + 跨度) 相交的已有记录
    /// 先让位（`make_room_for_record_on_device`）：起点就是这个槽的已回收记录改写成这次的分配，别的已回收记录删掉。
    pub(super) fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
        for device in &self.devices {
            match make_room_for_record_on_device(
                &mut self.records,
                &mut self.reclaimed,
                device.device,
                placement,
                generation,
            ) {
                RecordAtThePlacementSlot::RewrittenFromReclaimed => {}
                RecordAtThePlacementSlot::Absent => {
                    self.records.push(AllocationRecord {
                        device: device.device,
                        slot: placement.slot,
                        span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                        generation,
                        is_released: false,
                        quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                    });
                }
            }
        }
        for device in &mut self.devices {
            device.mark_allocated(placement.slot, placement.span);
        }
    }

    /// 释放一个落点（D3（空间分配） 已定项 7：「释放」= 放进 defer 队列那一刻）：每盘那条分配记录改写成已释放 + 释放代，
    /// 条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。
    pub fn release(&mut self, placement: Placement, release_generation: CheckpointTxg) {
        self.release_leaving_the_record_allocated_on(placement, release_generation, &[]);
    }

    /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录留在「已分配」、分配代不变，
    /// 跨度段置上隔离记号：释放之前按位置项读盘核校验和、核出对不上（读不出也算）的那个单元（D19（块指针的结构与宽度预算） 已定项 5，
    /// 记号是同一项「隔离的盘上记号」）——那一份的槽不还回空闲池；记录与这次发布的别的改动
    /// 一起落盘，重挂之后从分配记录树重建的分配器照样认它已分配、带着记号，准入里照已分配算、不另进式子。
    /// 逐盘各算：不在 `devices_whose_record_stays_allocated` 里的盘上那一份照常释放；释放的记录带着隔离记号时记号随之清掉
    /// （「释放隔离记录时改写成已释放，0x4000 随之清掉」）。
    pub fn release_leaving_the_record_allocated_on(
        &mut self,
        placement: Placement,
        release_generation: CheckpointTxg,
        devices_whose_record_stays_allocated: &[DeviceIdentity],
    ) {
        for device in &mut self.devices {
            let record = self
                .records
                .iter_mut()
                .find(|record| record.device == device.device && record.slot == placement.slot)
                .expect("每块盘上都有这个落点的分配记录：盘上读来的两棵账对不对称由 transaction::placement_registered_unreleased_on_every_device 逐盘判过（panic 面普查 R10；经映射与第一个文件版本换下 mkfs 那几片两条路都走它）");
            assert!(
                !record.is_released,
                "同一个落点释放了两次：这块盘的记录是不是已释放由 transaction::placement_registered_unreleased_on_every_device 逐盘判过"
            );
            assert_eq!(
                u64::from(record.span_slots),
                placement.span,
                "释放的跨度与分配记录不符：这块盘的记录跨度由 transaction::placement_registered_unreleased_on_every_device 逐盘判过"
            );
            if devices_whose_record_stays_allocated.contains(&device.device) {
                record.quarantine_mark =
                    AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck;
                continue;
            }
            record.is_released = true;
            record.quarantine_mark = AllocationRecordQuarantineMark::NotQuarantined;
            record.generation = release_generation;
            device.mark_released(placement.slot, placement.span);
        }
        match self.reuse_window {
            ReuseWindow::GatedByTheRollbackFloor => {}
            // 窗口置 0：回收判定的下界取这次的释放代本身（这个落点自己就满足「释放代 ≤ 下界」），
            // 延迟窗口按 `Immediately` 绕开。走的是与产品路径同一个回收函数，只是下界不是 F。
            ReuseWindow::ForcedToZero => {
                let reclaimed_now =
                    self.reclaim_released_up_to(release_generation, ReclaimedReuse::Immediately);
                self.placements_reclaimed_on_release_by_the_forced_zero_reuse_window +=
                    u64::try_from(reclaimed_now.len()).expect("一次回收的落点数装得进 u64");
            }
        }
    }

    /// 管理员回退的复活（D23（journal 的角色与格式） 已定项 14「复活」）：`device` 上起点是这个落点的那条已释放记录改回已分配，
    /// 分配代写回 `allocation_generation`（回退目标那一版账里的分配代）。按盘改：回退逐盘判哪几块盘上的记录是已释放的。
    /// 那一槽还在 defer 窗口里的，从 defer 挪回仍分配；已回收、还没被复用（位图上空着、记录照旧写着已释放）的，重新在位图上标占。
    ///
    /// # Panics
    /// 这块盘上没有起点是这个槽的记录、它不是已释放的、或跨度对不上：调用方在任何写之前逐盘判过
    /// （`mount::roll_back_by_a_forward_publish`）。
    pub fn resurrect_released_record(
        &mut self,
        device: DeviceIdentity,
        placement: Placement,
        allocation_generation: CheckpointTxg,
    ) {
        let record = self
            .records
            .iter_mut()
            .find(|record| record.device == device && record.slot == placement.slot)
            .expect("复活的落点在这块盘上有记录：回退在任何写之前逐盘判过");
        assert!(
            record.is_released,
            "复活的是已释放的记录：回退在任何写之前逐盘判过"
        );
        assert_eq!(
            u64::from(record.span_slots),
            placement.span,
            "复活的跨度与记录相同：回退在任何写之前逐盘判过"
        );
        record.is_released = false;
        record.generation = allocation_generation;
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("复活的记录的盘在池里");
        if self.reclaimed.remove(&(device, placement.slot)) {
            device_map.mark_allocated(placement.slot, placement.span);
        } else {
            device_map.mark_resurrected_out_of_the_defer_queue(placement.slot, placement.span);
        }
    }

    /// 管理员回退那一次清掉隔离记号（D19（块指针的结构与宽度预算） 已定项 5「回退复活那一次，同一次发布清掉两块盘上的 0x4000」）：
    /// 回退目标引用的单元在 `device` 上那条记录仍分配、带着记号，改成不带记号；分配代与位图都不动（隔离的槽本来就占着）。
    ///
    /// # Panics
    /// 这块盘上没有起点是这个槽的记录、它已释放、不带记号或跨度对不上：调用方在任何写之前逐盘判过
    /// （`mount::roll_back_by_a_forward_publish`）。
    pub fn clear_the_quarantine_mark_of_a_record_the_rollback_target_references(
        &mut self,
        device: DeviceIdentity,
        placement: Placement,
    ) {
        let record = self
            .records
            .iter_mut()
            .find(|record| record.device == device && record.slot == placement.slot)
            .expect("清记号的落点在这块盘上有记录：回退在任何写之前逐盘判过");
        assert!(
            !record.is_released
                && record.quarantine_mark
                    == AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            "清记号的是仍分配、带着隔离记号的记录：回退在任何写之前逐盘判过"
        );
        assert_eq!(
            u64::from(record.span_slots),
            placement.span,
            "清记号的跨度与记录相同：回退在任何写之前逐盘判过"
        );
        record.quarantine_mark = AllocationRecordQuarantineMark::NotQuarantined;
    }

    /// 用户数据：见 `try_allocate_user_data`；拒绝的原因在这里丢掉，只给不看原因的调用方（测试里「一定分得到」的那几处）。
    /// 发布路径调 `try_allocate_user_data`，按原因报（`PublishError::PlacementRefused`）。
    pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
        self.try_allocate_user_data(generation).ok()
    }

    /// 用户数据：每块盘按政策函数各自取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取该设备内」），各盘一致才分配。
    ///
    /// # Errors
    /// 拒绝时分配器一样都没动：全部盘都没有落点、有的盘没有（盘不等大时小盘先满）、各盘的落点不同。
    pub fn try_allocate_user_data(
        &mut self,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        let cluster_segments = &self.cluster_segments;
        let excluded_region = self.compaction.region_excluded_from_placement();
        let slot_per_device: Vec<(DeviceIdentity, Option<SlotNumber>)> = self
            .devices
            .iter()
            .map(|device| {
                (
                    device.device,
                    device.lowest_user_data_slot_outside_the_region(
                        cluster_segments,
                        excluded_region,
                    ),
                )
            })
            .collect();
        let slot = match agreement_across_devices(&slot_per_device) {
            DeviceAgreement::Agreed(slot) => slot,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(slot_per_device) => {
                return Err(
                    PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                        slot_per_device,
                    },
                );
            }
        };
        let placement = Placement { slot, span: 2 };
        self.record(placement, generation);
        Ok(placement)
    }

    /// 提交内生块：见 `try_allocate_commit_generated`；拒绝的原因在这里丢掉，只给不看原因的调用方（测试里「一定分得到」的那几处）。
    /// 发布路径调 `try_allocate_commit_generated`，按原因报（`PublishError::PlacementRefused`）。
    pub fn allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Option<Placement> {
        self.try_allocate_commit_generated(footprint, generation)
            .ok()
    }

    /// 提交内生块：见 [`Self::try_allocate_commit_generated_preferring_the_rewrite_set`]，没有重写集可先取（回落直接取空槽最多的叶）。
    /// 不在发布路径上取落点的调用方走这里（新池新建文件的预演、测试）。
    ///
    /// # Errors
    /// 同 [`Self::try_allocate_commit_generated_preferring_the_rewrite_set`]。
    pub fn try_allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        self.try_allocate_commit_generated_preferring_the_rewrite_set(
            footprint,
            generation,
            &RewriteSetLeaves::none(),
        )
    }

    /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段没有或装不下时每块盘按自己的空闲图答一个去处——
    /// 最低的全空段（开新段），没有全空段就回落（D3（空间分配） 已定项 8 ②）。每块盘都答回落时池里取一个每块盘都空的回落落点
    /// （[`Self::commit_generated_fallback_slot_open_on_every_device`]：先 `rewrite_set_leaves` 里的叶，再空槽最多的叶，都排除 `R`），
    /// 每块盘的答案都换成它；有的盘答开段、有的盘答回落，或池里取不到每块盘都空的回落落点，照各盘自己的答案判一致——各盘答得一致才分配。
    ///
    /// # Errors
    /// 拒绝时分配器一样都没动（装不下的开放段也照旧开着）：全部盘都没有去处、有的盘没有、各盘的去处不同
    /// （每块盘都答回落而池里取不到每块盘都空的落点时报 [`PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice`]，
    /// 别的不同报段对齐那一条）。
    pub fn try_allocate_commit_generated_preferring_the_rewrite_set(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
        rewrite_set_leaves: &RewriteSetLeaves,
    ) -> Result<Placement, PlacementRefusal> {
        if let Some(open) = self.open_segment {
            if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {
                return Ok(self.record_bumped(slot, footprint, generation));
            }
        }
        let excluded_region = self.compaction.region_excluded_from_placement();
        let answer_per_device: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> = self
            .devices
            .iter()
            .map(|device| {
                (
                    device.device,
                    device.commit_generated_answer_without_open_segment_outside_the_region(
                        footprint,
                        excluded_region,
                    ),
                )
            })
            .collect();
        let every_device_falls_back = answer_per_device
            .iter()
            .all(|(_, answer)| matches!(answer, Some(CommitGeneratedDeviceAnswer::FallBack(_))));
        let answer_per_device = match every_device_falls_back
            .then(|| {
                self.commit_generated_fallback_slot_open_on_every_device(
                    footprint,
                    excluded_region,
                    rewrite_set_leaves,
                )
            })
            .flatten()
        {
            Some(fallback_slot_on_every_device) => answer_per_device
                .into_iter()
                .map(|(device, _)| {
                    (
                        device,
                        Some(CommitGeneratedDeviceAnswer::FallBack(
                            fallback_slot_on_every_device,
                        )),
                    )
                })
                .collect(),
            None => answer_per_device,
        };
        let answer = match agreement_across_devices(&answer_per_device) {
            DeviceAgreement::Agreed(answer) => answer,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(answer_per_device) => {
                return Err(refusal_of_commit_generated_answers_that_differ(
                    answer_per_device,
                ));
            }
        };
        match answer {
            CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start) => {
                self.open_segment = Some(segment_start);
                self.bump_cursor = segment_start.0;
                self.cluster_segments.insert(segment_start);
                let slot = self
                    .bump_slot_in_open_segment(segment_start, footprint)
                    .expect("每块盘都答了同一个全空段（已分配、隔离、扣住都是 0，整段在每块盘的单元区里）：64 槽装得下任何一个单元");
                Ok(self.record_bumped(slot, footprint, generation))
            }
            CommitGeneratedDeviceAnswer::FallBack(slot) => {
                // 回落之后没有段可 bump 了，但刚才那一段仍是聚簇段、仍装着这次提交的内生块：它留在 `cluster_segments` 里，
                // 用户数据照旧不许落进去（D3（空间分配） 已定项 8 第 2 条；增补 2 第 20c 行）。
                self.open_segment = None;
                self.commit_generated_placements_that_fell_back += 1;
                let placement = Placement {
                    slot,
                    span: footprint.slots(),
                };
                self.record(placement, generation);
                Ok(placement)
            }
        }
    }

    /// 每块盘都答回落之后池里取的回落落点（D3（空间分配） 已定项 8 第 2 条）：先 `rewrite_set_leaves` 里的叶（叶序号升序），取第一片叶里
    /// 每块盘都空的最低落点；那些叶里一个都没有，再取每块盘都空的单槽最多的那片叶（一样多取叶序号小的）里每块盘都空的最低落点——
    /// 那片叶里放不下这个跨度（两槽的单元要一对偶数起点的空槽）就让给下一片。「每块盘都空」= 整个跨度在每块盘的单元区里、
    /// 三种位都没置、不在活 `R` 里（[`DeviceFreeMap::is_whole_span_open_for_commit_generated`]）。一片叶罩的槽数是偶数、起点是偶数，
    /// 两槽的单元不跨叶。哪一步都取不到就 `None`。只读。
    fn commit_generated_fallback_slot_open_on_every_device(
        &self,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
        rewrite_set_leaves: &RewriteSetLeaves,
    ) -> Option<SlotNumber> {
        rewrite_set_leaves
            .leaf_indices
            .iter()
            .find_map(|leaf_index| {
                self.lowest_slot_open_on_every_device_in_the_leaf(
                    *leaf_index,
                    footprint,
                    excluded_region,
                )
            })
            .or_else(|| {
                self.lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(
                    footprint,
                    excluded_region,
                )
            })
    }

    /// 回落的后一步：每块盘都空的单槽最多的那片叶（一样多取叶序号小的）里每块盘都空的最低落点，放不下这个跨度的叶让给下一片。
    /// 每片叶先读每块盘增量维护的没挡槽数（[`DeviceFreeMap::unblocked_slots_in_allocation_record_tree_leaf`]），取各盘里最小的那个当上界：
    /// 每块盘都空的槽一定在每块盘上都没挡。叶序号升序走一遍，上界大于眼下最多的那片叶的数才逐槽数这片叶（数的是那片叶的槽，不扫单元区），
    /// 数出来更多、又放得下这个跨度才换成它；上界不大于眼下最多的，这片叶数出来也不会更多，一样多时叶序号小的那片已经在手里。
    fn lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(
        &self,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        let (first_leaf_index, last_leaf_index) = self.devices.iter().fold(
            None,
            |leaves_so_far: Option<(
                AllocationRecordTreeLeafIndex,
                AllocationRecordTreeLeafIndex,
            )>,
             device| {
                match (
                    leaves_so_far,
                    device.allocation_record_tree_leaves_of_the_unit_area(),
                ) {
                    (None, leaves_of_the_device) => leaves_of_the_device,
                    (Some(leaves), None) => Some(leaves),
                    (Some((first, last)), Some((first_of_the_device, last_of_the_device))) => {
                        Some((first.min(first_of_the_device), last.max(last_of_the_device)))
                    }
                }
            },
        )?;
        let mut most_open_slots_so_far = SlotCountInALeaf(0);
        let mut slot_in_the_leaf_with_the_most_open_slots: Option<SlotNumber> = None;
        // 迭代上界是单元区罩到的叶数；跨轮带的是眼下最多的那片叶的数与它里面的落点，没有提前出口。
        for leaf_number in first_leaf_index.0..=last_leaf_index.0 {
            let leaf_index = AllocationRecordTreeLeafIndex(leaf_number);
            let upper_bound_of_the_open_slots = self
                .devices
                .iter()
                .map(|device| device.unblocked_slots_in_allocation_record_tree_leaf(leaf_index))
                .min()
                .unwrap_or(SlotCountInALeaf(0));
            if upper_bound_of_the_open_slots <= most_open_slots_so_far {
                continue;
            }
            let open_slots =
                self.slots_open_on_every_device_in_the_leaf(leaf_index, excluded_region);
            if open_slots <= most_open_slots_so_far {
                continue;
            }
            if let Some(slot) = self.lowest_slot_open_on_every_device_in_the_leaf(
                leaf_index,
                footprint,
                excluded_region,
            ) {
                most_open_slots_so_far = open_slots;
                slot_in_the_leaf_with_the_most_open_slots = Some(slot);
            }
        }
        slot_in_the_leaf_with_the_most_open_slots
    }

    /// 分配记录树第 `leaf_index` 片叶里每块盘都空的单槽数（单元区外、活 `R` 里的不算）。
    fn slots_open_on_every_device_in_the_leaf(
        &self,
        leaf_index: AllocationRecordTreeLeafIndex,
        excluded_region: Option<CompactionRegion>,
    ) -> SlotCountInALeaf {
        let open_slots = (leaf_index.first_slot().0..leaf_index.past_the_last_slot().0)
            .map(SlotNumber)
            .filter(|slot| {
                self.devices.iter().all(|device| {
                    device.is_whole_span_open_for_commit_generated(
                        *slot,
                        UnitFootprint::OneSlot,
                        excluded_region,
                    )
                })
            })
            .count();
        SlotCountInALeaf(u64::try_from(open_slots).expect(
            "数的是一片叶里的槽，至多 ALLOCATION_RECORD_TREE_LEAF_SLOTS（812）个，装得进 u64",
        ))
    }

    /// 分配记录树第 `leaf_index` 片叶里起点最低、整个跨度每块盘都空的落点（起点按跨度的对齐档步进；叶起点是偶数）；没有就 `None`。
    fn lowest_slot_open_on_every_device_in_the_leaf(
        &self,
        leaf_index: AllocationRecordTreeLeafIndex,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        let past_the_last_slot_of_the_leaf = leaf_index.past_the_last_slot();
        let mut candidate = leaf_index.first_slot();
        // 迭代上界是一片叶的槽数 ÷ 对齐步长；没有跨轮携带的状态，找到就出。
        while candidate.0 + footprint.slots() <= past_the_last_slot_of_the_leaf.0 {
            if self.devices.iter().all(|device| {
                device.is_whole_span_open_for_commit_generated(
                    candidate,
                    footprint,
                    excluded_region,
                )
            }) {
                return Some(candidate);
            }
            candidate = SlotNumber(candidate.0 + footprint.alignment_in_slots());
        }
        None
    }

    /// 开放段里从游标起第一个整跨度在每块盘上都不被挡的对齐起点；段里装不下就 `None`。只读，不挪游标。
    fn bump_slot_in_open_segment(
        &self,
        open: SlotNumber,
        footprint: UnitFootprint,
    ) -> Option<SlotNumber> {
        let segment_end = open.0 + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        // 游标前方的槽在开段之后可能被隔离或扣住（抬 F 重算影子账、扣住回收的槽都发生在活分配器上）：跨度里有一个被挡就整个往后挪，
        // 挪的步长按对齐档。
        while start + footprint.slots() <= segment_end
            && (start..start + footprint.slots()).any(|slot| {
                self.devices
                    .iter()
                    .any(|device| device.is_blocked_for_commit_generated(SlotNumber(slot)))
            })
        {
            start += footprint.alignment_in_slots();
        }
        if start + footprint.slots() > segment_end {
            return None;
        }
        Some(SlotNumber(start))
    }

    /// 记下开放段里 bump 出的一个落点，游标挪到它后面。
    fn record_bumped(
        &mut self,
        slot: SlotNumber,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Placement {
        let placement = Placement {
            slot,
            span: footprint.slots(),
        };
        self.bump_cursor = slot.0 + footprint.slots();
        self.record(placement, generation);
        placement
    }

    /// 回收：释放代 ≤ `floor` 的已释放落点回到空闲，等着被再分配（D16（发布语义） 已定项 1：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；
    /// `floor` 由调用方按这个 max 算好：挂载时从盘上的根环算，挂载之内由 `record_root_written_by_this_process` 按分配器那张根环表算）。
    /// 回收过的不重复回收；返回这次回收的落点（两盘同槽，按盘 0 报）。
    /// `reuse` 说回收的槽什么时候能发：重建时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘（`release_reclaim_holds`）。
    pub fn reclaim_released_up_to(
        &mut self,
        floor: CheckpointTxg,
        reuse: ReclaimedReuse,
    ) -> Vec<Placement> {
        let first_device = self.devices[0].device;
        self.reclaim_released_records_up_to(floor, reuse)
            .into_iter()
            .filter(|record| record.device == first_device)
            .map(|record| Placement {
                slot: record.slot,
                span: u64::from(record.span_slots),
            })
            .collect()
    }

    /// 同 [`Self::reclaim_released_up_to`]，回收出来的槽里隔离还护着的（环里还在的被抛弃根引用的、还没解开的水位之上没施加的记录点名的，
    /// [`RootRingOccupancy::placements_the_isolation_still_protects`]）当场补隔离，与 [`Self::record_root_written_by_this_process`] 里回收之后那一步
    /// 同一件事：C 那一圈的落点挂载那一刻分配位置着的不隔离（[`DeviceFreeMap::isolate_the_slots_not_allocated`]，D28（挂载期承诺量） 已定项 1
    /// 第九项），抬 F 那一串回收它们时在这里补上。没装根环表时没有要护的，只回收。
    pub fn reclaim_released_up_to_isolating_what_the_isolation_still_protects(
        &mut self,
        floor: CheckpointTxg,
        reuse: ReclaimedReuse,
    ) -> Vec<Placement> {
        let first_device = self.devices[0].device;
        let reclaimed = self.reclaim_released_records_up_to(floor, reuse);
        let still_protected_by_the_isolation = self
            .root_ring
            .as_ref()
            .map(RootRingOccupancy::placements_the_isolation_still_protects)
            .unwrap_or_default();
        self.isolate_reclaimed_slots_the_isolation_still_protects(
            &reclaimed,
            &still_protected_by_the_isolation,
        );
        reclaimed
            .into_iter()
            .filter(|record| record.device == first_device)
            .map(|record| Placement {
                slot: record.slot,
                span: u64::from(record.span_slots),
            })
            .collect()
    }

    /// `reclaim_released_up_to` 的本体，交回这次回收的每一条记录（逐盘各一条）：挂载内回收要按盘判哪几个槽还被被抛弃的根引用着。
    fn reclaim_released_records_up_to(
        &mut self,
        floor: CheckpointTxg,
        reuse: ReclaimedReuse,
    ) -> Vec<AllocationRecord> {
        let mut reclaimed_now = Vec::new();
        let candidates: Vec<AllocationRecord> = self
            .records
            .iter()
            .filter(|record| {
                singlefs_replay_prefix::gc_crash_proofs::released_record_can_be_reclaimed(
                    record.is_released,
                    record.generation.0,
                    floor.0,
                )
            })
            .copied()
            .collect();
        for record in candidates {
            let key = (record.device, record.slot);
            if !self.reclaimed.insert(key) {
                continue;
            }
            let device_map = self
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("分配记录的盘在池里");
            device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
            match reuse {
                ReclaimedReuse::Immediately => {}
                ReclaimedReuse::HeldUntilFloorTakesEffect => {
                    device_map
                        .hold_until_floor_takes_effect(record.slot, u64::from(record.span_slots));
                }
            }
            reclaimed_now.push(record);
        }
        reclaimed_now
    }

    /// 抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。
    pub fn release_reclaim_holds(&mut self) {
        for device_map in &mut self.devices {
            device_map.release_holds();
        }
    }

    /// 水位之上没施加的 journal 记录点名、不被现行账引用的落点（C694 的 C 那一圈）在它那块盘上隔离，只隔离分配位没置着的槽
    /// （[`DeviceFreeMap::isolate_the_slots_not_allocated`]，D28（挂载期承诺量） 已定项 1 第九项「只除去现行账里仍分配着的槽」）。
    pub fn isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
        &mut self,
        device: DeviceIdentity,
        slot: SlotNumber,
        span: u64,
    ) {
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("C 那一圈的落点的盘在池里：盘上读来的那几份逐条经 recovery::placement_lies_in_the_unit_area_of_its_device 判过这块盘在池里，现行账里的记录在 recovery::allocation_records_fit_the_pool_geometry 比过");
        device_map.isolate_the_slots_not_allocated(slot, span);
    }

    /// 回退的影子账：把一个只被被抛弃根引用的落点，或水位之上没施加的 journal 记录点名、不被候选根与现行账引用的落点（C694 的 C），
    /// 在它那块盘上隔离。
    pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("被抛弃根的分配记录的盘在池里：盘上读来的记录在 recovery::allocation_records_fit_the_pool_geometry 逐条比过 PoolReader::device_identities");
        device_map.isolate(slot, span);
    }

    /// 同 [`Self::isolate_abandoned`]，只隔离落点里分配位没置着的槽（[`DeviceFreeMap::isolate_the_slots_not_allocated`]）：被抛弃根引用、
    /// 现行账里已释放而还在 defer 窗口里的槽分配位置着，算在「已分配」里，D28（挂载期承诺量） 已定项 1 第九项不在这里再算一次
    /// （与 C 那一圈同一个做法）。它们回收的那一刻由根环表照这条根记着的落点补隔离
    /// （[`RootRingOccupancy::placements_the_isolation_still_protects`] 里被抛弃根那一份，挂载内记根与抬 F 那一串回收两条路都走它）。
    /// 只给落点记在根环表里的被抛弃根用：可写挂载那一遍读出的（落点随后记进表里）、抬 F 那一串取表里记着的或借 C 那一圈的；
    /// 抬 F 那一串读盘得来、表里没记着的那几条照旧 [`Self::isolate_abandoned`]（回收时没有一份补得上它们，`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
    pub fn isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
        &mut self,
        device: DeviceIdentity,
        slot: SlotNumber,
        span: u64,
    ) {
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("被抛弃根的分配记录的盘在池里：盘上读来的记录在 recovery::allocation_records_fit_the_pool_geometry 逐条比过 PoolReader::device_identities，根环表里记着的是同一份");
        device_map.isolate_the_slots_not_allocated(slot, span);
    }

    /// `placements` 罩着的槽里这一刻隔离着的槽数，每块盘一个数、按分配器里盘的次序；同一个槽被几个落点罩着只数一次。
    /// 只给会话报「被宽护卡住」时数 C 那一圈挡着几个槽读（`mounted_session`），分配与准入的判定不读它。
    #[must_use]
    pub fn isolated_slots_covered_by(
        &self,
        placements: &[PlacementOnDevice],
    ) -> Vec<(DeviceIdentity, u64)> {
        self.devices
            .iter()
            .map(|device_map| {
                let isolated_covered: BTreeSet<u64> = placements
                    .iter()
                    .filter(|placement| placement.device == device_map.device)
                    .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
                    .filter(|slot| device_map.is_isolated(SlotNumber(*slot)))
                    .collect();
                (
                    device_map.device,
                    u64::try_from(isolated_covered.len()).expect("单元区槽数装得进 u64"),
                )
            })
            .collect()
    }

    /// 装上根环表（`RootRingOccupancy`：挂载时从盘上读出来的，或 mkfs 刚写下的），替掉原来那一张。
    pub fn install_root_ring_occupancy(&mut self, occupancy: RootRingOccupancy) {
        self.root_ring = Some(occupancy);
    }

    /// 换上进程内重读挂载时读不出的槽时判「按实例表仍然有效」的那张表（[`RootRingOccupancy::instance_table_judging_the_rereads`]）。
    /// 没装根环时什么都不做。
    pub fn note_the_instance_table_judging_the_rereads_of_root_ring_slots(
        &mut self,
        table: InstanceTableRecords,
    ) {
        if let Some(occupancy) = self.root_ring.as_mut() {
            occupancy.instance_table_judging_the_rereads = table;
        }
    }

    /// 记下可写挂载那一刻水位之上没施加的 journal 记录点名的落点（按记录的 txg 对应的根环槽归组，C694 的 C）：
    /// 隔离位由调用方在同一次影子账里置好（`mount::isolate_slots_referenced_only_by_abandoned_roots`），这里只记着什么时候解开
    /// （[`Self::record_root_written_by_this_process`] 写掉那一槽时）、回收出来的槽要不要补隔离。挂载装上根环表之后、第一次记根之前调一次。
    ///
    /// # Panics
    /// 没装根环表：可写挂载在重建分配器那一步先装上根环表再调它（`mount::rebuilt_allocator`），不装就没有解开的时机。
    pub fn note_placements_named_by_unapplied_journal_records(
        &mut self,
        named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
    ) {
        self.root_ring
            .as_mut()
            .expect("记水位之上没施加的记录点名的落点之前装了根环表：mount::rebuilt_allocator 先装根环表再记它")
            .placements_named_by_unapplied_journal_records = named_by_ring_slot;
    }

    /// 这会儿装着的根环；`None` 见 `root_ring` 字段的注释。
    #[must_use]
    pub fn root_ring_occupancy(&self) -> Option<&RootRingOccupancy> {
        self.root_ring.as_ref()
    }

    /// 这个进程写了一条根（txg `checkpoint_txg`，落在根环公式给的那个槽上；这个进程写的根按实例表恒有效）：它盖掉那个槽上原来的根。
    ///
    /// - 盖掉的是一条被抛弃的根：它引用的槽里，环里别的被抛弃根都不引用的那些清掉隔离位（D28（挂载期承诺量） 已定项 1 第九项
    ///   「被抛弃的根被轮转覆写时清零」，C503（隔离位清零的时机条文与实现说反话）改代码）。
    /// - 盖掉之后环里最旧有效根往前挪了：按可再分配谓词回收（D16（发布语义） 已定项 1：已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)，
    ///   C518（一次挂载之内环转过一圈之后不回收））。回收出来的槽里仍被环里某条被抛弃根引用着的，当场补隔离——候选根离开根环之后
    ///   它们就只被被抛弃根引用了，D23（journal 的角色与格式） 已定项 14 的主句不许它们在那条被抛弃根离开根环之前重新分配；
    ///   挂载时与抬 F 时按当时的候选集算影子账，是同一件事在另外两个时刻。
    ///
    /// **调用时机是调用方的前置条件**：这次发布的落点全部取完、之后不再分配（带单元的发布在装记账行之前调，记账行因此已经按回收之后的数写；
    /// 零单元发布在根落盘之后调）；这次发布失败时调用方把分配器整个换回发布之前（`transaction::publish_version` 与
    /// `transaction::publish_instance_table_on_version_without_file` 都这么做），这张表随之退回，盘上那个槽照旧按旧内容算
    /// （D16（发布语义） 已定项 1「写失败的槽按旧内容算」）。
    ///
    /// # Panics
    /// - 分配器没装根环表（`install_root_ring_occupancy`）：不装就一次不回收，转过根环之后已分配统计越积越多、I-3.1（已分配统计对得上）红
    ///   （C551（`build_pool` 的分配器没有根环表）：core 在这里断言，漏装的装置当场 panic，不再静默不回收）。
    ///   出路是建分配器的一方装上根环表：mkfs 同一个进程里用 `make_filesystem::allocator_after_make_filesystem`，或
    ///   `install_root_ring_occupancy(make_filesystem::root_ring_occupancy_after_make_filesystem(..))`；挂载建的分配器（`mount::rebuilt_allocator`）装着。
    /// - 这条根的 txg 不是上一条记到的加一：有一条根没经分配器写出去，这张表说的环已经不是盘上的环，再按它回收会把还被环里的根引用的槽发出去。
    ///
    /// 记完、回收完之后把这一次回收用的门槛与还钉着的槽里按（钉值, 槽）最小的那一槽记下（[`Self::reclaim_floor_when_the_latest_root_was_recorded`]，C773）：
    /// 带单元的发布拿它写根记录的 t 与 p；零单元发布在根落盘之后才调，它的 t、p 照抄上一版，用不到这一对。
    pub fn record_root_written_by_this_process(&mut self, checkpoint_txg: CheckpointTxg) {
        let occupancy = self.root_ring.as_mut().expect(
            "分配器没装根环表却记了一条根：写根的分配器都由建它的一方装上根环表（挂载在 mount::rebuilt_allocator 装，\
             mkfs 同一个进程里用 make_filesystem::allocator_after_make_filesystem，或 install_root_ring_occupancy(\
             make_filesystem::root_ring_occupancy_after_make_filesystem(..))）；没装就一次不回收，转过根环之后 I-3.1 红（C551）",
        );
        let expected_txg = CheckpointTxg(occupancy.latest_recorded_txg.0 + 1);
        assert_eq!(
            checkpoint_txg, expected_txg,
            "装了根环表的进程写的根 txg 逐个加一、每一条都经分配器记：带单元的发布在取完落点之后记，mount 发的零单元发布在根落盘之后记，\
             调用方直接发的零单元发布由 publish_first_file 在接上它之前补记（record_zero_unit_roots_leading_to）；\
             失败的发布连这张表一起退回、重发用同一个 txg。跳号说明有一条根绕过了这几处"
        );
        occupancy.latest_recorded_txg = checkpoint_txg;
        let ring_slot = target_for_publish(checkpoint_txg, occupancy.slots_per_region);
        let displaced = occupancy
            .occupants
            .insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });
        // 与挂载时同一个谓词（一处定义）：max(F_生效, 环里最旧有效根)（RGall）。
        let reclaim_floor = crate::mount::reclaim_floor(
            occupancy.effective_floor,
            occupancy.oldest_valid_root_in_the_ring(),
        );
        // C694 的 C 只在这里解开（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条 `ring`）：这个进程写掉那条记录的 txg
        // 对应的根环槽，这次发布的落点已经取完（调用时机是调用方的前置条件，见上）。
        let released_named_by_unapplied_journal_records = occupancy
            .placements_named_by_unapplied_journal_records
            .remove(&ring_slot)
            .unwrap_or_default();
        let still_protected_by_the_isolation = occupancy.placements_the_isolation_still_protects();
        // 钉槽 p 的采样与回收门槛同一刻：这条根已经盖掉它那一槽（盖掉的若是钉着的槽，钉随之解开），之后到装根之前不再记根。
        // 钉值与槽一起取，按（钉值, 槽）取最小：三个区域的槽 0 钉值都是 0，只比钉值就分不出取哪一槽（C773 判决 R1 乙）。
        let lowest_pin = occupancy
            .ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held()
            .into_iter()
            .map(|(pinned_ring_slot, pinned_txg)| LowestPinnedRingSlot {
                pinned_txg,
                ring_slot: pinned_ring_slot,
            })
            .min_by_key(|pin| (pin.pinned_txg, pin.ring_slot));
        match displaced {
            Some(
                RootRingOccupant::AbandonedRoot {
                    referenced_placements,
                    ..
                }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root:
                        RootReadOnOneRebuildPass::AbandonedRoot {
                            referenced_placements,
                        },
                    ..
                },
            ) => self.clear_isolation_nothing_still_protected_backs(
                &referenced_placements,
                &still_protected_by_the_isolation,
            ),
            Some(
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root: RootReadOnOneRebuildPass::ValidRoot { .. },
                    ..
                },
            )
            | None => {}
        }
        self.clear_isolation_nothing_still_protected_backs(
            &released_named_by_unapplied_journal_records,
            &still_protected_by_the_isolation,
        );
        let reclaimed =
            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
        self.isolate_reclaimed_slots_the_isolation_still_protects(
            &reclaimed,
            &still_protected_by_the_isolation,
        );
        self.reclaim_floor_when_the_latest_root_was_recorded =
            Some(ReclaimFloorWhenTheRootWasRecorded {
                reclaim_floor,
                lowest_pin,
            });
    }

    /// 这个分配器上一次记根（[`Self::record_root_written_by_this_process`]）回收用的门槛与记完之后还钉着的槽里按（钉值, 槽）最小的那一槽；
    /// 这个分配器还没记过根时 `None`。带单元的发布在记这次的根之后、装根之前读它（C773 的采样点）。
    #[must_use]
    pub fn reclaim_floor_when_the_latest_root_was_recorded(
        &self,
    ) -> Option<ReclaimFloorWhenTheRootWasRecorded> {
        self.reclaim_floor_when_the_latest_root_was_recorded
    }

    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读坏或读回全零、记了这条根之后会是环里最小的那个槽
    /// （D16（发布语义） 已定项 1：读坏的槽钉在它可能住过的最旧 txg，重读读出有效根之前
    /// 回收门槛不越过它——读得出且自证过就照读出来的根算，按实例表有效的照它的 txg（回收门槛取整个环，RGall，多旧都算），被抛弃的当没有；
    /// 仍读不出、读得出而自证不过或读回全零照旧钉着：读回全零不算确认为空）。
    /// `reread` 读盘、判全零、自证与实例表（`mount::reread_of_a_root_ring_slot_bad_at_mount`）；
    /// 这里只定重读哪一槽、按读到的改表。钉着的槽一个都不在最小的位置上时一个都不读。
    ///
    /// # Panics
    /// 同 [`Self::record_root_written_by_this_process`]：没装根环表、或这条根的 txg 不是上一条记到的加一。
    pub fn record_root_written_by_this_process_rereading_slots_bad_at_mount(
        &mut self,
        checkpoint_txg: CheckpointTxg,
        reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotBadAtMount,
    ) {
        if let Some(occupancy) = self.root_ring.as_mut() {
            let ring_slot_of_this_root =
                target_for_publish(checkpoint_txg, occupancy.slots_per_region);
            // 这条根要盖掉的那一槽不算：记了之后那里住的是这条根。
            let valid_root_txgs: Vec<CheckpointTxg> = occupancy
                .occupants
                .iter()
                .filter(|(ring_slot, _)| **ring_slot != ring_slot_of_this_root)
                .filter_map(|(_, occupant)| match occupant {
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: txg_of_the_occupant,
                    }
                    | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                        checkpoint_txg: txg_of_the_occupant,
                    }
                    | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                        root:
                            RootReadOnOneRebuildPass::ValidRoot {
                                checkpoint_txg: txg_of_the_occupant,
                            },
                        ..
                    } => Some(*txg_of_the_occupant),
                    RootRingOccupant::BadAtMount(_)
                    | RootRingOccupant::AllZeroAtMount { .. }
                    | RootRingOccupant::AbandonedRoot { .. }
                    | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                        root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                        ..
                    } => None,
                })
                .chain(std::iter::once(checkpoint_txg))
                .collect();
            let mut bad_at_mount =
                occupancy.ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held();
            bad_at_mount.remove(&ring_slot_of_this_root);
            let (_oldest, rereads) = oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
                &valid_root_txgs,
                bad_at_mount,
                reread,
            );
            for (ring_slot, reread_of_the_slot) in rereads {
                match reread_of_the_slot {
                    RereadOfARootRingSlotBadAtMount::ValidRoot {
                        checkpoint_txg: txg_read_on_the_reread,
                    } => {
                        occupancy.occupants.insert(
                            ring_slot,
                            RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                                checkpoint_txg: txg_read_on_the_reread,
                            },
                        );
                    }
                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {
                        occupancy.occupants.remove(&ring_slot);
                    }
                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
                    | RereadOfARootRingSlotBadAtMount::AllZero => {}
                }
            }
        }
        self.record_root_written_by_this_process(checkpoint_txg);
    }

    /// 第一个文件版本要建在上面的那一版（txg `version_to_build_on`，树表 0 条）之前，这个实例接连发过的零单元发布把根写到了
    /// 那个 txg：调用方直接调 `transaction::publish_without_units` 时那几条根没经分配器。它们是同一个实例在这个进程里接着写的
    /// （零单元发布一条接一条、txg 逐个加一，每一条交回 Ok 之后调用方才接着发），所以 [上一条记到的 txg + 1, 那一版的 txg] 里
    /// 每一个 txg 的槽上都是这个进程写的一条有效根——在这里逐条补记，按 `record_root_written_by_this_process` 的规则转环。
    /// 那一版的 txg 不比上一条记到的大时什么都不做（接着的那次发布记根时跳号就断言失败）。没装根环时什么都不做。
    pub fn record_zero_unit_roots_leading_to(&mut self, version_to_build_on: CheckpointTxg) {
        let Some(latest_recorded_txg) = self
            .root_ring
            .as_ref()
            .map(|occupancy| occupancy.latest_recorded_txg)
        else {
            return;
        };
        for txg in latest_recorded_txg.0 + 1..=version_to_build_on.0 {
            self.record_root_written_by_this_process(CheckpointTxg(txg));
        }
    }

    /// 一条被抛弃根离开根环，或水位之上没施加的记录点名的一组落点解开：`left_the_ring` 罩的槽里，`still_referenced`
    /// （环里还在的被抛弃根引用的、还没解开的记录点名的，[`RootRingOccupancy::placements_the_isolation_still_protects`]）不罩着的清掉隔离位。
    fn clear_isolation_nothing_still_protected_backs(
        &mut self,
        left_the_ring: &[PlacementOnDevice],
        still_referenced: &[PlacementOnDevice],
    ) {
        for placement in left_the_ring {
            let device_map = self
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == placement.device)
                .expect(
                    "被抛弃根的分配记录的盘在池里：挂载时隔离它们走的是同一条 isolate_abandoned",
                );
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                let is_still_referenced = still_referenced
                    .iter()
                    .any(|other| other.covers(placement.device, SlotNumber(slot)));
                if !is_still_referenced {
                    device_map.clear_isolation_of_slot(SlotNumber(slot));
                }
            }
        }
    }

    /// 挂载内回收出来的槽里，环里还在的被抛弃根引用着的、还没解开的水位之上没施加的记录点名的补隔离
    /// （同一个槽隔离两次不重复计数，`DeviceFreeMap::isolate`）。
    fn isolate_reclaimed_slots_the_isolation_still_protects(
        &mut self,
        reclaimed: &[AllocationRecord],
        still_referenced: &[PlacementOnDevice],
    ) {
        for record in reclaimed {
            let device_map = self
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("刚回收的记录的盘在池里");
            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                let is_referenced_by_an_abandoned_root = still_referenced
                    .iter()
                    .any(|placement| placement.covers(record.device, SlotNumber(slot)));
                if is_referenced_by_an_abandoned_root {
                    device_map.isolate(SlotNumber(slot), 1);
                }
            }
        }
    }

    #[must_use]
    pub fn open_segment(&self) -> Option<SlotNumber> {
        self.open_segment
    }

    /// 这次挂载开过的聚簇段的起点（含已经回落掉、这会儿不 bump 的那些）：用户数据一个都不许落进去。
    #[must_use]
    pub fn cluster_segments(&self) -> &BTreeSet<SlotNumber> {
        &self.cluster_segments
    }
}

#[cfg(test)]
mod tests;
