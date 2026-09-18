//! m2-wave2-code-r1 云端攻方腿（Opus）：未插桩副本上复核两处 panic（crates/ 与开工快照逐字节相同，只加这一个测试文件与它的共用模块）。

#[path = "opus_probe_common_pristine.rs"]
mod opus_probe_common;

use opus_probe_common::*;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::instance_table::InstanceTableRecords;
use singlefs_core::transaction::TransactionUnit;

fn upper_bound_passes(base: usize, planned: usize) -> bool {
    base + 10 + 8 * planned <= CAPACITY
}

#[test]
fn pristine_instance_table_rows_overflow_after_acquisition() {
    let mut pool = fresh_pool();
    let mut base = pool.current.allocation_records.len();
    for mount_number in 1..=400 {
        let superblocks_before = superblock_instances(&pool);
        match remount(&mut pool) {
            MountOutcome::Mounted(mounted) => {
                let row = mounted.output.row_publish.file_version().expect("带文件");
                let rows = InstanceTableRecords::parse(&row.unit(TransactionUnit::InstanceTable).bytes).expect("解得开").rows.len();
                let planned = mounted.output.warm_up_publishes.len();
                assert!(upper_bound_passes(base, planned));
                if mount_number % 100 == 0 || rows >= 368 {
                    println!("mount={mount_number} instance={} rows={rows} base={base} planned={planned} records_after={}", mounted.output.instance.0, pool.current.allocation_records.len());
                }
                base = pool.current.allocation_records.len();
            }
            MountOutcome::Refused(error) => panic!("mount={mount_number} refused {error:?}"),
            MountOutcome::Panicked(message) => {
                println!("mount={mount_number} PANICKED: {message}; base={base} (upper bound with 2 warm-ups: {} <= 812)", base + 26);
                println!("superblock instances before: {superblocks_before:?}; after: {:?}", superblock_instances(&pool));
                if let MountOutcome::Panicked(message) = remount(&mut pool) {
                    println!("retry PANICKED again: {message}; superblock instances now {:?}", superblock_instances(&pool));
                }
                return;
            }
        }
    }
    panic!("400 次挂载都没 panic");
}

fn overlaps_on_device_zero(records: &[AllocationRecord]) -> Vec<(AllocationRecord, AllocationRecord)> {
    let mut sorted: Vec<AllocationRecord> = records.iter().filter(|record| record.device.0 == 0).copied().collect();
    sorted.sort_by_key(AllocationRecord::sort_key);
    sorted
        .windows(2)
        .filter(|pair| pair[0].slot.0 + u64::from(pair[0].span_slots) > pair[1].slot.0)
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

#[test]
fn pristine_reuse_with_a_different_span_leaves_an_overlapping_record_and_the_next_mount_panics() {
    let mut pool = fresh_pool();
    let mut seed = 0usize;
    for mount_number in 1..=20 {
        match remount(&mut pool) {
            MountOutcome::Mounted(_) => {}
            MountOutcome::Refused(error) => panic!("mount={mount_number} refused {error:?}"),
            MountOutcome::Panicked(message) => {
                println!("mount={mount_number} PANICKED: {message}; superblock instances {:?}", superblock_instances(&pool));
                if let MountOutcome::Panicked(again) = remount(&mut pool) {
                    println!("mount={} PANICKED again: {again}", mount_number + 1);
                }
                return;
            }
        }
        for write in 1..=2 {
            seed += 1;
            let output = overwrite(&mut pool, seed).expect("覆盖写");
            let found = overlaps_on_device_zero(&output.allocation_records);
            if !found.is_empty() {
                println!("mount={mount_number} overwrite={write} txg={} overlapping records on device 0: {found:?}", output.root.checkpoint_txg.0);
            }
        }
    }
    panic!("20 次挂载都没 panic");
}
