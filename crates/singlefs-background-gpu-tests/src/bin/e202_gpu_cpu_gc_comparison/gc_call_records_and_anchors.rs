//! 一次跑里每次 GC 调用记下的，与甲类、乙类锚点。

use super::*;

// ---- 一次跑里每次 GC 调用记下的，与第七节甲类、乙类锚点 ----

/// 一次 GC 调用记下的（窗口外读，登记第五节「计时与计数」第 2 条）。
pub(super) struct GcCallRecord {
    /// 这次调用是第几次用户改动之后的（交替跑）或 0（并发跑不分组）。
    pub(super) after_the_user_change: u64,
    pub(super) window: Option<CallWindow>,
    pub(super) inner_rejudge_cpu: Duration,
    pub(super) rejudge_tally: RejudgeTally,
    pub(super) started: bool,
    pub(super) compaction_publishes: u64,
    pub(super) batches: u64,
    pub(super) units_relocated_of_each_batch: Vec<u64>,
    pub(super) stop_name: String,
    pub(super) compute_records: Vec<BackgroundComputeOfOneBatch>,
    pub(super) has_the_alarm: bool,
    /// 每块盘的全空聚簇段数，这次调用开始前与返回后各一个读数（按盘的次序；登记第十二节修订第 5 条，窗口外读）。
    pub(super) empty_segments_before_the_call: Vec<u64>,
    pub(super) empty_segments_after_the_call: Vec<u64>,
}

/// 每块盘此刻的全空聚簇段数，按盘的次序（第八节轨迹，登记第十二节修订第 5 条）。
pub(super) fn empty_segments_of_every_device(session: &MountedSession) -> Vec<u64> {
    session
        .allocator
        .devices
        .iter()
        .map(|device| device.empty_segments())
        .collect()
}

/// 从后台入口交回的那一份读出记账（窗口外）。每块盘全空段数的前后读数由调用方另填。
pub(super) fn record_of_the_call(
    call: &BackgroundCompactionCall,
    after_the_user_change: u64,
    measured_window: Option<CallWindow>,
    inner_rejudge_cpu: Duration,
    rejudge_tally: RejudgeTally,
) -> GcCallRecord {
    let has_the_alarm = call
        .alarm_of_the_compaction_stopped_after_a_compute_error()
        .is_some();
    match call {
        BackgroundCompactionCall::NotStarted { reason, .. } => GcCallRecord {
            after_the_user_change,
            window: measured_window,
            inner_rejudge_cpu,
            rejudge_tally,
            started: false,
            compaction_publishes: 0,
            batches: 0,
            units_relocated_of_each_batch: Vec::new(),
            stop_name: format!("not_started_{reason:?}"),
            compute_records: Vec::new(),
            has_the_alarm,
            empty_segments_before_the_call: Vec::new(),
            empty_segments_after_the_call: Vec::new(),
        },
        BackgroundCompactionCall::Ran { run, .. } => GcCallRecord {
            after_the_user_change,
            window: measured_window,
            inner_rejudge_cpu,
            rejudge_tally,
            started: true,
            compaction_publishes: u64::try_from(run.versions_published.len()).expect("版数"),
            batches: run.batches_published,
            units_relocated_of_each_batch: run
                .versions_published
                .iter()
                .filter(|version| !version.units_relocated_by_this_publish.is_empty())
                .map(|version| {
                    u64::try_from(version.units_relocated_by_this_publish.len()).expect("单元数")
                })
                .collect(),
            stop_name: match &run.outcome {
                CompactionRunOutcome::Stopped(stop) => stop.name().to_string(),
                CompactionRunOutcome::Failed(failure) => format!("failed_{failure:?}"),
            },
            compute_records: run.background_compute_of_each_batch.clone(),
            has_the_alarm,
            empty_segments_before_the_call: Vec::new(),
            empty_segments_after_the_call: Vec::new(),
        },
    }
}

/// 这次调用起跑而一次整理发布都没发，或没起跑：GC 线程停下、通知前台发下一次（登记第五节「交替跑」）。
pub(super) fn the_call_published_nothing(record: &GcCallRecord) -> bool {
    record.compaction_publishes == 0
}

/// 第七节甲类第 1 行（登记第十二节修订第 2 条）：每一项记账（签发、搬一批、删意图、停在判据上的那一次都是一项）的 `path_taken`
/// 照这条臂的设置；条数不少于这次跑的批数 N。条数与 N 两个数逐次报（`name=run` 的 `compute_records=` 与 `batches=`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PathAnchor {
    pub(super) compute_records: u64,
    pub(super) compute_records_on_the_path_of_the_settings: u64,
    pub(super) batches: u64,
}

