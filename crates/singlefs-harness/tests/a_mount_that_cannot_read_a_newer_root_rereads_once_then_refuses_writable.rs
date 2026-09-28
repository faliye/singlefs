//! C554 乙（用户 2026-09-27 定：重读后再判，R = 1；岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）：可写挂载读到的样子里
//! 有一样更新的东西读不出，就重读一次（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」）；重读读得出就照常挂载——
//! 用读出的那条根择根，不把它当成被抛弃；仍读不出就拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`，取号之前、
//! 盘上逐字节不变），只读挂载照常。「更新的东西」两样：
//!
//! - 系统配置见证过一次比所选那一版新的发布（判据 N-配置，E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md` 第 347 行）：
//!   系统配置在一次发布的根槽 FUA 之后才轮换，写这次发布末条记录的计数器（D16（发布语义） 已定项 7），见证到的发布它的根落过盘；
//! - 重建影子账时最新那条根的实例表（代码审阅第 22 条：改之前读不出就按「没有一条根被抛弃」往下走，无声关掉隔离）。
//!
//! 读故障由 `common::SharedUnreadableRanges` 造：点名的几段读报块设备错或读回全 0，持续到用例的钩子
//! （`BeforeTheOneReread::CallTheTestOnlyHookFirst`）撤掉为止，或每段只坏第一次读。系统配置没见证到的最新根（它那次轮换没落盘）
//! 暂时读不出时照旧被抛弃，这一格乙-配置罩不到，由 `a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread`
//! 钉住。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, disk_snapshot,
    format_pool, parameters, publish_overwrite_in_process, unreadable_root_slot_of,
    with_unreadable_ranges, without_unreadable_ranges, BuiltPool, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    BeforeTheOneReread, InstanceRow, InstanceTableOfTheNewestRootRead, MountError,
    NewerPublishWitness, ReadStageSettled, RereadsOfThisMount, RollbackTarget,
    SelectedVersionAgainstTheWitness, ShadowLedger, StillUnreadableAfterOneReread,
    WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    instance_table_of_root, recover, root_is_abandoned_by_the_instance_table, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_format::{DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// B 与 C 两次覆盖写的内容。
const SECOND_VERSION_BYTES: usize = 4100;
const NEWEST_VERSION_BYTES: usize = 2500;

/// 新池新建文件（A，实例 1，txg 3）之后在同一个进程里覆盖写 B（txg 4）、C（txg 5），每次一条记录：C 的根 FUA 之后系统配置轮换，
/// 写的是 C 那条记录的计数器——系统配置见证了 C。
struct PoolWithAWitnessedNewestPublish {
    pool: BuiltPool,
    second: TransactionOutput,
    newest: TransactionOutput,
}

fn pool_with_a_witnessed_newest_publish(tag: &str) -> PoolWithAWitnessedNewestPublish {
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
fn root_slot_and_data_units_of(
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

/// 一次发布末条记录的两份（每块盘同一个偏移，D23（journal 的角色与格式） 已定项 14「两份镜像」）。
fn last_record_of(publish: &TransactionOutput) -> Vec<UnreadableRange> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| UnreadableRange {
            device,
            offset_in_bytes: record_offset(
                publish.record.counter,
                parameters().geometry.journal_ring_bytes,
            )
            .0,
            length_in_bytes: JOURNAL_RECORD_BYTES,
            failing_reads: FailingReadsOfARange::Every,
        })
        .collect()
}

fn version_of(publish: &TransactionOutput) -> RollbackTarget {
    RollbackTarget {
        instance: publish.root.instance,
        checkpoint_txg: publish.root.checkpoint_txg,
    }
}

/// 读阶段选出 `selected` 那一版、系统配置见证到 `witnessed_journal_counter`、`selected` 那次发布的末条记录读得出时的读数。
fn selected_against_its_last_record(
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

fn violated_invariants(pool: &BuiltPool) -> Vec<(&'static str, String)> {
    check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

/// 系统配置见证了 C（jsn 5），C 的根槽与数据单元这一次挂载里每次读都报错：读阶段落到 B（jsn 4 < 5，判据为真），在读缓存上重读一遍仍落到 B
/// ⇒ 拒可写，两遍的读数都报出来；C 的根槽正好读了两次（第一遍一次、重读一次，R = 1），B 的根槽只读了一次（重读那一遍读缓存里有它）。
/// 取号之前拒：`DiskSnapshot` 不变。只读挂载在同一组读故障下照常，读回 B 那一版。
#[test]
fn a_witnessed_newest_root_still_unreadable_after_the_one_reread_refuses_the_writable_mount_before_any_write_while_the_read_only_mount_reads_the_version_before_it(
) {
    let PoolWithAWitnessedNewestPublish {
        mut pool,
        second,
        newest,
    } = pool_with_a_witnessed_newest_publish("c554-yi-still-unreadable");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut ranges = root_slot_and_data_units_of(&newest, FailingReadsOfARange::Every);
    let index_of_the_second_root_slot = ranges.len();
    ranges.push(unreadable_root_slot_of(
        second.root.checkpoint_txg,
        FailingReadsOfARange::Never,
    ));
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let refused = mount_writable(&parameters(), &mut devices);
    let expected_reading = selected_against_its_last_record(&second, newest.record.counter);
    assert!(
        matches!(
            &refused,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                if **still_unreadable
                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                        first_read: expected_reading,
                        reread: expected_reading,
                    }
        ),
        "两遍都落到 B、系统配置见证到 C：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        unreadable.reads_of_range(0),
        2,
        "C 的根槽：第一遍读一次、重读那一遍再读一次（R = 1），之后不再读"
    );
    assert_eq!(
        unreadable.reads_of_range(index_of_the_second_root_slot),
        1,
        "B 的根槽第一遍读成、进了读缓存：重读那一遍不再打到盘上"
    );

    let read_only = mount_read_only(&devices).expect("同一组读故障下只读挂载照常");
    assert_eq!(
        (
            read_only.effective_root.instance,
            read_only.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), second.root.checkpoint_txg),
        "只读挂载落到 B"
    );
    let file = read_only
        .mounted
        .open_file(&devices, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    let read_back = file
        .read_at(
            &devices,
            FileOffsetInBytes(0),
            u64::try_from(SECOND_VERSION_BYTES).expect("4100"),
        )
        .expect("读回 B");
    assert_eq!(read_back.bytes, content_of(SECOND_VERSION_BYTES, 3));
    pool.devices = Some(without_unreadable_ranges(devices));
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "取号之前拒：一个写、一道屏障都没发"
    );
}

/// 同一段历史，C 的根槽与数据单元暂时读不出、撤在重读之前（钩子里撤）：`read_back` 两种造法各走一遍——读报错、读回全 0。
/// 读回全 0 那一种靠读缓存「全 0 的不收」（E158 第 4 次跑登记 5.5 第 1 条）：第一遍读回的全 0 进了缓存，重读就还是全 0。
fn mount_with_the_newest_root_unreadable_until_the_one_reread(
    tag: &str,
    read_back: UnreadableRangeReadBack,
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
    assert_eq!(
        violated_invariants(&pool),
        Vec::new(),
        "{read_back:?}：池级 checker 一条不红（I-7.4 在内）"
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

fn version_of_root(root: &singlefs_core::root_record::RootRecord) -> RollbackTarget {
    RollbackTarget {
        instance: root.instance,
        checkpoint_txg: root.checkpoint_txg,
    }
}

#[test]
fn a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned(
) {
    mount_with_the_newest_root_unreadable_until_the_one_reread(
        "c554-yi-lifted-device-error",
        UnreadableRangeReadBack::DeviceError,
    );
}

#[test]
fn a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache(
) {
    mount_with_the_newest_root_unreadable_until_the_one_reread(
        "c554-yi-lifted-zeros",
        UnreadableRangeReadBack::Zeros,
    );
}

/// 产品路径（`mount_writable`，重读之前不等）：C 的根槽与数据单元每段只坏第一次读 ⇒ 立即重读就读得出，择根择 C。
#[test]
fn a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path(
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
    assert_eq!(violated_invariants(&pool), Vec::new());
}

/// 乙-配置罩不到的那一格：C 那次发布之后轮换的系统配置槽坏了、系统配置只见证到 B（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`），
/// C 的根槽与数据单元暂时读不出 ⇒ 判据为假（jsn 4 = 4），不重读，照旧落到 B、C 按新实例的表判被抛弃。
#[test]
fn a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread() {
    let PoolWithAWitnessedNewestPublish {
        mut pool,
        second,
        newest,
    } = pool_with_a_witnessed_newest_publish("c554-yi-never-witnessed");
    let mounted =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &newest);
    assert_eq!(
        mounted.output.rereads.read_stage,
        ReadStageSettled::OnTheFirstRead {
            first_read: selected_against_its_last_record(&second, second.record.counter),
        },
        "系统配置里读得出的只剩见证 B 的那一槽：c_见证 = 4 = B 的末条，判据为假，不重读"
    );
    assert_eq!(
        version_of_root(&mounted.output.effective_root),
        version_of(&second)
    );
    let table = instance_table_of_root(&pool.memory_pool(), mounted.current.root())
        .expect("新实例的实例表读得出");
    assert!(
        root_is_abandoned_by_the_instance_table(&newest.root, &table),
        "C 按新实例写的那一行被抛弃：{table:?}"
    );
}

/// 判据 N-配置 另两支（E158 第 3 次跑登记第 347 行）：B 那次发布的末条记录两份也读不出 ⇒ 拿计数器等于 c_见证 的那条记录比——
/// C 的记录读得出，它的 (实例, txg) 比 B 新，判真；C 的记录也读不出 ⇒ 判不出，按真。两格读故障都持续，重读仍判真、拒可写。
#[test]
fn without_the_last_record_of_the_selected_version_the_record_at_the_witnessed_counter_decides_and_without_both_the_mount_counts_it_as_witnessed(
) {
    for (tag, hide_the_newest_record, expected_comparison) in [
        (
            "c554-yi-record-at-the-witnessed-counter",
            false,
            WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter {
                record: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(5),
                },
            },
        ),
        (
            "c554-yi-undecidable",
            true,
            WitnessedCounterComparison::Undecidable,
        ),
    ] {
        let PoolWithAWitnessedNewestPublish {
            mut pool,
            second,
            newest,
        } = pool_with_a_witnessed_newest_publish(tag);
        let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
        let mut ranges = root_slot_and_data_units_of(&newest, FailingReadsOfARange::Every);
        ranges.extend(last_record_of(&second));
        if hide_the_newest_record {
            ranges.extend(last_record_of(&newest));
        }
        let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
        let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
        let refused = mount_writable(&parameters(), &mut devices);
        pool.devices = Some(without_unreadable_ranges(devices));
        let expected_reading = SelectedVersionAgainstTheWitness {
            selected_version: version_of(&second),
            witness: NewerPublishWitness {
                witnessed_journal_counter: newest.record.counter,
                comparison: expected_comparison,
            },
        };
        assert!(
            matches!(
                &refused,
                Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                    if **still_unreadable
                        == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                            first_read: expected_reading,
                            reread: expected_reading,
                        }
            ),
            "{tag}：{:?}",
            refused.as_ref().err()
        );
        assert_eq!(
            disk_snapshot(&pool.memory_pool(), &pool.stream),
            before,
            "{tag}：取号之前拒"
        );
    }
}

