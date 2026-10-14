//! harness 测试文件 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, disk_snapshot,
    file_content, parameters, rollback_after_a_recovery_abandoned_a_root_that_created_inodes,
    BuiltPool, Recorded, RollbackAfterARecoveryAbandonedARootThatCreatedInodes,
    FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    is_a_user_visible_unit, mount_writable, roll_back_by_a_forward_publish, Mounted,
    RollbackCandidateExclusion, RollbackError, RollbackTarget, RolledBack,
    StillUnreadableAfterOneReread,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, readable_roots, recover,
    user_visible_tree_root_pointers, BadRootRingSlotReading, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, resend_the_frozen_publish, FirstFile, PoolWriter,
    PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::MemoryPool;
use std::collections::BTreeSet;

pub const SECOND_FILE_BYTES: usize = 4100;

pub const THIRD_FILE_BYTES: usize = 2500;

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

pub fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
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

/// 固定脚本到 C 的池，连同 A、B、C 三版（进程内存里的，D 之前的样子）。
pub struct ThroughTheThirdPublish {
    pub pool: BuiltPool,
    pub first: TransactionOutput,
    pub third: TransactionOutput,
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
pub fn build_through_third_publish(tag: &str) -> ThroughTheThirdPublish {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    ThroughTheThirdPublish { pool, first, third }
}

pub fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

pub fn third_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
    }
}

/// 挂着的时候回退：交回回退的结局，做成时 `pool.output` 已换成回退那次发布之后的一版。
pub fn roll_back(
    pool: &mut BuiltPool,
    target: RollbackTarget,
) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
}

/// 交 checker 档：池级 checker 一条违例都没有，`judged_and_holding` 那几条要真被判过（不是「不适用」）且成立。
pub fn hand_out_without_any_violation(
    images: &mut PoolCheckerImages,
    step: &str,
    build_the_image: impl FnOnce() -> MemoryPool,
    judged_and_holding: &[&'static str],
) {
    let expected = judged_and_holding.iter().fold(
        PoolCheckerExpectation::no_invariant_violated(),
        |expected, invariant| expected.and_holds(invariant),
    );
    images.hand_out(step, build_the_image, expected);
}

/// I-3.1 的违例细节里记账的已分配多于遍历全部有效根得到的。
pub fn allocated_statistic_counts_more_than_the_walk(detail: &str) -> bool {
    singlefs_harness::history::allocated_and_walked_bytes(detail)
        .is_some_and(|(allocated, walked)| allocated > walked)
}

/// I-3.1 的违例细节里记账的已分配比遍历全部有效根得到的恰好多 `bytes`。
pub fn allocated_statistic_counts_these_bytes_more_than_the_walk(detail: &str, bytes: u64) -> bool {
    singlefs_harness::history::allocated_and_walked_bytes(detail)
        .is_some_and(|(allocated, walked)| allocated > walked && allocated - walked == bytes)
}

/// 挂载那一刻读坏（撕裂）或读回全零的那个根槽钉在它可能住过的最旧 txg，回收门槛停在那里（用户 2026-10-05 认下的代价：「钉最旧 txg 让 I-3.1
/// 在合法盘面上红在记账多的一边……用例钉『只许红在记账多的一边』，池级 checker 不放宽」，`records/2026-09-30-里程碑三开工第一轮.md`「钉最旧 txg 让 I-3.1 在合法盘面上红在记账多的一边」那一句；
/// 读回全零照自证不过钉住，同一条记录里「② 读回全零的槽在回收门槛里照自证不过钉住」那一句，第 10c 批）：池级 checker 只判 I-3.1 红，而且红在记账多的一边（记账的已分配比遍历全部有效根多）。
/// 交 checker 档：只核方向，不核大小（用户认的口径是方向加时长，判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第三节第一行）。
pub fn hand_out_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
    images: &mut PoolCheckerImages,
    step: &str,
    build_the_image: impl FnOnce() -> MemoryPool,
) {
    images.hand_out(
        step,
        build_the_image,
        PoolCheckerExpectation::exactly_these_violated(&["I-3.1"])
            .and_if_violated_the_detail_satisfies(
                "I-3.1",
                "记账的已分配多于遍历全部有效根得到的",
                allocated_statistic_counts_more_than_the_walk,
            ),
    );
}

/// 同 [`hand_out_red_only_by_the_allocated_statistic_counting_more_than_the_walk`]，再钉多出来的字节数
/// `expected_bytes_counted_more_than_the_walk`（挂载那一刻：慢档 4 条调查报告第 3.1 节，判决 `research/prompts/m3-batch10-code-r2-main-verification.md`
/// 第三节最后一行：多出来的是盘 0 上释放代在 (钉住的 txg, checker 取的环里最旧的自证根] 里的已释放字节）。
pub fn hand_out_red_only_by_the_allocated_statistic_counting_these_bytes_more_than_the_walk(
    images: &mut PoolCheckerImages,
    step: &str,
    build_the_image: impl FnOnce() -> MemoryPool,
    expected_bytes_counted_more_than_the_walk: u64,
) {
    images.hand_out(
        step,
        build_the_image,
        PoolCheckerExpectation::exactly_these_violated(&["I-3.1"])
            .and_if_violated_the_detail_satisfies_given_a_value(
                "I-3.1",
                "记账的已分配比遍历全部有效根得到的恰好多这些字节",
                expected_bytes_counted_more_than_the_walk,
                allocated_statistic_counts_these_bytes_more_than_the_walk,
            ),
    );
}

/// 恢复之后重开的那个进程接着覆盖写（实例 3），一直写到 txg `txg_writing_the_pinned_slot`（它的根槽就是挂载时钉住的那一槽，28 + 一圈）：
/// 写到它前一次为止，每一次覆盖写之后 I-3.1 都只红在记账多的一边（逐步核方向、不钉大小：多记的字节随覆盖写涨，第三轮两腿量过），
/// 写掉那一槽那一次之后池级 checker 一条违例都没有（钉住只到这个进程把那个槽写掉为止，
/// `allocator.rs` 的 `RootRingOccupant::AllZeroAtMount` / `BadAtMount`）。
pub fn overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
    images: &mut PoolCheckerImages,
    pool: &mut BuiltPool,
    txg_writing_the_pinned_slot: u64,
    step: &str,
) {
    assert_eq!(
        root_slot_position_of(txg_writing_the_pinned_slot),
        root_slot_position_of(28),
        "txg {txg_writing_the_pinned_slot} 的根槽就是挂载时钉住的那一槽"
    );
    let mut overwrite = 0usize;
    // 迭代上界：从挂载交回的那一版写到钉住那一槽的前一次（不到一圈）；没有提前出口（方向不对就断言失败）。
    while pool.output.root.checkpoint_txg.0 + 1 < txg_writing_the_pinned_slot {
        overwrite_in_process(
            pool,
            &content_of(1800 + overwrite, 61 + overwrite),
            InstanceGeneration(3),
        );
        overwrite += 1;
        hand_out_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
            images,
            &format!(
                "{step}：第 {overwrite} 次覆盖写之后（txg {}），那一槽还没被写掉",
                pool.output.root.checkpoint_txg.0
            ),
            || pool.memory_pool(),
        );
    }
    assert!(
        overwrite > 0,
        "{step}：挂载交回的那一版到钉住那一槽之间至少有一次覆盖写"
    );
    overwrite_in_process(pool, &content_of(1900, 97), InstanceGeneration(3));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(txg_writing_the_pinned_slot)
    );
    hand_out_without_any_violation(
        images,
        &format!("{step}：写掉钉住的那一槽（txg {txg_writing_the_pinned_slot}）之后"),
        || pool.memory_pool(),
        &[],
    );
}

