//! 攻方副本探针（m3-formal-code-b23-r1 opus）：只在副本里，不进仓。挂在 crash_injection 下面当子模块，用它的私有项。
//! K3-a 与 A1 两段照入库用例的写法原样重写一遍（写下标现算、断言同义），但前提不成立时报一行而不 panic。
use super::*;
use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_harness::history::{
    generate_history, ContentChoice, ContentLength, HistoryDeviceWidth, HistoryStartingPoint,
    PerStepChecker,
};
use std::fmt::Write as _;

fn execution_of(width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

#[must_use]
pub fn width_named(name: &str) -> HistoryDeviceWidth {
    match name {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
        "4g" => HistoryDeviceWidth::FourGibibytes,
        other => panic!("不认的宽度 {other}"),
    }
}

#[must_use]
pub fn length_named(name: &str) -> ContentLength {
    match name {
        "inside0" => ContentLength::InsideOneDataUnit { selector: 0 },
        "inside7" => ContentLength::InsideOneDataUnit { selector: 7 },
        "empty" => ContentLength::Empty,
        "unit" => ContentLength::ExactlyDataUnitBytes,
        "above" => ContentLength::OneByteAboveDataUnitPayloadCapacity,
        "span3" => ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units: 3, selector: 0 },
        other => panic!("不认的长度 {other}"),
    }
}

/// K3-a 入库用例同一套算法：历史 = 第一个文件之后 `overwrites` 次覆盖写（长度 `length`），几何 `width`。
/// 返回一行：前提成不成、摆出的两个态、两次只读恢复落的版本、两种接线下的计数与第三截的支。
#[must_use]
pub fn k3a(width_name: &str, length_name: &str, overwrites: usize) -> String {
    let execution = execution_of(width_named(width_name));
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (0..overwrites)
            .map(|k| HistoryOperation::PublishOverwrite(ContentChoice { length: length_named(length_name), fill_seed: 1 + k as u64 }))
            .collect(),
    };
    let tag = format!("k3a width={width_name} length={length_name} overwrites={overwrites}");
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| k3a_body(&history, execution))) {
        Ok(Ok(line)) => format!("{tag} {line}"),
        Ok(Err(why)) => format!("{tag} PRECONDITION-FAILED {why}"),
        Err(payload) => {
            let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string())).unwrap_or_default();
            format!("{tag} PANIC {}", text.chars().take(200).collect::<String>())
        }
    }
}

