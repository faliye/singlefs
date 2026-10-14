//! harness 测试文件 `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, parameters, publish_overwrite_in_process, unreadable_root_slot_of,
    with_unreadable_ranges, without_unreadable_ranges, BuiltPool, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    BeforeTheOneReread, InstanceRow, InstanceTableOfTheNewestRootRead, NewerPublishWitness,
    ReadStageSettled, RereadsOfThisMount, RollbackTarget, RootRingOfTheRebuiltAllocatorRead,
    SelectedVersionAgainstTheWitness, ShadowLedger, WitnessedCounterComparison,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::DATA_UNIT_BYTES;

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// B 与 C 两次覆盖写的内容。
pub const SECOND_VERSION_BYTES: usize = 4100;

pub const NEWEST_VERSION_BYTES: usize = 2500;

/// 新池新建文件（A，实例 1，txg 3）之后在同一个进程里覆盖写 B（txg 4）、C（txg 5），每次一条记录：C 的根 FUA 之后系统配置轮换，
/// 写的是 C 那条记录的计数器——系统配置见证了 C。
pub struct PoolWithAWitnessedNewestPublish {
    pub pool: BuiltPool,
    pub second: TransactionOutput,
    pub newest: TransactionOutput,
}

pub fn pool_with_a_witnessed_newest_publish(tag: &str) -> PoolWithAWitnessedNewestPublish {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(SECOND_VERSION_BYTES, 3),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let newest = publish_overwrite_in_process(
        &mut pool,
        &second,
        &content_of(NEWEST_VERSION_BYTES, 11),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("C");
    pool.output = newest.clone();
    assert_eq!(
        (
            second.root.checkpoint_txg,
            second.record.counter,
            newest.root.checkpoint_txg,
            newest.record.counter
        ),
        (CheckpointTxg(4), 4, CheckpointTxg(5), 5),
        "暖机 txg 1、2（jsn 1、2），A txg 3（jsn 3），B、C 各一条记录：txg 4、5，jsn 4、5"
    );
    PoolWithAWitnessedNewestPublish {
        pool,
        second,
        newest,
    }
}

/// C 的根槽（第 0 段）与它两份数据单元：择根看不见 C、C 那条记录施加前验点名单元验不过——恢复落到 B（崩溃恢复抛弃根的造法）。
pub fn root_slot_and_data_units_of(
    publish: &TransactionOutput,
    failing_reads: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    std::iter::once(unreadable_root_slot_of(
        publish.root.checkpoint_txg,
        failing_reads,
    ))
    .chain(
        publish
            .data_pointers
            .iter()
            .flat_map(|pointer| pointer.locations)
            .map(|location| UnreadableRange {
                device: location.device,
                offset_in_bytes: location.slot.to_device_offset().0,
                length_in_bytes: DATA_UNIT_BYTES,
                failing_reads,
            }),
    )
    .collect()
}

pub fn version_of(publish: &TransactionOutput) -> RollbackTarget {
    RollbackTarget {
        instance: publish.root.instance,
        checkpoint_txg: publish.root.checkpoint_txg,
    }
}

/// 读阶段选出 `selected` 那一版、系统配置见证到 `witnessed_journal_counter`、`selected` 那次发布的末条记录读得出时的读数。
pub fn selected_against_its_last_record(
    selected: &TransactionOutput,
    witnessed_journal_counter: u64,
) -> SelectedVersionAgainstTheWitness {
    SelectedVersionAgainstTheWitness {
        selected_version: version_of(selected),
        witness: NewerPublishWitness {
            witnessed_journal_counter,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter: selected.record.counter,
            },
        },
    }
}

