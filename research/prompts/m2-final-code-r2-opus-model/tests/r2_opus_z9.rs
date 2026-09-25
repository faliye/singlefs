//! m2-final-code-r2 云端攻方 Z9：回退见证。只在冻结副本的拷贝上跑。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod common;

use common::{
    build_pool, crash_state_devices, file_content, parameters, publish_overwrite_in_process,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_rollback, mount_writable, MountError, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot_holds_a_root, readable_roots,
    recover, rollback_witness_of_the_pool, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::target_for_publish;
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::fault_injection::{
    NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget,
};
use singlefs_harness::{RecordedOperationKind, RetainedOperation, SharedStream};

fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

/// A（txg 3）之后覆盖写到 txg `last_txg`，实例 1。
fn pool_through(tag: &str, last_txg: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    let mut current = pool.output.clone();
    let mut seed = 1;
    while current.root.checkpoint_txg.0 < last_txg {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(seed),
            FIXED_WRITE_TIME_SECONDS + 60 * seed as u64,
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        seed += 1;
    }
    pool.output = current;
    pool
}

/// 一次挂载的录制流里，写行那次发布的根 FUA 在第几步（这次挂载的第一次 FUA 写）。
fn first_root_write(ops: &[RetainedOperation]) -> usize {
    ops.iter()
        .position(|op| op.operation.kind == RecordedOperationKind::WriteForceUnitAccess)
        .expect("写过根")
}

/// 根 FUA 之后紧跟的系统配置轮换（两盘各一次，偏移 < 8192）走完的那一步。
fn end_of_rotation_after(ops: &[RetainedOperation], root: usize) -> usize {
    let mut index = root + 1;
    while index < ops.len()
        && ops[index].operation.kind == RecordedOperationKind::Write
        && ops[index].operation.offset.0 < 8192
    {
        index += 1;
    }
    assert_eq!(index - root - 1, 2, "两盘各一次轮换");
    index
}

#[derive(Clone, Copy, Debug)]
enum Cut {
    /// 写行那次发布的轮换落盘之后、暖机之前崩。
    AfterTheRowPublishRotation,
    /// 不崩，整次挂载写完。
    Never,
    /// 暖机第一次的记录落了、它的根 FUA 之前崩。
    BeforeTheFirstWarmUpRoot,
}

#[derive(Clone, Copy, Debug)]
enum TargetPolicy {
    /// 回退目标 R0 在环里、是候选就一直回退到它；不在了改回退到最新根。
    StickyThenNewest(RollbackTarget),
    /// 每次回退到最新根（空回退）。
    Newest,
    /// 每次回退到环里最旧的一条候选根（不被见证抛弃、不在被实例表抛弃的时间线上，逐个试）。
    OldestCandidate,
}

fn roots_of(image: &MemoryPool) -> Vec<(InstanceGeneration, CheckpointTxg)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let mut roots: Vec<_> = readable_roots(
        image,
        &sc.immutable.region_devices,
        &sc.immutable.sizes,
        &sc.immutable.filesystem_identifier,
    )
    .iter()
    .map(|root| (root.instance, root.checkpoint_txg))
    .collect();
    roots.sort_by_key(|(i, t)| (*t, *i));
    roots
}

