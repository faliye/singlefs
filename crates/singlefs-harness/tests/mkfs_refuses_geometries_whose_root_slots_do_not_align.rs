//! mkfs 拒根槽、系统配置槽不按声明的判定宽度 max(physical_block_size, io_min) 对齐的几何，在任何写之前
//! （用户 2026-10-01 定「非常规几何由 mkfs 拒」，`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行、第四节；
//! C47（根槽没按判定宽度对齐））。两组几何取自正推腿 A3 一节与攻方腿第四节量过的那两组：
//! pbs 4096 + io_min 3072 改之前在 4096 宽的盘上清完环、写完单元与根，写到系统配置槽 1 才被块设备拒；
//! pbs 512 + io_min 3072 改之前整个收下，24 个根槽没有一个是 io_min 的整数倍。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemError, MakeFilesystemParameters, SlotJudgedForAlignment,
};
use singlefs_core::root_ring::{RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-m3-algn";
/// io_min 3072：不是 2 的幂，D2（RAID 条带策略） 已定项 19 的式子算出槽距 6144。
const MINIMUM_INPUT_OUTPUT_BYTES_THAT_IS_NOT_A_POWER_OF_TWO: u32 = 3072;

/// 一组 mkfs 参数：声明的 pbs 与 io_min 由调用方给，槽距照 D2（RAID 条带策略） 已定项 19 由 io_min 算。
fn parameters_declaring(
    physical_block_size: u32,
    minimum_input_output_bytes: u32,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size,
            minimum_input_output_bytes,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
                minimum_input_output_bytes,
            ),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 两块内存盘，块宽 `device_block_bytes`，外面包录制器、录进同一条流：mkfs 发出的每一个写、写零、屏障都在流里。
fn recorded_devices(
    device_block_bytes: u32,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(
                        DEVICE_BYTES,
                        PhysicalBlockSizeInBytes(device_block_bytes),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 在任何写之前拒：录制流一步都没有（一个写、一次写零、一道屏障都没发），两块盘的镜像还是空的。
fn assert_nothing_was_written(
    stream: &SharedStream,
    devices: &[(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)],
    geometry_named: &str,
) {
    assert_eq!(
        stream.operations().len(),
        0,
        "{geometry_named}：录制流里一步都没有"
    );
    for (identity, device) in devices {
        assert_eq!(
            device.wrapped_device().image,
            SparseDevice::default(),
            "{geometry_named}：盘 {identity:?} 一个扇区都没写"
        );
    }
}

/// pbs 4096 + io_min 3072（槽距 6144），4096 宽的盘：判定宽度 4096，按排布次序第一个不对齐的是系统配置槽 1（偏移 6144）。
/// 改之前清完根环与 journal 环、写完两个单元与三条第 0 代根，写到系统配置槽 1 才被块设备报 `Unaligned`，盘上留下半份格式。
#[test]
fn physical_block_size_4096_with_io_min_3072_is_refused_at_system_configuration_slot_one_before_any_write(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices(4096, &stream);
    let result = make_filesystem(
        &parameters_declaring(4096, MINIMUM_INPUT_OUTPUT_BYTES_THAT_IS_NOT_A_POWER_OF_TWO),
        &mut devices,
    );
    assert!(
        matches!(
            &result,
            Err(
                MakeFilesystemError::FixedStructureSlotNotAlignedToTheDecisionWidth {
                    slot: SlotJudgedForAlignment::SystemConfigurationSlot { slot_index: 1 },
                    offset: DeviceOffsetInBytes(6144),
                    decision_width_in_bytes: 4096,
                }
            )
        ),
        "pbs 4096 + io_min 3072：该报系统配置槽 1 在 6144 处不是 4096 的整数倍，实际 {:?}",
        result.as_ref().err()
    );
    assert_nothing_was_written(&stream, &devices, "pbs 4096 + io_min 3072");
}

/// pbs 512 + io_min 3072（槽距 6144），512 宽的盘：判定宽度 3072；系统配置两槽（0、6144）都对齐，
/// 按排布次序第一个不对齐的是根环区域 0 的槽 0（区域起点 1 MiB，1048576 ÷ 3072 不整除）。改之前 mkfs 整个收下。
#[test]
fn physical_block_size_512_with_io_min_3072_is_refused_at_the_first_root_slot_before_any_write() {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices(512, &stream);
    let result = make_filesystem(
        &parameters_declaring(512, MINIMUM_INPUT_OUTPUT_BYTES_THAT_IS_NOT_A_POWER_OF_TWO),
        &mut devices,
    );
    assert!(
        matches!(
            &result,
            Err(
                MakeFilesystemError::FixedStructureSlotNotAlignedToTheDecisionWidth {
                    slot: SlotJudgedForAlignment::RootRingSlot(RootRingSlot { region: 0, slot: 0 }),
                    offset: DeviceOffsetInBytes(1_048_576),
                    decision_width_in_bytes: 3072,
                }
            )
        ),
        "pbs 512 + io_min 3072：该报区域 0 槽 0 在 1 MiB 处不是 3072 的整数倍，实际 {:?}",
        result.as_ref().err()
    );
    assert_nothing_was_written(&stream, &devices, "pbs 512 + io_min 3072");
}

/// 对照：pbs 512 + io_min 4096（槽距 4096）判定宽度 4096，两个系统配置槽与 24 个根槽都对齐，mkfs 照收。
/// 钉住「拒」只落在不对齐的几何上，不是新判定把常规几何也挡了。
#[test]
fn physical_block_size_512_with_io_min_4096_still_makes_a_filesystem() {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices(512, &stream);
    let result = make_filesystem(&parameters_declaring(512, 4096), &mut devices);
    assert!(
        result.is_ok(),
        "pbs 512 + io_min 4096 是常规几何：{:?}",
        result.as_ref().err()
    );
}
