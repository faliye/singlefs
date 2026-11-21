//! 释放检查读坏的一份：计数并由那次发布替换单元，释放检查之后失败的发布保留计数、重发的版本隔离两份，第二份读坏文件读看不见、释放检查计数，读回空的一份与读坏同样计，释放检查中途被拒时保留拒绝之前读到的计数。

use super::*;

/// 释放之前读盘核读到只坏一份（整份顺序写重写弄坏的单元）：进别的读那个数；这个单元这次发布就换了落点、旧落点两份隔离，另发的那一批没事可做。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：释放之前读盘核」那一行。
#[test]
fn the_release_check_reading_one_damaged_copy_counts_it_and_the_unit_is_replaced_by_that_publish() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    assert_nothing_more_to_relocate(&published);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "释放之前读盘核那一读进数"
    );
    let after = current_version_of(&session);
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::Data(_))),
        device_and_slot_of_both_copies(&locations),
        "读盘核读到了那一份坏副本，两份一起隔离"
    );
    assert_eq!(whole_first_file(&pool.devices, &after), new_content);
}

/// 发布在读盘核之后才被拒（整份顺序写重写弄坏的单元，落盘中途写失败、冻结）：读盘核读到的那一份已经进别的读那个数，改动没发成也不退；
/// 原样重发之后旧落点两份隔离，再下一次改动不另发。判别力：`crates/mutations.tsv`「可写挂载别的读：释放之前读盘核读到只坏一份不记（发布后来被拒）」那一行。
#[test]
fn a_publish_failing_after_the_release_check_keeps_the_count_and_the_resent_version_quarantines_both_copies(
) {
    let pool = pool_holding(&content_of_units(3));
    let version = pool.version.clone();
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    let mut failing = pool_failing_on_request(pool.devices);
    let mounted = mount_writable(&parameters(), &mut failing.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let device_of_the_damaged_copy = &mut failing
        .devices
        .iter_mut()
        .find(|(identity, _)| *identity == locations[0].device)
        .expect("池里有这块盘")
        .1
        .inner;
    let mut damaged_copy = vec![0u8; BYTES_OF_A_DATA_UNIT];
    device_of_the_damaged_copy
        .read_at(locations[0].slot.to_device_offset(), &mut damaged_copy)
        .expect("读得出那一份");
    damaged_copy[FLIPPED_BYTE_IN_THE_UNIT] ^= 0xFF;
    device_of_the_damaged_copy
        .write_at(
            locations[0].slot.to_device_offset(),
            &damaged_copy,
            WriteDurability::Plain,
        )
        .expect("写回改过的那一份");
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    failing.writes_fail.store(true, Ordering::SeqCst);
    let refused = session.publish_user_change(
        &mut failing.devices,
        UserChange::SequentialWrite(FirstFile {
            content: &new_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
        }),
    );
    assert!(
        matches!(refused, Err(UserChangeRefused::Publish { .. })),
        "落盘中途写失败：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "读盘核读到的那一份进数，改动被拒不退"
    );
    failing.writes_fail.store(false, Ordering::SeqCst);
    assert!(
        session
            .resend_the_frozen_publish(&mut failing.devices)
            .expect("写不再失败，重发得成"),
        "有冻结着的那一次"
    );
    let after = current_version_of(&session);
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::Data(_))),
        device_and_slot_of_both_copies(&locations),
        "读盘核读到了那一份坏副本，重发的那一版两份一起隔离"
    );
    let next = session
        .publish_user_change(&mut failing.devices, new_inode_change(4))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 第二份坏：读路径头一份对上就停、读不到它，别的读那个数不加；释放之前读盘核两份都读，读到它：进数，两份一起隔离。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：释放之前读盘核只看头一份读得好不好（第二份坏不报）」那一行。