/// 连着回退 `mounts` 次，每次照 `cut` 崩或不崩。返回每一步的一行报告与最后的镜像。
fn rollback_series(
    tag: &str,
    last_txg: u64,
    policy: TargetPolicy,
    cut: Cut,
    mounts: usize,
) -> (Vec<String>, MemoryPool, Option<String>) {
    let pool = pool_through(tag, last_txg);
    let mut image = pool.memory_pool();
    drop(pool);
    let mut lines = Vec::new();
    for k in 1..=mounts {
        let sc = choose_system_configuration(&image).expect("系统配置");
        let all_readable = every_root_ring_slot_holds_a_root(&image, &sc).is_some();
        let witness_before = rollback_witness_of_the_pool(&image, &sc).entries();
        let newest = choose_root(&image, &sc).expect("有根");
        let targets: Vec<RollbackTarget> = match policy {
            TargetPolicy::StickyThenNewest(r0) => vec![
                r0,
                RollbackTarget { instance: newest.instance, checkpoint_txg: newest.checkpoint_txg },
            ],
            TargetPolicy::Newest => vec![RollbackTarget {
                instance: newest.instance,
                checkpoint_txg: newest.checkpoint_txg,
            }],
            TargetPolicy::OldestCandidate => roots_of(&image)
                .into_iter()
                .map(|(instance, checkpoint_txg)| RollbackTarget { instance, checkpoint_txg })
                .collect(),
        };
        let mut outcome = None;
        for target in targets {
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            match mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On) {
                Ok(mounted) => {
                    let ops = stream.retained_operations();
                    let persisted = match cut {
                        Cut::AfterTheRowPublishRotation => {
                            end_of_rotation_after(&ops, first_root_write(&ops))
                        }
                        Cut::Never => ops.len(),
                        Cut::BeforeTheFirstWarmUpRoot => {
                            let first = first_root_write(&ops);
                            first + 1 + first_root_write(&ops[first + 1..])
                        }
                    };
                    image.apply(&ops[..persisted]);
                    outcome = Some(Ok((target, mounted)));
                    break;
                }
                Err(MountError::RollbackTargetNotACandidate { .. }) => continue,
                Err(other) => {
                    outcome = Some(Err((target, other)));
                    break;
                }
            }
        }
        match outcome {
            Some(Ok((target, mounted))) => {
                let written: Vec<u64> = std::iter::once(&mounted.output.row_publish)
                    .chain(&mounted.output.warm_up_publishes)
                    .map(|v| v.root().checkpoint_txg.0)
                    .collect();
                lines.push(format!(
                    "k={k} ring_all_readable={all_readable} witness_entries_before={} target=({},{}) new_instance={} roots_written_txg={:?} persisted_roots={}",
                    witness_before.len(),
                    target.instance.0,
                    target.checkpoint_txg.0,
                    mounted.output.instance.0,
                    written,
                    match cut { Cut::AfterTheRowPublishRotation | Cut::BeforeTheFirstWarmUpRoot => 1, Cut::Never => written.len() },
                ));
            }
            Some(Err((target, error))) => {
                let line = format!(
                    "k={k} ring_all_readable={all_readable} witness_entries_before={} target=({},{}) REFUSED {:?}",
                    witness_before.len(),
                    target.instance.0,
                    target.checkpoint_txg.0,
                    error
                );
                lines.push(line.clone());
                return (lines, image, Some(line));
            }
            None => {
                let line = format!("k={k} no candidate accepted");
                lines.push(line.clone());
                return (lines, image, Some(line));
            }
        }
    }
    (lines, image, None)
}

fn print(title: &str, lines: &[String]) {
    println!("=== {title}");
    for line in lines {
        println!("{line}");
    }
}

/// Z9-A1：回退目标一直是 (1, 29)，每次回退挂载崩在写行那次的轮换之后、暖机之前（一次挂载只落一条根）。
#[test]
fn z9_a1_same_target_crash_after_row_rotation_fills_the_table() {
    let (lines, _image, stop) = rollback_series(
        "z9a1",
        30,
        TargetPolicy::StickyThenNewest(RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(29),
        }),
        Cut::AfterTheRowPublishRotation,
        30,
    );
    print("Z9-A1 sticky (1,29) then newest, crash after row-publish rotation", &lines);
    println!("stop: {stop:?}");
}

/// Z9-A2：每次回退到最新根（空回退），每次崩在写行那次的轮换之后。
#[test]
fn z9_a2_newest_target_crash_after_row_rotation_fills_the_table() {
    let (lines, _image, stop) =
        rollback_series("z9a2", 30, TargetPolicy::Newest, Cut::AfterTheRowPublishRotation, 30);
    print("Z9-A2 newest, crash after row-publish rotation", &lines);
    println!("stop: {stop:?}");
}

/// Z9-A 对照：不崩，三种回退目标取法各连着回退 40 次。
#[test]
fn z9_a_control_without_crash_three_target_policies() {
    for (name, policy) in [
        (
            "sticky (1,29) then newest",
            TargetPolicy::StickyThenNewest(RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(29),
            }),
        ),
        ("newest", TargetPolicy::Newest),
        ("oldest candidate", TargetPolicy::OldestCandidate),
    ] {
        let (lines, _image, stop) = rollback_series("z9ac", 30, policy, Cut::Never, 40);
        let max_entries = lines
            .iter()
            .filter_map(|l| l.split("witness_entries_before=").nth(1))
            .filter_map(|r| r.split(' ').next())
            .filter_map(|n| n.parse::<usize>().ok())
            .max();
        println!("=== control {name}: mounts={} max_witness_entries_before={max_entries:?} stop={stop:?}", lines.len());
        if let Some(last) = lines.last() {
            println!("last: {last}");
        }
    }
}

