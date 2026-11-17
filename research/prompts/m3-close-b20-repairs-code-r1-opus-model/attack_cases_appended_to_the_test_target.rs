
// ===== 攻方用例（只在副本里）=====

/// 一段字节只在第 `first_bad_read` 到 `last_bad_read` 次读（从 1 数）读回全零的盘：外面套在带常驻读故障的盘上。写到这一段就撤。
struct AttackDeviceWithOneRangeBadOnChosenReads {
    inner: DeviceWithReadFaults,
    range: Option<(u64, u64)>,
    first_bad_read: u64,
    last_bad_read: u64,
    reads_of_the_range: Rc<Cell<u64>>,
}

impl AttackDeviceWithOneRangeBadOnChosenReads {
    fn lift_when_written_over(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        if let Some((start, length)) = self.range {
            if offset_in_bytes < start + length && start < offset_in_bytes + length_in_bytes {
                self.range = None;
            }
        }
    }
}

impl BlockDevice for AttackDeviceWithOneRangeBadOnChosenReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Some((start, length)) = self.range {
            if offset.0 < start + length && start < offset.0 + buffer_length {
                let read_number = self.reads_of_the_range.get() + 1;
                self.reads_of_the_range.set(read_number);
                if self.first_bad_read <= read_number && read_number <= self.last_bad_read {
                    let from = usize::try_from(start.saturating_sub(offset.0)).expect("缓冲内偏移");
                    let to =
                        usize::try_from((start + length).min(offset.0 + buffer_length) - offset.0)
                            .expect("缓冲内偏移");
                    buffer[from..to].fill(0);
                }
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(
            offset.0,
            u64::try_from(bytes.len()).expect("写长装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.lift_when_written_over(offset.0, length);
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// D（txg 6）那次发布的全部记录（两块盘各一份）读回全零：D 的根槽落了、轮换没落，c_见证 是 C 的计数器。
fn attack_faults_on_the_records_of_d(history: &MinimalHistory) -> Vec<ReadFault> {
    let counters = &history
        .record_counters_by_txg
        .iter()
        .find(|(record_txg, _)| *record_txg == TXG_OF_THE_NEWEST_VERSION)
        .expect("最小历史里有 D")
        .1;
    let mut faults = Vec::new();
    for counter in counters {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            faults.push(ReadFault {
                device,
                offset_in_bytes: record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES).0,
                length_in_bytes: JOURNAL_RECORD_BYTES,
                kind: ReadFaultKind::ReadsBackAllZeros,
            });
        }
    }
    faults
}

/// 池级 checker 在这张盘上判违例的不变量名。
fn attack_violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| {
            matches!(
                verdict,
                singlefs_checker::image::InvariantVerdict::Violated(_)
            )
        })
        .map(|(identifier, _)| identifier)
        .collect()
}

/// D 的根自证过、D 的记录两份都读回全零（不带瞬时读错）：新实例第一次发布越过 D（txg 7），环里 D 这条根把 max 抬上去。
/// 判别力：变异「新实例第一个 txg 只看记录」（副本变异表最后一行）时写行 txg 落到 6，这一条红。
#[test]
fn attack_a_root_read_with_its_records_unreadable_takes_the_first_txg_past_it() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = attack_faults_on_the_records_of_d(&history);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &history.image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert_eq!(
        (
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.row_publish.root().checkpoint_txg.0
        ),
        (TXG_OF_THE_NEWEST_VERSION, TXG_OF_THE_NEWEST_VERSION + 1),
        "落到 D，写行那次发布越过 D"
    );
}

