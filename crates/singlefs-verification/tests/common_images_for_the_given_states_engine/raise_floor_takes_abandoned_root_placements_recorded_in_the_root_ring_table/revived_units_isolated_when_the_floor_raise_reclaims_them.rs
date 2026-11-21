//! 挂载时跳过的复活单元在抬 F 回收它们时补隔离（R 的根槽读回全零），与按打包记录类型取 R 写的打包单元、录一次删除的帮手。

use super::*;

/// R 写出的打包单元里打包记录类型是 `record_type` 的那几份写：从录下来的写的字节解，不经被测的方法。
pub fn packed_unit_writes_of_type(
    recorded: &RecordedPublish,
    record_type: u16,
) -> Vec<&RetainedWrite> {
    recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            write
                .bytes()
                .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
                .is_some_and(|packed| packed.identity.record_type == record_type)
        })
        .collect()
}

/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次、建一个 inode，再经会话删掉它，录删除的第一次发布得 R
/// （崩在 R 的根槽落盘之后、轮换之前那一份写表只截到第一条根：删除那一次写两条根）。R 的记录点名它写的 inode 叶容器与墓碑片。
pub fn record_the_first_publish_of_one_deletion_after_overwrites(
    overwrites_before: u64,
) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    pool.create_one_inode().expect("建要删的那个 inode 落得下");
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_the_recorded_publish = stream.operation_count();
    let inode = singlefs_core::address::InodeNumber(
        pool.current_file_version().inode_number_watermark() - 1,
    );
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        singlefs_core::mounted_session::UserChange::Unlink {
            inode,
            write_time_seconds,
        },
        None,
    )
    .expect("删掉刚建的那个 inode 做得成");
    let operations = stream.retained_operations();
    let (mut writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_recorded_publish..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_root_index = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("删除写了根");
    writes.truncate(first_root_index + 1);
    let root = RootRecord::parse_slot(
        writes[first_root_index].bytes().expect("根槽写带字节"),
        &pool.parameters.filesystem_identifier,
    )
    .expect("删除的第一条根自证");
    RecordedPublish {
        content: previous_content.clone(),
        pool,
        base,
        writes,
        root_index: first_root_index,
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    }
}

/// 回退之前覆盖写几次（隔离量离整盘还远、写不被拒）。
pub const OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS: u64 = 10;

/// 挂上之后至多覆盖写几次去等 R 那一槽写掉（R 离开根环）。
pub const OVERWRITES_UNTIL_THE_RECORDED_PUBLISH_LEAVES_THE_RING_AT_MOST: usize = 40;