impl PathAnchor {
    /// 条数少于 N 照第十条「条款与实现对不上」判（停机条款第 4 条）。
    pub(super) fn holds(self) -> bool {
        self.compute_records == self.compute_records_on_the_path_of_the_settings
            && self.compute_records >= self.batches
    }
}

pub(super) fn path_anchor(records: &[GcCallRecord], compute: ComputeOfTheArm) -> PathAnchor {
    let compute_records: Vec<&BackgroundComputeOfOneBatch> = records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .collect();
    let on_the_path = compute_records
        .iter()
        .filter(
            |compute_record| match (&compute_record.path_taken, compute.uses_the_gpu()) {
                (
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
                    ),
                    false,
                )
                | (BackgroundComputePathTaken::Gpu(_), true) => true,
                (BackgroundComputePathTaken::Cpu(_), _)
                | (BackgroundComputePathTaken::Gpu(_), false) => false,
            },
        )
        .count();
    PathAnchor {
        compute_records: u64::try_from(compute_records.len()).expect("条数"),
        compute_records_on_the_path_of_the_settings: u64::try_from(on_the_path).expect("条数"),
        batches: records.iter().map(|record| record.batches).sum(),
    }
}

/// 第七节甲类第 2、3 行与第十条第 2、3 行：分到活的卡、切没切片、两份核比对的条数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GpuWorkAnchors {
    pub(super) gpu_records: u64,
    pub(super) segments_given_to_a_card_other_than_the_lowest: u64,
    pub(super) segments_without_a_submission: u64,
    pub(super) recuts_after_running_out_of_memory: u64,
    pub(super) segments_with_pieces_other_than_submissions: u64,
    pub(super) uncertain_entries: u64,
    pub(super) entries_judged_by_the_cpu_reference: u64,
    pub(super) batches_abandoned_after_a_compute_error: u64,
}

impl GpuWorkAnchors {
    pub(super) fn hold(self) -> bool {
        self.segments_given_to_a_card_other_than_the_lowest == 0
            && self.recuts_after_running_out_of_memory == 0
            && self.segments_with_pieces_other_than_submissions == 0
            && self.uncertain_entries == 0
            && self.entries_judged_by_the_cpu_reference == 0
    }
}

/// 每批三段：交了提交的段分到活的卡恰好是 `lowest_card` 那一张，没交提交的段一张卡都不列；重切 0；片数等于提交数；不确定与交 CPU 判都是 0
/// （第九节变异「甲类锚点『分到活的卡』改成不核」改的是这里）。
pub(super) fn gpu_work_anchors(
    records: &[GcCallRecord],
    lowest_card: &GpuDeviceIdentifier,
) -> GpuWorkAnchors {
    let mut anchors = GpuWorkAnchors::default();
    for compute_record in records
        .iter()
        .flat_map(|record| record.compute_records.iter())
    {
        if compute_record
            .compute_error_that_abandoned_the_batch
            .is_some()
        {
            anchors.batches_abandoned_after_a_compute_error += 1;
        }
        let BackgroundComputePathTaken::Gpu(work) = &compute_record.path_taken else {
            continue;
        };
        anchors.gpu_records += 1;
        let entries = work.entries_of_the_three_segments();
        anchors.uncertain_entries += entries.uncertain_this_round;
        anchors.entries_judged_by_the_cpu_reference += entries.judged_by_the_cpu_reference;
        let submissions = &work.submissions_on_the_gpus;
        for segment in [
            &submissions.unit_checksum_verification,
            &submissions.region_record_classification,
            &submissions.live_slots_per_cluster_segment,
        ] {
            note_the_segment_submissions(&mut anchors, segment, lowest_card);
        }
    }
    anchors
}

fn note_the_segment_submissions(
    anchors: &mut GpuWorkAnchors,
    segment: &GpuSubmissionsOfOneSegment,
    lowest_card: &GpuDeviceIdentifier,
) {
    if segment.submissions_completed == 0 {
        anchors.segments_without_a_submission += 1;
        if !segment.devices_given_work.is_empty() {
            anchors.segments_given_to_a_card_other_than_the_lowest += 1;
        }
    } else if segment.devices_given_work.as_slice() != std::slice::from_ref(lowest_card) {
        anchors.segments_given_to_a_card_other_than_the_lowest += 1;
    }
    anchors.recuts_after_running_out_of_memory += segment.recuts_after_running_out_of_memory;
    if segment.pieces_submitted != segment.submissions_completed {
        anchors.segments_with_pieces_other_than_submissions += 1;
    }
}

