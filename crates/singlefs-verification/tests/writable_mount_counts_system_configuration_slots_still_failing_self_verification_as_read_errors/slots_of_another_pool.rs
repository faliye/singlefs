//! 别的池的系统配置槽：每块盘较旧那一槽来自别的池的拒、只有盘 1 的照挂，判器选本池的槽、照选出的槽判拒得合不合法，最新世代来自别的池时可写与只读都不成、较旧世代来自别的池照只读挂，给了本池 fsid 的只读挂载与恢复读本池，别的池的槽的区数、计不计，与产品同口径。

use super::*;

/// 乙的别池手造盘面：最小历史、不注读故障，`devices` 那几块盘两槽里较旧的那一槽换成一份别的池的系统配置
/// （同一个世代号，fsid 改一个字节、整槽校验和重封：magic、整槽校验和、布局位、每区槽数都过，只是 fsid 不是本池的）。
/// 换的是较旧那一槽：每块盘世代号最大的那一槽仍是本池的，挂载透明判官择系统配置（不按 fsid 筛）与可写挂载择到的是同一份。
fn minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
    parameters: &MakeFilesystemParameters,
    devices: &[DeviceIdentity],
) -> MemoryPool {
    let mut image = minimal_history_of_c971(parameters).image;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in devices {
        let offset = DeviceOffsetInBytes(older_system_configuration_slot(&image, *device));
        let bytes =
            PoolReader::read(&image, *device, offset, slot_bytes).expect("系统配置槽读得到");
        let mut another_pools = SystemConfiguration::parse_slot(&bytes).expect(
            "最小历史里每块盘两槽都自证过（系统配置轮换退回 D 之前的字节，两槽都是本池写的）",
        );
        another_pools.immutable.filesystem_identifier[15] ^= 0xff;
        image
            .devices
            .get_mut(device)
            .expect("有这块盘")
            .write(offset, &another_pools.to_slot());
    }
    image
}

/// 别池盘面一（两块盘较旧那一槽都换成别的池的）：设备没报读错、没有撕裂与全零的槽，每块盘一槽本池自证过、一槽整槽校验和过而 fsid 不是本池的；
/// 「自证过」要 fsid 与本池相同（登记「两种读法共用」那一行，C707 定按自证不过处置），于是没有一块盘两槽都自证过、有槽读得出而自证不过 ⇒
/// 乙入判不出，两遍读都是这一形，拒可写、盘上逐字节不变；挂载透明判官在同一份盘面上复算判这一拒合法。
/// 只换一块盘时另一块盘两槽都自证过、判得出（下一条阳性对照），所以两块盘都换：这是「没有一块盘两槽都自证过」而坏槽只有别的池的槽的最小一形。
#[test]
fn hand_made_image_with_the_older_slot_of_every_device_from_another_pool_is_refused() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            slot_counts_of(&image, device),
            (0, 1, 1),
            "前提：盘 {device:?} 设备没报读错、一槽本池自证过、一槽是别的池的（没有撕裂与全零的槽）"
        );
    }
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "两块盘各有一槽是别的池的、没有一块盘两槽都自证过：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在同一份盘面上把别的池的槽也算读得出而自证不过、复算乙入判不出为真：这一拒合法"
    );
}

/// 别池盘面二（阳性对照：盘 0 那一槽换回本池自证过的，只剩盘 1 较旧那一槽是别的池的）：盘 0 两槽都自证过，R2 的另一半不成立，判得出，
/// 照常可写挂载、取到实例 2，读阶段第一遍就判完（乙入判不出「不拒」）。
#[test]
fn hand_made_image_with_the_older_slot_of_only_device_one_from_another_pool_mounts_writable() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    assert_eq!(
        (
            slot_counts_of(&image, DeviceIdentity(0)),
            slot_counts_of(&image, DeviceIdentity(1))
        ),
        ((0, 2, 0), (0, 1, 1)),
        "前提：盘 0 两槽都本池自证过，盘 1 一槽本池自证过、一槽是别的池的"
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    let mounted = outcome.expect("盘 0 两槽都自证过：判得出，照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    assert!(
        matches!(
            mounted.output.rereads.read_stage,
            ReadStageSettled::OnTheFirstRead { .. }
        ),
        "判得出、判据为假：读阶段第一遍就判完，不重读；得到 {:?}",
        mounted.output.rereads.read_stage
    );
}

