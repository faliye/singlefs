//! 里程碑「第二个事务」步 5 的验收，接在步 4 的固定脚本后面：A、B、重开取号 2、写行、暖机两次、C，挂着的时候回退到 A——一次向前发布 D（txg 9，
//! 实例仍是 2，D23（journal 的角色与格式） 已定项 14）；回退之后再覆盖写五次（txg 10–14；第一次把 D 引用的 A 的文件单元与 D 那一版的固定点释放、
//! 释放代 10），抬回退下界 F 到 11（上限 = min(每块盘上最新的持久有效根, 第 4 个不同状态) = min(13, 11)；两次空发布 txg 15、16 让两块盘
//! 各有一条带 F = 11 的根），释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回 50176（mkfs 实例表那 2 槽，
//! 实例 2 写行那次换下、释放代 5，回收了；引用它的 A、B 在 F 之下，不在候选集）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），A 的根还在候选集里（F = 0）、它引用的 mkfs 实例表被 E 盖掉，checker 在它上面判 I-2.1 红。
//!
//! 旧形态（挂载时回退：回退那次挂载取号 3、写回退行 txg 9、暖机 txg 10，覆盖写四次 txg 11–14）下这里的几个数不同，改前改后见实三报告的钉死值表。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{Placement, ReclaimedReuse};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, MountError, RaisedFloor,
    RollbackCandidateExclusion, RollbackError, RollbackTarget, RolledBack, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
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

/// 固定脚本到 D：A、B、重开取号 2、写行、暖机两次、C、挂着的时候回退到 (1, 3)——一次向前发布 D（txg 9，实例仍是 2）。
fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    roll_back(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    )
    .expect("挂着的时候回退到 A");
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(9));
    pool
}

/// 挂着的时候回退：做成时 `pool.output` 换成回退那次发布之后的一版。
fn roll_back(pool: &mut BuiltPool, target: RollbackTarget) -> Result<RolledBack, RollbackError> {
    let rollback_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
}

/// 回退之后再覆盖写五次（txg 10–14）：第一次释放 D 引用、它不引用的用户可见单元（A 的四个文件单元）与 D 那一版的固定点（释放代 10）。
fn five_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [13usize, 17, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(2))
        })
        .collect()
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    pool.output = current;
    raised
}

fn newest_root_floor(pool: &BuiltPool) -> CheckpointTxg {
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|root| (root.checkpoint_txg, root.instance))
    .expect("根")
    .rollback_floor
}

