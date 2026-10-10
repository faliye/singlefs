// ===== 探针（正推腿自己的，不入库）：R 是一次管理员回退（向前发布），回退目标 R_old 的数据单元在 P 的账里已释放、被 R 复活 =====
fn record_one_rollback_after_overwrites(
    overwrites_before: u64,
) -> (RecordedPublish, Vec<PlacementOnDevice>) {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    let old_root = *pool.session().current.root();
    let old_data_pointers = pool.current_file_version().data_pointers.clone();
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let p_locations: Vec<(DeviceIdentity, u64)> = pool
        .current_file_version()
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| (location.device, location.slot.0))
        .collect();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    let mut session = pool.session.take().expect("会话");
    {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("带文件");
        };
        let rolled = roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            RollbackTarget {
                instance: old_root.instance,
                checkpoint_txg: old_root.checkpoint_txg,
            },
        )
        .expect("管理员回退做得成");
        eprintln!("PROBE rolled back: resurrected={} released={}", rolled.resurrected_user_visible_copies.len(), rolled.released_user_visible_units.len());
    }
    pool.session = Some(session);
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indices.len(), 1, "回退那次发布只写一条根");
    let revived: Vec<PlacementOnDevice> = old_data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .filter(|location| !p_locations.contains(&(location.device, location.slot.0)))
        .map(|location| PlacementOnDevice {
            device: location.device,
            slot: location.slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect();
    (
        RecordedPublish {
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
    )
}

#[test]
fn probe_units_revived_by_an_unwitnessed_rollback_publish_stay_through_writes_raising_the_floor() {
    let (recorded, revived) = record_one_rollback_after_overwrites(4);
    assert!(!revived.is_empty(), "回退目标的数据单元在 P 里不引用、被复活");
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded));
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("可写挂载做得成");
    eprintln!("PROBE mounted on {:?}, previous_key {:?}, r key {:?}", (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg), recorded.previous_key, recorded.key);
    let borrowed = mounted
        .allocator
        .root_ring_occupancy()
        .expect("根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        );
    eprintln!("PROBE borrowed for R = {:?} entries; revived n = {}", borrowed.as_ref().map(Vec::len), revived.len());
    let revived_not_borrowed: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|r| {
            !borrowed.as_ref().is_some_and(|b| {
                b.iter().any(|x| x.device == r.device && x.slot == r.slot)
            })
        })
        .collect();
    eprintln!("PROBE revived not in C[R]: {} of {}", revived_not_borrowed.len(), revived.len());
    lift.lift();
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content = recorded.pool.content_of_each_root.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned().expect("content");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    let mut overwrites = 0;
    let mut raised_while_r_in_ring = None;
    loop {
        assert!(overwrites < 60, "60 次内应当推出抬 F");
        let r_in_ring = root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
        overwrites += 1;
        if !published.floor_raises.is_empty() {
            raised_while_r_in_ring = Some(r_in_ring);
            break;
        }
    }
    let free_revived = free_slots_covered_by(&pool.session().allocator, &revived);
    eprintln!("PROBE after raise (after {overwrites} overwrites; R in ring before the step: {raised_while_r_in_ring:?}): free revived slots = {}", free_revived.len());
    let mut violations_after = Vec::new();
    for n in 1..=3 {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity()).expect("顺序写");
        let image = pool.image();
        let in_ring = root_is_in_the_ring(&image, recorded.key);
        let v = violations_of_unit_reuse_on(&image);
        eprintln!("PROBE after seq write {n}: R in ring={in_ring} violations={v:?}");
        violations_after.push((in_ring, v));
    }
    assert!(free_revived.is_empty(), "复活的数据单元在抬 F 之后一槽都不空闲：空闲 {free_revived:?}");
}
