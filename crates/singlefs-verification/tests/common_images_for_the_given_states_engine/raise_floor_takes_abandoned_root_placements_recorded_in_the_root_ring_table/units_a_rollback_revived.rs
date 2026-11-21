//! R 是一次管理员回退：回退复活的单元在 R 的账、谱系片、写的单元或回退目标的根槽与账读回全零时照样留着，推过抬 F 不红 I-7.4。

use super::*;

/// 正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-model/probe_rollback_revived.tail.rs` 那一形之前覆盖写几次（探针原样 4 次）。
pub const OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH: u64 = 4;

/// 回退复活那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；探针原样 3 次）。
pub const SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK: usize = 3;

/// 录下来的一次管理员回退 R（[`record_one_rollback_to_the_first_file_after_overwrites`]）。
pub struct RecordedRollback {
    pub recorded: RecordedPublish,
    /// R 复活的数据单元的落点：R_old 的文件数据指针里 P 不引用的那几条位置条目（跨度按数据单元宽折成槽数）。
    pub revived: Vec<PlacementOnDevice>,
    /// 回退目标 R_old（第一个文件那一版）的根。
    pub rollback_target: RootRecord,
    /// R_old 那一版的账里仍分配着的（盘、起点槽）：起池之后、覆盖写之前会话分配器里的记录（R_old 那次发布写下的账）。
    pub allocated_in_the_account_of_the_rollback_target: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 正推探针的 `record_one_rollback_after_overwrites`：起池（384 槽那一档、第一个文件，那一版是 R_old），覆盖写 `overwrites_before` 次得 P，
/// 再经会话做一次管理员回退到 R_old、录这一次发布得 R。交回录下来的 R、R 复活的单元的落点、R_old 的根与它账里仍分配着的落点
/// （都从录下来的会话算，不经被测的方法）。
pub fn record_one_rollback_to_the_first_file_after_overwrites(
    overwrites_before: u64,
) -> RecordedRollback {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    let root_of_the_first_file = *pool.session().current.root();
    let data_pointers_of_the_first_file = pool.current_file_version().data_pointers.clone();
    let allocated_in_the_account_of_the_rollback_target: Vec<(DeviceIdentity, SlotNumber)> = pool
        .session()
        .allocator
        .records()
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device, record.slot))
        .collect();
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let locations_referenced_by_the_previous_version: Vec<(DeviceIdentity, SlotNumber)> = pool
        .current_file_version()
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| (location.device, location.slot))
        .collect();
    let base = pool.image();
    let operations_before_the_recorded_publish = stream.operation_count();
    let mut session = pool.session.take().expect("会话");
    {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("第一个文件之后现行那一版带文件");
        };
        let current = Arc::make_mut(current);
        roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            RollbackTarget {
                instance: root_of_the_first_file.instance,
                checkpoint_txg: root_of_the_first_file.checkpoint_txg,
            },
        )
        .expect("回退到第一个文件那一版做得成（它在根环里、按实例表有效）");
    }
    pool.session = Some(session);
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_recorded_publish..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indices.len(), 1, "回退那次发布只写一条根");
    let revived: Vec<PlacementOnDevice> = data_pointers_of_the_first_file
        .iter()
        .flat_map(|pointer| pointer.locations)
        .filter(|location| {
            !locations_referenced_by_the_previous_version
                .contains(&(location.device, location.slot))
        })
        .map(|location| PlacementOnDevice {
            device: location.device,
            slot: location.slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect();
    RecordedRollback {
        recorded: RecordedPublish {
            content: previous_content.clone(),
            pool,
            base,
            writes,
            root_index: root_indices[0],
            key: (root.instance, root.checkpoint_txg),
            previous_key: (previous_root.instance, previous_root.checkpoint_txg),
            previous_content,
        },
        revived,
        rollback_target: root_of_the_first_file,
        allocated_in_the_account_of_the_rollback_target,
    }
}

/// 回退复活（正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-output.md` K2-1、探针 `probe_rollback_revived.tail.rs`；判决
/// `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形）：第一个文件之后覆盖写 4 次得 P，经会话管理员回退到
/// 第一个文件那一版（R_old）、录这一次发布得 R，崩在 R 的根槽落盘之后、轮换之前；可写挂载那一刻 R 的账（树表、中央映射树根、分配记录树根）
/// 读回全零而记录齐：择根退到 P，走丙。R 复活的数据单元在 P 的账里是已释放，丙那一截滤掉它们；补法认出 R 是回退（记录点名的谱系片里有
/// rollback_txg 等于 R 的那条谱系记录），把 R_old 账里仍分配、P 的账里已释放的落点并进 C 里 R 那一槽：复活的落点都在里面。
/// 挂上之后撤故障，会话覆盖写到准入推出第一串抬 F（推的那一刻 R 还在根环里），推过那一刻复活的落点一槽都不空闲；再顺序写 3 次，
/// 每一步之后池级 checker 不红 I-7.4 与 I-4.8。判别力：撤掉回退复活那一截（变异「回退复活 丙那一支不并回退目标账里仍分配、
/// 现行账里已释放的落点」）时 C 那一槽缺复活的落点那一句先红；去掉那一句，推过抬 F 复活的槽空闲、顺序写之后 I-7.4 红（正推探针原样）。
pub fn run_the_scenario_where_units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let RecordedRollback {
        recorded, revived, ..
    } = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    assert!(
        !revived.is_empty(),
        "第一个文件那一版的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded));
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
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
    let borrowed_for_the_recorded_publish = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let revived_not_borrowed: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !borrowed_for_the_recorded_publish.iter().any(|borrowed| {
                borrowed.device == revived_placement.device
                    && borrowed.slot == revived_placement.slot
            })
        })
        .collect();
    assert!(
        revived_not_borrowed.is_empty(),
        "挂载那一刻 C 里 R 那一槽并上了回退目标账里仍分配、P 的账里已释放的落点（回退复活）：复活的都在里面，缺 {revived_not_borrowed:?}"
    );
    lift.lift();
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
    // 同一个说明（「覆盖写之后」）在循环里交好几张，名字带上这是第几张，一个场景里不重名
    let mut images_handed_out_after_the_mount = 0_usize;
    let mut hand_out_expecting_nothing_of_the_recorded_publish_reused =
        |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
            images_handed_out_after_the_mount += 1;
            images.hand_out(
                &format!("第 {images_handed_out_after_the_mount} 张，{after_what}：被抛弃根引用的单元一个都没复用"),
                || pool_after_the_step.image(),
                PoolCheckerExpectation::only_the_named_invariants()
                    .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
            );
        };
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let root_of_the_recorded_publish_was_in_the_ring =
            root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        hand_out_expecting_nothing_of_the_recorded_publish_reused(&pool, "覆盖写之后");
        if !published.floor_raises.is_empty() {
            assert!(
                root_of_the_recorded_publish_was_in_the_ring,
                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
            );
            break;
        }
    }
    let free_among_the_revived_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &revived);
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("推过抬 F 之后第 {sequential_write_number} 次顺序写被拒：{refusal:?}")
            });
        hand_out_expecting_nothing_of_the_recorded_publish_reused(&pool, "推过抬 F 之后顺序写");
    }
    assert!(
        free_among_the_revived_after_the_raise.is_empty(),
        "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
    );
}

