//! 验证包模块：crash、crash_state_enumeration、crash_library
//! 整理搬只被快照引用的单元、deadlist 跨两代实例记与清、回收的流（录法在 `common_compaction_prefix` 的
//! `record_the_compaction_of_snapshot_units_flow`）。
//! 流头在基镜像里、不枚举：实例 1 第一个文件 → 建快照之前覆盖写第一个文件、连带建空 inode（inode 树连同第一个文件填满 6 个叶容器，
//! 容器、inode 根与 extent 根都落在整理区域 R 里）→ 建快照。
//! 枚举的是之后那一串：快照下覆盖写第一个文件（换下的四个单元记进 deadlist）→ 快照下逐个容器给一个 inode 写一个单元（快照抄的每个容器换下来，
//! deadlist 一次加一条）→ unlink 意图、最后一批 → 实例 2 重挂 → 整理签发、几批、最后一批（把 R 里只被快照引用的单元搬走）
//! → 会话里删快照（实例 2 一批放一条，清空 deadlist）→ 抬 F、建 inode，直到 R 整段回收 → 在 R 上再分配。
//! 读路径观察者核每一版的活快照与搬迁之后照映射读，可写观察者在崩溃镜像上续做整理。
//!
//! 状态数（录制流闭式计数）写在 `the_recorded_stream_counts_its_crash_states`，全量没判过；每一步的状态数与占比由
//! `compaction_and_deadlist_steps_hold_more_than_half_of_the_crash_states` 现算、逐步打出来，整理三种步与 deadlist 记、清那两种步合计过半。
//! 崩溃放量那一路的基镜像取在 mkfs 之后（标准前缀上的观察者要有状态可看），展开到 6 写的段。
//!
//! 每段都展开、与闭式比，登记成崩溃枚举用例，带断点续跑、可双机分片。

mod common;
mod common_compaction_crash_point_observers;
mod common_compaction_prefix;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_prefix;

use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_counter_of, landed_version_observer,
    mount_continuation_observer, observe_the_landed_version_reading_through_the_mapping,
    observe_the_mount_continuing_the_compaction,
};
use common_compaction_prefix::{
    prepare_the_compaction_of_snapshot_units_flow, BaseImageOfTheCompactionOfSnapshotUnitsFlow,
    CompactionFlowStepKind, PreparedCompactionCrashFlow,
};
use common_snapshot_prefix::SnapshotFlowStepKind;
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_verification::crash_library::StageOutcome;
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_verification::memory_pool::CrashImage;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_compaction_deadlist_reclamation_stream.rs";

/// 录制流闭式计数（`crash_state_count_with_torn_in_place_overwrites` 按全量展开数，基镜像取在流头之后）。
const FULL_STATE_COUNT: u64 = 10_728_767_988;

/// 录制流每段的写数（基镜像取在流头之后；只录探针录出的那一串）。
const SEGMENT_SIZES: [usize; 170] = [
    26, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 32, 2, 1, 2, 32, 2, 1, 2, 30, 2, 1, 2, 26,
    2, 1, 2, 2, 2, 22, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1, 2, 18, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2,
    24, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2, 22, 2, 1, 2, 18, 2, 1, 2, 18, 2, 1, 2, 18,
    2, 1, 2, 18, 2, 1, 2, 18, 2, 1, 2, 18, 2, 1, 2, 18, 2, 1, 2, 16, 2, 1, 2, 2, 16, 2, 1, 2, 16,
    2, 1, 2, 20, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 20, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2,
    20, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 20, 2, 1, 2, 22, 2, 1, 2, 24, 2, 1, 2,
];

/// 整理至少搬几个只被快照引用的单元（生命周期闭环流整理那一截一共搬 8 个单元，这条流只数只被快照引用的也不少于它）。
const SNAPSHOT_UNITS_RELOCATED_AT_LEAST: usize = 8;

/// 整理至少几次发布搬了单元（几批加最后一批，不少于生命周期闭环流的三批加最后一批）。
const RELOCATING_PUBLISHES_AT_LEAST: usize = 4;

fn prepare_with_the_base_image(
    tag: &str,
    base_image: BaseImageOfTheCompactionOfSnapshotUnitsFlow,
) -> PreparedCompactionCrashFlow {
    let prepared = prepare_the_compaction_of_snapshot_units_flow(tag, base_image);
    for kind in [
        CompactionFlowStepKind::OverwriteBeforeTheSnapshot,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
        CompactionFlowStepKind::MountWritable,
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
        CompactionFlowStepKind::CompactionFinalBatch,
        CompactionFlowStepKind::DestroySnapshotThroughTheSession,
        CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling,
        CompactionFlowStepKind::NewInodesThroughTheSession,
    ] {
        assert!(
            !prepared.recorded.steps_of(kind).is_empty(),
            "录制流必须实际包含 {kind:?}"
        );
    }
    assert!(
        prepared
            .recorded
            .steps
            .iter()
            .any(|step| !step.discards.is_empty()),
        "回收真实发出 discard"
    );
    prepared
}

