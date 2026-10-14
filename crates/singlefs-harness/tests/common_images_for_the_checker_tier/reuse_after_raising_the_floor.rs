//! harness 测试文件 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::{
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED, IMPLEMENTED_INVARIANTS,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
};
use singlefs_checker::walk::{
    NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
    NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER, NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
    NO_QUEUED_INODE_IN_ANY_WALKED_VERSION, NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
    NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
    NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD, NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{Placement, ReclaimedReuse};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, MountError, RaisedFloor,
    RollbackError, RollbackTarget, RolledBack, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

/// 没有删除的会话上，删除线那几条各自报的不适用理由（池级 checker `walk` 里为每条写的那一句）：不变量 → 只认的那一句。
/// 前六行是墓碑与删除队列那几条（I-9.18–I-9.23）；后六行是退役树与谱系、退役容器容量那几条（I-9.31–I-9.34、I-9.36、I-9.37），
/// 逐条照池级 checker 在 E 之后的镜像上实际报的登记：这条会话没有 unlink，退役树每一版都是零根，I-9.31 没有内部条目、I-9.32 没有退役记录、
/// I-9.33 与 I-9.34 没有谱系上的退役记录；回退 D 只写了一条谱系记录，谱系树只一片容器、退役树零根，I-9.36 没有非最右的容器；
/// `check_pool_image` 是旧入口、没给谱系与退役旋钮，I-9.37 不判。I-9.35（退役完备）不登记：候选集里相邻两版的父版读得出、叶身份集没少，
/// 真被判过、要成立。
pub const DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION: &[(&str, &str)] = &[
    (
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
    ),
    (
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
    ),
    (
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
    ),
    (
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
    ),
    (
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
        NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY,
    ),
    (
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    ),
    (
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    ),
    (
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
    ),
    (
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    ),
    (
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    ),
    (
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
    ),
    (
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
        NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
    ),
];

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub fn overwrite_in_process(
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
pub fn build_through_rollback(tag: &str) -> BuiltPool {
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
pub fn roll_back(
    pool: &mut BuiltPool,
    target: RollbackTarget,
) -> Result<RolledBack, RollbackError> {
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
pub fn five_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [13usize, 17, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(2))
        })
        .collect()
}

pub fn raise_floor(
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
) -> Result<RaisedFloor, MountError> {
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

pub fn newest_root_floor(pool: &BuiltPool) -> CheckpointTxg {
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
pub fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
    images: &mut PoolCheckerImages,
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
    // 池级 checker 那一遍交 checker 档：I-8.8、I-4.1、I-5.5、I-4.7、I-4.6、I-4.4 与删除线那几条
    // （[`DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION`]）没判到（不适用），别的每一条判到且成立
    // （I-2.1、I-3.1、I-5.2、I-7.2 在里面）。各条不适用的理由串对拍字里没有，checker 档只核没判到。
    let not_applicable_on_this_session: Vec<&'static str> =
        ["I-8.8", "I-4.1", "I-5.5", "I-4.7", "I-4.6", "I-4.4"]
            .into_iter()
            .chain(
                DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION
                    .iter()
                    .map(|(invariant, _)| *invariant),
            )
            .collect();
    let expected = IMPLEMENTED_INVARIANTS.iter().fold(
        PoolCheckerExpectation::no_invariant_violated(),
        |expected, invariant| {
            if not_applicable_on_this_session.contains(invariant) {
                expected.and_not_judged(invariant)
            } else {
                expected.and_holds(invariant)
            }
        },
    );
    images.hand_out("E 发布完、冷启动读回 E 之后", || image, expected);
}

/// 「4 个不同状态」按 (inode 树根指针, extent 树根指针) 去重（D16（发布语义） 已定项 1 的根环容量边界）：回退那次发布照抄目标那一版的
/// 两棵树，新根与目标是同一个状态。D（txg 9）是 A 的状态；覆盖写甲（txg 10）、再挂着的时候回退到 D（txg 11，又是 A 的状态）、覆盖写乙（txg 12）。
/// 按新到旧去重：12、11、10、（9 与 11 同一个状态，不再算）、8 ⇒ 第 4 个不同状态是 C 的 8；每块盘上最新的有效根是 12 与 11，
/// 上限 min(11, 8) = 8，抬到 9 被拒。不去重（每条带文件的根各算一个）的话第 4 新的是 9，上限 9、抬到 9 做成。
/// checker 那一侧同一条规则：把入口骗过（告诉入口今天的 F 是 9，入口按 txg ≥ 9 的根算，不同状态只剩 12、11、10 三个，上限落到最旧的有效根 9）
/// 抬到 9，镜像上 I-7.9（回退下界 F 不高于抬 F 的上限） 按抬之前的 F（0）去重算出上限 8，只红它；checker 不去重的话上限 9、判成立。
pub fn the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker(
    images: &mut PoolCheckerImages,
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
    images.hand_out(
        "抬到上限 8 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-7.9"),
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
    // F 抬到 9、去重之后的上限 8：只红 I-7.9
    images.hand_out(
        "入口被骗过、F 抬到 9 之后",
        || above.memory_pool(),
        PoolCheckerExpectation::exactly_these_violated(&["I-7.9"]),
    );
}

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 mkfs 实例表那 2 槽 50176——
/// F = 0 时 A（txg 3）还是候选、它的根指着那片实例表；checker 走 A 时那片实例表的校验和对不上 ⇒ I-2.1 红。
pub fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
    images: &mut PoolCheckerImages,
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
    // A 的根还是候选，它的数据单元被盖了：I-2.1 红
    // C22（刚释放的块立即重分配） 的「怎么拦」第 ① 条点的是 I-4.8（近 K 代根校验和自洽）：从最近 K 代任一根遍历，校验和必须全对。
    // 只断言 I-2.1 红，拦的是「有一个单元的校验和与父指针对不上」，没说那是从一条**候选根**出发的遍历走进去的
    // （收口表第 22 行 2026-09-17 逐句核出来的那一句）。
    // 复用窗口置 0 之后 I-4.8 必须变红：不红说明这条约束根本没被检查（C22（刚释放的块立即重分配） 第 ② 条）
    images.hand_out(
        "不抬 F 就回收、E 复用 50176 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-2.1")
            .and_violated("I-4.8"),
    );
}