pub fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一条根槽在盘上的位置：哪块盘、哪个偏移。槽距是 `fixed_structure_slot_spacing`，槽自己的判定宽度是 `physical_block_size`。
pub fn root_slot_position_of(txg: u64) -> (DeviceIdentity, DeviceOffsetInBytes) {
    (
        region_device(txg),
        slot_offset(
            target_for_publish(
                CheckpointTxg(txg),
                parameters().geometry.root_ring_slots_per_region,
            ),
            parameters().geometry.fixed_structure_slot_spacing,
        ),
    )
}

pub fn root_slot_bytes() -> usize {
    usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")
}

pub fn read_root_slot(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    (device, offset): (DeviceIdentity, DeviceOffsetInBytes),
) -> Vec<u8> {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("这条根的根槽所在的盘");
    let mut slot_bytes = vec![0u8; root_slot_bytes()];
    recorded.read_at(offset, &mut slot_bytes).expect("读根槽");
    slot_bytes
}

pub fn write_root_slot(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    (device, offset): (DeviceIdentity, DeviceOffsetInBytes),
    slot_bytes: &[u8],
) {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("这条根的根槽所在的盘");
    recorded
        .write_at(offset, slot_bytes, WriteDurability::Plain)
        .expect("写根槽");
}

/// 一版的用户可见单元在那一版里的落点（第一条位置条目的槽；两盘同槽），按角色。
pub fn user_visible_slots(version: &TransactionOutput) -> BTreeSet<(TransactionUnit, u64)> {
    version
        .units
        .iter()
        .filter(|unit| is_a_user_visible_unit(unit.identity))
        .map(|unit| (unit.identity, unit.slot.0))
        .collect()
}

pub fn record_of(
    records: &[AllocationRecord],
    device: DeviceIdentity,
    slot: u64,
) -> Option<AllocationRecord> {
    records
        .iter()
        .copied()
        .find(|record| record.device == device && record.slot.0 == slot)
}

/// 验收第一条：回退到 A 的根 (1, 3) 是一次普通发布 D (2, 9)——jsn 接着 C，只重写固定点单元（不写数据单元、不写实例表）；
/// D 的实例表指针就是 C 的，两棵用户可见树的根指针与 A 树表里的逐字节相同；C 的四个用户可见单元释放（释放代 9），
/// A 的四个在 C 的账里已释放的逐盘改回已分配（分配代 3）；水位照 C 的；系统配置的实例代号仍是 2；冷启动择 D 读回第一次的内容；
/// checker 全绿，I-3.1、I-3.9、I-3.11 真被判过。
pub fn rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content(
    images: &mut PoolCheckerImages,
) {
    let ThroughTheThirdPublish {
        mut pool,
        first,
        third,
        ..
    } = build_through_third_publish("step-four-forward");
    let rolled_back = roll_back(&mut pool, first_root()).expect("回退到 A");
    let rollback_version = pool.output.clone();
    assert_eq!(
        (
            rollback_version.root.instance,
            rollback_version.root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(9)),
        "D 接着 C：txg 9、实例仍是 2（不取号）"
    );
    assert_eq!(
        rollback_version.record.counter,
        third.record.counter + 1,
        "jsn 接着 C 那一条"
    );
    assert_eq!(
        rollback_version.record.transaction, 0,
        "D 不写数据单元，事务号写 0（同空发布）"
    );
    assert!(
        rollback_version
            .rewritten
            .iter()
            .all(|role| !is_a_user_visible_unit(*role) && !role.is_a_page_of_the_instance_table()),
        "D 只重写固定点单元：{:?}",
        rollback_version.rewritten
    );
    assert_eq!(
        rollback_version.root.instance_table, third.root.instance_table,
        "实例表照 C 的：同一条指针、同一份字节"
    );
    let image = pool.memory_pool();
    assert_eq!(
        user_visible_tree_root_pointers(&image, &rollback_version.root).expect("D 的树表"),
        user_visible_tree_root_pointers(&image, &first.root).expect("A 的树表"),
        "D 树表里 inode 树与 extent 树的根指针与 A 的逐字节相同"
    );
    assert_eq!(
        user_visible_slots(&rollback_version),
        user_visible_slots(&first),
        "D 的用户可见单元就是 A 的"
    );
    let released: BTreeSet<(TransactionUnit, u64)> = rolled_back
        .released_user_visible_units
        .iter()
        .map(|(unit, placement)| (*unit, placement.slot.0))
        .collect();
    assert_eq!(
        released,
        user_visible_slots(&third),
        "释放集：C 引用、D 不引用的用户可见单元（A 与 C 一个都不共用）"
    );
    assert!(rolled_back.quarantined_user_visible_copies.is_empty());
    let expected_resurrected: BTreeSet<(DeviceIdentity, u64, CheckpointTxg)> =
        user_visible_slots(&first)
            .into_iter()
            .flat_map(|(_, slot)| {
                [DeviceIdentity(0), DeviceIdentity(1)]
                    .map(|device| (device, slot, CheckpointTxg(3)))
            })
            .collect();
    let resurrected: BTreeSet<(DeviceIdentity, u64, CheckpointTxg)> = rolled_back
        .resurrected_user_visible_copies
        .iter()
        .map(|copy| {
            (
                copy.device,
                copy.placement.slot.0,
                copy.allocation_generation,
            )
        })
        .collect();
    assert_eq!(
        resurrected, expected_resurrected,
        "复活集：A 的四个用户可见单元在 C 的账里都已释放（B 换下的），逐盘改回、分配代写回 3"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for (_, slot) in user_visible_slots(&first) {
            let record = record_of(&rollback_version.allocation_records, device, slot)
                .expect("A 的单元在 D 的账里有记录");
            assert!(
                !record.is_released && record.generation == CheckpointTxg(3),
                "D 的账：A 的单元仍分配、分配代 3：{record:?}"
            );
        }
        for (_, slot) in user_visible_slots(&third) {
            let record = record_of(&rollback_version.allocation_records, device, slot)
                .expect("C 的单元在 D 的账里有记录");
            assert!(
                record.is_released && record.generation == CheckpointTxg(9),
                "D 的账：C 的单元已释放、释放代 9：{record:?}"
            );
        }
    }
    assert_eq!(
        rollback_version.inode_number_watermark(),
        third.inode_number_watermark(),
        "inode 号水位 = max(C 内存里的, 环里的)"
    );
    assert_eq!(
        rollback_version.root.tree_identifier_watermark,
        third.root.tree_identifier_watermark
    );
    assert_eq!(
        choose_system_configuration(&image)
            .expect("系统配置")
            .quantities
            .journal_instance,
        InstanceGeneration(2),
        "系统配置的实例代号仍是 2"
    );
    hand_out_without_any_violation(
        images,
        "D 之后",
        || image,
        &["I-3.1", "I-3.9", "I-3.11", "I-9.6"],
    );
    let cold = pool.reopen_cold();
    let report = recover(&cold, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(9)),
            content: file_content()
        },
        "冷启动择 D、读回第一次的内容"
    );
}

