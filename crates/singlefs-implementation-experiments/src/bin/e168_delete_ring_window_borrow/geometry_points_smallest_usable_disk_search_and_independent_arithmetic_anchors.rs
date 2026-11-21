//! 几何取样点与第七节 B 类锚点：最小可用盘的搜索与独立算术的锚点。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 几何取样点与第七节 B 类锚点
// ---------------------------------------------------------------------------------------------------------------------

/// 文件大小的装置上限（数据单元）：内容在内存里成一整份、P1 每个 N 另找一块盘；梯子续加到这里还没饱和的，照「撞上沿」报
/// `ladder_ceiling=apparatus_file_size_limit`（第十二节修订：登记 8.4 的上沿只写了「写不进」，没写装置的内存界）。
pub(super) const LARGEST_FILE_DATA_UNITS: u64 = 32768;
/// P1 搜索的单元区上限（槽）：倍增到这里仍做不成就报找不到。
const SMALLEST_USABLE_SEARCH_LIMIT_SLOTS: u64 = 3 + 2 * (1 << 20);

pub(super) fn make_filesystem_smallest_geometry(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::MakeFilesystemSmallest,
        device_bytes: make_filesystem_smallest_device_bytes(SMALL_JOURNAL_RING_BYTES),
        journal_ring_bytes: SMALL_JOURNAL_RING_BYTES,
        slots_per_region,
    }
}

/// P1 的一块候选盘：journal 环 `journal_ring_bytes`（修订第 15 条先定下来的那一个）、单元区 `unit_area_slots` 槽。
pub(super) fn smallest_usable_candidate(
    slots_per_region: u64,
    journal_ring_bytes: u64,
    unit_area_slots: u64,
) -> Geometry {
    Geometry {
        point: DeviceSizePoint::SmallestUsable,
        device_bytes: device_bytes_with_unit_area(journal_ring_bytes, unit_area_slots),
        journal_ring_bytes,
        slots_per_region,
    }
}

pub(super) fn narrow_geometry(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::Narrow,
        device_bytes: device_bytes_with_unit_area(
            NARROW_JOURNAL_RING_BYTES,
            NARROW_UNIT_AREA_SLOTS,
        ),
        journal_ring_bytes: NARROW_JOURNAL_RING_BYTES,
        slots_per_region,
    }
}

#[cfg(test)]
pub(super) fn narrow_geometry_with_the_ring_window_reserved(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::Narrow,
        device_bytes: device_bytes_with_unit_area(
            NARROW_JOURNAL_RING_BYTES,
            NARROW_UNIT_AREA_SLOTS_WITH_THE_RING_WINDOW_RESERVED,
        ),
        journal_ring_bytes: NARROW_JOURNAL_RING_BYTES,
        slots_per_region,
    }
}

pub(super) fn product_geometry(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::Product,
        device_bytes: PRODUCT_DEVICE_BYTES,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        slots_per_region,
    }
}

/// 5.1 第 1、2 步（写进 N 个单元的文件、重挂载）做不做得成。
fn pool_is_written(geometry: Geometry, data_units: u64) -> bool {
    matches!(
        pool_with_a_file(geometry, data_units),
        PoolWithAFile::Written(_)
    )
}

