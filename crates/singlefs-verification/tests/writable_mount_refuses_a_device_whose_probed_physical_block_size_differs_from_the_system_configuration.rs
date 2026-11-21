//! 可写挂载逐盘比探测到的 `physical_block_size` 与择到的系统配置记着的 mkfs 时的 `physical_block_size`（偏移 317，根槽宽）：
//! D12（目标介质） 已定项 4 比对清单第一行「拒绝挂载（可写与只读都拒）」，这份钉可写挂载那一半；D22（单元原子性怎么合成） 已定项 2「挂载时逐设备复算比对」，C1040。
//! 两盘同构的池建好之后，把一块盘换成字节逐个相同、而探测值改一档的盘：可写挂载在任何写之前拒，盘表那一项点名那块盘、
//! 探测值与记录值；每块盘逐字节不变。对照：同一份字节交回探测值与记录值相同的两块盘，可写挂载照常做成。
//! 两条用例各管一个方向：记录 512、探测 4096；记录 4096、探测 512。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, DeviceTableDisagreement, MountError};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_verification::memory_pool::{SparseBlockDevice, SparseDevice};

const IMAGE_BYTES: u64 = 4 << 30;
/// 盘表里的两块盘。
const DEVICE_IDENTITIES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

fn parameters_with_the_physical_block_size(physical_block_size: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-probed!",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size,
            minimum_input_output_bytes: 0,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(0),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 两块探测值都是 `physical_block_size` 的空盘上照 `parameters` 做 mkfs，交回两块盘的字节。
fn images_of_a_pool_made_on_devices_probing(
    parameters: &MakeFilesystemParameters,
    physical_block_size: u32,
) -> Vec<SparseDevice> {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = DEVICE_IDENTITIES
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(physical_block_size)),
            )
        })
        .collect();
    make_filesystem(parameters, &mut devices).expect("两块盘的探测值与参数相同，mkfs 收");
    devices
        .into_iter()
        .map(|(_, device)| device.image)
        .collect()
}

/// `images` 逐块装进探测值为 `probed_physical_block_size_of_each_device` 的盘，按盘表次序交回。
fn devices_carrying(
    images: &[SparseDevice],
    probed_physical_block_size_of_each_device: [u32; 2],
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    DEVICE_IDENTITIES
        .into_iter()
        .zip(images)
        .zip(probed_physical_block_size_of_each_device)
        .map(|((identity, image), probed_physical_block_size)| {
            let mut device = SparseBlockDevice::new(
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(probed_physical_block_size),
            );
            device.image = image.clone();
            (identity, device)
        })
        .collect()
}

/// 记录值 `recorded` 的池上，`swapped_device` 那块盘换成探测值 `probed` 的盘：可写挂载在任何写之前拒成参数全对、盘表只错那一项，
/// 两块盘逐字节不变；同一份字节交回探测值都是 `recorded` 的两块盘，可写挂载照常做成。
fn assert_a_device_probing_another_physical_block_size_is_refused_before_any_write(
    recorded: u32,
    swapped_device: DeviceIdentity,
    probed: u32,
) {
    let parameters = parameters_with_the_physical_block_size(recorded);
    let images = images_of_a_pool_made_on_devices_probing(&parameters, recorded);
    let probed_of_each_device = DEVICE_IDENTITIES.map(|identity| {
        if identity == swapped_device {
            probed
        } else {
            recorded
        }
    });
    let mut devices = devices_carrying(&images, probed_of_each_device);
    let refusal = mount_writable(&parameters, &mut devices)
        .expect_err("一块盘的探测值与系统配置记着的 physical_block_size 不同：可写挂载必须拒");
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
    assert_eq!(
        disagreeing_device_table,
        vec![
            DeviceTableDisagreement::PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration {
                identity_handed_in: swapped_device,
                probed_physical_block_size: PhysicalBlockSizeInBytes(probed),
                physical_block_size_in_the_system_configuration: PhysicalBlockSizeInBytes(recorded),
            }
        ],
        "盘表只错一项：换上去的那块盘探测到 {probed}、系统配置记着 {recorded}"
    );
    for ((identity, device), before) in devices.iter().zip(&images) {
        assert_eq!(
            &device.image, before,
            "{identity:?} 逐字节不变：拒在任何写之前"
        );
    }
    let mut devices_probing_the_recorded_value = devices_carrying(&images, [recorded, recorded]);
    mount_writable(&parameters, &mut devices_probing_the_recorded_value).unwrap_or_else(|error| {
        panic!("同一份字节、两块盘都探测到 {recorded}：可写挂载照常做成，实际 {error:?}")
    });
}

/// 记录 512 的池，盘 1 换成探测 4096 的盘（探测值往大改一档）。
#[test]
fn a_device_probing_4096_on_a_pool_recording_512_is_refused_before_any_write() {
    assert_a_device_probing_another_physical_block_size_is_refused_before_any_write(
        512,
        DeviceIdentity(1),
        4096,
    );
}

/// 记录 4096 的池，盘 0 换成探测 512 的盘（探测值往小改一档；E72（设备描述符表的挂载复算） 那一格的方向）。
#[test]
fn a_device_probing_512_on_a_pool_recording_4096_is_refused_before_any_write() {
    assert_a_device_probing_another_physical_block_size_is_refused_before_any_write(
        4096,
        DeviceIdentity(0),
        512,
    );
}
