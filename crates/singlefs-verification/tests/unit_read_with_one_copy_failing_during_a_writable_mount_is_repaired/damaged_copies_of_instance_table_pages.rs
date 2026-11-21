//! 实例表页有一份读坏：改动的发布读坏的页在抬回收下限之后由它的批替换，回收下限卡住时被拒的批里的页由下一次改动替换，修复的链读计数、同一批替换，读不出的实例表留在原处、照样搬无关的单元或报进结局，不为空间被拒的批在页再读一次时重试，后台整理的环读与挂载结束时的扫尾计数，系统配置写之前的读、抬回收下限时哪一次读坏都计，挂载结束时读行数、重发的发布、挂载期间哪一次读坏都计。

use super::*;

/// 实例表片读到只坏一份（可写挂载之后才坏）：有活快照时部分写读回只坏一份的单元，这次改动的发布写系统配置槽之前算 F 生效值读到实例表那一份，
/// 进别的读那个数（连同那一批搬留存单元读源那一次，共 2）；另发那一批头一次被空间拒（只供测试的开关）、推抬 F 之后发成，同一批搬那个单元、整条链重写实例表：
/// 行不变、落点换到新槽、两份新写，旧落点两份隔离，现行版本（连同留存给快照的映射条目）不再引用弄坏的那两个落点，放回弄坏的字节之后池级 checker 过。
/// 判别力：`crates/mutations.tsv`「主动修复：实例表不进那一批」那一行。
#[test]
fn an_instance_table_page_read_with_one_copy_damaged_by_the_publish_of_a_change_is_replaced_by_its_batch_after_raising_the_floor(
) {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let data_locations = mapped_locations(&version, &key).expect("在映射里");
    let table_locations = version.root.instance_table.locations;
    for location in [&data_locations[0], &table_locations[0]] {
        assert!(
            version_references_the_location(&version, &pool.devices, location),
            "前提：弄坏之前现行版本引用这个落点：{location:?}"
        );
    }
    let damaged_data_copy =
        damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    let damaged_table_copy =
        damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let rows_before = instance_table_chain_of_root(&memory_pool_of(&pool.devices), &version.root)
        .expect("另一份读得出")
        .records
        .rows;
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    let (mapping_keys, units_replaced, floor_raises) = relocated_of(&published);
    assert_eq!(mapping_keys, std::slice::from_ref(&key));
    assert_eq!(
        units_replaced,
        [TransactionUnit::InstanceTable],
        "那次发布读到实例表那一份，这一批的计划定下之前就在账上"
    );
    assert!(floor_raises >= 1, "被空间拒过、推过抬 F");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "写系统配置槽之前读实例表一次、搬留存单元读源一次"
    );
    let after = current_version_of(&session);
    assert!(
        !version_references_the_location(&after, &pool.devices, &data_locations[0]),
        "现行版本（连同留存给快照的映射条目）不再引用弄坏的那个数据单元落点"
    );
    assert!(
        !version_references_the_location(&after, &pool.devices, &table_locations[0]),
        "现行版本不再引用弄坏的那个实例表落点"
    );
    for (new_location, old_location) in after
        .root
        .instance_table
        .locations
        .iter()
        .zip(&table_locations)
    {
        assert_ne!(
            new_location.slot, old_location.slot,
            "实例表第 0 片换到新槽"
        );
        assert!(
            copy_self_verifies(&pool.devices, new_location, BYTES_OF_A_DATA_UNIT),
            "新槽上那一份新写"
        );
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    assert_eq!(
        instance_table_chain_of_root(&memory_pool_of(&pool.devices), &after.root)
            .expect("读得回")
            .records
            .rows,
        rows_before,
        "行不变"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&next);
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "instance-table-replaced",
        &mut pool.devices,
        &[damaged_data_copy, damaged_table_copy],
    );
}

/// 一个数只坏一份的读者沿 `version` 的链读一遍数到几份：对照，证那条链上还有没有一份坏的。
fn instance_table_reads_with_one_copy_failing(
    devices: &Devices,
    version: &TransactionOutput,
) -> u64 {
    let counting_reader =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
    instance_table_chain_of_root(&counting_reader, &version.root).expect("另一份读得出");
    counting_reader.unit_reads_with_one_copy_failing()
}

