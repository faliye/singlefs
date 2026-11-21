//! 数据单元与映射树叶有一份读坏：经会话读文件计数、下一次改动搬走那个单元，挂载期间不经会话读只记在它自己的观测里，保留单元的源读坏照计（批被拒也计），挂载时读坏的映射树叶在下一次改动之后重写、为空间被拒一次的抬回收下限之后发布、不为空间被拒的不再试，挂载时读的别的读计数对数据单元与映射树叶相同，不为空间被拒的批在那个单元再读一次时重试。

use super::*;

/// 经会话读文件读到只坏一份：内容照出、计 1；同一份坏副本等着下一次改动的这一程里再读不再加；下一次改动（建一个空 inode，不碰这个单元）发成之后
/// 另发的那一批把它搬到新槽、两份新写，旧落点两份隔离，现行版本不再引用弄坏的那个落点；账上清掉，再下一次改动不另发；放回弄坏的字节之后池级 checker 过。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：会话读文件」「现行版本不再引用弄坏的落点：搬迁那一批映射条目不换到目的地」两行。
#[test]
fn a_file_read_through_the_session_counts_the_damaged_copy_and_the_next_change_relocates_the_unit()
{
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    assert!(
        version_references_the_location(&version, &pool.devices, &locations[0]),
        "前提：弄坏之前现行版本引用这个落点"
    );
    let damaged_copy = damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let read = whole_first_file_read_through_the_session(&session, &pool.devices);
    assert_eq!(read.bytes, content, "读出原内容");
    assert_eq!(
        read.observation.data_unit_reads_with_one_copy_failing, 1,
        "这一次读读到只坏一份"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "进这次挂载别的读那个数"
    );
    whole_first_file_read_through_the_session(&session, &pool.devices);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "同一份坏副本等着下一次改动的这一程里只算一次"
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, units_replaced, floor_raises) = relocated_of(&published);
    assert_eq!(mapping_keys, std::slice::from_ref(&key), "搬的就是那个单元");
    assert!(units_replaced.is_empty(), "没有豁免映射的单元要换");
    assert_eq!(floor_raises, 0, "那一批一次都没被拒");
    let after = current_version_of(&session);
    assert!(
        !version_references_the_location(&after, &pool.devices, &locations[0]),
        "搬之后现行版本不再引用弄坏的那个落点"
    );
    let new_locations = mapped_locations(&after, &key).expect("搬之后映射里查得到");
    for (new_location, old_location) in new_locations.iter().zip(&locations) {
        assert_ne!(new_location.slot, old_location.slot, "落点换到新槽");
        assert!(
            copy_self_verifies(&pool.devices, new_location, BYTES_OF_A_DATA_UNIT),
            "新槽上那一份新写"
        );
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::Data(_))),
        device_and_slot_of_both_copies(&locations),
        "旧落点两份都隔离"
    );
    assert_eq!(whole_first_file(&pool.devices, &after), content);
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
    assert!(
        !version_references_the_location(
            &current_version_of(&session),
            &pool.devices,
            &locations[0]
        ),
        "再下一次改动之后现行版本也不引用弄坏的那个落点"
    );
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "file-read-relocated",
        &mut pool.devices,
        &[damaged_copy],
    );
}

/// `open_pool_for_read` 那条不挂在会话上的路是只读语义：读者自己的观测计 1，会话的数不加、下一次改动不搬、坏的那一份还在。
#[test]
fn a_file_read_off_the_session_during_the_mount_only_counts_in_its_own_observation() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let image = memory_pool_of(&pool.devices);
    let mounted_for_read = open_pool_for_read(&image, &version.root).expect("打开挂载态");
    let file = mounted_for_read
        .open_file(&image, FIRST_FILE)
        .expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("读得出");
    assert_eq!(output.observation.data_unit_reads_with_one_copy_failing, 1);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&published);
    let after = current_version_of(&session);
    assert_eq!(mapped_locations(&after, &key), Some(locations));
}

/// 搬留存单元读源读到只坏一份：另发的那一批被开关拒成不是空间不够、单元还钉在旧落点上；照它发一批搬迁，第二项计划对不上（这一批在任何写之前拒，走不到释放之前读盘核），
/// 头一项按留存条目读源那一次已经进别的读那个数。判别力：`crates/mutations.tsv`「可写挂载别的读：搬留存单元读源」那一行。
#[test]
fn reading_the_source_of_a_kept_unit_with_one_copy_damaged_counts_it_even_when_the_batch_is_refused(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "开关在读源之前就拒了那一批"
    );
    let after_the_partial_write = current_version_of(&session);
    let (_, kept_locations) = after_the_partial_write
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(kept_key, _)| *kept_key == key)
        .expect("映射条目留存");
    let source = Placement {
        slot: kept_locations[0].slot,
        span: 2,
    };
    let planned = [
        PlannedRelocation {
            mapping_key: key.clone(),
            source,
        },
        PlannedRelocation {
            mapping_key: vec![0u8; 27],
            source,
        },
    ];
    let pool_parameters = parameters();
    let mut writer = PoolWriter::new(&pool_parameters, pool.devices.as_mut_slice());
    let refused = publish_relocation_batch(
        &mut writer,
        &mut session.allocator,
        &after_the_partial_write,
        &planned,
        session.instance,
    );
    assert!(
        matches!(
            refused,
            Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { .. })
        ),
        "第二项计划对不上：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "头一项读源那一次进数"
    );
}