/// 判据的第一支：只做过 mkfs 的池两块盘的系统配置 tail 都是 0 ⇒ 没有见证，判假、不重读。
#[test]
fn the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread() {
    let mut formatted = format_pool("c554-yi-formatted-pool-nothing-witnessed");
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    formatted.devices = Some(devices);
    assert_eq!(
        mounted.output.rereads,
        RereadsOfThisMount {
            read_stage: ReadStageSettled::OnTheFirstRead {
                first_read: SelectedVersionAgainstTheWitness {
                    selected_version: RollbackTarget {
                        instance: InstanceGeneration(0),
                        checkpoint_txg: CheckpointTxg(0),
                    },
                    witness: NewerPublishWitness {
                        witnessed_journal_counter: 0,
                        comparison: WitnessedCounterComparison::NothingWitnessed,
                    },
                },
            },
            instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead::OnTheFirstRead,
        }
    );
}

/// 代码审阅第 22 条那一格用的历史（与 `unreadable_abandoned_root_slot.rs` 同一段脚本）：
/// A、B（实例 1）→ 重开取号 2、写行（txg 5）、暖机（6、7）→ C（txg 8）→ 崩溃恢复抛弃 C（系统配置没见证 C，实例 3 写行 9、暖机 10）。
/// 交回池、最新那条根（实例 3 暖机 txg 10）指着的实例表那一片的指针。
fn pool_after_a_recovery_abandoned_the_third_version(tag: &str) -> (BuiltPool, NodePointer) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(4100, 3),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let before_the_abandoned_publish = pool.output.clone();
    let abandoned = publish_overwrite_in_process(
        &mut pool,
        &before_the_abandoned_publish,
        &content_of(2500, 11),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(2),
    )
    .expect("C");
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
    let newest_root = *abandoning.current.root();
    assert_eq!(
        (newest_root.instance, newest_root.checkpoint_txg),
        (InstanceGeneration(3), CheckpointTxg(10)),
        "实例 3 写行 txg 9、暖机 txg 10"
    );
    (pool, newest_root.instance_table)
}