/// 同一盘一份别的池的系统配置，槽内 `offset_in_the_slot` 处那一个字节改成 `value`（产品与 checker 各自判不判这个字段不同），整槽校验和重封。
fn slot_of_another_pool_with_one_byte_changed_and_resealed(
    image: &MemoryPool,
    device: DeviceIdentity,
    generation: u64,
    offset_in_the_slot: usize,
    value: u8,
) -> Vec<u8> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut slot = slot_of_another_pool(image, device, generation);
    slot[offset_in_the_slot] = value;
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    slot
}

/// `devices` 那几块盘较旧那一槽的位置放一份世代号比本池最新那一槽大 100 的别的池的槽（不筛 fsid 的择法在这块盘上择到它）。
fn minimal_history_with_a_newest_generation_of_another_pool_on(
    parameters: &MakeFilesystemParameters,
    devices: &[DeviceIdentity],
) -> MemoryPool {
    let mut image = minimal_history_of_c971(parameters).image;
    for device in devices {
        let newest = newest_system_configuration_slot(&image, *device);
        let generation = generation_at(&image, *device, newest) + 100;
        let older = older_system_configuration_slot(&image, *device);
        let bytes = slot_of_another_pool(&image, *device, generation);
        write_slot(&mut image, *device, older, &bytes);
    }
    image
}

/// 最小历史上可写挂载读阶段落到的那一版：实例 1 的最新那一版 D。
fn newest_version_of_the_minimal_history() -> (InstanceGeneration, CheckpointTxg) {
    (
        InstanceGeneration(1),
        CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
    )
}

/// 所选那一版是 D 的读数（判官自己在镜像上复算判据，不看这里填的见证值）。
fn newest_version_against_an_undecidable_witness() -> SelectedVersionAgainstTheWitness {
    let (instance, checkpoint_txg) = newest_version_of_the_minimal_history();
    SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance,
            checkpoint_txg,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 1,
            comparison: WitnessedCounterComparison::Undecidable,
        },
    }
}

/// 用例自己拼的判据 N-配置 之拒。
fn witness_refusal_claimed_on_the_newest_version() -> MountError {
    let reading = newest_version_against_an_undecidable_witness();
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: reading,
            reread: reading,
        },
    ))
}

/// 用例自己拼的「读阶段判完之后系统配置又读不出」之拒（判官只认拒因的外层成员、自己复算判据）。
fn refusal_after_the_read_stage_claimed_on_the_newest_version() -> MountError {
    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage: newest_version_against_an_undecidable_witness(),
            unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DeviceIdentity(1),
                not_rewritten_because: DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet(
                    RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice,
                ),
            },
        },
    ))
}

/// 判官判 `claimed` 这一拒，所选那一版取 D（可写挂载在本池的槽上落到的那一版），U6 另两条都满足。
fn verdict_of_a_claimed_refusal_on(
    image: &MemoryPool,
    claimed: &MountError,
) -> WritableMountRefusalVerdict {
    judge_writable_mount_refusal(
        image,
        &FILESYSTEM_IDENTIFIER,
        Some(newest_version_of_the_minimal_history()),
        claimed,
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    )
}

fn says_the_criterion_is_false(verdict: &WritableMountRefusalVerdict) -> bool {
    matches!(
        verdict,
        WritableMountRefusalVerdict::Illegal { conditions_not_met, .. }
            if conditions_not_met.contains(
                &RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            )
    )
}

/// 盘 0 两槽都是别的池的，盘 1 较旧位置是世代号更大的别的池的槽（每块盘世代号最大的一槽都是同一个别的池的）。
fn minimal_history_with_both_slots_of_device_zero_from_another_pool(
    parameters: &MakeFilesystemParameters,
) -> MemoryPool {
    let mut image = minimal_history_with_a_newest_generation_of_another_pool_on(
        parameters,
        &[DeviceIdentity(1)],
    );
    let device_zero = DeviceIdentity(0);
    let newest = newest_system_configuration_slot(&image, device_zero);
    let older = older_system_configuration_slot(&image, device_zero);
    let generation = generation_at(&image, device_zero, newest) + 200;
    let older_slot_of_another_pool = slot_of_another_pool(&image, device_zero, generation);
    let newest_slot_of_another_pool = slot_of_another_pool(&image, device_zero, generation + 1);
    write_slot(&mut image, device_zero, older, &older_slot_of_another_pool);
    write_slot(
        &mut image,
        device_zero,
        newest,
        &newest_slot_of_another_pool,
    );
    assert_eq!(
        slot_counts_of(&image, device_zero),
        (0, 0, 2),
        "前提：盘 0 两槽都是别的池的"
    );
    assert_eq!(
        slot_counts_of(&image, DeviceIdentity(1)),
        (0, 1, 1),
        "前提：盘 1 一槽本池、一槽别池"
    );
    image
}

