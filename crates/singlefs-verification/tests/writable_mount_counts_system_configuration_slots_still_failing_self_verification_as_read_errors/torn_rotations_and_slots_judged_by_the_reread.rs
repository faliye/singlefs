//! 撕裂的系统配置轮换与按重读判的槽：第一个文件版本之后两块盘都撕裂的落在系统配置冲突格里拒，只在第一遍读自证不过的重读之后放行，第一遍读得好、重读自证不过的照重读判、拒。

use super::*;

/// 撕裂轮换那一态（E205 第二段 `slots_per_region=4 segment=11 kind=publish_end_rotation state=[torn,torn]`）：mkfs 之后第一次可写挂载、
/// 接着新池新建文件的写流，第 11 段两写都撕裂、之前各段整段持久。交回那一态的镜像。
fn torn_rotation_state(parameters: &MakeFilesystemParameters) -> MemoryPool {
    let mut formatted = empty_sparse_devices();
    make_filesystem(parameters, &mut formatted).expect("mkfs");
    let base = memory_pool_of_sparse_devices(&formatted, |device| device);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = formatted
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    assert!(
        current.file_version().is_none(),
        "前提：mkfs 之后第一次可写挂载交回的那一版还没有文件版本"
    );
    let content = file_content_of(1);
    let first_file_version = publish_first_file(
        &mut PoolWriter::new(parameters, devices.as_mut_slice()),
        &mut allocator,
        current.root(),
        FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        },
        output.instance,
        current.record_bytes(),
    )
    .expect("新池新建文件");
    assert_eq!(
        (
            first_file_version.root.instance,
            first_file_version.root.checkpoint_txg
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(TXG_OF_THE_PUBLISH_BEFORE_THE_TORN_ROTATION)
        ),
        "前提：第一个文件版本是实例 1 的 txg 3"
    );
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let torn_segment = &segments[SEGMENT_OF_THE_TORN_ROTATION_STATE];
    assert_eq!(
        torn_segment
            .iter()
            .map(|index| (writes[*index].kind, writes[*index].device))
            .collect::<Vec<_>>(),
        vec![
            (StepKind::SystemConfigurationSlot, DeviceIdentity(0)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(1)),
        ],
        "前提：第 11 段是两块盘各一写的系统配置轮换"
    );
    let write_before_the_segment = &writes[torn_segment[0] - 1];
    assert_eq!(
        (
            write_before_the_segment.kind,
            root_identity_of_write(write_before_the_segment)
        ),
        (
            StepKind::RootRecordFua,
            (
                InstanceGeneration(1),
                CheckpointTxg(TXG_OF_THE_PUBLISH_BEFORE_THE_TORN_ROTATION)
            )
        ),
        "前提：那一段是第一个文件版本（实例 1 txg 3）那次发布的根槽写之后的轮换（发布末尾的轮换）"
    );
    let mut image = base;
    for segment in &segments[..SEGMENT_OF_THE_TORN_ROTATION_STATE] {
        image.apply_writes(
            &segment
                .iter()
                .map(|index| writes[*index].clone())
                .collect::<Vec<_>>(),
        );
    }
    for index in torn_segment {
        let write = &writes[*index];
        let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
        let old = PoolReader::read(&image, write.device, write.offset, length).expect("写落在盘内");
        let torn = torn_image_of_in_place_overwrite(&old, &write.contents);
        image
            .devices
            .get_mut(&write.device)
            .expect("有这块盘")
            .write(write.offset, &torn);
    }
    image
}

/// 承诺内单故障（一次断电撕了第一个文件版本发布末尾那一段轮换的两写）：设备没报读错、每块盘都还有一槽自证过、坏的两槽都读得出而自证不过——
/// C596 已定的冲突格（D22（单元原子性怎么合成） 已定项 8 射程与 C596 写明这一形不拒可写）。乙入判不出照条款拒可写、盘上逐字节不变，
/// 只读挂载照常（只剩只读挂载，C596 那一形）；挂载透明判官把这一态当没注读故障的崩溃态判这一拒合法。
#[test]
fn torn_rotation_on_both_devices_after_the_first_file_version_is_refused_in_the_system_configuration_conflict_cell(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_TORN_ROTATION_STATE);
    let image = torn_rotation_state(&parameters);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let slots = system_configuration_slots_of_one_device_read_once(
            &image,
            device,
            u64::from(FIXED_STRUCTURE_SLOT_SPACING),
            &FILESYSTEM_IDENTIFIER,
        );
        assert_eq!(
            (
                slots.slots_reporting_a_read_error,
                slots.self_verified.len()
            ),
            (0, 1),
            "前提（C596 冲突格）：盘 {device:?} 设备没报读错、两槽里恰好一槽自证过"
        );
        let slots_failing_self_verification = (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
            .filter(|slot_index| {
                let bytes = PoolReader::read(
                    &image,
                    device,
                    DeviceOffsetInBytes(slot_index * u64::from(FIXED_STRUCTURE_SLOT_SPACING)),
                    slot_bytes,
                )
                .expect("系统配置槽读得到");
                matches!(
                    SystemConfiguration::parse_slot(&bytes),
                    Err(SystemConfigurationSlotRefusal::NotSelfDescribing)
                )
            })
            .count();
        assert_eq!(
            slots_failing_self_verification, 1,
            "前提（C596 冲突格）：盘 {device:?} 坏的那一槽是撕裂（读得出而自证不过）"
        );
    }
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "两块盘各撕一槽、没有一块盘两槽都自证过：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    mount_read_only(&image).expect("只读挂载照常");
    let Err(refusal) = &outcome else {
        panic!("上面判过是拒");
    };
    let fields = read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
            &image,
            &FILESYSTEM_IDENTIFIER,
            fields.report.effective_root,
            refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在这一态上复算乙入判不出为真、c_见证 跳过撕裂的槽之后判据不为真（C670 那一条不成立）：这一拒合法"
    );
}

