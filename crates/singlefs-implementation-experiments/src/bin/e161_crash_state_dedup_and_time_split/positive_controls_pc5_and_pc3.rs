//! 阳性对照 PC5、PC3：设备内存与计时对照。

use super::*;

// ───────────────────────────── PC5、PC3 ─────────────────────────────

/// P1 的一类：表里被恢复或 checker 正常那一遍读到过的内容的字节数之和。
fn content_bytes(table: &ContentTable) -> u128 {
    table
        .entries
        .values()
        .filter(|entry| entry.first_ordinal_any_pass.is_some())
        .map(|entry| u128::try_from(entry.bytes.len()).expect("长度"))
        .sum()
}

/// 合成的内容：32768 字节全 0xA5（PC5）。
const SYNTHETIC_CONTENT_BYTE: u8 = 0xA5;
/// 合成的读集：7 次调用（PC5）。
const SYNTHETIC_READ_SET_CALLS: u64 = 7;

/// PC5：往 P1 加一份从没读到过的 32768 字节、往 P3 加一条 7 次调用的合成读集、多记一个合成状态，四部分各自恰好涨多少。
pub(super) fn report_device_memory_control(
    output: &mut ResultLines,
    label: &str,
    tables: &mut ContentTables,
    summary: &CellSummary,
    write_table_length: usize,
) -> bool {
    let whole_unit = ReadCategory::WholeUnit.slot();
    let contents_before: u128 = tables.by_category.iter().map(content_bytes).sum();
    let synthetic =
        vec![SYNTHETIC_CONTENT_BYTE; usize::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768")];
    tables.by_category[whole_unit].insert(digest_of_bytes(&synthetic), &synthetic, u64::MAX, false);
    let contents_after: u128 = tables.by_category.iter().map(content_bytes).sum();
    let arm = Arm::FullReadSet;
    let before = device_memory_parts(summary, write_table_length, arm);
    let mut with_read_set = summary.clone();
    with_read_set.arm_distinct_key_elements[Arm::RecoveryConsult.slot()] +=
        SYNTHETIC_READ_SET_CALLS;
    with_read_set.arm_distinct_keys[Arm::RecoveryConsult.slot()] += 1;
    let after_read_set = device_memory_parts(&with_read_set, write_table_length, arm);
    let mut with_state = summary.clone();
    with_state.states += 1;
    let after_state = device_memory_parts(&with_state, write_table_length, arm);
    let contents_grew = contents_after - contents_before;
    let read_sets_grew = after_read_set.read_sets - before.read_sets;
    let masks_grew = after_state.masks - before.masks;
    let keys_grew = after_state.keys_per_state - before.keys_per_state;
    let holds = contents_grew == u128::from(LOCAL_DATA_UNIT_BYTES)
        && read_sets_grew == u128::from(SYNTHETIC_READ_SET_CALLS * READ_SET_BYTES_PER_CALL)
        && masks_grew == mask_bytes_per_state(write_table_length)
        && keys_grew == u128::from(KEYS_STORED_PER_STATE * KEY_BYTES);
    output.line(format!(
        "name=positive_control_device_memory cell={label} p1_grew={contents_grew} p3_grew={read_sets_grew} p2_grew={masks_grew} p4_per_state_grew={keys_grew} pc5_holds={}",
        bool_text(holds)
    ));
    holds
}

/// PC3 那一段的样子：（写数，全量计划里的状态数，序号最小的 1000 个，这 1000 个是不是都落在那一段）。
pub(super) struct TimingControlSegmentShape {
    pub(super) writes: usize,
    pub(super) states: u64,
    pub(super) ordinals: Vec<u64>,
    every_ordinal_in_the_segment: bool,
}

impl TimingControlSegmentShape {
    pub(super) fn of(stream: &PreparedStream, full: &LocalPlan<'_>) -> Self {
        let segment = TIMING_CONTROL_SEGMENT;
        let first = full.first_ordinal_by_segment[segment];
        let ordinals: Vec<u64> =
            (first..first + u64::try_from(TIMING_CONTROL_STATES).expect("1000")).collect();
        let every_ordinal_in_the_segment = ordinals
            .iter()
            .all(|ordinal| full.segment_of(*ordinal) == segment);
        Self {
            writes: stream.segments[segment].len(),
            states: full.count_by_segment[segment],
            ordinals,
            every_ordinal_in_the_segment,
        }
    }

    /// 重跑登记 I-6：那一段写数是 18、状态数不少于 1000、取的 1000 个序号都落在那一段。
    pub(super) fn holds(&self) -> bool {
        self.writes == TIMING_CONTROL_SEGMENT_WRITES
            && self.states >= u64::try_from(TIMING_CONTROL_STATES).expect("1000")
            && self.every_ordinal_in_the_segment
    }
}

/// PC3：覆盖写固定脚本流段 18 序号最小的 1000 个状态，先照常跑一遍，再六遍各在一个计时区里加 2 ms 忙等。交回全部六个区都过没过。
/// 那一段不是写行那 18 个单元写、或不够 1000 个状态时，六个区一律判不过、一个状态都不评（`pc3_segment_holds=false`）。
pub(super) fn report_timing_control(
    output: &mut ResultLines,
    stream: &PreparedStream,
    threads: usize,
) -> bool {
    let table = TornWriteTable::of(stream);
    let context = EvaluationContext::new(stream, &table);
    let full = LocalPlan::new(
        stream,
        &table,
        domain_expansions(stream, LocalExpansion::EveryCombination),
    );
    let shape = TimingControlSegmentShape::of(stream, &full);
    if !shape.holds() {
        for busy_zone in TIMING_ZONES {
            output.line(format!(
                "name=positive_control_timing busy_zone={} states=0 segment={TIMING_CONTROL_SEGMENT} segment_writes={} segment_states={} pc3_segment_holds=false pc3_zone_holds=false",
                busy_zone.label(),
                shape.writes,
                shape.states,
            ));
        }
        return false;
    }
    let ordinals = shape.ordinals;
    let zone_totals = |busy_zone: Option<TimingZone>| -> [u128; 6] {
        let totals = evaluate_stream_domain(&StreamDomainRun {
            label: format!("pc3-{}", busy_zone.map_or("none", TimingZone::label)),
            context: &context,
            plan: &full,
            ordinals: &ordinals,
            control_ordinals: &BTreeSet::new(),
            timing: TimingControl {
                busy_zone,
                clock: ZoneClock::Wall,
            },
            threads,
            depth: EvaluationDepth::FourSteps,
        });
        let mut sums = TimingSums::default();
        for segment_sums in totals.timing_by_segment.values() {
            sums.absorb(segment_sums);
        }
        sums.zone_nanoseconds
    };
    let without = zone_totals(None);
    let mut every_zone_holds = true;
    for (busy_slot, busy_zone) in TIMING_ZONES.iter().enumerate() {
        let with = zone_totals(Some(*busy_zone));
        let changes: Vec<i128> = with
            .iter()
            .zip(&without)
            .map(|(after, before)| {
                i128::try_from(*after).expect("纳秒") - i128::try_from(*before).expect("纳秒")
            })
            .collect();
        let added_enough = changes[busy_slot]
            >= i128::try_from(TIMING_CONTROL_MINIMUM_ADDED_NANOSECONDS).expect("纳秒");
        let others_small = changes.iter().enumerate().all(|(slot, change)| {
            slot == busy_slot
                || change.unsigned_abs() < TIMING_CONTROL_MAXIMUM_OTHER_CHANGE_NANOSECONDS
        });
        let holds = added_enough && others_small;
        every_zone_holds &= holds;
        let change_fields: Vec<String> = TIMING_ZONES
            .iter()
            .zip(&changes)
            .map(|(zone, change)| format!("{}_delta_elapsed_ns={change}", zone.label()))
            .collect();
        output.line(format!(
            "name=positive_control_timing busy_zone={} states={} {} busy_zone_added_at_least_1800ms={} other_zones_changed_under_1000ms={} pc3_segment_holds=true pc3_zone_holds={}",
            busy_zone.label(),
            ordinals.len(),
            change_fields.join(" "),
            bool_text(added_enough),
            bool_text(others_small),
            bool_text(holds),
        ));
    }
    every_zone_holds
}

/// PC3 取的段（5.3：第 1 次覆盖写固定脚本流段 14「写行发布 18 个单元写」那一段；C577 之后同一批写是段 16，重跑登记 I-6、A9；
/// 取号写两轮之后是段 18，第 3 次重跑登记 J-4、A9）。
pub(super) const TIMING_CONTROL_SEGMENT: usize = 18;
/// PC3 那一段的写数：写行发布的 18 个单元写（重跑登记 A9）。
const TIMING_CONTROL_SEGMENT_WRITES: usize = 18;

#[cfg(test)]
mod tests;
