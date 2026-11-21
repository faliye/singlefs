use super::*;

// ============================================================================
// 十三、常量回比、第七节锚点、闭式计数。出自被测条款的锚点（A1–A4）不符走失败条款 Z7（条款可能错）；
//    独立算出的（I1、I2）与常量回比不符整段作废（登记第十一节）。
//    登记第三节那几份源文件的 sha256 前 16 位与仓里现读值的比对由 `scripts/` 下的对比脚本做，装置不读别的包的源码。
// ============================================================================

struct ConstantCheck {
    constant: &'static str,
    local_value: u64,
    crates_value: u64,
    crates_source: &'static str,
}

fn local_constant_checks() -> Vec<ConstantCheck> {
    vec![
        ConstantCheck {
            constant: "slot_bytes",
            local_value: SLOT_BYTES_LOCAL,
            crates_value: SLOT_BYTES,
            crates_source: "singlefs_format::SLOT_BYTES",
        },
        ConstantCheck {
            constant: "unit_area_start_slot",
            local_value: UNIT_AREA_START_SLOT_LOCAL,
            crates_value: UNIT_AREA_START_SLOT,
            crates_source: "singlefs_format::UNIT_AREA_START_SLOT",
        },
        ConstantCheck {
            constant: "journal_ring_bytes",
            local_value: JOURNAL_RING_BYTES_LOCAL,
            crates_value: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            crates_source: "singlefs_format::JOURNAL_RING_DEFAULT_BYTES",
        },
        ConstantCheck {
            constant: "root_ring_slots_per_region",
            local_value: ROOT_RING_SLOTS_PER_REGION_LOCAL,
            crates_value: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
            crates_source: "RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM",
        },
        ConstantCheck {
            constant: "root_ring_regions",
            local_value: ROOT_RING_REGIONS_LOCAL,
            crates_value: singlefs_format::ROOT_RING_REGIONS,
            crates_source: "singlefs_format::ROOT_RING_REGIONS",
        },
        ConstantCheck {
            constant: "warm_up_publishes_at_most",
            local_value: u64::try_from(WARM_UP_PUBLISHES_AT_MOST_LOCAL).expect("3 装得进 u64"),
            crates_value:
                singlefs_core::admission::WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
            crates_source:
                "singlefs_core::admission::WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH",
        },
        ConstantCheck {
            constant: "data_unit_payload_bytes",
            local_value: u64::try_from(DATA_UNIT_PAYLOAD_BYTES_LOCAL).expect("32634 装得进 u64"),
            crates_value: u64::try_from(data_unit_payload_capacity()).expect("装得进 u64"),
            crates_source: "singlefs_core::unit::data_unit_payload_capacity()",
        },
        ConstantCheck {
            constant: "system_configuration_slot_bytes",
            local_value: u64::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096"),
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
            crates_source: "singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES",
        },
        ConstantCheck {
            constant: "fixed_structure_slot_spacing_minimum",
            local_value: u64::from(FIXED_STRUCTURE_SLOT_SPACING_LOCAL),
            crates_value: singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
            crates_source: "singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES",
        },
    ]
}

/// I1：每块盘的单元区槽数 = 盘字节数 ÷ 槽字节数 − 单元区起点槽号（登记第七节，与 E180 第七节 B1 同一个算式；两个字面数是用命令算过的）。
fn unit_area_slots_of(geometry: DiskGeometry) -> u64 {
    geometry.device_bytes() / SLOT_BYTES_LOCAL - UNIT_AREA_START_SLOT_LOCAL
}

const UNIT_AREA_SLOTS_DEFAULT_PINNED: u64 = 211_968;
const UNIT_AREA_SLOTS_NARROW_PINNED: u64 = 146_465;

/// 一个只有 `kind` 种类的合成写（闭式计数的取样点只看写的种类，不看字节）。
fn synthetic_write(kind: StepKind) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(0),
        kind,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(0),
        contents: singlefs_verification::memory_pool::WrittenContents::Bytes(vec![0; 512]),
    }
}

/// I2：一段里 `in_place` 次原地写、`unit` 次单元写，按装置的枚举器数出几个状态，与独立的闭式比。
/// 全量展开的段（写数不超过上界）闭式是 2^m − 1（m = 写数，全持久那个留给下一段）；甲二是 2^k × (单元写非空 ? 2 : 1) − 1。
fn enumerated_and_closed_form_state_counts(in_place: usize, unit: usize) -> (u64, u64) {
    let mut writes: Vec<RetainedWrite> = Vec::new();
    for _ in 0..in_place {
        writes.push(synthetic_write(StepKind::JournalRecord));
    }
    for _ in 0..unit {
        writes.push(synthetic_write(StepKind::UnitWrite));
    }
    let segments = vec![(0..writes.len()).collect::<Vec<usize>>()];
    let mut seen: u64 = 0;
    let enumeration = for_each_state_of_segment(&writes, &segments, 0, &mut |_persisted| {
        seen += 1;
        Ok(())
    });
    if enumeration.is_err() {
        // 枚举器自己核出个数与它的闭式不同：用一个必然对不上的数把这一格报出来。
        seen = u64::MAX;
    }
    let total = in_place + unit;
    let closed_form = if total <= FULL_EXPANSION_WRITES_AT_MOST_LOCAL {
        (1u64 << total) - 1
    } else {
        (1u64 << in_place) * if unit == 0 { 1 } else { 2 } - 1
    };
    (seen, closed_form)
}