/// 重读一次：两块盘最新那一槽第一遍读自证不过（读得出、翻了一个字节），「重读一次」之前撤掉 ⇒ 第一遍判乙入判不出、重读那一遍判得出，
/// 照常可写挂载、取到实例 2，读阶段在重读那一遍判完（这一拒只在重读之后仍是那一形时才拒，不在第一遍读到就拒）。
#[test]
fn system_configuration_slots_failing_self_verification_only_on_the_first_read_let_the_mount_through_on_the_reread(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let faults = newest_system_configuration_slot_of_each_device_read_bad(
        &image,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &faults, &faults_lifted);
    let mut lift_the_faults = || faults_lifted.set(true);
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_the_faults),
    )
    .expect("重读那一遍读得出：照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    let ReadStageSettled::OnTheOneReread { first_read, reread } = mounted.output.rereads.read_stage
    else {
        panic!(
            "读阶段要在重读那一遍判完：得到 {:?}",
            mounted.output.rereads.read_stage
        );
    };
    assert_eq!(
        first_read.witness.comparison,
        WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots,
        "第一遍：两块盘最新那一槽都自证不过，乙入判不出"
    );
    assert!(
        !reread.witnesses_a_publish_newer_than_the_selected_version(),
        "重读那一遍判得出、判据为假：{reread:?}"
    );
}

/// 重读粒度：读阶段那一次重读是整个读阶段重做一遍、只按重读那一遍的读数判。两块盘世代号最大的那一槽第一遍读自证不过（翻一个字节），
/// 较旧那一槽第一遍读好；「重读一次」之前撤掉第一遍的故障、改让较旧那一槽读得出而自证不过 ⇒ 第一遍乙入判不出为真，重读那一遍每块盘
/// 仍只有一槽自证过、仍是乙入判不出 ⇒ 照重读那一遍判、拒可写，盘上逐字节不变（只重读第一遍坏的那几槽时那几槽重读读好、会放行）；
/// 挂载透明判官在重读那一遍的盘面上判这一拒合法。
#[test]
fn system_configuration_slots_good_on_the_first_read_and_failing_self_verification_on_the_reread_are_judged_by_the_reread_and_refused(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let faults_on_the_first_read = newest_system_configuration_slot_of_each_device_read_bad(
        &image,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
    let faults_from_the_one_reread_on: Vec<ReadFault> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| ReadFault {
            device,
            offset_in_bytes: older_system_configuration_slot(&image, device),
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
            kind: ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        })
        .collect();
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults_replaced_at_the_one_reread(
        &image,
        &faults_on_the_first_read,
        &faults_from_the_one_reread_on,
        &faults_lifted,
    );
    let hook_calls = Cell::new(0u32);
    let mut replace_the_faults = || {
        hook_calls.set(hook_calls.get() + 1);
        faults_lifted.set(true);
    };
    let outcome = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut replace_the_faults),
    );
    assert_eq!(
        hook_calls.get(),
        1,
        "读阶段第一遍判真、重读了一次（重读之前调了一次钩子）"
    );
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "第一遍好的槽在重读那一遍读坏：照重读那一遍判、仍是乙入判不出，拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    let image_after = memory_pool_of_sparse_devices(&devices, |device| &device.inner);
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    assert_eq!(
        verdict_of_the_refusal_on(
            &ImageWithReadFaults {
                image: &image,
                faults: &faults_from_the_one_reread_on,
            },
            &outcome
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在重读那一遍的盘面上复算乙入判不出为真：这一拒合法"
    );
}
