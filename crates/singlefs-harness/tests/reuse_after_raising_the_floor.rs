//! 里程碑「覆盖写、释放、回退与复用」步 5 的验收，接在步 4 的固定脚本后面：A、B、重开取号 2、写行、暖机两次、C，挂着的时候回退到 A——一次向前发布 D（txg 9，
//! 实例仍是 2，D23（journal 的角色与格式） 已定项 14）；回退之后再覆盖写五次（txg 10–14；第一次把 D 引用的 A 的文件单元与 D 那一版的固定点释放、
//! 释放代 10），抬回退下界 F 到 11（上限 = min(每块盘上最新的持久有效根, 第 4 个不同状态) = min(13, 11)；两次空发布 txg 15、16 让两块盘
//! 各有一条带 F = 11 的根），释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回 50176（mkfs 实例表那 2 槽，
//! 实例 2 写行那次换下、释放代 5，回收了；引用它的 A、B 在 F 之下，不在候选集）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），A 的根还在候选集里（F = 0）、它引用的 mkfs 实例表被 E 盖掉，checker 在它上面判 I-2.1 红。
//!
//! 旧形态（挂载时回退：回退那次挂载取号 3、写回退行 txg 9、暖机 txg 10，覆盖写四次 txg 11–14）下这里的几个数不同，改前改后见实三报告的钉死值表。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/reuse_after_raising_the_floor.rs"]
mod scenarios_of_reuse_after_raising_the_floor;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    BuiltPool,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_reuse_after_raising_the_floor::{
    build_through_rollback, content_of, five_overwrites_after_the_rollback, newest_root_floor,
    overwrite_in_process, raise_floor, roll_back,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable, MountError, RollbackCandidateExclusion, RollbackError, RollbackTarget,
    StillUnreadableAfterOneReread,
};
use singlefs_core::recovery::{
    choose_system_configuration, scan_journal, BadRootRingSlotReading, PoolReader,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reuse_after_raising_the_floor::raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reuse_after_raising_the_floor::raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it,
    );
}

/// 生效（D16（发布语义） 已定项 1「生效」，2026-09-26 取 SysPre）：F_生效 = max(根上带的 F, 系统配置里读得出的 F)——把 txg 16 的根槽
/// （盘 1 上唯一带 F = 11 的根）改坏，重开之后 F_生效 仍是 11（盘 0 的 txg 15 与两块盘系统配置里先写进去的都带 11）：所选根是 txg 15、
/// 链上 txg 16 的记录照样施加（同实例），回收与根槽都好时一样多，每块盘 defer 里剩 92 个槽。按改之前「各盘所带 F 最大值的最小值」只读根，
/// 盘 1 上最大的是 0，F_生效 回到 0、一个都不回收（向前回退之后这一臂剩多少没重量；旧形态下是 125）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it(
) {
    floor_carried_by_the_system_configuration_takes_effect_on_remount(
        SecondCarrierRootSlot::Damaged,
        92,
        15,
    );
}

/// 同一条脚本、两块盘的载体根都好：所选根是 txg 16，F_生效 与回收同上。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn floor_carried_by_the_system_configuration_takes_effect_on_remount_with_both_carrier_roots_intact(
) {
    floor_carried_by_the_system_configuration_takes_effect_on_remount(
        SecondCarrierRootSlot::Intact,
        92,
        16,
    );
}

/// 抬 F 的第二次空发布（txg 16）落的那个根槽：好着，还是在重开之前被改坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondCarrierRootSlot {
    Intact,
    Damaged,
}

fn floor_carried_by_the_system_configuration_takes_effect_on_remount(
    second_carrier_root_slot: SecondCarrierRootSlot,
    expected_deferred: u64,
    expected_chosen_txg: u64,
) {
    let mut pool = build_through_rollback(match second_carrier_root_slot {
        SecondCarrierRootSlot::Intact => "step-five-effective-intact",
        SecondCarrierRootSlot::Damaged => "step-five-effective-damaged",
    });
    five_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    assert_eq!(second_carrier, CheckpointTxg(16));
    let mut devices = pool.reopen_recorded();
    if second_carrier_root_slot == SecondCarrierRootSlot::Damaged {
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
            "第二块盘的载体 {second_carrier_root_slot:?}：F_生效 = max(根上带的, 系统配置里的) = 11；数里是释放代 12–14 各 16、\
             抬 F 两次空发布各 8、写行放掉的 12 与暖机两次各 8"
        );
    }
}

/// 上限：第 4 个不同状态是 11（按新到旧 14、13、12、11；D 与 A 是同一个状态）⇒ 抬到 12 被拒；
/// 抬到 11 之后上限仍是 11，再抬 12 仍被拒。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reuse_after_raising_the_floor::the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reuse_after_raising_the_floor::the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker,
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reuse_after_raising_the_floor::reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reuse_after_raising_the_floor::reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red,
    );
}

