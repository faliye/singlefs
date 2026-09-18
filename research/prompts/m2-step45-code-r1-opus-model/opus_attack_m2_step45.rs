//! 云端攻方腿（m2-step45-code-r1）在副本上的攻击装置。原仓不含本文件。
#![allow(clippy::too_many_lines, reason = "一格一个攻击历史，整段读得出")]

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_superblock, instance_table_of_root, recover,
    readable_roots, rollback_high_water_of_root, scan_journal, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 把某个 txg 的根槽改坏（等价于「那次发布的根写还没落盘」这个崩溃点上读到的字节）。
fn damage_root_slot(pool: &mut BuiltPool, txg: u64) {
    let device = region_device(txg);
    let offset = slot_offset(target_for_publish(CheckpointTxg(txg)), 4096);
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("那块盘");
    let mut bytes = vec![0u8; 4096];
    recorded.read_at(offset, &mut bytes).expect("读根槽");
    bytes[100] ^= 0xff;
    recorded
        .write_at(offset, &bytes, WriteDurability::Plain)
        .expect("改坏根槽");
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    pool
}

fn rollback_to(pool: &mut BuiltPool, target: RollbackTarget, shadow: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled = mount_rollback(&parameters(), &mut devices, target, shadow).expect("回退");
    pool.devices = Some(devices);
    pool.allocator = rolled.allocator.clone();
    pool.output = rolled.current.clone();
    rolled
}

fn slots_under_root(pool: &BuiltPool, instance: u32, txg: u64) -> BTreeSet<(u32, u64)> {
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let root = readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        root.instance == InstanceGeneration(instance) && root.checkpoint_txg == CheckpointTxg(txg)
    })
    .expect("那条根在环里");
    allocation_records_under_root(&image, &root)
        .expect("那条根的账")
        .into_iter()
        .flat_map(|record| {
            (record.slot.0..record.slot.0 + u64::from(record.span_slots))
                .map(move |slot| (record.device.0, slot))
        })
        .collect()
}

