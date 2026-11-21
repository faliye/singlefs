//! 挂载那一刻的宽护：384 槽盘上不隔离仍记成已分配的槽、1536 槽盘上释放一大批之后不双扣，取现行账的根在删除与顺序写那两形上接下每个已释放的落点。

use super::*;

/// H2 双扣（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3、第三节第十四块第 1 条；正推第 1.3 节 384 槽盘 720 槽那一形）：
/// 覆盖写 40 次、回退到倒数第 7 版录 R，可写挂载那一刻 R 的账与谱系片读回全零：认不出是不是回退，C 里 R 那一槽宽护、并进现行账里全部已释放的落点。
/// 其中还在 defer 窗口里的（分配位置着）算在「已分配」里，D28（挂载期承诺量） 已定项 1 第九项「不在这里再算一次」：挂载那一刻一个槽都不同时
/// 置着分配位与隔离位，C 里 R 那一槽的每个槽也都不空闲（分配着或隔离着，恰好一样）。两块盘 104 个槽两项若各扣一次，隔离 720 槽。
/// 这一形去掉双扣之后写照旧全被拒（隔离 616 槽），不在这里钉，交主 agent。
#[test]
fn wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices(
) {
    let recorded = record_one_rollback_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
    )
    .recorded;
    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
    );
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&account_writes_of(&recorded))
            .with_the_writes(&lineage_unit_writes),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    let borrowed_for_the_recorded_publish = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let released_not_borrowed: Vec<&(DeviceIdentity, SlotNumber)> = released_in_the_current_account
        .iter()
        .filter(|(device, slot)| {
            !borrowed_for_the_recorded_publish
                .iter()
                .any(|borrowed| borrowed.device == *device && borrowed.slot == *slot)
        })
        .collect();
    assert!(
        released_not_borrowed.is_empty(),
        "认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（阳性对照：宽护走到了），缺 {released_not_borrowed:?}"
    );
    assert!(
        isolated_slots_of(&mounted.allocator) > 0,
        "挂载那一刻隔离着 C 里分配位没置着的槽（阳性对照）"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：现行账里仍分配着的、已释放而还在 defer 窗口里的不在这里再算一次）"
    );
    let free_among_the_borrowed =
        free_slots_covered_by(&mounted.allocator, &borrowed_for_the_recorded_publish);
    assert!(
        free_among_the_borrowed.is_empty(),
        "C 里 R 那一槽的每个槽挂载那一刻都不空闲（分配着或隔离着）：空闲的（盘、槽）{free_among_the_borrowed:?}"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices,
    );
}

/// R 每条记录（按本次发布内序号）点名的打包单元（类标签 3）各是哪一种打包记录类型，按点名的次序：从录下来的记录写与单元写的字节解，
/// 不经被测的方法。点名的单元 R 没写（照抄上一版的）记 `None`。
fn packed_record_types_named_by_each_record_of(
    recorded: &RecordedPublish,
) -> Vec<Vec<Option<u16>>> {
    let filesystem_identifier = singlefs_core::unit::unit_filesystem_identifier(
        &recorded.pool.parameters.filesystem_identifier,
    );
    (1..=recorded.record_count())
        .map(|ordinal_from_one| {
            let record_bytes = recorded.record_writes_of_the_record(ordinal_from_one)[0]
                .bytes()
                .expect("记录写带字节");
            let record =
                singlefs_core::journal::JournalRecord::parse(record_bytes, filesystem_identifier)
                    .expect("R 录下来的记录解得开");
            record
                .named
                .iter()
                .filter(|named| named.unit_class == singlefs_core::unit::UNIT_CLASS_PACKED)
                .map(|named| {
                    recorded
                        .unit_writes()
                        .into_iter()
                        .find(|write| {
                            write.device == named.locations[0].device
                                && write.offset == named.locations[0].slot.to_device_offset()
                        })
                        .and_then(|write| write.bytes())
                        .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
                        .map(|packed| packed.identity.record_type)
                })
                .collect()
        })
        .collect()
}