/// 验收「释放与复活」：D 之后接着覆盖写 E，E 换下的是 D 引用的 A 的那几个单元——它们在 D 的账里是仍分配的（复活过），
/// 发布路径照常释放；checker 全绿。只释放不复活时 D 的账里 A 的单元是已释放的：I-3.11 在 D 上红，E 被释放判定路径拒
/// （`ReleaseTargetAlreadyReleased`，第一轮判决第二节 F1 里 H3 那一格）——`crates/mutations.tsv` 里拿掉复活那一步的那一行证它。
pub fn after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds(
    images: &mut PoolCheckerImages,
) {
    let ThroughTheThirdPublish { mut pool, .. } =
        build_through_third_publish("step-four-forward-then-overwrite");
    roll_back(&mut pool, first_root()).expect("回退到 A");
    hand_out_without_any_violation(
        images,
        "D 之后",
        || pool.memory_pool(),
        &["I-3.9", "I-3.11"],
    );
    let fourth_content = content_of(3300, 17);
    let fourth = overwrite_in_process(&mut pool, &fourth_content, InstanceGeneration(2));
    assert_eq!(fourth.root.checkpoint_txg, CheckpointTxg(10));
    hand_out_without_any_violation(
        images,
        "E 之后",
        || pool.memory_pool(),
        &["I-3.9", "I-3.11"],
    );
    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&cold, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(10)),
            content: fourth_content
        }
    );
}

/// 验收「D 之后再回退到 C 的根」：向前发布不抛弃 B、C，D 之后 C (2, 8) 仍是候选；再回退一次（txg 10）读回第三次的内容，checker 全绿。
pub fn after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content(
    images: &mut PoolCheckerImages,
) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-forward-back-to-c");
    roll_back(&mut pool, first_root()).expect("回退到 A");
    roll_back(&mut pool, third_root()).expect("D 之后再回退到 C");
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(10));
    assert_eq!(user_visible_slots(&pool.output), user_visible_slots(&third));
    hand_out_without_any_violation(images, "回退到 C 之后", || pool.memory_pool(), &[]);
    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&cold, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(10)),
            content: third_content()
        }
    );
}

/// 崩溃恢复造出被抛弃的根（D23（journal 的角色与格式） 已定项 14 射程：影子账与按实例表判抛弃留着，理由是崩溃恢复——
/// 第一轮判决 H6 那一形）：C 之后进程退出，C 的根槽与 C 的数据单元（两份）暂时读不出（暂存、清零），重开可写挂载——择根落到
/// (2, 7)，C 那条记录施加前验点名单元失败、不施加；新实例 3 写行与暖机（txg 9、10）。再把暂存的字节原样写回：C 的根 (2, 8)
/// 又读得出，按实例 3 的实例表判是被抛弃的。交回那次挂载——它看不见 C，影子账隔离 0；之后的挂载才看得见 C。
pub fn abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(
    pool: &mut BuiltPool,
    third: &TransactionOutput,
) -> Mounted {
    let mounted = abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(pool, third);
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(7)),
        "择根落到 (2, 7)，C 那条记录不施加"
    );
    assert_eq!(mounted.output.instance, InstanceGeneration(3));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(10),
        "实例 3 写行 txg 9、暖机 txg 10"
    );
    mounted
}

/// 验收「候选集」：根环里没有的 (2, 42)、被崩溃恢复抛弃的时间线上的 (2, 8)、树表 0 条的暖机根 (1, 2) 都在任何写之前拒，
/// 报各自那一条排除；盘上逐字节不变（系统配置四槽、根环、录制流一步不多），分配器与现行版本不动。txg 低于 F_生效 那一格在
/// `rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`（F 只住系统配置时同样拒）。
pub fn rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write(
    images: &mut PoolCheckerImages,
) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-forward-candidates");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    for (target, expected_exclusion) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(42),
            },
            RollbackCandidateExclusion::NotInRing,
        ),
        (
            third_root(),
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(2),
            },
            RollbackCandidateExclusion::VersionWithoutFile,
        ),
    ] {
        let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
        let current_before = pool.output.clone();
        let records_before = pool.allocator.records().to_vec();
        match roll_back(&mut pool, target) {
            Err(RollbackError::TargetNotACandidate {
                target: refused,
                exclusion,
            }) => {
                assert_eq!(refused, target);
                assert_eq!(exclusion, expected_exclusion, "{target:?}");
            }
            other => panic!("回退到 {target:?} 要拒成 {expected_exclusion:?}，交回 {other:?}"),
        }
        assert_eq!(
            disk_snapshot(&pool.memory_pool(), &pool.stream),
            before,
            "{target:?}：拒之前盘上一个字节都不动"
        );
        assert_eq!(pool.output, current_before);
        assert_eq!(pool.allocator.records(), records_before.as_slice());
    }
    roll_back(&mut pool, first_root()).expect("A (1, 3) 仍是候选：实例 1 那一行是 (1, 4, ·)");
    hand_out_without_any_violation(
        images,
        "崩溃恢复抛弃 C 之后回退到 A",
        || pool.memory_pool(),
        &[],
    );
}

/// C 那一版新建了三个 inode（号 2、3、4，水位 5），回退那一刻 C 的根槽被改成全 0（自证不过）。C 的根是这个进程写过、FUA 返回过的，
/// 它住的槽在这个进程的根环表里：D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句——读不出或自证不过就重读一次，仍坏就拒——
/// 管理员回退判候选读根环时拒成 `CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { 自证不过, 自证不过 })`，
/// 在任何写之前：盘上逐字节不变、现行版本与分配器不动。根槽改回原样之后回退照做：D 的 inode 号水位 5，D 之后再新建的 inode 从 5 起、
/// 不重发 2–4；树 ID 水位同一个取法。
/// 这一条原来钉的是「C 的根槽读不出、回退照做、水位取内存里的」（合入后验证一只管「读不出」时自证不过当没有根）；照全句之后那一形在
/// 判候选时就拒（代码三方第二轮之后的规格第 3 条：已知根被改坏之后回退，改钉拒）。「水位取 max(内存里的, 环里读得出的)」那一判挪到
/// `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的
/// `the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root`
/// （判候选两遍读得出、算水位那一遍瞬时读不出），`crates/mutations.tsv` 里实三「水位」那一行跟着换靶。
pub fn rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("step-four-forward-watermark");
    let _second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut reopened = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut reopened).expect("可写挂载");
    pool.devices = Some(reopened);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third = {
        let publish_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let previous = pool.output.clone();
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            3,
            FIXED_WRITE_TIME_SECONDS + 120,
            InstanceGeneration(2),
        )
        .expect("C 新建三个 inode")
    };
    pool.output = third.clone();
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(8));
    assert_eq!(third.inode_number_watermark(), 5);
    let third_root_slot = root_slot_position_of(8);
    let saved_third_root_slot = {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let saved = read_root_slot(open_devices, third_root_slot);
        write_root_slot(open_devices, third_root_slot, &vec![0u8; root_slot_bytes()]);
        saved
    };
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let refused = roll_back(&mut pool, first_root());
    assert!(
        matches!(
            &refused,
            Err(RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_bad))
                if **still_bad
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: target_for_publish(
                            CheckpointTxg(8),
                            parameters().geometry.root_ring_slots_per_region,
                        ),
                        first_reading: BadRootRingSlotReading::NotSelfVerified,
                        reread: BadRootRingSlotReading::NotSelfVerified,
                    }
        ),
        "C 的根槽是这个进程写过的，自证不过重读仍坏：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前"
    );
    assert_eq!(pool.output, current_before, "现行版本不动");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器不动"
    );
    {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        write_root_slot(open_devices, third_root_slot, &saved_third_root_slot);
    }
    roll_back(&mut pool, first_root()).expect("C 的根槽改回原样之后回退到 A");
    assert_eq!(
        pool.output.inode_number_watermark(),
        5,
        "D 的 inode 号水位 = max(C 内存里的 5, 环里读得出的根的)"
    );
    assert_eq!(
        pool.output.root.tree_identifier_watermark, third.root.tree_identifier_watermark,
        "树 ID 水位同一个取法"
    );
    let after_the_rollback = {
        let publish_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let previous = pool.output.clone();
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            2,
            FIXED_WRITE_TIME_SECONDS + 180,
            InstanceGeneration(2),
        )
        .expect("D 之后新建两个 inode")
    };
    let inode_numbers: BTreeSet<u64> = after_the_rollback
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().map(|record| record.inode))
        .collect();
    assert_eq!(
        inode_numbers,
        BTreeSet::from([1, 5, 6]),
        "D 之后新建的号从 5 起：C 发过的 2–4 不重发"
    );
    hand_out_without_any_violation(
        images,
        "D、新建两个 inode 之后",
        || pool.memory_pool(),
        &["I-9.6"],
    );
}

