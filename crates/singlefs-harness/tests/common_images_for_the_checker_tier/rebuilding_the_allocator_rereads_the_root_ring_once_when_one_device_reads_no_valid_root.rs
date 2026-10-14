//! harness 测试文件 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, DeviceOverASparseImage, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    mount_writable_with_test_only_switches_and_deletion_switches,
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by, unmount,
    BeforeTheOneReread, MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    verified_system_configuration_slots, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// 一次不注错的再挂里，根环每个槽第几次被读是重建分配器读根环那一遍（从 1 数；与
/// `rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 那段历史同一个数，这里由钩子现核）。
pub const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;

/// 重建分配器整环重读那一遍是每个槽的第几次读（紧接着重建那一遍）。
pub const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD: u64 =
    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1;

/// 读回乱字节那一类填的字节（攻方原型 `k.rs` 同一个值）：不是全 0，自证不过。
pub const GARBAGE_BYTE: u8 = 0xA5;

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

/// 重建那一遍点名的槽这一次读成什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRingSlotReadFault {
    /// 报块设备错（攻方计划的 `e`）。
    ReadFails,
    /// 报成功、读回全 0（`z`）。
    ReadsBackAllZero,
    /// 报成功、读回不是全 0 而自证不过的乱字节（`g`）。
    ReadsBackGarbage,
}

/// 一块盘上重建那一遍点名哪几个根环槽。
#[derive(Clone, Copy, Debug)]
pub enum RootRingSlotsOfOneDevice {
    /// `-`：一个都不点名。
    NoSlot,
    /// `all`：每一个。
    Every,
    /// `but1`：除了这块盘上最新那条自证过的根所在的槽之外的每一个。
    EveryButTheSlotOfTheNewestRoot,
}

/// 攻方的一份读坏计划：两块盘各点名哪几个槽、点名的槽读成什么。
#[derive(Clone, Copy, Debug)]
pub struct RebuildPassPlan {
    pub device_zero: RootRingSlotsOfOneDevice,
    pub device_one: RootRingSlotsOfOneDevice,
    pub fault: RootRingSlotReadFault,
}

/// `P5:all/-:e`
pub const DEVICE_ZERO_REPORTS_READ_ERRORS: RebuildPassPlan = RebuildPassPlan {
    device_zero: RootRingSlotsOfOneDevice::Every,
    device_one: RootRingSlotsOfOneDevice::NoSlot,
    fault: RootRingSlotReadFault::ReadFails,
};

/// `P5:-/all:e`
pub const DEVICE_ONE_REPORTS_READ_ERRORS: RebuildPassPlan = RebuildPassPlan {
    device_zero: RootRingSlotsOfOneDevice::NoSlot,
    device_one: RootRingSlotsOfOneDevice::Every,
    fault: RootRingSlotReadFault::ReadFails,
};

/// `P5:but1/all:z`
pub const DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS: RebuildPassPlan =
    RebuildPassPlan {
        device_zero: RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot,
        device_one: RootRingSlotsOfOneDevice::Every,
        fault: RootRingSlotReadFault::ReadsBackAllZero,
    };

/// `P5:all/but1:g`
pub const DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE: RebuildPassPlan =
    RebuildPassPlan {
        device_zero: RootRingSlotsOfOneDevice::Every,
        device_one: RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot,
        fault: RootRingSlotReadFault::ReadsBackGarbage,
    };

/// 这个进程写过一个根环槽之后，点名读坏它的那几次还照不照坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultsOnARootRingSlotThisProcessWrote {
    /// 照坏（坏道写不好）。
    KeepFailing,
    /// 不再坏（写把坏道换掉了；攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs` 的 `FaultDevice` 同一个做法）。
    StopFailing,
}

/// 根环每个槽被读的次数与点名读坏的那几次（几块盘共用一份）。
pub struct RootRingReadsState {
    pub slot_spacing_in_bytes: u64,
    /// 每个根环槽在哪块盘、盘内从哪个偏移起。
    pub slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
    /// （槽，这个槽第几次被读，从 1 数）→ 这一次读成什么。
    pub faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    /// 槽 →（从这个槽第几次被读起，每一次都读成什么）：坏道从这一刻起一直坏。`faulted_reads_of_each_slot` 点名的那一次先照它。
    pub slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
    pub reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
    pub faulted_reads: u64,
    pub faults_on_a_slot_this_process_wrote: FaultsOnARootRingSlotThisProcessWrote,
    pub slots_this_process_wrote: std::collections::BTreeSet<RootRingSlot>,
    /// 每一次根环槽读（槽，这个槽第几次被读）按发生的次序，中间每一段写记成一项 `None`（连着的写只记一项）。
    pub reads_and_writes_in_order: Vec<Option<(RootRingSlot, u64)>>,
}

#[derive(Clone)]
pub struct SharedRootRingReads(Rc<RefCell<RootRingReadsState>>);

impl SharedRootRingReads {
    /// 这一次读落在哪个根环槽里就给那个槽记一次；是点名读坏的那一次，交回这一次读成什么与那个槽在盘内的起点。
    pub fn note_a_read_and_tell_its_fault(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    ) -> Option<(RootRingSlotReadFault, u64)> {
        let mut state = self.0.borrow_mut();
        let spacing = state.slot_spacing_in_bytes;
        let (ring_slot, slot_start) = state
            .slots
            .iter()
            .find(|(_, slot_device, start)| {
                *slot_device == device && offset.0 >= *start && offset.0 < *start + spacing
            })
            .map(|(ring_slot, _, start)| (*ring_slot, *start))?;
        let reads_of_this_slot = {
            let reads = state.reads_of_each_slot.entry(ring_slot).or_insert(0);
            *reads += 1;
            *reads
        };
        state
            .reads_and_writes_in_order
            .push(Some((ring_slot, reads_of_this_slot)));
        if state.faults_on_a_slot_this_process_wrote
            == FaultsOnARootRingSlotThisProcessWrote::StopFailing
            && state.slots_this_process_wrote.contains(&ring_slot)
        {
            return None;
        }
        let fault = state
            .faulted_reads_of_each_slot
            .get(&(ring_slot, reads_of_this_slot))
            .copied()
            .or_else(|| {
                state
                    .slots_failing_from_a_read_onward
                    .get(&ring_slot)
                    .filter(|(first_failing_read, _)| reads_of_this_slot >= *first_failing_read)
                    .map(|(_, fault)| *fault)
            })?;
        state.faulted_reads += 1;
        Some((fault, slot_start))
    }

    pub fn reads_of_each_slot(&self) -> BTreeMap<RootRingSlot, u64> {
        self.0.borrow().reads_of_each_slot.clone()
    }

    pub fn faulted_reads(&self) -> u64 {
        self.0.borrow().faulted_reads
    }

    /// 记一次写：写盖到的根环槽记成这个进程写过的；读写次序里记一项写（紧接着上一项写的不重复记）。
    pub fn note_a_write(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: u64) {
        let mut state = self.0.borrow_mut();
        let spacing = state.slot_spacing_in_bytes;
        let written: Vec<RootRingSlot> = state
            .slots
            .iter()
            .filter(|(_, slot_device, start)| {
                *slot_device == device && offset.0 < *start + spacing && offset.0 + length > *start
            })
            .map(|(ring_slot, _, _)| *ring_slot)
            .collect();
        state.slots_this_process_wrote.extend(written);
        if state.reads_and_writes_in_order.last() != Some(&None) {
            state.reads_and_writes_in_order.push(None);
        }
    }

    pub fn reads_and_writes_in_order(&self) -> Vec<Option<(RootRingSlot, u64)>> {
        self.0.borrow().reads_and_writes_in_order.clone()
    }
}

/// 读的时候按 [`SharedRootRingReads`] 让点名的那几次根环槽读读坏的盘；写、写零、屏障原样交给里面那块盘。
pub struct DeviceFailingTheRebuildPass {
    pub identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    pub root_ring_reads: SharedRootRingReads,
}

impl BlockDevice for DeviceFailingTheRebuildPass {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let fault = self
            .root_ring_reads
            .note_a_read_and_tell_its_fault(self.identity, offset);
        let filler = match fault {
            None => return self.inner.read_at(offset, buffer),
            Some((RootRingSlotReadFault::ReadFails, _)) => {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "用例点名的这一次根环槽读坏",
                )));
            }
            Some((RootRingSlotReadFault::ReadsBackAllZero, slot_start)) => (0u8, slot_start),
            Some((RootRingSlotReadFault::ReadsBackGarbage, slot_start)) => {
                (GARBAGE_BYTE, slot_start)
            }
        };
        self.inner.read_at(offset, buffer)?;
        let (byte, slot_start) = filler;
        let spacing = self.root_ring_reads.0.borrow().slot_spacing_in_bytes;
        let first_byte_in_the_buffer = usize::try_from(slot_start.saturating_sub(offset.0))
            .expect("缓冲区里的偏移装得进 usize");
        let end_in_the_buffer = usize::try_from(
            (slot_start + spacing - offset.0).min(u64::try_from(buffer.len()).expect("缓冲区长度")),
        )
        .expect("缓冲区里的偏移装得进 usize");
        buffer[first_byte_in_the_buffer..end_in_the_buffer].fill(byte);
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.root_ring_reads.note_a_write(
            self.identity,
            offset,
            u64::try_from(bytes.len()).expect("写的长度装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.root_ring_reads
            .note_a_write(self.identity, offset, length);
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

/// 攻方的两个起点，与一个丢任一块盘之后仍有活文件的起点。
#[derive(Clone, Copy, Debug)]
pub enum StartingPoint {
    /// `192:9:3:1` n = 8。
    UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
    /// `216:7:5:4` n = now。
    UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
    /// `264:6:4`：不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元就崩
    /// （攻方第一次抽样 B 段的 `264:6:4-live-crash` 起点）；重开可写（准入判着、影子账开）做完就崩：两块盘上最新的根都是那次暖机的，
    /// 带着同一个活文件。
    UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
    /// `narrow-384-20-2`（第三轮攻方原型 `research/prompts/m3-batch10-code-r3-opus-model/main.rs` 的 `narrow_start(384, 20, 2)`，只抄造法）：
    /// 不补环窗口、准入关着造的窄盘（单元区 384 槽），第一个文件之后覆盖写 20 次、顺序写长到 2 个单元，进程退出（不再挂）。
    /// 再可写挂载时准入在取号之前不够，写行与暖机之后推抬 F。
    UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
}

/// 再挂之前的盘面，与「卸载前」那一份（丢盘整读拿它比；n = now 那一个起点两份相同）。
pub struct PoolBeforeTheRemount {
    pub image_before_the_crash: MemoryPool,
    pub image_to_remount: MemoryPool,
    pub parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
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

/// 窄盘：第一个文件之后覆盖写 `overwrites` 次、顺序写长到 `data_units_of_the_file` 个单元，在这个进程里发 unlink 的意图、
/// 再删 `batches_after_the_intent` 批就崩；重开可写（准入判着、影子账开、批大小 1）续做走完，会话挂回池上。
pub fn narrow_pool_whose_deletion_was_continued(
    unit_area_slots: u64,
    overwrites: u64,
    data_units_of_the_file: usize,
    batches_after_the_intent: u64,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            unit_area_slots,
        );
    for overwrite_index in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
        });
    }
    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
        .unwrap_or_else(|refusal| {
            panic!("顺序写把文件长到 {data_units_of_the_file} 个单元：{refusal:?}")
        });
    {
        let PoolUnderTest {
            session,
            devices,
            parameters,
            write_time_seconds,
            ..
        } = &mut pool;
        let session = session.as_mut().expect("这一刻有可写会话");
        let mut current = session
            .current
            .version_of_trees()
            .expect("第一个文件之后有树")
            .clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图那次");
        // 迭代上界是 `batches_after_the_intent`；跨轮携带的是现行那一版。
        for batch_index in 0..batches_after_the_intent {
            current = publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                deletion_switches_with_batches_of_one().batch_size,
                session.instance,
            )
            .unwrap_or_else(|refusal| panic!("第 {batch_index} 批：{refusal:?}"));
        }
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
    pool
}