fn k3a_body(history: &GeneratedHistory, execution: HistoryExecution) -> Result<String, String> {
    // 同 tests::writes_segments_and_first_write_of_the_overwrite
    let stream = SharedStream::retaining_contents();
    let mut operations_after_the_starting_point = None;
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        if observation.position == StepPosition::StartingPoint {
            operations_after_the_starting_point = Some(stream.operation_count());
        }
    });
    if run.ending != HistoryEnding::Completed {
        return Err(format!("history-ending={:?}", run.ending).chars().take(120).collect());
    }
    let operations_after_the_starting_point = operations_after_the_starting_point.ok_or("no-starting-point")?;
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let first_write_of_the_overwrite = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= operations_after_the_starting_point)
        .ok_or("overwrite-wrote-nothing")?;
    // 以下与入库用例逐句同义
    let root_of_the_overwrite = (first_write_of_the_overwrite..writes.len())
        .find(|write| writes[*write].kind == StepKind::RootRecordFua)
        .ok_or("no-overwrite-root")?;
    let root_of_the_first_file = (0..first_write_of_the_overwrite)
        .rev()
        .find(|write| writes[*write].kind == StepKind::RootRecordFua)
        .ok_or("no-ff-root")?;
    let root_before_the_first_file = (0..root_of_the_first_file)
        .rev()
        .find(|write| writes[*write].kind == StepKind::RootRecordFua)
        .ok_or("no-root-before-ff")?;
    let records_of_the_first_file: Vec<usize> = (root_before_the_first_file + 1..root_of_the_first_file)
        .filter(|write| writes[*write].kind == StepKind::JournalRecord)
        .collect();
    let rotation_after_the_first_file: Vec<usize> = (root_of_the_first_file + 1..first_write_of_the_overwrite)
        .filter(|write| writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    if (records_of_the_first_file.len(), rotation_after_the_first_file.len()) != (2, 2) {
        return Err(format!("ff (records, rotation)=({}, {})", records_of_the_first_file.len(), rotation_after_the_first_file.len()));
    }
    let first_crash_segment_index = segment_containing_the_write(&segments, records_of_the_first_file[0]);
    let first_write_of_the_first_crash_segment = segments[first_crash_segment_index][0];
    let withheld_in_the_first_crash: Vec<usize> = records_of_the_first_file
        .iter()
        .chain(std::iter::once(&rotation_after_the_first_file[1]))
        .map(|write| write - first_write_of_the_first_crash_segment)
        .collect();
    let writes_of_the_first_crash = root_of_the_overwrite + 1 - first_write_of_the_first_crash_segment;
    let mut first_crash_image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], execution.device_width.device_bytes());
    first_crash_image.apply_writes(&writes[..first_write_of_the_first_crash_segment]);
    let persisted: Vec<RetainedWrite> = (0..writes_of_the_first_crash)
        .filter(|w| !withheld_in_the_first_crash.contains(w))
        .map(|w| writes[first_write_of_the_first_crash_segment + w].clone())
        .collect();
    first_crash_image.apply_writes(&persisted);
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &CrashPoint {
            segment_index: first_crash_segment_index,
            persisted_within_the_segment: (0..writes_of_the_first_crash).map(|w| !withheld_in_the_first_crash.contains(&w)).collect(),
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        },
    );
    if !matches!(mount_run.outcome, WritableMountOutcome::Published) {
        return Err(format!("first-crash-mount={:?}", mount_run.outcome).chars().take(160).collect());
    }
    let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &execution.device_width.fixed_geometry());
    let phase_of_each_write = writable_mount_phase_of_each_write(&mount_writes, mount_run.publishes_before_the_deletion_queue_continuation, mount_run.publishes_before_the_instance_table_sweep);
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|w| phase_of_each_write[*w] == WritableMountPhase::RowPublish && mount_writes[*w].kind == StepKind::RootRecordFua)
        .ok_or("no-row-root")?;
    let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
        .filter(|w| phase_of_each_write[*w] == WritableMountPhase::RowPublish && mount_writes[*w].kind == StepKind::JournalRecord)
        .collect();
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|w| mount_writes[*w].kind == StepKind::SystemConfigurationSlot)
        .collect();
    if (records_of_the_row_publish.len(), rotation_after_the_root.len()) != (2, 2) {
        return Err(format!("row (records, rotation)=({}, {})", records_of_the_row_publish.len(), rotation_after_the_root.len()));
    }
    let second_crash_segment_index = segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
    let first_write_of_the_second_crash_segment = mount_segments[second_crash_segment_index][0];
    let withheld_in_the_second_crash: Vec<usize> = records_of_the_row_publish
        .iter()
        .chain(std::iter::once(&rotation_after_the_root[0]))
        .map(|w| w - first_write_of_the_second_crash_segment)
        .collect();
    let mut second_crash_image = first_crash_image.clone();
    second_crash_image.apply_writes(&mount_writes[..first_write_of_the_second_crash_segment]);
    let persisted_second: Vec<RetainedWrite> = (first_write_of_the_second_crash_segment..=rotation_after_the_root[0])
        .filter(|w| !withheld_in_the_second_crash.contains(&(w - first_write_of_the_second_crash_segment)))
        .map(|w| mount_writes[w].clone())
        .collect();
    second_crash_image.apply_writes(&persisted_second);
    let vo = recover(&first_crash_image, JournalPolicy::Consult).effective_root;
    let vn = recover(&second_crash_image, JournalPolicy::Consult).effective_root;
    let precondition_vo_ne_vn = vo != vn;
    let writes_from_the_second = rotation_after_the_root[0] + 1 - first_write_of_the_second_crash_segment;
    let mut line = format!(
        "writes={} segments={} ff_records={records_of_the_first_file:?} ff_rotation={rotation_after_the_first_file:?} ow_first={first_write_of_the_overwrite} ow_root={root_of_the_overwrite} first(seg={first_crash_segment_index},start={first_write_of_the_first_crash_segment},count={writes_of_the_first_crash},withheld={withheld_in_the_first_crash:?}) mount_writes={} second(seg={second_crash_segment_index},start={first_write_of_the_second_crash_segment},count={writes_from_the_second},withheld={withheld_in_the_second_crash:?}) Vo={vo:?} Vn={vn:?} vo_ne_vn={precondition_vo_ne_vn}",
        writes.len(), segments.len(), mount_writes.len()
    );
    for wiring in ["second", "first"] {
        std::env::set_var("OPUS_K3_WIRING", wiring);
        let injection = inject_crashes_into_history_drawing_second_crashes(
            history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: first_crash_segment_index,
                writes_from_the_segment_on: writes_of_the_first_crash,
                withheld_among_them: withheld_in_the_first_crash.clone().leak(),
            },
            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                segment_index: second_crash_segment_index,
                writes_from_the_segment_on: writes_from_the_second,
                withheld_among_them: withheld_in_the_second_crash.clone().leak(),
            },
            None,
        );
        std::env::remove_var("OPUS_K3_WIRING");
        let t = &injection.tally;
        let branches: Vec<String> = injection
            .new_findings
            .iter()
            .filter(|f| matches!(f.stage, CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)))
            .map(|f| f.observation.model_disagreement.as_ref().map_or_else(|| "no-disagreement".to_string(), |d| {
                ["WitnessedCounterAboveTheLastRecordOfTheSelectedVersion", "RecordAtTheWitnessedCounterNewerThanTheSelectedVersion", "NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable", "CriterionFalseWhenRecomputedOnTheCrashImage"]
                    .iter().filter(|n| d.implementation_answer.contains(*n)).copied().collect::<Vec<_>>().join("+")
            }))
            .collect();
        let test_passes = (t.crash_points, t.writable_mounts_after_the_crash_succeeded, t.crash_points_after_a_step_reading_under_injected_faults) == (1, 1, 0)
            && (t.second_crash_points, t.second_crash_points_by_phase.get(&WritableMountPhase::RowPublish).copied()) == (1, Some(1))
            && (t.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration, t.remount_refusals_with_the_witness_criterion_true_without_read_faults) == (1, 1)
            && precondition_vo_ne_vn;
        let _ = write!(
            line,
            " | wiring={wiring} pre=({}, {}, {}) second=({}, {:?}) (judged, illegal)=({}, {}) third_stage_branches={branches:?} inlined_test={}",
            t.crash_points, t.writable_mounts_after_the_crash_succeeded, t.crash_points_after_a_step_reading_under_injected_faults,
            t.second_crash_points, t.second_crash_points_by_phase.get(&WritableMountPhase::RowPublish),
            t.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            t.remount_refusals_with_the_witness_criterion_true_without_read_faults,
            if test_passes { "green" } else { "red" }
        );
    }
    Ok(line)
}

