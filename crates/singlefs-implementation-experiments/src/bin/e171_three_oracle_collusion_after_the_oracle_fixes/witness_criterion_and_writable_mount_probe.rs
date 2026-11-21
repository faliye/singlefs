//! 崩溃镜像上的见证判据与可写挂载探针。

use super::*;

// ───── 可写挂载探针（Q4，读法 8、停机 S10、S11） ─────

/// 见证判据计数那一支（读法 8 (ii)）为真的是哪一句（D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」那一段的三句）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WitnessCounterSentenceThatHolds {
    /// 所选那一版那次发布带末条标志的记录读得出，c_见证 大于它的计数器。
    AboveTheLastRecordOfTheSelectedVersion,
    /// 末条读不出，计数器等于 c_见证 的记录读得出、它的 (实例代号, checkpoint_txg) 大于所选那一版。
    RecordAtTheWitnessedCounterIsNewer,
    /// 两条都读不出，按真。
    NeitherRecordReadable,
}

/// 一个崩溃态上见证判据的两支（读法 8，装置在崩溃镜像上自己算，不调第二截与 mount 的私有函数）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WitnessCriterionOnTheCrashImage {
    /// (i) 判不出那一支：某块盘自证过的系统配置槽少于两槽（`recovery::verified_system_configuration_slots` 现数）。
    pub(super) some_device_has_fewer_than_two_verified_slots: bool,
    /// c_见证：全部自证过的槽的 journal_tail 取最大（一槽都没有是 0）。
    pub(super) witnessed_counter: u64,
    /// (ii) 计数那一支：每块盘两槽都自证过、c_见证 ≠ 0、三句里哪一句为真（都不为真是 `None`）。
    pub(super) counter_sentence: Option<WitnessCounterSentenceThatHolds>,
}

