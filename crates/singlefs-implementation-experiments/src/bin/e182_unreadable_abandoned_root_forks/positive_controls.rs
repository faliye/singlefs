use super::*;

// ============================================================================
// 十四、阳性对照（登记 5.1、5.2，每个起点都跑；一次没中，那一段作废）。
// ============================================================================

/// 取三个崩溃状态：什么都没持久、前一半的段全持久、整条流全持久。
pub(super) fn three_states_for_the_controls(history: &RecordedHistory) -> Vec<Vec<bool>> {
    let mut halfway = vec![false; history.writes.len()];
    for segment in history.segments.iter().take(history.segments.len() / 2) {
        for write_index in segment {
            halfway[*write_index] = true;
        }
    }
    vec![
        vec![false; history.writes.len()],
        halfway,
        persisted_with_every_write(history.writes.len()),
    ]
}

/// 这个状态里有没有一次持久了的写，与这个单元在它那块盘上的字节区间相交（相交的单元被换了字节也会被这次写盖回去，换字节就不起作用）。
fn persisted_write_overlaps_the_unit(
    history: &RecordedHistory,
    persisted: &[bool],
    unit: &UnitLog,
) -> bool {
    let unit_end =
        unit.offset_in_bytes + u64::try_from(unit.bytes.len()).expect("单元长度装得进 u64");
    history
        .writes
        .iter()
        .zip(persisted)
        .any(|(write, is_persisted)| {
            *is_persisted
                && write.device.0 == unit.device
                && write.offset.0 < unit_end
                && unit.offset_in_bytes < write.offset.0 + write.length_in_bytes()
        })
}

/// 检测对照：每个状态、每块留下的盘，把今天择到的根引用的一个单元（落在这块盘上、没被这个状态里持久了的写盖住的）在这块盘上换成别的字节，
/// 「被盖」必须判出。交回 (试了几次, 判出几次, 没有可换的单元的次数)。
pub(super) fn detection_control(history: &RecordedHistory) -> (u64, u64, u64) {
    let (mut trials, mut detected, mut without_a_unit) = (0u64, 0u64, 0u64);
    for persisted in three_states_for_the_controls(history) {
        let image = CrashImage {
            base: &history.base,
            writes: &history.writes,
            persisted: persisted.clone(),
        };
        for device_number in [0u32, 1u32] {
            let device = DeviceIdentity(device_number);
            let report = recover(&kept_disk(&image, device), JournalPolicy::Consult);
            let Some(log) = outcome_root(&report.outcome).and_then(|key| history.roots.get(&key))
            else {
                continue;
            };
            let untouched_unit = log.units.iter().find(|unit| {
                unit.device == device_number
                    && !unit_differs_in_the_image(&image, unit)
                    && !persisted_write_overlaps_the_unit(history, &persisted, unit)
            });
            let Some(unit) = untouched_unit else {
                without_a_unit += 1;
                continue;
            };
            let mut changed_base = history.base.clone();
            changed_base.flip_byte(device, DeviceOffsetInBytes(unit.offset_in_bytes), 0);
            let changed_image = CrashImage {
                base: &changed_base,
                writes: &history.writes,
                persisted: persisted.clone(),
            };
            trials += 1;
            if some_unit_of_the_root_is_covered(&changed_image, log, Some(device_number)) {
                detected += 1;
            }
        }
    }
    (trials, detected, without_a_unit)
}

pub(super) fn emit_detection_control(history: &RecordedHistory) -> bool {
    let (trials, detected, without_a_unit) = detection_control(history);
    let holds = trials > 0 && trials == detected;
    emit_result(&format!(
        "name=positive_control control=detect_covered history={} trials={trials} detected={detected} no_unit_to_change={without_a_unit} status={}",
        history_name(&history.specification),
        if holds { "holds" } else { "violated" }
    ));
    holds
}