/// 最新那条根指着的实例表那一片：盘 0 那一份第 `first_failing_read_on_device_zero` 次读起坏，盘 1 那一份每次都坏（盘 1 那一份只在盘 0 读坏时才读）。
fn instance_table_page_failing_from(
    page: &NodePointer,
    first_failing_read_on_device_zero: u64,
) -> Vec<UnreadableRange> {
    page.locations
        .iter()
        .map(|location| UnreadableRange {
            device: location.device,
            offset_in_bytes: location.slot.to_device_offset().0,
            length_in_bytes: DATA_UNIT_BYTES,
            failing_reads: if location.device == DeviceIdentity(0) {
                FailingReadsOfARange::FromTheNthOnward(first_failing_read_on_device_zero)
            } else {
                FailingReadsOfARange::Every
            },
        })
        .collect()
}

/// 实例 4 那次可写挂载里，重建分配器之前盘 0 那一份实例表读了几次：`rebuild_previous_version` 里 `recovery::rebuild_version` 读实例表那个单元一次、
/// 再沿所选那一版的根读实例表链一次。之后第一次读就是影子账判抛弃那一读（代码审阅第 22 条）：生效 F 判有效根用的也是影子账读的那张表，
/// 不另读一遍（实审 A3b，C554 乙报告 Q6）。数法：「第 n 次起坏」取 n = 1、2 时 `rebuild_previous_version` 报 `InstanceTableMalformed`，
/// n = 3 起走到影子账那一读（这一数跟着读法变，变了就重数）。
const DEVICE_ZERO_READS_OF_THE_NEWEST_INSTANCE_TABLE_BEFORE_THE_ALLOCATOR_IS_REBUILT: u64 = 2;