/// 崩溃点重放那一路：基镜像取在流头之后。
fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_with_the_base_image(
        tag,
        BaseImageOfTheCompactionOfSnapshotUnitsFlow::AfterTheStreamHead,
    )
}

fn version_table(prepared: &PreparedCompactionCrashFlow) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(
        tally.violations, 0,
        "oracle 违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "忽略 journal 恢复时 oracle 违例：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "状态走读失败数必须为 0");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据违例"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 崩溃状态落得到的那几版的计数名（基镜像里更早的几版落不到，不进这张表）。
fn landed_counters(prepared: &PreparedCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in prepared.versions_landed_on_from_the_base_image() {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}

/// 一步是不是整理（签发、一批、最后一批）或 deadlist（快照下的写记进去、删快照清掉）那两截的。
fn is_a_step_of_the_compaction_or_the_deadlist(kind: CompactionFlowStepKind) -> bool {
    match kind {
        CompactionFlowStepKind::CompactionIssue
        | CompactionFlowStepKind::CompactionBatch
        | CompactionFlowStepKind::CompactionFinalBatch
        | CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot)
        | CompactionFlowStepKind::DestroySnapshotThroughTheSession => true,
        CompactionFlowStepKind::SequentialWrite
        | CompactionFlowStepKind::OverwriteBeforeTheSnapshot
        | CompactionFlowStepKind::Snapshot(
            SnapshotFlowStepKind::CreateSnapshot
            | SnapshotFlowStepKind::UnlinkIntent
            | SnapshotFlowStepKind::UnlinkFinalBatch
            | SnapshotFlowStepKind::SnapshotDestructionIntent
            | SnapshotFlowStepKind::SnapshotDestructionBatch
            | SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
        )
        | CompactionFlowStepKind::MountWritable
        | CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling
        | CompactionFlowStepKind::NewInodesThroughTheSession => false,
    }
}

/// 判段整份的结果。只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`、配了 GPU 而这份二进制没编 gpu 特性）交只录，
/// 照两段协议收工、不判；两台分片的一份交「只判了自己那一份」，只在设了分片变量时认，别的时候判红。
fn judged_tally_or_left_to_another_stage(
    outcome: StageOutcome<CrashPointReplayTally>,
) -> Option<CrashPointReplayTally> {
    match outcome {
        StageOutcome::Judged(tally) => Some(tally),
        StageOutcome::RecordedOnly => None,
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => {
            assert!(
                std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_SHARE").is_some(),
                "没设分片变量却只判了自己那一份（本机 {blocks_judged_here} 块、留给别的机器 {blocks_left_to_other_hosts} 块）"
            );
            None
        }
    }
}

/// 全量：每段展开任意真子集，可双机分片、断点续跑。
#[test]
fn full_enumeration_of_the_compaction_deadlist_reclamation_stream_is_exhaustive_and_clean() {
    let prepared = prepare("compaction-deadlist-reclamation-full");
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_through_the_mapping(
            versions_of_the_flow,
            image,
            report,
            counts,
        );
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuing_the_compaction(versions_of_the_flow, image, report, counts)
        };
    let tally_outcome = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "compaction_deadlist_reclamation::full",
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[landed_version_observer(&prepared, &observe)],
        &[mount_continuation_observer(&prepared, &observe_the_mount)],
    );
    let Some(tally) = judged_tally_or_left_to_another_stage(tally_outcome) else {
        return;
    };
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_DEADLIST_RECLAMATION states={} closed_form={} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        closed_form,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "compaction_deadlist_reclamation_stream：用例自己的展开下的闭式",
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// 崩溃放量：按步骤设点、展开不超过引擎展开上限的段，经判定库判。基镜像取在 mkfs 之后：流头那几步也设点，标准前缀上的观察者有状态可看。
#[test]
fn crash_amplification_of_the_compaction_deadlist_reclamation_stream_is_clean() {
    let prepared = prepare_with_the_base_image(
        "crash-amplification-compaction-deadlist-reclamation",
        BaseImageOfTheCompactionOfSnapshotUnitsFlow::AfterTheMakeFilesystem,
    );
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    common_compaction_crash_point_observers::amplify_a_compaction_flow(
        "compaction-deadlist-reclamation",
        &prepared,
        crash_points,
    );
}