// ---------------- F3 ----------------

/// 抽函数之前那一句原样（diff 删掉的那四行）。
fn seed_before_the_extraction(seed: HistorySeed, crash_point: &CrashPoint) -> u64 {
    seed.0
        ^ SECOND_CRASH_POINT_SEED_SALT
        ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
        ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment)
}

struct XorShift(u64);
impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn crash_point_of(segment_index: usize, persisted: Vec<bool>) -> CrashPoint {
    CrashPoint { segment_index, persisted_within_the_segment: persisted, step: StepPosition::Operation(0), operation_kind: Some(HistoryOperationKind::PublishOverwrite) }
}

/// 抽函数前后逐位相同：随机种子、段号、段长 0..=200、随机子集。
#[must_use]
pub fn f3_bitwise(samples: usize) -> String {
    let mut rng = XorShift(0x0123_4567_89ab_cdef);
    let mut differ = 0usize;
    for k in 0..samples {
        let seed = if k % 4 == 0 { k as u64 } else { rng.next() };
        let segment_index = (rng.next() % 100_000) as usize;
        let length = (rng.next() % 201) as usize;
        let persisted: Vec<bool> = (0..length).map(|_| rng.next() & 1 == 1).collect();
        let cp = crash_point_of(segment_index, persisted);
        if seed_before_the_extraction(HistorySeed(seed), &cp) != seed_of_the_second_crash_points(HistorySeed(seed), &cp).0 {
            differ += 1;
        }
    }
    format!("f3-bitwise samples={samples} differ={differ}")
}