/// 代码审阅第 22 条：重建分配器时最新那条根的实例表读不出——改之前按「没有一条根被抛弃」往下走，C 独占的 14 槽一个都不隔离、
/// `abandoned_roots_unreadable` 仍 0。现在重读一次：
/// - 持续读不出 ⇒ 拒可写（点名重读那一遍择到的最新根 (3, 10)），取号之前、`DiskSnapshot` 不变；盘 0 那一份在重建分配器里读了两次
///   （影子账第一遍、重读；生效 F 判有效根用影子账那张表、不另读），盘 1 那一份跟着两次；
/// - 撤在重读之前（钩子里撤）⇒ 照常可写挂载，报「实例表在重读那一遍读出来」，影子账照隔离 C 独占的 14 槽（两块盘各 14，
///   与 `unreadable_abandoned_root_slot.rs` 根槽读得出那一臂钉的同一个数）。
#[test]
fn a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_is_reread_once_then_refuses_writable_instead_of_turning_the_isolation_off(
) {
    let (mut pool, page) = pool_after_a_recovery_abandoned_the_third_version(
        "c554-yi-instance-table-stays-unreadable",
    );
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let unreadable = SharedUnreadableRanges::new(
        instance_table_page_failing_from(
            &page,
            DEVICE_ZERO_READS_OF_THE_NEWEST_INSTANCE_TABLE_BEFORE_THE_ALLOCATOR_IS_REBUILT + 1,
        ),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let refused = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(without_unreadable_ranges(devices));
    assert!(
        matches!(
            &refused,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                if **still_unreadable
                    == StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                        newest_root_on_the_reread: Some(RollbackTarget {
                            instance: InstanceGeneration(3),
                            checkpoint_txg: CheckpointTxg(10),
                        }),
                    }
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        (unreadable.reads_of_range(0), unreadable.reads_of_range(1)),
        (
            DEVICE_ZERO_READS_OF_THE_NEWEST_INSTANCE_TABLE_BEFORE_THE_ALLOCATOR_IS_REBUILT + 2,
            2
        ),
        "盘 0 那一份：重建分配器之前那几读 + 影子账第一遍、重读各一次（生效 F 不另读）；盘 1 那一份只在盘 0 读坏时读"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "取号之前拒：一个写、一道屏障都没发"
    );

    let (mut lifted_pool, lifted_page) =
        pool_after_a_recovery_abandoned_the_third_version("c554-yi-instance-table-lifted");
    let lifted = SharedUnreadableRanges::new(
        instance_table_page_failing_from(
            &lifted_page,
            DEVICE_ZERO_READS_OF_THE_NEWEST_INSTANCE_TABLE_BEFORE_THE_ALLOCATOR_IS_REBUILT + 1,
        ),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut lifted_devices = with_unreadable_ranges(lifted_pool.reopen_recorded(), &lifted);
    let lifting = lifted.clone();
    let mut lift_before_the_reread = || lifting.lift();
    let remounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut lifted_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_before_the_reread),
    )
    .expect("重读读得出：照常可写挂载");
    lifted_pool.devices = Some(without_unreadable_ranges(lifted_devices));
    assert_eq!(
        remounted.output.rereads.instance_table_of_the_newest_root,
        InstanceTableOfTheNewestRootRead::OnTheOneReread
    );
    assert_eq!(
        (
            remounted.output.isolated_slots_per_device.clone(),
            remounted.output.abandoned_roots_unreadable
        ),
        (vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)], 0),
        "影子账照隔离 C 独占的 14 槽"
    );
}