/// 判官按本池 fsid 择系统配置，合法的拒判合法：盘面见 [`minimal_history_with_both_slots_of_device_zero_from_another_pool`]。可写挂载按本池 fsid 判乙入判不出、拒；
/// 判官同样只从本池的槽里择（盘 1 那一份），数出没有一块盘两槽都自证过、有别的池的槽，判据复算为真。
#[test]
fn the_judge_chooses_the_slots_of_this_pool_and_calls_the_refusal_legal_when_both_slots_of_device_zero_are_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_both_slots_of_device_zero_from_another_pool(&parameters);
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "可写挂载乙入判不出拒"
    );
    assert!(image_after == image, "拒在取号之前");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "判官只从本池的槽里择，这一拒的判据复算为真：合法"
    );
}

/// 判官按本池 fsid 择系统配置，不该有的拒判不合法：一块盘世代号最大的那一槽是别的池的（单故障），可写挂载照常做成；同一份盘面上喂一个声称的判据 N-配置 之拒，
/// 判官只从本池的槽里择、各盘 fsid 对得上，见证值比出来不超过所选那一版，判据复算为假。别的池的槽世代号小的对照盘面同样判不合法。
#[test]
fn the_judge_chooses_the_slots_of_this_pool_and_calls_a_claimed_refusal_illegal_when_the_newest_slot_of_one_device_is_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "单故障，可写挂载做成"
    );
    let claimed = witness_refusal_claimed_on_the_newest_version();
    let verdict = verdict_of_a_claimed_refusal_on(&image, &claimed);
    assert!(
        says_the_criterion_is_false(&verdict),
        "这一拒不该有，判官判判据复算为假；得到 {verdict:?}"
    );
    let control = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    let verdict_on_the_control = verdict_of_a_claimed_refusal_on(&control, &claimed);
    assert!(
        says_the_criterion_is_false(&verdict_on_the_control),
        "对照盘面上判官同样判不合法；得到 {verdict_on_the_control:?}"
    );
}

/// 单子上别的理由也按本池 fsid 复算：同一份一块盘世代号最大的那一槽是别的池的盘面，每块盘都有本池自证过的槽、设备没报读错，
/// 「读阶段判完之后系统配置又读不出」的判据复算为假（择系统配置不筛 fsid 时各盘对不上、择不到、按真）。
#[test]
fn the_judge_chooses_the_slots_of_this_pool_and_calls_a_claimed_refusal_after_the_read_stage_illegal_when_the_newest_slot_of_one_device_is_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    let verdict = verdict_of_a_claimed_refusal_on(
        &image,
        &refusal_after_the_read_stage_claimed_on_the_newest_version(),
    );
    assert!(
        says_the_criterion_is_false(&verdict),
        "判官判判据复算为假；得到 {verdict:?}"
    );
}

/// 每块盘世代号最大的那一槽是别的池的：可写挂载乙入判不出拒；只读挂载不筛 fsid，择到别的池那一份、一条根都读不到；
/// 只剩离线出路（D22（单元原子性怎么合成） 已定项 8），它照断言的那一版重写别的池的那两槽，之后可写挂载做成。
#[test]
fn newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert_eq!(
        slot_counts_of(&image, DeviceIdentity(0)),
        (0, 1, 1),
        "前提：盘 0 一槽本池、一槽别池"
    );
    assert_eq!(
        slot_counts_of(&image, DeviceIdentity(1)),
        (0, 1, 1),
        "前提：盘 1 一槽本池、一槽别池"
    );
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "可写挂载乙入判不出拒"
    );
    assert!(image_after == image, "拒在取号之前");
    assert_eq!(
        mount_read_only(&image).err(),
        Some(MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot)),
        "只读挂载择到别的池那一份、一条根都读不到"
    );
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
    let rewritten = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        },
    )
    .expect("离线出路照断言的那一版重写别的池的那两槽");
    assert_eq!(rewritten.slots_rewritten.len(), 2, "每块盘重写一槽");
    let repaired = memory_pool_of_sparse_devices(&devices, |device| &device.inner);
    let (after_repair, _) = writable_mount_under_read_faults(&parameters, &repaired, &[]);
    assert_eq!(
        after_repair
            .as_ref()
            .map(|mounted| mounted.output.instance)
            .ok(),
        Some(InstanceGeneration(2)),
        "离线出路之后可写挂载做成"
    );
}