/// H2 改法要回收时补隔离那一半（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第三节第十四块第 1 条「I-7.4 照旧不红」）：
/// 覆盖写 10 次、回退到倒数第 7 版录 R，崩在 R 的根槽落盘之后、轮换之前，可写挂载那一刻 R 的根槽与 R 写的谱系片读回全零（C694 的 C：
/// 系统配置没见证 R、记录点名的谱系片验不过、记录不施加，择根落到 P）。R 的记录齐、用记录的新根段读得出 R 的账，C 那一圈把 R 的账里的落点
/// 隔离到这个进程写掉 R 那一槽；其中 R 复活的数据单元在 P 的账里已释放、
/// 还在 defer 窗口里（分配位置着），挂载那一刻不隔离（第九项不再算一次）。挂上之后先推一串抬 F 到准入上限：根环还没转，回收这几个复活单元的
/// 只有这一串；R 的根槽挂上之后照旧读回全零（到写掉那一槽为止），这一串读不出 R、不把它当被抛弃根算影子账，护着它们的只有 C 那一圈——分配器回收时照它补隔离
/// （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）。推过之后复活的槽一个都不空闲；之后覆盖写到 R 离开根环，
/// 每一步之后没有一个槽同时算进「已分配」与第九项、R 还在根环里时复活的槽不空闲、池级 checker 不红 I-7.4 与 I-4.8、写一次都不拒。
/// 抬 F 那一串回收之后不补隔离时，推过抬 F 那一句先红。
pub fn run_the_scenario_where_revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero(
    images: &mut PoolCheckerImages,
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
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_root_slot_of(&crash_image, recorded.key)
            .with_the_writes(&lineage_unit_writes),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
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
    let isolated_by_the_unapplied_records = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records();
    let revived_not_in_the_abandoned_publish: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !isolated_by_the_unapplied_records.iter().any(|placement| {
                placement.device == revived_placement.device
                    && placement.slot == revived_placement.slot
            })
        })
        .collect();
    assert!(
        revived_not_in_the_abandoned_publish.is_empty(),
        "C 那一圈并进了 R 的账（记录的新根段读得出），R 复活的数据单元都在里面（阳性对照），缺 {revived_not_in_the_abandoned_publish:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项"
    );
    // R 的根槽与谱系片挂上之后照旧读回全零，到这个进程写掉那几段为止（写过之后照读）：抬 F 那一串读不出 R，不把它当被抛弃根算影子账。
    let (output, session) = MountedSession::of_the_mount(mounted);
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
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    // 挂上之后先推一串抬 F 到准入上限（攻方动作 `F`）。
    let mut session_raising_the_floor = pool.session.take().expect("挂着的会话");
    let raised = {
        let PoolVersion::WithFile(current) = &mut session_raising_the_floor.current else {
            panic!("挂载落到的 P 带文件");
        };
        let current = Arc::make_mut(current);
        raise_rollback_floor_to_the_admission_ceiling(
            &pool.parameters,
            &mut pool.devices,
            &mut session_raising_the_floor.allocator,
            current,
            ShadowLedger::On,
        )
    };
    pool.session = Some(session_raising_the_floor);
    let reclaimed_by_the_raise = match raised {
        Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
            pool.note_floor_raises(std::slice::from_ref(&raised));
            raised.reclaimed
        }
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            panic!(
                "挂上之后准入上限高于 F、推得出一串抬 F（阳性对照）：F {floor:?}、上限 {ceiling:?}"
            )
        }
        Err(refusal) => panic!("挂上之后抬 F 到准入上限做得成：{refusal:?}"),
    };
    assert!(
        revived.iter().any(|revived_placement| {
            reclaimed_by_the_raise
                .iter()
                .any(|reclaimed| reclaimed.slot == revived_placement.slot)
        }),
        "这一串抬 F 回收了复活的数据单元（阳性对照：回收时补隔离那一处走到了），回收的 {reclaimed_by_the_raise:?}"
    );
    assert!(
        root_is_in_the_ring(&pool.image(), recorded.key),
        "推过那一串抬 F 时 R 还在根环里"
    );
    let free_among_the_revived_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &revived);
    assert!(
        free_among_the_revived_after_the_raise.is_empty(),
        "推过抬 F 那一刻 R 还在根环里：复活的数据单元覆盖的槽一个都不空闲（那一串回收时照 C 那一圈补了隔离），空闲的 {free_among_the_revived_after_the_raise:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
        0,
        "推过抬 F 之后没有一个槽同时算进「已分配」与第九项"
    );
    let mut overwrites = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_RECORDED_PUBLISH_LEAVES_THE_RING_AT_MOST；跨轮只带计数；出口：R 离开根环，
    // 被拒、双扣、复活的槽空闲、checker 红都当场判红（panic）。
    while root_is_in_the_ring(&pool.image(), recorded.key) {
        assert!(
            overwrites < OVERWRITES_UNTIL_THE_RECORDED_PUBLISH_LEAVES_THE_RING_AT_MOST,
            "{OVERWRITES_UNTIL_THE_RECORDED_PUBLISH_LEAVES_THE_RING_AT_MOST} 次覆盖写之内 R 那一槽写掉、R 离开根环"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1));
        overwrites += 1;
        assert_eq!(
            slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
            0,
            "第 {overwrites} 次覆盖写之后没有一个槽同时算进「已分配」与第九项"
        );
        if root_is_in_the_ring(&pool.image(), recorded.key) {
            let free_among_the_revived = free_slots_covered_by(&pool.session().allocator, &revived);
            assert!(
                free_among_the_revived.is_empty(),
                "第 {overwrites} 次覆盖写之后 R 还在根环里：复活的数据单元覆盖的槽一个都不空闲，空闲的 {free_among_the_revived:?}"
            );
        }
        images.hand_out(
            &format!("第 {overwrites} 次覆盖写之后被抛弃根引用的单元一个都没复用"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
}

/// `placements` 里每个落点覆盖的槽（起点槽起、跨度那么多槽），按落点的次序。
pub fn slots_covered_by(placements: &[PlacementOnDevice]) -> Vec<(DeviceIdentity, SlotNumber)> {
    placements
        .iter()
        .flat_map(|placement| {
            (placement.slot.0..placement.slot.0 + placement.span)
                .map(move |slot| (placement.device, SlotNumber(slot)))
        })
        .collect()
}

/// `placements` 里每个落点覆盖的槽里此刻隔离位置着的那几个（盘、槽），按落点的次序。
pub fn isolated_slots_covered_by(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
) -> Vec<(DeviceIdentity, SlotNumber)> {
    slots_covered_by(placements)
        .into_iter()
        .filter(|(device, slot)| {
            allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == *device)
                .expect("池里有这块盘")
                .is_isolated(*slot)
        })
        .collect()
}