/// C558（回退目标是环里最旧的根时回退那次发布会写坏它） 的注入方式：回退那次发布的根槽写那一次，按 `mode` 失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootSlotWriteFailure {
    /// 一个字节都不写，报块设备错。
    RefusedBeforeWriting,
    /// 只写下前一半（新内容），后一半留旧内容，报块设备错：撕裂的根槽。
    TornAfterTheFirstHalf,
    /// 整槽写成 0，报块设备错。
    ZeroesWritten,
}

/// 一块盘上某一个偏移的第一次写按 `mode` 失败，之后照转；别的写、读、屏障都照转。
pub struct FirstWriteAtOneOffsetFails<Inner: BlockDevice> {
    pub inner: Inner,
    pub failing_offset: DeviceOffsetInBytes,
    pub mode: RootSlotWriteFailure,
    pub has_failed: bool,
}

impl<Inner: BlockDevice> BlockDevice for FirstWriteAtOneOffsetFails<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if offset == self.failing_offset && !self.has_failed {
            self.has_failed = true;
            match self.mode {
                RootSlotWriteFailure::RefusedBeforeWriting => {}
                RootSlotWriteFailure::TornAfterTheFirstHalf => {
                    // 按整个物理块写（录制设备不收不满一块的写）：前一半是新内容，后一半是槽里原有的旧内容。
                    let mut torn = vec![0u8; bytes.len()];
                    self.inner.read_at(offset, &mut torn)?;
                    let half = bytes.len() / 2;
                    torn[..half].copy_from_slice(&bytes[..half]);
                    self.inner.write_at(offset, &torn, WriteDurability::Plain)?;
                }
                RootSlotWriteFailure::ZeroesWritten => {
                    self.inner
                        .write_at(offset, &vec![0u8; bytes.len()], WriteDurability::Plain)?;
                }
            }
            return Err(injected_block_device_error("回退那次发布的根槽写"));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// C558 的池：A（txg 3）、B（txg 4）之后同一个进程里覆盖写到 txg 10，重开取号 2（写行、暖机），再覆盖写到 txg 27：
/// 根环 24 槽里装的是 txg 4–27，最旧的是 B (1, 4)。回退到 B 的那次发布是 txg 28，它的根槽就是 B 住的那一槽（28 与 4 差一圈）。
/// 转过根环之前重开一次，让转环时的分配器是挂载重建的那一份——`build_pool` 的分配器没有根环表，转过根环之后 I-3.1（已分配统计对得上）
/// 红（C551（`build_pool` 的分配器没有根环表））。交回池与 B 的内容。
pub fn build_a_full_ring_whose_oldest_root_is_the_second_version(
    tag: &str,
) -> (BuiltPool, Vec<u8>) {
    let mut pool = build_pool(tag);
    let content_of_the_oldest_ring_root = content_of(1500, 23);
    overwrite_in_process(
        &mut pool,
        &content_of_the_oldest_ring_root,
        InstanceGeneration(1),
    );
    let mut overwrite = 0usize;
    while pool.output.root.checkpoint_txg.0 < 10 {
        overwrite_in_process(
            &mut pool,
            &content_of(1000 + overwrite, 29 + overwrite),
            InstanceGeneration(1),
        );
        overwrite += 1;
    }
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("转环之前重开");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    while pool.output.root.checkpoint_txg.0 < 27 {
        overwrite_in_process(
            &mut pool,
            &content_of(1000 + overwrite, 29 + overwrite),
            InstanceGeneration(2),
        );
        overwrite += 1;
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(27));
    assert_eq!(
        root_slot_position_of(28),
        root_slot_position_of(4),
        "回退那次发布（txg 28）的根槽就是 B 住的那一槽"
    );
    (pool, content_of_the_oldest_ring_root)
}

pub fn oldest_ring_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(4),
    }
}

/// 回退到 B，回退那次发布的根槽写按 `mode` 失败：交回回退的错。设备换回录制设备装回池里（注入只包这一次）。
pub fn roll_back_to_the_oldest_ring_root_with_its_root_slot_write_failing(
    pool: &mut BuiltPool,
    mode: RootSlotWriteFailure,
) -> RollbackError {
    let (failing_device, failing_offset) = root_slot_position_of(28);
    let devices = pool.devices.take().expect("镜像还开着");
    let mut wrapped: Vec<(DeviceIdentity, FirstWriteAtOneOffsetFails<Recorded>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FirstWriteAtOneOffsetFails {
                    inner: device,
                    failing_offset: if identity == failing_device {
                        failing_offset
                    } else {
                        DeviceOffsetInBytes(u64::MAX)
                    },
                    mode,
                    has_failed: false,
                },
            )
        })
        .collect();
    let refusal = roll_back_by_a_forward_publish(
        &parameters(),
        &mut wrapped,
        &mut pool.allocator,
        &mut pool.output,
        oldest_ring_root(),
    )
    .expect_err("根槽写注入失败，回退那次发布报错");
    assert!(
        wrapped.iter().any(|(_, device)| device.has_failed),
        "{mode:?}：注入真的打在了那一次根槽写上"
    );
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    refusal
}

