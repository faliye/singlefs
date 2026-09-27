//! 代码审阅第 18 条（实审 A1）：可写挂载的设备准入只数不同的设备身份（`mount::distinct_device_identities_handed_in`），
//! 取号、发布与系统配置轮换却按盘表逐项写（`transaction::write_acquired_instance` 按下标逐盘写）。盘表 `[(盘 0, A), (盘 1, B), (盘 0, C)]`
//! 数出 2 块、过了 w 的下限，C 不是本池的盘，却会收到本池的系统配置写。改成：盘表里有身份交了不止一次就在读任何一块盘之前拒
//! （`MountError::DeviceIdentitiesHandedInMoreThanOnce`），三块盘逐字节不变。

mod common;

use common::{build_pool, crash_state_devices, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable, MountError, RepeatedDeviceIdentity};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 第一个事务写完的池（盘 0 = A、盘 1 = B），再交一块空盘 C、也标成盘 0：可写挂载拒成 `DeviceIdentitiesHandedInMoreThanOnce`，
/// 报出盘 0 交了两次；录制流一步都没有（没发写、没发屏障），A、B、C 逐字节不变。
/// 对照：同一个池只交 A、B，可写挂载照常做成。
#[test]
fn mount_with_device_zero_handed_in_again_as_a_third_device_is_refused_before_any_write() {
    let pool = build_pool("a1-device-zero-twice");
    let image = pool.memory_pool();
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    devices.push((
        DeviceIdentity(0),
        RecordingBlockDevice::with_shared_stream(
            DeviceIdentity(0),
            SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            stream.clone(),
        ),
    ));
    let images_before: Vec<_> = devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect();
    let refusal = mount_writable(&common::parameters(), &mut devices)
        .expect_err("盘 0 交了两次，可写挂载必须拒");
    let MountError::DeviceIdentitiesHandedInMoreThanOnce { repeated } = refusal else {
        panic!("该报 DeviceIdentitiesHandedInMoreThanOnce，实际 {refusal:?}")
    };
    assert_eq!(
        repeated,
        vec![RepeatedDeviceIdentity {
            device: DeviceIdentity(0),
            times_handed_in: 2,
        }],
        "盘 0 交了两次，盘 1 一次"
    );
    assert_eq!(
        stream.operation_count(),
        0,
        "拒在任何写之前：录制流一步都没有"
    );
    for (index, ((_, device), before)) in devices.iter().zip(&images_before).enumerate() {
        assert_eq!(&device.inner().image, before, "盘表第 {index} 项逐字节不变");
    }

    let mut both_devices = crash_state_devices(&image, &[], &[], &SharedStream::new());
    mount_writable(&common::parameters(), &mut both_devices)
        .unwrap_or_else(|error| panic!("只交 A、B：可写挂载照常做成，实际 {error:?}"));
}