/// 录下的流有名字说的三样，只录流、不枚举：整理搬了不少于 [`SNAPSHOT_UNITS_RELOCATED_AT_LEAST`] 个只被快照引用的单元、
/// 搬单元的发布不少于 [`RELOCATING_PUBLISHES_AT_LEAST`] 次；快照下的写把换下的单元记进 deadlist（实例 1），删快照那一步之后 deadlist 清空、
/// 清它的是下一代实例（实例 2）；段形与录出来的那一串相同。R 整段回收、在 R 上再分配由录法自己核（做不成就录不下来）。
#[test]
fn the_recorded_stream_relocates_snapshot_units_and_a_later_instance_clears_the_deadlist() {
    let prepared = prepare("compaction-deadlist-reclamation-shape");
    let relocated = prepared.recorded.relocated_units();
    let snapshot_units_relocated = relocated
        .iter()
        .filter(|unit| unit.is_kept_only_for_the_snapshots)
        .count();
    assert!(
        snapshot_units_relocated >= SNAPSHOT_UNITS_RELOCATED_AT_LEAST,
        "整理搬了 {snapshot_units_relocated} 个只被快照引用的单元，要不少于 {SNAPSHOT_UNITS_RELOCATED_AT_LEAST} 个：{relocated:?}"
    );
    let relocating_publishes = prepared
        .recorded
        .steps
        .iter()
        .filter(|step| {
            matches!(
                step.kind,
                CompactionFlowStepKind::CompactionBatch
                    | CompactionFlowStepKind::CompactionFinalBatch
            )
        })
        .flat_map(|step| step.versions.iter())
        .filter(|version| !version.units_relocated_by_this_publish.is_empty())
        .count();
    assert!(
        relocating_publishes >= RELOCATING_PUBLISHES_AT_LEAST,
        "整理搬单元的发布 {relocating_publishes} 次，要不少于 {RELOCATING_PUBLISHES_AT_LEAST} 次（几批加最后一批）"
    );
    let deadlist_written = prepared
        .recorded
        .steps_of(CompactionFlowStepKind::Snapshot(
            SnapshotFlowStepKind::OverwriteUnderTheSnapshot,
        ))
        .last()
        .and_then(|step| step.versions.last())
        .expect("快照下最后一次写那一版")
        .clone();
    assert!(
        !deadlist_written.deadlist_entries.is_empty(),
        "快照下的写把换下的单元记进 deadlist"
    );
    let deadlist_cleared = prepared
        .recorded
        .steps_of(CompactionFlowStepKind::DestroySnapshotThroughTheSession)
        .last()
        .and_then(|step| step.versions.last())
        .expect("删快照那一步的最后一版")
        .clone();
    assert!(
        deadlist_cleared.deadlist_entries.is_empty(),
        "删快照之后 deadlist 清空：还剩 {} 条",
        deadlist_cleared.deadlist_entries.len()
    );
    assert_ne!(
        deadlist_cleared.root.instance, deadlist_written.root.instance,
        "deadlist 由下一代实例清空（跨代）"
    );
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES.to_vec(),
        "录制流的段形"
    );
}

/// 只录流，按步现算状态数（只展开那一步的段，其余段整段持久），逐步打一行；整理（签发、各批、最后一批）与 deadlist（快照下的写记、删快照清）
/// 两截的状态合计占全量过半。不枚举、不开判定库。
#[test]
fn compaction_and_deadlist_steps_hold_more_than_half_of_the_crash_states() {
    let prepared = prepare("compaction-deadlist-reclamation-share");
    let full = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let step_of_each_segment = prepared.step_position_of_each_segment();
    let states_of_the_segments_of = |is_counted: &dyn Fn(Option<usize>) -> bool| -> u64 {
        let expansion = |segment_index: usize, _segment: &[usize]| {
            if is_counted(step_of_each_segment[segment_index]) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        };
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &expansion,
        ) - 1
    };
    for (position, step) in prepared.recorded.steps.iter().enumerate() {
        let states =
            states_of_the_segments_of(&|step_of_the_segment| step_of_the_segment == Some(position));
        if states > 0 {
            println!(
                "CRASH_ENUMERATION_STEP_STATES name=compaction_deadlist_reclamation step={position}:{:?} states={states} per_ten_thousand_of_the_full_count={}",
                step.kind,
                u128::from(states) * 10_000 / u128::from(full)
            );
        }
    }
    let states_of_the_compaction_and_the_deadlist =
        states_of_the_segments_of(&|step_of_the_segment| {
            step_of_the_segment.is_some_and(|position| {
                is_a_step_of_the_compaction_or_the_deadlist(prepared.recorded.steps[position].kind)
            })
        });
    println!(
        "CRASH_ENUMERATION_COMPACTION_AND_DEADLIST_STATES name=compaction_deadlist_reclamation states={states_of_the_compaction_and_the_deadlist} full={full}"
    );
    assert!(
        u128::from(states_of_the_compaction_and_the_deadlist) * 2 > u128::from(full),
        "整理与 deadlist 两截的状态 {states_of_the_compaction_and_the_deadlist} 要占全量 {full} 的一半以上"
    );
}

/// 只录流，按写流算全量状态数，不枚举、不开判定库：录法变了而 `FULL_STATE_COUNT` 没跟着改就红。
#[test]
fn the_recorded_stream_counts_its_crash_states() {
    let prepared = prepare("compaction-deadlist-reclamation-count");
    let states = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    println!(
        "CRASH_ENUMERATION_STATE_COUNT name=compaction_deadlist_reclamation states={states} writes={} segments={:?}",
        prepared.writes.len(),
        prepared.segment_sizes()
    );
    assert_eq!(states, FULL_STATE_COUNT, "录制流的全量状态数");
}
