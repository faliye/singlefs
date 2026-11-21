//! 树表与账有一份读坏：树表由下一次改动替换、旧的几份隔离，准入上限读树表时计数，抬回收下限读现行版本的账时计数、下一次改动搬走那片叶，挂载读另一个候选根的账时计数、什么都不搬，建上一版读过的那一份读账时不再计。

use super::*;

/// 树表读到只坏一份（挂载之后才坏、经会话读文件读到）：计 1；树表每次发布都换下，下一次改动自己就把它换了落点、两份新写，旧落点两份照释放之前读盘核隔离，
/// 另发的那一批没事可做，现行版本不再引用弄坏的那个落点；放回弄坏的字节之后池级 checker 过。判别力：`crates/mutations.tsv`「主动修复：记下位置的豁免单元不走释放之前读盘核」那一行。
#[test]
fn a_tree_table_read_with_one_copy_damaged_is_replaced_by_the_next_change_and_its_old_copies_are_quarantined(
) {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let table_locations = version.root.tree_table.locations;
    assert!(
        version_references_the_location(&version, &pool.devices, &table_locations[0]),
        "前提：弄坏之前现行版本引用这个落点"
    );
    let damaged_copy = damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_NODE);
    let read = whole_first_file_read_through_the_session(&session, &pool.devices);
    assert_eq!(read.bytes, content);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "树表那一份进数"
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&published);
    let after = current_version_of(&session);
    for old_location in &table_locations {
        let old_record = after
            .allocation_records
            .iter()
            .find(|record| record.device == old_location.device && record.slot == old_location.slot)
            .expect("释放之前读盘核隔离的旧落点留在分配记录里");
        assert_eq!(
            old_record.quarantine_mark,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            "旧落点带隔离记号：{old_location:?}"
        );
    }
    for (new_location, old_location) in after.root.tree_table.locations.iter().zip(&table_locations)
    {
        assert_ne!(new_location.slot, old_location.slot, "树表换到新槽");
        assert!(
            copy_self_verifies(&pool.devices, new_location, BYTES_OF_A_NODE),
            "新槽上那一份新写"
        );
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::TreeTable)),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "tree-table-replaced",
        &mut pool.devices,
        &[damaged_copy],
    );
}

/// 算上限读每条有效根的树表（`mount::admission_ceilings_not_passing_ring_slots_bad_since_the_mount`）：改动之前那一版的树表弄坏一份，只有这一读读到，进别的读那个数。
/// 改动已经把现行树表换了，这一组到下一次改动认不出、不另发。判别力：`crates/mutations.tsv`「可写挂载别的读：准入算上限读树表不进账」那一行。
#[test]
fn the_tree_table_read_for_the_admission_ceiling_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    damage_the_data_unit_and_read_it_through_the_session(&session, &mut pool.devices);
    let tree_table_locations = current_version_of(&session).root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &tree_table_locations[0], BYTES_OF_A_NODE);
    force_one_space_refusal_of_the_batch(&mut session);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(20))
        .expect("建一个空 inode");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "算上限读树表读到的那一份进数"
    );
}

/// 抬 F 那一次跑下来的盘、会话与抬之前后的那两版（[`run_with_a_floor_raise`]）。
struct FloorRaiseRun {
    pool: Pool,
    session: MountedSession,
    before: TransactionOutput,
    after: TransactionOutput,
}

/// 400 个单元的文件（分配记录树多片叶）、[`CHANGES_BEFORE_THE_FLOOR_RAISE`] 次建 inode 之后抬 F 到上限，`damaged` 给了就在抬 F 之前弄坏那个分配记录树节点的头一份。
fn run_with_a_floor_raise(damaged: Option<AllocationRecordTreeNode>) -> FloorRaiseRun {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    // 迭代上界是这个常量；跨轮不带状态，每一次发一条非空有效根。
    for change in 0..CHANGES_BEFORE_THE_FLOOR_RAISE {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(10 + change))
            .expect("建一个空 inode");
    }
    let before = current_version_of(&session);
    if let Some(node) = damaged {
        let location = before
            .allocation_record_tree
            .pointer_of(node)
            .expect("节点在树里")
            .locations[0];
        damage_the_copy_at(&mut pool.devices, &location, BYTES_OF_A_NODE);
    }
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("现行那一版带文件");
    };
    let _raised_floor = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        Arc::make_mut(current),
        ShadowLedger::On,
    )
    .expect("抬 F 抬得动");
    let after = current_version_of(&session);
    FloorRaiseRun {
        pool,
        session,
        before,
        after,
    }
}