/// 验收第一、二条：上限 11；两次空发布带 F = 11 落到两块盘；释放代 ≤ 11 的 86 个落点（95 槽）回收、defer 队列从 143 槽减到 64；
/// A 的数据落点 50180（D 复活、txg 10 那次覆盖写换下，释放代 10）在回收的里面；E 的数据单元落 50176、它的分配记录改写成代 17、未释放；
/// 后释放的（代 12–14）仍占着；冷启动读回 E；根记录 F = 11；checker 全绿（A、B 的根在 F 之下、不在候选集）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    let mut pool = build_through_rollback("step-five-reuse");
    let overwrites = five_overwrites_after_the_rollback(&mut pool);
    assert_eq!(overwrites[0].root.checkpoint_txg, CheckpointTxg(10));
    assert_eq!(overwrites[4].root.checkpoint_txg, CheckpointTxg(14));
    for device in &pool.allocator.devices {
        // 向前回退不重建分配器：F = 0、环里最旧的有效根是 txg 0，一条已释放的记录都回收不了，每一次发布放掉的都还在 defer 里。
        assert_eq!(
            device.deferred_slots(),
            143,
            "按释放代：3 的 1 槽（A 换下 mkfs 树表）、4 的 8、5 的 10（写行那次，含 mkfs 实例表 2 槽）、6 与 7 各 8（两次暖机）、\
             8 的 14（C）、9 的 14（D 放掉 C 引用、它不引用的用户可见单元与 C 那一版的固定点）、10–14 各 16（五次覆盖写）"
        );
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    assert_eq!(
        raised.ceiling,
        CheckpointTxg(11),
        "min(每块盘最新的有效根 14 / 13, 第 4 个不同状态 11)"
    );
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (
                publish.root.checkpoint_txg.0,
                publish.root.rollback_floor.0,
                publish.record.transaction
            ))
            .collect::<Vec<_>>(),
        vec![(15, 11, 0), (16, 11, 0)],
        "两次带新 F 的空发布"
    );
    assert!(raised.reclaimed.contains(&Placement {
        slot: SlotNumber(50180),
        span: 2
    }));
    assert_eq!(
        raised.reclaimed.len(),
        86,
        "释放代 ≤ 11 的落点：代 3 的 1 个、4 的 8 个、5 的 9 个、6 与 7 各 8 个、8 与 9 各 12 个、10 与 11 各 14 个"
    );
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            64,
            "回收了 1 + 8 + 10 + 8 + 8 + 14 + 14 + 16 + 16 = 95 个槽，抬 F 的两次空发布又各放掉上一版的 8 个"
        );
        assert!(device.is_free(SlotNumber(50180)) && device.is_free(SlotNumber(50181)));
        for later in &overwrites[1..4] {
            let slot = later.data_pointers[0].locations[0].slot;
            assert!(
                !device.is_free(slot),
                "释放代 > F 的数据单元 {slot:?} 仍占着"
            );
        }
    }
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(2));
    assert_eq!(reuse.root.checkpoint_txg, CheckpointTxg(17));
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot,
        SlotNumber(50176),
        "E 的数据单元落回最低的可再分配偶数槽对 50176–50177：mkfs 实例表那 2 槽（实例 2 写行那次换下、释放代 5）回收了，\
         引用它的 A、B 在 F 之下、不在候选集，影子账不隔离"
    );
    let reused_record = pool
        .allocator
        .record_for(DeviceIdentity(0), SlotNumber(50176))
        .expect("50176 的记录");
    assert_eq!(
        (
            reused_record.generation,
            reused_record.is_released,
            reused_record.span_slots
        ),
        (CheckpointTxg(17), false, 2),
        "复用时那条记录改写"
    );
    assert_eq!(
        pool.allocator
            .records()
            .iter()
            .filter(|record| record.device == DeviceIdentity(0) && record.slot == SlotNumber(50176))
            .count(),
        1,
        "同盘同槽只有一条记录"
    );
    for device in &pool.allocator.devices {
        assert_eq!(device.deferred_slots(), 80, "E 又释放了第五版的 16 个槽");
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(17)),
            content: content_of(2000, 31)
        },
        "{:?}",
        report.journal
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        if *invariant == "I-8.8" {
            // 一事务一条、每条都带提交标记：I-8.8（前缀里的事务不被切开） 的 ③ ④ 没有对象，报不适用
            // （判别力在 `checker_known_bad_images.rs`）。
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{invariant} 在 E 之后的镜像上报不适用：{verdict:?}"
            );
            continue;
        }
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在 E 之后的镜像上要成立"
        );
    }
    for must_hold in ["I-2.1", "I-3.1", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 生效（D16（发布语义） 已定项 1「生效」，2026-09-26 取 SysPre）：F_生效 = max(根上带的 F, 系统配置里读得出的 F)——把 txg 16 的根槽
/// （盘 1 上唯一带 F = 11 的根）改坏，重开之后 F_生效 仍是 11（盘 0 的 txg 15 与两块盘系统配置里先写进去的都带 11）：所选根是 txg 15、
/// 链上 txg 16 的记录照样施加（同实例），回收与根槽都好时一样多，每块盘 defer 里剩 92 个槽。按改之前「各盘所带 F 最大值的最小值」只读根，
/// 盘 1 上最大的是 0，F_生效 回到 0、一个都不回收（向前回退之后这一臂剩多少没重量；旧形态下是 125）。
#[test]
fn floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it(
) {
    for (damage_second_carrier, expected_deferred, expected_chosen_txg) in
        [(false, 92, 16), (true, 92, 15)]
    {
        let mut pool = build_through_rollback("step-five-effective");
        five_overwrites_after_the_rollback(&mut pool);
        let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
        let second_carrier = raised.publishes[1].root.checkpoint_txg;
        assert_eq!(second_carrier, CheckpointTxg(16));
        let mut devices = pool.reopen_recorded();
        if damage_second_carrier {
            let target = target_for_publish(
                second_carrier,
                parameters().geometry.root_ring_slots_per_region,
            );
            let device =
                parameters().region_devices[usize::try_from(target.region).expect("区域号")];
            let offset = slot_offset(target, 4096);
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
        let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
        pool.devices = Some(devices);
        assert_eq!(
            mounted.output.chosen_root.checkpoint_txg,
            CheckpointTxg(expected_chosen_txg)
        );
        for device in &mounted.allocator.devices {
            assert_eq!(
                device.deferred_slots(),
                expected_deferred,
                "改坏第二块盘的载体 = {damage_second_carrier}：F_生效 = max(根上带的, 系统配置里的) = 11；数里是释放代 12–14 各 16、\
                 抬 F 两次空发布各 8、写行放掉的 12 与暖机两次各 8"
            );
        }
    }
}

/// 上限：第 4 个不同状态是 11（按新到旧 14、13、12、11；D 与 A 是同一个状态）⇒ 抬到 12 被拒；
/// 抬到 11 之后上限仍是 11，再抬 12 仍被拒。
#[test]
fn raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused() {
    let mut pool = build_through_rollback("step-five-ceiling");
    five_overwrites_after_the_rollback(&mut pool);
    for attempt in [0, 1] {
        let refused = raise_floor(&mut pool, CheckpointTxg(12));
        assert!(
            matches!(
                refused,
                Err(MountError::RollbackFloorAboveCeiling {
                    requested: CheckpointTxg(12),
                    ceiling: CheckpointTxg(11)
                })
            ),
            "第 {attempt} 次：上限 11"
        );
        if attempt == 0 {
            raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到上限");
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
}

/// 「非空」从盘上认（D16（发布语义） 已定项 1，2026-09-17 用户定案）：比树表里 inode 树与 extent 树的根指针，不看 journal 记录——
/// 把 txg 14 那次覆盖写的记录在两块盘上都改坏（根槽与单元不动），不同的状态仍是 14、13、12、11，上限仍是 11，抬到 12 被拒；
/// 按「环里有它自己那条记录且事务号非 0」认的话 txg 14 成了空根，第 4 个不同状态掉到 10，上限掉到 10。
#[test]
fn torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling() {
    let mut pool = build_through_rollback("step-five-ceiling-torn-record");
    let overwrites = five_overwrites_after_the_rollback(&mut pool);
    let fourth = &overwrites[4];
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(14), 14)
    );
    {
        let offset = record_offset(
            fourth.record.counter,
            parameters().geometry.journal_ring_bytes,
        );
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for (_, recorded) in devices.iter_mut() {
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读记录");
            bytes[300] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏记录");
        }
    }
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    assert!(
        !scan_journal(&image, &system_configuration).contains_key(&(InstanceGeneration(2), 14)),
        "txg 14 那条记录两份都读不出"
    );
    let refused = raise_floor(&mut pool, CheckpointTxg(12));
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorAboveCeiling {
                requested: CheckpointTxg(12),
                ceiling: CheckpointTxg(11)
            })
        ),
        "txg 14 的记录坏了，它的树表照样与 13 的不同：上限仍是 11：{:?}",
        refused.as_ref().err()
    );
}