// ----------------------------------------------------------------------------
// B4 的阳性对照（登记 5.2 最后一句，修订 12.12 补进装置）：每个起点取 2 个类 a 状态，把被藏根引用的一个单元在盘 0 上那一份换掉，必须改判类 b；
// 不改判，整段作废。类 a 只出在已确认档的带藏根历史里（登记 5.2：类 a = 被藏根已确认且它引用的单元两块盘上都没被盖），所以只在这一族上跑。
// ----------------------------------------------------------------------------

/// 登记 5.2 写的「盘 0」：对照换字节的那一份在设备号 0 的盘上。
const CONFIRMED_NONE_COVERED_CONTROL_DEVICE_NUMBER: u32 = 0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ConfirmedNoneCoveredControlReading {
    /// 挑到的类 a 状态数（至多 2）。
    class_confirmed_none_covered_states_chosen: u64,
    /// 其中找得到一个可换的单元（落在盘 0 上、没被这个状态里持久了的写盖住）的状态数，即试了几次。
    pub(super) trials: u64,
    /// 换之前重新判一遍仍是类 a 的次数（对照自检：不是类 a 的状态上换字节说明不了什么）。
    class_confirmed_none_covered_before_the_change: u64,
    /// 换之后判成类 b 的次数。
    changed_to_class_some_unit_covered: u64,
    /// 挑到的类 a 状态里盘 0 上没有可换的单元的状态数。
    no_unit_to_change: u64,
}

/// 对照的判定：没试成（0 次）记 not_made，不当通过；每次换之前是类 a、换之后是类 b 才算 holds；其余 violated。
fn the_confirmed_none_covered_control_status(
    trials: u64,
    class_confirmed_none_covered_before_the_change: u64,
    changed_to_class_some_unit_covered: u64,
) -> &'static str {
    if trials == 0 {
        "not_made"
    } else if class_confirmed_none_covered_before_the_change == trials
        && changed_to_class_some_unit_covered == trials
    {
        "holds"
    } else {
        "violated"
    }
}

/// 已确认档的可写挂载被拒时 `mount.refused` 里带的比法成员（C971 乙入判不出之后：读回全零的系统配置槽自证不过，当读错计，
/// 没有一块盘两槽都自证过，判不出，拒可写；这一形的对照记作废）。
const THE_COMPARISON_THAT_VOIDS_THE_CONFIRMED_NONE_COVERED_CONTROL: &str =
    "SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots";

/// 对照记「作废」：这条历史的挂载被拒、拒因里是那个比法成员，而且对照读数全零（没有类 a 状态可挑，因为拒了可写就没有根槽写、`row_publish_root_write` 是 `None`）。
/// 作废不是 not_made（那是装置没造出，算失败），也不是 holds（没试成，不能算过）；汇总里单独记数，不算检测对照失败。
fn the_confirmed_none_covered_control_is_voided(
    refused: Option<&str>,
    reading: &ConfirmedNoneCoveredControlReading,
) -> bool {
    refused.is_some_and(|text| {
        text.contains(THE_COMPARISON_THAT_VOIDS_THE_CONFIRMED_NONE_COVERED_CONTROL)
    }) && *reading == ConfirmedNoneCoveredControlReading::default()
}

/// 被藏根在这个状态里的类，镜像的基底单给（对照要换基底上的字节）。
fn the_hidden_root_class_with_the_base(
    history: &RecordedHistory,
    base: &MemoryPool,
    persisted: &[bool],
    hidden_log: &RootLog,
) -> &'static str {
    let image = CrashImage {
        base,
        writes: &history.writes,
        persisted: persisted.to_vec(),
    };
    class_of_the_hidden_root_in_the_state(
        &image,
        hidden_log,
        history.specification.family.confirmation(),
    )
}

