//! 挂载时跳过的复活单元、共用单元在抬 F 回收时补隔离，留到 F 生效。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero,
    );
}

/// 抬 F 那一串卸载那一格当场放开那一截回收时照样补隔离（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md`
/// 第二节 K2 ①；形照攻方 `research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-u1.txt`：384 槽、覆盖写 10 次、回退到倒数第 7 版录 R、
/// 挂载那一刻 R 的根槽与谱系片读回全零而挂上之后照旧、挂上就卸载）。与 [`revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero`]
/// 同一个池、同一次挂载，只把挂上之后那一串抬 F 换成正常卸载：R 复活的数据单元在 P 的账里已释放、还在 defer 窗口里，挂载那一刻不隔离
/// （D28（挂载期承诺量） 已定项 1 第九项不再算一次），卸载那一串回收它们；R 的根槽照旧读回全零，这一串不把 R 当被抛弃根，护着它们的只有 C 那一圈。
/// 卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，复活的那几个隔离着，没有一个槽同时算进「已分配」与第九项。
/// 当场放开那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，复活的槽卸载之后空闲，这一条红。扣到生效那一截改回只回收，
/// 这一条照绿，那一截由
/// [`unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_the_recorded_publish_reads_back_zero`] 钉。
#[test]
fn unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_the_root_slot_of_the_rollback_reads_back_zero(
) {
    let RecordedRollback {
        recorded, revived, ..
    } = record_one_rollback_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
    );
    assert!(
        !revived.is_empty(),
        "回退目标的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_root_slot_of(&crash_image, recorded.key)
            .with_the_writes(&lineage_unit_writes),
    );
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: None,
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: recorded.content.clone(),
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    let mounted = mount_writable_with_space_admission(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的根槽与谱系片读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读不出、系统配置没见证 R、记录不施加：择根落到 P（阳性对照：故障打到了）"
    );
    let named_by_the_unapplied_records_at_the_mount = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records();
    let revived_not_in_the_abandoned_publish: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !named_by_the_unapplied_records_at_the_mount
                .iter()
                .any(|placement| {
                    placement.device == revived_placement.device
                        && placement.slot == revived_placement.slot
                })
        })
        .collect();
    assert!(
        revived_not_in_the_abandoned_publish.is_empty(),
        "C 那一圈并进了 R 的账（记录的新根段读得出），R 复活的数据单元都在里面（阳性对照），缺 {revived_not_in_the_abandoned_publish:?}"
    );
    // R 的根槽与谱系片挂上之后照旧读回全零，到这个进程写掉那几段为止：卸载那一串读不出 R，不把它当被抛弃根算影子账。
    let (_output, mut session) = MountedSession::of_the_mount(mounted);
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    let reclaimed_by_the_unmount = match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => raised.reclaimed,
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("挂载落到的 P 带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("挂上就卸载做得成：{refusal:?}"),
    };
    assert!(
        revived.iter().any(|revived_placement| {
            reclaimed_by_the_unmount
                .iter()
                .any(|reclaimed| reclaimed.slot == revived_placement.slot)
        }),
        "卸载那一串回收了复活的数据单元（阳性对照：卸载那一格回收时补隔离那一处走到了），回收的 {reclaimed_by_the_unmount:?}"
    );
    assert!(
        root_is_in_the_ring(&pool.image(), recorded.key),
        "卸载之后 R 还在根环里（卸载那一串没写到 R 那一槽）"
    );
    let named_by_the_unapplied_records_after_the_unmount = session
        .allocator
        .root_ring_occupancy()
        .expect("卸载之后分配器照旧装着根环表")
        .placements_named_by_unapplied_journal_records();
    let free_among_the_named = free_slots_covered_by(
        &session.allocator,
        &named_by_the_unapplied_records_after_the_unmount,
    );
    assert!(
        free_among_the_named.is_empty(),
        "卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲（卸载那一串回收时照 C 那一圈补了隔离），空闲的（盘、槽）{free_among_the_named:?}"
    );
    let revived_slots_not_isolated: Vec<(DeviceIdentity, SlotNumber)> = revived
        .iter()
        .flat_map(|revived_placement| {
            let device_map = session
                .allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == revived_placement.device)
                .expect("池里有这块盘");
            (revived_placement.slot.0..revived_placement.slot.0 + revived_placement.span)
                .map(SlotNumber)
                .filter(move |slot| !device_map.is_isolated(*slot))
                .map(move |slot| (revived_placement.device, slot))
        })
        .collect();
    assert!(
        revived_slots_not_isolated.is_empty(),
        "卸载之后复活的数据单元（P 的账里已释放、卸载那一串回收了）覆盖的槽都隔离着，没隔离的（盘、槽）{revived_slots_not_isolated:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&session.allocator),
        0,
        "卸载之后没有一个槽同时算进「已分配」与第九项"
    );
}