/// 认不出时宽护（tri；
/// 判法照 tri 臂）
/// 那四形共用的走法：回退复活那一形（覆盖写 4 次得 P、回退到第一个文件那一版录 R），可写挂载那一刻 `ranges` 读回全零（R 的账另加一处，
/// 让认回退或算复活集那一步走不成）：择根退到 P，走丙。挂上之后撤故障，会话覆盖写到准入推出第一串抬 F（推的那一刻 R 还在根环里），
/// 再顺序写 3 次，每一步之后池级 checker 不红 I-7.4 与 I-4.8；全程过了之后再核 C 里 R 那一槽有复活的全部落点、推过抬 F 那一刻复活的槽不空闲。
/// 次序是故意的：撤掉宽护（这几形照丙原样交空）时先红的是写到半途的 I-7.4（复活的单元被回收、写掉），不是借到那一句。
pub fn units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
    recorded_rollback: &RecordedRollback,
    ranges: &RangesReadingBackAllZero,
) {
    let recorded = &recorded_rollback.recorded;
    let revived = &recorded_rollback.revived;
    assert!(
        !revived.is_empty(),
        "第一个文件那一版的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, ranges, &lift);
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
    let borrowed_for_the_recorded_publish = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    lift.lift();
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
    // 同一个说明（「覆盖写之后」）在循环里交好几张，名字带上这是第几张，一个场景里不重名
    let mut images_handed_out_after_the_mount = 0_usize;
    let mut hand_out_expecting_nothing_of_the_recorded_publish_reused =
        |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
            images_handed_out_after_the_mount += 1;
            images.hand_out(
                &format!("第 {images_handed_out_after_the_mount} 张，{after_what}：被抛弃根引用的单元一个都没复用"),
                || pool_after_the_step.image(),
                PoolCheckerExpectation::only_the_named_invariants()
                    .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
            );
        };
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let root_of_the_recorded_publish_was_in_the_ring =
            root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        hand_out_expecting_nothing_of_the_recorded_publish_reused(&pool, "覆盖写之后");
        if !published.floor_raises.is_empty() {
            assert!(
                root_of_the_recorded_publish_was_in_the_ring,
                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
            );
            break;
        }
    }
    let free_among_the_revived_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, revived);
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("推过抬 F 之后第 {sequential_write_number} 次顺序写被拒：{refusal:?}")
            });
        hand_out_expecting_nothing_of_the_recorded_publish_reused(&pool, "推过抬 F 之后顺序写");
    }
    let revived_not_borrowed: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !borrowed_for_the_recorded_publish.iter().any(|borrowed| {
                borrowed.device == revived_placement.device
                    && borrowed.slot == revived_placement.slot
            })
        })
        .collect();
    assert!(
        revived_not_borrowed.is_empty(),
        "挂载那一刻 C 里 R 那一槽退到现行账里全部已释放的落点（宽护）：复活的都在里面，缺 {revived_not_borrowed:?}"
    );
    assert!(
        free_among_the_revived_after_the_raise.is_empty(),
        "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
    );
}