/// 上一条的对照：每块盘别的池的槽世代号小，只读挂载择到本池那一份，照常做成。
#[test]
fn older_generation_of_another_pool_on_every_device_still_mounts_read_only() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert!(
        mount_read_only(&image).is_ok(),
        "择到本池那一份，只读挂载做成"
    );
}

/// 一块盘最新一槽是别的池的盘面上，判官的所选那一版取按本池 fsid 的恢复落到的那一版：盘 1 较旧那一槽换成世代号大 100 的别的池的槽（盘 0 两槽本池自证过，单故障），
/// C、D 的根与记录读回全零。产品按本池 fsid 走：盘 0 两槽都自证过，乙入判不出不判，c_见证 是 C 的计数器、所选那一版是 B，比法 `AgainstTheSelectedVersionsLastRecord` 为真，拒。
/// 不筛 fsid 的恢复报各盘 fsid 对不上、落不到任何一版；按本池 fsid 的恢复落到 B，与被拒的那次挂载无关地取得，判官判合法。
#[test]
fn the_judge_fed_the_version_recovery_of_the_pool_lands_on_calls_a_legal_witness_refusal_legal_when_one_slot_is_from_another_pool(
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
        reread,
        ..
    } = still_unreadable.as_ref()
    else {
        panic!("拒因是判据 N-配置");
    };
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
    assert_eq!(
        read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
            .report
            .effective_root,
        None,
        "不筛 fsid 的恢复落不到任何一版"
    );
    assert_eq!(
        selected_version_of_the_pool_on_the_crash_image(&crash_image, &FILESYSTEM_IDENTIFIER),
        Some((
            reread.selected_version.instance,
            reread.selected_version.checkpoint_txg
        )),
        "按本池 fsid 的恢复落到被拒的那次挂载落到的那一版"
    );
    assert_eq!(
        verdict_of_the_refusal_on(&crash_image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "判官的所选那一版取自按本池 fsid 的恢复，判合法"
    );
}

/// 只有一块盘带世代号大的别的池的槽。不带 fsid 的只读挂载与恢复报各盘 fsid 对不上；带本池 fsid 的只读挂载与恢复读到本池最新那一版；盘 0 两槽本池自证过时可写挂载做成。
/// 盘 0 再撕一槽时可写挂载乙入判不出拒，不带 fsid 的只读挂载同样对不上，带本池 fsid 的只读挂载照样读到本池。
#[test]
fn newest_generation_of_another_pool_on_only_one_device_reads_this_pool_read_only_given_the_pool_fsid(
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
        "不带 fsid 的只读挂载，两块盘择到的 fsid 对不上"
    );
    assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(&image);
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
        "不带 fsid 的只读挂载同样对不上"
    );
    assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(
        &with_a_torn_slot,
    );
}

/// 带本池 fsid 的只读挂载落到 D，带本池 fsid 的恢复读出 D 的文件内容（最小历史第 4 版内容）。
fn assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(
    image: &MemoryPool,
) {
    let (newest_instance, newest_txg) = newest_version_of_the_minimal_history();
    let read_only = mount_read_only_of_the_pool(image, &FILESYSTEM_IDENTIFIER)
        .unwrap_or_else(|failure| panic!("带本池 fsid 的只读挂载做成；得到 {failure:?}"));
    assert_eq!(
        (
            read_only.effective_root.instance,
            read_only.effective_root.checkpoint_txg
        ),
        (newest_instance, newest_txg),
        "带本池 fsid 的只读挂载落到 D"
    );
    let report = recover_of_the_pool(image, JournalPolicy::Consult, &FILESYSTEM_IDENTIFIER);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (newest_instance, newest_txg),
            content: file_content_of(4),
        },
        "带本池 fsid 的恢复读出 D 的文件内容"
    );
}

/// 每块盘世代号最大的一槽都是别的池的：不带 fsid 的只读挂载报 `NoValidRoot`（[`newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only`]），
/// 带本池 fsid 的只读挂载与恢复读到本池。
#[test]
fn newest_generation_of_another_pool_on_every_device_reads_this_pool_read_only_given_the_pool_fsid()
{
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(&image);
}