/// 按段的次序找第一个类 a 状态（实例 2 写行那次发布持久之后、被藏根的单元两块盘上都没被盖）。
fn first_class_confirmed_none_covered_state_in_the_order(
    history: &RecordedHistory,
    hidden_log: &RootLog,
    row_publish_write_index: usize,
    shard_order: impl Iterator<Item = usize>,
) -> Result<Option<Vec<bool>>, String> {
    for shard_index in shard_order {
        let mut found: Option<Vec<bool>> = None;
        for_each_state_of_segment(
            &history.writes,
            &history.segments,
            shard_index,
            &mut |persisted| {
                if found.is_none()
                    && persisted[row_publish_write_index]
                    && the_hidden_root_class_with_the_base(
                        history,
                        &history.base,
                        persisted,
                        hidden_log,
                    ) == "class_confirmed_none_covered"
                {
                    found = Some(persisted.to_vec());
                }
                Ok(())
            },
        )?;
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

/// 对照用的类 a 状态：段的次序里最早的一个、倒着数最晚的一个（整条流末尾那个全持久的状态是类 a 时就是它），两个相同时只取一个。
fn class_confirmed_none_covered_states_for_the_control(
    history: &RecordedHistory,
) -> Result<Vec<Vec<bool>>, String> {
    let (Some(hidden_root), Some(row_publish_write_index)) =
        (history.hidden_root, history.row_publish_root_write)
    else {
        return Ok(Vec::new());
    };
    let Some(hidden_log) = history.roots.get(&hidden_root) else {
        return Ok(Vec::new());
    };
    let segment_count = history.segments.len();
    let earliest = first_class_confirmed_none_covered_state_in_the_order(
        history,
        hidden_log,
        row_publish_write_index,
        0..=segment_count,
    )?;
    let latest = first_class_confirmed_none_covered_state_in_the_order(
        history,
        hidden_log,
        row_publish_write_index,
        (0..=segment_count).rev(),
    )?;
    let mut chosen: Vec<Vec<bool>> = Vec::new();
    for state in [earliest, latest].into_iter().flatten() {
        if !chosen.contains(&state) {
            chosen.push(state);
        }
    }
    Ok(chosen)
}

/// 登记 5.2 最后一句的对照：每个类 a 状态里，把被藏根引用的一个单元在盘 0 上那一份换掉（翻一个字节），必须改判类 b。
fn class_confirmed_none_covered_to_class_some_unit_covered_control(
    history: &RecordedHistory,
) -> Result<ConfirmedNoneCoveredControlReading, String> {
    let mut reading = ConfirmedNoneCoveredControlReading::default();
    let Some(hidden_log) = history
        .hidden_root
        .and_then(|hidden_root| history.roots.get(&hidden_root))
    else {
        return Ok(reading);
    };
    for persisted in class_confirmed_none_covered_states_for_the_control(history)? {
        reading.class_confirmed_none_covered_states_chosen += 1;
        let image = CrashImage {
            base: &history.base,
            writes: &history.writes,
            persisted: persisted.clone(),
        };
        let untouched_unit = hidden_log.units.iter().find(|unit| {
            unit.device == CONFIRMED_NONE_COVERED_CONTROL_DEVICE_NUMBER
                && !unit_differs_in_the_image(&image, unit)
                && !persisted_write_overlaps_the_unit(history, &persisted, unit)
        });
        let Some(unit) = untouched_unit else {
            reading.no_unit_to_change += 1;
            continue;
        };
        reading.trials += 1;
        if the_hidden_root_class_with_the_base(history, &history.base, &persisted, hidden_log)
            == "class_confirmed_none_covered"
        {
            reading.class_confirmed_none_covered_before_the_change += 1;
        }
        let mut changed_base = history.base.clone();
        changed_base.flip_byte(
            DeviceIdentity(CONFIRMED_NONE_COVERED_CONTROL_DEVICE_NUMBER),
            DeviceOffsetInBytes(unit.offset_in_bytes),
            0,
        );
        if the_hidden_root_class_with_the_base(history, &changed_base, &persisted, hidden_log)
            == "class_some_unit_covered"
        {
            reading.changed_to_class_some_unit_covered += 1;
        }
    }
    Ok(reading)
}

/// 所有已确认档带藏根的历史合起来的对照读数（产物里的汇总行）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ConfirmedNoneCoveredControlTotal {
    pub(super) histories: u64,
    histories_not_made: u64,
    histories_violated: u64,
    /// 对照作废的历史数（已确认档的可写挂载被拒，没有类 a 状态）。
    histories_voided: u64,
    pub(super) trials: u64,
    changed_to_class_some_unit_covered: u64,
}

impl ConfirmedNoneCoveredControlTotal {
    /// 汇总行的判定与判决行里算进检测对照失败数的条数：有一条历史 violated 或停机算 1 条；一条都没试成（总试数 0）也算 1 条，不当通过；
    /// 但每条历史都是作废（`histories_voided == histories`）时不算：作废不是失败，也不是通过。
    pub(super) fn failures(&self) -> u64 {
        self.histories_violated
            + u64::from(
                self.histories > 0 && self.trials == 0 && self.histories_voided < self.histories,
            )
    }

    pub(super) fn status(&self) -> &'static str {
        if self.histories_violated > 0 {
            "violated"
        } else if self.trials == 0 && self.histories > 0 && self.histories_voided == self.histories
        {
            "voided"
        } else if self.trials == 0 {
            "not_made"
        } else {
            "holds"
        }
    }
}