/// 「4 个不同状态」按 (inode 树根指针, extent 树根指针) 去重（D16（发布语义） 已定项 1 的根环容量边界）：回退那次发布照抄目标那一版的
/// 两棵树，新根与目标是同一个状态。D（txg 9）是 A 的状态；覆盖写甲（txg 10）、再挂着的时候回退到 D（txg 11，又是 A 的状态）、覆盖写乙（txg 12）。
/// 按新到旧去重：12、11、10、（9 与 11 同一个状态，不再算）、8 ⇒ 第 4 个不同状态是 C 的 8；每块盘上最新的有效根是 12 与 11，
/// 上限 min(11, 8) = 8，抬到 9 被拒。不去重（每条带文件的根各算一个）的话第 4 新的是 9，上限 9、抬到 9 做成。
/// checker 那一侧同一条规则：把入口骗过（告诉入口今天的 F 是 9，入口按 txg ≥ 9 的根算，不同状态只剩 12、11、10 三个，上限落到最旧的有效根 9）
/// 抬到 9，镜像上 I-7.9（回退下界 F 不高于抬 F 的上限） 按抬之前的 F（0）去重算出上限 8，只红它；checker 不去重的话上限 9、判成立。
#[test]
fn the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker(
) {
    let build = |tag: &str| {
        let mut pool = build_through_rollback(tag);
        overwrite_in_process(&mut pool, &content_of(3017, 17), InstanceGeneration(2));
        roll_back(
            &mut pool,
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(9),
            },
        )
        .expect("挂着的时候回退到 D");
        overwrite_in_process(&mut pool, &content_of(3019, 19), InstanceGeneration(2));
        assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(12));
        pool
    };
    let mut pool = build("step-five-ceiling-repeated-state");
    let refused = raise_floor(&mut pool, CheckpointTxg(9));
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorAboveCeiling {
                requested: CheckpointTxg(9),
                ceiling: CheckpointTxg(8)
            })
        ),
        "A 的状态只算一次、第 4 个不同状态是 C：{:?}",
        refused.as_ref().err()
    );
    raise_floor(&mut pool, CheckpointTxg(8)).expect("抬到上限 8");
    let at_the_ceiling = pool.memory_pool();
    assert_eq!(
        check_pool_image(&at_the_ceiling)
            .into_iter()
            .find(|(invariant, _)| *invariant == "I-7.9")
            .expect("清单里有 I-7.9")
            .1,
        InvariantVerdict::Holds,
        "抬到上限 8：I-7.9 真被评估过且成立"
    );

    let mut above = build("step-five-ceiling-repeated-state-above");
    let mut current = above.output.clone();
    current.root.rollback_floor = CheckpointTxg(9);
    let devices = above.devices.as_mut().expect("镜像还开着");
    raise_rollback_floor(
        &parameters(),
        devices,
        &mut above.allocator,
        &mut current,
        CheckpointTxg(9),
        ShadowLedger::On,
    )
    .expect("入口被骗过：按 txg ≥ 9 的根算上限 9");
    let verdicts = check_pool_image(&above.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated,
        ["I-7.9"],
        "F 抬到 9、去重之后的上限 8：只红 I-7.9：{verdicts:?}"
    );
}