/// X2：P1 在「崩在回退生效之前」这个窗口里丢掉一条已提交的发布。
/// 前缀：A（txg 3，jsn 3）落盘；B（txg 4，jsn 4）的记录落盘、根写还没落盘（发布顺序 单元 → 记录 → 根）。
/// 甲（不发起回退）：下一次恢复择 A 的根、施加 B 那条记录、读回 B 的内容。
/// 乙（发起回退到 A，崩在 D 的根之前）：D 的 jsn = R_old 的 jsn 3 + 1 = 4，盖掉 B 那条记录；
///   同一个崩溃点上恢复择 A 的根、一条记录都施加不了、读回 A 的内容，B 这次已提交的发布在盘上一个字节都不剩。
#[test]
fn x2_the_p1_record_slot_overwrite_destroys_a_committed_publish_in_the_window_the_clause_calls_equivalent(
) {
    // 甲：不发起回退。
    let mut plain = build_pool("opus-x2-plain");
    overwrite_in_process(&mut plain, &second_content(), InstanceGeneration(1));
    plain.devices = Some(plain.reopen_recorded());
    damage_root_slot(&mut plain, 4);
    let plain_image = plain.memory_pool();
    let plain_superblock = choose_superblock(&plain_image).expect("超级块");
    let plain_records = scan_journal(&plain_image, &plain_superblock);
    let plain_report = recover(&plain_image, JournalPolicy::Consult);
    println!(
        "甲 记录键 {:?}",
        plain_records.keys().collect::<Vec<_>>()
    );
    println!("甲 恢复 {:?} / {:?}", plain_report.effective_root.map(|root| (root.0.0, root.1.0)), plain_report.journal);
    assert_eq!(
        (plain_report.journal.prefix_applied, plain_report.effective_root),
        (1, Some((InstanceGeneration(1), CheckpointTxg(4)))),
        "甲：B 的记录已提交，恢复施加它"
    );
    assert_eq!(
        plain_report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: second_content()
        },
        "甲：读回 B 的内容（outcome 里的 root 是所选的那条）"
    );

    // 乙：同一个前缀上发起回退到 A，再崩在回退那次发布的根之前。
    let mut rolled = build_pool("opus-x2-rollback");
    overwrite_in_process(&mut rolled, &second_content(), InstanceGeneration(1));
    rolled.devices = Some(rolled.reopen_recorded());
    damage_root_slot(&mut rolled, 4);
    let mounted = rollback_to(
        &mut rolled,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    let rollback_txgs: Vec<u64> = std::iter::once(mounted.output.row_publish.root.checkpoint_txg.0)
        .chain(
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|publish| publish.root.checkpoint_txg.0),
        )
        .collect();
    println!(
        "乙 回退那次发布 txg {:?}，jsn {}",
        rollback_txgs, mounted.output.row_publish.record.counter
    );
    // 崩在 D 的根之前：回退那次发布写出的每条根都当没落盘。
    for txg in &rollback_txgs {
        damage_root_slot(&mut rolled, *txg);
    }
    let rolled_image = rolled.memory_pool();
    let rolled_superblock = choose_superblock(&rolled_image).expect("超级块");
    let rolled_records = scan_journal(&rolled_image, &rolled_superblock);
    let rolled_report = recover(&rolled_image, JournalPolicy::Consult);
    println!(
        "乙 记录键 {:?}",
        rolled_records.keys().collect::<Vec<_>>()
    );
    println!("乙 恢复 {:?} / {:?}", rolled_report.effective_root.map(|root| (root.0.0, root.1.0)), rolled_report.journal);
    assert!(
        !rolled_records.contains_key(&(InstanceGeneration(1), 4)),
        "B 那条记录被回退那次发布的记录盖掉了"
    );
    assert_eq!(
        (rolled_report.journal.prefix_applied, rolled_report.effective_root),
        (0, Some((InstanceGeneration(1), CheckpointTxg(3)))),
        "乙：同一个崩溃点，一条记录都施加不了"
    );
    assert_eq!(
        rolled_report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: common::file_content()
        },
        "乙：读回 A 的内容，B 丢了"
    );
}

