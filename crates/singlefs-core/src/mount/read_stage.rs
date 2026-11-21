//! 可写挂载的读阶段：一遍择根、扫 journal、重放，判据 N-配置 的读数，至多重读一次，
//! 水位之上缺记录而点名单元验不过的发布就拒。

use super::*;

/// 这次挂载内有效的读缓存（C554 乙：E158 第 3 次跑登记第 363 行「乙-配置」那一格；第 4 次跑登记 5.2 第 1 条与 5.5 第 1 条主 agent 的认定）：
/// 读阶段（择根、扫 journal、重放）两遍都经它读。只收读成、而且不是全零的落点的字节；读失败的与读回全零的不收，重读那一遍照样打到盘上——
/// 于是重读那一遍真正打到盘上的，是第一遍读失败或读回全零的落点，与重做时新走到的落点。读阶段判完之后的读直接读盘、不经它。
/// 代价（推的，没量）：扫 journal 时环里写过的每个记录槽都进缓存，环写满一圈之后是环长乘盘数（默认环两块盘约 1.5 GiB），活到读阶段判完为止。
pub(super) struct ReadStageCache<'devices, Reader: PoolReader> {
    devices: &'devices Reader,
    bytes_read_by_placement: std::cell::RefCell<
        BTreeMap<(DeviceIdentity, crate::address::DeviceOffsetInBytes, usize), Vec<u8>>,
    >,
}

impl<'devices, Reader: PoolReader> ReadStageCache<'devices, Reader> {
    fn empty_over(devices: &'devices Reader) -> Self {
        ReadStageCache {
            devices,
            bytes_read_by_placement: std::cell::RefCell::new(BTreeMap::new()),
        }
    }
}

impl<Reader: PoolReader> PoolReader for ReadStageCache<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.devices
            .physical_block_size_probed_on_the_device(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: crate::address::DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let placement = (device, offset, length);
        if let Some(bytes) = self.bytes_read_by_placement.borrow().get(&placement) {
            return Some(bytes.clone());
        }
        let bytes = self.devices.read(device, offset, length)?;
        if bytes.iter().any(|byte| *byte != 0) {
            self.bytes_read_by_placement
                .borrow_mut()
                .insert(placement, bytes.clone());
        }
        Some(bytes)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: crate::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<crate::address::DeviceOffsetInBytes>> {
        self.devices
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// F1 乙（用户 2026-10-07 定「缺记录就拒可写」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）：水位之上、没被施加的
/// 每一次发布（同一 (实例代号, checkpoint_txg) 的记录），读得出的那几条的「本次发布内序号」不是 1..N 连着（I-8.9：一次发布的序号依次是 1..N），
/// 或其中没有一条带末条标志，而它们点名的单元又有一个验不过——某个点名单元两份都读不出或校验和对不上（`F1_ANY`，判决
/// `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行与第三节第 1 条；`recovery::every_named_unit_of_the_record_has_a_copy_that_verifies`：
/// 一份坏是单故障，按两份都要验过判会让写记录中途断电、根没落的那次发布一份副本坏时每次可写挂载都拒；施加前验点名单元仍两份都要验过，
/// 不是这一份）——交回这样的发布，按 (实例代号, checkpoint_txg) 升序。
/// 缺的那条记录点名了哪几个单元挂载时没有别的东西记着（C 隔离不到，判决第二节 K2「C 的盲区」那一行），而那次发布可能已落过根、
/// 可能被下一次恢复择到：调用方重读一次仍这样就在取号之前拒可写。正常崩溃在写记录中途（单元在记录之前落盘，D16（发布语义） 已定项 7）
/// 读得出的记录点名的单元验得过，不交回。水位与 C 同一个（`placements_named_by_unapplied_journal_records`）。
fn publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
) -> Vec<RollbackTarget> {
    let mut records_of_each_publish: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<&crate::journal::JournalRecord>,
    > = BTreeMap::new();
    for record in records.values().filter(|record| {
        (record.instance, record.checkpoint_txg)
            > (selected_version.instance, selected_version.checkpoint_txg)
    }) {
        records_of_each_publish
            .entry((record.instance, record.checkpoint_txg))
            .or_default()
            .push(record);
    }
    records_of_each_publish
        .into_iter()
        .filter(|(_, records_of_the_publish)| {
            let some_named_unit_does_not_verify = records_of_the_publish.iter().any(|record| {
                !crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(
                    reader, record,
                )
            });
            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)
                && some_named_unit_does_not_verify
        })
        .map(|((instance, checkpoint_txg), _)| RollbackTarget {
            instance,
            checkpoint_txg,
        })
        .collect()
}

