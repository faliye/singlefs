//! harness 档探针二（正推腿，不属于入库的测试）：新流的一批代表状态（前 k 段全持久；再加第 k 段只落第一个写；共 60 个左右，
//! 都在层 0 枚举域里）上，用 core 的 `recover`（看 journal 那一遍）与可写挂载，复现新测试的观察者与 oracle 要核的东西：
//! 恢复走哪个根、读回什么、盘上最新根槽是哪个；可写挂载续做几步、走到空没有、挂完是哪一种池版本。
//! 输出行以 `PROBE2 ` 开头；判对错的是 `judge_states.py`（读这些行，按版本表与续做步数的期望判）。不碰 checker 档。
#[path = "common/mod.rs"]
mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    ShadowLedger,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    publish_sequential_write, publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    newest_persisted_root, writes_and_segments, CrashImage, SparseBlockDevice,
};
use singlefs_harness::scenario::{deletion_history_batch_size, deletion_history_switches};

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
fn probe_representative_states_recover_and_mount() {
    let unlinked = InodeNumber(FIRST_INODE_NUMBER);
    let mut pool = build_pool("m33-sonnet-state-probe");
    let content_a = file_content();
    let content_b = content_needing(2, 1);
    let content_c = content_needing(3, 2);
    let _second = sequential_write(&mut pool, &content_b);
    let _third = sequential_write(&mut pool, &content_c);
    let publish_parameters = parameters();
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
        pool.output = intent;
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
        pool.output = step;
    }
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    // 状态：(标签, 持久集合)。
    let mut states: Vec<(String, Vec<bool>)> = Vec::new();
    let mut prefix = vec![false; writes.len()];
    for (segment_index, segment) in segments.iter().enumerate() {
        states.push((format!("prefix{segment_index:02}"), prefix.clone()));
        if segment.len() >= 2 {
            let mut first_only = prefix.clone();
            first_only[segment[0]] = true;
            states.push((format!("prefix{segment_index:02}+first_write_of_segment"), first_only));
        }
        for write in segment {
            prefix[*write] = true;
        }
    }
    states.push(("all_persisted".to_string(), prefix));
    let results: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    let next = AtomicUsize::new(0);
    let worker_threads: usize = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    std::thread::scope(|scope| {
        for _ in 0..worker_threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some((label, persisted)) = states.get(index) else { break };
                let image = CrashImage { base: &base, writes: &writes, persisted: persisted.clone() };
                let newest = newest_persisted_root(image.writes, &image.persisted);
                let report = recover(&image, JournalPolicy::Consult);
                let outcome = match &report.outcome {
                    RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
                    RecoveryOutcome::FileRead { content, .. } => {
                        if *content == content_a { "A".to_string() }
                        else if *content == content_b { "B".to_string() }
                        else if *content == content_c { "C".to_string() }
                        else { format!("other{}", content.len()) }
                    }
                    RecoveryOutcome::Failed { failure, .. } => format!("Failed({failure:?})"),
                };
                let mut materialized = base.clone();
                for (write, is_persisted) in writes.iter().zip(&image.persisted) {
                    if *is_persisted {
                        materialized.apply_writes(std::slice::from_ref(write));
                    }
                }
                let mount_parameters = parameters();
                let mut devices: Vec<_> = materialized
                    .devices
                    .iter()
                    .map(|(identity, sectors)| {
                        let mut device = SparseBlockDevice::new(
                            materialized.device_size_in_bytes,
                            PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
                        );
                        device.image = sectors.clone();
                        (*identity, device)
                    })
                    .collect();
                let mounted_text = match mount_writable_with_test_only_switches_and_deletion_switches(
                    &mount_parameters,
                    &mut devices,
                    SpaceAdmission::JudgedByTheFormula,
                    ShadowLedger::On,
                    deletion_history_switches(),
                ) {
                    Ok(mounted) => {
                        let current = match &mounted.current {
                            PoolVersion::WithFile(_) => "WithFile",
                            PoolVersion::WithoutFile(_) => "WithoutFile",
                            PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
                        };
                        let walked = matches!(
                            mounted.output.deletion_queue_continuation_outcome,
                            DeletionQueueContinuationOutcome::WalkedToEmpty
                        );
                        format!(
                            "mount=ok steps={} walked_to_empty={walked} current={current}",
                            mounted.output.deletion_queue_continuation.len()
                        )
                    }
                    Err(error) => format!("mount=ERR {error:?}").replace('\n', " "),
                };
                let line = format!(
                    "PROBE2 state={label} newest_persisted_root(txg,instance)={:?} effective_root(instance,txg)={:?} outcome={outcome} {mounted_text}",
                    newest.map(|(txg, instance)| (txg.0, instance.0)),
                    report.effective_root.map(|(instance, txg)| (instance.0, txg.0)),
                );
                results.lock().expect("锁").push((index, line));
            });
        }
    });
    let mut lines = results.into_inner().expect("锁");
    lines.sort();
    for (_, line) in &lines {
        println!("{line}");
    }
    println!("PROBE2 states_judged={}", lines.len());
}