pub fn pool_before_the_remount(starting_point: StartingPoint) -> PoolBeforeTheRemount {
    match starting_point {
        StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites => {
            let unmount_writes_landed = 8;
            let pool = narrow_pool_whose_deletion_was_continued(192, 9, 3, 1);
            let image_before_the_unmount = pool.image();
            let write_plan = plan_failing_every_write_from(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                unmount_writes_landed + 1,
            );
            let mut unmount_devices: Vec<(
                DeviceIdentity,
                FaultInjectingBlockDevice<SparseBlockDevice>,
            )> = plain_devices_on(&image_before_the_unmount)
                .into_iter()
                .map(|(identity, device)| {
                    (
                        identity,
                        FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
                    )
                })
                .collect();
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
                "前提：卸载在第 {} 个写上掉电、报错",
                unmount_writes_landed + 1
            );
            let image_after_the_crashed_unmount = MemoryPool {
                devices: unmount_devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect(),
                device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
            };
            PoolBeforeTheRemount {
                image_before_the_crash: image_before_the_unmount,
                image_to_remount: image_after_the_crashed_unmount,
                parameters: pool.parameters.clone(),
            }
        }
        StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount => {
            let pool = narrow_pool_whose_deletion_was_continued(216, 7, 5, 4);
            let image = pool.image();
            PoolBeforeTheRemount {
                image_before_the_crash: image.clone(),
                image_to_remount: image,
                parameters: pool.parameters.clone(),
            }
        }
        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount => {
            let mut pool =
                start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                    HistoryDeviceWidth::UnitAreaOf384Slots,
                    264,
                );
            // 迭代上界是 6 次覆盖写；没有提前出口。
            for overwrite_index in 0..6 {
                pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
                    panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
                });
            }
            pool.sequential_write(3 * data_unit_payload_capacity() + 100)
                .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
            pool.session = None;
            mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                &mut pool.devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                deletion_switches_with_batches_of_one(),
            )
            .unwrap_or_else(|error| panic!("崩了之后重开可写：{error:?}"));
            let image = pool.image();
            PoolBeforeTheRemount {
                image_before_the_crash: image.clone(),
                image_to_remount: image,
                parameters: pool.parameters.clone(),
            }
        }
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits => {
            let mut pool =
                start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                    HistoryDeviceWidth::UnitAreaOf384Slots,
                    384,
                );
            // 迭代上界是 20 次覆盖写；没有提前出口。
            for overwrite_index in 0..20 {
                pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
                    panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
                });
            }
            pool.sequential_write(data_unit_payload_capacity() + 100)
                .unwrap_or_else(|refusal| panic!("顺序写把文件长到 2 个单元：{refusal:?}"));
            pool.session = None;
            let image = pool.image();
            PoolBeforeTheRemount {
                image_before_the_crash: image.clone(),
                image_to_remount: image,
                parameters: pool.parameters.clone(),
            }
        }
    }
}

/// 照计划点名的根环槽（`but1` 按这份盘面上每块盘最新那条自证过的根认）。
pub fn faulted_slots_of_the_plan(
    image: &MemoryPool,
    plan: RebuildPassPlan,
) -> BTreeMap<RootRingSlot, RootRingSlotReadFault> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = system_configuration.immutable;
    let device_of = |ring_slot: RootRingSlot| {
        immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
    };
    let ring_slots = every_root_ring_slot(&immutable.sizes);
    let mut slot_of_the_newest_root_on_each_device: BTreeMap<
        DeviceIdentity,
        (CheckpointTxg, RootRingSlot),
    > = BTreeMap::new();
    // 迭代上界是根环的槽数；没有提前出口。
    for ring_slot in &ring_slots {
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(
            image,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            *ring_slot,
        ) {
            let newest = slot_of_the_newest_root_on_each_device
                .entry(device_of(*ring_slot))
                .or_insert((root.checkpoint_txg, *ring_slot));
            if root.checkpoint_txg > newest.0 {
                *newest = (root.checkpoint_txg, *ring_slot);
            }
        }
    }
    ring_slots
        .into_iter()
        .filter(|ring_slot| {
            let device = device_of(*ring_slot);
            let slots_of_the_device = if device == DeviceIdentity(0) {
                plan.device_zero
            } else {
                plan.device_one
            };
            match slots_of_the_device {
                RootRingSlotsOfOneDevice::NoSlot => false,
                RootRingSlotsOfOneDevice::Every => true,
                RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot => {
                    slot_of_the_newest_root_on_each_device
                        .get(&device)
                        .is_none_or(|(_, newest_slot)| newest_slot != ring_slot)
                }
            }
        })
        .map(|ring_slot| (ring_slot, plan.fault))
        .collect()
}

/// 只读挂载、把挂载态里每个活着的文件从头读到尾：每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
pub fn live_files_read_whole(image: &MemoryPool) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
    let mut live_files = Vec::new();
    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那三种错。
    for record in read_only.mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = read_only
            .mounted
            .open_file(image, InodeNumber(record.inode))
            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
        let read = file
            .read_at(image, FileOffsetInBytes(0), record.size)
            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
        live_files.push((record.inode, read.bytes));
    }
    Ok(live_files)
}

/// 在 `root` 那一版上打开、整读每个活文件（inode 号、全部字节）；删除中的 inode 不读。
pub fn live_files_read_whole_at(
    image: &MemoryPool,
    root: &RootRecord,
) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let mounted =
        open_pool_for_read(image, root).map_err(|failure| format!("打开那一版：{failure:?}"))?;
    let mut live_files = Vec::new();
    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那两种错。
    for record in mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted
            .open_file(image, InodeNumber(record.inode))
            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
        let read = file
            .read_at(image, FileOffsetInBytes(0), record.size)
            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
        live_files.push((record.inode, read.bytes));
    }
    Ok(live_files)
}

/// 一份镜像的根环里每条自证过的根，带它读出的那个槽。
pub fn self_verified_roots_in_the_ring(image: &MemoryPool) -> Vec<(RootRingSlot, RootRecord)> {
    let immutable = choose_system_configuration(image)
        .expect("系统配置")
        .immutable;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                image,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

/// 挂载之前环里每条自证过的根，挂载之后还住在同一槽、txg 不低于挂载之后最新那条根带的 F（回退还能落到的），丢 `lost` 之后在剩下那块盘上
/// 整读每个活文件，与挂载之前丢同一块盘读出的相同（挂载之前就读不出的不比）。交回比了几条根、其中有活文件的几条与读得不一样的那几条的 txg。
pub fn roots_still_in_the_ring_read_on_a_lone_survivor(
    image_before_the_remount: &MemoryPool,
    image_after_the_remount: &MemoryPool,
    lost: DeviceIdentity,
) -> (usize, usize, Vec<u64>) {
    let roots_after: Vec<(RootRingSlot, RootRecord)> =
        self_verified_roots_in_the_ring(image_after_the_remount);
    let floor_after = roots_after
        .iter()
        .map(|(_, root)| (root.checkpoint_txg, root.instance, root.rollback_floor))
        .max()
        .map_or(CheckpointTxg(0), |(_, _, rollback_floor)| rollback_floor);
    let before_without = image_without(image_before_the_remount, lost);
    let after_without = image_without(image_after_the_remount, lost);
    let mut compared = 0;
    let mut compared_with_live_files = 0;
    let mut read_differently = Vec::new();
    // 迭代上界是根环的槽数；没有提前出口。
    for (ring_slot, root) in self_verified_roots_in_the_ring(image_before_the_remount) {
        if root.checkpoint_txg < floor_after || !roots_after.contains(&(ring_slot, root)) {
            continue;
        }
        let Ok(before) = live_files_read_whole_at(&before_without, &root) else {
            continue;
        };
        compared += 1;
        if !before.is_empty() {
            compared_with_live_files += 1;
        }
        if live_files_read_whole_at(&after_without, &root) != Ok(before) {
            read_differently.push(root.checkpoint_txg.0);
        }
    }
    (compared, compared_with_live_files, read_differently)
}

/// 一份镜像去掉一块盘。
pub fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
    MemoryPool {
        devices: image
            .devices
            .iter()
            .filter(|(identity, _)| **identity != lost)
            .map(|(identity, device)| (*identity, device.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 照计划只坏重建那一遍的再挂，看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountUnderThePlan {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
    pub hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
    /// 这一次读坏的恰是计划点名的那几个槽、各一次。
    pub faulted_reads_are_the_named_slots_once: bool,
    /// 交回的盘面与同一份盘面上不注错的再挂交回的逐字节相同。
    pub same_disk_as_a_remount_without_faults: bool,
    /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与崩溃之前同一块盘上读出的相同。
    pub lone_survivor_reads_what_it_read_before_the_crash: [bool; 2],
    /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同
    /// （[`roots_still_in_the_ring_read_on_a_lone_survivor`]）。
    pub lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
}

/// 再挂一次（准入判着、影子账开），读盘经 `root_ring_reads`；重读之前调钩子记下那一刻每个根环槽读过几次。
pub fn remount_through(
    before: &PoolBeforeTheRemount,
    root_ring_reads: &SharedRootRingReads,
) -> (
    Result<Mounted, MountError>,
    MemoryPool,
    Vec<BTreeMap<RootRingSlot, u64>>,
) {
    let mut devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)> =
        plain_devices_on(&before.image_to_remount)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    DeviceFailingTheRebuildPass {
                        identity,
                        inner: device,
                        root_ring_reads: root_ring_reads.clone(),
                    },
                )
            })
            .collect();
    let reads_seen_by_the_hook = root_ring_reads.clone();
    let mut reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>> = Vec::new();
    let mut note_the_reads = || {
        reads_of_each_slot_at_each_hook_call.push(reads_seen_by_the_hook.reads_of_each_slot());
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &before.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut note_the_reads),
    );
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: before.image_to_remount.device_size_in_bytes,
    };
    (mounted, image, reads_of_each_slot_at_each_hook_call)
}

pub fn root_ring_reads_failing(
    image: &MemoryPool,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
) -> SharedRootRingReads {
    root_ring_reads_failing_also_from_a_read_onward(
        image,
        faulted_reads_of_each_slot,
        BTreeMap::new(),
    )
}

/// 同 [`root_ring_reads_failing`]，另有几个槽从它第几次被读起每一次都读坏。
pub fn root_ring_reads_failing_also_from_a_read_onward(
    image: &MemoryPool,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
) -> SharedRootRingReads {
    root_ring_reads_failing_with(
        image,
        faulted_reads_of_each_slot,
        slots_failing_from_a_read_onward,
        FaultsOnARootRingSlotThisProcessWrote::KeepFailing,
    )
}

/// 同 [`root_ring_reads_failing_also_from_a_read_onward`]，另按 `faults_on_a_slot_this_process_wrote` 定写过的槽还坏不坏。
pub fn root_ring_reads_failing_with(
    image: &MemoryPool,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
    faults_on_a_slot_this_process_wrote: FaultsOnARootRingSlotThisProcessWrote,
) -> SharedRootRingReads {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = system_configuration.immutable;
    let slots = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .map(|ring_slot| {
            (
                ring_slot,
                immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing).0,
            )
        })
        .collect();
    SharedRootRingReads(Rc::new(RefCell::new(RootRingReadsState {
        slot_spacing_in_bytes: u64::from(immutable.sizes.fixed_structure_slot_spacing),
        slots,
        faulted_reads_of_each_slot,
        slots_failing_from_a_read_onward,
        reads_of_each_slot: BTreeMap::new(),
        faulted_reads: 0,
        faults_on_a_slot_this_process_wrote,
        slots_this_process_wrote: std::collections::BTreeSet::new(),
        reads_and_writes_in_order: Vec::new(),
    })))
}