/// C558 判可达性（主 agent 2026-09-26 立：两块盘、回退目标取环里最旧那一条、在回退那次发布的根槽写上逐点注入失败，看之后同一条回退还能不能再做）：
/// 回退目标 B (1, 4) 是环里最旧的根，回退那次发布 D（txg 28）的根槽就是 B 住的那一槽。根槽写三种失败（一字节不写、写下一半撕裂、写成 0）
/// 各跑两条路：
/// - 同一个进程里：D 冻结在分配器上（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」），再调同一条回退在任何读写之前拒成
///   `PublishFrozenAfterAWriteFailureIsNotResentYet`；原样重发 D 做成，冷启动读回 B 的内容——回退做成了。
/// - 进程在重发之前崩了：D 的记录在根槽写之前已持久（D16（发布语义） 已定项 7 的持久顺序），恢复由记录重建 D 的根（已定项 15），
///   读回 B 的内容；B 的根槽哪怕被撕坏，回退也已经生效，不用再做一次。
///
/// 两条路上回退都没有丢，C558 说的「退不回去」这两条路上走不到。checker 在重发之后全绿；崩在重发之前、
/// B 的根槽被写坏的两格，恢复之前的镜像上 I-3.1（已分配统计对得上） 红，与普通发布同形（见用例里那一段）；崩溃之后重开时一字节不写那一格全绿，
/// 撕裂、写成 0 那两格挂载把那一槽钉在它可能住过的最旧 txg（用户 2026-10-05 认下的代价），只许 I-3.1 红在记账多的一边、差 819200，
/// 这个进程写掉那一槽（txg 52）之后全绿。
pub fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash(
    images: &mut PoolCheckerImages,
) {
    failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
        images,
        RootSlotWriteFailure::RefusedBeforeWriting,
    );
}

pub fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash(
    images: &mut PoolCheckerImages,
) {
    failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
        images,
        RootSlotWriteFailure::TornAfterTheFirstHalf,
    );
}

pub fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash(
    images: &mut PoolCheckerImages,
) {
    failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
        images,
        RootSlotWriteFailure::ZeroesWritten,
    );
}

pub fn failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
    images: &mut PoolCheckerImages,
    mode: RootSlotWriteFailure,
) {
    let (mut pool, content_of_the_oldest_ring_root) =
        build_a_full_ring_whose_oldest_root_is_the_second_version(&format!(
            "step-four-c558-in-process-{mode:?}"
        ));
    let refusal =
        roll_back_to_the_oldest_ring_root_with_its_root_slot_write_failing(&mut pool, mode);
    assert!(
        matches!(&refusal, RollbackError::Publish(failed) if matches!(failed.cause, PublishError::BlockDevice(_))),
        "{mode:?}：{refusal:?}"
    );
    assert!(
        pool.allocator.frozen_publish().is_some(),
        "{mode:?}：D 冻结着"
    );
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(27),
        "{mode:?}：调用方的现行版本还是 txg 27"
    );
    match roll_back(&mut pool, oldest_ring_root()) {
        Err(RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet {
            checkpoint_txg,
            ..
        }) => assert_eq!(checkpoint_txg, CheckpointTxg(28), "{mode:?}"),
        other => panic!("{mode:?}：冻结着时再回退要先拒：{other:?}"),
    }
    let resent = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        resend_the_frozen_publish(&mut writer, &mut pool.allocator)
            .expect("原样重发 D")
            .expect("冻结着一次")
    };
    pool.output = resent.into_file_version().expect("D 带文件");
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(28));
    hand_out_without_any_violation(
        images,
        &format!("{mode:?}：重发之后"),
        || pool.memory_pool(),
        &[],
    );
    let cold = pool.reopen_cold();
    let report = recover(&cold, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(28)))
    );
    assert!(
        matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == content_of_the_oldest_ring_root),
        "{mode:?}：重发之后读回 B 的内容：{:?}",
        report.outcome
    );

    let (mut crashed, content_of_the_oldest_ring_root_in_the_crashed_pool) =
        build_a_full_ring_whose_oldest_root_is_the_second_version(&format!(
            "step-four-c558-crash-{mode:?}"
        ));
    roll_back_to_the_oldest_ring_root_with_its_root_slot_write_failing(&mut crashed, mode);
    let expected_violated_before_recovery: &[&str] = match mode {
        RootSlotWriteFailure::RefusedBeforeWriting => &[],
        // B 的根槽被写坏、D 的记录还没被哪次恢复施加：最新根 txg 27 的账还算着只被 B 引用、释放代 5 的单元，遍历里少了 B——
        // I-3.1（已分配统计对得上） 记账多于遍历。普通覆盖写在同一格上把根槽写坏，checker 判出的是逐字相同的一条（实三草稿里量过），
        // 与回退无关；恢复重开之后全绿（下面）。这一格 checker 该不该判红交主 agent（实三报告「交主 agent 的设计问题」）。
        RootSlotWriteFailure::TornAfterTheFirstHalf | RootSlotWriteFailure::ZeroesWritten => {
            &["I-3.1"]
        }
    };
    images.hand_out(
        &format!("{mode:?}：崩在重发之前、恢复之前"),
        || crashed.memory_pool(),
        PoolCheckerExpectation::exactly_these_violated(expected_violated_before_recovery),
    );
    let cold_after_the_crash = crashed.reopen_cold();
    let report_after_the_crash = recover(&cold_after_the_crash, JournalPolicy::Consult);
    assert_eq!(
        report_after_the_crash.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(28))),
        "{mode:?}：D 的记录已持久，恢复由记录重建 D 的根"
    );
    assert!(
        matches!(&report_after_the_crash.outcome, RecoveryOutcome::FileRead { content, .. } if *content == content_of_the_oldest_ring_root_in_the_crashed_pool),
        "{mode:?}：崩在重发之前，读回 B 的内容：{:?}",
        report_after_the_crash.outcome
    );
    drop(cold_after_the_crash);
    let mut devices = crashed.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("崩溃之后可写挂载");
    assert_eq!(
        (
            remounted.output.effective_root.instance,
            remounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(28)),
        "{mode:?}：新实例接在 D 后面"
    );
    crashed.devices = Some(devices);
    let step = format!("{mode:?}：崩溃之后重开");
    match mode {
        RootSlotWriteFailure::RefusedBeforeWriting => {
            hand_out_without_any_violation(images, &step, || crashed.memory_pool(), &[]);
        }
        // B 的根槽（D 那一次写坏的那一槽）挂载时撕裂读坏、或读回全零：钉在它可能住过的最旧 txg 4，回收门槛停在 4，checker 取环里最旧的自证根 8；
        // 多记账的是盘 0 上释放代 5–8 的已释放字节（慢档调查报告第 3.1 节：Torn、Zeroes 两种都是 819200 = 6406144 − 5586944）。
        RootSlotWriteFailure::TornAfterTheFirstHalf | RootSlotWriteFailure::ZeroesWritten => {
            hand_out_red_only_by_the_allocated_statistic_counting_these_bytes_more_than_the_walk(
                images,
                &step,
                || crashed.memory_pool(),
                819_200,
            );
            crashed.allocator = remounted.allocator;
            crashed.output = remounted
                .current
                .into_file_version()
                .expect("崩溃之后重开，现行那一版带文件");
            overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
                images,
                &mut crashed,
                52,
                &step,
            );
        }
    }
}