fn fnv(persisted: &[bool]) -> u64 {
    fingerprint_of_the_persisted_subset(persisted)
}
fn fnv_over<'a>(bits: impl Iterator<Item = &'a bool>) -> u64 {
    bits.fold(PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS, |h, b| (h ^ u64::from(*b)).wrapping_mul(PERSISTED_SUBSET_FINGERPRINT_PRIME))
}

/// 一种种子算法：(历史种子, 段号, 段内子集) ↦ 种子。
type SeedFn = fn(u64, u64, &[bool]) -> u64;

const S: u64 = SECOND_CRASH_POINT_SEED_SALT;

pub const VARIANTS: &[(&str, SeedFn)] = &[
    ("entered(fnv1a)", |seed, seg, p| seed_of_the_second_crash_points(HistorySeed(seed), &crash_point_of(seg as usize, p.to_vec())).0),
    ("take(1)", |seed, seg, p| seed ^ S ^ seg ^ fnv_over(p.iter().take(1))),
    ("take(11)", |seed, seg, p| seed ^ S ^ seg ^ fnv_over(p.iter().take(11))),
    ("take(12)", |seed, seg, p| seed ^ S ^ seg ^ fnv_over(p.iter().take(12))),
    ("take(16)", |seed, seg, p| seed ^ S ^ seg ^ fnv_over(p.iter().take(16))),
    ("skip(1)", |seed, seg, p| seed ^ S ^ seg ^ fnv_over(p.iter().skip(1))),
    ("rev", |seed, seg, p| seed ^ S ^ seg ^ fnv_over(p.iter().rev())),
    ("bitmask-shl", |seed, seg, p| seed ^ S ^ seg ^ p.iter().fold(0u64, |h, b| (h << 1) | u64::from(*b))),
    ("fnv-as-u32", |seed, seg, p| seed ^ S ^ seg ^ (fnv(p) & 0xffff_ffff)),
    ("fnv-as-u16", |seed, seg, p| seed ^ S ^ seg ^ (fnv(p) & 0xffff)),
    ("popcount", |seed, seg, p| seed ^ S ^ seg ^ p.iter().filter(|b| **b).count() as u64),
    ("first-withheld", |seed, seg, p| seed ^ S ^ seg ^ p.iter().position(|b| !*b).unwrap_or(usize::MAX) as u64),
    ("drop-history-seed", |_seed, seg, p| S ^ seg ^ fnv(p)),
    ("drop-salt", |seed, seg, p| seed ^ seg ^ fnv(p)),
    ("drop-segment", |seed, _seg, p| seed ^ S ^ fnv(p)),
    ("add-not-xor", |seed, seg, p| seed.wrapping_add(S).wrapping_add(seg).wrapping_add(fnv(p))),
    ("or-not-xor", |seed, seg, p| seed | S | seg | fnv(p)),
];

fn bits_of_mask(mask: u64, length: usize) -> Vec<bool> {
    (0..length).map(|bit| (mask >> bit) & 1 == 1).collect()
}

/// 内联用例原样：种子 0、段号 16、长 12 的 4095 个子集算出的种子有几个不同。
fn inline_case(f: SeedFn) -> usize {
    let mut seeds: Vec<u64> = (0..(1u64 << 12) - 1).map(|m| f(0, 16, &bits_of_mask(m, 12))).collect();
    seeds.sort_unstable();
    seeds.dedup();
    seeds.len()
}