/// 抬到 11 之后把盘 1 的载体（txg 16 的根）改坏一个字节（自证不过），挂着的时候回退到 txg 9 的根 D：那条载体是这个进程写过、FUA 返回过的，
/// 它住的槽在这个进程的根环表里——D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句：读不出或自证不过就重读一次，仍坏就拒。
/// 回退判候选读根环时拒成 `CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { 自证不过, 自证不过 })`，
/// 在任何写之前：盘上逐字节不变、分配器与现行版本不动。
/// 这一条原来钉的是「载体坏了当没有根，F_生效 照 SysPre（max(根上带的, 系统配置里读得出的)）仍是 11、D 拒成 `BelowEffectiveFloor`」；
/// 照全句之后已知根被改坏那一形在判候选时就拒（代码三方第二轮之后的规格第 3 条：已知根被改坏之后回退，改钉拒）。
/// 改坏挪到挂载之前（那时它不是已知根）保不住原来那一判：之后可写挂载的写行与暖机两块盘都写带 F 11 的根，「F 只在系统配置里」
/// 那一形就没了（推的，没造）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write(
) {
    let mut pool = build_through_rollback("step-five-candidate-floor");
    five_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    let target = target_for_publish(
        second_carrier,
        parameters().geometry.root_ring_slots_per_region,
    );
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
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
            Err(RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_bad))
                if **still_bad
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: target,
                        first_reading: BadRootRingSlotReading::NotSelfVerified,
                        reread: BadRootRingSlotReading::NotSelfVerified,
                    }
        ),
        "载体是这个进程写过的，自证不过重读仍坏：{:?}",
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

/// 挂着的回退判候选时 F_生效 取 max(根上带的, 系统配置里读得出的)（D16（发布语义） 已定项 1「生效」，SysPre）：系统配置里的 F
/// 高于环里每条根带的 F、已知根全读得出时，回退用的是系统配置那一个。
/// 造法：固定脚本到 D（A、B、重开取号 2、写行、暖机两次、C（txg 8）、挂着的时候回退到 A 那次向前发布 D（txg 9）），环里每条根带的 F 都是 0；
/// 不经抬 F 的路径，把两块盘四个系统配置槽里的 F 改成 9、经读者解开再由写者重写（整槽校验和随之重封）——即 SysPre 先把新 F 写进每块盘的
/// 系统配置、带新 F 的根还一条没落的那个盘面（抬 F 的第一次空发布失败时留下的样子）。挂着的时候回退到 C（txg 8 < 9）：
/// 拒成 `TargetNotACandidate { BelowEffectiveFloor }`，在任何写之前：盘上逐字节不变、现行版本与分配器不动。
/// 原来守这一判的是 `file_overwrite_step_five_reuse.rs` 的
/// `floor_carried_by_only_one_device_root_and_the_system_configuration_keeps_the_roots_below_it_out_of_the_rollback_candidates` 那一条（载体根被改坏、F 只剩系统配置里那一份），它改钉成「已知根读坏重读仍坏就拒」
/// 之后这一判没有用例了；只取根上带的 F 时 F_生效 是 0、C 是候选、回退照做（`crates/mutations.tsv`）。
#[test]
fn rolling_back_while_mounted_takes_the_floor_from_the_system_configuration_when_it_is_above_every_root_carried_floor(
) {
    let mut pool = build_through_rollback("step-five-floor-only-in-the-system-configuration");
    let planted_floor = CheckpointTxg(9);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for offset in [DeviceOffsetInBytes(0), DeviceOffsetInBytes(spacing)] {
                let bytes = PoolReader::read(
                    &*devices,
                    device,
                    offset,
                    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
                )
                .expect("系统配置槽读得到");
                let mut system_configuration =
                    SystemConfiguration::parse_slot(&bytes).expect("这一槽自证得过");
                assert!(
                    system_configuration.quantities.rollback_floor < planted_floor,
                    "改之前系统配置里的 F 低于 9"
                );
                system_configuration.quantities.rollback_floor = planted_floor;
                let (_, recorded) = devices
                    .iter_mut()
                    .find(|(identity, _)| *identity == device)
                    .expect("那块盘");
                recorded
                    .write_at(
                        offset,
                        &system_configuration.to_slot(),
                        WriteDurability::Plain,
                    )
                    .expect("写回系统配置槽");
            }
        }
    }
    assert_eq!(
        newest_root_floor(&pool),
        CheckpointTxg(0),
        "环里最新那条根带的 F 是 0：F 只在系统配置里"
    );
    let before = common::disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let target = RollbackTarget {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
    };
    let refused = roll_back(&mut pool, target);
    assert!(
        matches!(
            &refused,
            Err(RollbackError::TargetNotACandidate {
                target: refused_target,
                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
            }) if *refused_target == target
        ),
        "F_生效 = max(根上带的 0, 系统配置里的 9) = 9，C（txg 8）在 F 之下：{:?}",
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reuse_after_raising_the_floor::raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reuse_after_raising_the_floor::raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root,
    );
}

/// C502 那一格改成从根指针读表之后，读不出的那一格：现行那一版的根指着的实例表两份都读不出，抬 F 在任何写之前拒绝
/// （`InstanceTableUnreadable`：判不了候选集；读不出与解不开各报各的成员，C866 改法甲），盘上逐字节不变（系统配置槽、根环、录制流步数）。
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
        matches!(refused, Err(MountError::InstanceTableUnreadable)),
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reuse_after_raising_the_floor::four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reuse_after_raising_the_floor::four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reuse_after_raising_the_floor::the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reuse_after_raising_the_floor::the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor,
    );
}