pub fn remount_under_the_plan(
    images: &mut PoolCheckerImages,
    starting_point: StartingPoint,
    plan: RebuildPassPlan,
) -> RemountUnderThePlan {
    let before = pool_before_the_remount(starting_point);
    let faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
        faulted_slots_of_the_plan(&before.image_to_remount, plan)
            .into_iter()
            .map(|(ring_slot, fault)| {
                (
                    (
                        ring_slot,
                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
                    ),
                    fault,
                )
            })
            .collect();
    let faulted_slot_count = u64::try_from(faulted_reads_of_each_slot.len()).expect("槽数");
    let root_ring_reads =
        root_ring_reads_failing(&before.image_to_remount, faulted_reads_of_each_slot);
    let (mounted, image_after_the_remount, reads_at_each_hook_call) =
        remount_through(&before, &root_ring_reads);
    let (mounted_without_faults, image_after_a_remount_without_faults, _) = remount_through(
        &before,
        &root_ring_reads_failing(&before.image_to_remount, BTreeMap::new()),
    );
    mounted_without_faults
        .unwrap_or_else(|error| panic!("前提：同一份盘面不注错地再可写挂载做得成：{error:?}"));
    let lone_survivor_reads_what_it_read_before_the_crash = [DeviceIdentity(0), DeviceIdentity(1)]
        .map(|lost| {
            let before_the_crash =
                live_files_read_whole(&image_without(&before.image_before_the_crash, lost));
            before_the_crash.is_ok()
                && live_files_read_whole(&image_without(&image_after_the_remount, lost))
                    == before_the_crash
        });
    let lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount =
        [DeviceIdentity(0), DeviceIdentity(1)].map(|lost| {
            let (_, _, read_differently) = roots_still_in_the_ring_read_on_a_lone_survivor(
                &before.image_to_remount,
                &image_after_the_remount,
                lost,
            );
            read_differently.is_empty()
        });
    // 两块盘都在时池级 checker 不判违例
    images.hand_out(
        "照计划只坏重建那一遍的再挂之后",
        || image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    RemountUnderThePlan {
        root_ring_of_the_rebuilt_allocator: mounted
            .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
            .map_err(|error| format!("{error:?}")),
        hook_calls_after_exactly_the_rebuild_pass: reads_at_each_hook_call
            .iter()
            .map(|reads_of_each_slot| {
                !reads_of_each_slot.is_empty()
                    && reads_of_each_slot
                        .values()
                        .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
            })
            .collect(),
        faulted_reads_are_the_named_slots_once: faulted_slot_count > 0
            && root_ring_reads.faulted_reads() == faulted_slot_count,
        same_disk_as_a_remount_without_faults: image_after_the_remount
            == image_after_a_remount_without_faults,
        lone_survivor_reads_what_it_read_before_the_crash,
        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount,
    }
}

/// 只坏重建那一遍的计划在这几个起点上的预期：照常挂上、重读过一次（钩子恰调一次，在重建那一遍之后），交回与不注错的再挂同一份盘面，
/// 两块盘都在时池级 checker 不判违例，丢盘整读与崩溃之前相同，挂载之前环里还在、不低于 F 的每条自证根丢盘整读与挂载之前相同。
pub fn rereads_and_mounts_like_a_remount_without_faults() -> RemountUnderThePlan {
    RemountUnderThePlan {
        root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
        hook_calls_after_exactly_the_rebuild_pass: vec![true],
        faulted_reads_are_the_named_slots_once: true,
        same_disk_as_a_remount_without_faults: true,
        lone_survivor_reads_what_it_read_before_the_crash: [true, true],
        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
    }
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
/// 预期：盘 0 一条有效根都没读出，整环重读一次（钩子恰调一次），重读读得出，照常可写挂载，交回与不注错的再挂同一份盘面；
/// 两块盘都在时池级 checker 不判违例；丢盘整读与卸载之前相同。
/// 判别力：拿掉「一块盘没读出也重读」（`crates/mutations.tsv`「第10g批 Q4 rr2…」）时 m 取不到、当场只放到环里最旧有效根，
/// 窄盘上写行取不到落点、取号之前拒，第一项红。
pub fn on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ZERO_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 报读错：整环重读、照常挂上"
    );
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
/// 预期与判别力同上一条。
pub fn on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ONE_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 报读错：整环重读、照常挂上"
    );
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`）。
/// 预期与判别力同上（盘 1 一条有效根都没读出）。
pub fn on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上"
    );
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
/// 预期与判别力同上（盘 0 一条有效根都没读出）。
pub fn on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
/// 预期与判别力同第一条。
pub fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ZERO_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 报读错：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
/// 预期与判别力同第一条。
pub fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ONE_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 报读错：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`）。
/// 预期与判别力同第一条。
pub fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
/// 预期与判别力同第一条。
pub fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
    );
}

/// `264:6:4` 那个起点上同一份读坏计划之前，先核它真有可比的：挂载之前丢任一块盘，只读挂载择到的那一版有活文件，环里还在、不低于 F 的根里
/// 也有带活文件的（不然丢盘整读两项比的又是空表）。
pub fn assert_the_live_file_starting_point_has_live_files_to_compare(plan: RebuildPassPlan) {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
    );
    // 迭代上界是两块盘；没有提前出口。
    for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
        let live_files =
            live_files_read_whole(&image_without(&before.image_before_the_crash, lost))
                .unwrap_or_else(|failure| {
                    panic!("前提：丢盘 {} 之后只读挂载、整读做得成：{failure}", lost.0)
                });
        assert!(
            live_files.iter().any(|(_, bytes)| !bytes.is_empty()),
            "前提：丢盘 {} 之后剩下那块盘上有一个不空的活文件（{plan:?}）",
            lost.0
        );
        let (compared, compared_with_live_files, _) =
            roots_still_in_the_ring_read_on_a_lone_survivor(
                &before.image_to_remount,
                &before.image_to_remount,
                lost,
            );
        assert!(
            compared > 0 && compared_with_live_files > 0,
            "前提：丢盘 {} 之后环里不低于 F 的根有 {compared} 条读得出，其中 {compared_with_live_files} 条带活文件",
            lost.0
        );
    }
}

/// 再挂之后接着覆盖写几次（只读挂载择到的那一版之外，环里更旧的有效根引用的单元要在接着写的时候才会被发出去）。264 槽的窄盘上
/// 第 12 次覆盖写抬过 F 仍放不下（`NoSpaceAfterRaisingTheFloor`，现量），取它之前的 10 次。
pub const OVERWRITES_AFTER_THE_REMOUNT: usize = 10;

/// `264:6:4` 那个起点上照计划只坏重建那一遍再挂，之后接着覆盖写 [`OVERWRITES_AFTER_THE_REMOUNT`] 次（准入关着：窄盘扣不起环窗口，起点就是
/// 关着准入造的，落点是唯一的墙）；交回丢盘 0、丢盘 1 时 [`roots_still_in_the_ring_read_on_a_lone_survivor`] 比的（几条、带活文件的几条、
/// 读得不一样的 txg）——挂载之前环里那几条根，写到这一刻仍在环里、不低于 F 的。
pub fn lone_survivors_after_overwrites_following_the_remount(
    plan: RebuildPassPlan,
) -> [(usize, usize, Vec<u64>); 2] {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
    );
    let faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
        faulted_slots_of_the_plan(&before.image_to_remount, plan)
            .into_iter()
            .map(|(ring_slot, fault)| {
                (
                    (
                        ring_slot,
                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
                    ),
                    fault,
                )
            })
            .collect();
    let root_ring_reads =
        root_ring_reads_failing(&before.image_to_remount, faulted_reads_of_each_slot);
    let (mounted, image_after_the_remount, _) = remount_through(&before, &root_ring_reads);
    let mounted = mounted.unwrap_or_else(|error| panic!("照计划再可写挂载做得成：{error:?}"));
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
        parameters: before.parameters.clone(),
        device_bytes: before.image_to_remount.device_size_in_bytes,
        devices: plain_devices_on(&image_after_the_remount),
        session: Some(session),
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 100_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    // 迭代上界是 OVERWRITES_AFTER_THE_REMOUNT；没有提前出口。
    for overwrite_index in 0..OVERWRITES_AFTER_THE_REMOUNT {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    let image_after_the_overwrites = pool.image();
    [DeviceIdentity(0), DeviceIdentity(1)].map(|lost| {
        roots_still_in_the_ring_read_on_a_lone_survivor(
            &before.image_to_remount,
            &image_after_the_overwrites,
            lost,
        )
    })
}

/// 接着覆盖写之后的预期：丢任一块盘，挂载之前环里还在、不低于 F 的根都比到了（至少一条、其中至少一条带活文件），没有一条读得不一样。
pub fn assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
    compared_on_each_lone_survivor: [(usize, usize, Vec<u64>); 2],
) {
    // 迭代上界是两块盘；没有提前出口。
    for (lost, (compared, compared_with_live_files, read_differently)) in
        [0u32, 1].into_iter().zip(compared_on_each_lone_survivor)
    {
        assert!(
            compared > 0 && compared_with_live_files > 0,
            "丢盘 {lost}：环里还在、不低于 F 的根比了 {compared} 条，其中 {compared_with_live_files} 条带活文件（要比到带活文件的根）"
        );
        assert_eq!(
            read_differently,
            Vec::<u64>::new(),
            "丢盘 {lost}：再挂、接着覆盖写之后，挂载之前环里还在、不低于 F 的根在剩下那块盘上整读与挂载之前相同（读得不一样的根的 txg）"
        );
    }
}