/// F 抬不动（非空根不足 4 条，上限就是今天的 F）：改动的发布读到实例表那一份、那一批带着它被空间拒、停在抬不动上；试过的那两组从账上去掉，
/// 下一次改动的发布再读到、再记一次，那一批把实例表整条链换掉（数据单元那一组不再试）。算上限那一读进不进账在
/// [`whichever_read_of_the_instance_table_page_fails_while_raising_the_floor_it_is_counted`]。
#[test]
fn an_instance_table_page_in_a_batch_refused_with_the_floor_stuck_is_replaced_by_the_next_change() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    damage_the_data_unit_and_read_it_through_the_session(&session, &mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    force_one_space_refusal_of_the_batch(&mut session);
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(20))
        .expect("建一个空 inode");
    assert!(
        matches!(
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor { .. }
        ),
        "前提：F 抬不动，停在算上限那一步：{:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "数据单元经会话读一次、改动的发布读实例表一次"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(21))
        .expect("再建一个空 inode");
    let (mapping_keys, units_replaced, _) = relocated_of(&next);
    assert!(mapping_keys.is_empty(), "数据单元那一组试过一次了");
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
    let after = current_version_of(&session);
    assert_ne!(
        after.root.instance_table.locations[0].slot, table_locations[0].slot,
        "实例表换到新槽"
    );
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
}

/// 改动的发布每块盘写系统配置槽之前各读一次实例表（两块盘两次）：头两次都读得好，账上只有这次改动换下的数据单元落点。
/// 第三次读是另发那一批认位置时自己沿链读实例表；从这一次起那一份读坏，只有这一读及时读到。
const READ_OF_THE_INSTANCE_TABLE_PAGE_BY_THE_CHAIN_READ_OF_THE_REPAIR: u64 = 3;

/// 另发那一批认位置时自己沿链读实例表：账上只有一组这次改动换下的数据单元落点时走这一读。读到的那一片进别的读那个数、同一批接着换：
/// 下一次改动之前链上已经没有坏的。判别力：`crates/mutations.tsv`「主动修复：认位置沿链读实例表读到的不进账」「主动修复：认位置读到的片不进那一批」两行。
#[test]
fn the_chain_read_of_the_repair_counts_the_damaged_page_and_the_same_batch_replaces_it() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let data_locations = mapped_locations(&version, &key).expect("在映射里");
    let table_locations = version.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    let reads_of_the_page = reads_of_the_copy(
        &table_locations[0],
        FailingReadsOfARange::FromTheNthOnward(
            READ_OF_THE_INSTANCE_TABLE_PAGE_BY_THE_CHAIN_READ_OF_THE_REPAIR,
        ),
    );
    let mut devices = with_unreadable_ranges(pool.devices, &reads_of_the_page);
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    let (mapping_keys, units_replaced, _) = relocated_of(&published);
    assert!(mapping_keys.is_empty(), "数据单元那次改动已经重写");
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "释放之前读盘核一次、认位置沿链读实例表一次"
    );
    assert!(
        reads_of_the_page.failed_reads() >= 1,
        "前提：真有一次读坏：那一份一共读了 {} 次",
        reads_of_the_page.reads_of_range(0)
    );
    let mut devices = without_unreadable_ranges(devices);
    let after = current_version_of(&session);
    assert_eq!(
        instance_table_reads_with_one_copy_failing(&devices, &after),
        0,
        "同一批换了那一片：链上没有坏的了"
    );
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    let next = session
        .publish_user_change(&mut devices, new_inode_change(4))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 账上有一组这次改动已换下的数据单元落点、实例表两份都读不出：认不出那一组是不是实例表片，实例表那一半跳过；与实例表无关的好 key 照搬，结局报明实例表没换。