/// R 走丙、认不出是不是回退那一形的共同判法（tri 第一种）：可写挂载那一刻 `ranges` 读回全零，择根退到 P；C 里 R 那一槽里在现行账里记成已释放的落点
/// 就是现行账里全部已释放的（宽护：一个不缺，按盘与起点槽）。
fn assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
    recorded: &RecordedPublish,
    ranges: &RangesReadingBackAllZero,
) {
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(&crash_image, ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    assert!(
        !released_in_the_current_account.is_empty(),
        "挂载那一刻现行账里有已释放的落点（阳性对照：并少了看得出来）"
    );
    let released_borrowed_for_the_recorded_publish: std::collections::BTreeSet<(
        DeviceIdentity,
        SlotNumber,
    )> = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）")
        .into_iter()
        .map(|placement| (placement.device, placement.slot))
        .filter(|placement| released_in_the_current_account.contains(placement))
        .collect();
    assert_eq!(
        released_borrowed_for_the_recorded_publish, released_in_the_current_account,
        "R 的记录点名的打包单元有一片读不出、认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（tri 宽护）"
    );
}

/// 删除那一形之前覆盖写几次。
const OVERWRITES_BEFORE_THE_DELETION_RECORDED_AS_THE_RECORDED_PUBLISH: u64 = 3;

/// tri 第一种的删除那一形（正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 2 节「删除发布墓碑片读不出 → 宽护没有入库用例」，
/// 探针 `faults=racct,rtomb`）：R 是一次删除的第一次发布，记录点名它写的 inode 叶容器与墓碑片（叶容器排在前面）；挂载那一刻 R 的账与墓碑片
/// 读回全零：认不出是不是回退，C 里 R 那一槽并进现行账里全部已释放的落点。读盘一层只看第一个打包单元（正推 X5）时只看到读得出的叶容器、
/// 认成肯定不是回退，这一句红。
#[test]
fn a_deletion_whose_tombstone_unit_and_account_read_back_zero_at_the_mount_takes_every_released_placement(
) {
    let recorded = record_the_first_publish_of_one_deletion_after_overwrites(
        OVERWRITES_BEFORE_THE_DELETION_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let tombstone_unit_writes =
        packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_TOMBSTONE);
    assert!(
        !tombstone_unit_writes.is_empty(),
        "删除的第一次发布写了墓碑片（阳性对照：故障有处可打）"
    );
    let named_in_order: Vec<Option<u16>> = packed_record_types_named_by_each_record_of(&recorded)
        .into_iter()
        .flatten()
        .collect();
    let tombstone_position = named_in_order
        .iter()
        .position(|record_type| *record_type == Some(singlefs_core::unit::PACKED_TYPE_TOMBSTONE))
        .expect("R 的记录点名墓碑片");
    assert!(
        tombstone_position > 0,
        "R 的记录在墓碑片之前还点名了别的打包单元（这一形分得出只看第一个打包单元）：{named_in_order:?}"
    );
    assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
        &recorded,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&account_writes_of(&recorded))
            .with_the_writes(&tombstone_unit_writes),
    );
}

/// 顺序写那一形之前覆盖写几次。
const OVERWRITES_BEFORE_THE_SEQUENTIAL_WRITE_RECORDED_AS_THE_RECORDED_PUBLISH: u64 = 3;

/// tri 第一种的多条记录那一形：R 顺序写三个数据单元（三条记录，前两条只点名数据单元，inode 叶容器只在末条点名）；挂载那一刻 R 的账与它写的
/// inode 叶容器读回全零：认不出是不是回退，C 里 R 那一槽并进现行账里全部已释放的落点。读盘一层只看第一条记录点名的打包单元（正推 X4）时
/// 一个打包单元都看不到、认成肯定不是回退，这一句红。
#[test]
fn a_sequential_write_whose_inode_unit_named_by_its_last_record_and_account_read_back_zero_at_the_mount_takes_every_released_placement(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_SEQUENTIAL_WRITE_RECORDED_AS_THE_RECORDED_PUBLISH,
        RecordedPublishKind::SequentialWriteOfThreeDataUnits,
    );
    let named_by_each_record = packed_record_types_named_by_each_record_of(&recorded);
    assert!(
        named_by_each_record.len() >= 2 && named_by_each_record[0].is_empty(),
        "R 有两条以上记录、第一条不点名打包单元（这一形分得出只看第一条记录）：{named_by_each_record:?}"
    );
    let inode_unit_writes =
        packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_INODE);
    assert!(
        !inode_unit_writes.is_empty(),
        "R 写了 inode 叶容器（阳性对照：故障有处可打）"
    );
    assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
        &recorded,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&account_writes_of(&recorded))
            .with_the_writes(&inode_unit_writes),
    );
}