/// 写进 N 个单元、重挂载、按 `ledger_for_the_fill` 压满（5.3）之后每块盘空闲槽的最大值；写不进或压不满（压之前式子可用就
/// ≤ −16384）交回 None。
pub(super) fn free_slots_after_the_fill(
    geometry: Geometry,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> Option<u64> {
    let PoolWithAFile::Written(mut pool) = pool_with_a_file(geometry, data_units) else {
        return None;
    };
    (ledger_for_the_fill.set_ledger)(ledger_for_the_fill.ledger);
    let outcome = fill_from_the_unit_area_end(&mut pool);
    (ledger_for_the_fill.set_ledger)(None);
    match outcome {
        FillOutcome::WithinOneSlotAtOrBelowZero { free_slots, .. } => free_slots.into_iter().max(),
        FillOutcome::AlreadyBelowTheIntervalBeforeFilling { .. } => None,
    }
}

/// 5.1 第 1–3 步（写进、重挂载、压满）做不做得成：读法 C 的 P1 要的那一格（登记第十二节修订第 10 条）。
fn pool_is_written_and_fillable(
    geometry: Geometry,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> bool {
    free_slots_after_the_fill(geometry, data_units, ledger_for_the_fill).is_some()
}

/// P1 搜索的一步：单元区 3 + 2 × 步数 槽（从 P0 的 3 槽起按 2 槽一步）。
fn smallest_usable_unit_area_slots_at_step(step: u64) -> u64 {
    3 + 2 * step
}

/// 从 `first_step` 起往大找 `works` 做得成的最小步数：先倍增离 `first_step` 的距离找做得成的上界，再二分；二分收口时比它小一步
/// （小 2 槽）那一格已经判过做不成（`first_step` 本身做得成时，小一步那一格不在这次搜索里）。单元区超过
/// [`SMALLEST_USABLE_SEARCH_LIMIT_SLOTS`] 仍做不成交回 None。交回 (最小步数, 判过的格数)。
fn smallest_step_that_works(first_step: u64, works: &dyn Fn(u64) -> bool) -> (Option<u64>, u64) {
    let mut judged = 1;
    if works(first_step) {
        return (Some(first_step), judged);
    }
    let mut known_unusable = first_step;
    let mut distance = 1;
    let mut known_usable = loop {
        let candidate = first_step + distance;
        if smallest_usable_unit_area_slots_at_step(candidate) > SMALLEST_USABLE_SEARCH_LIMIT_SLOTS {
            return (None, judged);
        }
        judged += 1;
        if works(candidate) {
            break candidate;
        }
        known_unusable = candidate;
        distance *= 2;
    };
    while known_usable - known_unusable > 1 {
        let middle = known_unusable + (known_usable - known_unusable) / 2;
        judged += 1;
        if works(middle) {
            known_usable = middle;
        } else {
            known_unusable = middle;
        }
    }
    (Some(known_usable), judged)
}

/// 修订第 15 条「先定环」那一步，在搜索上沿那块盘（单元区 [`SMALLEST_USABLE_SEARCH_LIMIT_SLOTS`] 槽）上试一个环的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RingProbeOutcome {
    /// 5.1 第 1、2 步做成了：这个环装得下第 1 步那次写。
    Written,
    /// 第 1 步那次写被 journal 记录数上限拒，带着那次发布的记录条数。
    RecordsOverTheLimit(u64),
    /// 在这一步被别的原因拒（不是记录数上限）。
    RefusedAt(&'static str),
}

/// P1 找不到盘的原因（`E name=smallest_usable` 那一行的 `not_found_reason`；只加输出、不改判据）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmallestUsableNotFoundReason {
    /// 搜索上沿那块盘上第 1 步那次写被 journal 记录数上限拒，而环已经放大满 [`JOURNAL_RING_ENLARGEMENTS_AT_MOST`] 次。
    JournalRecordsOfOnePublishExceedTheLimit(JournalRecordsOverTheLimit),
    /// 搜索上沿那块盘上第 1、2 步在这一步被别的原因拒。
    FileNotWrittenOnTheLargestDiskSearched { step: &'static str },
    /// 第 1、2 步找得到盘，第 1–3 步到搜索上沿都做不成（压不满）。
    NotFillableOnAnyDiskSearched,
}

impl SmallestUsableNotFoundReason {
    pub(super) fn label(&self) -> String {
        match self {
            SmallestUsableNotFoundReason::JournalRecordsOfOnePublishExceedTheLimit(_) => {
                "journal_records_of_one_publish_exceed_the_limit".to_owned()
            }
            SmallestUsableNotFoundReason::FileNotWrittenOnTheLargestDiskSearched { step } => {
                format!("file_not_written_on_the_largest_disk_searched_step_{step}")
            }
            SmallestUsableNotFoundReason::NotFillableOnAnyDiskSearched => {
                "not_fillable_on_any_disk_searched".to_owned()
            }
        }
    }
}

/// 修订第 15 条「先定环」：P1(S, N) 的 journal 环。
#[derive(Clone, Debug, PartialEq, Eq)]
struct JournalRingOfTheSmallestUsableDisk {
    /// 定下来的环；定不下来时是最后试的那一个。
    pub(super) journal_ring_bytes: u64,
    /// 试过的每个环与它在搜索上沿那块盘上的结局，按试的次序。
    pub(super) tried: Vec<(u64, RingProbeOutcome)>,
    /// 定不下来的原因；None 是定下来了（最后那个环在搜索上沿那块盘上第 1、2 步做得成）。
    not_settled: Option<SmallestUsableNotFoundReason>,
}

/// 修订第 15 条「先定环」：从 1 MiB 起，在搜索上沿那块盘上写进 N 个单元；第 1 步那次写被 journal 记录数上限拒，就把环换成装得下
/// 那次发布记录条数的最小合法环（[`smallest_legal_journal_ring_holding`]）再试，至多放大 `enlargements_at_most` 次。搜索上沿那块盘
/// 是找盘那一步（[`smallest_step_that_works`]）最后会判的那一格，空间不是这一步的约束。
fn journal_ring_of_the_smallest_usable_disk(
    slots_per_region: u64,
    data_units: u64,
    enlargements_at_most: usize,
) -> JournalRingOfTheSmallestUsableDisk {
    let mut journal_ring_bytes = SMALL_JOURNAL_RING_BYTES;
    let mut tried = Vec::new();
    let mut enlargements = 0;
    loop {
        let at_the_search_limit = smallest_usable_candidate(
            slots_per_region,
            journal_ring_bytes,
            SMALLEST_USABLE_SEARCH_LIMIT_SLOTS,
        );
        let not_settled = match pool_with_a_file(at_the_search_limit, data_units) {
            PoolWithAFile::Written(_) => {
                tried.push((journal_ring_bytes, RingProbeOutcome::Written));
                None
            }
            PoolWithAFile::Refused {
                journal_records_over_the_limit: Some(over_the_limit),
                ..
            } => {
                if over_the_limit.record_limit
                    != journal_record_limit_by_the_format(journal_ring_bytes)
                {
                    stop_with(
                        "journal_record_limit_differs_from_the_format",
                        &format!(
                            "journal_ring_bytes={journal_ring_bytes} implementation={} local={}",
                            over_the_limit.record_limit,
                            journal_record_limit_by_the_format(journal_ring_bytes)
                        ),
                    );
                }
                tried.push((
                    journal_ring_bytes,
                    RingProbeOutcome::RecordsOverTheLimit(over_the_limit.records_of_the_publish),
                ));
                let holding =
                    smallest_legal_journal_ring_holding(over_the_limit.records_of_the_publish);
                if enlargements < enlargements_at_most && holding > journal_ring_bytes {
                    journal_ring_bytes = holding;
                    enlargements += 1;
                    continue;
                }
                Some(
                    SmallestUsableNotFoundReason::JournalRecordsOfOnePublishExceedTheLimit(
                        over_the_limit,
                    ),
                )
            }
            PoolWithAFile::Refused {
                step,
                journal_records_over_the_limit: None,
                ..
            } => {
                tried.push((journal_ring_bytes, RingProbeOutcome::RefusedAt(step)));
                Some(SmallestUsableNotFoundReason::FileNotWrittenOnTheLargestDiskSearched { step })
            }
        };
        return JournalRingOfTheSmallestUsableDisk {
            journal_ring_bytes,
            tried,
            not_settled,
        };
    }
}

/// P1(S, N) 的搜索结局（登记 5.7；读法 C 见第十二节修订第 10 条；环见修订第 15 条）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SmallestUsableSearch {
    /// 先定下来的环（修订第 15 条）；下面两个最小盘都在这个环上找。
    journal_ring: JournalRingOfTheSmallestUsableDisk,
    /// 原定义：5.1 第 1、2 步做得成的最小盘（从 P0 起找）。
    pub(super) written: Option<Geometry>,
    judged_for_written: u64,
    /// 读法 C：第 1–3 步做得成的最小盘（从原定义那一格起往大找）；P1 取它。
    pub(super) written_and_fillable: Option<Geometry>,
    judged_for_fillable: u64,
    /// 环大于 1 MiB 时，小一档的环在读法 C 那块盘的单元区上第 1 步那次写被 journal 记录数上限拒（修订第 15 条的最小性；不拒就停机）；
    /// None 是没核（环就是 1 MiB，或没找到盘）。
    smaller_journal_ring_refused_by_the_record_limit: Option<bool>,
    /// 找不到盘的原因；None 是找到了。
    not_found_reason: Option<SmallestUsableNotFoundReason>,
}

/// P1(S, N)：先定环（修订第 15 条），再在那个环上照原定义从 P0 起找第 1、2 步做得成的最小盘，再从那一格起往大找第 1–3 步做得成的
/// 最小盘（读法 C）。压满按 `ledger_for_the_fill` 那一份账压（N 臂两个 0，W 臂另记那一份）。
pub(super) fn smallest_usable_geometry(
    slots_per_region: u64,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> SmallestUsableSearch {
    smallest_usable_geometry_with_the_ring_enlarged_at_most(
        slots_per_region,
        data_units,
        ledger_for_the_fill,
        JOURNAL_RING_ENLARGEMENTS_AT_MOST,
    )
}

/// [`smallest_usable_geometry`]，环至多放大 `ring_enlargements_at_most` 次（单测拿 0 造「环不许放大」那一格的找不到）。
fn smallest_usable_geometry_with_the_ring_enlarged_at_most(
    slots_per_region: u64,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
    ring_enlargements_at_most: usize,
) -> SmallestUsableSearch {
    let journal_ring = journal_ring_of_the_smallest_usable_disk(
        slots_per_region,
        data_units,
        ring_enlargements_at_most,
    );
    if let Some(reason) = journal_ring.not_settled {
        return SmallestUsableSearch {
            journal_ring,
            written: None,
            judged_for_written: 0,
            written_and_fillable: None,
            judged_for_fillable: 0,
            smaller_journal_ring_refused_by_the_record_limit: None,
            not_found_reason: Some(reason),
        };
    }
    let journal_ring_bytes = journal_ring.journal_ring_bytes;
    let candidate = |step: u64| {
        smallest_usable_candidate(
            slots_per_region,
            journal_ring_bytes,
            smallest_usable_unit_area_slots_at_step(step),
        )
    };
    let (written_step, judged_for_written) =
        smallest_step_that_works(0, &|step| pool_is_written(candidate(step), data_units));
    let (fillable_step, judged_for_fillable) = match written_step {
        Some(first_step) => smallest_step_that_works(first_step, &|step| {
            pool_is_written_and_fillable(candidate(step), data_units, ledger_for_the_fill)
        }),
        None => stop_with(
            "p1_search_misses_the_written_search_limit",
            &format!("S={slots_per_region} data_units={data_units} journal_ring_bytes={journal_ring_bytes}"),
        ),
    };
    let mut search = SmallestUsableSearch {
        journal_ring,
        written: written_step.map(candidate),
        judged_for_written,
        written_and_fillable: fillable_step.map(candidate),
        judged_for_fillable,
        smaller_journal_ring_refused_by_the_record_limit: None,
        not_found_reason: None,
    };
    match search.written_and_fillable {
        None => {
            search.not_found_reason =
                Some(SmallestUsableNotFoundReason::NotFillableOnAnyDiskSearched);
        }
        Some(found) if journal_ring_bytes > SMALL_JOURNAL_RING_BYTES => {
            let one_step_smaller = smallest_usable_candidate(
                slots_per_region,
                journal_ring_bytes - JOURNAL_RING_STEP_BYTES,
                found.unit_area_slots(),
            );
            let is_refused_by_the_record_limit = matches!(
                pool_with_a_file(one_step_smaller, data_units),
                PoolWithAFile::Refused {
                    journal_records_over_the_limit: Some(_),
                    ..
                }
            );
            if !is_refused_by_the_record_limit {
                stop_with(
                    "p1_journal_ring_one_step_smaller_holds_the_write",
                    &format!("S={slots_per_region} data_units={data_units} journal_ring_bytes={journal_ring_bytes} unit_area_slots={}", found.unit_area_slots()),
                );
            }
            search.smaller_journal_ring_refused_by_the_record_limit = Some(true);
        }
        Some(_) => {}
    }
    search
}

/// `E name=smallest_usable` 那一行：原定义（第 1、2 步）与读法 C（第 1–3 步）两个最小盘、各判过几格、搜索上沿（登记 F8）；
/// 修订第 15 条加环、试过的环、小一档的环核没核、找不到的原因四个字段（只加输出）。
pub(super) fn smallest_usable_line(
    slots_per_region: u64,
    data_units: u64,
    search: &SmallestUsableSearch,
) -> String {
    let unit_area_slots_or_not_found = |geometry: Option<Geometry>| {
        geometry.map_or_else(
            || "not_found".to_owned(),
            |geometry| geometry.unit_area_slots().to_string(),
        )
    };
    format!(
        "E name=smallest_usable S={slots_per_region} data_units={data_units} judged_steps_1_2={} unit_area_slots_steps_1_2={} judged_steps_1_3={} unit_area_slots={} search_limit_unit_area_slots={SMALLEST_USABLE_SEARCH_LIMIT_SLOTS} above_the_narrow_point={} journal_ring_bytes={} journal_ring_tried={:?} smaller_journal_ring_refused_by_the_record_limit={} not_found_reason={}",
        search.judged_for_written,
        unit_area_slots_or_not_found(search.written),
        search.judged_for_fillable,
        unit_area_slots_or_not_found(search.written_and_fillable),
        search
            .written_and_fillable
            .is_none_or(|geometry| geometry.unit_area_slots() > NARROW_UNIT_AREA_SLOTS),
        search.journal_ring.journal_ring_bytes,
        search.journal_ring.tried,
        search
            .smaller_journal_ring_refused_by_the_record_limit
            .map_or_else(|| "not_checked".to_owned(), |is_refused| is_refused.to_string()),
        search
            .not_found_reason
            .as_ref()
            .map_or_else(|| "none".to_owned(), SmallestUsableNotFoundReason::label),
    )
}

/// 第七节 B1、B2、B3：独立算术（B1、B2）与 mkfs 的边（B3）；不成立就作废（V3）。交回核过的条数。
pub(super) fn anchors_of_the_independent_arithmetic_and_make_filesystem_edge() -> usize {
    let device_slots = |bytes: u64| bytes / SLOT_BYTES;
    let two_devices = |start_slot: u64, end_slot_exclusive: u64| {
        vec![
            UnitAreaOnOneDevice {
                start_slot,
                end_slot_exclusive
            };
            2
        ]
    };
    let expected: [(&str, Vec<UnitAreaOnOneDevice>, u32, u64); 5] = [
        (
            "4GiB_default_ring",
            two_devices(
                unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES),
                device_slots(PRODUCT_DEVICE_BYTES),
            ),
            2,
            529,
        ),
        (
            "unit_area_240_ring_6MiB",
            two_devices(1408, 1408 + 240),
            1,
            5,
        ),
        (
            "unit_area_384_ring_6MiB",
            two_devices(1408, 1408 + 384),
            1,
            5,
        ),
        (
            "1GiB_ring_1MiB",
            two_devices(1088, device_slots(1 << 30)),
            1,
            161,
        ),
        (
            "256GiB_default_ring",
            two_devices(
                unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES),
                device_slots(256 << 30),
            ),
            3,
            41451,
        ),
    ];
    let mut checked = 0;
    for (name, unit_areas, expected_root_level, expected_positions) in expected {
        let (root_level, positions) = allocation_record_tree_positions_by_the_clause(&unit_areas);
        let holds = root_level == expected_root_level && positions == expected_positions;
        emit_product_line!("E name=anchor_b anchor=B1_B2 geometry={name} root_level={root_level} allocation_record_tree_positions={positions} expected_root_level={expected_root_level} expected_positions={expected_positions} holds={holds}");
        if !holds {
            stop_with("v3_anchor_b1_b2_does_not_hold", &format!("geometry={name}"));
        }
        checked += 1;
    }
    let smallest = make_filesystem_smallest_device_bytes(SMALL_JOURNAL_RING_BYTES);
    let default_smallest = make_filesystem_smallest_device_bytes(JOURNAL_RING_DEFAULT_BYTES);
    let make_filesystem_on = |device_bytes: u64, journal_ring_bytes: u64| {
        let geometry = Geometry {
            point: DeviceSizePoint::MakeFilesystemSmallest,
            device_bytes,
            journal_ring_bytes,
            slots_per_region: ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        };
        make_filesystem(
            &make_filesystem_parameters(&geometry),
            &mut blank_devices(device_bytes),
        )
    };
    let edges = [
        (
            "ring_1MiB_smallest",
            smallest,
            make_filesystem_on(smallest, SMALL_JOURNAL_RING_BYTES).is_ok(),
        ),
        (
            "ring_1MiB_smallest_minus_512",
            smallest - 512,
            matches!(
                make_filesystem_on(smallest - 512, SMALL_JOURNAL_RING_BYTES),
                Err(MakeFilesystemError::UnitAreaBeyondDevice { .. })
            ),
        ),
        (
            "default_ring_smallest",
            default_smallest,
            make_filesystem_on(default_smallest, JOURNAL_RING_DEFAULT_BYTES).is_ok(),
        ),
        (
            "default_ring_smallest_minus_512",
            default_smallest - 512,
            matches!(
                make_filesystem_on(default_smallest - 512, JOURNAL_RING_DEFAULT_BYTES),
                Err(MakeFilesystemError::JournalRingTooLargeForDevice { .. })
            ),
        ),
    ];
    for (name, device_bytes, holds) in edges {
        emit_product_line!(
            "E name=anchor_b anchor=B3 case={name} device_bytes={device_bytes} holds={holds}"
        );
        if !holds {
            stop_with("v3_anchor_b3_does_not_hold", &format!("case={name}"));
        }
        checked += 1;
    }
    checked
}

#[cfg(test)]
mod tests;