/// 在产物里打这条历史的对照行，并把读数记进汇总。
pub(super) fn emit_the_confirmed_none_covered_control(
    history: &RecordedHistory,
    total: &mut ConfirmedNoneCoveredControlTotal,
) {
    total.histories += 1;
    match class_confirmed_none_covered_to_class_some_unit_covered_control(history) {
        Ok(reading) => {
            let status = if the_confirmed_none_covered_control_is_voided(
                history.mount.refused.as_deref(),
                &reading,
            ) {
                "voided"
            } else {
                the_confirmed_none_covered_control_status(
                    reading.trials,
                    reading.class_confirmed_none_covered_before_the_change,
                    reading.changed_to_class_some_unit_covered,
                )
            };
            total.trials += reading.trials;
            total.changed_to_class_some_unit_covered += reading.changed_to_class_some_unit_covered;
            match status {
                "voided" => total.histories_voided += 1,
                "not_made" => total.histories_not_made += 1,
                "violated" => total.histories_violated += 1,
                _ => {}
            }
            emit_result(&format!(
                "name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history={} class_confirmed_none_covered_states_chosen={} trials={} class_confirmed_none_covered_before_the_change={} changed_to_class_some_unit_covered={} no_unit_to_change={} status={status}",
                history_name(&history.specification),
                reading.class_confirmed_none_covered_states_chosen,
                reading.trials,
                reading.class_confirmed_none_covered_before_the_change,
                reading.changed_to_class_some_unit_covered,
                reading.no_unit_to_change
            ));
        }
        Err(reason) => {
            total.histories_violated += 1;
            emit_result(&format!(
                "name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history={} status=stopped detail={}",
                history_name(&history.specification),
                token_of(&reason)
            ));
        }
    }
}

pub(super) fn emit_the_confirmed_none_covered_control_total(
    total: &ConfirmedNoneCoveredControlTotal,
) {
    emit_result(&format!(
        "name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered_total histories={} histories_not_made={} histories_violated={} histories_voided={} trials={} changed_to_class_some_unit_covered={} status={}",
        total.histories,
        total.histories_not_made,
        total.histories_violated,
        total.histories_voided,
        total.trials,
        total.changed_to_class_some_unit_covered,
        total.status()
    ));
}

/// 剩下那块盘上有没有实例代号比 `instance` 新的、自证过的 journal 记录（装置自己扫：候选槽 + `JournalRecord::parse`，不经核心层的 `scan_journal`）。
fn kept_disk_holds_a_record_of_a_newer_instance(
    image: &CrashImage<'_>,
    device_number: u32,
    instance: u32,
) -> bool {
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    ImageReader::candidate_journal_slots(image, device_number)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|slot| {
            ImageReader::read(
                image,
                device_number,
                ring_start + slot * JOURNAL_RECORD_BYTES,
                record_bytes,
            )
        })
        .filter_map(|bytes| JournalRecord::parse(&bytes, expected_filesystem_identifier))
        .any(|record| record.instance.0 > instance)
}