/// 打中面一、面二的主形：盘面同上一条（D 的记录两份都读回全零，崩在 D 的根槽之后、轮换之前），读阶段把 D 的根读对了（所选那一版是 D），
/// 取第一个 txg 那一遍（`first_txg_of_new_instance`，直接读盘、不经读缓存）读到 D 的根槽读回全零：max(根, 记录) 只到 C，写行那次发布 txg 不大于所选那一版。
/// 第几次读落在那一遍里不写死，k 与「只这一次 / 从这一次起一直」都扫一遍，逐行打 `ATTACK_FIRST_TXG` 行。
/// 断言：至少一格可写挂载做成而写行 txg 不大于所选那一版。这一条红就是我推错了。
#[test]
fn attack_a_root_read_bad_only_after_the_read_stage_numbers_the_new_instance_at_or_below_the_selected_version(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = attack_faults_on_the_records_of_d(&history);
    let target = target_for_publish(
        CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0;
    let mut cells_at_or_below_the_selected_version = 0u64;
    for only_this_read in [true, false] {
        for first_bad_read in 1..=16u64 {
            let last_bad_read = if only_this_read {
                first_bad_read
            } else {
                u64::MAX
            };
            let faults_lifted = Rc::new(Cell::new(false));
            let reads_of_the_range = Rc::new(Cell::new(0u64));
            let mut devices: Vec<(DeviceIdentity, AttackDeviceWithOneRangeBadOnChosenReads)> =
                devices_with_read_faults(&history.image, &faults, &faults_lifted)
                    .into_iter()
                    .map(|(identity, inner)| {
                        (
                            identity,
                            AttackDeviceWithOneRangeBadOnChosenReads {
                                inner,
                                range: (identity == root_device)
                                    .then_some((root_offset, ROOT_SLOT_BYTES)),
                                first_bad_read,
                                last_bad_read,
                                reads_of_the_range: Rc::clone(&reads_of_the_range),
                            },
                        )
                    })
                    .collect();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                mount_writable(&parameters, &mut devices)
            }));
            let image_after = memory_pool_of_sparse_devices(&devices, |device| &device.inner.inner);
            match outcome {
                Ok(Ok(mounted)) => {
                    let effective = mounted.output.effective_root.checkpoint_txg.0;
                    let row = mounted.output.row_publish.root().checkpoint_txg.0;
                    if row <= effective {
                        cells_at_or_below_the_selected_version += 1;
                    }
                    let (second_mount, _) =
                        writable_mount_under_read_faults(&parameters, &image_after, &[]);
                    println!(
                        "ATTACK_FIRST_TXG only_this_read={only_this_read} k={first_bad_read} reads={} mount=ok effective={effective} row={row} instance={:?} violated={:?} second_mount_ok={}",
                        reads_of_the_range.get(),
                        mounted.output.instance,
                        attack_violated_invariants(&image_after),
                        second_mount.is_ok()
                    );
                }
                Ok(Err(refusal)) => {
                    let text = format!("{refusal:?}");
                    let head: String = text.chars().take(160).collect();
                    println!(
                        "ATTACK_FIRST_TXG only_this_read={only_this_read} k={first_bad_read} reads={} mount=refused image_unchanged={} refusal={head}",
                        reads_of_the_range.get(),
                        image_after == history.image
                    );
                }
                Err(_) => println!(
                    "ATTACK_FIRST_TXG only_this_read={only_this_read} k={first_bad_read} reads={} mount=panic",
                    reads_of_the_range.get()
                ),
            }
        }
    }
    println!("ATTACK_FIRST_TXG_SUMMARY cells_at_or_below_the_selected_version={cells_at_or_below_the_selected_version}");
    assert!(
        cells_at_or_below_the_selected_version > 0,
        "取第一个 txg 那一遍读坏所选那一版的根，写行 txg 不大于所选那一版"
    );
}