/// C542 剩下的普通可写挂载那一支（`checks-owed.md` C542 行「所选根是最新根时，这次挂载的写行与暖机能不能把环里比它旧的有效根全盖掉」，
/// 调度记录实现批次排法实四那一项「C542 普通挂载那一支用例判可达」）：**盖不掉，走不到**。取号之前的预演不做释放前读盘核
/// （`mount.rs` 的 `dry_run_of_the_publishes_after_acquisition`），它与真发只在「这一串换下、核出对不上被隔离的那一份在这一串里被回收」时分叉；
/// 这一串换下的单元释放代 ≥ 这一串第一次发布的 txg（28），回收要环里最旧的有效根 ≥ 28，而所选的最新根 txg 27 在这一串之后还在环里
/// （这一串写 2–3 条根，只盖环里最旧的那几槽，根环 24 槽）⇒ 这一串自己换下的一个都回收不了，预演与真发取的落点逐项相同，谈不上取号之后被落点拒。
///
/// 搭法：满环（txg 4–27，最旧 B (1, 4)，最新 (2, 27)）之后进程退出，盘 1 上最新那一版记账树根那一份改坏一字节（写行那次要换下它）；
/// 普通可写挂载：所选根是最新根 (2, 27)，写行 txg 28 读盘核出盘 1 那一份对不上、两份的记录都留在已分配（D19 已定项 5）；
/// 挂载做成（取号 3），环里 txg 27 那条根还在；这一串换下的（释放代 ≥ 28）在挂载交回的账里全是已释放、还在 defer 里，一个都没被这一串回收；
/// 这一串取的落点没有一个是被隔离的那一槽。池级 checker：坏的那一份仍被环里的 txg 27 引用，I-2.1 / I-4.8 / I-7.4 照判红
/// （改坏盘本身的投影），I-3.1 / I-3.11 成立（隔离的那一份核出坏的，按单元豁免）。
///
/// 所选根不是最新根的那一形（崩溃恢复落到环里最旧的根、更新的根槽都读不出、它们的记录验不过）这一串会盖掉所选根、回收这一串自己换下的，
/// `mount.rs` 那段注释说的分叉在那一形上走得到；取号之后被落点拒要池子紧到只剩那一槽合政策，实四乙在空间准入照判的小盘上扫过没扫出来（报告里写范围）。
pub fn writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, _) =
        build_a_full_ring_whose_oldest_root_is_the_second_version("step-four-c542-newest-root");
    let newest = pool.output.clone();
    assert_eq!(newest.root.checkpoint_txg, CheckpointTxg(27));
    // 写行那次要换下、释放之前读盘核的一个单元：最新那一版的记账树根（经映射寻址，释放前核它每一份；树表与映射树自举豁免，不核）。
    let accounting_root_slot = newest.unit(TransactionUnit::AccountingTree).slot;
    let mut devices = pool.reopen_recorded();
    {
        let (_, device_one) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == DeviceIdentity(1))
            .expect("盘 1");
        let offset = accounting_root_slot.to_device_offset();
        let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
        device_one
            .read_at(offset, &mut bytes)
            .expect("读盘 1 那一份记账树根");
        bytes[100] ^= 0xff;
        device_one
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏盘 1 那一份记账树根");
    }
    let mounted = mount_writable(&parameters(), &mut devices)
        .expect("所选根是最新根：写行那次核出一份坏的、隔离它，挂载照常做成");
    pool.devices = Some(devices);
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.row_publish.root().checkpoint_txg
        ),
        (
            InstanceGeneration(3),
            InstanceGeneration(2),
            CheckpointTxg(27),
            CheckpointTxg(28)
        ),
        "所选根是环里最新的 (2, 27)；写行 txg 28"
    );
    let chain: Vec<&TransactionOutput> = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|publish| {
            publish
                .file_version()
                .expect("所选那一版带文件，这一串每一次都带文件")
        })
        .collect();
    let quarantined: Vec<(DeviceIdentity, u64)> = chain
        .iter()
        .flat_map(|version| {
            version
                .quarantined_after_release_checksum_mismatch
                .iter()
                .map(|copy| (copy.device, copy.placement.slot.0))
        })
        .collect();
    assert_eq!(
        quarantined,
        vec![
            (DeviceIdentity(0), accounting_root_slot.0),
            (DeviceIdentity(1), accounting_root_slot.0)
        ],
        "写行那次读盘核出盘 1 那一份记账树根对不上：两块盘上的记录一起留在已分配"
    );
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let ring_txgs: BTreeSet<u64> = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg.0)
    .collect();
    assert!(
        ring_txgs.contains(&27),
        "这一串写完，所选的最新根 txg 27 还在环里（这一串只盖最旧的几槽）：{ring_txgs:?}"
    );
    // 写行那次换下的：最新那一版里、写行那次重写了的角色原来住的槽（释放代 28，这一串自己换下的）。
    let row_publish = chain[0];
    let replaced_by_the_row_publish: Vec<SlotNumber> = row_publish
        .rewritten
        .iter()
        .filter_map(|role| newest.units.iter().find(|unit| unit.identity == *role))
        .map(|unit| unit.slot)
        .collect();
    assert!(
        !replaced_by_the_row_publish.is_empty(),
        "写行那次重写了最新那一版的固定点"
    );
    let taken_by_the_chain: BTreeSet<SlotNumber> = chain
        .iter()
        .flat_map(|version| {
            version
                .rewritten
                .iter()
                .map(|role| version.unit(*role).slot)
        })
        .collect();
    for slot in &replaced_by_the_row_publish {
        assert!(
            !taken_by_the_chain.contains(slot),
            "写行那次换下的槽 {slot:?} 没被这一串再发出去"
        );
        for device_map in &mounted.allocator.devices {
            assert!(
                !device_map.is_free(*slot),
                "盘 {}：写行那次换下的槽 {slot:?} 挂载之后还不是空闲槽（在 defer 里或隔离着）：环里最旧的有效根没越过 txg 28，这一串回收不了它",
                device_map.device.0
            );
        }
    }
    for version in &chain {
        assert!(
            version
                .rewritten
                .iter()
                .all(|role| version.unit(*role).slot != accounting_root_slot),
            "这一串取的落点没有一个是被隔离的那一槽"
        );
    }
    // 盘 1 那一份记账树根坏在盘上、仍被环里的 txg 27 引用：只红这一份坏单元的投影；I-3.1、I-3.11 成立（隔离的那一份核出坏的，按单元豁免）
    images.hand_out(
        "普通可写挂载写行核出盘 1 那一份记账树根坏了之后",
        || image,
        PoolCheckerExpectation::exactly_these_violated(&["I-2.1", "I-4.8", "I-7.4"])
            .and_holds("I-3.1")
            .and_holds("I-3.11"),
    );
}