/// 分配器里抬 F 会动的东西：分配记录、开放段、逐盘的占着 / 空闲 / defer / 隔离 / runs / 全空段计数。
fn allocator_state(pool: &BuiltPool) -> AllocatorState {
    AllocatorState {
        records: pool.allocator.records().to_vec(),
        open_segment: pool.allocator.open_segment(),
        per_device: pool
            .allocator
            .devices
            .iter()
            .map(|device| {
                [
                    device.allocated_slots(),
                    device.free_slots(),
                    device.deferred_slots(),
                    device.isolated_slots(),
                    device.free_runs(),
                    device.empty_segments(),
                ]
            })
            .collect(),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AllocatorState {
    records: Vec<singlefs_core::allocator::AllocationRecord>,
    open_segment: Option<SlotNumber>,
    per_device: Vec<[u64; 6]>,
}

/// 算上限时一条有效根的树表读不出：拒绝抬 F，不按空或非空猜（用户：读不出就要走修复、不能跳过）——回退之后覆盖写五次，
/// 把有效根 txg 12 的树表两盘都改坏，抬 F 到 11 ⇒ 返回 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（点名 (2, 12)）；
/// 分配器、现行那一版、录制流都不变（没回收、没发布）。按「读不出算空」猜的话 txg 12、13 都与前一条不同，上限照样是 11、抬 F 成功。
#[test]
fn raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable() {
    let mut pool = build_through_rollback("step-five-ceiling-unreadable-valid-root");
    let overwrites = five_overwrites_after_the_rollback(&mut pool);
    let damaged = &overwrites[2];
    assert_eq!(damaged.root.checkpoint_txg, CheckpointTxg(12));
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for location in &damaged.root.tree_table.locations {
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("txg 12 的树表所在的盘");
            let offset = location.slot.to_device_offset();
            let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
            recorded
                .read_at(offset, &mut bytes)
                .expect("读 txg 12 的树表");
            bytes[200] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏 txg 12 的树表");
        }
    }
    let allocator_before = allocator_state(&pool);
    let current_before = pool.output.clone();
    let recorded_before = pool.stream.operations().len();
    let refused = raise_floor(&mut pool, CheckpointTxg(11));
    assert!(
        matches!(
            refused,
            Err(
                MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                    root: RollbackTarget {
                        instance: InstanceGeneration(2),
                        checkpoint_txg: CheckpointTxg(12)
                    },
                    ..
                }
            )
        ),
        "有效根 txg 12 的树表读不出：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        allocator_state(&pool),
        allocator_before,
        "没有回收、没有隔离"
    );
    assert_eq!(pool.output, current_before, "现行那一版没动");
    assert_eq!(
        pool.stream.operations().len(),
        recorded_before,
        "一个写、一道屏障都没发"
    );
}

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 mkfs 实例表那 2 槽 50176——
/// F = 0 时 A（txg 3）还是候选、它的根指着那片实例表；checker 走 A 时那片实例表的校验和对不上 ⇒ I-2.1 红。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("step-five-window-zero");
    five_overwrites_after_the_rollback(&mut pool);
    let reclaimed = pool
        .allocator
        .reclaim_released_up_to(CheckpointTxg(11), ReclaimedReuse::Immediately);
    assert_eq!(
        reclaimed.len(),
        86,
        "释放代 ≤ 11 的落点：代 3 的 1 个、4 的 8 个、5 的 9 个、6 与 7 各 8 个、8 与 9 各 12 个、10 与 11 各 14 个"
    );
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(2));
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot,
        SlotNumber(50176),
        "mkfs 实例表那 2 槽被拿走：A（F = 0 时仍是候选）的根还指着它"
    );
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(0), "F 没抬");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        violated.contains(&"I-2.1"),
        "A 的根还是候选，它的数据单元被盖了：{verdicts:?}"
    );
    // C22（刚释放的块立即重分配） 的「怎么拦」第 ① 条点的是 I-4.8（近 K 代根校验和自洽）：从最近 K 代任一根遍历，校验和必须全对。
    // 只断言 I-2.1 红，拦的是「有一个单元的校验和与父指针对不上」，没说那是从一条**候选根**出发的遍历走进去的
    // （收口表第 22 行 2026-09-17 逐句核出来的那一句）。
    assert!(
        violated.contains(&"I-4.8"),
        "复用窗口置 0 之后 I-4.8 必须变红：不红说明这条约束根本没被检查（C22（刚释放的块立即重分配） 第 ② 条）：{verdicts:?}"
    );
}