/// 映射树叶挂载读到只坏一份：挂载计 1、认不出映射 key；挂载这一串不碰它；头一次改动（建一个空 inode，也不碰它）发成之后另发的那一批把从根到它的路整条重写：
/// 叶换到新槽、两份新写，旧落点两份隔离，现行版本不再引用弄坏的那个落点；账上清掉，再下一次改动不另发；下一次挂载读不到坏副本（计 0），文件读出原内容，放回弄坏的字节之后池级 checker 过。
/// 判别力：`crates/mutations.tsv`「主动修复：映射树节点」两行。
#[test]
fn a_mapping_tree_leaf_read_with_one_copy_damaged_while_mounting_is_rewritten_after_the_next_change(
) {
    let content = content_of_units(UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE);
    let mut pool = pool_holding(&content);
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    let damaged_copy = damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    assert_eq!(session.unit_reads_with_one_copy_failing_while_mounting(), 1);
    assert!(
        session
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
            .is_empty(),
        "映射树节点认不出映射 key"
    );
    assert!(
        current_version_of(&session)
            .central_mapping_tree
            .pointers
            .contains(&leaf),
        "前提：挂载这一串没碰这片叶"
    );
    assert!(
        version_references_the_location(
            &current_version_of(&session),
            &pool.devices,
            &leaf.locations[0]
        ),
        "前提：头一次改动之前现行版本引用这个落点"
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, units_replaced, floor_raises) = relocated_of(&published);
    assert!(mapping_keys.is_empty(), "没有进映射的单元要搬");
    assert_eq!(
        units_replaced.len(),
        1,
        "换下的就是那片叶：{units_replaced:?}"
    );
    assert!(
        matches!(
            units_replaced[0],
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        ),
        "{units_replaced:?}"
    );
    assert_eq!(floor_raises, 0);
    let after = current_version_of(&session);
    assert!(
        !version_references_the_location(&after, &pool.devices, &leaf.locations[0]),
        "那片叶换了落点：现行版本不再引用弄坏的那个落点"
    );
    assert!(
        !after.central_mapping_tree.pointers.contains(&leaf),
        "那片叶的指针换了"
    );
    for pointer in &after.central_mapping_tree.pointers {
        for location in &pointer.locations {
            assert!(
                copy_self_verifies(&pool.devices, location, BYTES_OF_A_NODE),
                "映射树每个节点的每一份都读得回：{location:?}"
            );
        }
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        )),
        device_and_slot_of_both_copies(&leaf.locations),
        "旧落点两份都隔离"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
    assert!(
        !version_references_the_location(
            &current_version_of(&session),
            &pool.devices,
            &leaf.locations[0]
        ),
        "再下一次改动之后现行版本也不引用弄坏的那个落点"
    );
    assert_eq!(
        session_of_a_writable_mount(&mut pool.devices)
            .unit_reads_with_one_copy_failing_while_mounting(),
        0,
        "下一次挂载读不到坏副本"
    );
    assert_eq!(
        whole_first_file_read_through_the_session(
            &session_of_a_writable_mount(&mut pool.devices),
            &pool.devices
        )
        .bytes,
        content
    );
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "mapping-tree-leaf-rewritten",
        &mut pool.devices,
        &[damaged_copy],
    );
}

/// 同上，那一批头一次被空间拒（只供测试的开关）：推一串抬 F 之后再发同一批，叶照样换了落点。抬 F 的上限是第 4 新的非空有效根，不足 4 条取最旧的有效根
/// （D16（发布语义） 已定项 1）：新建文件、顺序写与弄坏之后那次改动只有 3 条，上限等于今天的 F、没有可抬的，所以弄坏之前先发两次建 inode，留下可抬的 F。
#[test]
fn a_mapping_tree_leaf_rewrite_refused_once_for_space_is_published_after_raising_the_floor() {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let version_before_the_damage = {
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        for write_time_offset in [3, 4] {
            session
                .publish_user_change(&mut pool.devices, new_inode_change(write_time_offset))
                .expect("建一个空 inode，留下一条非空有效根");
        }
        current_version_of(&session)
    };
    let leaf = leftmost_mapping_tree_leaf_of(&version_before_the_damage);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(5))
        .expect("建一个空 inode");
    let (_, units_replaced, floor_raises) = relocated_of(&published);
    assert_eq!(units_replaced.len(), 1, "{units_replaced:?}");
    assert!(floor_raises >= 1, "被空间拒过、推过抬 F");
    assert!(
        !current_version_of(&session)
            .central_mapping_tree
            .pointers
            .contains(&leaf),
        "那片叶换了落点"
    );
}