/// 实三交回 Q2 立的新欠账（满环时根槽写坏的恢复前镜像，checker 判 I-3.1 红）查原因：与回退无关，普通覆盖写同一格逐字相同；
/// 红的量正好是被盖掉的那条根独占的那几个单元。
///
/// 满环（txg 4–27，最旧 B (1, 4)）之后普通覆盖写 D（txg 28），D 的根槽就是 B 住的那一槽；那一次根槽写把整块写成 0、报错，进程在重发之前崩了。
/// 恢复之前的镜像：B 的根槽没了，D 的记录已持久（根槽写之前落盘）但还没被哪次恢复施加，环里最新的是 C（txg 27）。checker 只判 I-3.1 红，
/// 每块盘「记账的已分配 − 遍历全部有效根」= 只被 B 引用的那几个单元的字节数（229 376 = 14 槽）；这几个单元在 C 的账里是已释放、释放代 5
/// （txg 5 那次换下的），回收要环里最旧的有效根 ≥ 5——B 还在环里时回收不了，C 的账照算它们；B 的根槽被 D 那一次写坏之后遍历里没有 B，
/// 而 checker 只把「已被某次恢复施加」的记录并进遍历（`walk.rs` 的 `versions_applied_only_by_records`），D 不在其内，用的仍是 C 的账。
/// D 那一版的账（D 盖掉 B 的槽之后环里最旧的有效根是 5，释放代 5 的已回收）与这个环对得上。恢复（由 D 的记录重建 D）之后重开：
/// 挂载时 B 那一槽读回全零，钉在它可能住过的最旧 txg 4（用户 2026-10-05 认下的代价），只许 I-3.1 红在记账多的一边、差 917504；
/// 这个进程写掉那一槽（txg 52）之后全绿。
/// checker 在这一格上该不该判红、要不要把「根槽写坏、记录已持久没施加」那一版并进来，是 checker 判法的事，交主 agent。
pub fn plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, _) = build_a_full_ring_whose_oldest_root_is_the_second_version(
        "step-four-pre-recovery-allocated-statistic",
    );
    let image_before_the_overwrite = pool.memory_pool();
    let system_configuration =
        choose_system_configuration(&image_before_the_overwrite).expect("系统配置");
    let mut ring = readable_roots(
        &image_before_the_overwrite,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    ring.sort_by_key(|root| root.checkpoint_txg);
    assert_eq!(
        ring.iter()
            .map(|root| root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        (4..=27).collect::<Vec<u64>>(),
        "满环：txg 4–27"
    );
    let referenced_by =
        |root: &singlefs_core::root_record::RootRecord| -> BTreeSet<(DeviceIdentity, u64, u16)> {
            allocation_records_under_root(&image_before_the_overwrite, root)
                .expect("环里每条根的账读得出")
                .into_iter()
                .filter(|record| !record.is_released)
                .map(|record| (record.device, record.slot.0, record.span_slots))
                .collect()
        };
    let referenced_by_the_newer_roots: BTreeSet<(DeviceIdentity, u64)> = ring[1..]
        .iter()
        .flat_map(|root| {
            referenced_by(root)
                .into_iter()
                .map(|(device, slot, _)| (device, slot))
        })
        .collect();
    let only_the_oldest_references: Vec<(DeviceIdentity, u64, u16)> = referenced_by(&ring[0])
        .into_iter()
        .filter(|(device, slot, _)| !referenced_by_the_newer_roots.contains(&(*device, *slot)))
        .collect();
    let newest_account =
        allocation_records_under_root(&image_before_the_overwrite, &ring[23]).expect("C 的账");
    for (device, slot, _) in &only_the_oldest_references {
        assert!(
            newest_account.iter().any(|record| record.device == *device
                && record.slot.0 == *slot
                && record.is_released
                && record.generation == CheckpointTxg(5)),
            "只被 B 引用的盘 {} 槽 {slot} 在 C 的账里是已释放、释放代 5",
            device.0
        );
    }

    let (failing_device, failing_offset) = root_slot_position_of(28);
    let devices = pool.devices.take().expect("镜像还开着");
    let mut wrapped: Vec<(DeviceIdentity, FirstWriteAtOneOffsetFails<Recorded>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FirstWriteAtOneOffsetFails {
                    inner: device,
                    failing_offset: if identity == failing_device {
                        failing_offset
                    } else {
                        DeviceOffsetInBytes(u64::MAX)
                    },
                    mode: RootSlotWriteFailure::ZeroesWritten,
                    has_failed: false,
                },
            )
        })
        .collect();
    let previous = pool.output.clone();
    let failed = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content_of(1700, 41),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(2),
        )
    };
    assert!(
        matches!(failed, Err(PublishError::BlockDevice(_))),
        "D 的根槽写报错：{:?}",
        failed.as_ref().err()
    );
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );

    let bytes_only_the_oldest_references_on_the_first_red_device: u64 = only_the_oldest_references
        .iter()
        .filter(|(device, _, _)| *device == DeviceIdentity(0))
        .map(|(_, _, span)| u64::from(*span) * singlefs_format::SLOT_BYTES)
        .sum();
    assert_eq!(
        bytes_only_the_oldest_references_on_the_first_red_device, 229_376,
        "盘 0 上只被 B 引用的那几个单元（14 槽）"
    );
    // 恢复之前的镜像：只有 I-3.1 红；盘 0 记账多于遍历的正好是只被 B 引用的那几个单元
    images.hand_out(
        "D 的根槽写报错、恢复之前",
        || pool.memory_pool(),
        PoolCheckerExpectation::exactly_these_violated(&["I-3.1"])
            .and_if_violated_the_detail_satisfies_given_a_value(
                "I-3.1",
                "记账的已分配比遍历全部有效根得到的恰好多只被 B 引用的那几个单元（14 槽）",
                bytes_only_the_oldest_references_on_the_first_red_device,
                allocated_statistic_counts_these_bytes_more_than_the_walk,
            ),
    );

    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&cold, JournalPolicy::Consult).effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(28))),
        "D 的记录已持久：恢复由记录重建 D"
    );
    drop(cold);
    let mut reopened_devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut reopened_devices).expect("恢复之后可写挂载");
    pool.devices = Some(reopened_devices);
    // B 的根槽读回全零：钉在它可能住过的最旧 txg 4（用户 2026-10-05 认下的代价，第 10c 批读回全零照自证不过钉住），多记账的是盘 0 上
    // 释放代 5–8 的已释放字节（慢档调查报告第 3.1 节：917504 = 6455296 − 5537792）；写掉那一槽（txg 52）之后转绿。
    hand_out_red_only_by_the_allocated_statistic_counting_these_bytes_more_than_the_walk(
        images,
        "恢复之后重开",
        || pool.memory_pool(),
        917_504,
    );
    pool.allocator = remounted.allocator;
    pool.output = remounted
        .current
        .into_file_version()
        .expect("恢复之后重开，现行那一版带文件");
    overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
        images,
        &mut pool,
        52,
        "恢复之后重开",
    );
}

/// 一版记账里 inode 号水位那一统计量的每一行 (代段, 值)，按 key 升序（与盘上那棵记账树同序）。
pub fn inode_number_watermark_rows_of(version: &TransactionOutput) -> Vec<(u64, u64)> {
    version
        .accounting_entries
        .iter()
        .filter(|entry| entry.statistic == singlefs_core::records::STATISTIC_INODE_WATERMARK)
        .map(|entry| (entry.generation.0, entry.value))
        .collect()
}

/// 一版的台阶行 (g, H)（`TransactionOutput::inode_number_watermark_steps`）。
pub fn inode_number_watermark_steps_of(version: &TransactionOutput) -> Vec<(u64, u64)> {
    version
        .inode_number_watermark_steps()
        .iter()
        .map(|step| (step.generation.0, step.value))
        .collect()
}

