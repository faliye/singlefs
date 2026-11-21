//! 1536 槽两块盘上先释放一大批再回退、R 建一个 inode：宽护并进现行账里全部已释放的落点时一槽不双扣，之后的写放得下。

use super::*;

/// 正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 1.3 节那一形之前覆盖写几次（探针 `S2_PREFIXES` 原样 40 次）。
pub const OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK: u64 = 40;

/// 那一形回退到倒数第几版（探针 `S2_RS=b6`：录的版本表里倒数第 1 + 6 版）。
pub const VERSIONS_BACK_OF_THE_ROLLBACK_TARGET: usize = 6;

/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次得 P，经会话管理员回退到之前写过的倒数第 1 + `versions_back` 版（R_old），
/// 录这一次发布得 R（正推探针的 `record` 那一种 `b<数>`）。交回录下来的 R、R 复活的数据单元的落点（R_old 的文件数据指针里 P 不引用的那几条位置条目）、
/// R_old 的根与它账里仍分配着的落点（都从录下来的会话算，不经被测的方法）。
pub fn record_one_rollback_after_overwrites(
    overwrites_before: u64,
    versions_back: usize,
) -> RecordedRollback {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    // 每一版：根、文件数据指针的位置条目（盘、槽）、账里仍分配着的（盘、起点槽）。
    let version_of = |pool_at_this_version: &PoolUnderTest<
        RecordingBlockDevice<SparseBlockDevice>,
    >|
     -> RolledBackVersion {
        RolledBackVersion {
            root: *pool_at_this_version.session().current.root(),
            data_locations: pool_at_this_version
                .current_file_version()
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| (location.device, location.slot))
                .collect(),
            allocated_in_its_account: pool_at_this_version
                .session()
                .allocator
                .records()
                .iter()
                .filter(|record| !record.is_released)
                .map(|record| (record.device, record.slot))
                .collect(),
        }
    };
    let mut versions_written = vec![version_of(&pool)];
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下（准入推抬 F 也做得成）");
        versions_written.push(version_of(&pool));
    }
    let target_version = &versions_written[versions_written.len() - 1 - versions_back];
    let rollback_target = target_version.root;
    let locations_referenced_by_the_previous_version =
        &versions_written[versions_written.len() - 1].data_locations;
    let revived: Vec<PlacementOnDevice> = target_version
        .data_locations
        .iter()
        .filter(|location| !locations_referenced_by_the_previous_version.contains(location))
        .map(|(device, slot)| PlacementOnDevice {
            device: *device,
            slot: *slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect();
    let allocated_in_the_account_of_the_rollback_target =
        target_version.allocated_in_its_account.clone();
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
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
                instance: rollback_target.instance,
                checkpoint_txg: rollback_target.checkpoint_txg,
            },
        )
        .expect("回退到倒数第 7 版做得成（它在根环里、按实例表有效）");
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
        rollback_target,
        allocated_in_the_account_of_the_rollback_target,
    }
}

/// [`record_one_rollback_after_overwrites`] 录的每一版。
pub struct RolledBackVersion {
    pub root: RootRecord,
    pub data_locations: Vec<(DeviceIdentity, SlotNumber)>,
    pub allocated_in_its_account: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 单元区 1536 槽那一档每块盘的字节数：384 槽那一档的盘宽再加 1536 − 384 槽（攻方 `device_bytes_with_unit_area_slots(1536)` 同一个数）。
pub fn device_bytes_of_the_1536_slot_devices() -> u64 {
    const UNIT_AREA_SLOTS_OF_THE_1536_SLOT_DEVICES: u64 = 1536;
    const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;
    device_bytes_of_the_384_slot_small_devices()
        + (UNIT_AREA_SLOTS_OF_THE_1536_SLOT_DEVICES - UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES)
            * singlefs_format::SLOT_BYTES
}

/// 攻方 `research/prompts/m3-batch10z-lm-code-r1-opus-model/runs-summary/mount-counts-b11.txt` 那一形的前缀：顺序写几次（`S` 22 次）。
pub const SEQUENTIAL_WRITES_OF_TWENTY_DATA_UNITS_BEFORE_THE_RECORDED_PUBLISH: usize = 22;

/// 每次顺序写几个数据单元（攻方 `S`）。
pub const DATA_UNITS_OF_EACH_SEQUENTIAL_WRITE_BEFORE_THE_RECORDED_PUBLISH: usize = 20;

/// 那一形前缀末尾回退到倒数第几版（攻方 `b8`）。
pub const VERSIONS_BACK_OF_THE_ROLLBACK_BEFORE_THE_RECORDED_PUBLISH: usize = 8;

/// 挂上之后覆盖写几次、再顺序写几次（每次两个数据单元）：攻方 b9 那一串 30 步全被拒，这里 40 步。
pub const OVERWRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES: usize = 30;

pub const SEQUENTIAL_WRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES: usize = 10;

/// 挂上之后会话做的一步写。
#[derive(Clone, Copy, Debug)]
pub enum UserWriteAfterTheMount {
    Overwrite,
    SequentialWriteOfTwoDataUnits,
}

/// 攻方那一形（1536 槽两块盘）：起池、第一个文件之后顺序写 22 次（每次 20 个数据单元，先释放一大批），回退到倒数第 9 版，录一次建 inode 得 R。
pub fn record_one_inode_creation_after_releasing_a_large_batch_on_the_1536_slot_devices(
) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_1536_slot_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    let mut roots_written = vec![*pool.session().current.root()];
    for _sequential_write in 0..SEQUENTIAL_WRITES_OF_TWENTY_DATA_UNITS_BEFORE_THE_RECORDED_PUBLISH {
        pool.sequential_write(
            DATA_UNITS_OF_EACH_SEQUENTIAL_WRITE_BEFORE_THE_RECORDED_PUBLISH
                * singlefs_core::unit::data_unit_payload_capacity(),
        )
        .expect("1536 槽两块盘上这几次顺序写都落得下");
        roots_written.push(*pool.session().current.root());
    }
    let rollback_target = roots_written
        [roots_written.len() - 1 - VERSIONS_BACK_OF_THE_ROLLBACK_BEFORE_THE_RECORDED_PUBLISH];
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
                instance: rollback_target.instance,
                checkpoint_txg: rollback_target.checkpoint_txg,
            },
        )
        .expect("回退到倒数第 9 版做得成（它在根环里、按实例表有效）");
    }
    pool.session = Some(session);
    pool.content_of_the_current_version = pool
        .content_of_each_root
        .get(&(rollback_target.instance, rollback_target.checkpoint_txg))
        .cloned()
        .expect("回退目标是录的池写过的一版");
    let root_after_the_rollback = *pool.session().current.root();
    pool.note_root_of_the_current_content(&root_after_the_rollback);
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_the_recorded_publish = stream.operation_count();
    pool.create_one_inode().expect("录的那次建 inode 落得下");
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
    assert_eq!(
        root_indices.len(),
        1,
        "录的那一次建 inode 只写一条根（准入够，没推抬 F）"
    );
    RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (
            root_after_the_rollback.instance,
            root_after_the_rollback.checkpoint_txg,
        ),
        previous_content,
    }
}