/// 镜像根环里每块盘上读得出、自证过的根（`except` 那一条除外）各取 txg 最大的一条，再在各盘之间取最小：卸载那一格当场放开那一截
/// 放到哪儿（`mount::newest_valid_root_txg_on_every_device`）在这一形上的值，从盘上独立读、不经被测的方法。`except` 是按现行那一版的
/// 实例表被抛弃的那一条（这一形里只有 R）；有一块盘上一条根都读不出就判红（这一形里每块盘上都有这次挂载落的根）。
fn smallest_among_the_newest_root_txg_on_each_device(
    image: &MemoryPool,
    except: (InstanceGeneration, CheckpointTxg),
) -> CheckpointTxg {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let immutable = &configuration.immutable;
    let mut newest_on_each_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();
    // 迭代上界是根环槽数（3 区 × 每区槽数）；跨轮携带的是每块盘上到这一槽为止读到的最大 txg。
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        match read_root_ring_slot(
            image,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root)
                if (root.instance, root.checkpoint_txg) != except =>
            {
                let newest = newest_on_each_device
                    .entry(device)
                    .or_insert(root.checkpoint_txg);
                *newest = (*newest).max(root.checkpoint_txg);
            }
            RootRingSlotReading::SelfVerified(_) | RootRingSlotReading::Bad(_) => {}
        }
    }
    let devices_of_the_pool: std::collections::BTreeSet<DeviceIdentity> =
        immutable.region_devices.iter().copied().collect();
    assert_eq!(
        newest_on_each_device
            .keys()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        devices_of_the_pool,
        "每块盘上都读得出一条不被抛弃的根（这次挂载的暖机在每块盘上都落了根）"
    );
    newest_on_each_device
        .into_values()
        .min()
        .expect("池里至少一块盘")
}

