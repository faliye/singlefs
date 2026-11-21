//! 格式化之后撕裂的获取轮：离线修复之后发布第一个文件版本，离线出路断言创世之后的版本缺末记录、单元写过之后断言创世都拒，尾巴判不出时断言创世的离线出路拒。

use super::*;

/// mkfs 之后第一次可写挂载的写流里第 `segment` 段（取号写的一轮，两块盘各一写）两写都撕裂、之前各段整段持久。
fn torn_acquisition_round_on_a_formatted_pool(
    parameters: &MakeFilesystemParameters,
    segment: usize,
) -> MemoryPool {
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
    mount_writable(parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let torn_segment = &segments[segment];
    assert_eq!(
        torn_segment
            .iter()
            .map(|index| (writes[*index].kind, writes[*index].device))
            .collect::<Vec<_>>(),
        vec![
            (StepKind::SystemConfigurationSlot, DeviceIdentity(0)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(1)),
        ],
        "前提：第 {segment} 段是取号写的一轮，两块盘各一写"
    );
    let mut image = base;
    for earlier in &segments[..segment] {
        image.apply_writes(
            &earlier
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

/// 一次断电撕了 mkfs 之后第一次可写挂载取号写一轮的两写（单故障、设备没报读错）：可写挂载乙入判不出拒，只读挂载答树表 0 条，
/// 离线出路断言第 0 代（mkfs 不写记录，tail 取 mkfs 写的 0）重写撕裂的两槽；之后可写挂载做成、取号照常加一，发得出第一个文件版本，池级 checker 无违例，只读恢复读得出它。
#[test]
fn torn_acquisition_round_on_a_formatted_pool_is_repaired_offline_and_publishes_the_first_file_version(
) {
    for slots_per_region in [4u64, 8, 16] {
        for segment in [0usize, 1] {
            let parameters = pool_parameters(slots_per_region);
            let image = torn_acquisition_round_on_a_formatted_pool(&parameters, segment);
            let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
            assert!(
                refused_as_failing_self_verification_on_both_reads(&outcome),
                "S = {slots_per_region} 段 {segment}：乙入判不出拒"
            );
            assert!(image_after == image, "拒在取号之前");
            assert!(
                matches!(
                    mount_read_only(&image),
                    Err(MountReadOnlyFailure::Open(
                        OpenPoolForReadFailure::TreeTableHasNoEntries
                    ))
                ),
                "S = {slots_per_region} 段 {segment}：只读挂载答树表 0 条"
            );
            let faults_lifted = Rc::new(Cell::new(false));
            let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
            let rewritten = rewrite_system_configuration_slots_for_the_asserted_version(
                &parameters,
                &mut devices,
                AssertedVersion {
                    instance: InstanceGeneration(0),
                    checkpoint_txg: CheckpointTxg(0),
                },
            )
            .unwrap_or_else(|refusal| {
                panic!("S = {slots_per_region} 段 {segment}：离线出路断言第 0 代做成；得到 {refusal:?}")
            });
            assert_eq!(
                (
                    rewritten.journal_tail_written,
                    rewritten.slots_rewritten.len()
                ),
                (0, 2),
                "S = {slots_per_region} 段 {segment}：tail 取 mkfs 写的 0，每块盘重写撕裂的那一槽"
            );
            let Mounted {
                output,
                mut allocator,
                current,
            } = mount_writable(&parameters, &mut devices).unwrap_or_else(|refusal| {
                panic!("S = {slots_per_region} 段 {segment}：离线出路之后可写挂载做成；得到 {refusal:?}")
            });
            assert_eq!(
                output.instance.0,
                rewritten.journal_instance_written.0 + 1,
                "S = {slots_per_region} 段 {segment}：离线出路不加一，取号照常加一"
            );
            let content = file_content_of(1);
            publish_first_file(
                &mut PoolWriter::new(&parameters, devices.as_mut_slice()),
                &mut allocator,
                current.root(),
                FirstFile {
                    content: &content,
                    write_time_seconds: FIRST_WRITE_TIME_SECONDS,
                },
                output.instance,
                current.record_bytes(),
            )
            .unwrap_or_else(|refusal| {
                panic!(
                    "S = {slots_per_region} 段 {segment}：发得出第一个文件版本；得到 {refusal:?}"
                )
            });
            let image_after_the_file_version =
                memory_pool_of_sparse_devices(&devices, |device| &device.inner);
            assert_eq!(
                violated_invariants(&image_after_the_file_version),
                Vec::<&'static str>::new(),
                "S = {slots_per_region} 段 {segment}：池级 checker 无违例"
            );
            assert!(
                matches!(
                    recover_of_the_pool(
                        &image_after_the_file_version,
                        JournalPolicy::Consult,
                        &FILESYSTEM_IDENTIFIER
                    )
                    .outcome,
                    RecoveryOutcome::FileRead { content: read_back, .. } if read_back == content
                ),
                "S = {slots_per_region} 段 {segment}：恢复读得出第一个文件版本"
            );
        }
    }
}

/// 离线出路断言第 0 代之外的版本照拒：D 的根读得出、D 的记录两份都读回全零，断言 D 时写进系统配置的 tail 条款没写，一个字节不写。
#[test]
fn the_offline_way_out_asserting_a_version_after_the_genesis_without_its_last_record_is_still_refused(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = faults_on_the_records_of_the_newest_version(&history);
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&history.image, &faults, &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion { .. })
        ),
        "断言 D：根读得出、末条读不出，拒；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == history.image,
        "拒的时候一个字节不写"
    );
}

/// 断言第 0 代而第 0 代之后写过单元：tail 取得到，`birth_txg` 闸照拒，一个字节不写。
#[test]
fn the_offline_way_out_asserting_the_genesis_after_units_were_written_is_refused_by_the_birth_gate()
{
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::UnitBornAfterTheAssertedVersion { .. })
        ),
        "断言第 0 代：比它新的版本写过单元，拒；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == image,
        "拒的时候一个字节不写"
    );
}

