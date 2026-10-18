//! harness 测试文件 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, OVERWRITE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
    InstanceTableOfTheNewestRootRead, MountError, Mounted, RereadsOfThisMount,
    RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::root_ring::RootRingSlot;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, NamedRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::rc::Rc;

/// 一次不注错的再挂里，根环每个槽第几次被读是重建分配器读根环那一遍（从 1 数；见文件头）。
pub const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;

/// 这段历史上一次不注错的再挂里根环每个槽读到盘上的次数，由不注错那条用例现量、钉住：读阶段与重建分配器那几遍
/// （第 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次是重建那一遍），之后可写挂载取号之前读一遍、记进预演与真做共用的那一份
/// （第 7 次），真做取号写之前那几次读照旧读盘、其中读了一遍根环（第 8 次）；写行、暖机、抬 F 与续做再读根环都取共用的那一份、不再读盘
/// （岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一「读一次两边共用」，用户定）。攻方在那之前量的是 15 遍。
pub const READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS: u64 = 8;

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
pub fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 根环每个槽被读的次数与哪几次读坏（几块盘共用一份）。
pub struct RootRingReadsState {
    pub target: RootRingSlotTarget,
    /// 每个槽按它自己被读的次数从 1 数，落在这个区间里的那几次读报块设备错；空区间是一次都不坏。
    pub failing_reads_of_each_slot: RangeInclusive<u64>,
    pub reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
    pub failed_reads: u64,
}

#[derive(Clone)]
pub struct SharedRootRingReads(Rc<RefCell<RootRingReadsState>>);

impl SharedRootRingReads {
    pub fn new(
        target: RootRingSlotTarget,
        failing_reads_of_each_slot: RangeInclusive<u64>,
    ) -> Self {
        Self(Rc::new(RefCell::new(RootRingReadsState {
            target,
            failing_reads_of_each_slot,
            reads_of_each_slot: BTreeMap::new(),
            failed_reads: 0,
        })))
    }

    /// 这一次读落在哪个根环槽里就给那个槽记一次，交回这一次要不要读坏。
    pub fn note_a_read_and_tell_whether_it_fails(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    ) -> bool {
        let mut state = self.0.borrow_mut();
        let Some(ring_slot) = state.target.slot_covering(device, offset) else {
            return false;
        };
        let reads_of_this_slot = {
            let reads = state.reads_of_each_slot.entry(ring_slot).or_insert(0);
            *reads += 1;
            *reads
        };
        let fails_this_time = state
            .failing_reads_of_each_slot
            .contains(&reads_of_this_slot);
        if fails_this_time {
            state.failed_reads += 1;
        }
        fails_this_time
    }

    /// 根环每个槽被读了几次（没被读过的槽不在表里）。
    pub fn reads_of_each_slot(&self) -> BTreeMap<RootRingSlot, u64> {
        self.0.borrow().reads_of_each_slot.clone()
    }

    pub fn failed_reads(&self) -> u64 {
        self.0.borrow().failed_reads
    }

    /// 从这一刻起每个根环槽的每次读都报块设备错（用例在「重读一次」之前的钩子里调）。
    pub fn fail_every_later_read_of_each_slot(&self) {
        self.0.borrow_mut().failing_reads_of_each_slot = 1..=u64::MAX;
    }
}

/// 读的时候按 [`SharedRootRingReads`] 让根环槽的某几次读报块设备错的盘；写、写零、屏障原样交给里面那块盘。
pub struct DeviceFailingPassesOfTheRootRing<Inner: BlockDevice> {
    pub identity: DeviceIdentity,
    pub inner: Inner,
    pub root_ring_reads: SharedRootRingReads,
}

impl<Inner: BlockDevice> BlockDevice for DeviceFailingPassesOfTheRootRing<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if self
            .root_ring_reads
            .note_a_read_and_tell_whether_it_fails(self.identity, offset)
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一次根环槽读坏",
            )));
        }
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 再挂那一次根环哪几遍读坏（两块盘全部槽）。
#[derive(Clone, Copy, Debug)]
pub enum RootRingPassesFailingDuringTheRemount {
    NoneOfThePasses,
    OnlyTheAllocatorRebuild,
    TheAllocatorRebuildAndItsReread,
}