/// 第七节甲类第 3 行（多判臂上逐批核）：每批三段各自 确认 + 不确定 + 交 CPU 判 = 交进去的条目数。包装每批记下的交进去的条数与 GPU 那一臂的
/// 记账按先后一一对应；条数对不上（包装记下的批数与记账里走 GPU 的批数不等）也算不成立。交回不成立的批数。
pub(super) fn batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(
    records: &[GcCallRecord],
    handed_in_of_each_batch: &[EntriesHandedIn],
) -> u64 {
    let gpu_works: Vec<_> = records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .filter_map(|compute_record| match &compute_record.path_taken {
            BackgroundComputePathTaken::Gpu(work) => Some(work),
            BackgroundComputePathTaken::Cpu(_) => None,
        })
        .collect();
    let count_mismatch =
        u64::try_from(gpu_works.len().abs_diff(handed_in_of_each_batch.len())).expect("批数");
    let mismatched_batches = gpu_works
        .iter()
        .zip(handed_in_of_each_batch)
        .filter(|(work, handed_in)| {
            let added = |counts: singlefs_core::background_compute::GpuEntryCounts| {
                counts.confirmed_by_both_kernels
                    + counts.uncertain_this_round
                    + counts.judged_by_the_cpu_reference
            };
            added(work.unit_checksum_verification) != handed_in.unit_checksums
                || added(work.region_record_classification) != handed_in.region_records
                || added(work.live_slots_per_cluster_segment) != handed_in.cluster_segments
        })
        .count();
    count_mismatch + u64::try_from(mismatched_batches).expect("批数")
}

/// 第七节乙类第 2 行（不多判的 GPU 臂上只核得到相加形式）：每批活槽段的条目数是那块盘整段数的整数倍。交回不成立的批数。
pub(super) fn batches_whose_live_slot_entries_are_not_whole_device_passes(
    records: &[GcCallRecord],
    full_segments_of_a_device: u64,
) -> u64 {
    let not_whole = records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .filter_map(|compute_record| match &compute_record.path_taken {
            BackgroundComputePathTaken::Gpu(work) => Some(work.live_slots_per_cluster_segment),
            BackgroundComputePathTaken::Cpu(_) => None,
        })
        .filter(|counts| {
            let entries = counts.confirmed_by_both_kernels
                + counts.uncertain_this_round
                + counts.judged_by_the_cpu_reference;
            entries % full_segments_of_a_device != 0
        })
        .count();
    u64::try_from(not_whole).expect("批数")
}

/// 第八节「轨迹」三样（被谓词消费的量）：峰值、读数大于 0 的次数、期末值（最后一个读数）。没有读数时三个都是 0。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TrajectoryOfTheReadings {
    peak: u64,
    positive_readings: u64,
    last: u64,
}

pub(super) fn trajectory_of_the_readings(readings: &[u64]) -> TrajectoryOfTheReadings {
    TrajectoryOfTheReadings {
        peak: readings.iter().copied().max().unwrap_or(0),
        positive_readings: u64::try_from(readings.iter().filter(|reading| **reading > 0).count())
            .expect("读数个数"),
        last: readings.last().copied().unwrap_or(0),
    }
}

/// 第八节轨迹「每块盘的全空聚簇段数」的一块盘（登记第十二节修订第 5 条：每次 GC 调用开始前、返回后各一个读数，不按意图对齐）：
/// 读数的峰值、返回后比开始前多的调用数、期末值（最后一次调用返回后那一个）。没有读数时三个都是 0。只描述，不进任何判据。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct EmptySegmentsTrajectoryOfADevice {
    peak: u64,
    calls_gaining_empty_segments: u64,
    last: u64,
}

pub(super) fn empty_segments_trajectory_of_each_device(
    records: &[GcCallRecord],
) -> Vec<EmptySegmentsTrajectoryOfADevice> {
    let devices = records
        .iter()
        .map(|record| record.empty_segments_after_the_call.len())
        .max()
        .unwrap_or(0);
    (0..devices)
        .map(|device| {
            let mut trajectory = EmptySegmentsTrajectoryOfADevice::default();
            for record in records {
                let (Some(before), Some(after)) = (
                    record.empty_segments_before_the_call.get(device),
                    record.empty_segments_after_the_call.get(device),
                ) else {
                    continue;
                };
                trajectory.peak = trajectory.peak.max(*before).max(*after);
                if after > before {
                    trajectory.calls_gaining_empty_segments += 1;
                }
                trajectory.last = *after;
            }
            trajectory
        })
        .collect()
}