/// X4：`rollback_high_water_of_root` 在回退之后的镜像上对环里每一条可读根都是 None——前缀第五条一次都不触发。
#[test]
fn x4_the_rollback_high_water_predicate_is_none_for_every_readable_root_after_a_rollback() {
    let mut pool = build_through_third_publish("opus-x4");
    rollback_to(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let roots = readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let mut some_count = 0;
    for root in &roots {
        let high_water = rollback_high_water_of_root(&image, root);
        let table = instance_table_of_root(&image, root).map(|table| {
            table
                .rows
                .iter()
                .map(|row| {
                    (
                        row.instance.0,
                        row.selected_root_txg.0,
                        row.applied_transaction_high_water,
                        row.is_rollback,
                    )
                })
                .collect::<Vec<_>>()
        });
        println!(
            "根 (实例 {}, txg {}) 的表 {:?} ⇒ 第五条的 W = {:?}",
            root.instance.0, root.checkpoint_txg.0, table, high_water
        );
        if high_water.is_some() {
            some_count += 1;
        }
    }
    assert_eq!(roots.len(), 11, "环里的可读根");
    assert_eq!(some_count, 0, "一条根都没有触发前缀第五条");
}

/// X1 甲：回退之后回退实例的两条根都读不出（那正是步 4 用例自己造的那个状态），最新根变成被抛弃的 C——
/// 于是候选集按 C 那张旧表判，B 的根 (1, 4) 与 C 自己的根 (2, 8) 都能被退到。
#[test]
fn x1_with_the_rollback_instances_roots_unreadable_the_candidate_set_accepts_abandoned_roots() {
    let mut pool = build_through_third_publish("opus-x1-stale-table");
    let mounted = rollback_to(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    let rollback_txgs: Vec<u64> = std::iter::once(mounted.output.row_publish.root.checkpoint_txg.0)
        .chain(
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|publish| publish.root.checkpoint_txg.0),
        )
        .collect();
    drop(mounted);
    for txg in &rollback_txgs {
        damage_root_slot(&mut pool, *txg);
    }
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let newest = choose_root(&image, &superblock).expect("最新根");
    println!(
        "回退实例的根 {:?} 都读不出之后，最新根 = (实例 {}, txg {})",
        rollback_txgs, newest.instance.0, newest.checkpoint_txg.0
    );
    assert_eq!(
        (newest.instance, newest.checkpoint_txg),
        (InstanceGeneration(2), CheckpointTxg(8)),
        "最新根是被抛弃时间线的 C"
    );
    let mut accepted_abandoned = 0;
    for (instance, txg) in [(1u32, 4u64), (2, 6), (2, 7), (2, 8)] {
        let target = RollbackTarget {
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(txg),
        };
        let mut devices = pool.reopen_recorded();
        let accepted = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        match accepted {
            Ok(mounted) => {
                accepted_abandoned += 1;
                println!(
                    "退到被抛弃的根 (实例 {instance}, txg {txg}) 被接受：新实例 {}，第一个新根 txg {}，读回的版本 (实例 {}, txg {})",
                    mounted.output.instance.0,
                    mounted.output.row_publish.root.checkpoint_txg.0,
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0
                );
            }
            Err(error) => println!(
                "退到被抛弃的根 (实例 {instance}, txg {txg}) 被拒：{error:?}（不是候选集拒的）"
            ),
        }
    }
    assert!(
        accepted_abandoned > 0,
        "至少一条被抛弃时间线的根被候选集放过"
    );
}

/// X1 乙：最新根指着的实例表单元两份都读不出时，回退到一条按每种读法都在候选集里的根（A，(1, 3)）也被拒——
/// 而同一个镜像上普通可写挂载也走不通，回退这条退路被同一个坏单元一起关掉。
#[test]
fn x1_an_unreadable_newest_instance_table_refuses_a_target_that_is_in_the_candidate_set() {
    let mut pool = build_through_third_publish("opus-x1-malformed");
    let table_locations = pool.output.root.instance_table.locations;
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for location in &table_locations {
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("那块盘");
            let offset = location.slot.to_device_offset();
            let mut bytes = vec![0u8; 512];
            recorded.read_at(offset, &mut bytes).expect("读单元头");
            bytes[200] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏实例表单元");
        }
    }
    println!("改坏的实例表落点 {:?}", table_locations.iter().map(|location| (location.device.0, location.slot.0)).collect::<Vec<_>>());
    let mut devices = pool.reopen_recorded();
    let refused = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    let refused_kind = format!("{:?}", refused.as_ref().err());
    pool.devices = Some(devices);
    println!("回退到 (1, 3) 的结果：{refused_kind}");
    assert!(
        matches!(refused, Err(MountError::InstanceTableMalformed)),
        "{refused_kind}"
    );
    let mut devices = pool.reopen_recorded();
    let plain = mount_writable(&parameters(), &mut devices);
    let plain_kind = format!("{:?}", plain.as_ref().err());
    pool.devices = Some(devices);
    println!("同一个镜像上普通可写挂载的结果：{plain_kind}");
    assert!(plain.is_err(), "{plain_kind}");
}