/// 一次发布读得出的那几条记录（`records_of_the_publish`：同一 (实例代号, checkpoint_txg)、读阶段扫得出的）不齐：「本次发布内序号」不是 1..N 连着
/// （I-8.9：一次发布的序号依次是 1..N），或其中没有一条带末条标志；一条都读不出也算不齐。F1 乙
/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）与 C 的账读不出时从中央映射现算的那一支
/// （`C_MAPPING` 收窄，[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）用同一份。只读内存里的记录。
pub(super) fn readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
    records_of_the_publish: &[&crate::journal::JournalRecord],
) -> bool {
    let mut ordinals: Vec<u32> = records_of_the_publish
        .iter()
        .map(|record| record.ordinal_within_publish.0)
        .collect();
    ordinals.sort_unstable();
    let ordinals_run_from_one_without_a_gap = ordinals
        .iter()
        .zip(1..)
        .all(|(ordinal, expected)| *ordinal == expected);
    let carries_a_last_record = records_of_the_publish.iter().any(|record| {
        record.place_in_publish
            == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
    });
    !(ordinals_run_from_one_without_a_gap && carries_a_last_record)
}

/// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`system_configuration_slots_of_one_device_read_once`，每块盘两槽），
/// 不经读缓存（第 4 次跑登记 5.1 第 1 条：重读那一遍也直接从盘上重读这几槽）；记录取读阶段那一遍扫出来的。
/// 判不出两种：R2 本身（[`WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots`]），与乙入判不出——
/// 读得出而自证不过的槽当读错计（[`WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots`]）。
fn newer_publish_witness<Reader: PoolReader + ?Sized>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    selected_version: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> NewerPublishWitness {
    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    // 按盘读（S-辨）：每块盘两槽各读一次，自证过的进 c_见证，设备报读错的计数（D22（单元原子性怎么合成） 已定项 8 射程）。
    // 读的一方外面套一层记每一读的结论（[`SystemConfigurationSlotReadsRecordedByTheCheck`]）：判法照旧是
    // `system_configuration_slots_of_one_device_read_once` 那一份，这里只多认出读得出而自证不过的那几槽（乙入判不出要它们）。
    // 循环的上界是盘数；跨轮只往两张表里追加；没有提前出口。
    let mut slots_of_each_device: Vec<SystemConfigurationSlotsOfOneDevice> = Vec::new();
    let mut system_configuration_slots_as_read: Vec<SystemConfigurationSlotAsChecked> = Vec::new();
    for device in devices.device_identities() {
        let slot_reads_recorded = SystemConfigurationSlotReadsRecordedByTheCheck {
            reader: devices,
            slots_read: RefCell::new(Vec::new()),
        };
        slots_of_each_device.push(system_configuration_slots_of_one_device_read_once(
            &slot_reads_recorded,
            device,
            slot_spacing_in_bytes,
            &system_configuration.immutable.filesystem_identifier,
        ));
        system_configuration_slots_as_read.extend(slot_reads_recorded.slots_read.into_inner());
    }
    let witnessed_journal_counter = slots_of_each_device
        .iter()
        .flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    let comparison = if system_configuration_witness_is_undecidable(&slots_of_each_device) {
        WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
    } else if some_system_configuration_slot_fails_self_verification_and_no_device_self_verified_both_slots(
        &slots_of_each_device,
        &system_configuration_slots_as_read,
    ) {
        WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots
    } else if witnessed_journal_counter == 0 {
        WitnessedCounterComparison::NothingWitnessed
    } else if let Some(last_record_of_the_selected_version) = records
        .values()
        .filter(|record| {
            record.instance == selected_version.instance
                && record.checkpoint_txg == selected_version.checkpoint_txg
                && record.place_in_publish
                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
    {
        WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
            selected_version_last_record_counter: last_record_of_the_selected_version.counter,
        }
    } else if let Some(record_at_the_witnessed_counter) = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .max_by_key(|record| (record.instance, record.checkpoint_txg))
    {
        WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter {
            record: RollbackTarget {
                instance: record_at_the_witnessed_counter.instance,
                checkpoint_txg: record_at_the_witnessed_counter.checkpoint_txg,
            },
        }
    } else {
        WitnessedCounterComparison::Undecidable
    };
    NewerPublishWitness {
        witnessed_journal_counter,
        comparison,
    }
}

/// 乙入判不出（C971（多处读坏时可写挂载写出倒挂的新实例根），`research/prompts/e205-preregistration.md`「乙入判不出」那一行）这一遍读的判法：
/// 这一遍读到的系统配置槽里有读得出而自证不过的，且没有一块盘两槽都自证过（R2 的另一半照旧）。读得出而自证不过的有两样：
/// `slots_as_read` 里结论是读坏的那几槽（撕裂、读回全零），与整槽校验和过而 fsid 不是本池的槽
/// （[`SystemConfigurationSlotsOfOneDevice::slots_of_another_pool_whose_checksum_verifies`]：登记「两种读法共用」那一行「自证过」要 fsid 与本池相同，
/// C707（系统配置判不出里 fsid 不符落哪一边没写）定按自证不过处置；读的一方对它不报读坏，`slots_as_read` 里记成读得出、没报自证不过，按盘数出来的那一份认它）。
/// 布局位不认识、每区槽数越界的槽不算（登记同一行）。设备报读错的那一半由
/// [`system_configuration_witness_is_undecidable`] 在它前面判，这里不重算。这一遍为真按判据 N-配置 为真处置，「重读一次」是读阶段那一次重读
/// （[`read_stage_settled_at_most_on_the_one_reread`]：整个读阶段重做一遍，系统配置槽直接从盘上重读、按同一个判法再判，只按重读那一遍的读数定拒不拒）。
fn some_system_configuration_slot_fails_self_verification_and_no_device_self_verified_both_slots(
    slots_of_each_device: &[SystemConfigurationSlotsOfOneDevice],
    slots_as_read: &[SystemConfigurationSlotAsChecked],
) -> bool {
    if slots_of_each_device
        .iter()
        .any(|slots| slots.slots_of_another_pool_whose_checksum_verifies > 0)
        && !slots_of_each_device
            .iter()
            .any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)
    {
        return true;
    }
    slots_as_read.iter().any(|slot| {
        slot.conclusion == SystemConfigurationSlotConclusionWhenChecked::ReadFailingSelfVerification
    }) && !slots_of_each_device
        .iter()
        .any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)
}