/// 判别力：`crates/mutations.tsv`「主动修复：实例表读不出整批不发」「主动修复：结局不报实例表没换」两行。
#[test]
fn an_unreadable_instance_table_leaves_the_table_in_place_and_still_relocates_an_unrelated_unit() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key_of_the_unit_kept =
        mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(0)));
    let key_of_the_unit_rewritten =
        mapping_key_of_role(&version, TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER));
    let locations_of_the_unit_kept =
        mapped_locations(&version, &key_of_the_unit_kept).expect("在映射里");
    let locations_of_the_unit_rewritten =
        mapped_locations(&version, &key_of_the_unit_rewritten).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_kept[0],
        BYTES_OF_A_DATA_UNIT,
    );
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_rewritten[0],
        BYTES_OF_A_DATA_UNIT,
    );
    whole_first_file_read_through_the_session(&session, &pool.devices);
    let table_locations = version.root.instance_table.locations;
    for location in &table_locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_DATA_UNIT);
    }
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            mapping_keys,
            units_exempt_from_the_mapping_replaced,
            instance_table_chain_unreadable,
            ..
        } => {
            assert_eq!(mapping_keys, std::slice::from_ref(&key_of_the_unit_kept));
            assert!(units_exempt_from_the_mapping_replaced.is_empty());
            assert!(
                matches!(
                    instance_table_chain_unreadable,
                    Some(RecoveryFailure::UnitUnreadable { .. })
                ),
                "结局报明实例表没换：{instance_table_chain_unreadable:?}"
            );
        }
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("与实例表无关的好 key 照搬，交回 {other:?}"),
    }
    let after = current_version_of(&session);
    assert_ne!(
        mapped_locations(&after, &key_of_the_unit_kept),
        Some(locations_of_the_unit_kept),
        "好 key 搬了"
    );
    assert_eq!(
        after.root.instance_table.locations, table_locations,
        "实例表没换"
    );
}

/// 同上，一样要发的都没有（账上只有这次改动已换下的那一组）：不发，读不出照报在结局里，不报成不用发。
/// 判别力：`crates/mutations.tsv`「主动修复：实例表读不出、没别的要发时报成不用发」那一行。
#[test]
fn an_unreadable_instance_table_with_nothing_else_to_relocate_is_reported_in_the_outcome() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    whole_first_file_read_through_the_session(&session, &pool.devices);
    for location in &version.root.instance_table.locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_DATA_UNIT);
    }
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写重写弄坏的单元");
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
            refusal,
            ..
        } => assert!(
            matches!(
                **refusal,
                RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(_)
            ),
            "{refusal:?}"
        ),
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("读不出报在结局里，交回 {other:?}"),
    }
}

/// 实例表片那一组不是那一批计划里的源落点（实例表豁免映射），账上清掉它的只有「试过的组」那一支：每一遍经一次被拒的回退读一次实例表，再记一次，下一次改动再试。
/// 判别力：`crates/mutations.tsv`「主动修复：试过的组再读到不再记」那一行。
#[test]
fn a_batch_refused_not_for_space_for_the_instance_table_is_tried_again_once_the_page_is_read_again()
{
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    // 迭代上界是下面这三遍；跨轮不带状态，每一遍都读一次、发一次改动。
    for attempt in 0..3u64 {
        read_the_current_instance_table_through_a_refused_rollback(&mut session, &mut pool.devices);
        assert_eq!(
            session.other_unit_reads_with_one_copy_failing_in_this_mount(),
            attempt + 1,
            "账上清掉之后同一组再读到再记一次"
        );
        let published = session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + attempt))
            .expect("建一个空 inode");
        assert!(
            matches!(
                published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
            ),
            "每一次读过之后的那次改动都再试：{:?}",
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
        );
    }
    assert_eq!(
        current_version_of(&session).root.instance_table.locations,
        table_locations,
        "那一批每次都被拒，实例表还在旧落点上"
    );
}

/// 后台入口的调用方：钟停在起点，前台没有 fsync 在等。
struct CallerWithoutAForegroundFsync;

impl CallerOfTheBackgroundCompaction for CallerWithoutAForegroundFsync {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }
    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 挂载之后头一次叫后台入口读根环与每条有效根的树表、现行那一版的实例表（`mount::rollback_states_of_the_ring`）：读到只坏一份的实例表片进别的读那个数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：后台入口读根环不进账」那一行。