/// 一条锚点的读数：成立 / 不成立 / 这条历史上不适用（挂载被拒、没有可比的对象）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AnchorReading {
    pub(super) anchor: String,
    pub(super) subject: String,
    pub(super) applicable: bool,
    pub(super) holds: bool,
    pub(super) detail: String,
}

/// A1：实例 2 每一次根槽写落在 txg mod 3 那个区域、按区域归属 0 / 1 / 0 归盘，偏移是 checker 独立解码出的那个槽位。
pub(super) fn anchor_root_slot_writes_land_by_the_ring_rule(
    history: &RecordedHistory,
) -> AnchorReading {
    let positions = checker_image::root_slot_positions(&history.pool_geometry);
    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();
    for write in history
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
    {
        let Some(root) = write
            .bytes()
            .and_then(|bytes| RootRecord::parse_slot(bytes, &FILESYSTEM_IDENTIFIER))
        else {
            violations.push("根槽写解不出根记录".to_string());
            continue;
        };
        checked += 1;
        let (region, slot) = ring_slot_of_txg(root.checkpoint_txg.0);
        let expected_device = REGION_DEVICES_LOCAL[usize::try_from(region).expect("区域号")];
        let position = positions
            .iter()
            .find(|(candidate_region, candidate_slot, _, _)| {
                *candidate_region == region && *candidate_slot == slot
            });
        let landed_right = position.is_some_and(|(_, _, device, offset)| {
            *device == expected_device
                && write.device.0 == expected_device
                && write.offset.0 == *offset
        });
        if !landed_right {
            violations.push(format!(
                "txg {} 的根槽写落在盘 {} 偏移 {}",
                root.checkpoint_txg.0, write.device.0, write.offset.0
            ));
        }
    }
    AnchorReading {
        anchor: "root_slot_writes_land_by_the_ring_rule".to_string(),
        subject: history_name(&history.specification),
        applicable: checked > 0,
        holds: violations.is_empty(),
        detail: format!(
            "root_slot_writes_checked={checked} violations={}",
            violations.join(";")
        ),
    }
}

/// A2：每次可写挂载的暖机零单元发布至多 3 次。
pub(super) fn anchor_warm_up_publishes_are_at_most_three(
    history: &RecordedHistory,
) -> AnchorReading {
    AnchorReading {
        anchor: "warm_up_publishes_are_at_most_three".to_string(),
        subject: history_name(&history.specification),
        applicable: history.mount.refused.is_none(),
        holds: history.mount.warm_up_publishes <= WARM_UP_PUBLISHES_AT_MOST_LOCAL,
        detail: format!("warm_up_publishes={}", history.mount.warm_up_publishes),
    }
}

/// A3 的期望：实例 2 第一次发布的 txg = max(装置日志里根环中读得出的根的 txg, 读得出的记录的 txg) + 1，读回全零的不算。
fn first_publish_txg_expected_by_the_clause(history: &RecordedHistory) -> Option<u64> {
    let hidden = history.hidden_root;
    let records_are_hidden_too = matches!(history.specification.family, Family::HiddenRoot(_));
    let newest_root_txg = checker_image::valid_roots(&history.base, &history.pool_geometry)
        .into_iter()
        .map(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg))
        .filter(|key| Some(*key) != hidden)
        .map(|(_, txg)| txg)
        .max()?;
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut newest_record_txg = 0u64;
    for device in [0u32, 1u32] {
        for slot in ImageReader::candidate_journal_slots(&history.base, device).unwrap_or_default()
        {
            let offset = ring_start + slot * JOURNAL_RECORD_BYTES;
            let Some(bytes) = ImageReader::read(&history.base, device, offset, record_bytes) else {
                continue;
            };
            let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) else {
                continue;
            };
            let key = (record.instance.0, record.checkpoint_txg.0);
            if records_are_hidden_too && Some(key) == hidden {
                continue;
            }
            newest_record_txg = newest_record_txg.max(record.checkpoint_txg.0);
        }
    }
    Some(newest_root_txg.max(newest_record_txg) + 1)
}

pub(super) fn anchor_first_publish_txg_follows_the_newest_readable(
    history: &RecordedHistory,
) -> AnchorReading {
    let expected = first_publish_txg_expected_by_the_clause(history);
    let actual = history.mount.row_publish_root.map(|(_, txg)| txg);
    AnchorReading {
        anchor: "first_publish_txg_follows_the_newest_readable".to_string(),
        subject: history_name(&history.specification),
        applicable: history.mount.refused.is_none(),
        holds: expected.is_some() && expected == actual,
        detail: format!("expected={expected:?} actual={actual:?}"),
    }
}