/// 打中面三（判官吃的所选那一版仍不筛 fsid）：盘 1 较旧那一槽换成世代号大 100 的别的池的槽（盘 0 两槽本池自证过，单故障），C、D 的根与记录读回全零。
/// 产品按本池 fsid 走：盘 0 两槽都自证过，乙入判不出不判，c_见证 是 C 的计数器、所选那一版是 B，比法 `AgainstTheSelectedVersionsLastRecord` 为真，拒——这一拒是对的。
/// 判官自己择系统配置筛了 fsid，而 `selected_version` 按文档取只读恢复落到的那一版（`read_recovery_fields` 不筛 fsid）：恢复报 fsid 对不上、没有所选那一版，
/// 判据「无从复算」交回假，合法的拒判成不合法。同一份盘面喂产品落到的 B（对照）判合法：只差 `selected_version` 那一个入参。
#[test]
fn attack_the_judge_fed_the_version_recovery_lands_on_calls_a_legal_witness_refusal_illegal_when_one_slot_is_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let mut image = history.image.clone();
    let device_one = DeviceIdentity(1);
    let newest = newest_system_configuration_slot(&image, device_one);
    let older = older_system_configuration_slot(&image, device_one);
    let generation = generation_at(&image, device_one, newest) + 100;
    let bytes = slot_of_another_pool(&image, device_one, generation);
    write_slot(&mut image, device_one, older, &bytes);
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING));
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &faults);
    assert!(image_after == image, "拒在取号之前");
    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = &outcome
    else {
        panic!("可写挂载按判据 N-配置 拒");
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = still_unreadable.as_ref()
    else {
        panic!("拒因是判据 N-配置");
    };
    println!(
        "ATTACK_JUDGE selected={:?} first_comparison={:?} reread_comparison={:?}",
        reread.selected_version, first_read.witness.comparison, reread.witness.comparison
    );
    assert!(
        matches!(
            reread.witness.comparison,
            WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord { .. }
        ),
        "盘 0 两槽都自证过，比法是计数器比，不是判不出"
    );
    let crash_image = ImageWithReadFaults {
        image: &image,
        faults: &faults,
    };
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    println!(
        "ATTACK_JUDGE recovery_effective_root={:?} recovery_outcome={:?}",
        fields.report.effective_root, fields.report.outcome
    );
    assert_eq!(
        fields.report.effective_root, None,
        "恢复不筛 fsid，落不到任何一版"
    );
    let verdict = verdict_of_the_refusal_on(&crash_image, &outcome);
    println!("ATTACK_JUDGE verdict_with_the_recovered_version={verdict:?}");
    assert!(
        says_the_criterion_is_false(&verdict),
        "判官把这一拒判成判据复算为假"
    );
    let Err(refusal) = &outcome else {
        panic!("上面判过是拒");
    };
    assert_eq!(
        judge_writable_mount_refusal(
            &crash_image,
            &FILESYSTEM_IDENTIFIER,
            Some((
                reread.selected_version.instance,
                reread.selected_version.checkpoint_txg
            )),
            refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
        ),
        "对照：所选那一版喂产品落到的那一版，判合法"
    );
}

/// 打中面三（条款射程少一形）：只有一块盘带世代号大的别的池的槽。盘 0 两槽本池自证过时可写挂载做成，只读挂载与恢复报各盘 fsid 对不上：可写过得去、只读过不去。
/// 盘 0 再撕一槽时可写挂载乙入判不出拒，只读挂载同样报对不上：不是「每块盘都有这样一槽」，同样只剩离线出路。
#[test]
fn attack_a_newest_generation_of_another_pool_on_only_one_device_refuses_read_only_while_writable_goes_through_or_refuses_too(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    assert_eq!(
        mount_read_only(&image).err(),
        Some(MountReadOnlyFailure::Recovery(
            RecoveryFailure::SystemConfigurationsDisagree
        )),
        "只读挂载不筛 fsid，两块盘择到的 fsid 对不上"
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "可写挂载筛 fsid，做成"
    );
    let mut with_a_torn_slot = image.clone();
    let device_zero = DeviceIdentity(0);
    let older = older_system_configuration_slot(&with_a_torn_slot, device_zero);
    let mut torn = slot_bytes_at(&with_a_torn_slot, device_zero, older);
    torn[usize::try_from(FLIPPED_BYTE_INSIDE_A_RANGE).expect("100")] ^= 0xff;
    write_slot(&mut with_a_torn_slot, device_zero, older, &torn);
    let (refused, image_after) =
        writable_mount_under_read_faults(&parameters, &with_a_torn_slot, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&refused),
        "盘 0 撕一槽、盘 1 一槽别池：乙入判不出拒"
    );
    assert!(image_after == with_a_torn_slot, "拒在取号之前");
    assert_eq!(
        mount_read_only(&with_a_torn_slot).err(),
        Some(MountReadOnlyFailure::Recovery(
            RecoveryFailure::SystemConfigurationsDisagree
        )),
        "只读挂载同样对不上"
    );
}

