//! 系统配置见证读判不出取 R2（C331（择根倒挂压过已确认的写），D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」，零轮）：
//! 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（`recovery::system_configuration_witness_is_undecidable`）。
//! 可写挂载读阶段判 N-配置（`mount::newer_publish_witness`）与取号那一刻读见证值（`transaction::self_verified_system_configurations_of_every_device`）
//! 同一判；判不出按判据为真：可写挂载重读一次仍判不出就在取号之前拒、盘上逐字节不变，只读挂载照常；取号在第一个写之前拒。
//!
//! 盘面（新池新建文件之后每块盘两槽：世代 4、世代 5，世代 5 那一槽是新池新建文件那次轮换写的、槽号 5 mod 2 = 1），与 R2 网格的格对上：
//! - 纯形（全池读回一个值：两块盘世代 5 那一槽都报读错）：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出、拒可写；取号也拒。
//! - 只有一块盘世代 5 那一槽报读错、另一块盘两槽都自证过 ⇒ 判得出，照常可写挂载。
//! - 混合形（一盘报读错、另一盘读回全零）：判不出、拒可写（取甲-读错-每盘时这一形丢写，R2 拒，网格上混合形丢写 0）。
//! - 撕裂轮换（两块盘世代 5 那一槽都读回全零：读得出、自证不过，设备没报错）：判得出、照常可写挂载（取甲时拒、之后只剩只读挂载）。
mod common;

use common::{
    build_pool, parameters, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_writable, MountError, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{acquire_instance, InstanceAcquisitionFailed, PoolWriter};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};

/// 新池新建文件那次轮换写的世代号：每块盘世代 5 那一槽（槽号 5 mod 2 = 1）。
const GENERATION_OF_THE_NEW_POOL_FILE_CREATION_ROTATION: u64 = 5;

fn offset_of_the_newest_system_configuration_slot() -> u64 {
    (GENERATION_OF_THE_NEW_POOL_FILE_CREATION_ROTATION % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
        * u64::from(parameters().geometry.fixed_structure_slot_spacing)
}

/// `device` 世代 5 那一槽，每次读都报读错。
fn newest_system_configuration_slot_reporting_a_read_error_on(
    device: DeviceIdentity,
) -> UnreadableRange {
    UnreadableRange {
        device,
        offset_in_bytes: offset_of_the_newest_system_configuration_slot(),
        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        failing_reads: FailingReadsOfARange::Every,
    }
}

/// 把 `device` 世代 5 那一槽在盘上清零（撕裂的轮换：读得出、自证不过）。
fn zero_the_newest_system_configuration_slot_of(pool: &mut BuiltPool, device: DeviceIdentity) {
    let devices = pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
    let (_, zeroed_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    zeroed_device
        .write_at(
            DeviceOffsetInBytes(offset_of_the_newest_system_configuration_slot()),
            &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
            WriteDurability::Plain,
        )
        .expect("世代 5 那一槽清零");
}

/// 重开、包上 `ranges` 的读故障可写挂载一次，交回结局；盘拆掉读故障交回池。
fn writable_mount_with_unreadable_ranges(
    pool: &mut BuiltPool,
    ranges: Vec<UnreadableRange>,
) -> Result<singlefs_core::mount::Mounted, MountError> {
    let shared = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
    let outcome = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(without_unreadable_ranges(devices));
    outcome
}

/// 拒因：判据 N-配置 为真、重读仍为真，两遍读数都判 R2 的判不出。
fn refused_as_undecidable_on_both_reads(
    outcome: &Result<singlefs_core::mount::Mounted, MountError>,
) -> bool {
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

/// 纯形：两块盘世代 5 那一槽都报读错 ⇒ 判不出，重读一次仍判不出，拒可写、盘上逐字节不变；只读挂载照常。
#[test]
fn a_writable_mount_is_refused_when_the_newest_slot_of_every_device_reports_a_read_error() {
    let mut pool = build_pool("system-configuration-witness-pure-form-refused");
    let image_before = pool.memory_pool();
    let outcome = writable_mount_with_unreadable_ranges(
        &mut pool,
        vec![
            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(0)),
            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(1)),
        ],
    );
    assert!(
        refused_as_undecidable_on_both_reads(&outcome),
        "有槽报读错、没有一块盘两槽都自证过：判不出，拒可写；得到 {:?}",
        outcome.as_ref().err()
    );
    assert!(
        pool.memory_pool() == image_before,
        "拒在取号之前：盘上逐字节不变"
    );
    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
}

/// 只有盘 0 世代 5 那一槽报读错，盘 1 两槽都自证过 ⇒ 判得出（见证值取盘 1 的），照常可写挂载、取到实例 2。
#[test]
fn a_writable_mount_goes_ahead_when_one_device_reports_a_read_error_and_the_other_verified_both_slots(
) {
    let mut pool = build_pool("system-configuration-witness-one-device-read-error");
    let outcome = writable_mount_with_unreadable_ranges(
        &mut pool,
        vec![newest_system_configuration_slot_reporting_a_read_error_on(
            DeviceIdentity(0),
        )],
    );
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "盘 1 两槽都自证过：判得出，照常可写挂载；得到 {:?}",
        outcome.as_ref().err()
    );
}

