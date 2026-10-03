//! 系统配置见证读取 R2（C331（择根倒挂压过已确认的写），D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」）在
//! 槽 0（偶数世代）那一侧的可写挂载与取号（遗留改动代码轮判决 `research/prompts/m3-leftover-code-r1-main-verification.md` 第三节 LR1）。
//!
//! 新池新建文件之后每块盘两槽是世代 4（槽 0）、世代 5（槽 1）；再可写挂载一次，系统配置轮换两次（世代 7 落在槽 1、世代 8 落在槽 0），
//! 最新的世代 8，8 mod 2 = 0，在槽 0。
//! `system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs` 里写坏的都是世代 5
//! 那一槽（槽 1）：见证读与取号只看每块盘的槽 1 的错实现在那几条上全过（攻方变异 slot1-only-both）。这里写坏的是最新的世代 8 那一槽（槽 0）：
//! - 两块盘槽 0 都报读错：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，可写挂载重读一次仍判不出、取号之前拒，取号那一刻也拒；
//! - 盘 0 槽 0 读回全零（撕裂的轮换）、盘 1 槽 0 报读错：报读错的是第二块盘，同样判不出、拒可写；
//! - 读阶段两槽都读得出、到取号之前的逐盘核那一遍盘 1 两槽才读不出（两次读之间的瞬时读错）：逐盘核交 `Err(盘 1)`，照见证读的拒因报。
mod common;

use std::cell::RefCell;
use std::rc::Rc;

use common::{
    build_pool, disk_snapshot, parameters, with_unreadable_ranges, without_unreadable_ranges,
    BuiltPool, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable, MountError, Mounted, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::PoolReader;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{acquire_instance, InstanceAcquisitionFailed, PoolWriter};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};

/// 新池新建文件之后可写挂载一次、系统配置轮换两次之后最新的世代号（新池新建文件那次是 5）：落在槽 8 mod 2 = 0。
const NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT: u64 = 8;

const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

fn slot_spacing_in_bytes() -> u64 {
    u64::from(parameters().geometry.fixed_structure_slot_spacing)
}

