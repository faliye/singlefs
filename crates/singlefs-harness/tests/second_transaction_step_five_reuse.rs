//! 里程碑「第二个事务」步 5 的验收：回退之后再覆盖写四次（txg 11–14；第一次把 A 的四个文件单元与暖机那一版的固定点释放、释放代 11），抬回退下界 F 到 11
//! （上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根) = min(13, 11)；两次空发布 txg 15、16 让两块盘各有一条带 F = 11 的根），
//! 释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回 50178（mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、影子账隔离着；A 的数据单元 50180 排在后面）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），第 0 代根还在候选集里（F = 0）、它们引用的 mkfs 树表单元被 E 盖掉，checker 在它们上判 I-2.1 红。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{Placement, ReclaimedReuse};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RaisedFloor, RollbackTarget,
    ShadowLedger,
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

/// 固定脚本到 D：A、B、重开取号 2、写行、暖机两次、C、重开回退到 (1, 3)、D、暖机一次。
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
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back
        .current
        .into_file_version()
        .expect("回退到 A，现行那一版带文件");
    pool
}

/// 回退之后再覆盖写四次（txg 11–14）：第一次释放 A 的四个文件单元与暖机那一版的固定点（释放代 11）。
fn four_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [17usize, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(3))
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

/// 验收第一、二条：上限 11；两次空发布带 F = 11 落到两块盘；A 的八个落点（10 槽）回收、defer 队列从 40 槽减到 30；E 的数据单元落 50180、
/// 它的分配记录改写成代 17、未释放；后释放的（代 12–14）仍占着；冷启动读回 E；根记录 F = 11；checker 全绿（A 的根在 F 之下、不在候选集）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    let mut pool = build_through_rollback("step-five-reuse");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    assert_eq!(overwrites[0].root.checkpoint_txg, CheckpointTxg(11));
    assert_eq!(overwrites[3].root.checkpoint_txg, CheckpointTxg(14));
    for device in &pool.allocator.devices {
        // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层；D 那一版的账里有被抛弃的 B、C 之后才用到的槽
        // （叶 62），D 重写两块盘各自的叶 61、叶 62、第 1 层节点 0 与根，暖机只重写叶 62 那一支，四次覆盖写都重写叶 61 与叶 62 两支。
        assert_eq!(
            device.deferred_slots(),
            83,
            "A 的账里 mkfs 树表 1 槽已释放；D 释放 A 的四个固定点单元（分配记录树五个节点加三个角色，8 槽）与 mkfs 实例表（共 10 槽）、\
             暖机释放 D 的叶 62 那一支与三个角色（8 槽）、四次覆盖写各释放上一版的 16 个槽（文件四个单元 6 槽、分配记录树七个节点、三个角色）"
        );
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    assert_eq!(
        raised.ceiling,
        CheckpointTxg(11),
        "min(每块盘最新的有效根 14 / 13, 第 4 新的非空 11)"
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
        32,
        "释放代 ≤ 11 的落点：A 放掉的 mkfs 树表（代 3）、D 放掉的 9 个（代 9）、暖机放掉的 8 个（代 10）、第一次覆盖写放掉的 14 个（代 11）"
    );
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            64,
            "回收了 1 + 10 + 8 + 16 = 35 个槽，抬 F 的两次空发布又各放掉上一版的 8 个"
        );
        assert!(device.is_free(SlotNumber(50180)) && device.is_free(SlotNumber(50181)));
        for later in &overwrites[..3] {
            let slot = later.data_pointers[0].locations[0].slot;
            assert!(
                !device.is_free(slot),
                "释放代 > F 的数据单元 {slot:?} 仍占着"
            );
        }
    }
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(reuse.root.checkpoint_txg, CheckpointTxg(17));
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot,
        SlotNumber(50178),
        "E 的数据单元落回最低的可再分配偶数槽对 50178–50179：mkfs 树表那 1 槽（A 换下、释放代 3）回收了、50179 从没分配过；mkfs 实例表那片 50176 虽被 D 放掉、也回收了，但 B 的根还引用它、被影子账隔离；A 的数据单元 50180 排在后面"
    );
    let reused_record = pool
        .allocator
        .record_for(DeviceIdentity(0), SlotNumber(50178))
        .expect("50178 的记录");
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
            .filter(|record| record.device == DeviceIdentity(0) && record.slot == SlotNumber(50178))
            .count(),
        1,
        "同盘同槽只有一条记录"
    );
    for device in &pool.allocator.devices {
        assert_eq!(device.deferred_slots(), 80, "E 又释放了第四版的 16 个槽");
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(17)),
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