/// 读阶段一遍：择根、扫 journal、重放（经 `cache` 读），再判 N-配置（系统配置槽经 `devices` 直接读）。
///
/// # Errors
/// 一条自证过的根都没有（`RecoveryFailure::NoValidRoot`）；重放报的（`replay_journal` 的 `Errors`）。
pub(super) fn read_stage<Reader: PoolReader>(
    cache: &ReadStageCache<'_, Reader>,
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Result<ReadStage, MountError> {
    let RootChosenWithEverySelfVerifiedRootRead {
        chosen_root: chosen_root_if_any_self_verified,
        self_verified_roots_in_ring_order: self_verified_roots_read,
    } = choose_root_keeping_every_self_verified_root_read(cache, system_configuration);
    let chosen_root = chosen_root_if_any_self_verified.ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(cache, system_configuration);
    let (journal, effective_root) = replay_journal(
        cache,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        NamedUnitVerification::Verify,
    )?;
    let witness = newer_publish_witness(devices, system_configuration, &effective_root, &records);
    let publishes_missing_records_whose_named_units_do_not_verify =
        publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
            cache,
            &records,
            &effective_root,
        );
    Ok(ReadStage {
        chosen_root,
        self_verified_roots_read,
        records,
        journal,
        effective_root,
        against_the_witness: SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: effective_root.instance,
                checkpoint_txg: effective_root.checkpoint_txg,
            },
            witness,
        },
        publishes_missing_records_whose_named_units_do_not_verify,
    })
}