#[test]
fn the_ring_read_of_the_background_compaction_counts_the_damaged_instance_table_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let _call_outcome_is_not_what_this_case_judges = session
        .run_compaction_in_the_background(
            &mut pool.devices,
            CompactionPacing::BY_DEFAULT,
            &CallerWithoutAForegroundFsync,
        )
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 清扫判要删哪几行时读现行那一版的实例表（`instance_table_sweep::judge_the_instance_table_sweep`）：根环每一槽都自证过才走到这一读，所以先把环转满。
/// 判通过之后抹头那一步（`erase_the_heads_and_publish_the_sweep`）还要沿同一条链读一遍，账按位置只算一次，判那一读进不进账就看不出来；
/// 所以让判自己停在这一读之后：现行那一版的分配记录树根两份都坏，判读账读不出、一个头都不抹，这一读是唯一读到坏的那一片的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：清扫判的时候读实例表不进账」那一行。
#[test]
fn the_instance_table_sweep_reading_the_table_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let root_ring_slots = 3 * parameters().geometry.root_ring_slots_per_region.count();
    // 迭代上界是根环的槽数；跨轮不带状态，每一次发一条根。
    for write_time_offset in 0..root_ring_slots {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + write_time_offset))
            .expect("建一个空 inode，转一格根环");
    }
    let version = current_version_of(&session);
    damage_the_copy_at(
        &mut pool.devices,
        &version.root.instance_table.locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    damage_both_copies_of_the_allocation_record_tree_root(&version, &mut pool.devices);
    let parameters_of_the_pool = parameters();
    let outcome_of_the_sweep = sweep_the_instance_table_while_mounted(
        &parameters_of_the_pool,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    );
    assert!(
        matches!(
            outcome_of_the_sweep,
            InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AccountOfARingRootUnreadable { .. }
            )
        ),
        "判停在读账，走不到抹头之前那一读：{outcome_of_the_sweep:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 只数（或按 `failing_reads` 读坏）`location` 那一份的读：从交出去那一刻起数。
fn reads_of_the_copy(
    location: &LocationEntry,
    failing_reads: FailingReadsOfARange,
) -> SharedUnreadableRanges {
    SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: location.device,
            offset_in_bytes: location.slot.to_device_offset().0,
            length_in_bytes: 512,
            failing_reads,
        }],
        UnreadableRangeReadBack::DeviceError,
    )
}

/// 写系统配置槽之前算 F 生效值（`transaction::PoolWriter` 轮换系统配置那一步，每次发布每块盘都走）读最新那条根的实例表：实例表第 0 片弄坏一份，发一次建 inode，
/// 这一读是这次改动里唯一读到那一片的读。进别的读那个数；这次改动另发的那一批整条链重写，旧落点两份隔离；再下一次改动不另发。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：写系统配置槽之前读实例表不进账」「可写挂载别的读：写入口上的账发布之后不转进分配器」两行。
#[test]
fn the_read_before_the_system_configuration_writes_counts_the_damaged_page_and_the_batch_of_that_change_replaces_it(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(10))
        .expect("建一个空 inode");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    let (mapping_keys, units_replaced, _) = relocated_of(&published);
    assert!(mapping_keys.is_empty());
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
    let after = current_version_of(&session);
    assert_ne!(
        after.root.instance_table.locations[0].slot, table_locations[0].slot,
        "实例表换到新槽"
    );
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(11))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 抬 F 那一次的读数（[`floor_raise_with_reads_of_the_instance_table_page`]）。
struct FloorRaiseReadings {
    reads_of_the_page: u64,
    failed_reads: u64,
    other_reads: u64,
    is_raised: bool,
}

/// 抬 F 那一串的写报不报错。
#[derive(Clone, Copy)]
enum WritesOfTheFloorRaise {
    Succeed,
    FailOnEveryDevice,
}

