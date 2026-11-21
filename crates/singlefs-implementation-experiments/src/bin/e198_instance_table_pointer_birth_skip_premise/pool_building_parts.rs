use super::*;

// ============================================================================
// 三、建池的几样共用件：参数、录制盘、确定性内容。
// ============================================================================

pub(super) fn pool_parameters(
    slots_per_region: u64,
    journal_ring_bytes: u64,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: REGION_DEVICES.map(DeviceIdentity),
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING,
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("登记的 S 都在 [4, 16] 区间内"),
        },
    }
}

pub(super) fn fixed_geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 确定性内容（写法同线索模型 `lib.rs` 的 `content`）：同一个盐同一个长度两次逐字节相同。
pub(super) fn deterministic_content(salt: u64, length: usize) -> Vec<u8> {
    let salt_term = usize::try_from(salt % 251).expect("模 251 装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index + salt_term * 7 + 3) % 251).expect("模 251 装得进 u8"))
        .collect()
}

pub(super) fn device_identities() -> [DeviceIdentity; 2] {
    [DeviceIdentity(0), DeviceIdentity(1)]
}

pub(super) type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 以 `image` 为起点的两块录制盘，共用一条录制流。每一步（一次挂载、一次发布）各起一份：镜像在这一步之前与盘上字节逐字节相同。
pub(super) fn recording_devices_of(image: &MemoryPool, stream: &SharedStream) -> RecordedDevices {
    device_identities()
        .into_iter()
        .map(|identity| {
            let mut device = SparseBlockDevice::new(
                DEVICE_BYTES,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            if let Some(sparse) = image.devices.get(&identity) {
                device.image = sparse.clone();
            }
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}
