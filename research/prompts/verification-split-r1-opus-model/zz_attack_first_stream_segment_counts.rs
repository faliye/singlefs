//! 攻方腿原型（verification-split-r1）：第一条流在内存稀疏盘上录一遍（照 tests/common/mod.rs 的 build_pool，镜像换成 SparseBlockDevice），
//! 打印段长、写数、两态闭式、三态全量与甲二快档的闭式数；不枚举崩溃状态。
mod common;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};
use singlefs_harness::crash::{
    closed_form_state_count, full_expansion, layer0_state_count, layer0_state_count_with_torn_in_place_overwrites,
    quick_tier_expansion, writes_and_segments, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

#[test]
fn print_the_first_stream_segment_counts() {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content = file_content();
    let parameters = parameters();
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个事务");
    }
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..mkfs_operation_count]);
    let (writes, segments) = writes_and_segments(&operations[mkfs_operation_count..], &geometry());
    let lengths: Vec<usize> = segments.iter().map(Vec::len).collect();
    let unit_writes = writes.iter().filter(|write| write.kind == StepKind::UnitWrite).count();
    println!("ATTACK_FIRST_STREAM mkfs_operations={mkfs_operation_count} writes={} unit_writes={unit_writes} segments={lengths:?}", writes.len());
    println!(
        "ATTACK_FIRST_STREAM closed_form_two_states={} quick_two_states={} quick_torn={} full_torn={}",
        closed_form_state_count(&segments),
        layer0_state_count(&writes, &segments, &quick_tier_expansion),
        layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &quick_tier_expansion),
        layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion),
    );
    // removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker 的切法照搬
    let mut after = operations[mkfs_operation_count..].to_vec();
    let root_position = after
        .iter()
        .rposition(|retained| geometry().classify(&retained.operation) == StepKind::RootRecordFua)
        .expect("有根槽写");
    let first_barrier_before_the_root = after[..root_position]
        .iter()
        .rposition(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
        .expect("根槽之前有写")
        + 1;
    let barrier_steps = root_position - first_barrier_before_the_root;
    after.drain(first_barrier_before_the_root..root_position);
    let (_writes_without, segments_without) = writes_and_segments(&after, &geometry());
    println!(
        "ATTACK_FIRST_STREAM barrier_steps_before_root={barrier_steps} segments_without_that_barrier={:?}",
        segments_without.iter().map(Vec::len).collect::<Vec<_>>()
    );
}