/// 混合形：盘 0 世代 5 那一槽报读错、盘 1 世代 5 那一槽读回全零 ⇒ 有槽报读错、没有一块盘两槽都自证过：判不出，拒可写、盘上逐字节不变。
#[test]
fn a_writable_mount_is_refused_when_one_device_reports_a_read_error_and_the_other_reads_back_zeros()
{
    let mut pool = build_pool("system-configuration-witness-mixed-form-refused");
    zero_the_newest_system_configuration_slot_of(&mut pool, DeviceIdentity(1));
    let image_before = pool.memory_pool();
    let outcome = writable_mount_with_unreadable_ranges(
        &mut pool,
        vec![newest_system_configuration_slot_reporting_a_read_error_on(
            DeviceIdentity(0),
        )],
    );
    assert!(
        refused_as_undecidable_on_both_reads(&outcome),
        "一盘报读错、另一盘读回全零：判不出，拒可写；得到 {:?}",
        outcome.as_ref().err()
    );
    assert!(
        pool.memory_pool() == image_before,
        "拒在取号之前：盘上逐字节不变"
    );
}

/// 撕裂的系统配置轮换：两块盘世代 5 那一槽都读回全零（设备没报错）⇒ 判得出（见证值取两块盘世代 4 那一槽的），照常可写挂载、取到实例 2。
#[test]
fn a_writable_mount_goes_ahead_over_a_torn_system_configuration_rotation_on_every_device() {
    let mut pool = build_pool("system-configuration-witness-torn-rotation");
    zero_the_newest_system_configuration_slot_of(&mut pool, DeviceIdentity(0));
    zero_the_newest_system_configuration_slot_of(&mut pool, DeviceIdentity(1));
    let outcome = writable_mount_with_unreadable_ranges(&mut pool, Vec::new());
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "撕裂的轮换不算判不出，照常可写挂载；得到 {:?}",
        outcome.as_ref().err()
    );
}

/// 取号那一刻同一判：两块盘世代 5 那一槽都报读错，不经可写挂载直接取号 ⇒ 判不出，在第一个取号写之前拒，报盘表次序第一块报读错的盘（盘 0）；
/// 盘上逐字节不变。
#[test]
fn an_acquisition_is_refused_before_any_write_when_the_newest_slot_of_every_device_reports_a_read_error(
) {
    let mut pool = build_pool("system-configuration-witness-acquisition-refused");
    let image_before = pool.memory_pool();
    let shared = SharedUnreadableRanges::new(
        vec![
            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(0)),
            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(1)),
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
                first_device_reporting_a_read_error: DeviceIdentity(0),
            })
        ),
        "判不出：取号在任何写之前拒，报盘 0；得到 {acquisition:?}"
    );
    assert!(
        pool.memory_pool() == image_before,
        "拒在第一个取号写之前：盘上逐字节不变"
    );
}