/// 管理员回退取 inode 号水位只取按 cur 的实例表没被崩溃恢复抛弃的根（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2，用户 2026-10-08 Y1）：
/// `crate::common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes` 那段历史——建过三个 inode、水位 5 的那一版被崩溃恢复抛弃、根读得出、
/// 从盘上重建得出水位 5（搭建里自证），实例 2 再建一个（2 号重发，水位 3），再回退到 A：回退那次发布的水位是
/// max(cur 那一版内存里的 3, 没被抛弃的根的) = 3，不跳到被抛弃那一版的 5。环里读得出的根全都算时（改之前的取法，
/// `crates/mutations.tsv` 里「W3 Y1 Opt2」那一行）是 5，这一条红。台阶行照抄 cur 的：(A, 2)、(重发那次, 3)，回退那次水位没变、不留新台阶；
/// 谱系记录带 H_cur = 3。池级 checker 一条违例都没有，I-9.30（台阶行与水位、记录、谱系对得上） 与 I-3.13 真被判过且成立。
pub fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root(
    images: &mut PoolCheckerImages,
) {
    let RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
        pool,
        first_file_version,
        abandoned,
        created_again,
        rollback,
    } = rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
        "rollback-watermark-skips-a-root-abandoned-by-a-recovery",
    );
    assert_eq!(
        (
            abandoned.inode_number_watermark(),
            created_again.inode_number_watermark()
        ),
        (5, 3),
        "被抛弃那一版水位 5；崩溃恢复落到 A 之后再建一个，2 号重发、水位 3"
    );
    assert_eq!(
        rollback.inode_number_watermark(),
        3,
        "回退那次发布的 inode 号水位 = max(cur 内存里的 3, 没被抛弃的根的)：不取被抛弃那一版的 5"
    );
    assert_eq!(
        inode_number_watermark_steps_of(&rollback),
        vec![
            (first_file_version.root.checkpoint_txg.0, 2),
            (created_again.root.checkpoint_txg.0, 3)
        ],
        "台阶行照抄 cur 的；回退那次水位没变，不留新台阶"
    );
    assert_eq!(
        rollback
            .lineage_containers
            .iter()
            .flat_map(|container| container.contents.records.iter())
            .map(|record| record.inode_number_watermark_of_the_version_rolled_back_from)
            .collect::<Vec<u64>>(),
        vec![3],
        "谱系记录带 H_cur = cur 的水位 3"
    );
    hand_out_without_any_violation(
        images,
        "崩溃恢复抛弃、再建、再回退之后",
        || pool.memory_pool(),
        &["I-9.30", "I-3.13"],
    );
}

/// inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程，W3 的 C2aL 形态，用户 2026-10-08 Y2）：A（第一个文件，txg 3，水位 2）、
/// B（建两个空 inode，txg 4，水位 4）、C（覆盖写，txg 5）、D（建一个，txg 6，水位 5）、挂着的时候回退到 B（E，txg 7）。
/// 建 inode 的发布留一级台阶 (那次的 txg, 那次之后的水位)，就是它写的那一行；别的发布照抄、自己那一行不是台阶，下一次发布不带它；
/// 回退那次照抄 cur（D）的，不取 B 的。逐版钉 inode 号水位那一统计量的每一行 (代段, 值)、台阶与记账行数（15 + 早于这一版的台阶数），
/// 从盘上重建读回的与写出的相同（`recovery::rebuild_version` 认得台阶行：记账行数那一判不把它们算进 3 + 6 × 盘数）。
pub fn inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("inode-number-watermark-steps");
    let first = pool.output.clone();
    let publish_new_inodes_in_process = |pool_to_publish_on: &mut BuiltPool, count: u64| {
        let publish_parameters = parameters();
        let previous = pool_to_publish_on.output.clone();
        let devices = pool_to_publish_on.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let output = publish_new_inodes(
            &mut writer,
            &mut pool_to_publish_on.allocator,
            &previous,
            count,
            FIXED_WRITE_TIME_SECONDS + 120,
            InstanceGeneration(1),
        )
        .expect("建空 inode");
        pool_to_publish_on.output = output.clone();
        output
    };
    let second = publish_new_inodes_in_process(&mut pool, 2);
    let third = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let fourth = publish_new_inodes_in_process(&mut pool, 1);
    roll_back(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: second.root.checkpoint_txg,
        },
    )
    .expect("挂着的时候回退到 B");
    let rollback = pool.output.clone();
    assert_eq!(
        [&first, &second, &third, &fourth, &rollback].map(|version| version.root.checkpoint_txg.0),
        [3, 4, 5, 6, 7],
        "A、B、C、D、E 的 txg"
    );
    assert_eq!(
        [&first, &second, &third, &fourth, &rollback].map(inode_number_watermark_rows_of),
        [
            vec![(3, 2)],
            vec![(3, 2), (4, 4)],
            vec![(3, 2), (4, 4), (5, 4)],
            vec![(3, 2), (4, 4), (6, 5)],
            vec![(3, 2), (4, 4), (6, 5), (7, 5)],
        ],
        "inode 号水位那一统计量的行 (代段, 值)：每一版自己那一行加照抄来的台阶"
    );
    assert_eq!(
        [&first, &second, &third, &fourth, &rollback].map(inode_number_watermark_steps_of),
        [
            vec![(3, 2)],
            vec![(3, 2), (4, 4)],
            vec![(3, 2), (4, 4)],
            vec![(3, 2), (4, 4), (6, 5)],
            vec![(3, 2), (4, 4), (6, 5)],
        ],
        "台阶：建 inode 的发布各留一级（第一个文件那一版从没有水位到 2 也算一级），覆盖写与回退不留"
    );
    assert_eq!(
        [&first, &second, &third, &fourth, &rollback]
            .map(|version| version.accounting_entries.len()),
        [15, 16, 17, 17, 18],
        "记账行数 = 每一版自己的 15 行 + 早于这一版 txg 的台阶数"
    );
    assert_eq!(
        (
            rollback.inode_number_watermark(),
            rollback
                .lineage_containers
                .iter()
                .flat_map(|container| container.contents.records.iter())
                .map(|record| record.inode_number_watermark_of_the_version_rolled_back_from)
                .collect::<Vec<u64>>(),
        ),
        (5, vec![5]),
        "回退到 B（水位 4）那一版的水位与谱系记录的 H_cur 都是 cur（D）的 5"
    );
    let rebuilt = match singlefs_core::recovery::rebuild_version(
        &pool.memory_pool(),
        &rollback.root,
        Some(rollback.record.clone()),
    ) {
        Ok(
            singlefs_core::recovery::RebuiltVersion::WithFile(version)
            | singlefs_core::recovery::RebuiltVersion::WithTreesAndNoFile(version),
        ) => version,
        other => panic!("回退那一版从盘上重建：{other:?}"),
    };
    assert_eq!(
        (
            inode_number_watermark_rows_of(&rebuilt),
            rebuilt.inode_number_watermark()
        ),
        (vec![(3, 2), (4, 4), (6, 5), (7, 5)], 5),
        "从盘上重建读回的台阶行与现行值与写出的相同"
    );
    hand_out_without_any_violation(
        images,
        "回退到 B 之后",
        || pool.memory_pool(),
        &["I-9.30", "I-3.13"],
    );
}