impl RootRingPassesFailingDuringTheRemount {
    pub fn failing_reads_of_each_slot(self) -> RangeInclusive<u64> {
        let rebuild = READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD;
        match self {
            // 空区间：一次都不坏，只数。
            RootRingPassesFailingDuringTheRemount::NoneOfThePasses => RangeInclusive::new(1, 0),
            RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild => rebuild..=rebuild,
            RootRingPassesFailingDuringTheRemount::TheAllocatorRebuildAndItsReread => {
                rebuild..=rebuild + 1
            }
        }
    }
}

/// 卸载半途崩溃之后的那份盘面，与卸载之前的盘面、这个池的参数（再挂与读回要用）。
pub struct NarrowPoolAfterACrashedUnmount {
    pub pool: PoolUnderTest<SparseBlockDevice>,
    pub image_before_the_unmount: MemoryPool,
    pub image_after_the_crashed_unmount: MemoryPool,
}

impl NarrowPoolAfterACrashedUnmount {
    /// 两块盘全部根环槽。
    pub fn every_root_ring_slot(&self) -> RootRingSlotTarget {
        RootRingSlotTarget {
            named_slots: NamedRootRingSlots::every_slot_in_the_ring(
                self.pool.parameters.geometry.root_ring_slots_per_region,
            ),
            region_devices: self.pool.parameters.region_devices,
            fixed_structure_slot_spacing: self
                .pool
                .parameters
                .geometry
                .fixed_structure_slot_spacing,
        }
    }

    pub fn root_ring_slot_count(&self) -> u64 {
        u64::from(self.every_root_ring_slot().named_slots.count())
    }

    /// 卸载半途崩溃之后的盘面拷一份、包上根环读故障。
    pub fn devices_failing<Inner: BlockDevice>(
        &self,
        root_ring_reads: &SharedRootRingReads,
        wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> Inner,
    ) -> Vec<(DeviceIdentity, DeviceFailingPassesOfTheRootRing<Inner>)> {
        plain_devices_on(&self.image_after_the_crashed_unmount)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    DeviceFailingPassesOfTheRootRing {
                        identity,
                        inner: wrap(identity, device),
                        root_ring_reads: root_ring_reads.clone(),
                    },
                )
            })
            .collect()
    }

    /// 再可写挂载（准入判着、影子账开、批大小 1）。
    pub fn mount_writable<Device: BlockDevice>(
        &self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<Mounted, MountError> {
        mount_writable_with_test_only_switches_and_deletion_switches(
            &self.pool.parameters,
            devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
    }

    /// 卸载半途崩溃之后的盘面上再可写挂载一次，根环照 `passes_failing` 读坏；交回挂载的结局、挂载之后的盘面与根环读数。
    pub fn remount(
        &self,
        passes_failing: RootRingPassesFailingDuringTheRemount,
    ) -> (Result<Mounted, MountError>, MemoryPool, SharedRootRingReads) {
        let root_ring_reads = SharedRootRingReads::new(
            self.every_root_ring_slot(),
            passes_failing.failing_reads_of_each_slot(),
        );
        let (mounted, image) = self.remount_reading_the_root_ring_through(&root_ring_reads);
        (mounted, image, root_ring_reads)
    }

    /// `device` 那几个区域的每个根环槽。
    pub fn root_ring_slots_of(&self, device: DeviceIdentity) -> RootRingSlotTarget {
        let every_slot = self.every_root_ring_slot();
        let ring_slots_of_the_device: Vec<RootRingSlot> = every_slot
            .named_slots
            .named_slots()
            .into_iter()
            .filter(|ring_slot| {
                self.pool.parameters.region_devices
                    [usize::try_from(ring_slot.region).expect("区域号")]
                    == device
            })
            .collect();
        RootRingSlotTarget {
            named_slots: NamedRootRingSlots::naming(&ring_slots_of_the_device),
            region_devices: every_slot.region_devices,
            fixed_structure_slot_spacing: every_slot.fixed_structure_slot_spacing,
        }
    }

    /// 卸载半途崩溃之后的盘面上再可写挂载一次，根环照 `root_ring_reads` 读坏；交回挂载的结局与挂载之后的盘面。
    pub fn remount_reading_the_root_ring_through(
        &self,
        root_ring_reads: &SharedRootRingReads,
    ) -> (Result<Mounted, MountError>, MemoryPool) {
        let mut devices = self.devices_failing(root_ring_reads, |_, device| device);
        let mounted = self.mount_writable(&mut devices);
        let image = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner.image.clone()))
                .collect(),
            device_size_in_bytes: self.image_after_the_crashed_unmount.device_size_in_bytes,
        };
        (mounted, image)
    }
}