/// X3：影子账只住内存、只在 `mount_rollback` 里算——回退之后重开一次（普通可写挂载），
/// 隔离整批消失，只被被抛弃根引用的槽当场可分配，而被抛弃的根一条都没离开根环。
#[test]
fn x3_a_plain_remount_after_the_rollback_drops_the_whole_isolation_and_hands_out_abandoned_slots() {
    let mut pool = build_through_third_publish("opus-x3-remount");
    let rolled = rollback_to(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    println!("回退那次挂载隔离 {:?}", rolled.output.isolated_slots_per_device);
    drop(rolled);
    let abandoned: BTreeSet<(u32, u64)> = slots_under_root(&pool, 2, 8)
        .difference(&slots_under_root(&pool, 1, 3))
        .copied()
        .collect();
    println!("只被被抛弃的 C 引用而 A 不引用的槽 {} 个", abandoned.len());

    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("重开");
    pool.devices = Some(devices);
    println!(
        "重开之后：实例 {}，隔离 {:?}",
        remounted.output.instance.0, remounted.output.isolated_slots_per_device
    );
    let free_abandoned: Vec<(u32, u64)> = abandoned
        .iter()
        .copied()
        .filter(|(device, slot)| {
            remounted
                .allocator
                .devices
                .iter()
                .find(|map| map.device == DeviceIdentity(*device))
                .expect("那块盘")
                .is_free(SlotNumber(*slot))
        })
        .collect();
    println!(
        "重开之后这些槽里当场可分配的 {} 个：{:?}",
        free_abandoned.len(),
        free_abandoned
    );
    pool.allocator = remounted.allocator;
    pool.output = remounted.current;
    let mut landed: Vec<(u32, u64)> = Vec::new();
    for seed in [41usize, 43, 47] {
        let publish = overwrite_in_process(&mut pool, &content_of(2600 + seed, seed), InstanceGeneration(4));
        for record in &publish.allocation_records {
            if record.generation == publish.root.checkpoint_txg {
                for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                    if abandoned.contains(&(record.device.0, slot)) {
                        landed.push((record.device.0, slot));
                    }
                }
            }
        }
    }
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let ring: Vec<(u32, u64)> = readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .map(|root| (root.instance.0, root.checkpoint_txg.0))
    .collect();
    println!("落在被抛弃槽上的新落点 {:?}", landed);
    println!("这时候根环里还读得出的根 {:?}", ring);
    assert!(
        ring.contains(&(2, 8)),
        "被抛弃的 C 一条都没离开根环：{ring:?}"
    );
    assert!(
        !landed.is_empty(),
        "重开之后的发布落在只被被抛弃根引用的槽上"
    );
}

/// X8：P1 让「所选根自己那条记录读不出」这件事零故障就发生——回退之后 jsn 4、5 被新实例盖掉，
/// 根 (1, 4) 与 (2, 5) 的锚点一起没了。对 (2, 5) 走一次 `replay_journal`：无锚点规则认 txg = 5 + 1 那条，
/// 于是被抛弃的实例 2 时间线 txg 6、7、8 整段重放回来，而前缀第五条（回退行的 W）一句话都说不上。
#[test]
fn x8_the_p1_overwrite_removes_the_anchor_with_zero_faults_and_the_chain_start_rule_replays_the_abandoned_timeline(
) {
    let mut pool = build_through_third_publish("opus-x8");
    rollback_to(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let records = scan_journal(&image, &superblock);
    println!(
        "回退之后的记录键 {:?}",
        records
            .keys()
            .map(|(instance, counter)| (instance.0, *counter))
            .collect::<Vec<_>>()
    );
    let roots = readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let mut anchorless = Vec::new();
    for root in &roots {
        let has_own_record = records.values().any(|record| {
            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
        });
        if !has_own_record {
            anchorless.push((root.instance.0, root.checkpoint_txg.0));
        }
    }
    println!("零故障就没有锚点的根 {anchorless:?}");
    let target = roots
        .iter()
        .find(|root| {
            root.instance == InstanceGeneration(2) && root.checkpoint_txg == CheckpointTxg(5)
        })
        .copied()
        .expect("(2, 5) 在环里");
    let (report, effective) = singlefs_core::recovery::replay_journal(
        &image,
        &target,
        superblock.geometry.journal_ring_bytes,
        &records,
        true,
        rollback_high_water_of_root(&image, &target),
    );
    println!(
        "从 (2, 5) 重放：第五条的 W = {:?}，施加 {} 条，施加之后的根 (实例 {}, txg {})",
        rollback_high_water_of_root(&image, &target),
        report.prefix_applied,
        effective.instance.0,
        effective.checkpoint_txg.0
    );
    assert!(anchorless.contains(&(1, 4)) && anchorless.contains(&(2, 5)));
    assert_eq!(
        (report.prefix_applied, effective.checkpoint_txg.0),
        (3, 8),
        "被抛弃的实例 2 时间线整段重放回来"
    );
}