/// 生效（D16 已定项 1）：每块幸存盘上都有带新 F 的持久根才生效，恢复后生效值 = 各盘所带 F 最大值的最小值——把 txg 16 的根槽（盘 1 上唯一带 F = 11 的根）改坏，
/// 重开之后 F_生效 回到 0：所选根是 txg 15、链上 txg 16 的记录照样施加（同实例），盘上写着已释放的 59 个槽一个都不回收（新实例写行再放 6、暖机两次各放 4 ⇒ 73）；根槽都好时回收 21 个槽（59 − 21 + 14 = 52）。
#[test]
fn one_device_carrying_the_floor_alone_does_not_take_effect_on_remount() {
    for (damage_second_carrier, expected_deferred, expected_chosen_txg) in
        [(false, 90, 16), (true, 125, 15)]
    {
        let mut pool = build_through_rollback("step-five-effective");
        four_overwrites_after_the_rollback(&mut pool);
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
                "改坏第二块盘的载体 = {damage_second_carrier}：F_生效 = 各盘 F 最大值的最小值；数里含写行放掉的 6 与暖机两次放掉的 8"
            );
        }
    }
}

/// 上限：第 4 新的非空持久有效根是 11（非空的有 14、13、12、11；D 与暖机是空发布不算；A 的根 3 是第 5 新）⇒ 抬到 12 被拒；
/// 抬到 11 之后上限仍是 11，再抬 12 仍被拒。
#[test]
fn raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused() {
    let mut pool = build_through_rollback("step-five-ceiling");
    four_overwrites_after_the_rollback(&mut pool);
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
/// 把 txg 14 那次覆盖写的记录在两块盘上都改坏（根槽与单元不动），非空的有效根仍是 14、13、12、11，上限仍是 11，抬到 12 被拒；
/// 按「环里有它自己那条记录且事务号非 0」认的话 txg 14 成了空根，第 4 新的非空根掉到 A 的 3，上限掉到 3。
#[test]
fn torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling() {
    let mut pool = build_through_rollback("step-five-ceiling-torn-record");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    let fourth = &overwrites[3];
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
        !scan_journal(&image, &system_configuration).contains_key(&(InstanceGeneration(3), 14)),
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

/// 「前一条」只在有效根里找：回退那次发布 D（txg 9）照抄 A 的文件，树表里两棵树的根指针与前一条有效根 A（txg 3）的相同 ⇒ 空；
/// 夹在 A 与 D 之间的被抛弃根 C（txg 8）带着第三次的内容，拿它比 D 就成了非空。回退之后覆盖写三次（txg 11、12、13）：
/// 非空的有效根是 13、12、11、3，第 4 新的是 3，每块盘上最新的有效根是 12（盘 0）与 13（盘 1），上限 min(12, 3) = 3，抬到 4 被拒。
#[test]
fn the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it(
) {
    let mut pool = build_through_rollback("step-five-ceiling-previous-valid-root");
    for seed in [17usize, 19, 23] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(3),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    let refused = raise_floor(&mut pool, CheckpointTxg(4));
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorAboveCeiling {
                requested: CheckpointTxg(4),
                ceiling: CheckpointTxg(3)
            })
        ),
        "D 不算非空、第 4 新的非空有效根是 A：{:?}",
        refused.as_ref().err()
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

/// 算上限时一条有效根的树表读不出：拒绝抬 F，不按空或非空猜（用户：读不出就要走修复、不能跳过）——回退之后覆盖写四次，
/// 把有效根 txg 12 的树表两盘都改坏，抬 F 到 11 ⇒ 返回 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（点名 (3, 12)）；
/// 分配器、现行那一版、录制流都不变（没回收、没发布）。按「读不出算空」猜的话 txg 12、13 都与前一条不同，上限照样是 11、抬 F 成功。
#[test]
fn raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable() {
    let mut pool = build_through_rollback("step-five-ceiling-unreadable-valid-root");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    let damaged = &overwrites[1];
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
                        instance: InstanceGeneration(3),
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

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 A 的数据落点 50176——
/// F = 0 时 A（txg 3）还是候选，影子账按窄读法豁免它引用的槽、没隔离 50176；checker 走 A 时那片数据的校验和对不上 ⇒ I-2.1 红。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("step-five-window-zero");
    four_overwrites_after_the_rollback(&mut pool);
    let reclaimed = pool
        .allocator
        .reclaim_released_up_to(CheckpointTxg(11), ReclaimedReuse::Immediately);
    assert_eq!(
        reclaimed.len(),
        32,
        "释放代 ≤ 11 的落点：mkfs 树表 1 个、D 放掉的 9 个、暖机放掉的 8 个、第一次覆盖写放掉的 14 个"
    );
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot,
        SlotNumber(50176),
        "A 的数据落点被拿走：A（F = 0 时仍是候选）还引用它，窄读法没隔离它"
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

/// 回退候选集的 F 用 F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F：抬到 11 之后把盘 1 的载体（txg 16）改坏，
/// F_生效 回到 0，txg 9 的根 D 仍是候选、退得到；按最新根（txg 15，F = 11）自己的 F 判会把它拒掉。
#[test]
fn roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates() {
    let mut pool = build_through_rollback("step-five-candidate-floor");
    four_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    let mut devices = pool.reopen_recorded();
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
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(9),
        },
        ShadowLedger::On,
    )
    .expect("F_生效 是 0，txg 9 的根仍在候选集里");
    pool.devices = Some(devices);
    assert_eq!(rolled_back.output.instance, InstanceGeneration(4));
    assert_eq!(
        rolled_back.output.row_publish.root().rollback_floor,
        CheckpointTxg(0),
        "新实例的根写 F_生效"
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

/// 抬 F 重算影子账时读不出账的被抛弃根也要计数（代码三方第三轮云端攻方腿打中：那一处此前把计数丢掉）：
/// 回退之后把 C 的树表两盘都改坏，抬 F 到 11 报 1 条读不出的被抛弃根。
#[test]
fn raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable() {
    let mut pool = build_through_rollback("step-five-raise-counts-unreadable");
    four_overwrites_after_the_rollback(&mut pool);
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let third = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| root.checkpoint_txg == CheckpointTxg(8))
    .expect("C 的根在环里");
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for location in &third.tree_table.locations {
            let (_, recorded) = devices
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

/// 事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7），**不承载事务的空发布写 0 也不许把计数拉回去**。
///
/// 抬 F 推的两次空发布在记录上写事务号 0；发布 E 若取「上一条记录的事务号 + 1」就拿到 0 + 1 = 1，
/// 与实例 3 第一次覆盖写用过的 1 重号。重号之后同一实例的两个版本写序逐字节相同（写序存事务号低 48 位），
/// I-1.8（归并后版本全序） 判不开它们，而实例表行的 W 能当精确前缀也正是靠「记录按事务号顺序追加」这条纪律。
#[test]
fn the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor(
) {
    let mut pool = build_through_rollback("txn-number-per-instance");
    let four = four_overwrites_after_the_rollback(&mut pool);
    let before_raising: Vec<u64> = four
        .iter()
        .map(|output| output.record.transaction)
        .collect();
    assert_eq!(
        before_raising,
        vec![1, 2, 3, 4],
        "实例 3 的四次覆盖写按实例计数、从 1 起"
    );

    raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到 11");
    assert_eq!(
        pool.output.record.transaction, 0,
        "抬 F 推的空发布不承载事务，记录上写 0（D23 已定项 19 ①）"
    );

    let after_raising =
        overwrite_in_process(&mut pool, &content_of(3100, 31), InstanceGeneration(3));
    assert_eq!(
        after_raising.record.transaction, 5,
        "空发布不推进计数，也不许把它拉回去：E 接在 4 之后是 5，不是 0 + 1"
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
    // （事务号 0）**夹在**实例 3 的 4 与 5 中间，而池级 checker 在这份镜像上判 I-8.7 绿且真被评估过。
    // 把 0 也算进序列的写法在这里就红了（4 → 0 不是严格递增），它红不了才说明 0 真被排除掉。
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let transactions_of_the_third_instance: Vec<u64> = scan_journal(&image, &system_configuration)
        .into_iter()
        .filter(|((instance, _), _)| *instance == InstanceGeneration(3))
        .map(|(_, record)| record.transaction)
        .collect();
    assert_eq!(
        transactions_of_the_third_instance,
        vec![0, 0, 1, 2, 3, 4, 0, 0, 5],
        "实例 3 按 jsn 排下来的事务号：回退那次建实例推的写行与暖机空发布 0，之后四次覆盖写 1–4，抬 F 两次空发布 0，E 是 5"
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