/// 第一个文件 3 个单元、[`CHANGES_BEFORE_THE_FLOOR_RAISE`] 次建 inode 之后抬 F 到上限，现行实例表第 0 片那一份照 `failing_reads` 读坏；`writes` 定抬 F 一开始写报不报错。
fn floor_raise_with_reads_of_the_instance_table_page(
    failing_reads: FailingReadsOfARange,
    writes: WritesOfTheFloorRaise,
) -> FloorRaiseReadings {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    // 迭代上界是这个常量；跨轮不带状态，每一次发一条非空有效根。
    for change in 0..CHANGES_BEFORE_THE_FLOOR_RAISE {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(10 + change))
            .expect("建一个空 inode");
    }
    let page = current_version_of(&session).root.instance_table.locations[0];
    let failing_on_request = pool_failing_on_request(pool.devices);
    failing_on_request.writes_fail.store(
        matches!(writes, WritesOfTheFloorRaise::FailOnEveryDevice),
        Ordering::SeqCst,
    );
    let reads_of_the_page = reads_of_the_copy(&page, failing_reads);
    let mut devices = with_unreadable_ranges(failing_on_request.devices, &reads_of_the_page);
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("现行那一版带文件");
    };
    let raise = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        Arc::make_mut(current),
        ShadowLedger::On,
    );
    FloorRaiseReadings {
        reads_of_the_page: reads_of_the_page.reads_of_range(0),
        failed_reads: reads_of_the_page.failed_reads(),
        other_reads: session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        is_raised: raise.is_ok(),
    }
}

/// 抬 F 那一串读现行实例表的每一读（算上限、读候选集那张表、算 F 生效值、先写系统配置与每次空发布写系统配置槽之前）都进账：哪一读单独读坏一次，别的读那个数都是 1。
/// 先空跑一遍数这一串读了几次，再对每一次各跑一遍。判别力：`crates/mutations.tsv`「可写挂载别的读：准入算上限沿链读实例表不进账」「可写挂载别的读：推抬 F 读实例表读到只坏一份不进账」
/// 「可写挂载别的读：推抬 F 算生效值不进账」三行。
// harness-test-granularity:one-scenario 同一段历史上同一次抬 F，取值只差读坏的是第几读，「每一读都进账」要在同一条用例里数全
#[test]
fn whichever_read_of_the_instance_table_page_fails_while_raising_the_floor_it_is_counted() {
    let dry_run = floor_raise_with_reads_of_the_instance_table_page(
        FailingReadsOfARange::Never,
        WritesOfTheFloorRaise::Succeed,
    );
    assert!(dry_run.is_raised, "前提：抬 F 抬得动");
    assert_eq!(dry_run.other_reads, 0, "一读都没坏就不记");
    assert!(
        dry_run.reads_of_the_page >= 4,
        "前提：这一串读过算上限、候选集那张表、算生效值、写系统配置槽之前：{}",
        dry_run.reads_of_the_page
    );
    // 迭代上界是空跑数到的读次数；跨轮不带状态，每一轮新造一个池。
    for failing_read in 1..=dry_run.reads_of_the_page {
        let readings = floor_raise_with_reads_of_the_instance_table_page(
            FailingReadsOfARange::OnlyTheNth(failing_read),
            WritesOfTheFloorRaise::Succeed,
        );
        assert_eq!(readings.failed_reads, 1, "第 {failing_read} 读读坏了");
        assert!(
            readings.is_raised,
            "第 {failing_read} 读读坏：另一份读得出，抬 F 照样做成"
        );
        assert_eq!(readings.other_reads, 1, "第 {failing_read} 读读坏");
    }
}

