//! 腿自己的 N3 核对（只放进腿的草稿副本编，不进仓）：`on_device_modes::allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process`
//! 在内存里拼的根环表（照 core 的 `root_ring_occupancy_after_make_filesystem` 现拼，再补记暖机与新池新建文件那几条根），
//! 与 `run_new_pool_file_creation` 同一次跑里 core 自己的分配器（`allocator_after_make_filesystem` 装、`publish_first_file` 记）最后持有的根环表，`{:?}` 文本是不是相同。
//! 拼法逐行抄 on_device_modes.rs 里那个函数（冻结副本 crates/singlefs-checker-tier/src/on_device_modes.rs 第 189–235 行）。
mod common;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, RootRingOccupancy, RootRingOccupant};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::ROLLBACK_FLOOR_AT_MKFS;
use singlefs_core::root_ring::RootRingSlot;
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{e142_parameters, run_new_pool_file_creation_keeping_the_allocator};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[test]
fn the_ring_the_on_device_helper_assembles_equals_the_ring_core_holds_after_the_new_pool_file_creation() {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let (run, original_allocator) =
        run_new_pool_file_creation_keeping_the_allocator(&parameters, devices.as_mut_slice(), &stream, |_point, _devices| {})
            .expect("新池新建文件");
    // ── 以下抄 on_device_modes.rs 那个函数的函数体 ──
    let mut allocator = PoolAllocator::rebuild_from_records(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
        run.output.allocation_records.clone(),
    );
    let genesis_txg = CheckpointTxg(
        run.warm_up
            .roots
            .first()
            .expect("暖机写了根")
            .checkpoint_txg
            .0
            .checked_sub(1)
            .expect("暖机第一条根接在第 0 代根上"),
    );
    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        parameters.geometry.root_ring_slots_per_region,
        (0..ROOT_RING_REGIONS)
            .map(|region| {
                (
                    RootRingSlot { region, slot: 0 },
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: genesis_txg,
                    },
                )
            })
            .collect(),
        ROLLBACK_FLOOR_AT_MKFS,
        genesis_txg,
    ));
    for root in run
        .warm_up
        .roots
        .iter()
        .chain(std::iter::once(&run.output.root))
    {
        allocator.record_root_written_by_this_process(root.checkpoint_txg);
    }
    // ── 抄完 ──
    let assembled = format!("{:?}", allocator.root_ring_occupancy().expect("装了"));
    let held_by_core = format!("{:?}", original_allocator.root_ring_occupancy().expect("装了"));
    println!("genesis_txg={} roots={:?}", genesis_txg.0, run.warm_up.roots.iter().map(|r| r.checkpoint_txg.0).collect::<Vec<_>>());
    println!("ASSEMBLED {assembled}");
    assert_eq!(assembled, held_by_core, "拼出来的根环表与 core 自己持有的逐字相同");
}
