use super::*;

// ============================================================================================
// 池的造法：参数、两块等大的稀疏盘（包上计数）、mkfs、挂载结局的分类。
// ============================================================================================

pub(super) type CountedDevice = CountingBlockDevice<SparseBlockDevice>;
pub(super) type CountedPool = Vec<(DeviceIdentity, CountedDevice)>;

pub(super) fn make_filesystem_parameters(
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_BYTES,
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("每区槽数落在 4..16 闭区间"),
        },
    }
}

pub(super) fn counted_pool(
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> CountedPool {
    let regions = FixedRegions::of_geometry(
        u64::from(FIXED_STRUCTURE_SLOT_SPACING_BYTES),
        slots_per_region,
        journal_ring_bytes,
    );
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                CountingBlockDevice::new(
                    SparseBlockDevice::new(
                        device_bytes,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                    ),
                    regions,
                ),
            )
        })
        .collect()
}

/// 两块盘各自此刻的计数，按盘序。
pub(super) fn tallies_of(pool: &CountedPool) -> Vec<DeviceTally> {
    pool.iter().map(|(_, device)| device.tally_now()).collect()
}

/// 每块盘从 `earlier` 到此刻的计数之差。
pub(super) fn tally_difference(pool: &CountedPool, earlier: &[DeviceTally]) -> Vec<DeviceTally> {
    pool.iter()
        .zip(earlier)
        .map(|((_, device), before)| device.tally_now().minus(*before))
        .collect()
}

/// 第一个文件的内容：定长 3000 字节，随序号变（新池新建文件的内容）。
pub(super) fn first_file_content() -> Vec<u8> {
    (0..3000usize)
        .map(|index| u8::try_from((index + 3) % 251).expect("小于 256"))
        .collect()
}

/// 报错只取头一个标识符（变体名）：`Debug` 全文可能带整张结构，产物里不留。
pub(super) fn variant_name_of(debug_text: &str) -> String {
    debug_text
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect()
}

#[cfg(test)]
mod tests;