/// 先写系统配置那一步写失败、这一串的根一条都不发：写系统配置槽之前那一读同样进账（写入口随错丢掉之前转交）。哪一读单独读坏一次，别的读那个数都是 1。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：抬 F 先写系统配置失败时写入口上的账不转交」那一行。
// harness-test-granularity:one-scenario 同上：取值只差读坏的是第几读
#[test]
fn whichever_read_of_the_instance_table_page_fails_before_a_floor_raise_whose_write_fails_it_is_counted(
) {
    let dry_run = floor_raise_with_reads_of_the_instance_table_page(
        FailingReadsOfARange::Never,
        WritesOfTheFloorRaise::FailOnEveryDevice,
    );
    assert!(!dry_run.is_raised, "前提：写报错，抬 F 做不成");
    assert!(
        dry_run.reads_of_the_page >= 1,
        "前提：写之前读过实例表：{}",
        dry_run.reads_of_the_page
    );
    // 迭代上界是空跑数到的读次数；跨轮不带状态，每一轮新造一个池。
    for failing_read in 1..=dry_run.reads_of_the_page {
        let readings = floor_raise_with_reads_of_the_instance_table_page(
            FailingReadsOfARange::OnlyTheNth(failing_read),
            WritesOfTheFloorRaise::FailOnEveryDevice,
        );
        assert_eq!(readings.failed_reads, 1, "第 {failing_read} 读读坏了");
        assert_eq!(readings.other_reads, 1, "第 {failing_read} 读读坏");
    }
}

