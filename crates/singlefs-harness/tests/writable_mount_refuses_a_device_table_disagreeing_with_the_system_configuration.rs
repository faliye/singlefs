//! 实审 A1b Q4（主 agent 2026-09-27 定）：可写挂载拿调用方的参数与盘上择到的系统配置逐项比（代码审阅第 17 条），比的只有
//! mkfs 参数里有的几项；系统配置里的设备数 `devs` 与每块盘自己的本盘设备号不在参数里，没比。取号与每次发布末尾的轮换按盘表逐项写系统配置
//! （`transaction::PoolWriter::write_system_configuration_slot` 把设备数写成盘表的项数、本盘设备号写成盘表给的身份），
//! 盘表多交一块盘、或把一块盘标成别的身份，盘上系统不可变配置（D22（单元原子性怎么合成） 已定项 26 第一档）就被改写。
//! 改成：交进来的盘数与 `devs` 不同、某块盘自证过的系统配置里的本盘设备号与盘表给的身份不同，都在任何写之前拒
//! （`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`，盘表那几项另列）。

mod common;

use common::{build_pool, crash_state_devices, parameters, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable, DeviceCount, DeviceTableDisagreement, MountError};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type RecordedSparse = RecordingBlockDevice<SparseBlockDevice>;

/// `image` 里 `source` 那块盘的一份拷贝，标成 `identity` 交进来，写照旧录进 `stream`。
fn copy_of_device_handed_in_as(
    image: &MemoryPool,
    source: DeviceIdentity,
    identity: DeviceIdentity,
    stream: &SharedStream,
) -> (DeviceIdentity, RecordedSparse) {
    let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
    device.image = image.devices.get(&source).cloned().expect("镜像里有这块盘");
    (
        identity,
        RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    )
}

/// 可写挂载 `devices`：必须拒，录制流一步都没有、每块盘逐字节不变。交回挂载的错。
fn mount_refused_before_any_write(
    devices: &mut Vec<(DeviceIdentity, RecordedSparse)>,
    stream: &SharedStream,
) -> MountError {
    let images_before: Vec<_> = devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect();
    let refusal = mount_writable(&parameters(), devices)
        .expect_err("盘表与盘上系统配置不一致，可写挂载必须拒");
    assert_eq!(
        stream.operation_count(),
        0,
        "拒在任何写之前：录制流一步都没有（没发写、没发屏障）"
    );
    for (index, ((_, device), before)) in devices.iter().zip(&images_before).enumerate() {
        assert_eq!(&device.inner().image, before, "盘表第 {index} 项逐字节不变");
    }
    refusal
}

/// 可写挂载拒成 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`、参数一项都没错时，交回盘表那几项。
fn device_table_disagreements_of(refusal: MountError) -> Vec<DeviceTableDisagreement> {
    let MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields,
        disagreeing_device_table,
    } = refusal
    else {
        panic!("该报 CallerParametersDisagreeWithTheSelectedSystemConfiguration，实际 {refusal:?}")
    };
    assert_eq!(
        disagreeing_fields,
        Vec::new(),
        "参数用的是建池的那一份：一项都不该错"
    );
    disagreeing_device_table
}

/// 第一个事务写完的池（devs = 2），盘表交三块：盘 0、盘 1，与一块标成盘 2 的盘 1 的拷贝（它的系统配置里本盘设备号是 1）。
/// 三块盘都自证得过同一份系统配置（fsid 与 devs 相同）；改之前拒在更后面、报的是写行那次发布的预演（盘 2 的账里没有要换下的记账树单元，
/// `RowPublishAdmissionRefusedBeforeAcquisition`）——对不上的是盘表，报错却指向账。现在在选系统配置之后就拒：盘数 3 与 devs 2 不同、
/// 盘 2 的本盘设备号是 1。对照：同一个池只交盘 0、盘 1，照常做成。
#[test]
fn mount_with_a_third_device_carrying_a_copy_of_device_one_is_refused_before_any_write() {
    let pool = build_pool("a1b-third-device-copy");
    let image = pool.memory_pool();
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    devices.push(copy_of_device_handed_in_as(
        &image,
        DeviceIdentity(1),
        DeviceIdentity(2),
        &stream,
    ));
    let refusal = mount_refused_before_any_write(&mut devices, &stream);
    assert_eq!(
        device_table_disagreements_of(refusal),
        vec![
            DeviceTableDisagreement::DeviceCountDiffersFromTheSystemConfiguration {
                devices_handed_in: DeviceCount(3),
                device_count_in_the_system_configuration: DeviceCount(2),
            },
            DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                identity_handed_in: DeviceIdentity(2),
                own_device_number_on_disk: DeviceIdentity(1),
            },
        ],
        "盘数 3 与 devs 2 不同；盘 2 两槽里记的本盘设备号都是 1，报一项"
    );

    let mut both_devices = crash_state_devices(&image, &[], &[], &SharedStream::new());
    mount_writable(&parameters(), &mut both_devices)
        .unwrap_or_else(|error| panic!("只交盘 0、盘 1：可写挂载照常做成，实际 {error:?}"));
}

/// 盘表交两块：盘 0，与一块标成盘 1 的盘 0 的拷贝（它的系统配置里本盘设备号是 0）。盘数与 devs 相同；改之前根环里区域 0、2 的根
/// 都在盘 0 上读得出、journal 与单元两盘各一份，挂载照常做成，取号把那块拷贝的本盘设备号写成 1。现在拒：盘 1 的本盘设备号是 0。
#[test]
fn mount_with_device_one_carrying_a_copy_of_device_zero_is_refused_before_any_write() {
    let pool = build_pool("a1b-device-one-copy-of-zero");
    let image = pool.memory_pool();
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    devices.retain(|(identity, _)| *identity == DeviceIdentity(0));
    devices.push(copy_of_device_handed_in_as(
        &image,
        DeviceIdentity(0),
        DeviceIdentity(1),
        &stream,
    ));
    let refusal = mount_refused_before_any_write(&mut devices, &stream);
    assert_eq!(
        device_table_disagreements_of(refusal),
        vec![
            DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                identity_handed_in: DeviceIdentity(1),
                own_device_number_on_disk: DeviceIdentity(0),
            },
        ],
        "盘数与 devs 相同；盘 1 两槽里记的本盘设备号都是 0，报一项"
    );
}