#[test]
fn a_second_copy_damaged_is_not_seen_by_the_file_read_and_is_counted_by_the_release_check() {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[1], BYTES_OF_A_DATA_UNIT);
    assert_eq!(
        whole_first_file_read_through_the_session(&session, &pool.devices).bytes,
        content
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "头一份对上就停，读不到第二份"
    );
    let new_content: Vec<u8> = content.iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    assert_nothing_more_to_relocate(&published);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "释放之前读盘核读到第二份"
    );
    assert_eq!(
        quarantined_copies_of(&current_version_of(&session), |unit| matches!(
            unit,
            TransactionUnit::Data(_)
        )),
        device_and_slot_of_both_copies(&locations),
        "两份一起隔离"
    );
}

/// 头一份读回 `None`（设备报读错，不是校验和对不上）：与读成而对不上同一支：内容照出、进别的读那个数，下一次改动发成之后另发的那一批搬到新槽。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：读回 None 不算只坏一份（只认读成而对不上）」那一行。
#[test]
fn a_copy_reading_back_none_counts_like_a_damaged_copy_and_the_next_change_relocates_the_unit() {
    let content = content_of_units(3);
    let pool = pool_holding(&content);
    let mut failing = pool_failing_on_request(pool.devices);
    let mounted = mount_writable(&parameters(), &mut failing.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    *failing
        .copy_whose_reads_fail
        .lock()
        .expect("这条用例单线程，锁没有中毒") =
        Some((locations[0].device, locations[0].slot.to_device_offset().0));
    let size = version.inode_record.expect("第一个文件那条记录").size;
    let read = session
        .read_the_file(&failing.devices, FIRST_FILE, FileOffsetInBytes(0), size)
        .expect("另一份读得出");
    assert_eq!(read.bytes, content);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    let published = session
        .publish_user_change(&mut failing.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, _, _) = relocated_of(&published);
    assert_eq!(mapping_keys, std::slice::from_ref(&key));
    assert_ne!(
        mapped_locations(&current_version_of(&session), &key),
        Some(locations),
        "搬到新槽"
    );
}

/// [`publish_relocation_batch`] 的计划头一项（现行那一版的单元）读盘核读到只坏一份，第二项（留存单元，内存里那一版把两条位置项改成同一块盘）在读盘核那一圈被拒、提前返回：
/// 返回之前读到的那一份照样进账。判别力：`crates/mutations.tsv`「可写挂载别的读：读盘核那一圈提前返回不记账」那一行。
#[test]
fn a_release_check_refused_part_way_keeps_what_it_read_before_the_refusal_in_the_count() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let key_of_the_kept_unit = mapping_key_of_role(
        &current_version_of(&session),
        TransactionUnit::Data(DAMAGED_UNIT),
    );
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    let mut forged_version = current_version_of(&session);
    let key_of_the_unit_of_this_version = mapping_key_of_role(
        &forged_version,
        TransactionUnit::Data(DataUnitIndexInFile(0)),
    );
    let locations_of_the_unit_of_this_version =
        mapped_locations(&forged_version, &key_of_the_unit_of_this_version).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_of_this_version[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let kept_locations = &mut forged_version
        .mapping_entries_kept_for_the_snapshots
        .iter_mut()
        .find(|(kept_key, _)| *kept_key == key_of_the_kept_unit)
        .expect("映射条目留存")
        .1;
    kept_locations[1].device = kept_locations[0].device;
    let source_of_the_kept_unit = Placement {
        slot: kept_locations[0].slot,
        span: 2,
    };
    let planned = [
        PlannedRelocation {
            mapping_key: key_of_the_unit_of_this_version,
            source: Placement {
                slot: locations_of_the_unit_of_this_version[0].slot,
                span: 2,
            },
        },
        PlannedRelocation {
            mapping_key: key_of_the_kept_unit,
            source: source_of_the_kept_unit,
        },
    ];
    let pool_parameters = parameters();
    let mut writer = PoolWriter::new(&pool_parameters, pool.devices.as_mut_slice());
    let refused = publish_relocation_batch(
        &mut writer,
        &mut session.allocator,
        &forged_version,
        &planned,
        session.instance,
    );
    assert!(
        matches!(
            refused,
            Err(RelocationBatchRefused::Publish(
                PublishError::MappingEntryLocationsOnTheSameDevice { .. }
            ))
        ),
        "第二项在读盘核那一圈被拒：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "头一项读盘核读到的那一份进数"
    );
}