/// 回退候选集的 F 用 F_生效（D16（发布语义） 已定项 1「生效」，SysPre：max(根上带的, 系统配置里读得出的)）：抬到 11 之后把盘 1 的载体
/// （txg 16）改坏，F_生效 仍是 11（盘 0 的 txg 15 与两块盘系统配置里都带 11），txg 9 的根 D 在 F 之下、不是候选，挂着的时候回退在任何写之前
/// 拒成 `BelowEffectiveFloor`、盘上逐字节不变、分配器与现行版本不动。改之前按「各盘所带 F 最大值的最小值」只读根，F_生效 回到 0、D 退得到。
#[test]
fn floor_carried_by_only_one_device_root_and_the_system_configuration_keeps_the_roots_below_it_out_of_the_rollback_candidates(
) {
    let mut pool = build_through_rollback("step-five-candidate-floor");
    five_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let target = target_for_publish(
            second_carrier,
            parameters().geometry.root_ring_slots_per_region,
        );
        let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
        let offset = slot_offset(target, 4096);
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
    let before = common::disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let refused = roll_back(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(9),
        },
    );
    assert!(
        matches!(
            &refused,
            Err(RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
                ..
            })
        ),
        "F_生效 仍是 11，txg 9 的根不在候选集里：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        common::disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前"
    );
    assert_eq!(pool.output, current_before, "现行版本不动");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器不动"
    );
}