/// 一版 `version` 覆盖的最后一条记录（建上一版时拿它当上一版的记录）：所选根覆盖的最后一条记录读得出就是它——同实例、同 checkpoint_txg
/// 的记录里带「本次发布末条」标志的那一条（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定：「那条」按末条标志认；
/// 一次发布切成多条记录时它是那次发布的末条）。读不出（两份都撕了，或那次发布读得出的几条都不带标志）就拿环里最大 jsn 那条顶着——
/// 本实例的第一条反向链恒 0、事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录
/// （只做过 mkfs 的池环里一条都没有）。
pub(crate) fn record_standing_for_the_version(
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    version: &RootRecord,
) -> Option<crate::journal::JournalRecord> {
    records
        .values()
        .filter(|record| {
            record.instance == version.instance
                && record.checkpoint_txg == version.checkpoint_txg
                && record.place_in_publish
                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
}

/// C554 乙（用户 2026-09-27 定，R = 1）：读阶段读一遍，判据 N-配置 为真（系统配置见证过比所选那一版新的发布）就在同一个读缓存上
/// 重做一遍（重做之前调 `before_the_one_reread`）。哪一遍判假，就交回哪一遍：往下走用它的所选根、记录与所选那一版。
/// 在 `replay_journal` 交回之后、`rebuild_previous_version` 之前判（E158 第 3 次跑登记 5.1 末段），取号之前。
/// F1 乙（用户 2026-10-07 定）同一个做法：水位之上有一次缺记录、读得出的记录点名的单元又验不过的发布
/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）也在同一个读缓存上重做读阶段一遍，仍有就拒。
///
/// # Errors
/// 重读那一遍仍判真 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`；
/// 重读那一遍判假而仍有那样一次发布 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)`；
/// 任一遍 [`read_stage`] 报的错原样交回。
pub(super) fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<(ReadStage, ReadStageSettled), MountError> {
    let cache = ReadStageCache::empty_over(devices);
    let first = read_stage(&cache, devices, system_configuration)?;
    let first_read = first.against_the_witness;
    if !first_read.witnesses_a_publish_newer_than_the_selected_version()
        && first
            .publishes_missing_records_whose_named_units_do_not_verify
            .is_empty()
    {
        return Ok((first, ReadStageSettled::OnTheFirstRead { first_read }));
    }
    before_the_one_reread.before_rereading();
    let reread = read_stage(&cache, devices, system_configuration)?;
    let reread_against_the_witness = reread.against_the_witness;
    if reread_against_the_witness.witnesses_a_publish_newer_than_the_selected_version() {
        return Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                    first_read,
                    reread: reread_against_the_witness,
                },
            ),
        ));
    }
    if let Some(publish) = reread
        .publishes_missing_records_whose_named_units_do_not_verify
        .first()
    {
        return Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                    publish: *publish,
                },
            ),
        ));
    }
    Ok((
        reread,
        ReadStageSettled::OnTheOneReread {
            first_read,
            reread: reread_against_the_witness,
        },
    ))
}

/// F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H2 那一行与
/// 第三节第 1 条）：读阶段判 F1 时的水位是退之前的所选那一版——所选根读得出、走读失败、退回的那一形里，被退掉的那次发布不在那时的水位之上，
/// 一条都不判；退到 `version_fallen_back_to` 之后它在水位之上了。按退到的那一版当水位判
/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]，点名单元直接读盘）；记录取调用方退回之后、判之前重扫环的那一份
/// （`records_rescanned_before_rejudging`，C754（退回之后再判缺记录不重扫环），用户 2026-10-20 定「重扫」，不用读阶段扫出的那一份）。
/// 有那样一次发布就重读一次：重读之前调 `before_the_one_reread`，再重扫一遍 journal 环、点名单元再直接读一遍盘——环与点名单元一起重读，
/// 与读阶段那一次重读（整个读阶段重做，环在内）同一个口径；仍有就拒，与读阶段那一判同一个拒因。只读盘、不写。
///
/// # Errors
/// 重读一次仍有 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)`（按 (实例代号, checkpoint_txg)
/// 最小的那一次），在取号之前、盘上逐字节不变。
pub(super) fn refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify<
    Reader: PoolReader,
>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records_rescanned_before_rejudging: &BTreeMap<
        (InstanceGeneration, u64),
        crate::journal::JournalRecord,
    >,
    version_fallen_back_to: &RootRecord,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<(), MountError> {
    if publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
        devices,
        records_rescanned_before_rejudging,
        version_fallen_back_to,
    )
    .is_empty()
    {
        return Ok(());
    }
    before_the_one_reread.before_rereading();
    match publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
        devices,
        &scan_journal(devices, system_configuration),
        version_fallen_back_to,
    )
    .first()
    {
        Some(publish) => Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                    publish: *publish,
                },
            ),
        )),
        None => Ok(()),
    }
}