/// A4：已确认档里只把 R_h 的根槽与记录读回全零、不罩较新的系统配置槽时，实例 2 的可写挂载在取号之前被拒、两块盘逐字节不变。
fn anchor_confirmed_without_the_newer_configuration_is_refused_before_any_write(
) -> Result<AnchorReading, String> {
    let specification = HistorySpecification {
        family: Family::HiddenRoot(Confirmation::Confirmed),
        geometry: DiskGeometry::Default,
        first_file_units: 1,
        overwrites_before_the_hidden_publish: 5,
        overwrite_shape: OverwriteShape::OneUnit,
    };
    let first = run_the_first_instance(&specification)?;
    let (base, fault_ranges) = base_and_fault_ranges(&specification, &first)?;
    let without_the_newer_configuration: Vec<ReadsZeroRange> = fault_ranges
        .into_iter()
        .filter(|range| range.label != "newer_system_configuration_slot")
        .collect();
    let second = run_the_second_instance(&specification, &base, without_the_newer_configuration);
    let wrote_nothing = second
        .operations
        .iter()
        .all(|operation| matches!(operation.operation.kind, RecordedOperationKind::Barrier));
    Ok(AnchorReading {
        anchor: "confirmed_without_the_newer_configuration_is_refused_before_any_write".to_string(),
        subject: history_name(&specification),
        applicable: true,
        holds: second.mount.refused.is_some() && second.devices_equal_the_base && wrote_nothing,
        detail: format!(
            "refused={:?} devices_equal_the_base={} operations={}",
            second
                .mount
                .refused
                .as_deref()
                .map(|text| text.chars().take(80).collect::<String>()),
            second.devices_equal_the_base,
            second.operations.len()
        ),
    })
}

/// I1 与 I2：不建历史就能算的两类锚点。
fn static_anchors() -> Vec<AnchorReading> {
    let mut readings = Vec::new();
    for (geometry, pinned) in [
        (DiskGeometry::Default, UNIT_AREA_SLOTS_DEFAULT_PINNED),
        (DiskGeometry::Narrow, UNIT_AREA_SLOTS_NARROW_PINNED),
    ] {
        let computed = unit_area_slots_of(geometry);
        readings.push(AnchorReading {
            anchor: "unit_area_slots_pinned".to_string(),
            subject: geometry.name().to_string(),
            applicable: true,
            holds: computed == pinned,
            detail: format!("unit_area_slots={computed} pinned={pinned}"),
        });
    }
    for (in_place, unit) in [
        (0usize, 0usize),
        (3, 0),
        (8, 8),
        (16, 0),
        (0, 16),
        (6, 11),
        (17, 0),
        (0, 17),
        (5, 40),
        (9, 30),
    ] {
        let (seen, closed_form) = enumerated_and_closed_form_state_counts(in_place, unit);
        readings.push(AnchorReading {
            anchor: "enumerated_state_count_equals_the_closed_form".to_string(),
            subject: format!("in_place={in_place},unit={unit}"),
            applicable: true,
            holds: seen == closed_form,
            detail: format!("enumerated={seen} closed_form={closed_form}"),
        });
    }
    readings
}

pub(super) fn token_of(text: &str) -> String {
    text.replace([' ', '\n', '\t'], "_")
}

pub(super) fn emit_anchor_reading(reading: &AnchorReading) {
    let status = match (reading.applicable, reading.holds) {
        (false, _) => "not_applicable",
        (true, true) => "holds",
        (true, false) => "violated",
    };
    emit_result(&format!(
        "name=anchor anchor={} subject={} status={status} detail={}",
        reading.anchor,
        token_of(&reading.subject),
        token_of(&reading.detail)
    ));
}

/// 模式 `anchors` 与单测 `constants_and_anchors_all_pass` 共用：常量回比、I1、I2、A4。交回没过的条数。
pub(super) fn run_constants_and_anchors() -> u64 {
    let mut mismatches = 0u64;
    for check in local_constant_checks() {
        let matches = check.local_value == check.crates_value;
        if !matches {
            mismatches += 1;
        }
        emit_result(&format!(
            "name=constant_check constant={} local={} crates={} crates_source={} match={matches}",
            check.constant, check.local_value, check.crates_value, check.crates_source
        ));
    }
    emit_result(&format!(
        "name=constant_check constant=full_expansion_writes_at_most local={FULL_EXPANSION_WRITES_AT_MOST_LOCAL} crates=none crates_source=registration_5_decided_here match=not_comparable"
    ));
    for reading in static_anchors() {
        if !reading.holds {
            mismatches += 1;
        }
        emit_anchor_reading(&reading);
    }
    match anchor_confirmed_without_the_newer_configuration_is_refused_before_any_write() {
        Ok(reading) => {
            if !reading.holds {
                mismatches += 1;
            }
            emit_anchor_reading(&reading);
        }
        Err(reason) => {
            mismatches += 1;
            emit_result(&format!(
                "name=anchor anchor=confirmed_without_the_newer_configuration_is_refused_before_any_write subject=build status=violated detail={}",
                token_of(&reason)
            ));
        }
    }
    mismatches
}

#[cfg(test)]
mod tests;