/// 场景：`264:6:4`（有活文件，可写挂载做完就崩），重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
/// 预期：整环重读一次、照常挂上，与不注错的再挂同一份盘面；丢任一块盘之后剩下那块盘只读挂载整读与崩溃之前相同（这一回比的是一个不空的活文件），
/// 挂载之前环里还在、不低于 F 的每条自证根丢盘整读也与挂载之前相同；再接着覆盖写 [`OVERWRITES_AFTER_THE_REMOUNT`] 次之后，
/// 挂载之前环里仍在、不低于 F 的根丢盘整读还与挂载之前相同。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10h批 rr2 用例丢盘整读：重读之后的读数只剩最新那条根…」）时
/// 回收门槛与当场放开越过环里更旧的有效根，接着覆盖写把它们引用的单元写掉，接着写之后那一项先红（它排在前面；后面那一项里两块盘都在时
/// 池级 checker 也判 I-3.1 记账少于遍历）。
pub fn on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(DEVICE_ZERO_REPORTS_READ_ERRORS);
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(DEVICE_ZERO_REPORTS_READ_ERRORS),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ZERO_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 0 报读错：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 场景：`264:6:4`，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
/// 预期与判别力同上一条。
pub fn on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(DEVICE_ONE_REPORTS_READ_ERRORS);
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(DEVICE_ONE_REPORTS_READ_ERRORS),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ONE_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 1 报读错：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 场景：`264:6:4`，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`；第三轮辩方 K3：新起点只跑过读错一种）。
/// 预期与判别力同 `…device_zero_failing_the_rebuild_pass…` 那一条（盘 1 一条有效根都没读出，整环重读；变异行
/// 「第10i批 rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根…」）。
pub fn on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(
        DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS,
    );
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS,
        ),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 场景：`264:6:4`，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
/// 预期与判别力同上一条（盘 0 一条有效根都没读出，整环重读）。
pub fn on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(
        DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE,
    );
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE,
        ),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 一份镜像上盘 `device` 那几个区域里最新那条自证过的根住的根环槽。
pub fn ring_slot_of_the_newest_root_on(image: &MemoryPool, device: DeviceIdentity) -> RootRingSlot {
    let region_devices = choose_system_configuration(image)
        .expect("系统配置")
        .immutable
        .region_devices;
    self_verified_roots_in_the_ring(image)
        .into_iter()
        .filter(|(ring_slot, _)| {
            region_devices[usize::try_from(ring_slot.region).expect("区域号")] == device
        })
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .map(|(ring_slot, _)| ring_slot)
        .expect("这块盘上有一条自证过的根")
}

/// 那一槽在重建那一遍（第一遍）之后怎么读坏。
#[derive(Clone, Copy, Debug)]
pub enum ReadsOfTheSlotFailingAfterTheRebuildPass {
    /// 从重读那一遍起每一次都报读错（第三轮攻方 `research/prompts/m3-batch10-code-r3-opus-output.md` 第 1.3 节那一形：只在「从重读起一直坏」上出现）。
    FromTheRereadOnward,
    /// 重读那一遍与写行、暖机之后推的抬 F 算上限读它那一次（[`READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`]）报读错，
    /// 别的读得出（抬 F 那一遍瞬时读坏）。
    OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly,
    /// 只有重读那一遍报读错（对照）。
    OnlyOnTheReread,
}

/// `narrow-384-20-2` 上照 [`ReadsOfTheSlotFailingAfterTheRebuildPass`] 再挂时，盘 0 最新那条根住的槽第几次被读是写行与暖机之后推的
/// 第一次抬 F 算上限读根环那一遍（`mount::roots_of_the_ring_for_the_ceiling`）：现量的（第 10i 批在副本上逐次打出读它的调用栈：
/// 1–5 读阶段，6、7 重建分配器两遍，8、9 取号判最大实例代号，10–15 先写系统配置那几步算生效 F，16 算上限）。挂载里读根环的次序变了，
/// 这个数跟着变：瞬时那条用例另断言这一次读落在暖机最后一段写之后的第一遍读根环里、紧接着当场重读、写之前这一槽还被读过
/// （[`read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once`]），读序一变那一项就红。
pub const READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE: u64 = 16;

/// S1 那一形再挂之后看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
    pub hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
    /// 这次挂载写行与暖机之后推了抬 F（取号之前准入不够）。
    pub floor_raises_pushed_by_the_mount: bool,
    /// 那一槽在重读那一遍之后还被读过（抬 F 那一遍读了它）。
    pub the_slot_was_read_after_the_reread: bool,
    /// 交回的盘面与「那一槽只在重读那一遍报读错」的再挂交回的逐字节相同。
    pub same_disk_as_a_remount_failing_the_slot_only_on_the_reread: bool,
    /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与挂载之前两块盘都在时读出的相同（挂载之前最新那条根只落在盘 0 上，
    /// 那时丢盘 0 读出的是更旧的一版，拿它比不出这次挂载丢没丢东西）。
    pub lone_survivor_reads_what_both_devices_read_before_the_remount: [bool; 2],
    /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同。
    pub lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
    /// 那一槽第 [`READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`] 次读落在它之前最后一段写之后的第一遍读根环里、
    /// 紧接着是同一槽的重读（[`read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once`]）。
    pub the_named_read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once: bool,
}

/// 读写次序里（`ring_slot`，第 `read` 次读）那一次读是不是：① 落在它之前最后一段写之后的第一遍读根环里——那一段写之后的根环槽读照
/// `every_root_ring_slot` 的次序从第一个槽起、一个槽一次地读到它；② 紧接着的一次读是同一槽的重读；③ 重读之后、下一段写之前这一槽还被读过。
/// 挂载里暖机之后推的抬 F，第一遍读根环是准入算上限那一遍（`mount::roots_of_the_ring_for_the_ceiling`，这一类槽读坏当场重读），
/// 之后写之前还有再算一次上限、重算影子账那几遍（③）。别的几遍对不上：先写系统配置那几步算生效 F 的那一遍也是上一次发布的写之后第一遍、
/// 读坏也当场重读，紧接着却是这次发布的写（③ 不成立）；抬 F 那一串里第二次算上限、重算影子账那几遍不是写之后的第一遍（① 不成立）。
pub fn read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once(
    reads_and_writes_in_order: &[Option<(RootRingSlot, u64)>],
    ring_slots_in_order: &[RootRingSlot],
    ring_slot: RootRingSlot,
    read: u64,
) -> bool {
    let Some(position) = reads_and_writes_in_order
        .iter()
        .position(|event| *event == Some((ring_slot, read)))
    else {
        return false;
    };
    let reread_at_once =
        reads_and_writes_in_order.get(position + 1) == Some(&Some((ring_slot, read + 1)));
    let first_event_after_the_last_write = reads_and_writes_in_order[..position]
        .iter()
        .rposition(Option::is_none)
        .map_or(0, |last_write| last_write + 1);
    let slots_read_since_the_last_write: Vec<RootRingSlot> = reads_and_writes_in_order
        [first_event_after_the_last_write..=position]
        .iter()
        .filter_map(|event| event.map(|(slot_read, _)| slot_read))
        .collect();
    let Some(position_in_the_ring) = ring_slots_in_order
        .iter()
        .position(|slot_in_the_ring| *slot_in_the_ring == ring_slot)
    else {
        return false;
    };
    let read_again_before_the_next_write = reads_and_writes_in_order
        .iter()
        .skip(position + 2)
        .take_while(|event| event.is_some())
        .any(|event| event.is_some_and(|(slot_read, _)| slot_read == ring_slot));
    reread_at_once
        && read_again_before_the_next_write
        && slots_read_since_the_last_write == ring_slots_in_order[..=position_in_the_ring]
}

/// `narrow-384-20-2` 上照一份读坏计划再挂一次交回的：挂载的结局、挂载之后的盘面、钩子每次调的那一刻每个根环槽读过几次、
/// 挂载结束时每个根环槽读过几次与读写次序。
pub struct NarrowRemountThroughFaultedReads {
    pub mounted: Result<Mounted, MountError>,
    pub image_after_the_remount: MemoryPool,
    pub reads_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>>,
    pub reads_of_each_slot_by_the_end_of_the_remount: BTreeMap<RootRingSlot, u64>,
    pub reads_and_writes_in_order: Vec<Option<(RootRingSlot, u64)>>,
}

/// `narrow-384-20-2` 上再可写挂载一次（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
/// 盘 0 上的 `slot` 第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
pub fn remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
    before: &PoolBeforeTheRemount,
    slot: RootRingSlot,
    reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
) -> NarrowRemountThroughFaultedReads {
    let mut faulted_reads: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
        faulted_slots_of_the_plan(&before.image_to_remount, DEVICE_ONE_REPORTS_READ_ERRORS)
            .into_iter()
            .map(|(ring_slot, fault)| {
                (
                    (
                        ring_slot,
                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
                    ),
                    fault,
                )
            })
            .collect();
    let mut slots_failing_from_a_read_onward = BTreeMap::new();
    match reads_of_the_slot {
        ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward => {
            slots_failing_from_a_read_onward.insert(
                slot,
                (
                    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
                    RootRingSlotReadFault::ReadFails,
                ),
            );
        }
        ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly => {
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
                RootRingSlotReadFault::ReadFails,
            ));
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
                RootRingSlotReadFault::ReadFails,
            ));
        }
        ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread => {
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
                RootRingSlotReadFault::ReadFails,
            ));
        }
    }
    let root_ring_reads = root_ring_reads_failing_also_from_a_read_onward(
        &before.image_to_remount,
        faulted_reads,
        slots_failing_from_a_read_onward,
    );
    let (mounted, image_after_the_remount, reads_at_each_hook_call) =
        remount_through(before, &root_ring_reads);
    NarrowRemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_at_each_hook_call,
        reads_of_each_slot_by_the_end_of_the_remount: root_ring_reads.reads_of_each_slot(),
        reads_and_writes_in_order: root_ring_reads.reads_and_writes_in_order(),
    }
}

/// `narrow-384-20-2` 上再可写挂载（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
/// 盘 0 最新那条根住的槽第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
pub fn remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
    images: &mut PoolCheckerImages,
    reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
) -> RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
    );
    let slot_of_the_newest_root_on_device_zero =
        ring_slot_of_the_newest_root_on(&before.image_to_remount, DeviceIdentity(0));
    let remount_with =
        |reads_of_the_slot_in_this_remount: ReadsOfTheSlotFailingAfterTheRebuildPass| {
            let NarrowRemountThroughFaultedReads {
                mounted,
                image_after_the_remount,
                reads_at_each_hook_call,
                reads_of_each_slot_by_the_end_of_the_remount,
                reads_and_writes_in_order,
            } = remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
                &before,
                slot_of_the_newest_root_on_device_zero,
                reads_of_the_slot_in_this_remount,
            );
            (
                mounted,
                image_after_the_remount,
                reads_at_each_hook_call,
                reads_of_each_slot_by_the_end_of_the_remount,
                reads_and_writes_in_order,
            )
        };
    let (
        mounted,
        image_after_the_remount,
        reads_at_each_hook_call,
        reads_of_each_slot_by_the_end_of_the_remount,
        reads_and_writes_in_order,
    ) = remount_with(reads_of_the_slot);
    let (mounted_for_reference, image_after_the_reference_remount, _, _, _) =
        remount_with(ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread);
    mounted_for_reference
        .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
    // 两块盘都在时池级 checker 不判违例
    images.hand_out(
        "盘 0 最新那条根两遍里只有一遍自证过的再挂之后",
        || image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
        hook_calls_after_exactly_the_rebuild_pass: reads_at_each_hook_call
            .iter()
            .map(|reads_of_each_slot| {
                !reads_of_each_slot.is_empty()
                    && reads_of_each_slot
                        .values()
                        .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
            })
            .collect(),
        floor_raises_pushed_by_the_mount: mounted
            .as_ref()
            .is_ok_and(|mounted| !mounted.output.space_admission.floor_raises().is_empty()),
        root_ring_of_the_rebuilt_allocator: mounted
            .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
            .map_err(|error| format!("{error:?}")),
        the_slot_was_read_after_the_reread: reads_of_each_slot_by_the_end_of_the_remount
            .get(&slot_of_the_newest_root_on_device_zero)
            .is_some_and(|reads| *reads > READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD),
        same_disk_as_a_remount_failing_the_slot_only_on_the_reread: image_after_the_remount
            == image_after_the_reference_remount,
        lone_survivor_reads_what_both_devices_read_before_the_remount: [
            DeviceIdentity(0),
            DeviceIdentity(1),
        ]
        .map(|lost| {
            live_files_read_whole(&image_without(&image_after_the_remount, lost))
                == live_files_read_whole(&before.image_to_remount)
        }),
        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [
            DeviceIdentity(0),
            DeviceIdentity(1),
        ]
        .map(|lost| {
            let (_, _, read_differently) = roots_still_in_the_ring_read_on_a_lone_survivor(
                &before.image_to_remount,
                &image_after_the_remount,
                lost,
            );
            read_differently.is_empty()
        }),
        the_named_read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once:
            read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once(
                &reads_and_writes_in_order,
                &every_root_ring_slot(&before.parameters.geometry),
                slot_of_the_newest_root_on_device_zero,
                READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
            ),
    }
}

/// 场景：`narrow-384-20-2`，重建那一遍盘 1 每个根环槽报读错（rr2 整环重读），盘 0 最新那条根住的槽第一遍读得出、从重读那一遍起每次都报读错
/// （第三轮攻方第 1.3 节那一形）：按槽并两遍，那一槽并成第一遍读出的根，根环表另记「两遍里只有一遍自证过」那一类。
/// 预期：照常挂上（重读过一次），写行与暖机之后推的抬 F 读那一槽仍坏、重读一次仍坏，照挂载那一刻读坏处置、不拒（用户 2026-10-06 定 S1）；
/// 两块盘都在时池级 checker 不判违例，丢任一块盘剩下那块盘整读与挂载之前两块盘读出的相同，挂载之前环里还在、不低于 F 的根丢盘整读也相同；
/// 交回的盘面与「那一槽只在重读那一遍报读错」的再挂相同（这一形里抬 F 抬到的上限不受那一槽封顶，回收门槛按新 F 取、钉住的最旧 txg 在它之下）。
/// 判别力：改回记成「这个进程知道住着根的槽」（`crates/mutations.tsv`「第10i批 S1 两遍里只有一遍自证过的根改回记成知道住着根的槽」）时
/// 抬 F 重读那一槽仍坏就拒，挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
pub fn a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount(
    images: &mut PoolCheckerImages,
) {
    let remount = remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(images,
        ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward,
    );
    assert_eq!(
        (
            &remount.root_ring_of_the_rebuilt_allocator,
            &remount.hook_calls_after_exactly_the_rebuild_pass,
            remount.floor_raises_pushed_by_the_mount,
            remount.the_slot_was_read_after_the_reread,
            remount.same_disk_as_a_remount_failing_the_slot_only_on_the_reread,
            remount.lone_survivor_reads_what_both_devices_read_before_the_remount,
            remount.lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount,
        ),
        (
            &Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            &vec![true],
            true,
            true,
            true,
            [true, true],
            [true, true],
        ),
        "两遍里只有一遍自证过的根、从重读起一直读坏：照常挂上，抬 F 读它仍坏不拒，checker 不红、丢盘整读不变（{remount:?}）"
    );
}

/// 场景：同上，那一槽只在重读那一遍与写行、暖机之后推的抬 F 算上限读它那一次报读错，之后读得出（抬 F 那一遍瞬时读坏）。
/// 预期：照旧——抬 F 那一遍读它读坏就重读一次、重读读出有效根就照读出来的算，与「那一槽只在重读那一遍报读错」的再挂同一份盘面；
/// checker 不红，丢盘整读不变；点名的那一次读确是暖机最后一段写之后第一遍读根环（准入算上限那一遍）读它、当场重读（正推腿 K3-e：只靠常量时读序一变照样绿）。
/// 判别力：抬 F 算上限那一遍读它读坏一次就拒、不重读（`crates/mutations.tsv`「第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读」）时
/// 挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
pub fn a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(images,
            ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly,
        ),
        RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            hook_calls_after_exactly_the_rebuild_pass: vec![true],
            floor_raises_pushed_by_the_mount: true,
            the_slot_was_read_after_the_reread: true,
            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: true,
            lone_survivor_reads_what_both_devices_read_before_the_remount: [true, true],
            lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
            the_named_read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once: true,
        },
        "两遍里只有一遍自证过的根、之后瞬时读坏一次：抬 F 重读读出来就照读出来的算"
    );
}

/// `narrow-384-20-2` 上第 4 新的那条根住的槽两遍里只有一遍自证过、之后一直读坏时再挂看到的那几样（用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 挂上之后那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_slot_is_self_verified_on_only_one_rebuild_pass: bool,
    /// 那一槽在重读那一遍之后还被读过（挂载推的抬 F 读了它，读坏）。
    pub the_slot_was_read_after_the_reread: bool,
    /// 这次挂载写行与暖机之后推的每一串抬 F 的上限，按推的先后。
    pub ceilings_of_the_floor_raises_pushed_by_the_mount: Vec<CheckpointTxg>,
    /// 挂载之后现行那一版带的 F。
    pub rollback_floor_after_the_mount: Option<CheckpointTxg>,
    /// 交回的盘面与「那一槽只在重读那一遍报读错」（抬 F 读得出它）的再挂交回的逐字节相同。
    pub same_disk_as_a_remount_failing_the_slot_only_on_the_reread: bool,
}

/// 场景：`narrow-384-20-2`（再挂写行、暖机之后推抬 F），重建那一遍盘 1 每个根环槽报读错（rr2 整环重读），环里第 4 新的那条根（txg 21，
/// 住盘 0）住的槽第一遍读得出、从重读那一遍起每次都报读错：它记成两遍里只有一遍自证过、有效的那一类。环里最新的 5 条根各带一版不同的
/// 文件内容（前提里现核），挂载自己写的根不改文件，所以这条根就是抬 F 那一刻「第 4 新的非空持久有效根」。
/// 预期：挂载推的抬 F 算上限那一遍读它仍坏、重读一次仍坏，拿根环表记着的那条根顶上、照它本身算（用户 2026-10-06 定 T1 甲；
/// D16（发布语义） 已定项 1「抬 F 的上限」：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)）：上限恰是这条根的 txg，F 抬到它；
/// 与「那一槽只在重读那一遍报读错」的再挂同一份盘面；两块盘都在时池级 checker 不判违例。
/// 判别力：算上限那一遍读坏重读仍坏的有效根当没有根（`crates/mutations.tsv`「第10m批 T1 甲 算上限那一遍读坏重读仍坏的有效根当没有根」）时
/// 第 4 新的换成 txg 20，上限与 F 都落到 20、盘面跟着不同。K1-3 那条根（txg 4）最旧、上一条 S1 用例那条（txg 24）最新，都罩不住这一处：
/// 少了 txg 24，挂载写的第一条根就与 txg 23 比成非空、顶上它的位置（推的；第 10m 批在副本上逐槽量过这份盘面：
/// 盘 1 读坏时盘 0 上只有 txg 21 这一槽、盘 0 读坏时盘 1 上只有 txg 22 那一槽，在这一变异下上限变）。
pub fn the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise(
    images: &mut PoolCheckerImages,
) {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
    );
    let region_devices = choose_system_configuration(&before.image_to_remount)
        .expect("系统配置")
        .immutable
        .region_devices;
    let mut roots_newest_first = self_verified_roots_in_the_ring(&before.image_to_remount);
    roots_newest_first
        .sort_by_key(|(_, root)| std::cmp::Reverse((root.checkpoint_txg, root.instance)));
    let distinct_contents_of_the_five_newest_roots: std::collections::BTreeSet<
        Vec<(u64, Vec<u8>)>,
    > = roots_newest_first
        .iter()
        .take(5)
        .map(|(_, root)| {
            live_files_read_whole_at(&before.image_to_remount, root)
                .unwrap_or_else(|failure| panic!("前提：最新的 5 条根整读得出：{failure}"))
        })
        .collect();
    let (slot_of_the_fourth_newest_root, fourth_newest_root) = roots_newest_first[3];
    assert_eq!(
        (
            fourth_newest_root.checkpoint_txg,
            region_devices[usize::try_from(slot_of_the_fourth_newest_root.region).expect("区域号")],
            distinct_contents_of_the_five_newest_roots.len(),
        ),
        (CheckpointTxg(21), DeviceIdentity(0), 5),
        "前提：第 4 新的根是 txg 21、住盘 0，最新的 5 条根各带一版不同的文件内容"
    );
    let reference =
        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
            &before,
            slot_of_the_fourth_newest_root,
            ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread,
        );
    let mounted_for_reference = reference
        .mounted
        .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
    assert_eq!(
        mounted_for_reference
            .output
            .space_admission
            .floor_raises()
            .iter()
            .map(|raised| raised.ceiling)
            .collect::<Vec<_>>(),
        vec![fourth_newest_root.checkpoint_txg],
        "前提：抬 F 读得出那一槽的对照再挂只推一串抬 F，上限恰是第 4 新的那条根"
    );
    let remount =
        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
            &before,
            slot_of_the_fourth_newest_root,
            ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward,
        );
    // 两块盘都在时池级 checker 不判违例
    images.hand_out(
        "第 4 新的那条根从重读起一直读坏的再挂之后",
        || remount.image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let mounted = remount.mounted.as_ref();
    assert_eq!(
        RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: mounted
                .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
                .map_err(|error| format!("{error:?}")),
            the_slot_is_self_verified_on_only_one_rebuild_pass: mounted.is_ok_and(|mounted| {
                slot_is_self_verified_on_only_one_rebuild_pass(mounted, slot_of_the_fourth_newest_root)
            }),
            the_slot_was_read_after_the_reread: remount
                .reads_of_each_slot_by_the_end_of_the_remount
                .get(&slot_of_the_fourth_newest_root)
                .is_some_and(|reads| *reads > READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD),
            ceilings_of_the_floor_raises_pushed_by_the_mount: mounted.map_or_else(
                |_| Vec::new(),
                |mounted| {
                    mounted
                        .output
                        .space_admission
                        .floor_raises()
                        .iter()
                        .map(|raised| raised.ceiling)
                        .collect()
                }
            ),
            rollback_floor_after_the_mount: mounted
                .ok()
                .map(|mounted| mounted.current.root().rollback_floor),
            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: remount
                .image_after_the_remount
                == reference.image_after_the_remount,
        },
        RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_was_read_after_the_reread: true,
            ceilings_of_the_floor_raises_pushed_by_the_mount: vec![fourth_newest_root.checkpoint_txg],
            rollback_floor_after_the_mount: Some(fourth_newest_root.checkpoint_txg),
            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: true,
        },
        "第 4 新的非空有效根两遍里只有一遍自证过、之后一直读坏：抬 F 拿记着的那条根顶上，上限与 F 恰是它的 txg"
    );
}

/// K2-2 那组历史停在进程退出那一刻：盘面、参数，被抛弃的根 C、它住的根环槽，与 C 写出的每个落点（盘、槽、那时盘上的字节）。
pub struct PoolWithAnAbandonedRoot {
    pub image: MemoryPool,
    pub parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    pub device_bytes: u64,
    pub write_time_seconds: u64,
    pub abandoned_root: RootRecord,
    pub ring_slot_of_the_abandoned_root: RootRingSlot,
    pub places_the_abandoned_root_wrote: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)>,
}

pub fn bytes_on(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length: usize,
) -> Vec<u8> {
    let (_, block_device) = devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let mut bytes = vec![0u8; length];
    block_device
        .read_at(offset, &mut bytes)
        .expect("内存盘读得出");
    bytes
}

pub fn write_on(
    devices: &mut [(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    bytes: &[u8],
) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    block_device
        .write_at(offset, bytes, WriteDurability::Plain)
        .expect("内存盘写得进");
}

/// 两块盘（盘宽 `device_width`：K2-2 那组历史两档，4 GiB 与单元区 384 槽）：第一个文件、覆盖写 `overwrites_before_the_abandoned_root` 次
/// （实例 1；K2-2 两档，1 次与 3 次）；崩了再挂（实例 2），覆盖写一次得 C；照 `tests/common` 的
/// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形抛弃 C——C 的根槽与数据单元暂存、清零，见证 C 的那一次系统配置轮换
/// 写回成轮换之前的那一代，崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机，再把暂存的字节写回；进程退出。
pub fn pool_with_an_abandoned_root(
    device_width: HistoryDeviceWidth,
    overwrites_before_the_abandoned_root: usize,
) -> PoolWithAnAbandonedRoot {
    let mut pool = start_plain(device_width);
    // 迭代上界是 overwrites_before_the_abandoned_root；没有提前出口。
    for overwrite_index in 0..overwrites_before_the_abandoned_root {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("实例 1 第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
    pool.overwrite(OVERWRITE_BYTES - 499)
        .expect("实例 2 覆盖写一次得 C");
    let abandoned = pool.current_file_version().clone();
    let geometry = pool.parameters.geometry;
    let ring_slot_of_the_abandoned_root = target_for_publish(
        abandoned.root.checkpoint_txg,
        geometry.root_ring_slots_per_region,
    );
    let root_slot_device = pool.parameters.region_devices
        [usize::try_from(ring_slot_of_the_abandoned_root.region).expect("区域号")];
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let places_to_hide: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = std::iter::once((
        root_slot_device,
        slot_offset(
            ring_slot_of_the_abandoned_root,
            geometry.fixed_structure_slot_spacing,
        ),
        usize::try_from(geometry.physical_block_size).expect("根槽宽"),
    ))
    .chain(
        abandoned
            .data_pointers
            .iter()
            .flat_map(|pointer| pointer.locations)
            .map(|location| {
                (
                    location.device,
                    location.slot.to_device_offset(),
                    data_unit_bytes,
                )
            }),
    )
    .collect();
    let saved: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = places_to_hide
        .iter()
        .map(|(device, offset, length)| {
            let bytes = bytes_on(&pool.devices, *device, *offset, *length);
            write_on(&mut pool.devices, *device, *offset, &vec![0u8; *length]);
            (*device, *offset, bytes)
        })
        .collect();
    // 见证 C 的那一次轮换（每块盘世代号最大的那一槽）写回成轮换之前的那一代：另一槽的内容、世代号减 2（与 `tests/common` 那一形同一个造法）。
    let slot_spacing_in_bytes = u64::from(geometry.fixed_structure_slot_spacing);
    let rotations_not_written: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = verified_system_configuration_slots(
                pool.devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &pool.parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] =
                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
            let newest_generation = newest.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("C 之前至少还有两次轮换");
            (
                *identity,
                DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    // 迭代上界是两块盘；没有提前出口。
    for (device, offset, bytes) in &rotations_not_written {
        write_on(&mut pool.devices, *device, *offset, bytes);
    }
    let abandoning = pool
        .crash_and_mount_writable()
        .expect("C 的根槽与数据单元读不出：择根落到 C 之前那一条，照常可写挂载（实例 3）");
    assert!(
        abandoning.effective_root.checkpoint_txg < abandoned.root.checkpoint_txg,
        "前提：实例 3 落到 C 之前"
    );
    // 迭代上界是暂存的落点数；没有提前出口。
    for (device, offset, bytes) in &saved {
        write_on(&mut pool.devices, *device, *offset, bytes);
    }
    pool.session = None;
    let mut places_the_abandoned_root_wrote: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)> =
        Vec::new();
    // 迭代上界是 C 写出的单元数 × 两块盘；没有提前出口。
    for unit in &abandoned.units {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let on_disk = bytes_on(
                &pool.devices,
                device,
                unit.slot.to_device_offset(),
                unit.bytes.len(),
            );
            if on_disk == unit.bytes {
                places_the_abandoned_root_wrote.push((device, unit.slot, on_disk));
            }
        }
    }
    // 迭代上界是 C 的数据指针数 × 两个位置；没有提前出口。
    for location in abandoned
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
    {
        if !places_the_abandoned_root_wrote
            .iter()
            .any(|(device, slot, _)| *device == location.device && *slot == location.slot)
        {
            let on_disk = bytes_on(
                &pool.devices,
                location.device,
                location.slot.to_device_offset(),
                data_unit_bytes,
            );
            places_the_abandoned_root_wrote.push((location.device, location.slot, on_disk));
        }
    }
    PoolWithAnAbandonedRoot {
        image: pool.image(),
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        write_time_seconds: pool.write_time_seconds,
        abandoned_root: abandoned.root,
        ring_slot_of_the_abandoned_root,
        places_the_abandoned_root_wrote,
    }
}

/// K2-2 那组历史上再可写挂载一次、之后接着覆盖写到 C 离开根环，看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountOfThePoolWithAnAbandonedRoot {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
    pub hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
    /// 读坏的恰是点名的那几次、各一次。
    pub faulted_reads_are_the_named_ones: bool,
    /// 交回的盘面与同一份盘面上照对照的读坏计划再挂交回的逐字节相同。
    pub same_disk_as_the_reference_remount: bool,
    /// 挂上那一刻两块盘都隔离了只被被抛弃根引用的槽（影子账不为 0）。
    pub both_devices_isolate_slots_after_the_mount: bool,
    /// 挂上那一刻 C 写出的落点没有一个在分配器里是空闲的。
    pub no_place_of_the_abandoned_root_is_free_after_the_mount: bool,
    /// 接着覆盖写到 C 的根槽被盖掉（C 离开根环）为止，每次发布之后 C 写出的每个落点在盘上仍是 C 写的字节。
    pub places_of_the_abandoned_root_kept_until_it_leaves_the_ring: bool,
}

/// 照一份读坏计划再挂一次交回的：挂载的结局、挂载之后的盘面、钩子每次调的那一刻每个根环槽读过几次、这一次读坏了几次。
pub struct RemountThroughFaultedReads {
    pub mounted: Result<Mounted, MountError>,
    pub image_after_the_remount: MemoryPool,
    pub reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>>,
    pub faulted_reads: u64,
}

/// 照 `faulted_reads_of_each_slot` 读坏地再可写挂载一次（准入判着、影子账开），重读之前调钩子记下那一刻每个根环槽读过几次。
pub fn remount_the_pool_with_an_abandoned_root(
    history: &PoolWithAnAbandonedRoot,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
) -> RemountThroughFaultedReads {
    let root_ring_reads = root_ring_reads_failing(&history.image, faulted_reads_of_each_slot);
    let before = PoolBeforeTheRemount {
        image_before_the_crash: history.image.clone(),
        image_to_remount: history.image.clone(),
        parameters: history.parameters.clone(),
    };
    let (mounted, image_after_the_remount, reads_of_each_slot_at_each_hook_call) =
        remount_through(&before, &root_ring_reads);
    RemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_of_each_slot_at_each_hook_call,
        faulted_reads: root_ring_reads.faulted_reads(),
    }
}

/// 根环里这一槽这一刻是不是还住着 C。
pub fn abandoned_root_still_in_the_ring(
    image: &MemoryPool,
    history: &PoolWithAnAbandonedRoot,
) -> bool {
    self_verified_roots_in_the_ring(image).contains(&(
        history.ring_slot_of_the_abandoned_root,
        history.abandoned_root,
    ))
}

/// 在 K2-2 那组历史上照 `faulted_reads_of_each_slot` 再挂、接着覆盖写到 C 离开根环；盘面拿 `reference_faulted_reads_of_each_slot` 那一次再挂比。
pub fn remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
    history: &PoolWithAnAbandonedRoot,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    reference_faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
) -> RemountOfThePoolWithAnAbandonedRoot {
    let named_faulted_reads = u64::try_from(faulted_reads_of_each_slot.len()).expect("次数");
    let RemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_of_each_slot_at_each_hook_call: reads_at_each_hook_call,
        faulted_reads,
    } = remount_the_pool_with_an_abandoned_root(history, faulted_reads_of_each_slot);
    let reference =
        remount_the_pool_with_an_abandoned_root(history, reference_faulted_reads_of_each_slot);
    let image_after_the_reference_remount = reference.image_after_the_remount;
    reference
        .mounted
        .unwrap_or_else(|error| panic!("前提：照对照的读坏计划再可写挂载做得成：{error:?}"));
    let hook_calls_after_exactly_the_rebuild_pass: Vec<bool> = reads_at_each_hook_call
        .iter()
        .map(|reads_of_each_slot| {
            !reads_of_each_slot.is_empty()
                && reads_of_each_slot
                    .values()
                    .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
        })
        .collect();
    let faulted_reads_are_the_named_ones =
        named_faulted_reads > 0 && faulted_reads == named_faulted_reads;
    let same_disk_as_the_reference_remount =
        image_after_the_remount == image_after_the_reference_remount;
    let mounted = match mounted {
        Ok(mounted) => mounted,
        Err(error) => {
            return RemountOfThePoolWithAnAbandonedRoot {
                root_ring_of_the_rebuilt_allocator: Err(format!("{error:?}")),
                hook_calls_after_exactly_the_rebuild_pass,
                faulted_reads_are_the_named_ones,
                same_disk_as_the_reference_remount,
                both_devices_isolate_slots_after_the_mount: false,
                no_place_of_the_abandoned_root_is_free_after_the_mount: false,
                places_of_the_abandoned_root_kept_until_it_leaves_the_ring: false,
            };
        }
    };
    let root_ring_of_the_rebuilt_allocator =
        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
    let both_devices_isolate_slots_after_the_mount = mounted.output.isolated_slots_per_device.len()
        == 2
        && mounted
            .output
            .isolated_slots_per_device
            .iter()
            .all(|(_, isolated_slots)| *isolated_slots > 0);
    let no_place_of_the_abandoned_root_is_free_after_the_mount = history
        .places_the_abandoned_root_wrote
        .iter()
        .all(|(device, slot, _)| {
            !mounted.allocator.devices[usize::try_from(device.0).expect("盘号")].is_free(*slot)
        });
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices: plain_devices_on(&image_after_the_remount),
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let root_ring_slot_count = every_root_ring_slot(&history.parameters.geometry).len();
    let mut places_of_the_abandoned_root_kept_until_it_leaves_the_ring = true;
    let mut overwrites = 0usize;
    // 迭代上界是根环的槽数（再挂写了几条根，C 的根槽在一圈之内被盖掉）；提前出口：C 的哪个落点被改写了就停。
    while abandoned_root_still_in_the_ring(&pool.image(), history) {
        assert!(
            overwrites < root_ring_slot_count,
            "C 的根槽在一圈（{root_ring_slot_count} 次发布）之内被盖掉"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrites} 次覆盖写：{refusal:?}"));
        overwrites += 1;
        if history
            .places_the_abandoned_root_wrote
            .iter()
            .any(|(device, slot, bytes)| {
                bytes_on(&pool.devices, *device, slot.to_device_offset(), bytes.len()) != *bytes
            })
        {
            places_of_the_abandoned_root_kept_until_it_leaves_the_ring = false;
            break;
        }
    }
    // C 离开根环那一刻两块盘都在时池级 checker 不判违例
    images.hand_out(
        "接着覆盖写到 C 离开根环之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    RemountOfThePoolWithAnAbandonedRoot {
        root_ring_of_the_rebuilt_allocator,
        hook_calls_after_exactly_the_rebuild_pass,
        faulted_reads_are_the_named_ones,
        same_disk_as_the_reference_remount,
        both_devices_isolate_slots_after_the_mount,
        no_place_of_the_abandoned_root_is_free_after_the_mount,
        places_of_the_abandoned_root_kept_until_it_leaves_the_ring,
    }
}

/// 盘 1 那几个区域的每个根环槽。
pub fn root_ring_slots_of_device_one(history: &PoolWithAnAbandonedRoot) -> Vec<RootRingSlot> {
    every_root_ring_slot(&history.parameters.geometry)
        .into_iter()
        .filter(|ring_slot| {
            history.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
                == DeviceIdentity(1)
        })
        .collect()
}

/// 每个点名的槽在第 `read_of_each_slot` 次读时读成 `fault`。
pub fn faulted_reads_of(
    ring_slots: &[RootRingSlot],
    read_of_each_slot: u64,
    fault: RootRingSlotReadFault,
) -> BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> {
    ring_slots
        .iter()
        .map(|ring_slot| ((*ring_slot, read_of_each_slot), fault))
        .collect()
}

/// 并两遍之后的预期：重读过一次（钩子恰调一次，在重建那一遍之后）、照常挂上，与对照那一次再挂同一份盘面；两块盘都隔离、C 的落点一个都不空闲，
/// 接着覆盖写到 C 离开根环之前 C 写出的落点一个都没被改写，那一刻池级 checker 不判违例。
pub fn merged_and_keeps_the_abandoned_root_isolated() -> RemountOfThePoolWithAnAbandonedRoot {
    RemountOfThePoolWithAnAbandonedRoot {
        root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
        hook_calls_after_exactly_the_rebuild_pass: vec![true],
        faulted_reads_are_the_named_ones: true,
        same_disk_as_the_reference_remount: true,
        both_devices_isolate_slots_after_the_mount: true,
        no_place_of_the_abandoned_root_is_free_after_the_mount: true,
        places_of_the_abandoned_root_kept_until_it_leaves_the_ring: true,
    }
}

/// 第一遍盘 1 整段报读错、重读只坏 C 的槽（读成 `fault`）的那一格（攻方 `pass1:dev1-all-e reread:c-*`）。
pub fn first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
    images: &mut PoolCheckerImages,
    device_width: HistoryDeviceWidth,
    overwrites_before_the_abandoned_root: usize,
    fault: RootRingSlotReadFault,
) -> RemountOfThePoolWithAnAbandonedRoot {
    let history = pool_with_an_abandoned_root(device_width, overwrites_before_the_abandoned_root);
    let mut faulted_reads = faulted_reads_of(
        &root_ring_slots_of_device_one(&history),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &[history.ring_slot_of_the_abandoned_root],
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        fault,
    ));
    remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
        images,
        &history,
        faulted_reads,
        BTreeMap::new(),
    )
}

/// 第一遍盘 1 每个根环槽与 C 的根槽都报读错、重读那一遍都读得出的那一格（K2-2 的反方向，攻方 `pass1:dev1-all-e+c-e reread:-`）。
pub fn first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
    images: &mut PoolCheckerImages,
    device_width: HistoryDeviceWidth,
    overwrites_before_the_abandoned_root: usize,
) -> RemountOfThePoolWithAnAbandonedRoot {
    let history = pool_with_an_abandoned_root(device_width, overwrites_before_the_abandoned_root);
    let mut faulted_reads = faulted_reads_of(
        &root_ring_slots_of_device_one(&history),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &[history.ring_slot_of_the_abandoned_root],
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    ));
    remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
        images,
        &history,
        faulted_reads,
        BTreeMap::new(),
    )
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍只有 C 的根槽报读错。
/// 预期：按槽并两遍，C 那一槽留第一遍读出的 C——影子账照常隔离 C 独占的单元，盘面与不注错的再挂相同；接着覆盖写到 C 离开根环，
/// C 写出的落点一个都没被改写，池级 checker 不红（D23（journal 的角色与格式） 已定项 14「被崩溃恢复抛弃的时间线的根离开根环之前，
/// 它们引用的单元不许重新分配也不许抹头」）。
/// 判别力：改回整份换成重读的读数（`crates/mutations.tsv`「第10h批 R1 改回整份换成重读的读数」）时 C 掉出影子账、两块盘隔离 0，
/// 第一次覆盖写就复用 C 的单元（攻方量过 I-7.4 红），隔离那一项起红；「第10h批 R1 第一遍自证过压不过重读读坏」同样红。
pub fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：同上，重读那一遍 C 的根槽读回全 0。
/// 预期与判别力同上一条（第一遍自证过压过重读全零）。
pub fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：同上，重读那一遍 C 的根槽读回乱字节。
/// 预期与判别力同上一条。
pub fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（攻方 `pass1:dev1-all-e+c-e reread:-`；单元区 384 槽、实例 1 覆盖写 1 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出。
/// 预期：按槽并两遍，C 那一槽取重读读出的 C，与不注错的再挂同一份盘面，影子账照隔离，C 离开根环之前它的落点不被改写，池级 checker 不红。
/// 判别力：并的时候第一遍读坏压过重读自证过的根（`crates/mutations.tsv`「第10h批 R1 第一遍读坏压过重读自证过的根」）时 C 掉出影子账，
/// 隔离那一项起红。
pub fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（单元区 384 槽、实例 1 覆盖写 3 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 定案补的两维）。
/// 预期同 `the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力同上一条（同一行变异的另一个名字：「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）。
pub fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（4 GiB 盘、实例 1 覆盖写 1 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 定案补的两维）。
/// 预期同 `the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力同上一条（同一行变异的另一个名字：「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）。
pub fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 定案补的盘宽与前置覆盖写两维，
/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（4 GiB 盘、实例 1 覆盖写 3 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 定案补的两维）。
/// 预期同 `the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力同上一条（同一行变异的另一个名字：「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）。
pub fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍整个环都报读错（攻方 `pass1:dev1-all-e reread:all-e`，判决 K2「多拒」那一格）。
/// 预期：按槽并两遍，第一遍盘 0 读出的根都留着，照常挂上（不是 Q3 那一拒），与「两遍都只坏盘 1」的再挂同一份盘面（两份并出来的读数相同：
/// 盘 0 读出、盘 1 读坏，照 m 取不到那一支往下走）；影子账照隔离 C，C 离开根环之前它的落点不被改写，池级 checker 不红。
/// 判别力：改回整份换成重读的读数时重读那一遍一条有效根都没有，拒成 `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，第一项红。
pub fn a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 1);
    let ring_slots_of_device_one = root_ring_slots_of_device_one(&history);
    let mut faulted_reads = faulted_reads_of(
        &ring_slots_of_device_one,
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &every_root_ring_slot(&history.parameters.geometry),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        RootRingSlotReadFault::ReadFails,
    ));
    let mut faulted_reads_only_on_device_one = faulted_reads_of(
        &ring_slots_of_device_one,
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads_only_on_device_one.extend(faulted_reads_of(
        &ring_slots_of_device_one,
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        RootRingSlotReadFault::ReadFails,
    ));
    assert_eq!(
        remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
            images,
            &history,
            faulted_reads,
            faulted_reads_only_on_device_one,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "重读整环都坏、第一遍读出了盘 0 的根：并两遍照常挂上"
    );
}