/// 只试一次：那一批被拒、不是空间不够（只供测试的开关）时结局随那次改动报出，叶还在旧落点上；账上清掉，下一次改动不接着换。
/// 判别力：`crates/mutations.tsv`「主动修复：被拒之后接着试」那一行。
#[test]
fn a_mapping_tree_leaf_rewrite_refused_not_for_space_is_not_tried_again_by_the_next_change() {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert!(
        matches!(
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        ),
        "结局随那次改动报出：{:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    assert!(
        current_version_of(&session)
            .central_mapping_tree
            .pointers
            .contains(&leaf),
        "叶还在旧落点上"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 账的口径：同一份坏副本挂载读到一次，数据单元（认得出 key）与映射树叶（认不出）一样：挂载那个数 1，之后再读、或另发那一批换下时释放之前读盘核，别的读那个数都不加。
/// 判别力：`crates/mutations.tsv`「账的口径：挂载交回之前把认成 key 的那几组从账上去掉」那一行。
#[test]
fn the_other_reads_count_of_a_copy_read_while_mounting_is_the_same_for_a_data_unit_and_a_mapping_tree_leaf(
) {
    let mut data_pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(&data_pool.version, TransactionUnit::Data(DAMAGED_UNIT));
    let data_locations = mapped_locations(&data_pool.version, &key).expect("在映射里");
    damage_the_copy_at(
        &mut data_pool.devices,
        &data_locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let mut devices_read_again = copy_of_the_devices(&data_pool.devices);
    let session_reading_again = session_of_a_writable_mount(&mut devices_read_again);
    let data_mount = session_reading_again.unit_reads_with_one_copy_failing_while_mounting();
    whole_first_file_read_through_the_session(&session_reading_again, &devices_read_again);
    let data_read_again =
        session_reading_again.other_unit_reads_with_one_copy_failing_in_this_mount();
    let mut devices_not_read_again = copy_of_the_devices(&data_pool.devices);
    let mut session_not_reading_again = session_of_a_writable_mount(&mut devices_not_read_again);
    let published = session_not_reading_again
        .publish_user_change(&mut devices_not_read_again, new_inode_change(3))
        .expect("建一个空 inode");
    let (data_mapping_keys, _, _) = relocated_of(&published);
    assert_eq!(data_mapping_keys, std::slice::from_ref(&key));
    let data_relocated_without_another_read =
        session_not_reading_again.other_unit_reads_with_one_copy_failing_in_this_mount();

    let mut leaf_pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&leaf_pool.version);
    damage_the_copy_at(&mut leaf_pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut leaf_session = session_of_a_writable_mount(&mut leaf_pool.devices);
    let leaf_mount = leaf_session.unit_reads_with_one_copy_failing_while_mounting();
    whole_first_file_read_through_the_session(&leaf_session, &leaf_pool.devices);
    let leaf_read_again = leaf_session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let leaf_published = leaf_session
        .publish_user_change(&mut leaf_pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (_, leaf_units_replaced, _) = relocated_of(&leaf_published);
    assert_eq!(leaf_units_replaced.len(), 1, "{leaf_units_replaced:?}");
    let leaf_replaced = leaf_session.other_unit_reads_with_one_copy_failing_in_this_mount();

    assert_eq!(
        (
            data_mount,
            data_read_again,
            data_relocated_without_another_read
        ),
        (1, 0, 0),
        "数据单元：挂载、再读、换下时读盘核"
    );
    assert_eq!(
        (leaf_mount, leaf_read_again, leaf_replaced),
        (1, 0, 0),
        "映射树叶：挂载、再读、换下时读盘核"
    );
}

/// 「只试一次」的射程：这一组记下的这一次只试一次；账上清掉之后同一组再读到就再记、下一次改动再试。
/// 数据单元这一组是那一批计划里的源落点，账上清掉它的有「试过的组」「计划里的源落点」两支；豁免映射的单元只有前一支，在下面那一条用例。
/// 判别力：`crates/mutations.tsv`「主动修复：那一批发完账上一组都不去掉」那一行。
#[test]
fn a_batch_refused_not_for_space_is_tried_again_once_the_unit_is_read_again() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    // 迭代上界是下面这三遍；跨轮不带状态，每一遍都读一次、发一次改动。
    for attempt in 0..3u64 {
        whole_first_file_read_through_the_session(&session, &pool.devices);
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
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        3
    );
}