fn distinct(mut v: Vec<u64>) -> usize {
    v.sort_unstable();
    v.dedup();
    v.len()
}

/// 一段真历史的候选段（与 draw_crash_points 同一条界：整段落在 mkfs 之后、最后一个跑完的操作之前）的段长。
fn candidate_segment_lengths(history: &GeneratedHistory, execution: HistoryExecution) -> Vec<(usize, usize)> {
    let stream = SharedStream::retaining_contents();
    let mut last = 0usize;
    let run = execute_history_with(history, execution, &stream, &mut |_| {
        last = stream.operation_count();
    });
    let (_writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &execution.device_width.fixed_geometry());
    (0..segments.len())
        .filter(|s| segments[*s].iter().all(|w| (run.operations_written_by_make_filesystem..last).contains(&stream_indexes[*w])))
        .map(|s| (s, segments[s].len()))
        .collect()
}

/// 段长分布：`histories` 段 `operations` 步的生成历史。
#[must_use]
pub fn f3_segments(width_name: &str, histories: u64, operations: usize) -> String {
    let execution = execution_of(width_named(width_name));
    let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
    let mut total = 0usize;
    for seed in 0..histories {
        for (_, length) in candidate_segment_lengths(&generate_history(HistorySeed(seed), operations), execution) {
            *histogram.entry(length).or_insert(0) += 1;
            total += 1;
        }
    }
    let over12: usize = histogram.iter().filter(|(l, _)| **l > 12).map(|(_, c)| *c).sum();
    let over16: usize = histogram.iter().filter(|(l, _)| **l > 16).map(|(_, c)| *c).sum();
    let max = histogram.keys().last().copied().unwrap_or(0);
    format!("f3-segments width={width_name} histories={histories} operations={operations} candidate_segments={total} longer_than_12={over12} longer_than_16={over16} longest={max} histogram={histogram:?}")
}

/// 每种种子算法：内联用例（4095 个两两不同才绿），段内长段，跨段（真历史的全部第一次崩溃态），跨历史（历史种子不同、段号子集相同）。
#[must_use]
pub fn f3_variants(width_name: &str, histories: u64, operations: usize) -> String {
    let execution = execution_of(width_named(width_name));
    // 真历史的第一次崩溃态：候选段 ≤ 12 写的全部真子集，更长的段各抽 64 个真子集（与 EveryProperSubsetOfShortSegments 同形）。
    let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
    let mut states_per_history: Vec<(u64, Vec<(u64, Vec<bool>)>)> = Vec::new();
    for seed in 0..histories {
        let mut states = Vec::new();
        for (segment, length) in candidate_segment_lengths(&generate_history(HistorySeed(seed), operations), execution) {
            if length <= 12 {
                for m in 0..(1u64 << length) - 1 {
                    states.push((segment as u64, bits_of_mask(m, length)));
                }
            } else {
                let mut seen = BTreeSet::new();
                while seen.len() < 64 {
                    let p: Vec<bool> = (0..length).map(|_| rng.next() & 1 == 1).collect();
                    if p.iter().all(|b| *b) { continue; }
                    seen.insert(p);
                }
                states.extend(seen.into_iter().map(|p| (segment as u64, p)));
            }
        }
        states_per_history.push((seed, states));
    }
    let total_states: usize = states_per_history.iter().map(|(_, s)| s.len()).sum();
    let mut out = String::new();
    let _ = writeln!(out, "f3-variants width={width_name} histories={histories} operations={operations} first_crash_states={total_states}");
    for (name, f) in VARIANTS {
        let inline = inline_case(*f);
        // 段内：长 L 的段，L ≤ 16 全部真子集，更长抽 20000 个不同的真子集
        let mut within = Vec::new();
        for length in [12usize, 13, 16, 24, 40, 64, 65, 100] {
            let subsets: Vec<Vec<bool>> = if length <= 16 {
                (0..(1u64 << length) - 1).map(|m| bits_of_mask(m, length)).collect()
            } else {
                let mut seen = BTreeSet::new();
                while seen.len() < 20_000 {
                    let p: Vec<bool> = (0..length).map(|_| rng.next() & 1 == 1).collect();
                    seen.insert(p);
                }
                seen.into_iter().collect()
            };
            let n = subsets.len();
            let d = distinct(subsets.iter().map(|p| f(0, 16, p)).collect());
            within.push(format!("L{length}:{d}/{n}"));
        }
        // 跨段：每段历史自己的全部第一次崩溃态里种子撞了几对（段号不同、子集不同都算）
        let mut per_history_lost = 0usize;
        for (seed, states) in &states_per_history {
            let n = states.len();
            let d = distinct(states.iter().map(|(seg, p)| f(*seed, *seg, p)).collect());
            per_history_lost += n - d;
        }
        // 跨历史：全部历史的全部态合起来
        let all: Vec<u64> = states_per_history.iter().flat_map(|(seed, states)| states.iter().map(move |(seg, p)| f(*seed, *seg, p))).collect();
        let all_distinct = distinct(all);
        let _ = writeln!(
            out,
            "variant={name} inline_4095_test={} distinct_in_inline={inline} within_segment[{}] states_sharing_a_seed_within_a_history={per_history_lost} distinct_over_all_histories={all_distinct}/{total_states}",
            if inline == 4095 { "green" } else { "red" },
            within.join(" ")
        );
    }
    out
}