/// 可写挂载末尾那一判数实例表的行（`instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount`）：阈值之下不扫，这一读是唯一读到那一片的读。
/// 直接对会话调它：挂载的那一次在写行之后读新表，盘上还没有坏的。判别力：`crates/mutations.tsv`「可写挂载别的读：挂载末尾数实例表行数不进账」那一行。
#[test]
fn the_read_of_the_row_count_at_the_end_of_the_mount_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let outcome = sweep_the_instance_table_at_the_end_of_the_mount(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        InstanceTableSweepConfiguration {
            switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
            row_count_threshold: InstanceTableRowCountThreshold(u64::MAX),
            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
        },
    );
    assert!(
        matches!(
            outcome,
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
        ),
        "阈值之下不扫：{outcome:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 落盘中途写失败、这次发布冻结，轮换系统配置那一步没走到；原样重发时写系统配置槽之前那一读读到实例表一份读坏：重发那个写入口上的读进别的读那个数，下一次改动发成之后另发的那一批换掉那一片。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：重发冻结的发布时写入口上的账不转交」那一行。
#[test]
fn the_read_before_the_system_configuration_writes_of_a_resent_publish_counts_the_page_that_fails()
{
    let pool = pool_holding(&content_of_units(3));
    let mut failing = pool_failing_on_request(pool.devices);
    let mounted = mount_writable(&parameters(), &mut failing.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let page = current_version_of(&session).root.instance_table.locations[0];
    failing.writes_fail.store(true, Ordering::SeqCst);
    let refused = session.publish_user_change(&mut failing.devices, new_inode_change(3));
    assert!(
        matches!(refused, Err(UserChangeRefused::Publish { .. })),
        "落盘中途写失败：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "前提：头一个写就失败，走不到写系统配置槽之前那一读"
    );
    failing.writes_fail.store(false, Ordering::SeqCst);
    *failing
        .copy_whose_reads_fail
        .lock()
        .expect("用例单线程改这个开关，锁没有中毒") =
        Some((page.device, page.slot.to_device_offset().0));
    assert!(
        session
            .resend_the_frozen_publish(&mut failing.devices)
            .expect("写不再失败，重发得成"),
        "有冻结着的那一次"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    *failing
        .copy_whose_reads_fail
        .lock()
        .expect("用例单线程改这个开关，锁没有中毒") = None;
    let next = session
        .publish_user_change(&mut failing.devices, new_inode_change(4))
        .expect("建一个空 inode");
    let (_, units_replaced, _) = relocated_of(&next);
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
}

/// 一次可写挂载里读 `location` 那一份的读数：那一份一共读了几次、读坏几次，挂载那个数与别的读那个数。
struct ReadingsOfAMount {
    reads_of_the_copy: u64,
    failed_reads: u64,
    unit_reads_while_mounting: u64,
    other_unit_reads: u64,
}

/// 在 `devices` 的一份拷贝上可写挂载一次，`location` 那一份照 `failing_reads` 读坏（报块设备错，另一份读得出）。
fn writable_mount_with_reads_of_the_copy(
    devices: &Devices,
    location: &LocationEntry,
    failing_reads: FailingReadsOfARange,
) -> ReadingsOfAMount {
    let reads = reads_of_the_copy(location, failing_reads);
    let mut devices_with_the_reads = with_unreadable_ranges(copy_of_the_devices(devices), &reads);
    let mounted = mount_writable(&parameters(), &mut devices_with_the_reads)
        .expect("另一份读得出，可写挂载做得成");
    let (_, session) = MountedSession::of_the_mount(mounted);
    ReadingsOfAMount {
        reads_of_the_copy: reads.reads_of_range(0),
        failed_reads: reads.failed_reads(),
        unit_reads_while_mounting: session.unit_reads_with_one_copy_failing_while_mounting(),
        other_unit_reads: session.other_unit_reads_with_one_copy_failing_in_this_mount(),
    }
}

/// 先不读坏挂一遍数这一次挂载读了那一份几次，再对每一次各挂一遍：哪一读单独读坏一次，挂载那个数与别的读那个数合起来都是 1。
fn assert_every_read_of_the_copy_during_a_writable_mount_is_counted(
    devices: &Devices,
    location: &LocationEntry,
) {
    let dry_run =
        writable_mount_with_reads_of_the_copy(devices, location, FailingReadsOfARange::Never);
    assert_eq!(
        dry_run.unit_reads_while_mounting + dry_run.other_unit_reads,
        0,
        "一读都没坏就不记"
    );
    assert!(dry_run.reads_of_the_copy >= 1, "前提：挂载读过那一份");
    // 迭代上界是空跑数到的读次数；跨轮不带状态，每一轮在一份新拷贝上挂载。
    for failing_read in 1..=dry_run.reads_of_the_copy {
        let readings = writable_mount_with_reads_of_the_copy(
            devices,
            location,
            FailingReadsOfARange::OnlyTheNth(failing_read),
        );
        assert_eq!(readings.failed_reads, 1, "第 {failing_read} 读读坏了");
        assert_eq!(
            readings.unit_reads_while_mounting + readings.other_unit_reads,
            1,
            "第 {failing_read} 读读坏（一共读 {} 次）：挂载那个数 {}、别的读那个数 {}",
            dry_run.reads_of_the_copy,
            readings.unit_reads_while_mounting,
            readings.other_unit_reads
        );
    }
}

/// mkfs 之后头一次可写挂载（树表 0 条那一版，要写的行为空，写行那次是零单元发布）：哪一读单独读坏一次都记 1。这一版独有的一读是取号之前取单元清单（`mount::PreviousVersion::units`，带文件的一版用重建出来的、不读盘）：
/// 它是这次挂载共用的那一份第一次读实例表那一片，之后取号写、零单元发布与暖机写系统配置槽之前读的都取记下的那一份，读不到盘，读到盘的只剩挂载末尾数行数那一读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：取号之前取树表 0 条那一版的单元清单不进账」那一行。
// harness-test-granularity:one-scenario 同一个池上同一次挂载，取值只差读坏的是第几读，「每一读都进账」要在同一条用例里数全
#[test]
fn whichever_read_of_the_instance_table_page_fails_during_the_mount_of_a_version_without_a_file_it_is_counted(
) {
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    assert_every_read_of_the_copy_during_a_writable_mount_is_counted(
        &devices,
        &genesis.root.instance_table.locations[0],
    );
}

/// 带文件那一版的可写挂载：重建分配器开头读最新那条根的实例表（`mount::instance_table_of_the_newest_root_read_at_most_twice`）与之后读这张表的每一读都进账；
/// 建上一版读好了、只有这一读读坏时归别的读那个数。判别力：`crates/mutations.tsv`「可写挂载别的读：重建分配器读最新那条根的实例表不进账」那一行。
// harness-test-granularity:one-scenario 同上：取值只差读坏的是第几读
#[test]
fn whichever_read_of_the_instance_table_page_fails_during_the_mount_of_a_version_with_a_file_it_is_counted(
) {
    let pool = pool_holding(&content_of_units(3));
    assert_every_read_of_the_copy_during_a_writable_mount_is_counted(
        &pool.devices,
        &pool.version.root.instance_table.locations[0],
    );
}