/// 抬 F 生效之前回收的槽不许发出去（D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；alloc-basis 第二轮云端攻方腿打中：
/// 抬 F 自己的空发布在开放段满了之后按最低全空段开新段，刚回收空的那一段正好中选，崩在两条带新 F 的根之间时 F 之下的根仍是候选、
/// 它们的单元已被盖）。历史照那条腿的：A、B、重开取号 2、六次覆盖写（txg 8–13，开放段用满）、抬 F 到 8——两次空发布的落点一个都不在
/// 这次回收的槽上。
#[test]
fn slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect() {
    let mut pool = build_pool("step-five-hold-until-effective");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for seed in [31usize, 37, 41, 43, 47, 53] {
        overwrite_in_process(
            &mut pool,
            &content_of(2000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    let raised = raise_floor(&mut pool, CheckpointTxg(8)).expect("抬到 8");
    assert!(
        raised.publishes.len() >= 2,
        "至少两次空发布才能让两块盘各有一条带新 F 的根：{}",
        raised.publishes.len()
    );
    assert!(!raised.reclaimed.is_empty(), "回收了 A 与前几版释放的落点");
    let reclaimed_slots: std::collections::BTreeSet<u64> = raised
        .reclaimed
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    for publish in &raised.publishes {
        for placement in publish.placements() {
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                assert!(
                    !reclaimed_slots.contains(&slot),
                    "txg {} 的固定点落在这次回收的槽 {slot} 上：F 还没在两块盘上生效",
                    publish.root.checkpoint_txg.0
                );
            }
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(8));
}

/// C502（抬 F 时现行版本里没有实例表单元）：mkfs 同一个进程里第一个事务之后再覆盖写三次（txg 4、5、6），抬 F 做成。
/// 这个进程的现行版本是 `publish_first_file` 那一版往下接的，`TransactionOutput::units` 里一个实例表单元都没有；
/// 候选集按现行那一版的根指针上读出来的实例表判（mkfs 那片 0 行的表），不看那个内存数组。
/// 上限 = min(每块盘上最新的有效根 min(6, 4), 第 4 新的非空有效根 3) = 3（D16（发布语义） 已定项 1）；两次空发布 txg 7（盘 1）、8（盘 0）
/// 让两块盘各有一条带 F = 3 的根；释放代 ≤ 3 的只有第一个文件版本换下的 mkfs 那片第 0 版树表（1 槽）。
#[test]
fn raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root(
) {
    let mut pool = build_pool("step-five-raise-in-the-make-filesystem-process");
    for seed in [3usize, 5, 7] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(1),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(6));
    assert!(
        pool.output
            .units
            .iter()
            .all(|unit| unit.identity != singlefs_core::transaction::TransactionUnit::InstanceTable),
        "这个进程内存里的现行版本不带实例表单元：抬 F 要的那张表只在根指针后面"
    );
    let raised = raise_floor(&mut pool, CheckpointTxg(3)).expect("抬 F 到 3");
    assert_eq!(raised.ceiling, CheckpointTxg(3), "上限 min(4, 3)");
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(7), CheckpointTxg(8)],
        "两次空发布让两块盘各有一条带新 F 的根"
    );
    assert_eq!(
        raised.reclaimed,
        vec![Placement {
            slot: singlefs_core::make_filesystem::TREE_TABLE_GENESIS_SLOT,
            span: 1
        }],
        "释放代 ≤ 3 的只有 mkfs 那片第 0 版树表"
    );
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(3));
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert!(
        violated.is_empty(),
        "抬 F 之后 checker 一条都不红：{verdicts:?}"
    );
}

/// C502 那一格改成从根指针读表之后，读不出的那一格：现行那一版的根指着的实例表两份都读不出，抬 F 在任何写之前拒绝
/// （`InstanceTableMalformed`：判不了候选集），盘上逐字节不变（系统配置槽、根环、录制流步数）。
#[test]
fn raising_the_floor_when_the_current_roots_instance_table_is_unreadable_is_refused_before_any_write(
) {
    let mut pool = build_pool("step-five-raise-with-an-unreadable-instance-table");
    overwrite_in_process(&mut pool, &content_of(3100, 13), InstanceGeneration(1));
    {
        let instance_table_locations = pool.output.root.instance_table.locations;
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for location in &instance_table_locations {
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("实例表所在的盘");
            let offset = location.slot.to_device_offset();
            let mut bytes = vec![0u8; 512];
            recorded.read_at(offset, &mut bytes).expect("读实例表");
            bytes[200] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏实例表");
        }
    }
    let before = common::disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = raise_floor(&mut pool, CheckpointTxg(3));
    assert!(
        matches!(refused, Err(MountError::InstanceTableMalformed)),
        "现行那一版的实例表两份都读不出：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        common::disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒绝在任何写之前"
    );
}

/// 抬 F 重算影子账时读不出账的被抛弃根也要计数（代码三方第三轮云端攻方腿打中：那一处此前把计数丢掉）。被抛弃的根由崩溃恢复造出
/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出，恢复落到 (2, 7)，
/// 实例 3 写行与暖机 txg 9、10，再把 C 写回）；实例 3 覆盖写四次（txg 11–14），把 C 的树表两盘都改坏，抬 F 到 11 报 1 条读不出的被抛弃根。
#[test]
fn raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable() {
    let mut pool = build_pool("step-five-raise-counts-unreadable");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third = overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    assert_eq!(
        (
            abandoning.output.effective_root.checkpoint_txg,
            abandoning.output.instance,
            pool.output.root.checkpoint_txg
        ),
        (CheckpointTxg(7), InstanceGeneration(3), CheckpointTxg(10)),
        "恢复落到 (2, 7)，实例 3 写行与暖机到 txg 10"
    );
    for seed in [17usize, 19, 23, 29] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(3),
        );
    }
    {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        for location in &third.root.tree_table.locations {
            let (_, recorded) = open_devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("C 的树表所在的盘");
            let offset = location.slot.to_device_offset();
            let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
            recorded.read_at(offset, &mut bytes).expect("读 C 的树表");
            bytes[200] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏 C 的树表");
        }
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到 11");
    assert_eq!(raised.abandoned_roots_unreadable, 1, "C 的账读不出");
}