/// 交进来的 fsid 盘上一槽都没有：带 fsid 的只读挂载与恢复报 `NoValidSystemConfiguration`，不读盘上那个调用方没点名的池。
#[test]
fn the_read_only_mount_given_a_filesystem_identifier_no_slot_carries_does_not_read_the_pool_on_the_disks(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let mut filesystem_identifier_of_no_slot = FILESYSTEM_IDENTIFIER;
    filesystem_identifier_of_no_slot[15] ^= 0xff;
    assert!(
        matches!(
            mount_read_only_of_the_pool(&image, &filesystem_identifier_of_no_slot),
            Err(MountReadOnlyFailure::Recovery(
                RecoveryFailure::NoValidSystemConfiguration { .. }
            ))
        ),
        "只读挂载拒，不读别的池"
    );
    let report = recover_of_the_pool(
        &image,
        JournalPolicy::Consult,
        &filesystem_identifier_of_no_slot,
    );
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Failed {
                root: None,
                failure: RecoveryFailure::NoValidSystemConfiguration { .. }
            }
        ),
        "恢复报择不到系统配置；得到 {:?}",
        report.outcome
    );
}

/// 判官与产品读别的池的槽同一个口径：两块盘较旧那一槽换成别的池的、区域数字段改成 2、整槽校验和重封的槽。
/// 产品 `SystemConfiguration::parse_slot` 不看区域数，算别的池的槽并进乙入判不出，拒；判官照产品读这一槽的那几关数它（不走 checker 的几何判），判据复算为真，判合法。
#[test]
fn slot_of_another_pool_with_a_region_count_the_checker_refuses_is_counted_by_the_judge_like_the_product(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let mut image = minimal_history_of_c971(&parameters).image;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let older = older_system_configuration_slot(&image, device);
        let generation = generation_at(&image, device, older);
        let slot = slot_of_another_pool_with_one_byte_changed_and_resealed(
            &image,
            device,
            generation,
            REGION_COUNT_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
            2,
        );
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
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "判官与产品一样数这一槽，判合法"
    );
}

/// 判官与产品读别的池的槽同一个口径的另一面：别的池的槽布局位不认识、每区槽数越界，产品不把它算别的池的槽（两块盘各一份），判官也不算；
/// 乙入判不出不成立，声称的判据 N-配置 之拒判判据复算为假。
#[test]
fn slot_of_another_pool_the_product_does_not_count_is_not_counted_by_the_judge() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    for (offset_in_the_slot, value_not_accepted) in [
        (
            INCOMPAT_BITMAP_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
            0x80,
        ),
        (
            ROOT_RING_SLOTS_PER_REGION_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
            17,
        ),
    ] {
        let mut image = minimal_history_of_c971(&parameters).image;
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let older = older_system_configuration_slot(&image, device);
            let generation = generation_at(&image, device, older);
            let slot = slot_of_another_pool_with_one_byte_changed_and_resealed(
                &image,
                device,
                generation,
                offset_in_the_slot,
                value_not_accepted,
            );
            write_slot(&mut image, device, older, &slot);
            assert_eq!(
                slot_counts_of(&image, device),
                (0, 1, 0),
                "前提：偏移 {offset_in_the_slot} 改了之后产品不把它算别的池的槽"
            );
        }
        let verdict = verdict_of_a_claimed_refusal_on(
            &image,
            &witness_refusal_claimed_on_the_newest_version(),
        );
        assert!(
            says_the_criterion_is_false(&verdict),
            "偏移 {offset_in_the_slot}：判官不数这一槽，判据复算为假；得到 {verdict:?}"
        );
    }
}

/// 崩溃态判官带本池 fsid，别的池的槽世代号再大也择不到：盘 0 两槽都是别的池的、盘 1 较旧位置是世代号更大的别的池的槽，可写挂载按本池 fsid 判乙入判不出、拒，判官只从本池的槽里择，判这一拒合法。
#[test]
fn the_crash_state_judge_given_the_pool_fsid_calls_a_legal_refusal_legal_when_every_device_chooses_a_slot_of_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_both_slots_of_device_zero_from_another_pool(&parameters);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "可写挂载乙入判不出拒"
    );
    let Err(refusal) = &outcome else {
        panic!("上面判过是拒");
    };
    let selected_version =
        selected_version_of_the_pool_on_the_crash_image(&image, &FILESYSTEM_IDENTIFIER);
    assert_eq!(
        selected_version,
        Some(newest_version_of_the_minimal_history()),
        "按本池 fsid 的恢复落到 D"
    );
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
            &image,
            &FILESYSTEM_IDENTIFIER,
            selected_version,
            refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "带本池 fsid：只从本池的槽里择，判合法"
    );
}