// ---------------- A1 ----------------

/// A1 入库用例同一套算法与断言；坏发布由 `OPUS_K2A_VARIANT`（只第二截那一支）、`OPUS_ARM`（调用点参数）、`OPUS_CORE_MUT`（每个调用）选。
#[must_use]
pub fn a1() -> String {
    let tag = format!(
        "a1 arm_variant={} arm_param={} core={}",
        std::env::var("OPUS_K2A_VARIANT").unwrap_or_else(|_| "-".into()),
        std::env::var("OPUS_ARM").unwrap_or_else(|_| "-".into()),
        std::env::var("OPUS_CORE_MUT").unwrap_or_else(|_| "-".into())
    );
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(a1_body)) {
        Ok(line) => format!("{tag} {line}"),
        Err(payload) => {
            let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string())).unwrap_or_default();
            format!("{tag} PANIC {}", text.chars().take(240).collect::<String>())
        }
    }
}

fn a1_body() -> String {
    let execution = execution_of(HistoryDeviceWidth::UnitAreaOf384Slots);
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode] };
    let stream = SharedStream::retaining_contents();
    let mut operations_after_the_unlink = None;
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        if observation.position == StepPosition::Operation(0) {
            operations_after_the_unlink = Some(stream.operation_count());
        }
    });
    if run.ending != HistoryEnding::Completed {
        return format!("RED(history-not-completed {:?})", run.ending).chars().take(300).collect();
    }
    let operations_after_the_unlink = operations_after_the_unlink.expect("登记过");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &execution.device_width.fixed_geometry());
    let root = (0..writes.len()).rev().find(|w| stream_indexes[*w] >= operations_after_the_unlink && writes[*w].kind == StepKind::RootRecordFua).expect("根");
    let segment_of_the_root = segment_containing_the_write(&segments, root);
    let first = segments[segment_of_the_root][0];
    let injection = inject_crashes_into_history(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState { segment_index: segment_of_the_root, writes_from_the_segment_on: root + 2 - first, withheld_among_them: vec![root + 1 - first].leak() },
        None,
    );
    let t = &injection.tally;
    let findings_after: Vec<String> = injection.new_findings.iter().filter(|f| f.stage == CrashStateStage::AfterTheWritableMountAndOnePublish).map(|f| f.render().chars().take(200).collect()).collect();
    let other_findings: Vec<String> = injection.new_findings.iter().filter(|f| f.stage != CrashStateStage::AfterTheWritableMountAndOnePublish).map(|f| format!("{:?}", f.stage)).collect();
    let pre_ok = t.crash_points_that_land_before_the_last_committed_version == 0 && t.writable_mounts_after_the_crash_succeeded == 1;
    let publish_ok = (t.publishes_after_the_writable_mount, t.publishes_after_the_writable_mount_refused) == (1, 0) && findings_after.is_empty();
    let mut crash_image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], execution.device_width.device_bytes());
    crash_image.apply_writes(&writes[..=root]);
    let mount_run = mount_and_publish_once_on_the_crash_image(&crash_image, &execution.device_width.parameters(), execution, history.seed, &injection.crash_points[0]);
    let published = matches!(mount_run.outcome, WritableMountOutcome::Published);
    let crashed = singlefs_core::mounted_read::mount_read_only(&crash_image).expect("崩溃那一版只读挂载");
    let after = singlefs_core::mounted_read::mount_read_only(&mount_run.pool_after).expect("发布之后只读挂载");
    let crashed_numbers: Vec<u64> = crashed.mounted.inode_records().iter().map(|r| r.inode).collect();
    let txg = after.effective_root.checkpoint_txg;
    let mut expected = crashed.mounted.inode_records().to_vec();
    expected.push(InodeRecord { inode: FIRST_INODE_NUMBER + 2, object_birth: txg, nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES, size: 0, change_count: txg.0, write_time_seconds: FIXED_WRITE_TIME_SECONDS });
    let full_compare = after.mounted.inode_records() == expected.as_slice();
    let green = pre_ok && publish_ok && published && crashed_numbers == [FIRST_INODE_NUMBER + 1] && full_compare;
    let violations: Vec<&'static str> = checker_violations_of(&mount_run.pool_after).into_iter().map(|(i, _)| i).collect();
    let short = |r: &InodeRecord| (r.inode, r.object_birth.0, r.nlink, r.size, r.change_count, r.write_time_seconds);
    format!(
        "{} pre_ok={pre_ok} publish_ok={publish_ok} published={published} crashed_numbers={crashed_numbers:?} crashed_root_txg={} after_root_txg={} full_compare={full_compare} after={:?} checker_on_pool_after={violations:?} findings_after={findings_after:?} other_stage_findings={other_findings:?}",
        if green { "A1=GREEN" } else { "A1=RED" },
        crashed.effective_root.checkpoint_txg.0,
        txg.0,
        after.mounted.inode_records().iter().map(short).collect::<Vec<_>>(),
    )
}