/// 建池（第一个文件 A，txg 3）→ 同一个进程里覆盖写 B（txg 4）、C（txg 5）→ 崩溃恢复抛弃 C（C 的根槽与数据单元暂时读不出，
/// 择根落到 B、C 那条记录验不过；实例 2 写行 txg 6、暖机 txg 7；再写回，C 按实例 2 的表判被抛弃）→ 实例 2 覆盖写四次（txg 8–11）。
/// 交回之前的镜像 checker 全绿；抬 F 的上限 = min(每块盘上最新的有效根, 第 4 新的不同状态非空有效根 8) = 8。
fn pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let third = overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(1));
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    assert_eq!(
        (
            third.root.checkpoint_txg,
            abandoning.output.effective_root.checkpoint_txg,
            abandoning.output.instance,
            abandoning.output.row_publish.root().checkpoint_txg,
            pool.output.root.checkpoint_txg
        ),
        (
            CheckpointTxg(5),
            CheckpointTxg(4),
            InstanceGeneration(2),
            CheckpointTxg(6),
            CheckpointTxg(7)
        ),
        "C 是 txg 5；恢复落到 B（txg 4），实例 2 写行 txg 6、暖机 txg 7"
    );
    for seed in [17usize, 19, 23, 29] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(11));
    let verdicts = check_pool_image(&pool.memory_pool());
    assert!(
        verdicts
            .iter()
            .all(|(_, verdict)| !matches!(verdict, InvariantVerdict::Violated(_))),
        "抬 F 之前一条违例都没有：{verdicts:?}"
    );
    pool
}

/// 增补 2 收口表第 43 行那一形（`history::KNOWN_RED_FORMS` 第 1 条）在管理员回退改成挂着时的向前发布之后还走不走得到
/// （实三交回 Q4：随机历史与崩溃注入里都没再复现；「没复现」不等于「不可达」）：**走得到**，被抛弃的时间线由崩溃恢复造。
/// F 抬到 5——C 那个 txg，环里 txg 5 上只有 C 一条根、它属于被抛弃的实例 1 那一段（抬之前的镜像上
/// `raised_floor_lands_only_on_abandoned_roots` 为真）——之后 checker 只判 I-3.1 红、记账的已分配多于遍历：B（txg 4）在 F 之下、
/// 出了候选集，它独占、被实例 2 写行那次换下的单元释放代 6，F = 5 回收不了，还算在已分配里。拿这一次的观察按「已知红」清单归类，
/// 归到第 1 条（收口表第 43 行）。对照：同一段历史 F 抬到 6（写行那次的根）或 7（暖机），回收到释放代 6，checker 全绿。
/// 修法没定（收口表第 43 行「要三方」），这里只钉今天的结局。
#[test]
fn raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43(
) {
    let mut pool =
        pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites("step-five-gap-5");
    let image_before_raising = pool.memory_pool();
    assert_eq!(
        singlefs_harness::history::raised_floor_lands_only_on_abandoned_roots(
            &image_before_raising,
            CheckpointTxg(5)
        ),
        Some(true),
        "抬之前的镜像上 txg 5 那一条根（C）属于被抛弃的实例"
    );
    let raised = raise_floor(&mut pool, CheckpointTxg(5)).expect("F 抬到 5：不超过上限 8");
    assert_eq!(raised.ceiling, CheckpointTxg(8));
    let verdicts = check_pool_image(&pool.memory_pool());
    let violations: Vec<(&'static str, String)> = verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert_eq!(
        violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        vec!["I-3.1"],
        "只有 I-3.1 红：{violations:?}"
    );
    let (newest_ring_root_txg, root_ring_slot_count) =
        singlefs_harness::history::newest_ring_root_and_slot_count(&pool.memory_pool());
    let observation = singlefs_harness::history::FailureObservation {
        position: singlefs_harness::history::StepPosition::Operation(0),
        operation_kind: Some(singlefs_harness::history::HistoryOperationKind::RaiseRollbackFloor),
        violations,
        panic: None,
        newest_ring_root_txg,
        root_ring_slot_count,
        harness_judgement: None,
        model_disagreement: None,
        raised_floor_lands_only_on_abandoned_roots:
            singlefs_harness::history::raised_floor_lands_only_on_abandoned_roots(
                &image_before_raising,
                CheckpointTxg(5),
            ),
        record_check: singlefs_harness::memory_pool::RecordCheck::default(),
    };
    let ending = singlefs_harness::history::classify_failure(observation);
    assert!(
        matches!(
            ending,
            singlefs_harness::history::HistoryEnding::KnownRed { form: 0, .. }
        ),
        "归到「已知红」清单第 1 条（{}）：{ending:?}",
        singlefs_harness::history::KNOWN_RED_FORMS[0].closeout_table_row
    );

    for floor in [6u64, 7] {
        let mut control = pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites(
            &format!("step-five-gap-control-{floor}"),
        );
        raise_floor(&mut control, CheckpointTxg(floor)).expect("对照：F 抬到写行或暖机那一条根");
        let control_verdicts = check_pool_image(&control.memory_pool());
        assert!(
            control_verdicts
                .iter()
                .all(|(_, verdict)| !matches!(verdict, InvariantVerdict::Violated(_))),
            "对照：F 抬到 {floor}（不在空档里）一条违例都没有：{control_verdicts:?}"
        );
    }
}