/// 抬 F 重算影子账读每条候选根的账（`mount::isolate_slots_referenced_only_by_abandoned_roots`），现行那一版也在候选里，读的是现行那一版的分配记录树：
/// 取一片抬 F 那一串没动的叶，弄坏头一份。这一读读到，进别的读那个数；下一次改动发成之后现行版本不再引用弄坏的那个落点。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：抬 F 重算影子账读各根的账不进账」那一行。
#[test]
fn the_floor_raise_reading_the_account_of_the_current_version_counts_the_damaged_leaf_and_the_next_change_moves_it(
) {
    let dry_run = run_with_a_floor_raise(None);
    let leaves_the_raise_does_not_touch: BTreeSet<AllocationRecordTreeNode> = dry_run
        .before
        .allocation_record_tree
        .nodes
        .iter()
        .filter(|(node, pointer)| {
            dry_run.after.allocation_record_tree.pointer_of(*node) == Some(*pointer)
        })
        .map(|(node, _)| *node)
        .filter(|node| {
            matches!(
                node,
                AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
                    level: 0,
                    ..
                })
            )
        })
        .collect();
    let node = *leaves_the_raise_does_not_touch
        .iter()
        .next()
        .expect("前提：抬 F 那一串有没动的叶");
    let mut run = run_with_a_floor_raise(Some(node));
    let location = run
        .before
        .allocation_record_tree
        .pointer_of(node)
        .expect("节点在树里")
        .locations[0];
    assert_eq!(
        run.session
            .other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    run.session
        .publish_user_change(&mut run.pool.devices, new_inode_change(40))
        .expect("再建一个空 inode");
    assert!(
        !version_references_the_location(
            &current_version_of(&run.session),
            &run.pool.devices,
            &location
        ),
        "现行版本不再引用弄坏的那个落点"
    );
    assert_ne!(
        current_version_of(&run.session)
            .allocation_record_tree
            .pointer_of(node),
        run.before.allocation_record_tree.pointer_of(node),
        "那片叶换了落点"
    );
}

/// 可写挂载重建分配器读每条候选根的账：只有更早那一版引用的分配记录树根弄坏一份，只有这一读读到。进别的读那个数；读的是别的版本，下一次改动发成之后那一批认不出、不搬。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：重建分配器读各根的账不进账」那一行。
#[test]
fn the_mount_reading_the_account_of_another_candidate_root_counts_the_damaged_copy_and_nothing_moves(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let (earlier, current) = {
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3))
            .expect("建一个空 inode");
        let earlier = current_version_of(&session);
        session
            .publish_user_change(&mut pool.devices, new_inode_change(4))
            .expect("再建一个空 inode");
        (earlier, current_version_of(&session))
    };
    let key = mapping_key_of_role(&earlier, TransactionUnit::AllocationTree);
    assert_eq!(
        mapped_locations(&current, &key),
        None,
        "前提：现行那一版不再引用更早那一版的分配记录树根"
    );
    let locations = mapped_locations(&earlier, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    assert_eq!(session.unit_reads_with_one_copy_failing_while_mounting(), 0);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(5))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&published);
}

/// 可写挂载建上一版读到的那一份（现行树表），重建分配器读候选根的账又读到一遍：归挂载那个数，别的读那个数不加。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：重建分配器读到建上一版已读到的那一份又进别的读」那一行。
#[test]
fn a_copy_read_by_building_the_previous_version_is_not_counted_again_by_the_read_of_the_accounts() {
    let mut pool = pool_holding(&content_of_units(3));
    let tree_table_locations = pool.version.root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &tree_table_locations[0], BYTES_OF_A_NODE);
    let session = session_of_a_writable_mount(&mut pool.devices);
    assert_eq!(session.unit_reads_with_one_copy_failing_while_mounting(), 1);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0
    );
}
