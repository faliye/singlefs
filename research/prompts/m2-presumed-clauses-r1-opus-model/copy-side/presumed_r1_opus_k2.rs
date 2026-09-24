//! 副本装置（m2-presumed-clauses-r1 云端攻方，K2 / P3）：后续挂载、回退之后、挂载中途崩了再挂三种起点上，
//! `warm_up_publish_txgs` 现算的暖机次数够不够、有没有白推。每一行打印一个挂载；不断言，整张表交给报告判。
//! 每次挂载之后再发一版数据（fsync 返回），然后分别丢掉盘 0、盘 1 冷恢复，读回的必须是那一版——这是暖机要买的东西。
mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters,
    BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{mount_rollback, mount_writable, Mounted, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::crash::{writes_and_segments, MemoryPool};
use singlefs_harness::{RecordedOperationKind, RetainedOperation, SharedStream};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn device_of(txg: CheckpointTxg) -> DeviceIdentity {
    let target = target_for_publish(txg, parameters().geometry.root_ring_slots_per_region);
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

fn arm_name() -> String {
    format!(
        "cap={} jump={}",
        std::env::var("PRESUMED_R1_WARM_UP_CAP").unwrap_or_else(|_| "-".into()),
        std::env::var("PRESUMED_R1_ROW_TXG_JUMP").unwrap_or_else(|_| "-".into())
    )
}

/// 写行 txg、暖机 txg 列表、各落哪块盘、白推几次（落到已覆盖的盘上）、最后覆不覆盖两块盘。
fn describe(mounted: &Mounted) -> String {
    let row = mounted.output.row_publish.root().checkpoint_txg;
    let mut covered: BTreeSet<u32> = BTreeSet::new();
    covered.insert(device_of(row).0);
    let mut white = 0;
    let mut warm = Vec::new();
    for publish in &mounted.output.warm_up_publishes {
        let txg = publish.root().checkpoint_txg;
        let device = device_of(txg).0;
        if !covered.insert(device) {
            white += 1;
        }
        warm.push(format!("{}@d{}", txg.0, device));
    }
    format!(
        "inst={} chosen=({},{}) row={}@d{} row_mod3={} warm=[{}] warm_count={} white={} covered_both={}",
        mounted.output.instance.0,
        mounted.output.chosen_root.instance.0,
        mounted.output.chosen_root.checkpoint_txg.0,
        row.0,
        device_of(row).0,
        row.0 % 3,
        warm.join(","),
        mounted.output.warm_up_publishes.len(),
        white,
        covered.len() == 2
    )
}

fn publish_data<Device: BlockDevice>(
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mounted: &mut Mounted,
    content: &[u8],
) -> TransactionOutput {
    let current = mounted
        .current
        .file_version()
        .expect("带文件的一版")
        .clone();
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut mounted.allocator,
        &current,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 999,
        },
        mounted.output.instance,
    )
    .expect("挂载之后发一版数据")
}