fn slot_target(txgs: &[u64]) -> RootRingSlotTarget {
    let slots: Vec<_> = txgs
        .iter()
        .map(|txg| target_for_publish(CheckpointTxg(*txg), parameters().geometry.root_ring_slots_per_region))
        .collect();
    RootRingSlotTarget {
        named_slots: NamedRootRingSlots::naming(&slots),
        region_devices: parameters().region_devices,
        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
    }
}

/// 一次整次写完的可写挂载。
fn full_writable_mount(image: &mut MemoryPool) {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    mount_writable(&parameters(), &mut devices).expect("可写挂载");
    image.apply(&stream.retained_operations());
}

/// Z9-B：回退挂载崩在写行那次的根 FUA 之后、轮换之前（见证那一次写没落）；之后用户照常可写挂载 m 次（m = 0..=3 扫一遍）。
/// 对照：同一段历史，回退挂载不崩。两边都把「比被抛弃的 C（1, 5）新、且没被抛弃」的全部根设成读不出，看择根落在哪。
#[test]
fn z9_b_crash_between_row_root_and_its_rotation_never_gets_the_witness() {
    for m in 0..=3usize {
        for crash in [true, false] {
            let pool = pool_through(&format!("z9b-{m}-{crash}"), 5);
            let mut image = pool.memory_pool();
            drop(pool);
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            mount_rollback(
                &parameters(),
                &mut devices,
                RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },
                ShadowLedger::On,
            )
            .expect("回退到 (1, 3)");
            let ops = stream.retained_operations();
            let persisted = if crash { first_root_write(&ops) + 1 } else { ops.len() };
            image.apply(&ops[..persisted]);
            for _ in 0..m {
                full_writable_mount(&mut image);
            }
            let sc = choose_system_configuration(&image).expect("系统配置");
            let witness = rollback_witness_of_the_pool(&image, &sc).entries();
            let newest = choose_root(&image, &sc).expect("有根");
            let rows = singlefs_core::recovery::instance_table_of_root(&image, &newest)
                .map(|table| {
                    table
                        .rows
                        .iter()
                        .map(|row| format!("({},{},{},{})", row.instance.0, row.selected_root_txg.0, row.applied_transaction_high_water, if row.is_rollback { "rollback" } else { "-" }))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            println!("m={m} crash={crash} newest_root=({},{}) its_instance_table_rows=[{rows}]", newest.instance.0, newest.checkpoint_txg.0);
            let newer_than_c: Vec<u64> = roots_of(&image)
                .into_iter()
                .filter(|(i, t)| (t.0, i.0) > (5, 1))
                .map(|(_, t)| t.0)
                .collect();
            let reader = PoolReaderWithUnreadableRootRingSlots::new(&image, slot_target(&newer_than_c));
            let report = recover(&reader, JournalPolicy::Consult);
            let chosen = match &report.outcome {
                RecoveryOutcome::FileRead { root, content } => format!(
                    "FileRead root=({},{}) content_is_A={} content_len={}",
                    root.0 .0,
                    root.1 .0,
                    content == &file_content(),
                    content.len()
                ),
                other => format!("{other:?}"),
            };
            println!(
                "m={m} crash_between_row_root_and_rotation={crash} witness={:?} unreadable_root_txgs={:?} faults={} -> {chosen}",
                witness.iter().map(|e| (e.new_instance.0, e.rollback_target_instance.0, e.rollback_target_txg.0)).collect::<Vec<_>>(),
                newer_than_c,
                newer_than_c.len()
            );
        }
    }
}

/// Z9-A3：每次回退到最新根，每次崩在暖机第一次的根 FUA 之前（那一次的记录已落、根没落）。
#[test]
fn z9_a3_newest_target_crash_before_the_first_warm_up_root() {
    let (lines, _image, stop) =
        rollback_series("z9a3", 30, TargetPolicy::Newest, Cut::BeforeTheFirstWarmUpRoot, 40);
    print("Z9-A3 newest, crash before first warm-up root (its record persisted)", &lines);
    println!("stop: {stop:?}");
}