/// 最新的世代 8 那一槽在盘上的偏移：槽 0。
fn offset_of_the_newest_slot() -> u64 {
    (NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
        * slot_spacing_in_bytes()
}

/// `device` 那一槽（偏移 `offset_in_bytes`）读出来自证得过的系统配置的世代号；读不出或自证不过交 `None`。
fn self_verified_generation_at(
    pool: &BuiltPool,
    device: DeviceIdentity,
    offset_in_bytes: u64,
) -> Option<u64> {
    PoolReader::read(
        &pool.memory_pool(),
        device,
        DeviceOffsetInBytes(offset_in_bytes),
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
    )
    .and_then(|bytes| SystemConfiguration::parse_slot(&bytes).ok())
    .map(|system_configuration| system_configuration.quantities.slot_generation)
}

/// 新池新建文件之后照常可写挂载一次（取到实例 2，系统配置轮换两次：世代 7 进每块盘的槽 1、世代 8 进槽 0），会话随即丢掉（进程退出）。
/// 交回之前核这一形：每块盘槽 0 是世代 8、槽 1 是世代 7——最新的那一槽是槽 0。
fn pool_whose_newest_system_configuration_slot_is_slot_zero(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let mounted =
        mount_writable(&parameters(), &mut devices).expect("新池新建文件之后照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    drop(mounted);
    pool.devices = Some(devices);
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        assert_eq!(
            (
                self_verified_generation_at(&pool, device, 0),
                self_verified_generation_at(&pool, device, slot_spacing_in_bytes()),
            ),
            (
                Some(NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT),
                Some(NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT - 1)
            ),
            "盘 {device:?}：槽 0 是最新的世代 8、槽 1 是世代 7"
        );
    }
    pool
}

/// `device` 最新的世代 8 那一槽（槽 0），每次读都报读错。
fn slot_zero_reporting_a_read_error_on(device: DeviceIdentity) -> UnreadableRange {
    UnreadableRange {
        device,
        offset_in_bytes: offset_of_the_newest_slot(),
        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        failing_reads: FailingReadsOfARange::Every,
    }
}

/// 重开、包上 `ranges` 的读故障可写挂载一次，交回结局；盘拆掉读故障交回池。
fn writable_mount_with_unreadable_ranges(
    pool: &mut BuiltPool,
    ranges: Vec<UnreadableRange>,
) -> Result<Mounted, MountError> {
    let shared = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
    let outcome = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(without_unreadable_ranges(devices));
    outcome
}

/// 拒因：判据 N-配置 为真、重读仍为真，两遍读数都判 R2 的判不出。
fn refused_as_undecidable_on_both_reads(outcome: &Result<Mounted, MountError>) -> bool {
    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = outcome else {
        return false;
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = still_unreadable.as_ref()
    else {
        return false;
    };
    [first_read, reread].iter().all(|reading| {
        reading.witness.comparison
            == WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
    })
}

fn describe(outcome: &Result<Mounted, MountError>) -> String {
    match outcome {
        Ok(mounted) => format!("挂载做成：取到实例 {:?}", mounted.output.instance),
        Err(error) => format!("{error:?}"),
    }
}

/// 场景：最新的系统配置在槽 0（世代 8），两块盘的槽 0 都报读错。
/// 预期：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，重读一次仍判不出，取号之前拒可写；系统配置槽、根环、录制流步数都不变；只读挂载照常。
/// 判别力：见证读与取号只看每块盘的槽 1 的错实现看不见槽 0 的读错、两槽算一槽，判得出、照常挂上取到实例 3。
#[test]
fn a_writable_mount_is_refused_when_slot_zero_of_every_device_reports_a_read_error() {
    let mut pool = pool_whose_newest_system_configuration_slot_is_slot_zero(
        "lr1-slot-zero-read-error-every-device",
    );
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let outcome = writable_mount_with_unreadable_ranges(
        &mut pool,
        vec![
            slot_zero_reporting_a_read_error_on(DEVICE_ZERO),
            slot_zero_reporting_a_read_error_on(DEVICE_ONE),
        ],
    );
    assert!(
        refused_as_undecidable_on_both_reads(&outcome),
        "两块盘最新那一槽（槽 0）都报读错：判不出，拒可写；得到 {}",
        describe(&outcome)
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在取号之前：系统配置槽、根环、录制流步数都不变"
    );
    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
}

/// 场景：同上一形的盘，不经可写挂载直接取号，两块盘的槽 0 都报读错。
/// 预期：取号那一刻同一判（判不出），在第一个取号写之前拒，报盘表次序第一块报读错的盘（盘 0）；盘上不变。
/// 判别力：取号只看槽 1 时取号照做、写进实例 3。
#[test]
fn an_acquisition_is_refused_before_any_write_when_slot_zero_of_every_device_reports_a_read_error()
{
    let mut pool =
        pool_whose_newest_system_configuration_slot_is_slot_zero("lr1-slot-zero-acquisition");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let shared = SharedUnreadableRanges::new(
        vec![
            slot_zero_reporting_a_read_error_on(DEVICE_ZERO),
            slot_zero_reporting_a_read_error_on(DEVICE_ONE),
        ],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
    let pool_parameters = parameters();
    let acquisition = {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        acquire_instance(&mut writer)
    };
    pool.devices = Some(without_unreadable_ranges(devices));
    assert!(
        matches!(
            acquisition,
            Err(InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
                first_device_reporting_a_read_error: DEVICE_ZERO,
            })
        ),
        "判不出：取号在任何写之前拒，报盘 0；得到 {acquisition:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在第一个取号写之前：盘上不变"
    );
}

/// 场景：最新那一槽（槽 0）在盘 0 读回全零（撕裂的轮换：读得出、自证不过，设备没报错），在盘 1 报读错。
/// 预期：报读错的是第二块盘，有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，拒可写、盘上不变。
/// 判别力：判不出的前一半只看盘表次序第一块盘有没有报读错时，盘 0 没报，判得出、照常挂上。
#[test]
fn a_writable_mount_is_refused_when_only_the_second_device_reports_a_read_error_on_slot_zero_and_the_first_reads_back_zeros(
) {
    let mut pool =
        pool_whose_newest_system_configuration_slot_is_slot_zero("lr1-slot-zero-second-device");
    {
        let devices = pool.devices.as_mut().expect("挂载之后盘还开着");
        let (_, zeroed_device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == DEVICE_ZERO)
            .expect("池里有盘 0");
        zeroed_device
            .write_at(
                DeviceOffsetInBytes(offset_of_the_newest_slot()),
                &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
                WriteDurability::Plain,
            )
            .expect("盘 0 槽 0 清零");
    }
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let outcome = writable_mount_with_unreadable_ranges(
        &mut pool,
        vec![slot_zero_reporting_a_read_error_on(DEVICE_ONE)],
    );
    assert!(
        refused_as_undecidable_on_both_reads(&outcome),
        "盘 0 槽 0 读回全零、盘 1 槽 0 报读错：判不出，拒可写；得到 {}",
        describe(&outcome)
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在取号之前：盘上不变"
    );
}

/// 盘 1 两个系统配置槽各被读了几次（每一次碰到那一槽的读算一次），第一道屏障那一刻各几次；从第几次读起读坏、读坏过之后碰到别处的第一次读撤掉读故障。
/// 几块盘共用一份。
struct SlotReadsOfDeviceOne {
    reads_of_each_slot: [u64; 2],
    first_failing_read_of_each_slot: Option<[u64; 2]>,
    failing_reads_lifted: bool,
    failed_reads: u64,
    reads_of_each_slot_at_the_first_barrier: Option<[u64; 2]>,
}

#[derive(Clone)]
struct SharedSlotReadsOfDeviceOne(Rc<RefCell<SlotReadsOfDeviceOne>>);

impl SharedSlotReadsOfDeviceOne {
    fn new(first_failing_read_of_each_slot: Option<[u64; 2]>) -> Self {
        Self(Rc::new(RefCell::new(SlotReadsOfDeviceOne {
            reads_of_each_slot: [0; 2],
            first_failing_read_of_each_slot,
            failing_reads_lifted: false,
            failed_reads: 0,
            reads_of_each_slot_at_the_first_barrier: None,
        })))
    }
}

/// 读按 [`SharedSlotReadsOfDeviceOne`] 数盘 1 两槽、读坏、撤掉；写、写零原样交给里面那块盘；屏障那一刻记下两槽的读数。
struct DeviceCountingSlotReadsOfDeviceOne<Inner: BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    shared: SharedSlotReadsOfDeviceOne,
}

impl<Inner: BlockDevice> BlockDevice for DeviceCountingSlotReadsOfDeviceOne<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let slot_read = if self.identity == DEVICE_ONE {
            [0, slot_spacing_in_bytes()]
                .into_iter()
                .position(|slot_offset| {
                    offset.0 < slot_offset + SYSTEM_CONFIGURATION_SLOT_BYTES
                        && slot_offset < offset.0 + length_in_bytes
                })
        } else {
            None
        };
        let mut state = self.shared.0.borrow_mut();
        match slot_read {
            Some(slot_index) => {
                state.reads_of_each_slot[slot_index] += 1;
                let reads_of_this_slot = state.reads_of_each_slot[slot_index];
                let fails_this_time = !state.failing_reads_lifted
                    && state
                        .first_failing_read_of_each_slot
                        .is_some_and(|first_failing| {
                            reads_of_this_slot >= first_failing[slot_index]
                        });
                if fails_this_time {
                    state.failed_reads += 1;
                    return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                        "用例点名的这一槽这一次读坏",
                    )));
                }
            }
            None => {
                if state.failed_reads > 0 {
                    state.failing_reads_lifted = true;
                }
            }
        }
        drop(state);
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
        let mut state = self.shared.0.borrow_mut();
        if state.reads_of_each_slot_at_the_first_barrier.is_none() {
            state.reads_of_each_slot_at_the_first_barrier = Some(state.reads_of_each_slot);
        }
        drop(state);
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn counting_slot_reads_of_device_one<Inner: BlockDevice>(
    devices: Vec<(DeviceIdentity, Inner)>,
    shared: &SharedSlotReadsOfDeviceOne,
) -> Vec<(DeviceIdentity, DeviceCountingSlotReadsOfDeviceOne<Inner>)> {
    devices
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceCountingSlotReadsOfDeviceOne {
                    identity,
                    inner,
                    shared: shared.clone(),
                },
            )
        })
        .collect()
}