/// 在崩溃镜像上算见证判据的两支。`selected_version` 是只读恢复（看 journal）落到的那一版。
fn witness_criterion_on_the_crash_image(
    crash_image: &MemoryPool,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> WitnessCriterionOnTheCrashImage {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        // 一份可择的系统配置都没有：自证过的槽数必然少于两槽，记 (i)。
        return WitnessCriterionOnTheCrashImage {
            some_device_has_fewer_than_two_verified_slots: true,
            witnessed_counter: 0,
            counter_sentence: None,
        };
    };
    let slot_spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let verified_by_device: Vec<Vec<SystemConfiguration>> = crash_image
        .device_identities()
        .into_iter()
        .map(|device| {
            verified_system_configuration_slots(
                crash_image,
                device,
                slot_spacing,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    let some_device_has_fewer_than_two_verified_slots = verified_by_device
        .iter()
        .any(|verified| verified.len() < LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE);
    let witnessed_counter = verified_by_device
        .iter()
        .flatten()
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    let counter_sentence = if some_device_has_fewer_than_two_verified_slots
        || witnessed_counter == 0
    {
        None
    } else {
        selected_version.and_then(|selected| {
            let records = scan_journal(crash_image, &system_configuration);
            let last_record_counter = records
                .values()
                .filter(|record| {
                    (record.instance, record.checkpoint_txg) == selected
                        && record.place_in_publish
                            == JournalRecordPlaceInPublish::LastRecordOfThePublish
                })
                .map(|record| record.counter)
                .max();
            if let Some(last_record_counter) = last_record_counter {
                return (witnessed_counter > last_record_counter).then_some(
                    WitnessCounterSentenceThatHolds::AboveTheLastRecordOfTheSelectedVersion,
                );
            }
            let record_at_the_witnessed_counter = records
                .values()
                .filter(|record| record.counter == witnessed_counter)
                .map(|record| (record.instance, record.checkpoint_txg))
                .max();
            match record_at_the_witnessed_counter {
                Some(record) => (record > selected)
                    .then_some(WitnessCounterSentenceThatHolds::RecordAtTheWitnessedCounterIsNewer),
                None => Some(WitnessCounterSentenceThatHolds::NeitherRecordReadable),
            }
        })
    };
    WitnessCriterionOnTheCrashImage {
        some_device_has_fewer_than_two_verified_slots,
        witnessed_counter,
        counter_sentence,
    }
}

/// 一个崩溃态上可写挂载探针看到的（`writable-mount-probe.tsv` 一行）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WritableMountProbe {
    pub(super) criterion: WitnessCriterionOnTheCrashImage,
    pub(super) selected_version: String,
    /// `mounted`、`refused`。
    pub(super) mount: &'static str,
    /// 拒的错误成员（`{:?}` 的第一个标识符）与它在不在 D23 已定项 14 那张单子上（不在写 `-`）。
    pub(super) refusal_member: String,
    pub(super) refusal_on_the_legal_refusal_list: String,
    /// 第二截的判定：`legal`、`illegal:<没满足的条件>`、`reason_not_on_the_d23_list`；挂上了写 `-`。
    pub(super) second_crop: String,
    /// 拒的理由是系统配置见证（单子上那一条）。
    pub(super) refused_for_the_system_configuration_witness: bool,
}

/// 第二截判拒绝时交的「这一态没注入读故障」：本装置的崩溃态都是崩溃点重放截断出来的，没有注入任何读故障（读法 8）。
const READ_FAULTS_OF_A_CRASH_STATE: ReadFaultsInTheCrashState =
    ReadFaultsInTheCrashState::NoneInjected;

/// 一个崩溃态上的可写挂载探针（读法 8；入参逐项照崩溃注入第二截那一处取，停机 S10）：崩溃镜像落成内存池
/// （`crash_injection::materialized_crash_image`，第二截同一个函数），在副本上起一次可写挂载（`crash_injection::writable_mount_alone_on`，
/// 空间准入照公式判，同第二截默认的 `SpaceAdmission::JudgedByTheFormula`）；拒了就用第二截同一个调用
/// （`judge_writable_mount_refusal_of_a_crash_state_of_the_pool`）判合不合法：本池 fsid 取建池参数（`experiment_pool::parameters()`，可写挂载用的同一份），
/// 恢复落到的版本取看 journal 那一遍的 `effective_root`（盘面由一个池的写流盖在 mkfs 的盘上造出，没有别的池的槽，筛不筛 fsid 落到同一版），
/// 盘上同不同比拒之后的池与崩溃镜像，只读读出在不在允许集合里取 O1-模型崩溃判官红没红（第二截取同一次崩溃上模型判的结论），
/// 读故障交「没注入」，被拒的那次挂载读的就是崩溃镜像、不带故障层。
pub(super) fn writable_mount_probe_on(
    crash_image_materialized: &MemoryPool,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    crash_judge: &CrashJudgeVerdict,
) -> WritableMountProbe {
    let parameters = experiment_pool::parameters();
    let criterion =
        witness_criterion_on_the_crash_image(crash_image_materialized, selected_version);
    let read_only_read_back = match crash_judge {
        CrashJudgeVerdict::Red { .. } => ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet,
        CrashJudgeVerdict::Green
        | CrashJudgeVerdict::NotApplicable
        | CrashJudgeVerdict::Off
        | CrashJudgeVerdict::RecoveryPanicked
        | CrashJudgeVerdict::JudgePanicked => ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    };
    let mounted = writable_mount_alone_on(
        crash_image_materialized,
        &parameters,
        SpaceAdmission::JudgedByTheFormula,
    );
    let (
        mount,
        refusal_member,
        refusal_on_the_legal_refusal_list,
        second_crop,
        refused_for_the_witness,
    ) = match mounted {
        WritableMountAlone::Mounted { .. } => (
            "mounted",
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            false,
        ),
        WritableMountAlone::Refused {
            refusal,
            pool_after,
            ..
        } => {
            let on_the_list = refusal_reason_on_the_legal_refusal_list(&refusal);
            let disk = if pool_after == *crash_image_materialized {
                DiskAfterTheRefusal::ByteForByteUnchanged
            } else {
                DiskAfterTheRefusal::Changed
            };
            let verdict = judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
                crash_image_materialized,
                &parameters.filesystem_identifier,
                selected_version,
                &refusal,
                disk,
                read_only_read_back,
                READ_FAULTS_OF_A_CRASH_STATE,
                ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
            );
            let second_crop = match verdict {
                WritableMountRefusalVerdict::Legal(_) => "legal".to_string(),
                WritableMountRefusalVerdict::Illegal {
                    conditions_not_met, ..
                } => format!(
                    "illegal:{}",
                    conditions_not_met
                        .iter()
                        .map(refusal_condition_text)
                        .collect::<Vec<String>>()
                        .join(",")
                ),
                WritableMountRefusalVerdict::ReasonNotOnTheLegalRefusalList => {
                    "reason_not_on_the_d23_list".to_string()
                }
            };
            (
                    "refused",
                    error_member_of(&*refusal),
                    on_the_list.map_or_else(|| "-".to_string(), |reason| format!("{reason:?}")),
                    second_crop,
                    on_the_list
                        == Some(
                            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration,
                        ),
                )
        }
    };
    WritableMountProbe {
        criterion,
        selected_version: root_text(selected_version),
        mount,
        refusal_member,
        refusal_on_the_legal_refusal_list,
        second_crop,
        refused_for_the_system_configuration_witness: refused_for_the_witness,
    }
}

/// 第二截没满足的条件写成一个词（见证那一条带是哪一支）。
fn refusal_condition_text(condition: &RefusalConditionNotMet) -> String {
    match condition {
        RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage => {
            "criterion_false_when_recomputed".to_string()
        }
        RefusalConditionNotMet::DiskChangedByTheRefusedMount => "disk_changed".to_string(),
        RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet => {
            "read_only_read_back_outside_the_allowed_set".to_string()
        }
        RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(branch) => {
            format!("witness_criterion_true_without_read_faults:{branch:?}")
        }
        RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
            "system_configuration_unreadable_after_the_read_stage_criterion_true_without_read_faults"
                .to_string()
        }
        RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults => {
            "no_valid_root_in_the_root_ring_criterion_true_without_read_faults".to_string()
        }
    }
}

/// 阳性对照「见证」（登记 5.2）：每块盘两槽系统配置的 journal_tail 改成所选那一版末条记录的计数器加一，用 core 的系统配置编码函数
/// （`SystemConfiguration::to_slot`）重编、自证过，写回这份镜像。交回改了几槽（自证不过的槽不动）。
pub(super) fn raise_the_witnessed_counter_past_the_selected_version(
    pool: &mut MemoryPool,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> usize {
    let Ok(system_configuration) = choose_system_configuration(&*pool) else {
        return 0;
    };
    let Some(selected) = selected_version else {
        return 0;
    };
    let last_record_counter = scan_journal(&*pool, &system_configuration)
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == selected
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let slot_spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(LOCAL_SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut rewritten = 0usize;
    for device in pool.devices.values_mut() {
        for slot_index in 0..u64::try_from(LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).expect("2")
        {
            let offset = DeviceOffsetInBytes(slot_index * slot_spacing);
            let Ok(mut slot) = SystemConfiguration::parse_slot(&device.read(offset, slot_bytes))
            else {
                continue;
            };
            slot.quantities.journal_tail = last_record_counter + 1;
            device.write(offset, &slot.to_slot());
            rewritten += 1;
        }
    }
    rewritten
}

#[cfg(test)]
mod tests;