/// 整池从第 `first_failing_write` 次写起每次写都报错的注错计划（掉电那一形：盘停在之前那几个写之后）。
pub fn plan_failing_every_write_from(
    device_width: HistoryDeviceWidth,
    first_failing_write: u64,
) -> SharedFaultPlan {
    SharedFaultPlan::armed(
        device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
        },
    )
}

/// 攻方那一段的起点（见文件头）：卸载落下 15 个写就崩（两块盘的系统配置都带了新 F、带新 F 的根一条都没落）。
pub fn narrow_pool_after_a_crashed_unmount() -> NarrowPoolAfterACrashedUnmount {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let overwrites_before_the_file_grows = 6;
    let data_units_of_the_file = 4;
    let unmount_writes_landed = 15;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            264,
        );
    for overwrite_index in 0..overwrites_before_the_file_grows {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
        });
    }
    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
    {
        let PoolUnderTest {
            session,
            devices,
            parameters,
            write_time_seconds,
            ..
        } = &mut pool;
        let session = session.as_mut().expect("这一刻有可写会话");
        let current = session
            .current
            .version_of_trees()
            .expect("第一个文件之后有树")
            .clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图那次");
    }
    pool.session = None;
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .unwrap_or_else(|error| panic!("崩了之后重开可写、续做：{error:?}"));
    let (_, session_of_the_continued_mount) = MountedSession::of_the_mount(mounted);
    pool.session = Some(session_of_the_continued_mount);
    let image_before_the_unmount = pool.image();
    let write_plan = plan_failing_every_write_from(device_width, unmount_writes_landed + 1);
    let mut unmount_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&image_before_the_unmount)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
                )
            })
            .collect();
    let image_after_the_crashed_unmount = {
        let session = pool.session();
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        let unmounted = unmount(
            &pool.parameters,
            &mut unmount_devices,
            &mut allocator,
            &mut current,
            session.shadow_ledger,
        );
        assert!(
            unmounted.is_err(),
            "卸载在第 {} 个写上掉电、报错：{:?}",
            unmount_writes_landed + 1,
            unmounted.as_ref().ok().map(|_| "做成了")
        );
        MemoryPool {
            devices: unmount_devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
        }
    };
    NarrowPoolAfterACrashedUnmount {
        pool,
        image_before_the_unmount,
        image_after_the_crashed_unmount,
    }
}

