//! 回退读到读坏的一份：回退的释放检查、重建回退目标（之后马上被拒也算）、读现行实例表（之后马上被拒也算）都计数，回退的水位读只看见目标与现行版本之间的根的单元，在复活单元上被拒的回退只为重建目标计一次，读现行账之后被拒的回退计数读坏的根。

use super::*;

/// 两次改动（每次建一个空 inode）之后的会话与头一次改动那一版的根：回退的目标。
fn session_after_two_changes_and_the_rollback_target(
    pool: &mut Pool,
) -> (MountedSession, RollbackTarget, TransactionOutput) {
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let version_of_the_target = current_version_of(&session);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    (
        session,
        RollbackTarget {
            instance: version_of_the_target.root.instance,
            checkpoint_txg: version_of_the_target.root.checkpoint_txg,
        },
        version_of_the_target,
    )
}

/// 回退释放现行那一版换下的单元之前读盘核（只有现行那一版引用的数据单元）：读到只坏一份进别的读那个数。
/// 回退算 inode 号水位还要重建环里每条根（`mount::highest_inode_number_watermark_in_the_ring`），现行那一版引用的单元它都读，账按位置只算一次，读盘核进不进账就看不出来；
/// 所以把现行那一版的 extent 树根两份都弄坏：重建这条根在读数据单元之前就读不出（水位不算这条根），读盘核是唯一读到坏的那一份数据单元的读。两份都坏的 extent 树根不进这个数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退的释放之前读盘核不进账」那一行。
#[test]
fn the_release_check_of_a_rollback_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let version_of_the_target = current_version_of(&session);
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 4),
        )
        .expect("部分写");
    let mut current = current_version_of(&session);
    let data_locations = mapped_locations(
        &current,
        &mapping_key_of_role(&current, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("数据单元在映射里");
    let extent_root_locations = mapped_locations(
        &current,
        &mapping_key_of_role(&current, TransactionUnit::ExtentRoot),
    )
    .expect("extent 树根在映射里");
    for location in [&data_locations[0], &extent_root_locations[0]] {
        assert!(
            !version_references_the_location(&version_of_the_target, &pool.devices, location),
            "前提：只有现行那一版引用这个落点：{location:?}"
        );
    }
    damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    for location in &extent_root_locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_NODE);
    }
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_of_the_target.root.instance,
            checkpoint_txg: version_of_the_target.root.checkpoint_txg,
        },
    )
    .expect("回退到头一次改动那一版");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退重建目标那一版读到只坏一份进别的读那个数。坏的是只有目标那一版引用的记账树根：它不随回退取回（`mount::is_a_unit_taken_from_the_rollback_target`），只有重建读它。
/// 取回的单元（inode 树根这一类）进复活集，坏一份回退就拒（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」，`rollback_by_a_forward_publish` 目标钉着），走不到这个数。
/// 这一条钉回退照样做成、算水位再重建这一版时同一份坏副本不再加次数；重建那一读自己进不进账在下一条用例。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退做成时同一份坏副本再读又加次数（重建目标那一版，账上不记）」那一行。
#[test]
fn rebuilding_the_target_of_a_rollback_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, target, version_of_the_target) =
        session_after_two_changes_and_the_rollback_target(&mut pool);
    let mut current = current_version_of(&session);
    let accounting_root_key =
        mapping_key_of_role(&version_of_the_target, TransactionUnit::AccountingTree);
    assert_eq!(
        mapped_locations(&current, &accounting_root_key),
        None,
        "前提：现行那一版不再引用目标那一版的记账树根，回退的别处读不到它"
    );
    let accounting_root_locations =
        mapped_locations(&version_of_the_target, &accounting_root_key).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &accounting_root_locations[0],
        BYTES_OF_A_NODE,
    );
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .expect("回退到头一次改动那一版");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退重建目标那一版之后接着读现行那一版的账：账读不出（分配记录树根两份都坏，不进这个数）就拒，走不到算水位那一遍重建，重建目标那一版是唯一读到坏的那一份的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退重建目标那一版不进账」那一行。
#[test]
fn rebuilding_the_target_of_a_rollback_refused_right_after_it_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, target, version_of_the_target) =
        session_after_two_changes_and_the_rollback_target(&mut pool);
    let mut current = current_version_of(&session);
    let accounting_root_key =
        mapping_key_of_role(&version_of_the_target, TransactionUnit::AccountingTree);
    assert_eq!(
        mapped_locations(&current, &accounting_root_key),
        None,
        "前提：现行那一版不再引用目标那一版的记账树根"
    );
    let accounting_root_locations =
        mapped_locations(&version_of_the_target, &accounting_root_key).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &accounting_root_locations[0],
        BYTES_OF_A_NODE,
    );
    damage_both_copies_of_the_allocation_record_tree_root(&current, &mut pool.devices);
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::CurrentAccountUnreadable { .. })
        ),
        "重建目标那一版之后读现行那一版的账拒：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退读现行那一版的实例表（挂载写行之后那张，目标那一版在上一次挂载，指的是另一张）：读到只坏一份进别的读那个数；回退照样做成，判候选与算水位再读这一片时不再加次数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退做成时同一份坏副本再读又加次数（读现行实例表，判重比不上）」那一行。