/// R 写出的谱系片（打包记录类型 5 的打包单元）的写：从录下来的写的字节解，不经被测的方法。
pub fn lineage_unit_writes_of(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            write
                .bytes()
                .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
                .is_some_and(|packed| {
                    packed.identity.record_type == singlefs_core::unit::PACKED_TYPE_LINEAGE
                })
        })
        .collect()
}

/// `image` 的根环里 (实例, txg) 为 `key`、自证过的那条根（盘上那一份：带着它的树表、分配记录树根等指针）。
///
/// # Panics
/// `image` 上择不到系统配置，或根环里没有自证过的这条根。
pub fn root_read_from_the_ring_of(
    image: &MemoryPool,
    key: (InstanceGeneration, CheckpointTxg),
) -> RootRecord {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let sizes = &configuration.immutable.sizes;
    singlefs_core::recovery::every_root_ring_slot(sizes)
        .into_iter()
        .find_map(|ring_slot| {
            if let singlefs_core::recovery::RootRingSlotReading::SelfVerified(root) =
                singlefs_core::recovery::read_root_ring_slot(
                    image,
                    &configuration.immutable.region_devices,
                    sizes,
                    &configuration.immutable.filesystem_identifier,
                    ring_slot,
                )
            {
                ((root.instance, root.checkpoint_txg) == key).then_some(root)
            } else {
                None
            }
        })
        .expect("根环里有自证过的这条根")
}