/// mkfs 之后第一次可写挂载整段落盘（取号、写行、暖机）：交回落盘之后的盘面、那次挂载的全部写与它推了几次暖机。
fn image_and_writes_after_the_first_mount_on_a_formatted_pool(
    parameters: &MakeFilesystemParameters,
) -> (MemoryPool, Vec<RetainedWrite>, usize) {
    let mut formatted = empty_sparse_devices();
    make_filesystem(parameters, &mut formatted).expect("mkfs");
    let mut image = memory_pool_of_sparse_devices(&formatted, |device| device);
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
    let mounted = mount_writable(parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    image.apply_writes(&writes);
    (image, writes, mounted.output.warm_up_publishes.len())
}

/// 第一次可写挂载写下的 `kinds` 那几种写的落点，每段读回全零。
fn faults_on_the_writes_of_the_kinds(
    writes: &[RetainedWrite],
    kinds: &[StepKind],
) -> Vec<ReadFault> {
    writes
        .iter()
        .filter(|write| kinds.contains(&write.kind))
        .map(|write| ReadFault {
            device: write.device,
            offset_in_bytes: write.offset.0,
            length_in_bytes: write.length_in_bytes(),
            kind: ReadFaultKind::ReadsBackAllZeros,
        })
        .collect()
}

/// 第一次可写挂载（写行与暖机）整段落盘的盘面上，`kinds_reading_back_zero` 那几种写读回全零，离线出路断言第 0 代：
/// 单元读不出时 birth_txg 闸数不到（模块文档「本实例写下的单元一个都解不开」那一形），环里读得到比第 0 代新的东西就照拒，tail 条款没写；一个字节不写。
/// mkfs 之后第一次可写挂载的写行与暖机都是零单元发布、一个单元都没写，`StepKind::UnitWrite` 那一种在这份写表里没有落点；判的只是读坏的那一半环。
fn assert_the_offline_way_out_asserting_the_genesis_is_refused_with_the_tail_undecided(
    kinds_reading_back_zero: &[StepKind],
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let (image, writes, warm_up_publishes) =
        image_and_writes_after_the_first_mount_on_a_formatted_pool(&parameters);
    assert!(warm_up_publishes > 0, "前提：这次挂载推了暖机");
    let faults = faults_on_the_writes_of_the_kinds(&writes, kinds_reading_back_zero);
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &faults, &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion { .. })
        ),
        "断言第 0 代：写下的单元读不出、环里读得到比它新的东西，拒；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == image,
        "拒的时候一个字节不写"
    );
}

/// 写行与暖机的根读得出，单元与记录都读回全零：判的是根环那一半。
#[test]
fn the_offline_way_out_asserting_the_genesis_is_refused_when_newer_roots_read_while_units_and_records_after_it_read_back_zero(
) {
    assert_the_offline_way_out_asserting_the_genesis_is_refused_with_the_tail_undecided(&[
        StepKind::UnitWrite,
        StepKind::JournalRecord,
    ]);
}

/// 写行与暖机的记录读得出，单元与根都读回全零：判的是 journal 环那一半。
#[test]
fn the_offline_way_out_asserting_the_genesis_is_refused_when_newer_records_read_while_units_and_roots_after_it_read_back_zero(
) {
    assert_the_offline_way_out_asserting_the_genesis_is_refused_with_the_tail_undecided(&[
        StepKind::UnitWrite,
        StepKind::RootRecordFua,
    ]);
}

/// 同一份盘面没注入读故障：mkfs 之后第一次可写挂载要写的行为空（上一个实例是 mkfs 的第 0 代），写行那次发布与暖机都是零单元发布
/// （`mount.rs` 的 `acquire_then_publish_until_the_continuation` 里 `PreviousVersion::WithoutFile` 那一支），单元区里没有比第 0 代新的单元，
/// birth_txg 闸数不到东西；根环与 journal 环里读得到写行与暖机的根和记录，照拒在 tail 条款没写上，一个字节不写。
#[test]
fn the_offline_way_out_asserting_the_genesis_after_the_first_mount_of_zero_unit_publishes_is_refused_with_the_tail_undecided_without_read_faults(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let (image, writes, warm_up_publishes) =
        image_and_writes_after_the_first_mount_on_a_formatted_pool(&parameters);
    assert!(warm_up_publishes > 0, "前提：这次挂载推了暖机");
    assert!(
        writes.iter().all(|write| write.kind != StepKind::UnitWrite),
        "前提：mkfs 之后第一次可写挂载的写行与暖机都是零单元发布，一个单元都没写"
    );
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion { .. })
        ),
        "断言第 0 代：环里读得到比它新的根与记录，拒在 tail 条款没写上；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == image,
        "拒的时候一个字节不写"
    );
}