/// 打中面三（判官与产品读一槽的口径差，A3 作废之后另一种取值）：两块盘较旧那一槽换成别的池的、区域数字段改成 2、整槽校验和重封的槽。
/// 产品 `SystemConfiguration::parse_slot` 不看区域数，算别的池的槽并进乙入判不出，拒；判官经 checker 的 `geometry_of` 解不开、又不是自证不过，两个计数都不进，判据复算为假。
#[test]
fn attack_a_slot_of_another_pool_with_a_region_count_the_checker_refuses_splits_the_judge_from_the_product(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let mut image = minimal_history_of_c971(&parameters).image;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let older = older_system_configuration_slot(&image, device);
        let generation = generation_at(&image, device, older);
        let mut slot = slot_of_another_pool(&image, device, generation);
        slot[361] = 2;
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            slot_bytes,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        write_slot(&mut image, device, older, &slot);
        assert_eq!(
            slot_counts_of(&image, device),
            (0, 1, 1),
            "前提：产品把它算别的池的槽"
        );
    }
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "产品乙入判不出拒"
    );
    assert!(image_after == image, "拒在取号之前");
    let Err(refusal) = &outcome else {
        panic!("上面判过是拒");
    };
    let verdict = judge_writable_mount_refusal(
        &image,
        &FILESYSTEM_IDENTIFIER,
        Some(newest_version_of_the_minimal_history()),
        refusal,
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    );
    println!("ATTACK_REGION_COUNT verdict={verdict:?}");
    assert!(
        says_the_criterion_is_false(&verdict),
        "判官把这一拒判成判据复算为假"
    );
}

/// mkfs 之后第一次可写挂载的写流里第 `segment` 段（取号写的一轮，两块盘各一写）两写都撕裂、之前各段整段持久。
fn attack_torn_acquisition_round_on_a_formatted_pool(
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

/// 打中面五（认下的这一拒在新格式化的池上是死路）：一次断电撕了 mkfs 之后第一次可写挂载取号写一轮的两写（单故障、设备没报读错）。
/// 可写挂载乙入判不出拒，只读挂载答树表 0 条（观察者认下这一答、计 `read_only_mount_found_no_file_version_after_the_refusal`），
/// 离线出路断言第 0 代也拒（第 0 代根读得出、没有记录，`TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`）。
/// 三样都走不通，只能重做 mkfs；D22 已定项 8「判不出的盘面有一条离线出路」在这一形不成立。
#[test]
fn attack_a_torn_acquisition_round_on_a_formatted_pool_leaves_no_way_out() {
    for slots_per_region in [4u64, 8, 16] {
        for segment in [0usize, 1] {
            let parameters = pool_parameters(slots_per_region);
            let image = attack_torn_acquisition_round_on_a_formatted_pool(&parameters, segment);
            let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
            assert!(
                refused_as_failing_self_verification_on_both_reads(&outcome),
                "S = {slots_per_region} 段 {segment}：乙入判不出拒"
            );
            assert!(image_after == image, "拒在取号之前");
            let read_only = format!("{:?}", mount_read_only(&image).err());
            let faults_lifted = Rc::new(Cell::new(false));
            let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
            let offline = rewrite_system_configuration_slots_for_the_asserted_version(
                &parameters,
                &mut devices,
                AssertedVersion {
                    instance: InstanceGeneration(0),
                    checkpoint_txg: CheckpointTxg(0),
                },
            );
            let offline = format!("{:?}", offline.err());
            println!(
                "ATTACK_FORMATTED slots_per_region={slots_per_region} segment={segment} writable=refused read_only={read_only} offline={offline}"
            );
            assert!(
                read_only.contains("TreeTableHasNoEntries"),
                "只读挂载答树表 0 条：{read_only}"
            );
            assert!(
                offline.contains("TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion"),
                "离线出路断言第 0 代拒：{offline}"
            );
        }
    }
}