pub(super) fn empty_segments_fields(trajectories: &[EmptySegmentsTrajectoryOfADevice]) -> String {
    let joined = |value: fn(&EmptySegmentsTrajectoryOfADevice) -> u64| {
        field_value(
            &trajectories
                .iter()
                .map(|trajectory| value(trajectory).to_string())
                .collect::<Vec<String>>()
                .join(","),
        )
    };
    format!(
        "empty_segments_peak={} empty_segments_calls_gaining={} empty_segments_last={}",
        joined(|trajectory| trajectory.peak),
        joined(|trajectory| trajectory.calls_gaining_empty_segments),
        joined(|trajectory| trajectory.last),
    )
}

pub(super) fn trajectory_fields(name: &str, trajectory: TrajectoryOfTheReadings) -> String {
    format!(
        "{name}_peak={} {name}_positive_readings={} {name}_last={}",
        trajectory.peak, trajectory.positive_readings, trajectory.last
    )
}

/// 第八节轨迹「上一次用户改动给的预算还剩几次」：每次 GC 调用前后各一个读数，照核心层的式子（`B_gc` 减这次用户改动之后到此刻的整理发布数，
/// 不到 0 记 0）由记账推出，不是从核心层读的。
pub(super) fn budget_left_before_and_after_each_call(records: &[GcCallRecord]) -> Vec<u64> {
    let mut readings: Vec<u64> = Vec::new();
    let mut publishes_since_the_user_change: BTreeMap<u64, u64> = BTreeMap::new();
    for record in records {
        let publishes = publishes_since_the_user_change
            .entry(record.after_the_user_change)
            .or_insert(0);
        readings.push(LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE.saturating_sub(*publishes));
        *publishes += record.compaction_publishes;
        readings.push(LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE.saturating_sub(*publishes));
    }
    readings
}

/// 第八节轨迹「每批三段的不确定条目数」：每一项走 GPU 的记账一个读数（健康的卡上预期全 0）。
pub(super) fn uncertain_entries_of_each_gpu_record(records: &[GcCallRecord]) -> Vec<u64> {
    records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .filter_map(|compute_record| match &compute_record.path_taken {
            BackgroundComputePathTaken::Gpu(work) => {
                Some(work.entries_of_the_three_segments().uncertain_this_round)
            }
            BackgroundComputePathTaken::Cpu(_) => None,
        })
        .collect()
}

/// 第七节甲类第 4 行：相邻两次用户改动之间的整理发布次数的最大值（≤ B_gc = 20）。
pub(super) fn most_compaction_publishes_between_two_user_changes(records: &[GcCallRecord]) -> u64 {
    let mut publishes_after_each_change: BTreeMap<u64, u64> = BTreeMap::new();
    for record in records {
        *publishes_after_each_change
            .entry(record.after_the_user_change)
            .or_insert(0) += record.compaction_publishes;
    }
    publishes_after_each_change
        .values()
        .copied()
        .max()
        .unwrap_or(0)
}

/// 量 W：这次跑每一次 GC 调用的挂钟相加，起不起跑的都算（第九节变异「只计发了批的 GC 调用」改的是这里）。
pub(super) fn gc_wall_clock_of_the_run(
    windows: &[CallWindow],
    records: &[GcCallRecord],
) -> Duration {
    windows
        .iter()
        .zip(records)
        .map(|(window, _record)| window.wall)
        .sum()
}

/// 第五节包含自检（作废第 2 条）：每次调用挂钟 ≥ GC 线程 CPU；进程口径 ≥ GC 线程口径。交回不成立的调用数。
pub(super) fn calls_failing_the_inclusion_checks(windows: &[CallWindow]) -> u64 {
    let failing = windows
        .iter()
        .filter(|window| {
            window.wall < window.gc_thread_cpu || window.process_cpu < window.gc_thread_cpu
        })
        .count();
    u64::try_from(failing).expect("调用数")
}

/// CPU 臂（与 CPU 多判臂）每次跑 GC 线程 CPU ÷ GC 挂钟 ≥ 0.9（第七节乙类）。
pub(super) fn cpu_arm_thread_cpu_covers_the_wall(
    gc_thread_cpu: Duration,
    gc_wall: Duration,
) -> bool {
    gc_thread_cpu.as_secs_f64()
        >= CPU_ARM_THREAD_CPU_SHARE_OF_THE_WALL_AT_LEAST * gc_wall.as_secs_f64()
}

#[cfg(test)]
mod tests;