/// 同一段历史，C 的根槽与数据单元暂时读不出、撤在重读之前（钩子里撤）：`read_back` 两种造法各走一遍——读报错、读回全 0。
/// 读回全 0 那一种靠读缓存「全 0 的不收」（E158 第 4 次跑登记 5.5 第 1 条）：第一遍读回的全 0 进了缓存，重读就还是全 0。
pub fn mount_with_the_newest_root_unreadable_until_the_one_reread(
    tag: &str,
    read_back: UnreadableRangeReadBack,
    images: &mut PoolCheckerImages,
) {
    let PoolWithAWitnessedNewestPublish {
        mut pool,
        second,
        newest,
    } = pool_with_a_witnessed_newest_publish(tag);
    let unreadable = SharedUnreadableRanges::new(
        root_slot_and_data_units_of(&newest, FailingReadsOfARange::Every),
        read_back,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let lifting = unreadable.clone();
    let mut hook_calls = 0;
    let mut lift_before_the_reread = || {
        hook_calls += 1;
        lifting.lift();
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_before_the_reread),
    )
    .expect("重读读得出：照常可写挂载");
    assert_eq!(
        hook_calls, 1,
        "{read_back:?}：钩子只在读阶段重读之前调了一次"
    );
    assert!(
        unreadable.failed_reads() > 0,
        "{read_back:?}：第一遍真读坏过"
    );
    assert_eq!(
        mounted.output.rereads,
        RereadsOfThisMount {
            read_stage: ReadStageSettled::OnTheOneReread {
                first_read: selected_against_its_last_record(&second, newest.record.counter),
                reread: selected_against_its_last_record(&newest, newest.record.counter),
            },
            instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead::OnTheFirstRead,
            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
        },
        "{read_back:?}：第一遍落到 B、判据为真；重读落到 C、判据为假，往下走用重读那一遍"
    );
    assert_eq!(
        (
            version_of_root(&mounted.output.chosen_root),
            version_of_root(&mounted.output.effective_root)
        ),
        (version_of(&newest), version_of(&newest)),
        "{read_back:?}：择根择的是 C，不是它前一条"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: newest.root.checkpoint_txg,
            applied_transaction_high_water: 0,
        }],
        "{read_back:?}：实例 1 那一行记到 C 的 txg，C 不被抛弃"
    );
    assert_eq!(
        (
            mounted.output.isolated_slots_per_device.clone(),
            mounted.output.abandoned_roots_unreadable
        ),
        (vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)], 0),
        "{read_back:?}：没有被抛弃的根，影子账一槽不隔离"
    );
    pool.devices = Some(without_unreadable_ranges(devices));
    // 池级 checker 一条不红（I-7.4 在内）
    images.hand_out(
        &format!("{read_back:?}：挂载之后"),
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), mounted.current.root().checkpoint_txg),
            content: content_of(NEWEST_VERSION_BYTES, 11),
        },
        "{read_back:?}：冷启动读回 C 的内容"
    );
}

pub fn version_of_root(root: &singlefs_core::root_record::RootRecord) -> RollbackTarget {
    RollbackTarget {
        instance: root.instance,
        checkpoint_txg: root.checkpoint_txg,
    }
}

pub fn a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned(
    images: &mut PoolCheckerImages,
) {
    mount_with_the_newest_root_unreadable_until_the_one_reread(
        "c554-yi-lifted-device-error",
        UnreadableRangeReadBack::DeviceError,
        images,
    );
}

pub fn a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache(
    images: &mut PoolCheckerImages,
) {
    mount_with_the_newest_root_unreadable_until_the_one_reread(
        "c554-yi-lifted-zeros",
        UnreadableRangeReadBack::Zeros,
        images,
    );
}

/// 产品路径（`mount_writable`，重读之前不等）：C 的根槽与数据单元每段只坏第一次读 ⇒ 立即重读就读得出，择根择 C。
pub fn a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path(
    images: &mut PoolCheckerImages,
) {
    let PoolWithAWitnessedNewestPublish {
        mut pool,
        second,
        newest,
    } = pool_with_a_witnessed_newest_publish("c554-yi-fails-only-once");
    let unreadable = SharedUnreadableRanges::new(
        root_slot_and_data_units_of(&newest, FailingReadsOfARange::OnlyTheFirst),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let mounted = mount_writable(&parameters(), &mut devices).expect("立即重读读得出");
    pool.devices = Some(without_unreadable_ranges(devices));
    assert_eq!(
        mounted.output.rereads.read_stage,
        ReadStageSettled::OnTheOneReread {
            first_read: selected_against_its_last_record(&second, newest.record.counter),
            reread: selected_against_its_last_record(&newest, newest.record.counter),
        },
        "第一遍落到 B，立即重读落到 C"
    );
    assert_eq!(
        version_of_root(&mounted.output.effective_root),
        version_of(&newest)
    );
    images.hand_out(
        "择根择 C、挂载之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