/// 事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7），**不承载事务的空发布写 0 也不许把计数拉回去**。
///
/// 抬 F 推的两次空发布在记录上写事务号 0；发布 E 若取「上一条记录的事务号 + 1」就拿到 0 + 1 = 1，
/// 与实例 2 的 C 用过的 1 重号。回退那次发布 D 的记录也写 0（它不施加也不删除任何记录，实现取 0，条款没写，见实三报告）。重号之后同一实例的两个版本写序逐字节相同（写序存事务号低 48 位），
/// I-1.8（归并后版本全序） 判不开它们，而实例表行的 W 能当精确前缀也正是靠「记录按事务号顺序追加」这条纪律。
#[test]
fn the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor(
) {
    let mut pool = build_through_rollback("txn-number-per-instance");
    let five = five_overwrites_after_the_rollback(&mut pool);
    let before_raising: Vec<u64> = five
        .iter()
        .map(|output| output.record.transaction)
        .collect();
    assert_eq!(
        before_raising,
        vec![2, 3, 4, 5, 6],
        "实例 2 的五次覆盖写接在 C 的 1 之后按实例计数（D 写 0，不推进）"
    );

    raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到 11");
    assert_eq!(
        pool.output.record.transaction, 0,
        "抬 F 推的空发布不承载事务，记录上写 0（D23 已定项 19 ①）"
    );

    let after_raising =
        overwrite_in_process(&mut pool, &content_of(3100, 31), InstanceGeneration(2));
    assert_eq!(
        after_raising.record.transaction, 7,
        "空发布不推进计数，也不许把它拉回去：E 接在 6 之后是 7，不是 0 + 1"
    );

    let mut used: Vec<u64> = before_raising;
    used.push(after_raising.record.transaction);
    let mut deduplicated = used.clone();
    deduplicated.sort_unstable();
    deduplicated.dedup();
    assert_eq!(
        deduplicated.len(),
        used.len(),
        "同一个实例里非 0 的事务号互不重复：{used:?}"
    );

    // 盘上的样子，兼 I-8.7（实例内事务号不重号） 那条射程「事务号 0 不进序列」的阳性对照：抬 F 推出来的两条空发布记录
    // （事务号 0）**夹在**实例 2 的 6 与 7 中间，而池级 checker 在这份镜像上判 I-8.7 绿且真被评估过。
    // 把 0 也算进序列的写法在这里就红了（4 → 0 不是严格递增），它红不了才说明 0 真被排除掉。
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let transactions_of_the_second_instance: Vec<u64> = scan_journal(&image, &system_configuration)
        .into_iter()
        .filter(|((instance, _), _)| *instance == InstanceGeneration(2))
        .map(|(_, record)| record.transaction)
        .collect();
    assert_eq!(
        transactions_of_the_second_instance,
        vec![0, 0, 0, 1, 0, 2, 3, 4, 5, 6, 0, 0, 7],
        "实例 2 按 jsn 排下来的事务号：写行与两次暖机空发布 0，C 是 1，回退那次发布 D 写 0，五次覆盖写 2–6，抬 F 两次空发布 0，E 是 7"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        verdicts
            .iter()
            .find(|(invariant, _)| *invariant == "I-8.7")
            .expect("清单里有 I-8.7")
            .1,
        InvariantVerdict::Holds,
        "I-8.7 要真被评估过且成立：中间夹着的空发布记录不进那个序列：{verdicts:?}"
    );
}
