//! 内存池的快照、拷贝与逐扇区读回比较。

use super::*;

pub(super) fn memory_pool_snapshot(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 反方向：从一份镜像重建出可写的设备 Vec（c2 崩溃之后在切出来的镜像上可写挂载要这一条）。
pub(super) fn devices_from_memory_pool(
    pool: &MemoryPool,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device =
                SparseBlockDevice::new(pool.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

/// 同一份镜像的一套可写副本，每块盘外面包一层共用 `stream` 的录制器（W6 录写切段、PQ2 录抬 F 那一串）。
pub(super) fn recording_copies_of(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_from_memory_pool(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 两份镜像读回来不同的扇区：(盘, 扇区起点的字节偏移)，按盘、偏移升序。
pub(super) fn sectors_read_back_differently(
    first: &MemoryPool,
    second: &MemoryPool,
) -> Vec<(u32, u64)> {
    let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
    let mut differing = Vec::new();
    for (identity, first_device) in &first.devices {
        let Some(second_device) = second.devices.get(identity) else {
            continue;
        };
        let written_sectors: BTreeSet<u64> = first_device
            .written_sectors_in(DeviceOffsetInBytes(0), first.device_size_in_bytes)
            .into_iter()
            .chain(
                second_device
                    .written_sectors_in(DeviceOffsetInBytes(0), second.device_size_in_bytes),
            )
            .collect();
        for sector in written_sectors {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            if first_device.read(offset, sector_bytes) != second_device.read(offset, sector_bytes) {
                differing.push((identity.0, offset.0));
            }
        }
    }
    differing
}

/// 两份镜像读回来的字节是不是逐字节相同：两边写过的扇区取并，逐扇区读回比（稀疏盘上「写过一扇区全 0」与「没写过」读回相同，
/// 按读回的字节比，不按内部表示比）。
pub(super) fn memory_pools_read_back_identically(first: &MemoryPool, second: &MemoryPool) -> bool {
    if !first.devices.keys().eq(second.devices.keys()) {
        return false;
    }
    first.devices.iter().all(|(identity, first_device)| {
        let second_device = &second.devices[identity];
        let written_sectors: BTreeSet<u64> = first_device
            .written_sectors_in(DeviceOffsetInBytes(0), first.device_size_in_bytes)
            .into_iter()
            .chain(
                second_device
                    .written_sectors_in(DeviceOffsetInBytes(0), second.device_size_in_bytes),
            )
            .collect();
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        written_sectors.iter().all(|sector| {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            first_device.read(offset, sector_bytes) == second_device.read(offset, sector_bytes)
        })
    })
}