/// 甲一、甲二的判法本身（登记 5.1 第三条）：在这条历史里按日志挑一个「新实例的记录与系统配置都已落到剩下那块盘、被藏根仍在环里」的状态，
/// 甲一、甲二都必须避开被藏根。没挑到状态记「没造出」（量作废，不当通过）。交回读数。
pub(super) fn arm_witness_control(history: &RecordedHistory) -> AnchorReading {
    let subject = history_name(&history.specification);
    let Some(hidden_root) = history.hidden_root else {
        return AnchorReading {
            anchor: "arm_witness_control".to_string(),
            subject,
            applicable: false,
            holds: true,
            detail: "no_hidden_root".to_string(),
        };
    };
    let mut examined = 0u64;
    let mut found: Option<String> = None;
    let mut failure: Option<String> = None;
    'segments: for shard_index in 0..=history.segments.len() {
        let mut candidate_states: Vec<Vec<bool>> = Vec::new();
        let collected = for_each_state_of_segment(
            &history.writes,
            &history.segments,
            shard_index,
            &mut |persisted| {
                let configuration_persisted = |device: u32| {
                    history
                        .writes
                        .iter()
                        .zip(persisted)
                        .any(|(write, is_persisted)| {
                            *is_persisted
                                && write.kind == StepKind::SystemConfigurationSlot
                                && write.device.0 == device
                        })
                };
                let record_persisted = |device: u32| {
                    history
                        .writes
                        .iter()
                        .zip(persisted)
                        .any(|(write, is_persisted)| {
                            *is_persisted
                                && write.kind == StepKind::JournalRecord
                                && write.device.0 == device
                        })
                };
                if [0u32, 1u32]
                    .into_iter()
                    .any(|device| configuration_persisted(device) && record_persisted(device))
                {
                    candidate_states.push(persisted.to_vec());
                }
                Ok(())
            },
        );
        if collected.is_err() {
            continue;
        }
        for persisted in candidate_states {
            let image = CrashImage {
                base: &history.base,
                writes: &history.writes,
                persisted,
            };
            for device_number in [0u32, 1u32] {
                let device = DeviceIdentity(device_number);
                let kept = kept_disk(&image, device);
                let today_report = recover(&kept, JournalPolicy::Consult);
                if outcome_root(&today_report.outcome) != Some(hidden_root)
                    || !kept_disk_holds_a_record_of_a_newer_instance(
                        &image,
                        device_number,
                        hidden_root.0,
                    )
                    || newest_configuration_instance_on_the_kept_disk(&kept, device)
                        .is_none_or(|instance| instance <= hidden_root.0)
                {
                    continue;
                }
                examined += 1;
                for arm in [Arm::NewerInstanceRecord, Arm::ConfigurationInstance] {
                    let result = arm_result_on_the_kept_disk(
                        &image,
                        device,
                        &history.pool_geometry,
                        arm,
                        &today_report,
                    );
                    if result.avoided_roots == 0 {
                        failure = Some(format!(
                            "{} 没避开被藏根（只留盘 {device_number}，段 {shard_index}）",
                            arm.name()
                        ));
                    }
                }
                found = Some(format!("segment={shard_index},kept_disk={device_number}"));
                break 'segments;
            }
        }
    }
    AnchorReading {
        anchor: "arm_witness_control".to_string(),
        subject,
        applicable: found.is_some(),
        holds: failure.is_none(),
        detail: format!(
            "state={} examined={examined} failure={}",
            found.as_deref().unwrap_or("none_found"),
            failure.as_deref().unwrap_or("none")
        ),
    }
}

#[cfg(test)]
mod tests;