/// C502（抬 F 时现行版本里没有实例表单元）：mkfs 同一个进程里新池新建文件之后再覆盖写三次（txg 4、5、6），抬 F 做成。
/// 这个进程的现行版本是 `publish_first_file` 那一版往下接的，`TransactionOutput::units` 里一个实例表单元都没有；
/// 候选集按现行那一版的根指针上读出来的实例表判（mkfs 那片 0 行的表），不看那个内存数组。
/// 上限 = min(每块盘上最新的有效根 min(6, 4), 第 4 新的非空有效根 3) = 3（D16（发布语义） 已定项 1）；两次空发布 txg 7（盘 1）、8（盘 0）
/// 让两块盘各有一条带 F = 3 的根；释放代 ≤ 3 的只有第一个文件版本换下的 mkfs 那片第 0 版树表（1 槽）。
pub fn raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root(
    images: &mut PoolCheckerImages,
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
    images.hand_out(
        "mkfs 进程里抬 F 到 3 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 建池（第一个文件 A，txg 3）→ 同一个进程里覆盖写 B（txg 4）、C（txg 5）→ 崩溃恢复抛弃 C（C 的根槽与数据单元暂时读不出，
/// 择根落到 B、C 那条记录验不过；实例 2 写行 txg 6、暖机 txg 7；再写回，C 按实例 2 的表判被抛弃）→ 实例 2 覆盖写四次（txg 8–11）。
pub fn pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites(
    tag: &str,
) -> BuiltPool {
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
    pool
}

/// 双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：崩溃恢复抛弃 C（实例 1、txg 5）之后实例 2 覆盖写四次。
/// 那次挂载时 C 的根槽读不出、系统配置没见证 C（C554 乙罩不到的那一格），影子账看不见 C；可写挂载时的预隔离（D23（journal 的角色与格式）
/// 已定项 14「可写挂载时的预隔离」）把水位之上没施加的 C 的记录点名的落点与 C 的账里的落点隔离到这个进程写掉 txg 5 那一槽为止，
/// 四次覆盖写发不出 C 的单元，抬 F 之前池级 checker 一条都不红（`random_histories.rs` 那一条同形、同样改钉）。
/// 判别力：挂载时不隔离那一份（变异「删除线第38批 可写挂载时的预隔离不隔离水位之上没施加的记录点名的落点（reuse_after_raising_the_floor 双故障形）」）
/// 时 C 引用的单元又被发出去，I-7.4（近 K 代块未被复用）红、点名实例 1、txg 5 那条被抛弃的根，这一条红。
pub fn four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
    images: &mut PoolCheckerImages,
) {
    let pool =
        pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites("step-five-gap-5");
    // 抬 F 之前池级 checker 一条都不红（C 的单元被预隔离护着）
    images.hand_out(
        "崩溃恢复抛弃第三版、覆盖写四次之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7），**不承载事务的空发布写 0 也不许把计数拉回去**。
///
/// 抬 F 推的两次空发布在记录上写事务号 0；发布 E 若取「上一条记录的事务号 + 1」就拿到 0 + 1 = 1，
/// 与实例 2 的 C 用过的 1 重号。回退那次发布 D 的记录也写 0（它不施加也不删除任何记录，实现取 0，条款没写，见实三报告）。重号之后同一实例的两个版本写序逐字节相同（写序存事务号低 48 位），
/// I-1.8（归并后版本全序） 判不开它们，而实例表行的 W 能当精确前缀也正是靠「记录按事务号顺序追加」这条纪律。
pub fn the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor(
    images: &mut PoolCheckerImages,
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
    // I-8.7 要真被评估过且成立：中间夹着的空发布记录不进那个序列
    images.hand_out(
        "抬 F 两次空发布之后发布 E",
        || image,
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-8.7"),
    );
}