impl DeviceOverASparseImage for DeviceFailingTheRebuildPass {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

/// K2-2 那组历史里盘 `device` 那几个区域上每条不被抛弃的自证根住的槽（这组历史里被抛弃的只有 C，[`pool_with_an_abandoned_root`]）。
pub fn ring_slots_of_the_roots_not_abandoned_on(
    history: &PoolWithAnAbandonedRoot,
    device: DeviceIdentity,
) -> Vec<RootRingSlot> {
    self_verified_roots_in_the_ring(&history.image)
        .into_iter()
        .filter(|(ring_slot, _)| {
            history.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
                == device
                && *ring_slot != history.ring_slot_of_the_abandoned_root
        })
        .map(|(ring_slot, _)| ring_slot)
        .collect()
}

/// 重建那一遍读得出的那一槽之后怎么读坏（第 10i 批代码轮攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs` 的 `s1_plans`
/// 里 a、c 两形）。
#[derive(Clone, Copy, Debug)]
pub enum ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass {
    /// a：从重读那一遍起每一次都读成这一样。
    FailingFromTheRereadOnward(RootRingSlotReadFault),
    /// c：从重读那一遍起每三次读里前两次读成这一样、第三次读得出（算上限那一遍读得出、抬 F 那一遍读它两次都读坏的那一形）。
    FailingTwoReadsInEveryThreeFromTheRereadOn(RootRingSlotReadFault),
}

/// 交替读坏那一形点名到重建那一遍之后第几次读为止：远多于一次挂载加 [`USER_CHANGES_AFTER_THE_REMOUNT`] 次改动读那一槽的次数。
pub const READS_NAMED_BY_THE_ALTERNATING_FAULT: u64 = 400;

/// 再挂之后接着发几次改动（攻方 K1-1、K1-2、K1-3 同一个数）。
pub const USER_CHANGES_AFTER_THE_REMOUNT: usize = 40;

/// 再挂之后接着发的用户改动。
#[derive(Clone, Copy, Debug)]
pub enum UserChangesAfterTheRemount {
    /// 每次覆盖写一次。
    Overwrites,
    /// 第 `first_sequential_write` 次之前覆盖写，从它起每次顺序写 8 个数据单元（攻方 K1-1 那一档）。
    OverwritesThenSequentialWritesOfEightDataUnitsFrom { first_sequential_write: usize },
}

/// 一次可写挂载交回的盘留着读坏的计划接着用：挂载的结局、挂载之后的盘、读根环的那一份计数。
pub struct RemountKeepingTheFaultedDevices {
    pub mounted: Result<Mounted, MountError>,
    pub devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)>,
    pub root_ring_reads: SharedRootRingReads,
}

