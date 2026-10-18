//! 崩溃镜像读者（`CrashImage`）与落成池的读者（`MemoryPool`）给恢复的候选单元槽要相同：一次 32 KiB 的数据单元写跨两个 16 KiB 的槽，
//! 两个槽都是候选。早先崩溃镜像只报每次单元写起头的那个槽，候选集比落成池少一半，恢复在两种读者上扫到的单元不同。
mod common;

use common::{build_pool, geometry, IMAGE_BYTES};
use singlefs_checker::image::ImageReader;
use singlefs_core::address::DeviceIdentity;
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{writes_and_segments, CrashImage, MemoryPool};
use singlefs_harness::segments::StepKind;

#[test]
fn a_crash_image_names_every_slot_of_a_data_unit_write_as_a_candidate_like_the_materialized_pool() {
    let pool = build_pool("candidate-unit-slots-of-a-crash-image");
    let operations = pool.retained_operations();
    let devices = [DeviceIdentity(0), DeviceIdentity(1)];
    let mut base = MemoryPool::with_devices(&devices, IMAGE_BYTES);
    base.apply(&operations[..pool.mkfs_operation_count]);
    let (writes, _segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert!(
        writes.iter().any(|write| {
            write.kind == StepKind::UnitWrite && write.length_in_bytes() == DATA_UNIT_BYTES
        }) && DATA_UNIT_BYTES > SLOT_BYTES,
        "前提：暖机与新池新建文件写了至少一个跨两个槽的数据单元"
    );
    let mut materialized = base.clone();
    materialized.apply_writes(&writes);
    let image = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    for device in devices {
        assert_eq!(
            ImageReader::candidate_unit_slots(&image, device.0),
            ImageReader::candidate_unit_slots(&materialized, device.0),
            "盘 {} 上两种读者给的候选单元槽要相同",
            device.0
        );
    }
}
