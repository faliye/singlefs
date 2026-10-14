//! 稀疏内存盘当 `BlockDevice` 交给可写挂载时给恢复的 journal 提示（`SparseBlockDevice::journal_record_offsets_hint`，经
//! `PoolReader for [(DeviceIdentity, Device)]` 传进 `scan_journal`）要与真盘上的全环扫描等价：同一组盘上跑两遍 `scan_journal`，
//! 一遍照常（给提示）、一遍把提示换成 None，读出的记录逐条相同；提示恰好是环里写过的、落在记录槽开头的扇区，
//! 与内存池当读口子时给的那一份相同（同一个算法）。
//!
//! 没有这份提示，稀疏盘上的每次可写挂载把 768 MiB 的 journal 环按 4096 逐槽读一遍（崩溃放量的观察者在每个崩溃状态上挂一次：
//! 2026-10-12 量到一次挂载 341 ms，几乎全花在这一扫上）；提示错了，挂载落到的那一版就与真盘不同。

mod common;

use std::collections::BTreeSet;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::recovery::{choose_system_configuration, scan_journal, PoolReader};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_harness::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SECTOR_BYTES};

/// 同一个读口子，只是不给 journal 提示：恢复照真盘那样按 4096 对齐逐槽扫全环。
struct WithoutJournalHint<'reader, Reader: PoolReader>(&'reader Reader);

impl<Reader: PoolReader> PoolReader for WithoutJournalHint<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.0.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.0.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.0.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

/// 池的每块盘照观察者那样搬进一块稀疏盘：挂载读到的字节与内存池相同。
fn sparse_block_devices_of(pool: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let physical_block_size =
        PhysicalBlockSizeInBytes(common::parameters().geometry.physical_block_size);
    pool.devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(pool.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

#[test]
fn the_sparse_block_device_hint_finds_what_the_full_ring_scan_finds_after_mkfs_warm_up_and_the_first_file(
) {
    let built = common::build_pool("sparse-device-journal-hint-scan");
    let pool = built.memory_pool();
    let devices = sparse_block_devices_of(&pool);
    let system_configuration =
        choose_system_configuration(&devices).expect("mkfs 之后的池择得出系统配置");
    let by_the_hint = scan_journal(&devices, &system_configuration);
    let by_the_full_ring_scan = scan_journal(&WithoutJournalHint(&devices), &system_configuration);
    assert_eq!(
        by_the_hint, by_the_full_ring_scan,
        "按提示扫与全环扫描读出的记录要逐条相同"
    );
    assert!(
        by_the_full_ring_scan.len() >= 2,
        "取号加两次暖机至少留两条记录，两边都空的比对不算数：读到 {} 条",
        by_the_full_ring_scan.len()
    );
}

#[test]
fn the_slice_pool_reader_passes_the_sparse_block_device_hint_through_as_the_record_slots_with_written_sectors(
) {
    let built = common::build_pool("sparse-device-journal-hint-slots");
    let pool = built.memory_pool();
    let devices = sparse_block_devices_of(&pool);
    let system_configuration =
        choose_system_configuration(&devices).expect("mkfs 之后的池择得出系统配置");
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    for (identity, device) in &devices {
        let through_the_slice_reader: BTreeSet<DeviceOffsetInBytes> =
            PoolReader::journal_record_offsets_hint(
                &devices[..],
                *identity,
                ring_start,
                ring_bytes,
            )
            .expect("稀疏盘给得出提示，切片读口子要原样传给恢复")
            .into_iter()
            .collect();
        let by_the_device: BTreeSet<DeviceOffsetInBytes> = device
            .journal_record_offsets_hint(ring_start, ring_bytes)
            .expect("稀疏盘记着写过的扇区，总给得出提示")
            .into_iter()
            .collect();
        let written_sectors_at_record_slot_starts: BTreeSet<DeviceOffsetInBytes> = device
            .image
            .written_sectors_in(ring_start, ring_bytes)
            .into_iter()
            .map(|sector| sector * SECTOR_BYTES)
            .filter(|offset| (offset - ring_start.0).is_multiple_of(JOURNAL_RECORD_BYTES))
            .map(DeviceOffsetInBytes)
            .collect();
        let by_the_memory_pool: BTreeSet<DeviceOffsetInBytes> =
            PoolReader::journal_record_offsets_hint(&pool, *identity, ring_start, ring_bytes)
                .expect("内存池总给得出提示")
                .into_iter()
                .collect();
        assert_eq!(
            through_the_slice_reader, by_the_device,
            "盘 {identity:?}：切片读口子传给恢复的就是盘自己答的"
        );
        assert_eq!(
            by_the_device, written_sectors_at_record_slot_starts,
            "盘 {identity:?}：提示恰好是环里写过的、落在记录槽开头的扇区"
        );
        assert_eq!(
            by_the_device, by_the_memory_pool,
            "盘 {identity:?}：与内存池当读口子时给的提示相同"
        );
        assert!(
            !by_the_device.is_empty(),
            "盘 {identity:?}：暖机的记录每块盘各一份，提示不空"
        );
    }
}

/// 随机历史与崩溃注入把稀疏盘包在注入故障的盘里再挂载（`HistoryDevice`）：包一层提示照里面那块盘答，按提示扫与全环扫描仍然逐条相同。
#[test]
fn the_fault_injecting_device_passes_the_sparse_block_device_hint_through_and_the_scans_still_agree(
) {
    let built = common::build_pool("sparse-device-journal-hint-fault-injecting");
    let pool = built.memory_pool();
    let plan = SharedFaultPlan::unarmed(common::geometry());
    let devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        sparse_block_devices_of(&pool)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let system_configuration =
        choose_system_configuration(&devices).expect("mkfs 之后的池择得出系统配置");
    let by_the_hint = scan_journal(&devices, &system_configuration);
    let by_the_full_ring_scan = scan_journal(&WithoutJournalHint(&devices), &system_configuration);
    assert_eq!(
        by_the_hint, by_the_full_ring_scan,
        "包在注入故障的盘里：按提示扫与全环扫描读出的记录要逐条相同"
    );
    assert!(
        by_the_full_ring_scan.len() >= 2,
        "取号加两次暖机至少留两条记录：读到 {} 条",
        by_the_full_ring_scan.len()
    );
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    for (identity, device) in &devices {
        let through_the_wrapper = device
            .journal_record_offsets_hint(ring_start, ring_bytes)
            .expect("注入故障的盘把里面稀疏盘的提示原样答出来");
        let by_the_sparse_device = device
            .wrapped_device()
            .journal_record_offsets_hint(ring_start, ring_bytes)
            .expect("稀疏盘总给得出提示");
        assert_eq!(
            through_the_wrapper, by_the_sparse_device,
            "盘 {identity:?}：包一层不改提示"
        );
    }
}