/// K2-2 那组历史上再可写挂载（准入判着、影子账开）：重建那一遍盘 `device_failing_the_rebuild_pass` 上每条不被抛弃的根住的槽报读错
/// （那块盘一条有效根都没读出，rr2 整环重读），`slot` 第一遍读得出、之后照 `reads_of_the_slot` 读坏；这个进程写过的槽不再读坏。
pub fn remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
    history: &PoolWithAnAbandonedRoot,
    device_failing_the_rebuild_pass: DeviceIdentity,
    slot: RootRingSlot,
    reads_of_the_slot: ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass,
) -> RemountKeepingTheFaultedDevices {
    let mut faulted_reads = faulted_reads_of(
        &ring_slots_of_the_roots_not_abandoned_on(history, device_failing_the_rebuild_pass)
            .into_iter()
            .filter(|ring_slot| *ring_slot != slot)
            .collect::<Vec<_>>(),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    let mut slots_failing_from_a_read_onward = BTreeMap::new();
    match reads_of_the_slot {
        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(fault) => {
            slots_failing_from_a_read_onward
                .insert(slot, (READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD, fault));
        }
        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingTwoReadsInEveryThreeFromTheRereadOn(
            fault,
        ) => {
            // 迭代上界是点名的读数；没有提前出口。
            for read in READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD
                ..READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD
                    + READS_NAMED_BY_THE_ALTERNATING_FAULT
            {
                if !(read - READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD).is_multiple_of(3) {
                    faulted_reads.insert((slot, read), fault);
                }
            }
        }
    }
    let root_ring_reads = root_ring_reads_failing_with(
        &history.image,
        faulted_reads,
        slots_failing_from_a_read_onward,
        FaultsOnARootRingSlotThisProcessWrote::StopFailing,
    );
    let mut devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)> =
        plain_devices_on(&history.image)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    DeviceFailingTheRebuildPass {
                        identity,
                        inner: device,
                        root_ring_reads: root_ring_reads.clone(),
                    },
                )
            })
            .collect();
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &history.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::RereadImmediately,
    );
    RemountKeepingTheFaultedDevices {
        mounted,
        devices,
        root_ring_reads,
    }
}

