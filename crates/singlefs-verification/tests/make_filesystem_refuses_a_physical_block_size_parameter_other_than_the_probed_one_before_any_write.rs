//! mkfs 逐盘探测 `physical_block_size`，调用方参数里的那个与任一块盘的探测值不同就拒、一个字节不写
//! （D22（单元原子性怎么合成） 已定项 2 根槽宽一格「调用方给的 `physical_block_size` 与 mkfs 时探测到的不同就拒绝 mkfs」；C1040）。
//! 照参数写出的池，挂载逐盘比探测值都会拒（D12（目标介质） 已定项 4），所以拒在 mkfs 这一步、盘上不留半份格式。
//! 两块盘都探测 4096、参数 512 的一条；只有盘 1 探测 4096 的一条（判的是每一块盘，不只盘 0）；参数与探测都是 4096 的阳性对照一条。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemError, MakeFilesystemParameters,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, ROOT_RING_BASE_SLOT, SLOT_BYTES,
};
use singlefs_verification::memory_pool::{SparseBlockDevice, SparseDevice};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-pbsmkfs";
/// 盘上预先写的花样：mkfs 头几步会清零的三处（系统配置槽 0、根环基址、journal 环起点）各一段，清零与写都改得掉它。
const PATTERN_BYTE: u8 = 0xA5;
const PATTERN_BYTES: usize = 4096;

/// 参数只差 `physical_block_size`：io_min 不声明，槽距 4096，根槽宽 512 与 4096 都过几何那几判。
fn parameters_declaring(physical_block_size: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
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

/// 两块带花样的内存盘，盘 0、盘 1 各自探测到 `probed_physical_block_sizes` 里的那个块宽，外面包录制器、录进同一条流。
/// 交回包好的盘与包之前两块盘的镜像（比「逐字节不变」用）。
fn patterned_recorded_devices(
    probed_physical_block_sizes: [u32; 2],
    stream: &SharedStream,
) -> (
    Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
    Vec<SparseDevice>,
) {
    let mut images_before = Vec::new();
    let devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .zip(probed_physical_block_sizes)
            .map(|(identity, probed_physical_block_size)| {
                let mut device = SparseBlockDevice::new(
                    DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(probed_physical_block_size),
                );
                for offset in [
                    0,
                    ROOT_RING_BASE_SLOT * SLOT_BYTES,
                    JOURNAL_RING_START_SLOT * SLOT_BYTES,
                ] {
                    device
                        .image
                        .write(DeviceOffsetInBytes(offset), &[PATTERN_BYTE; PATTERN_BYTES]);
                }
                images_before.push(device.image.clone());
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    (devices, images_before)
}

/// 在任何写之前拒：录制流一步都没有（写、写零、屏障都没发），两块盘与 mkfs 之前逐字节相同。
fn assert_nothing_was_written(
    stream: &SharedStream,
    devices: &[(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)],
    images_before: &[SparseDevice],
    scenario_named: &str,
) {
    assert_eq!(
        stream.operations().len(),
        0,
        "{scenario_named}：录制流里一步都没有"
    );
    for ((identity, device), before) in devices.iter().zip(images_before) {
        assert!(
            device.wrapped_device().image == *before,
            "{scenario_named}：盘 {identity:?} 与 mkfs 之前逐字节相同"
        );
    }
}

/// 参数 512、两块盘都探测 4096：拒成参数与探测不同，点名盘 0（按盘表次序第一块不等的），带探测值 4096 与参数 512；一个字节不写。
#[test]
fn a_physical_block_size_parameter_of_512_on_devices_probing_4096_is_refused_before_any_write() {
    let stream = SharedStream::retaining_contents();
    let (mut devices, images_before) = patterned_recorded_devices([4096, 4096], &stream);
    let result = make_filesystem(&parameters_declaring(512), &mut devices);
    assert!(
        matches!(
            &result,
            Err(
                MakeFilesystemError::PhysicalBlockSizeParameterDiffersFromTheProbedValue {
                    device: DeviceIdentity(0),
                    probed_physical_block_size: PhysicalBlockSizeInBytes(4096),
                    physical_block_size_in_the_parameters: PhysicalBlockSizeInBytes(512),
                }
            )
        ),
        "参数 512、两块盘探测 4096：该拒成参数与探测不同、点名盘 0，实际 {:?}",
        result.as_ref().err()
    );
    assert_nothing_was_written(&stream, &devices, &images_before, "参数 512、探测 4096");
}

/// 参数 512、盘 0 探测 512、盘 1 探测 4096：判的是每一块盘，拒成参数与探测不同、点名盘 1；一个字节不写。
#[test]
fn a_physical_block_size_parameter_matching_only_device_zero_is_refused_naming_device_one_before_any_write(
) {
    let stream = SharedStream::retaining_contents();
    let (mut devices, images_before) = patterned_recorded_devices([512, 4096], &stream);
    let result = make_filesystem(&parameters_declaring(512), &mut devices);
    assert!(
        matches!(
            &result,
            Err(
                MakeFilesystemError::PhysicalBlockSizeParameterDiffersFromTheProbedValue {
                    device: DeviceIdentity(1),
                    probed_physical_block_size: PhysicalBlockSizeInBytes(4096),
                    physical_block_size_in_the_parameters: PhysicalBlockSizeInBytes(512),
                }
            )
        ),
        "参数 512、盘 0 探测 512、盘 1 探测 4096：该拒成参数与探测不同、点名盘 1，实际 {:?}",
        result.as_ref().err()
    );
    assert_nothing_was_written(
        &stream,
        &devices,
        &images_before,
        "参数 512、盘 1 探测 4096",
    );
}

/// 对照：参数与两块盘的探测值都是 4096，mkfs 照收。钉住「拒」只落在参数与探测不同的盘上。
#[test]
fn a_physical_block_size_parameter_equal_to_the_probed_value_on_every_device_still_makes_a_filesystem(
) {
    let stream = SharedStream::retaining_contents();
    let (mut devices, _) = patterned_recorded_devices([4096, 4096], &stream);
    let result = make_filesystem(&parameters_declaring(4096), &mut devices);
    assert!(
        result.is_ok(),
        "参数与两块盘的探测值都是 4096：{:?}",
        result.as_ref().err()
    );
}
