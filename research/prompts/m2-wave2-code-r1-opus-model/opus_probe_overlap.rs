//! m2-wave2-code-r1 云端攻方腿（Opus）Z1 旁支探针：复用已回收的记录时跨度变了，旧记录没被改写，盘上两条记录罩住同一个槽，
//! 下一次可写挂载在 `rebuild_from_records` → `mark_allocated` 的断言上 panic（在取号之前那道准入之前）。

mod opus_probe_common;

use opus_probe_common::*;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::crash::MemoryPool;

/// 同一块盘上按槽号排好的记录里，前一条的跨度罩到后一条的起点。
fn overlaps(records: &[AllocationRecord]) -> Vec<(AllocationRecord, AllocationRecord)> {
    let mut sorted: Vec<AllocationRecord> = records.to_vec();
    sorted.sort_by_key(AllocationRecord::sort_key);
    sorted
        .windows(2)
        .filter(|pair| pair[0].device == pair[1].device && pair[0].slot.0 + u64::from(pair[0].span_slots) > pair[1].slot.0)
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

fn image_of(pool: &Pool) -> MemoryPool {
    MemoryPool {
        devices: pool.devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect::<std::collections::BTreeMap<_, _>>(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn describe(output: &TransactionOutput) -> String {
    format!(
        "txg={} rewritten={:?} units={:?}",
        output.root.checkpoint_txg.0,
        output.rewritten,
        output.units.iter().filter(|unit| output.rewritten.contains(&unit.identity)).map(|unit| (unit.identity, unit.slot.0)).collect::<Vec<_>>()
    )
}

fn report_first_overlap(pool: &Pool, output: &TransactionOutput, context: &str) -> bool {
    let found = overlaps(&output.allocation_records);
    if found.is_empty() {
        return false;
    }
    println!("FIRST OVERLAP after {context}: publish {}", describe(output));
    for (left, right) in found.iter().filter(|(left, _)| left.device.0 == 0) {
        println!("  overlapping records on device 0: {left:?}  and  {right:?}");
    }
    let verdicts = check_pool_image(&image_of(pool));
    let red: Vec<String> = verdicts
        .iter()
        .filter(|(_, verdict)| format!("{verdict:?}").contains("Violat") || format!("{verdict:?}").contains("Red") || format!("{verdict:?}").contains("Fail"))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect();
    println!("  checker on the image right after this publish: {} verdicts, non-green: {red:?}", verdicts.len());
    true
}

fn run(per_session: usize, max_mounts: usize) {
    let mut pool = fresh_pool();
    let mut seed = 0usize;
    let mut overlap_seen = false;
    for mount_number in 1..=max_mounts {
        let outcome = remount(&mut pool);
        let _ = take_probe_log();
        match outcome {
            MountOutcome::Mounted(mounted) => {
                let mut outputs = vec![mounted.output.row_publish.file_version().expect("带文件").clone()];
                outputs.extend(mounted.output.warm_up_publishes.iter().map(|version| version.file_version().expect("带文件").clone()));
                for output in &outputs {
                    if !overlap_seen && report_first_overlap(&pool, output, &format!("per_session={per_session} mount={mount_number} (row/warm-up)")) {
                        overlap_seen = true;
                    }
                }
            }
            MountOutcome::Refused(error) => {
                println!("per_session={per_session} mount={mount_number} refused {error:?}");
                return;
            }
            MountOutcome::Panicked(message) => {
                println!("per_session={per_session} mount={mount_number} PANICKED at mount: {message}; overlap seen before = {overlap_seen}");
                return;
            }
        }
        for write in 0..per_session {
            seed += 1;
            match overwrite(&mut pool, seed) {
                Ok(output) => {
                    if !overlap_seen && report_first_overlap(&pool, &output, &format!("per_session={per_session} mount={mount_number} overwrite={}", write + 1)) {
                        overlap_seen = true;
                    }
                }
                Err(error) => {
                    println!("per_session={per_session} mount={mount_number} overwrite refused {error:?}");
                    break;
                }
            }
        }
    }
    println!("per_session={per_session}: {max_mounts} mounts, no panic; overlap seen = {overlap_seen}");
}

#[test]
fn overlapping_records_after_reuse_panic_the_next_mount() {
    let max_mounts = std::env::var("OPUS_OVERLAP_MOUNTS").ok().and_then(|value| value.parse().ok()).unwrap_or(60);
    for per_session in [0usize, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20] {
        run(per_session, max_mounts);
    }
}

fn non_green(pool: &Pool) -> Vec<String> {
    check_pool_image(&image_of(pool))
        .iter()
        .filter(|(_, verdict)| format!("{verdict:?}").starts_with("Violated"))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect()
}

/// 定点：每次挂载之后覆盖写 2 次，第 8 次挂载之后第 1 次覆盖写造出重叠；那一次之前、之后各跑一遍 checker，再挂载一次。
#[test]
fn overlap_pinpoint_per_session_two() {
    let mut pool = fresh_pool();
    let mut seed = 0usize;
    println!("checker after first transaction: {:?}", non_green(&pool));
    for mount_number in 1..=8 {
        match remount(&mut pool) {
            MountOutcome::Mounted(_) => {}
            _ => panic!("前 8 次挂载都要成立"),
        }
        let _ = take_probe_log();
        println!("mount={mount_number} instance={} checker after mount: {:?}", pool.instance.0, non_green(&pool));
        let writes = if mount_number == 8 { 1 } else { 2 };
        for write in 0..writes {
            seed += 1;
            let before = non_green(&pool);
            let output = overwrite(&mut pool, seed).expect("覆盖写");
            let after = non_green(&pool);
            let found = overlaps(&output.allocation_records);
            println!(
                "mount={mount_number} overwrite={} txg={} overlaps_dev0={:?} checker_before={before:?} checker_after={after:?}",
                write + 1,
                output.root.checkpoint_txg.0,
                found.iter().filter(|(left, _)| left.device.0 == 0).collect::<Vec<_>>()
            );
        }
    }
    let superblocks_before = superblock_instances(&pool);
    match remount(&mut pool) {
        MountOutcome::Panicked(message) => println!("mount=9 PANICKED: {message}; superblocks before {superblocks_before:?} after {:?}", superblock_instances(&pool)),
        MountOutcome::Mounted(_) => println!("mount=9 mounted (no panic)"),
        MountOutcome::Refused(error) => println!("mount=9 refused {error:?}"),
    }
    let _ = take_probe_log();
    match remount(&mut pool) {
        MountOutcome::Panicked(message) => println!("mount=10 PANICKED again: {message}"),
        _ => println!("mount=10 did not panic"),
    }
}