/// 丢掉一块盘冷恢复：读回的内容是不是那一版、择到的根与施加后的根是谁。
fn disk_loss(image: &MemoryPool, expected: &[u8], data_txg: CheckpointTxg) -> (usize, String) {
    let mut failures = 0;
    let mut single_slot_failures = 0usize;
    let mut detail = Vec::new();
    // 故障模型 F1：只有那一版数据的根槽读不出（一个故障）。
    {
        let mut one_slot = image.clone();
        let target = target_for_publish(data_txg, parameters().geometry.root_ring_slots_per_region);
        one_slot.flip_byte(device_of(data_txg), slot_offset(target, 4096), 100);
        let report = recover(&one_slot, JournalPolicy::Consult);
        let ok = matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if content == expected);
        detail.push(format!(
            "F1_data_root_slot:{} effective={:?} applied={}",
            if ok { "ok" } else { "LOST" },
            report.effective_root.map(|(instance, txg)| (instance.0, txg.0)),
            report.journal.prefix_applied
        ));
        if !ok {
            single_slot_failures += 1;
        }
    }
    for lost in [0u32, 1] {
        // 整块盘掉了：那块盘留在池里（分配记录里的设备身份要认得出），内容换成一块从没写过的盘——每一处读回全 0、自证全不过。
        let survivor = MemoryPool {
            devices: image
                .devices
                .iter()
                .map(|(identity, device)| {
                    if identity.0 == lost {
                        (*identity, singlefs_harness::crash::SparseDevice::default())
                    } else {
                        (*identity, device.clone())
                    }
                })
                .collect(),
            device_size_in_bytes: image.device_size_in_bytes,
        };
        let report = recover(&survivor, JournalPolicy::Consult);
        let (ok, root) = match &report.outcome {
            RecoveryOutcome::FileRead { root, content } => (content == expected, format!("{root:?}")),
            other => (false, format!("{other:?}")),
        };
        if !ok {
            failures += 1;
        }
        let unverified = recover(&survivor, JournalPolicy::ConsultWithoutNamedVerification);
        let unverified_ok = matches!(&unverified.outcome, RecoveryOutcome::FileRead { content, .. } if content == expected);
        detail.push(format!(
            "lose_d{lost}:{} chosen={root} effective={:?} applied={} above_water={} verification_failed={} | without_named_verification:{} effective={:?} applied={}",
            if ok { "ok" } else { "LOST" },
            report.effective_root.map(|(instance, txg)| (instance.0, txg.0)),
            report.journal.prefix_applied,
            report.journal.above_water,
            report.journal.verification_failed,
            if unverified_ok { "ok" } else { "LOST" },
            unverified.effective_root.map(|(instance, txg)| (instance.0, txg.0)),
            unverified.journal.prefix_applied
        ));
    }
    detail.push(format!("F1_failures={single_slot_failures} F2_failures={failures}"));
    (failures + 1000 * single_slot_failures, detail.join(" "))
}

fn written_bytes(operations: &[RetainedOperation]) -> (u64, usize, usize) {
    let mut bytes = 0;
    let mut barriers = 0;
    let mut root_like_fua = 0;
    for retained in operations {
        match retained.operation.kind {
            RecordedOperationKind::Write | RecordedOperationKind::WriteZeroes => {
                bytes += retained.operation.length;
            }
            RecordedOperationKind::WriteForceUnitAccess => {
                bytes += retained.operation.length;
                root_like_fua += 1;
            }
            RecordedOperationKind::Barrier => barriers += 1,
        }
    }
    (bytes, barriers, root_like_fua)
}