/// 挂上之后那张根环表里 `slot` 是不是记成重建分配器两遍里只有一遍自证过的那一类。
pub fn slot_is_self_verified_on_only_one_rebuild_pass(
    mounted: &Mounted,
    slot: RootRingSlot,
) -> bool {
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
        &mounted.allocator,
    )
    .contains_key(&slot)
}

/// 再挂、接着发改动之后看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 挂上之后那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_slot_is_self_verified_on_only_one_rebuild_pass: bool,
    /// 挂上之后那一槽还读坏过（读坏落在抬 F 那一串上，不只落在挂载里）。
    pub the_slot_read_bad_after_the_mount: bool,
    /// 做成了几次改动。
    pub changes_published: usize,
    /// 被拒的那一次改动报的错（没被拒是 `None`）。
    pub refusal: Option<String>,
    /// C 还在根环里时 F 抬过了 C 的 txg（之后的回收与影子账按新 F 算，C 引用、挂载那一刻被候选根豁免的单元要补隔离）。
    pub floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: bool,
}

/// 照 [`remount_with_a_slot_self_verified_only_on_the_rebuild_pass`] 再挂，盘留着读坏的计划，接着发 [`USER_CHANGES_AFTER_THE_REMOUNT`] 次
/// `user_changes`（被拒就停），每次之后两块盘都在时跑池级 checker。
pub fn changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
    images: &mut PoolCheckerImages,
    history: &PoolWithAnAbandonedRoot,
    device_failing_the_rebuild_pass: DeviceIdentity,
    slot: RootRingSlot,
    reads_of_the_slot: ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass,
    user_changes: UserChangesAfterTheRemount,
) -> ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
    let RemountKeepingTheFaultedDevices {
        mounted,
        devices,
        root_ring_reads,
    } = remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
        history,
        device_failing_the_rebuild_pass,
        slot,
        reads_of_the_slot,
    );
    let mounted = match mounted {
        Ok(mounted) => mounted,
        Err(error) => {
            return ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
                root_ring_of_the_rebuilt_allocator: Err(format!("{error:?}")),
                the_slot_is_self_verified_on_only_one_rebuild_pass: false,
                the_slot_read_bad_after_the_mount: false,
                changes_published: 0,
                refusal: None,
                floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: false,
            };
        }
    };
    let faulted_reads_by_the_end_of_the_mount = root_ring_reads.faulted_reads();
    let root_ring_of_the_rebuilt_allocator =
        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
    let the_slot_is_self_verified_on_only_one_rebuild_pass =
        slot_is_self_verified_on_only_one_rebuild_pass(&mounted, slot);
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<DeviceFailingTheRebuildPass> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut changes_published = 0;
    let mut refusal = None;
    let mut floor_raised_above_the_abandoned_root_while_it_was_in_the_ring = false;
    // 迭代上界是 USER_CHANGES_AFTER_THE_REMOUNT；跨轮携带做成的次数与第一次违例；提前出口只有改动被拒。
    for change in 1..=USER_CHANGES_AFTER_THE_REMOUNT {
        let published = match user_changes {
            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
                first_sequential_write,
            } if change >= first_sequential_write => pool
                .sequential_write(8 * data_unit_payload_capacity() + 100)
                .map(|_| ()),
            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
                ..
            }
            | UserChangesAfterTheRemount::Overwrites => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
        };
        if let Err(refused) = published {
            refusal = Some(format!("{refused:?}"));
            break;
        }
        changes_published = change;
        let image = pool.image();
        if pool.rollback_floor() > history.abandoned_root.checkpoint_txg
            && abandoned_root_still_in_the_ring(&image, history)
        {
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring = true;
        }
        // 每次改动之后两块盘都在时池级 checker 不判违例
        images.hand_out(
            &format!("再挂之后第 {change} 次改动之后"),
            || image,
            PoolCheckerExpectation::no_invariant_violated(),
        );
    }
    ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
        root_ring_of_the_rebuilt_allocator,
        the_slot_is_self_verified_on_only_one_rebuild_pass,
        the_slot_read_bad_after_the_mount: root_ring_reads.faulted_reads()
            > faulted_reads_by_the_end_of_the_mount,
        changes_published,
        refusal,
        floor_raised_above_the_abandoned_root_while_it_was_in_the_ring,
    }
}

/// 场景：攻方 K1-1 那一形（`research/prompts/m3-batch10i-code-r1-opus-output.md`「K1-1」一节；原型 `abandoned_start(UnitAreaOf384Slots, 3, 0)`、
/// 计划 `s1-A@0/3(d0)-trig0-c-Z`）：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），再可写挂载时重建那一遍盘 0 每条不被抛弃的根住的槽报读错
/// （rr2 整环重读），被抛弃的根 C 那一槽第一遍读得出、从重读起每三次读里前两次读回全 0：并两遍时 C 记成两遍里只有一遍自证过的那一类。
/// 挂上之后覆盖写 5 次、从第 6 次起每次顺序写 8 个数据单元，共 40 次；准入抬 F 时算上限那一遍读 C 那一槽读得出，抬 F 那一遍读它两次都读坏。
/// 预期：抬 F 那一遍拿根环表记着的 C 顶上（用户 2026-10-06 定 T1 甲），按新 F 重算的影子账照隔离 C 引用、挂载那一刻被候选根豁免的单元：
/// 40 次都做成，每次之后池级 checker 都不判违例；C 还在环里时 F 就抬过了 C 的 txg（这一形的前提）。
/// 改之前（第 10i 批：抬 F 那一遍读坏重读仍坏的这一类只钉回收门槛、不进交回的根表）在副本上实测：第 9 次改动之后（F = 17、txg 24）
/// checker 判 I-7.4「被抛弃的根（实例 2、txg 9）引用的单元已被重新分配或抹头……实例表单元 在盘 0 槽 1536 的那一份与位置条目里的校验和对不上」。
/// 判别力：抬 F 那一遍读坏重读仍坏的这一类不拿记着的根顶上（`crates/mutations.tsv`「第10l批 T1 甲 抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上」）。
pub fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    assert_eq!(
        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(images,
            &history,
            DeviceIdentity(0),
            history.ring_slot_of_the_abandoned_root,
            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingTwoReadsInEveryThreeFromTheRereadOn(
                RootRingSlotReadFault::ReadsBackAllZero,
            ),
            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
                first_sequential_write: 6,
            },
        ),
        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_read_bad_after_the_mount: true,
            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
            refusal: None,
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
        },
        "被抛弃的根两遍里只有一遍自证过、抬 F 那一遍读它读坏：拿记着的根顶上，影子账照隔离，checker 不红"
    );
}

/// 场景：攻方 K1-2 那一形：同一组历史，C 那一槽第一遍读得出、从重读起一直报读错（计划 a），挂上之后覆盖写 40 次。
/// 预期：算上限那一遍拿记着的 C 顶上，按实例表判被抛弃、不进上限（用户 2026-10-06 定 T1 甲）：F 抬得过 C 的 txg，40 次都做成，checker 每次都不判违例。
/// 改之前（第 10i 批：照挂载那一刻读坏按能住的最大 txg 并进上限）在副本上实测：F 卡在 C 的 txg 9，第 15 次覆盖写报 `NoSpaceAfterRaisingTheFloor`。
/// 判别力：算上限那一遍读它重读仍坏照拒（`crates/mutations.tsv`「第10l批 T1 甲 算上限那一遍读坏重读仍坏的这一类照拒」）时
/// 抬 F 报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`、改动被拒。
pub fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    assert_eq!(
        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
            images,
            &history,
            DeviceIdentity(0),
            history.ring_slot_of_the_abandoned_root,
            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
                RootRingSlotReadFault::ReadFails,
            ),
            UserChangesAfterTheRemount::Overwrites,
        ),
        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(
                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
            ),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_read_bad_after_the_mount: true,
            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
            refusal: None,
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
        },
        "被抛弃的根两遍里只有一遍自证过、之后一直读坏：不卡上限，40 次覆盖写都做成"
    );
}