/// H2 双扣是 H1 卡死的主来源那一形（攻方 `research/prompts/m3-batch10z-lm-code-r1-opus-output.md`「K1-3 打中」两节，`mount-counts-b11.txt`、
/// `wedge-b6-b10.txt` 的 b9）：1536 槽两块盘、先顺序写释放一大批再回退，R 建一个 inode、R 写的全部单元挂载那一刻读回全零（不是回退而认不出，
/// 宽护并进现行账里全部已释放的落点）。挂载那一刻隔离 2242 槽、其中 338 槽两项各扣一次的话，之后每次写都被准入拒、R 一直在根环里。
/// 要求：挂载那一刻、每一步之后都没有一个槽同时算进「已分配」与第九项；覆盖写 30 次、顺序写 10 次一次都不拒；R 那一槽写掉（R 离开根环）；
/// R 还在根环里时 C 里 R 那一槽的槽一个都不空闲，每一步之后池级 checker 不红 I-7.4 与 I-4.8（回收之后补的隔离照罩着）。
pub fn run_the_scenario_where_wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices(
    images: &mut PoolCheckerImages,
) {
    let recorded =
        record_one_inode_creation_after_releasing_a_large_batch_on_the_1536_slot_devices();
    assert!(
        lineage_unit_writes_of(&recorded).is_empty(),
        "建 inode 不写谱系片（R 不是回退：宽护只因为它写的单元读不出）"
    );
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes()),
        &lift,
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 写的单元读回全零而记录齐：可写挂载落到 P、做得成");
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
    assert!(
        released_in_the_current_account
            .iter()
            .all(|(device, slot)| {
                borrowed_for_the_recorded_publish
                    .iter()
                    .any(|borrowed| borrowed.device == *device && borrowed.slot == *slot)
            }),
        "认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（阳性对照：宽护走到了）"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（改之前 338 个）"
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
    let steps = std::iter::repeat_n(
        UserWriteAfterTheMount::Overwrite,
        OVERWRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES,
    )
    .chain(std::iter::repeat_n(
        UserWriteAfterTheMount::SequentialWriteOfTwoDataUnits,
        SEQUENTIAL_WRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES,
    ));
    let mut root_of_the_recorded_publish_left_the_ring = false;
    // 迭代上界是步数（40）；跨轮带的是「R 离没离开根环」；出口：被拒、双扣、C 里 R 那一槽有空闲的槽、checker 红都当场判红（panic）。
    for (step_number, user_write) in (1..).zip(steps) {
        let root_of_the_recorded_publish_was_in_the_ring =
            root_is_in_the_ring(&pool.image(), recorded.key);
        let published = match user_write {
            UserWriteAfterTheMount::Overwrite => pool.overwrite(OVERWRITE_BYTES),
            UserWriteAfterTheMount::SequentialWriteOfTwoDataUnits => {
                pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            }
        };
        if let Err(refusal) = published {
            panic!("第 {step_number} 步被拒（改之前每一步都被准入拒）：{refusal:?}");
        }
        assert_eq!(
            slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
            0,
            "第 {step_number} 步之后没有一个槽同时算进「已分配」与第九项"
        );
        if root_of_the_recorded_publish_was_in_the_ring
            && root_is_in_the_ring(&pool.image(), recorded.key)
        {
            let free_among_the_borrowed = free_slots_covered_by(
                &pool.session().allocator,
                &borrowed_for_the_recorded_publish,
            );
            assert!(
                free_among_the_borrowed.is_empty(),
                "第 {step_number} 步之后 R 还在根环里：C 里 R 那一槽的槽一个都不空闲（回收时补了隔离），空闲的 {free_among_the_borrowed:?}"
            );
        } else {
            root_of_the_recorded_publish_left_the_ring = true;
        }
        images.hand_out(
            &format!("第 {step_number} 步之后被抛弃根引用的单元一个都没复用"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
    assert!(
        root_of_the_recorded_publish_left_the_ring,
        "40 步之内 R 那一槽写掉、R 离开根环（改之前 R 一直在环里、隔离到进程结束）"
    );
}
