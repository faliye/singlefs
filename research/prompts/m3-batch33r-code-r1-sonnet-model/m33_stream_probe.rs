//! harness 档探针（正推腿，不属于入库的测试）：照 `crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` 的
//! `record_the_stream` / `prepare` 录同一条流，打印每次发布的 (实例, txg)、段长、根槽写数、状态数闭式，核版本表里写的 3..9 代对不对。
//! 只用 harness 与 core，不碰 checker 档。输出行都以 `PROBE ` 开头，rerun.sh 取这些行。
#[path = "common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::transaction::{
    publish_sequential_write, publish_unlink_intent, publish_unlink_step, FirstFile, PoolWriter,
    TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    closed_form_state_count, publishes_in, writes_and_segments, RecordStreamContinuity,
};
use singlefs_harness::scenario::deletion_history_batch_size;
use singlefs_harness::segments::StepKind;

fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 11 + seed * 3 + 7) % 251).expect("小于 256"))
        .collect()
}

fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

#[test]
fn probe_the_unlink_after_the_multi_record_stream() {
    let unlinked = InodeNumber(FIRST_INODE_NUMBER);
    let mut pool = build_pool("m33-sonnet-probe");
    let first = pool.output.clone();
    let second = sequential_write(&mut pool, &content_needing(2, 1));
    let third = sequential_write(&mut pool, &content_needing(3, 2));
    let publish_parameters = parameters();
    let mut steps = Vec::new();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let previous = pool.output.clone();
        let intent = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            unlinked,
            FIXED_WRITE_TIME_SECONDS + 120,
            InstanceGeneration(1),
        )
        .expect("意图");
        pool.output = intent.clone();
        steps.push(intent);
    }
    for _ in 0..3 {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let previous = pool.output.clone();
        let step = publish_unlink_step(
            &mut writer,
            &mut pool.allocator,
            &previous,
            unlinked,
            deletion_history_batch_size(),
            InstanceGeneration(1),
        )
        .expect("一批");
        pool.output = step.clone();
        steps.push(step);
    }
    for (label, output) in [("A", &first), ("B", &second), ("C", &third)] {
        println!(
            "PROBE version {label} instance={} txg={} data_units={} nlink_record={:?}",
            output.root.instance.0,
            output.root.checkpoint_txg.0,
            output.data_pointers.len(),
            output.inode_record.map(|record| record.nlink)
        );
    }
    for (index, step) in steps.iter().enumerate() {
        println!(
            "PROBE deletion_step {index} instance={} txg={} data_units={} record_present={} nlink={:?} deleted_inodes={:?}",
            step.root.instance.0,
            step.root.checkpoint_txg.0,
            step.data_pointers.len(),
            step.inode_record.is_some(),
            step.inode_record.map(|record| record.nlink),
            step.deleted_inodes
        );
    }
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("PROBE segment_sizes {sizes:?}");
    println!("PROBE segment_count {}", sizes.len());
    let every_write_persisted = vec![true; writes.len()];
    let publishes = publishes_in(&writes, &every_write_persisted, RecordStreamContinuity::OneRecording);
    for publish in &publishes {
        println!(
            "PROBE publish instance={} txg={} units={} records={} root_write_index={}",
            publish.instance,
            publish.checkpoint_txg,
            publish.units.len(),
            publish.records.len(),
            publish.root
        );
    }
    let root_writes = writes.iter().filter(|write| write.kind == StepKind::RootRecordFua).count();
    println!("PROBE root_slot_fua_writes {root_writes}");
    println!("PROBE closed_form_two_states_per_write {}", closed_form_state_count(&segments));
    let expanded: Vec<Vec<usize>> = segments.iter().filter(|segment| segment.len() < 10).cloned().collect();
    println!("PROBE closed_form_of_segments_shorter_than_ten {}", closed_form_state_count(&expanded));
    let two_write_segments = sizes.iter().filter(|size| **size == 2).count();
    let one_write_segments = sizes.iter().filter(|size| **size == 1).count();
    println!("PROBE two_write_segments {two_write_segments} one_write_segments {one_write_segments}");
    let _ = file_content();
}