/// 场景：攻方 K1-3 那一形：同一组历史，重建那一遍盘 0 每条不被抛弃的根住的槽报读错（rr2 整环重读），盘 1 上 txg 4 那条有效根住的槽
/// （区域 1、槽 1）第一遍读得出、从重读起一直报读错：它记成两遍里只有一遍自证过、有效的那一类。挂上之后覆盖写 40 次。
/// 预期：算上限、抬 F 那一串读它读坏、重读仍坏时拿记着的那条根顶上、照它本身算（用户 2026-10-06 定 T1 甲）：40 次都做成，checker 每次都不判违例。
/// 改之前（第 10i 批：照挂载那一刻读坏按它能住的最大 txg 并进上限）在副本上实测：第 12 次覆盖写报 `NoSpaceAfterRaisingTheFloor`
/// （`FloorAlreadyAtTheCeiling { floor: 4, ceiling: 4 }`）；同一组历史上别的 10 个住有效根的槽（盘 0 读坏时盘 1 上的、盘 1 读坏时盘 0 上的）
/// 改之前 7 个在第 12–16 次覆盖写上停在同一种拒、txg 0–2 那 3 个做满 40 次，改之后 10 个都做满 40 次、checker 不红。
/// 判别力：算上限那一遍读它重读仍坏照拒（`crates/mutations.tsv`「第10l批 T1 甲 算上限那一遍读坏重读仍坏的这一类照拒（有效根那一槽）」）时改动被拒。
pub fn a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    let slot_of_the_valid_root_of_txg_four = RootRingSlot { region: 1, slot: 1 };
    assert_eq!(
        self_verified_roots_in_the_ring(&history.image)
            .into_iter()
            .find(|(ring_slot, _)| *ring_slot == slot_of_the_valid_root_of_txg_four)
            .map(|(_, root)| root.checkpoint_txg),
        Some(CheckpointTxg(4)),
        "前提：这组历史里区域 1、槽 1 住着 txg 4 那条有效根"
    );
    assert_eq!(
        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(images,
            &history,
            DeviceIdentity(0),
            slot_of_the_valid_root_of_txg_four,
            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
                RootRingSlotReadFault::ReadFails,
            ),
            UserChangesAfterTheRemount::Overwrites,
        ),
        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_read_bad_after_the_mount: true,
            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
            refusal: None,
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
        },
        "有效根两遍里只有一遍自证过、之后一直读坏：照记着的那条根算上限，40 次覆盖写都做成、checker 不红"
    );
}

/// K2-2 那组历史上再挂、覆盖写几次、再正常卸载之后看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂上之后 C 那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: bool,
    /// 卸载之前的覆盖写做成了几次。
    pub overwrites_published: usize,
    /// 卸载的结局（错只记成员名之前那一段）。
    pub unmounted: Result<(), String>,
    /// 卸载之后 F 高于 C 的 txg（卸载那一串把 F 抬过了 C 那一条时间线上还有效的根）。
    pub floor_above_the_abandoned_root_after_the_unmount: bool,
    /// C 还在根环里。
    pub the_abandoned_root_still_in_the_ring_after_the_unmount: bool,
    /// 卸载之后分配器里空闲的 C 引用的实例表单元那几份（盘，槽）。
    pub instance_table_slots_of_the_abandoned_root_free_after_the_unmount:
        Vec<(DeviceIdentity, SlotNumber)>,
}

/// 卸载之前覆盖写几次：抬 F 之前（攻方 K1-1 那一档准入第一次抬 F 在第 8 次改动上）。
pub const OVERWRITES_BEFORE_THE_UNMOUNT: usize = 3;

/// 场景：攻方 K1-1 末段那一形（「卸载那一串不判上限」那一句，判决 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第五节第一行）：
/// K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），重建那一遍盘 0 每条不被抛弃的根住的槽报读错，C 那一槽第一遍读得出、从重读起一直报读错；
/// 挂上之后覆盖写 3 次，再正常卸载，盘留着读坏。
/// 预期：卸载那一串重算影子账时拿记着的 C 顶上（用户 2026-10-06 定 T1 甲），C 引用的实例表单元（挂载那一刻被实例 2 的有效根豁免，
/// 卸载把 F 抬过那几条根之后只剩 C 引用）照隔离：卸载做成，之后分配器里那两份都不空闲，C 还在环里，checker 不判违例。
/// 判别力：抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上（`crates/mutations.tsv`「第10l批 T1 甲 卸载那一串读坏重读仍坏的这一类不拿记着的根顶上」）
/// 时那两份卸载之后空闲，最后一项红。
pub fn unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    let RemountKeepingTheFaultedDevices {
        mounted, devices, ..
    } = remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
        &history,
        DeviceIdentity(0),
        history.ring_slot_of_the_abandoned_root,
        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
            RootRingSlotReadFault::ReadFails,
        ),
    );
    let mounted = mounted.unwrap_or_else(|error| panic!("前提：再可写挂载做得成：{error:?}"));
    let the_abandoned_root_is_self_verified_on_only_one_rebuild_pass =
        slot_is_self_verified_on_only_one_rebuild_pass(
            &mounted,
            history.ring_slot_of_the_abandoned_root,
        );
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<DeviceFailingTheRebuildPass> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut overwrites_published = 0;
    // 迭代上界是 OVERWRITES_BEFORE_THE_UNMOUNT；提前出口只有被拒。
    for _ in 0..OVERWRITES_BEFORE_THE_UNMOUNT {
        if pool.overwrite(OVERWRITE_BYTES).is_err() {
            break;
        }
        overwrites_published += 1;
    }
    let mut session_being_unmounted = pool.session.take().expect("覆盖写之后会话还在");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session_being_unmounted.allocator,
        &mut session_being_unmounted.current,
        ShadowLedger::On,
    );
    let image = pool.image();
    let outcome = UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
        the_abandoned_root_is_self_verified_on_only_one_rebuild_pass,
        overwrites_published,
        unmounted: unmounted.map(|_| ()).map_err(|error| format!("{error:?}")),
        floor_above_the_abandoned_root_after_the_unmount: session_being_unmounted
            .current
            .root()
            .rollback_floor
            > history.abandoned_root.checkpoint_txg,
        the_abandoned_root_still_in_the_ring_after_the_unmount: abandoned_root_still_in_the_ring(
            &image, &history,
        ),
        instance_table_slots_of_the_abandoned_root_free_after_the_unmount: history
            .abandoned_root
            .instance_table
            .locations
            .iter()
            .filter(|location| {
                session_being_unmounted.allocator.devices
                    [usize::try_from(location.device.0).expect("盘号")]
                .is_free(location.slot)
            })
            .map(|location| (location.device, location.slot))
            .collect(),
    };
    images.hand_out(
        "卸载之后",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    assert_eq!(
        outcome,
        UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: true,
            overwrites_published: OVERWRITES_BEFORE_THE_UNMOUNT,
            unmounted: Ok(()),
            floor_above_the_abandoned_root_after_the_unmount: true,
            the_abandoned_root_still_in_the_ring_after_the_unmount: true,
            instance_table_slots_of_the_abandoned_root_free_after_the_unmount: Vec::new(),
        },
        "卸载那一串读 C 读坏、重读仍坏：拿记着的 C 顶上，C 引用的实例表单元照隔离"
    );
}

/// K2-2 那组历史上再挂（读坏只落在挂载里）、接着覆盖写到 C 离开根环，看分配器里 C 的落点（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 挂上之后 C 那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: bool,
    /// C 还在根环里的每次覆盖写之后，C 引用的实例表单元在分配器里空闲的那几份（第几次覆盖写，盘，槽）。
    pub instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring:
        Vec<(usize, DeviceIdentity, SlotNumber)>,
    /// 这一趟里环里最旧有效根越过了 C 的 txg、C 还在环里（进程内记根那一次的回收门槛越过 C，C 引用的、挂载那一刻被候选根豁免的单元被回收）。
    pub oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring: bool,
    /// C 离开根环那一次覆盖写之后，C 写出的落点在分配器里不空闲的那几个（盘，槽）：只被 C 引用、挂载时隔离的，C 离环时清掉。
    pub places_the_abandoned_root_wrote_not_free_once_it_left_the_ring:
        Vec<(DeviceIdentity, SlotNumber)>,
}

/// 场景：K2-2 那组历史（4 GiB、实例 1 覆盖写 1 次：盘宽，进程内不推抬 F），第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读只坏 C 的根槽（报读错）：
/// C 记成两遍里只有一遍自证过、被抛弃的那一类。挂上之后照常读盘，覆盖写到 C 离开根环。
/// 预期：进程内记根时这一类被抛弃根照它记着的落点算进「环里还在的被抛弃根引用着的落点」——环里最旧有效根越过 C 的 txg 那一次（C 还在环里）回收出来的
/// C 的实例表单元照补隔离，C 在环里时那两份一次都不空闲；C 离开根环那一次按它记着的落点清隔离，C 写出的落点那一刻全空闲；checker 不红。
/// 判别力：这一类被抛弃根不算进还被引用的落点（`crates/mutations.tsv`「第10l批 新类被抛弃那一半不算进环里还被引用的落点」）时第三项红；
/// 被换下时不清隔离（「第10l批 新类被抛弃那一半被换下时不清隔离」）时第五项红。
pub fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::FourGibibytes, 1);
    let mut faulted_reads = faulted_reads_of(
        &root_ring_slots_of_device_one(&history),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &[history.ring_slot_of_the_abandoned_root],
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        RootRingSlotReadFault::ReadFails,
    ));
    let RemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        ..
    } = remount_the_pool_with_an_abandoned_root(&history, faulted_reads);
    let mounted = mounted.unwrap_or_else(|error| panic!("前提：再可写挂载做得成：{error:?}"));
    let root_ring_of_the_rebuilt_allocator =
        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
    let the_abandoned_root_is_self_verified_on_only_one_rebuild_pass =
        slot_is_self_verified_on_only_one_rebuild_pass(
            &mounted,
            history.ring_slot_of_the_abandoned_root,
        );
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices: plain_devices_on(&image_after_the_remount),
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let root_ring_slot_count = every_root_ring_slot(&history.parameters.geometry).len();
    let instance_table_slots_of_the_abandoned_root_free =
        |pool_watched: &PoolUnderTest<SparseBlockDevice>| {
            history
                .abandoned_root
                .instance_table
                .locations
                .iter()
                .filter(|location| {
                    pool_watched.session().allocator.devices
                        [usize::try_from(location.device.0).expect("盘号")]
                    .is_free(location.slot)
                })
                .map(|location| (location.device, location.slot))
                .collect::<Vec<_>>()
        };
    let mut instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring = Vec::new();
    let mut oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring = false;
    let mut overwrites = 0usize;
    // 迭代上界是根环的槽数（C 的根槽在一圈之内被盖掉）；没有提前出口。
    while abandoned_root_still_in_the_ring(&pool.image(), &history) {
        assert!(
            overwrites < root_ring_slot_count,
            "C 的根槽在一圈（{root_ring_slot_count} 次发布）之内被盖掉"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrites} 次覆盖写：{refusal:?}"));
        overwrites += 1;
        let image = pool.image();
        if !abandoned_root_still_in_the_ring(&image, &history) {
            break;
        }
        let oldest_valid_root_txg = self_verified_roots_in_the_ring(&image)
            .into_iter()
            .filter(|(ring_slot, _)| *ring_slot != history.ring_slot_of_the_abandoned_root)
            .map(|(_, root)| root.checkpoint_txg)
            .min();
        if oldest_valid_root_txg
            .is_some_and(|oldest| oldest > history.abandoned_root.checkpoint_txg)
        {
            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring = true;
        }
        instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring.extend(
            instance_table_slots_of_the_abandoned_root_free(&pool)
                .into_iter()
                .map(|(device, slot)| (overwrites, device, slot)),
        );
    }
    let places_the_abandoned_root_wrote_not_free_once_it_left_the_ring = history
        .places_the_abandoned_root_wrote
        .iter()
        .filter(|(device, slot, _)| {
            !pool.session().allocator.devices[usize::try_from(device.0).expect("盘号")]
                .is_free(*slot)
        })
        .map(|(device, slot, _)| (*device, *slot))
        .collect();
    images.hand_out(
        "C 离开根环那一刻",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    assert_eq!(
        AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
            root_ring_of_the_rebuilt_allocator,
            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass,
            instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring,
            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring,
            places_the_abandoned_root_wrote_not_free_once_it_left_the_ring,
        },
        AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
            root_ring_of_the_rebuilt_allocator: Ok(
                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
            ),
            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: true,
            instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring: Vec::new(),
            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring: true,
            places_the_abandoned_root_wrote_not_free_once_it_left_the_ring: Vec::new(),
        },
        "两遍里只有一遍自证过的被抛弃根：在环里时它引用的单元一直扣着，离环时按它记着的落点清"
    );
}
