//! 探针（m4-batch2-code-r1 正推腿，checker 档，只编不跑，交主 agent 在提交时跑）：
//! 一次 discard 落在基镜像里有非零字节的单元区槽上，崩溃点重放的撕裂判据（crash.rs 的 is_tearable_in_place_overwrite）把它认成原地覆写（三态）还是两态。
//! 判决 G5 与 harness 的 memory_pool.rs 注释都说两态；D13 已定项 4 对原地覆写的字面是「不是单元写、长于一个扇区、罩住的范围里原来有东西」。

use singlefs_checker_tier::crash::TearableInPlaceOverwrites;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 1 << 30;
const UNIT_OFFSET: u64 = 50176 * 16384;

fn tearable_of_one_discard(length: u64) -> (bool, Vec<usize>) {
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: 768 << 20,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    };
    let stream = SharedStream::retaining_contents();
    let mut recorder = RecordingBlockDevice::with_shared_stream(
        DeviceIdentity(0),
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
        stream.clone(),
    );
    recorder
        .write_at(DeviceOffsetInBytes(UNIT_OFFSET), &[0xC3u8; 16384], WriteDurability::Plain)
        .expect("旧写");
    recorder.barrier().expect("旧写的屏障");
    let base_operations = stream.retained_operations();
    recorder
        .discard_at(DeviceOffsetInBytes(UNIT_OFFSET), length)
        .expect("discard");
    recorder.barrier().expect("屏障");
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], DEVICE_BYTES);
    base.apply(&base_operations);
    let operations = stream.retained_operations()[base_operations.len()..].to_vec();
    let (writes, _segments) = writes_and_segments(&operations, &geometry);
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].kind, StepKind::ZeroFill);
    let tearable = TearableInPlaceOverwrites::of(&base, &writes);
    (tearable.contains(0), tearable.write_indexes())
}

#[test]
fn probe_a_sixteen_kib_discard_over_old_bytes_is_tearable_or_not() {
    let (contains, indexes) = tearable_of_one_discard(16384);
    println!("PROBE_T7 discard_16384_over_old_bytes contains={contains} write_indexes={indexes:?}");
}

#[test]
fn probe_a_one_sector_discard_over_old_bytes_is_not_tearable() {
    let (contains, indexes) = tearable_of_one_discard(512);
    println!("PROBE_T7 discard_512_over_old_bytes contains={contains} write_indexes={indexes:?}");
    assert_eq!(contains, false);
}