fn overwrite_in_process(pool: &mut BuiltPool, seed: usize) {
    let previous = pool.output.clone();
    pool.output = common::publish_overwrite_in_process(
        pool,
        &previous,
        &content_of(2000 + seed * 13, seed),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
}

/// 一次正常重开（可写挂载）+ 发一版数据 + 丢盘检查；交回 (白推次数, 丢盘失败数)。
fn remount_publish_check(pool: &mut BuiltPool, label: &str, seed: usize) -> (usize, usize) {
    let before = pool.stream.operation_count();
    let mut devices = pool.reopen_recorded();
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let after_mount = pool.stream.operation_count();
    let line = describe(&mounted);
    let data = content_of(3000 + seed, seed + 101);
    let output = publish_data(&mut devices, &mut mounted, &data);
    pool.devices = Some(devices);
    pool.output = output.clone();
    pool.allocator = mounted.allocator.clone();
    let operations = pool.retained_operations();
    let (bytes, barriers, fua) = written_bytes(&operations[before..after_mount]);
    let (failures, detail) = disk_loss(&pool.memory_pool(), &data, output.root.checkpoint_txg);
    let white = mounted
        .output
        .warm_up_publishes
        .iter()
        .scan(BTreeSet::from([device_of(mounted.output.row_publish.root().checkpoint_txg).0]), |covered, publish| {
            Some(!covered.insert(device_of(publish.root().checkpoint_txg).0))
        })
        .filter(|was_white| *was_white)
        .count();
    println!(
        "K2 [{}] {label} {line} mount_bytes={bytes} mount_barriers={barriers} mount_fua={fua} data=({},{}) {detail}",
        arm_name(),
        output.root.instance.0,
        output.root.checkpoint_txg.0
    );
    (white, failures)
}

/// 后续挂载：实例 1 里第一个文件之后再发 k 版（txg 3 + k 收尾），关掉重开；再接一串「重开、发一版」。
#[test]
fn k2_subsequent_mounts_sweep() {
    let mut total_white = 0;
    let mut total_failures = 0;
    let mut mounts = 0;
    for extra in 0..=5usize {
        let mut pool = build_pool(&format!("k2-sub-{extra}"));
        for seed in 0..extra {
            overwrite_in_process(&mut pool, seed + 1);
        }
        for round in 0..4usize {
            let (white, failures) =
                remount_publish_check(&mut pool, &format!("sub extra={extra} round={round}"), extra * 10 + round);
            total_white += white;
            total_failures += failures;
            mounts += 1;
        }
    }
    println!(
        "K2-SUMMARY [{}] subsequent mounts={mounts} white_publishes={total_white} disk_loss_failures={total_failures}",
        arm_name()
    );
}

/// 只重开、不写：稳态下每次挂载推几次、白推几次。
#[test]
fn k2_remount_only_chain() {
    for extra in 0..=2usize {
        let mut pool = build_pool(&format!("k2-chain-{extra}"));
        for seed in 0..extra {
            overwrite_in_process(&mut pool, seed + 1);
        }
        let mut sequence = Vec::new();
        for round in 0..6usize {
            let before = pool.stream.operation_count();
            let mut devices = pool.reopen_recorded();
            let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
            pool.devices = Some(devices);
            let operations = pool.retained_operations();
            let (bytes, barriers, fua) = written_bytes(&operations[before..]);
            println!(
                "K2 [{}] chain extra={extra} round={round} {} mount_bytes={bytes} mount_barriers={barriers} mount_fua={fua}",
                arm_name(),
                describe(&mounted)
            );
            sequence.push(1 + mounted.output.warm_up_publishes.len());
        }
        println!(
            "K2-CHAIN [{}] extra={extra} publishes_per_mount={sequence:?}",
            arm_name()
        );
    }
}

/// 回退之后：实例 1 发到 txg 3 + k，回退到 (1, 3) 或 (1, 3 + k − 1)，发一版、丢盘检查；再正常重开一次、发一版、丢盘检查。
#[test]
fn k2_rollback_sweep() {
    let mut total_failures = 0;
    let mut mounts = 0;
    for extra in 1..=5usize {
        for target_txg in [3u64, 3 + u64::try_from(extra).expect("小") - 1] {
            let mut pool = build_pool(&format!("k2-rb-{extra}-{target_txg}"));
            for seed in 0..extra {
                overwrite_in_process(&mut pool, seed + 1);
            }
            let before = pool.stream.operation_count();
            let mut devices = pool.reopen_recorded();
            let target = RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(target_txg),
            };
            let mut mounted = match mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On) {
                Ok(mounted) => mounted,
                Err(error) => {
                    println!("K2 [{}] rollback extra={extra} target=(1,{target_txg}) REFUSED {error:?}", arm_name());
                    pool.devices = Some(devices);
                    continue;
                }
            };
            let after_mount = pool.stream.operation_count();
            let line = describe(&mounted);
            let data = content_of(3100 + extra, extra + 7);
            let output = publish_data(&mut devices, &mut mounted, &data);
            pool.devices = Some(devices);
            pool.output = output.clone();
            pool.allocator = mounted.allocator.clone();
            let operations = pool.retained_operations();
            let (bytes, barriers, fua) = written_bytes(&operations[before..after_mount]);
            let (failures, detail) = disk_loss(&pool.memory_pool(), &data, output.root.checkpoint_txg);
            total_failures += failures;
            mounts += 1;
            println!(
                "K2 [{}] rollback extra={extra} target=(1,{target_txg}) {line} mount_bytes={bytes} mount_barriers={barriers} mount_fua={fua} data=({},{}) {detail}",
                arm_name(),
                output.root.instance.0,
                output.root.checkpoint_txg.0
            );
            let (_white, failures) = remount_publish_check(
                &mut pool,
                &format!("after-rollback extra={extra} target=(1,{target_txg})"),
                extra * 100 + usize::try_from(target_txg).expect("小"),
            );
            total_failures += failures;
            mounts += 1;
        }
    }
    println!(
        "K2-SUMMARY [{}] rollback mounts={mounts} disk_loss_failures={total_failures}",
        arm_name()
    );
}