/// 抬 F 那一串卸载那一格扣到生效那一截回收时照样补隔离（K2 ②）。
/// 池与挂载同丙那一形
/// （[`the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor`]：
/// 覆盖写 3 次得 P、录一次建 inode 得 R，崩在 R 的根槽落盘之后、轮换之前，R 的账读回全零、挂上之后照旧）：择根退到 P，C 里 R 那一槽并上
/// 挂载那一刻现行账里仍分配着的落点（R 与 P 共用的数据单元），它们分配位置着、挂载那一刻不隔离（D28（挂载期承诺量） 已定项 1 第九项不再算一次）。
/// 挂上之后覆盖写一次：会话自己释放这几个数据单元，释放代是这次覆盖写的 txg，高于各盘最新根取最小（这一 txg 的根只落在一块盘上），
/// 卸载那一串回收它们落在扣到生效那一截、不在当场放开那一截。卸载之后 R 还在根环里：它们隔离着、一槽都不空闲，C 那一圈罩着的槽一个都不空闲，
/// 没有一个槽同时算进「已分配」与第九项（第九项「这次挂载里释放、回收时分配器照补隔离」，I-7.4（近 K 代块未被复用））。
/// 扣到生效那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，卸载末尾放开扣住位之后它们空闲，这一条红。
#[test]
fn unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_the_recorded_publish_reads_back_zero(
) {
    let recorded = record_one_inode_creation_after_overwrites(
        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let root_of_the_recorded_publish = recorded.root();
    let shared_with_the_previous_version =
        data_unit_placements_the_recorded_publish_shares_with_the_previous_version(&recorded);
    assert!(
        !shared_with_the_previous_version.is_empty(),
        "R 建一个 inode、不写文件的数据单元：它与 P 共用文件的数据单元（阳性对照：有要护的槽）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded)),
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
        "R 的账读回全零：R 走读不成、择根退到 P（阳性对照：故障打到了）"
    );
    let borrowed_at_the_mount = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点");
    let shared_not_borrowed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
        .iter()
        .filter(|shared| {
            !borrowed_at_the_mount
                .iter()
                .any(|borrowed| borrowed.device == shared.device && borrowed.slot == shared.slot)
        })
        .collect();
    assert!(
        shared_not_borrowed.is_empty(),
        "挂载那一刻 C 里 R 那一槽并上了现行账里仍分配着的落点（丙）：R 与 P 共用的数据单元都在里面，缺 {shared_not_borrowed:?}"
    );
    let shared_free_at_the_mount =
        free_slots_covered_by(&mounted.allocator, &shared_with_the_previous_version);
    assert!(
        shared_free_at_the_mount.is_empty(),
        "挂载那一刻 R 与 P 共用的数据单元在现行账里仍分配着：一槽都不空闲，空闲的（盘、槽）{shared_free_at_the_mount:?}"
    );
    let shared_isolated_at_the_mount =
        isolated_slots_covered_by(&mounted.allocator, &shared_with_the_previous_version);
    assert!(
        shared_isolated_at_the_mount.is_empty(),
        "挂载那一刻 C 那一圈不隔离分配位置着的槽（阳性对照：卸载之后的隔离是回收那一刻补的），隔离着的（盘、槽）{shared_isolated_at_the_mount:?}"
    );
    let (output, session_of_the_mount) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session_of_the_mount),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    let published = pool
        .overwrite(OVERWRITE_BYTES)
        .expect("挂上之后第一次覆盖写落得下");
    assert_eq!(
        published.floor_raises.len(),
        0,
        "挂上之后第一次覆盖写准入够、不推抬 F（共用的数据单元留给卸载那一串回收）"
    );
    let txg_of_the_overwrite = pool.session().current.root().checkpoint_txg;
    let shared_not_released_by_the_overwrite: Vec<&PlacementOnDevice> =
        shared_with_the_previous_version
            .iter()
            .filter(|shared| {
                !pool.session().allocator.records().iter().any(|record| {
                    record.device == shared.device
                        && record.slot == shared.slot
                        && record.is_released
                        && record.generation == txg_of_the_overwrite
                })
            })
            .collect();
    assert!(
        shared_not_released_by_the_overwrite.is_empty(),
        "这次覆盖写释放了 R 与 P 共用的数据单元、释放代是它的 txg {txg_of_the_overwrite:?}（阳性对照：释放是这次挂载里会话自己做的），没这样记着的 {shared_not_released_by_the_overwrite:?}"
    );
    let released_immediately_up_to =
        smallest_among_the_newest_root_txg_on_each_device(&pool.image(), recorded.key);
    assert!(
        txg_of_the_overwrite > released_immediately_up_to,
        "释放代 {txg_of_the_overwrite:?} 高于各盘最新根取最小 {released_immediately_up_to:?}：卸载那一串回收它们落在扣到生效那一截（阳性对照：走到要钉的那一截，不在当场放开那一截）"
    );
    let mut session = pool.session.take().expect("挂着的会话");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    let raised = match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => raised,
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("挂上之后覆盖写一次再卸载做得成：{refusal:?}"),
    };
    assert_eq!(
        raised.rollback_floor, txg_of_the_overwrite,
        "卸载把 F 抬到现行那一版（这次覆盖写）的 txg：回收门槛够得着它释放的槽"
    );
    let shared_not_reclaimed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
        .iter()
        .filter(|shared| {
            !raised
                .reclaimed
                .iter()
                .any(|reclaimed| reclaimed.slot == shared.slot)
        })
        .collect();
    assert!(
        shared_not_reclaimed.is_empty(),
        "卸载那一串回收了这次覆盖写释放的共用数据单元（阳性对照：回收时补隔离那一处走到了），没回收的 {shared_not_reclaimed:?}，回收的 {:?}",
        raised.reclaimed
    );
    assert!(
        root_is_in_the_ring(&pool.image(), recorded.key),
        "卸载之后 R 还在根环里（卸载那一串没写到 R 那一槽）"
    );
    let shared_free_after_the_unmount =
        free_slots_covered_by(&session.allocator, &shared_with_the_previous_version);
    assert!(
        shared_free_after_the_unmount.is_empty(),
        "卸载之后 R 还在根环里：扣到生效那一截回收的共用数据单元覆盖的槽一槽都不空闲（回收时照 C 那一圈补了隔离，卸载末尾放开扣住位之后照旧挡着），空闲的（盘、槽）{shared_free_after_the_unmount:?}"
    );
    assert_eq!(
        isolated_slots_covered_by(&session.allocator, &shared_with_the_previous_version),
        slots_covered_by(&shared_with_the_previous_version),
        "卸载之后共用数据单元覆盖的槽都隔离着"
    );
    let named_by_the_unapplied_records_after_the_unmount = session
        .allocator
        .root_ring_occupancy()
        .expect("卸载之后分配器照旧装着根环表")
        .placements_named_by_unapplied_journal_records();
    let free_among_the_named = free_slots_covered_by(
        &session.allocator,
        &named_by_the_unapplied_records_after_the_unmount,
    );
    assert!(
        free_among_the_named.is_empty(),
        "卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，空闲的（盘、槽）{free_among_the_named:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&session.allocator),
        0,
        "卸载之后没有一个槽同时算进「已分配」与第九项"
    );
}