#[test]
fn a_rollback_reading_the_current_instance_table_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let version_of_the_target = {
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3))
            .expect("建一个空 inode");
        current_version_of(&session)
    };
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    let mut current = current_version_of(&session);
    assert_ne!(
        current.root.instance_table.locations[0].slot,
        version_of_the_target.root.instance_table.locations[0].slot,
        "前提：挂载写行换了实例表"
    );
    let table_locations = current.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_of_the_target.root.instance,
            checkpoint_txg: version_of_the_target.root.checkpoint_txg,
        },
    )
    .expect("回退到上一次挂载那一版");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退读过现行那一版的实例表、判候选时目标不在根环里就拒：算水位那一遍重建走不到，这一读是唯一读到坏的那一片的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退读现行实例表与判候选不进账」那一行。
#[test]
fn a_rollback_refused_right_after_reading_the_current_instance_table_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    read_the_current_instance_table_through_a_refused_rollback(&mut session, &mut pool.devices);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退取环里各根的 inode 号水位（`mount::highest_inode_number_watermark_in_the_ring`）：环里一条既不是现行也不是目标的根引用的单元，只有这一读读到。回退照样做成，别的读那个数是 1。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退取水位不进账」那一行。
#[test]
fn only_the_watermark_read_of_a_rollback_sees_a_unit_of_a_root_between_the_target_and_the_current_version(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let target = current_version_of(&session);
    let patch_of_the_middle = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch_of_the_middle, 4),
        )
        .expect("部分写");
    let middle = current_version_of(&session);
    let middle_locations = mapped_locations(
        &middle,
        &mapping_key_of_role(&middle, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    let patch_of_the_current = vec![0xA5u8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch_of_the_current, 5),
        )
        .expect("再部分写");
    let mut current = current_version_of(&session);
    damage_the_copy_at(
        &mut pool.devices,
        &middle_locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
    )
    .expect("回退照样做成");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "只有水位那一读读到中间那一版的单元"
    );
}

/// 部分写之前那一版是回退目标、部分写把 [`DAMAGED_UNIT`] 换到新槽的会话，连同现行那一版、目标那一版那个单元（回退复活它，逐份验）的两条位置条目。
fn session_after_a_partial_write_and_the_resurrected_unit_of_the_target(
    pool: &mut Pool,
) -> (
    MountedSession,
    TransactionOutput,
    RollbackTarget,
    [LocationEntry; 2],
) {
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let target = current_version_of(&session);
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 4),
        )
        .expect("部分写");
    let current = current_version_of(&session);
    let unit_of_the_target = mapped_locations(
        &target,
        &mapping_key_of_role(&target, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    (
        session,
        current,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
        unit_of_the_target,
    )
}

/// 回退重建目标那一版把复活单元读回来：前一条位置条目那一份坏，这一读读到只坏一份进别的读那个数；复活单元那一判逐份读盘、拒在这一份，不经按位置条目读，不再加次数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退重建目标那一版读到复活单元不进账」那一行。
#[test]
fn a_rollback_refused_at_a_resurrected_unit_counts_its_damaged_copy_once_for_rebuilding_the_target()
{
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, mut current, target, unit_of_the_target) =
        session_after_a_partial_write_and_the_resurrected_unit_of_the_target(&mut pool);
    damage_the_copy_at(
        &mut pool.devices,
        &unit_of_the_target[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. })
        ),
        "拒在复活单元那一判：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "重建目标那一版读到它一次，复活单元那一判不进这个数"
    );
}

/// 回退读现行那一版的账（从盘上沿根读一遍分配记录树）：现行那一版的分配记录树根弄坏一份，目标那一版的一个复活单元弄坏后一条位置条目那一份，回退拒在复活单元那一判，走不到别的读。
/// 重建目标那一版读复活单元读到前一条就对上，碰不到后一条（前一条那一份坏时它计一次，`a_rollback_refused_at_a_resurrected_unit_counts_its_damaged_copy_once_for_rebuilding_the_target`）；复活单元那一判不进这个数；两个单元的位置条目不同，一把 key 只计一次也并不到一起。
/// 所以别的读那个数的 1 只能是读现行账记的。判别力：`crates/mutations.tsv`「可写挂载别的读：回退读现行那一版的账不进账」那一行。
#[test]
fn a_rollback_refused_after_reading_the_current_account_counts_its_damaged_root() {
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, mut current, target, unit_of_the_target) =
        session_after_a_partial_write_and_the_resurrected_unit_of_the_target(&mut pool);
    damage_the_copy_at(
        &mut pool.devices,
        &unit_of_the_target[1],
        BYTES_OF_A_DATA_UNIT,
    );
    let account_root_of_the_current_version = mapped_locations(
        &current,
        &mapping_key_of_role(&current, TransactionUnit::AllocationTree),
    )
    .expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &account_root_of_the_current_version[0],
        BYTES_OF_A_NODE,
    );
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. })
        ),
        "拒在复活单元那一判：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}