/// 挂载中途崩了再挂：把一次可写挂载（或回退）的写切段，每个段前缀加段内子集当一个崩溃状态，崩溃状态上再可写挂载、发一版、丢盘检查。
#[test]
fn k2_crash_during_mount_then_remount_sweep() {
    let mut states = 0;
    let mut remount_errors = 0;
    let mut total_failures = 0;
    let mut white_by_row_mod3 = [0usize; 3];
    let mut count_by_row_mod3 = [0usize; 3];
    for (extra, rollback) in [(0usize, false), (1, false), (2, false), (2, true), (3, true)] {
        let mut pool = build_pool(&format!("k2-crash-{extra}-{rollback}"));
        for seed in 0..extra {
            overwrite_in_process(&mut pool, seed + 1);
        }
        let before = pool.stream.operation_count();
        let base = {
            let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
            image.apply(&pool.retained_operations()[..before]);
            image
        };
        let mut devices = pool.reopen_recorded();
        if rollback {
            mount_rollback(
                &parameters(),
                &mut devices,
                RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(3),
                },
                ShadowLedger::On,
            )
            .expect("回退");
        } else {
            mount_writable(&parameters(), &mut devices).expect("可写挂载");
        }
        pool.devices = Some(devices);
        let operations = pool.retained_operations();
        let (writes, segments) = writes_and_segments(&operations[before..], &geometry());
        let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
        for crash_segment in 0..=segments.len() {
            let mut prefix = vec![false; writes.len()];
            for segment in &segments[..crash_segment] {
                for write in segment {
                    prefix[*write] = true;
                }
            }
            persisted_sets.push(prefix.clone());
            if crash_segment == segments.len() {
                continue;
            }
            let segment = &segments[crash_segment];
            let width = segment.len();
            let subsets: Vec<u64> = if width <= 10 {
                (1..(1u64 << width) - 1).collect()
            } else {
                let full = (1u64 << width) - 1;
                (0..width)
                    .flat_map(|bit| [1u64 << bit, full ^ (1u64 << bit)])
                    .collect()
            };
            for subset in subsets {
                let mut persisted = prefix.clone();
                for (position, write) in segment.iter().enumerate() {
                    if subset & (1 << position) != 0 {
                        persisted[*write] = true;
                    }
                }
                persisted_sets.push(persisted);
            }
        }
        println!(
            "K2 [{}] crash-source extra={extra} rollback={rollback} writes={} segments={:?} states={}",
            arm_name(),
            writes.len(),
            segments.iter().map(Vec::len).collect::<Vec<_>>(),
            persisted_sets.len()
        );
        for (state_index, persisted) in persisted_sets.iter().enumerate() {
            states += 1;
            let stream = SharedStream::retaining_contents();
            let mut crash_devices = crash_state_devices(&base, &writes, persisted, &stream);
            let mut mounted = match mount_writable(&parameters(), &mut crash_devices) {
                Ok(mounted) => mounted,
                Err(error) => {
                    remount_errors += 1;
                    println!("K2 [{}] crash extra={extra} rollback={rollback} state={state_index} REMOUNT_ERROR {error:?}", arm_name());
                    continue;
                }
            };
            let row_mod3 = usize::try_from(mounted.output.row_publish.root().checkpoint_txg.0 % 3).expect("小");
            let line = describe(&mounted);
            let white = mounted
                .output
                .warm_up_publishes
                .iter()
                .scan(BTreeSet::from([device_of(mounted.output.row_publish.root().checkpoint_txg).0]), |covered, publish| {
                    Some(!covered.insert(device_of(publish.root().checkpoint_txg).0))
                })
                .filter(|was_white| *was_white)
                .count();
            white_by_row_mod3[row_mod3] += white;
            count_by_row_mod3[row_mod3] += 1;
            let data = content_of(2900 + state_index % 97, state_index);
            let output = publish_data(&mut crash_devices, &mut mounted, &data);
            let image = memory_pool_of_sparse_devices(&crash_devices);
            let (failures, detail) = disk_loss(&image, &data, output.root.checkpoint_txg);
            total_failures += failures;
            if failures > 0 || state_index % 50 == 0 {
                println!(
                    "K2 [{}] crash extra={extra} rollback={rollback} state={state_index} {line} data=({},{}) {detail}",
                    arm_name(),
                    output.root.instance.0,
                    output.root.checkpoint_txg.0
                );
            }
        }
    }
    println!(
        "K2-SUMMARY [{}] crash-then-remount states={states} remount_errors={remount_errors} disk_loss_failures={total_failures} mounts_by_row_mod3={count_by_row_mod3:?} white_by_row_mod3={white_by_row_mod3:?}",
        arm_name()
    );
}