/// 改法 F3-d 的三条候选断言（只在这个模型上量）：
/// A 历史种子 0、段号 0..=63、每段长 12 的 4095 个子集，262080 个种子两两不同；
/// B 历史种子 0、段号 16、长 28、前 16 个写都落了而后 12 个写取 4095 种（全落那一种除外），4095 个种子两两不同；
/// C 段号 16、长 12 的 4095 个子集在历史种子 0 与 1 上各算一遍，8190 个种子两两不同。
#[must_use]
pub fn f3_fix() -> String {
    let mut out = String::new();
    for (name, f) in VARIANTS {
        let a = distinct((0u64..64).flat_map(|seg| (0..(1u64 << 12) - 1).map(move |m| f(0, seg, &bits_of_mask(m, 12)))).collect());
        let b = distinct((0..(1u64 << 12) - 1).map(|m| {
            let mut p = vec![true; 16];
            p.extend(bits_of_mask(m, 12));
            f(0, 16, &p)
        }).collect());
        let c = distinct([0u64, 1].iter().flat_map(|seed| (0..(1u64 << 12) - 1).map(move |m| f(*seed, 16, &bits_of_mask(m, 12)))).collect());
        let _ = writeln!(
            out,
            "f3-fix variant={name} inline={} A={a}/262080:{} B={b}/4095:{} C={c}/8190:{}",
            if inline_case(*f) == 4095 { "green" } else { "red" },
            if a == 262_080 { "green" } else { "red" },
            if b == 4095 { "green" } else { "red" },
            if c == 8190 { "green" } else { "red" }
        );
    }
    out
}