/// 挂载那一刻现行账（所选那一版 P 的账）里记成已释放、挂上之后还记着的（盘、起点槽）：可写挂载交回的分配器里释放代不高于 P 的 txg 的那几条
/// （释放代高于它的是这次挂载自己的写行、暖机释放的，挂载那一刻还分配着）。
pub fn released_in_the_account_of_the_selected_version(
    mounted: &singlefs_core::mount::Mounted,
) -> std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> {
    let selected_version_txg = mounted.output.effective_root.checkpoint_txg;
    mounted
        .allocator
        .records()
        .iter()
        .filter(|record| record.is_released && record.generation <= selected_version_txg)
        .map(|record| (record.device, record.slot))
        .collect()
}

/// tri 第一种「不知道是不是回退」：R 的账与 R 写的谱系片都读回全零（正推 `rlin`）——R 的记录点名的打包单元有一片读不出，认不出是不是回退。
pub fn run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let recorded = &recorded_rollback.recorded;
    let lineage_unit_writes = lineage_unit_writes_of(recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（D18（块里携带什么信息） 已定项 24 第 5 条；阳性对照：故障有处可打）"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&account_writes_of(recorded))
        .with_the_writes(&lineage_unit_writes);
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// tri 第一种的另一形：R 这次发布写的全部单元都读回全零（攻方 s2 `rall`），记录齐。
pub fn run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded_rollback.recorded.unit_writes());
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// tri 第二种「是回退、复活集算不出」：认得出回退（谱系片读得出），而 R_old 的根槽读回全零（正推 `rold`）——根环里没读出 R_old。
pub fn run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let recorded = &recorded_rollback.recorded;
    let rollback_target = recorded_rollback.rollback_target;
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&account_writes_of(recorded))
        .with_the_root_slot_of(
            &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
            (rollback_target.instance, rollback_target.checkpoint_txg),
        );
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// tri 第二种的另一形：认得出回退、根环里读得出 R_old，而 R_old 的树表与分配记录树根读回全零（正推 `roldacct`）——R_old 的账读不出。
pub fn run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let rollback_target = root_read_from_the_ring_of(
        &recorded_rollback
            .recorded
            .crash_image_with_the_root_persisted_and_the_rotation_lost(),
        (
            recorded_rollback.rollback_target.instance,
            recorded_rollback.rollback_target.checkpoint_txg,
        ),
    );
    let mut ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&account_writes_of(&recorded_rollback.recorded));
    for (pointer, unit) in [
        (
            rollback_target.tree_table,
            singlefs_core::transaction::TransactionUnit::TreeTable,
        ),
        (
            rollback_target.allocation_record_tree_root,
            singlefs_core::transaction::TransactionUnit::AllocationTree,
        ),
    ]
    .into_iter()
    .filter(|(pointer, _)| *pointer != NodePointer::empty_root())
    {
        for location in pointer.locations {
            ranges.0.push((
                location.device,
                location.slot.to_device_offset().0,
                unit.span_slots() * singlefs_format::SLOT_BYTES,
            ));
        }
    }
    assert!(
        ranges.0.len() > account_writes_of(&recorded_rollback.recorded).len(),
        "R_old（第一个文件那一版）的树表或分配记录树根有落点（阳性对照：故障有处可打）"
    );
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// 两块盘上分配位与隔离位同时置着的槽数之和：D28（挂载期承诺量） 已定项 1 第九项与「已分配」各扣一次的那几个。
pub fn slots_counted_both_as_allocated_and_as_isolated(allocator: &PoolAllocator) -> u64 {
    allocator
        .devices
        .iter()
        .map(singlefs_core::allocator::DeviceFreeMap::slots_counted_both_as_allocated_and_as_isolated)
        .sum()
}

/// 两块盘隔离的槽数之和（D28（挂载期承诺量） 已定项 1 第九项那一项）。
pub fn isolated_slots_of(allocator: &PoolAllocator) -> u64 {
    allocator
        .devices
        .iter()
        .map(singlefs_core::allocator::DeviceFreeMap::isolated_slots)
        .sum()
}