/// 只坏重建分配器那一遍（两块盘全部根环槽报读错）：重读那一遍读得出，挂载照常可写，交回与不注错的再挂同一份盘面（判定不变）；
/// 两块盘都在时池级 checker 一条都不判违例（I-7.4「近 K 代块未被复用」在内）。
/// 判别力：删掉重读（第一遍读不出就拒）时第一项红（Q3 那一拒）；不重读、照旧给空表往下走时，写行那次记根把释放代高于 F_生效 的一截
/// 当场放开、暖机写进盘上环里更旧的有效根（txg 20）还引用的单元，写行之后那次准入抬 F 读不出它的树表，挂载落了盘之后报
/// `FloorRaiseFailedAfterTheMountsPublishes`（攻方第 1.3 节那一形；变异表里这一刀量过，`UnitUnreadable { slot: SlotNumber(1442) }`），第一项同样红。
pub fn only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over(
    images: &mut PoolCheckerImages,
) {
    let history = narrow_pool_after_a_crashed_unmount();
    let (remounted, image_after_the_remount, root_ring_reads) =
        history.remount(RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild);
    let mounted = remounted.unwrap_or_else(|error| {
        panic!("只坏重建分配器那一遍：重读读得出，照常可写挂载：{error:?}")
    });
    // 两块盘都在：池级 checker 一条都不判违例（I-7.4 在内）
    images.hand_out(
        "只坏重建分配器那一遍的再挂之后",
        || image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let (remounted_without_faults, image_after_the_remount_without_faults, _) =
        history.remount(RootRingPassesFailingDuringTheRemount::NoneOfThePasses);
    remounted_without_faults
        .unwrap_or_else(|error| panic!("同一份盘面不注错地再可写挂载：{error:?}"));
    assert!(
        image_after_the_remount == image_after_the_remount_without_faults,
        "只坏重建分配器那一遍的再挂与不注错的再挂交回逐字节相同的盘面（重读的读数与不注错时相同，判定不变）"
    );
    assert_eq!(
        mounted.output.rereads.root_ring_of_the_rebuilt_allocator,
        RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
        "重建分配器读根环那一遍一条有效根都没读出、重读读出：读坏的确是重建分配器那一遍（第 {READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD} 次读每个槽）"
    );
    assert_eq!(
        mounted.output.rereads.instance_table_of_the_newest_root,
        InstanceTableOfTheNewestRootRead::OnTheFirstRead,
        "最新那条根的实例表第一遍就读出（读坏的不是它那一遍）"
    );
    assert_eq!(
        root_ring_reads.failed_reads(),
        history.root_ring_slot_count(),
        "读坏的恰是一整遍：每个根环槽一次"
    );
    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
    assert!(
        reads_of_each_slot.len() == usize::try_from(history.root_ring_slot_count()).expect("槽数")
            && reads_of_each_slot.values().all(|reads| *reads
                == READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS + 1),
        "每个根环槽比不注错的再挂多读恰一次（重读那一遍）：{reads_of_each_slot:?}"
    );
}

/// 不注错的对照：同一份盘面再可写挂载，一次都不拒、不重读，根环每个槽读到盘上的次数是 [`READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS`]，两块盘都在时池级 checker 不判违例。
pub fn a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before(
    images: &mut PoolCheckerImages,
) {
    let history = narrow_pool_after_a_crashed_unmount();
    let (remounted, image_after_the_remount, root_ring_reads) =
        history.remount(RootRingPassesFailingDuringTheRemount::NoneOfThePasses);
    let mounted = remounted.unwrap_or_else(|error| panic!("不注错的再挂照常可写：{error:?}"));
    let RereadsOfThisMount {
        read_stage: _,
        instance_table_of_the_newest_root,
        root_ring_of_the_rebuilt_allocator,
    } = mounted.output.rereads;
    assert_eq!(
        (
            instance_table_of_the_newest_root,
            root_ring_of_the_rebuilt_allocator
        ),
        (
            InstanceTableOfTheNewestRootRead::OnTheFirstRead,
            RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead
        ),
        "不注错时实例表与重建分配器读根环都第一遍就读出"
    );
    assert_eq!(root_ring_reads.failed_reads(), 0, "不注错：一次都没读坏");
    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
    assert!(
        reads_of_each_slot.len() == usize::try_from(history.root_ring_slot_count()).expect("槽数")
            && reads_of_each_slot
                .values()
                .all(|reads| *reads == READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS),
        "不注错的再挂里每个根环槽读 {READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS} 次（没有多出的重读）：{reads_of_each_slot:?}"
    );
    images.hand_out(
        "不注错的再挂之后",
        || image_after_the_remount,
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 重读那一遍只有一部分槽读坏（交主 agent 的那一格，能造的一形）：重建分配器那一遍两块盘全部根环槽报读错，重读那一遍盘 1 的根环槽仍报读错、
/// 盘 0 的读得出。两遍并起来的读数与「重建那一遍与重读都只坏盘 1」（rr2 之后只坏盘 1 的那一遍也整环重读，Q4）逐槽相同：挂载做成、交回的盘面与
/// 两遍都只坏盘 1 的再挂逐字节相同（两者都重读过、重读那一遍都只读出盘 0 的根，照 m 取不到那一支往下走），两块盘都在时池级 checker 不判违例。
/// 判别力：删掉重读时第一项红（Q3 那一拒）；不重读、照旧给空表往下走时同上一形，挂载落了盘之后报
/// `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
pub fn a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over(
    images: &mut PoolCheckerImages,
) {
    let history = narrow_pool_after_a_crashed_unmount();
    let rebuild = READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD;
    // 外层数每一次根环读、只坏重建那一遍；里层只数外层放过去的读，它的第 6 次是外层的第 7 次（重读那一遍），只坏盘 1 的槽。
    let reads_of_every_slot =
        SharedRootRingReads::new(history.every_root_ring_slot(), rebuild..=rebuild);
    let reads_of_device_one_let_through = SharedRootRingReads::new(
        history.root_ring_slots_of(DeviceIdentity(1)),
        rebuild..=rebuild,
    );
    let mut devices = history.devices_failing(&reads_of_every_slot, |identity, device| {
        DeviceFailingPassesOfTheRootRing {
            identity,
            inner: device,
            root_ring_reads: reads_of_device_one_let_through.clone(),
        }
    });
    let remounted = history.mount_writable(&mut devices);
    let mounted = remounted
        .unwrap_or_else(|error| panic!("重读那一遍读得出盘 0 的根：照常可写挂载：{error:?}"));
    let image_after_the_remount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.inner.image.clone()))
            .collect(),
        device_size_in_bytes: history.image_after_the_crashed_unmount.device_size_in_bytes,
    };
    let reads_failing_only_on_device_one = SharedRootRingReads::new(
        history.root_ring_slots_of(DeviceIdentity(1)),
        rebuild..=rebuild + 1,
    );
    let (remounted_failing_only_on_device_one, image_after_the_remount_failing_only_on_device_one) =
        history.remount_reading_the_root_ring_through(&reads_failing_only_on_device_one);
    let mounted_failing_only_on_device_one = remounted_failing_only_on_device_one
        .unwrap_or_else(|error| panic!("重建那一遍与重读都只坏盘 1：照常可写挂载：{error:?}"));
    assert!(
        image_after_the_remount == image_after_the_remount_failing_only_on_device_one,
        "重读那一遍只坏盘 1 的再挂与两遍都只坏盘 1 的再挂交回逐字节相同的盘面（重读的读数一样，判定一样）"
    );
    assert_eq!(
        (
            mounted.output.rereads.root_ring_of_the_rebuilt_allocator,
            mounted_failing_only_on_device_one
                .output
                .rereads
                .root_ring_of_the_rebuilt_allocator
        ),
        (
            RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
            RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
        ),
        "两次都重读过（后一次第一遍盘 1 没读出有效根，rr2 照样重读）"
    );
    assert_eq!(
        (
            reads_of_every_slot.failed_reads(),
            reads_of_device_one_let_through.failed_reads()
        ),
        (
            history.root_ring_slot_count(),
            u64::from(
                history
                    .root_ring_slots_of(DeviceIdentity(1))
                    .named_slots
                    .count()
            )
        ),
        "重建那一遍每个槽坏一次，重读那一遍盘 1 的每个槽坏一次"
    );
    images.hand_out(
        "重建那一遍每个槽坏一次、重读那一遍盘 1 每个槽坏一次的再挂之后",
        || image_after_the_remount,
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