/// 取号写之前那道屏障之前、取号自己读盘 1 每一槽的次数：重算要取的号一次（`transaction::next_instance_generation_to_acquire`）、
/// 取号写之前读上一个号一次、见证值那一核一次（`transaction::self_verified_system_configurations_of_every_device`）。
/// 逐盘核那一遍是它们之前最后一次读这一槽。
const SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER: u64 = 3;

/// 场景：新池新建文件之后重开、可写挂载；读阶段与之前几遍读盘 1 两槽都读得出，到取号之前的逐盘核那一遍（取号写之前那道屏障那一刻
/// 两槽各被读的次数减去取号自己的三次）两槽读坏，之后碰到别处的第一次读起读故障撤掉（两次读之间的瞬时读错）。
/// 次数取另一份逐字节相同的池上照常挂载时数到的。
/// 预期：逐盘核交 `Err(盘 1)`，照见证读的拒因报 `NewerStateStillUnreadableAfterOneReread`（后一遍判「有一块盘一份自证过的系统配置都没有」）；
/// 只读坏了逐盘核那两次；系统配置槽、根环、录制流步数都不变。
/// 判别力：逐盘核读不出就跳过这块盘时，取号那几遍读得出，挂载做成。
#[test]
fn a_writable_mount_is_refused_when_the_slots_of_the_second_device_turn_unreadable_only_for_the_check_of_every_device(
) {
    let mut probe_pool = build_pool("lr1-check-of-every-device-probe");
    let probe = SharedSlotReadsOfDeviceOne::new(None);
    let mut probe_devices = counting_slot_reads_of_device_one(probe_pool.reopen_recorded(), &probe);
    let probe_mount = mount_writable(&parameters(), &mut probe_devices);
    assert!(
        probe_mount.is_ok(),
        "一个读都不坏时这份池照常可写挂载：{}",
        describe(&probe_mount)
    );
    drop(probe_mount);
    drop(probe_devices);
    let reads_at_the_first_barrier = probe
        .0
        .borrow()
        .reads_of_each_slot_at_the_first_barrier
        .expect("可写挂载取号之前发过屏障");
    assert!(
        reads_at_the_first_barrier
            .iter()
            .all(|reads| *reads > SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER + 1),
        "取号写之前盘 1 两槽各被读过：择系统配置、见证读、逐盘核与取号那三次：{reads_at_the_first_barrier:?}"
    );
    let read_of_the_check_of_every_device = reads_at_the_first_barrier
        .map(|reads| reads - SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER);
    let mut pool = build_pool("lr1-check-of-every-device");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let shared = SharedSlotReadsOfDeviceOne::new(Some(read_of_the_check_of_every_device));
    let mut devices = counting_slot_reads_of_device_one(pool.reopen_recorded(), &shared);
    let outcome = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(
        devices
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    let refused_by_the_check_of_every_device = matches!(
        &outcome,
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
            if matches!(
                still_unreadable.as_ref(),
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                    reread,
                    ..
                } if reread.witness.comparison
                    == WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot
            )
    );
    assert!(
        refused_by_the_check_of_every_device,
        "逐盘核那一遍盘 1 两槽读不出：拒可写，后一遍判「有一块盘一份自证过的系统配置都没有」；得到 {}",
        describe(&outcome)
    );
    assert_eq!(
        shared.0.borrow().failed_reads,
        2,
        "只读坏了逐盘核读盘 1 两槽那两次（第 {read_of_the_check_of_every_device:?} 次）"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在取号之前：盘上不变"
    );
}
