//! 实五那几条 harness 档用例共用的搭建（`admission_raises_the_floor_before_refusing.rs` 等）：池本身（两块内存稀疏盘上 mkfs、取号、暖机、
//! 第一个文件，之后经挂着的会话发布、崩了再挂，记下每条根带的文件内容）在 `tests/common_admission_pool/mod.rs`，这里全部转出去；
//! 这里只补上不归那一份的两样：按随机历史的盘宽档（`HistoryDeviceWidth`）起池，与故障注入那一层（`FaultInjectingBlockDevice`）。
//! checker 档的 `crash_enumeration_floor_raise_pushed_by_admission.rs` 不经这一份、直接引那一份：这里引的 history.rs 与 fault_injection.rs
//! 会把随机历史脚手架带进那条登记用例的判法闭包。
#![allow(dead_code, reason = "几个测试文件各自只用到其中一部分")]

#[path = "../common_admission_pool/mod.rs"]
mod common_admission_pool;

pub use common_admission_pool::*;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError, MountSpaceAdmission};
use singlefs_harness::fault_injection::FaultInjectingBlockDevice;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

impl DeviceOverASparseImage for FaultInjectingBlockDevice<SparseBlockDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.wrapped_device().image
    }
}

impl<Device: DeviceOverASparseImage> PoolUnderTest<Device> {
    /// 按随机历史的一档盘宽起池（mkfs、取号 1、暖机、第一个文件，见 `start_after_the_first_file_on_devices_of`）。
    /// 起 384 槽那一档时先核 `tests/common_admission_pool/mod.rs` 抄的那一份与 history.rs 那一档逐项相同：
    /// checker 档的抬 F 用例经那一份起池，与这里的 harness 档用例必须是同一种盘。
    pub fn start_after_the_first_file(
        device_width: HistoryDeviceWidth,
        wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> Device,
    ) -> Self {
        match device_width {
            HistoryDeviceWidth::UnitAreaOf384Slots => {
                assert_the_384_slot_copy_matches_the_random_history_width();
            }
            HistoryDeviceWidth::FourGibibytes
            | HistoryDeviceWidth::UnitAreaOf256Slots
            | HistoryDeviceWidth::UnitAreaOf240Slots => {}
        }
        Self::start_after_the_first_file_on_devices_of(
            device_width.parameters(),
            device_width.device_bytes(),
            wrap,
        )
    }
}

/// `tests/common_admission_pool/mod.rs` 抄的单元区 384 槽那一档（参数、盘宽、按落点分写的几何）与 history.rs 的
/// `HistoryDeviceWidth::UnitAreaOf384Slots` 逐项相同；不同就 panic，消息写明哪一项。
fn assert_the_384_slot_copy_matches_the_random_history_width() {
    let original = HistoryDeviceWidth::UnitAreaOf384Slots;
    assert_eq!(
        parameters_of_the_384_slot_small_devices(),
        original.parameters(),
        "common_admission_pool 抄的 384 槽那一档 mkfs 参数与 history.rs 的 UnitAreaOf384Slots 相同（改了一边另一边也要改）"
    );
    assert_eq!(
        device_bytes_of_the_384_slot_small_devices(),
        original.device_bytes(),
        "common_admission_pool 抄的 384 槽那一档盘宽与 history.rs 的 UnitAreaOf384Slots 相同（改了一边另一边也要改）"
    );
    let copied_geometry = fixed_geometry_of_the_384_slot_small_devices();
    let original_geometry = original.fixed_geometry();
    assert_eq!(
        (
            copied_geometry.fixed_structure_slot_spacing,
            copied_geometry.journal_ring_bytes,
            copied_geometry.root_ring_slots_per_region,
        ),
        (
            original_geometry.fixed_structure_slot_spacing,
            original_geometry.journal_ring_bytes,
            original_geometry.root_ring_slots_per_region,
        ),
        "common_admission_pool 抄的 384 槽那一档几何（槽距、环长、每区根槽数）与 history.rs 的 UnitAreaOf384Slots 相同"
    );
}

/// 不为环窗口补足的窄盘（用户 2026-10-02 定 Q-T「甲」：关准入的那几条单开不补足的窄盘）：参数取 `device_width` 那一档（6 MiB 环），
/// 单元区只有 `unit_area_slots` 槽、不加补足；mkfs、取号 1、暖机之后、第一个文件之前就把这个进程的分配器上的空间准入关掉
/// （只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`：这样窄的盘扣不起环窗口，准入开着连第一个文件都放不进），落点是唯一的墙。
/// 与 `start_after_the_first_file_on_devices_of` 同一套步骤，只多关准入那一步；会话接着第一个文件那一版、准入关着。
pub fn start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
    device_width: HistoryDeviceWidth,
    unit_area_slots: u64,
) -> PoolUnderTest<SparseBlockDevice> {
    use singlefs_core::allocator::UnitAreaStart;
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
    use singlefs_core::mount::{ParametersAndDeviceTableOfTheMount, ShadowLedger};
    use singlefs_core::mounted_session::MountedSession;
    use singlefs_core::transaction::{
        acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    };
    use singlefs_format::SLOT_BYTES;
    use std::collections::BTreeMap;

    let parameters = device_width.parameters();
    let unit_area_start_slot =
        UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
            .expect("小盘的环末尾的下一个槽落在聚簇段边界上")
            .slot()
            .0;
    let device_bytes = (unit_area_start_slot + unit_area_slots) * SLOT_BYTES;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("不补足的窄盘上 mkfs 做得成");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    let first_content = content_of(3000, 0);
    let (instance, first_version) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("准入关着：第一个文件落得下");
        (instance, first_version)
    };
    let mut content_of_each_root = BTreeMap::new();
    content_of_each_root.insert(root_key(&first_version.root), first_content.clone());
    let parameters_and_device_table = ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(
        &devices,
    )
    .expect("mkfs 同一个进程刚写成的两块盘：盘表就是 mkfs 用的那一份，系统配置读得出、与盘表相符");
    PoolUnderTest {
        parameters,
        device_bytes,
        devices,
        session: Some(MountedSession {
            allocator,
            current: PoolVersion::WithFile(first_version),
            instance,
            shadow_ledger: ShadowLedger::On,
            parameters_and_device_table,
        }),
        write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: first_content,
        content_of_each_root,
    }
}

pub fn start_plain(device_width: HistoryDeviceWidth) -> PoolUnderTest<SparseBlockDevice> {
    PoolUnderTest::start_after_the_first_file(device_width, plain_device)
}

/// 这一刻的盘面拷一份、只崩了再挂那一次可写挂载（空间准入判着）交回的空间准入结局。
pub fn space_admission_of_a_crash_remount_on_a_copy(
    pool: &PoolUnderTest<SparseBlockDevice>,
) -> Result<MountSpaceAdmission, MountError> {
    let mut devices = plain_devices_on(&pool.image());
    mount_writable_with_space_admission(
        &pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .map(|mounted| mounted.output.space_admission)
}
