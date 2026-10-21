//! checker 档模块：crash、crash_state_enumeration、device_log
//! 生命周期闭环流（里程碑四三·1：分配 → 覆盖写 → 建快照 → 再覆盖写 → 删文件 → GC 搬共享块 → 删快照 → 回收 → 在搬出来的空段上再分配）：
//! 「取号 → 暖机 × 2 → A（分配，一个数据单元）→ 覆盖写 → S（建快照）→ 快照下再覆盖写（覆盖写那一版的四个单元进 deadlist、映射条目留存）
//! → unlink 意图 → 最后一批 → 重挂 → 整理签发 → 三批 → 最后一批（R 里装着只被快照引用的单元，留存的映射条目跟着搬）→ 删快照（会话里一次调用：
//! 意图与各批，放掉的是搬到目的地之后的那几个单元）→ 回收（抬 F 到准入上限、建一次 inode，几轮之后 R 整段回收、会话对它发 discard）
//! → 在搬出来的空段上再分配（建 inode，直到 R 里有一条未释放的记录）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态（discard 罩住旧内容，
//! 照「范围写零」取三态）。公共前缀照里程碑二「增补 4」：取号加暖机、A 两个节点照标准前缀设，之后每一步照公共模块 `common_crash_points` 的入口名设，
//! 建快照、快照下覆盖写、unlink 两步与快照流共享，重挂与整理那几步与两条整理流共享。
//! 每个状态跑恢复 + 多版本 oracle（A、覆盖写、S、再覆盖写有文件，unlink 意图那一版起没有）、池级 checker、记录核对器；另逐状态核两样
//! （观察者在 `tests/common_compaction_crash_point_observers/mod.rs`）：落到的那一版的活快照读出建它那一刻（覆盖写那一版）的内容，抹掉整理搬走的源落点
//! （回收之后又分配出去的不抹）再读一遍；可写挂载一次（整理开关同录流），删除续做把 unlink 与删快照走到空、整理续做推到意图删了，挂完的盘过池级 checker。
//! 另有设备日志判器在这条流的录制流上判 discard（可发区间由判器自己从录制流里的发布算），连同判决 G5 的两个必红世界。
//!
//! 流的搭建住在公共模块 `crates/singlefs-harness/tests/common_compaction_prefix/mod.rs`。
//!
//! **比已有的流多罩了什么**：已有的流各罩生命周期的一截；这条流把快照、整理搬共享块、删快照、回收、discard 与再分配串在一条流上，
//! 第一次枚举到「删快照放掉的是整理搬过去的目的地」「discard 之后同一段再分配」两处的崩溃状态，也是唯一一条录制流里有 discard 的崩溃流。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例，带断点续跑、可双机分片。系统配置槽写与罩住旧内容的 discard 取三态，其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_compaction_crash_point_observers;
#[path = "../../singlefs-harness/tests/common_compaction_prefix/mod.rs"]
mod common_compaction_prefix;
mod common_crash_point_observers;
mod common_crash_points;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;

use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_counter_of, landed_version_observer,
    mount_continuation_observer, observe_the_landed_version_reading_through_the_mapping,
    observe_the_mount_continuing_the_compaction,
};
use common_compaction_prefix::{
    prepare_the_compaction_crash_flow, region_of_the_crash_flows, CompactionCrashFlow,
    CompactionFlowStepKind, PreparedCompactionCrashFlow, REGION_START_SLOT_OF_THE_CRASH_FLOWS,
};
use common_snapshot_prefix::SnapshotFlowStepKind;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_checker_tier::device_log::{
    expected_device_events, judge_discards_against_the_publishes, MisplacedDiscard,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
use singlefs_core::recovery::RecoveryReport;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs";

/// 段形（录流按真写表数出来的，141 段）：取号两轮、暖机、A 24；覆盖写 24；S 16；再覆盖写 26；unlink 意图 26、最后一批 24；重挂（取号两轮系统配置各 2、写行 22、
/// 暖机 16）；签发 18、三批 24、24、24、最后一批 24；删快照意图 16、三批 22、18、18、最后一批 16；三轮回收（抬 F：系统配置 2、两次空发布 16、16；
/// 建 inode 20）；再分配五次建 inode（回收发的 R 两块盘的 discard 并进紧接着那次建 inode 的单元段，22；其余 20、20、20、24）。
/// 每次发布各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 141] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2, 26, 2, 1, 2, 26, 2, 1, 2, 24, 2,
    1, 2, 2, 2, 22, 2, 1, 2, 16, 2, 1, 2, 18, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 24,
    2, 1, 2, 16, 2, 1, 2, 22, 2, 1, 2, 18, 2, 1, 2, 18, 2, 1, 2, 16, 2, 1, 2, 2, 16, 2, 1, 2, 16,
    2, 1, 2, 20, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 20, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2,
    20, 2, 1, 2, 22, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 24, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（取号写一轮时 288_751_838；两次取号各多一段 2，各 +3）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 288_751_844;

/// 快档展开的段每次写只取两态时的闭式（取号写一轮时 254）。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 260;

fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(tag, CompactionCrashFlow::LifecycleClosure, &[])
}

fn version_table(prepared: &PreparedCompactionCrashFlow) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
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

fn expansion_of_the_segments_shorter_than_ten(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < 10 {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 流里每一种那一版的计数名，每一种都要有状态。
fn landed_counters(prepared: &PreparedCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in &prepared.versions_of_the_flow {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}

/// 录流的形状（判器判完之后再核）：生命周期的九段按次序都在；整理搬了只被快照引用的单元；删快照之后快照、deadlist 与留存的映射条目都空；
/// 录制流里的 discard 恰好是 R 两块盘各一段；最后一版 R 里有一条未释放的记录（在搬出来的空段上再分配了）。
fn assert_the_shape_of_the_flow(prepared: &PreparedCompactionCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "生命周期闭环流的段形"
    );
    let recorded = &prepared.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded.steps.iter().map(|step| step.kind).collect();
    assert_eq!(
        kinds[..7],
        [
            CompactionFlowStepKind::OverwriteBeforeTheSnapshot,
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkIntent),
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkFinalBatch),
            CompactionFlowStepKind::MountWritable,
            CompactionFlowStepKind::CompactionIssue,
        ],
        "覆盖写 → S → 再覆盖写 → 删文件 → 重挂 → 整理签发"
    );
    for later_kind in [
        CompactionFlowStepKind::CompactionFinalBatch,
        CompactionFlowStepKind::DestroySnapshotThroughTheSession,
        CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling,
        CompactionFlowStepKind::NewInodesThroughTheSession,
    ] {
        assert!(kinds.contains(&later_kind), "流里有 {later_kind:?}");
    }
    assert!(
        recorded
            .relocated_units()
            .iter()
            .any(|unit| unit.is_kept_only_for_the_snapshots),
        "整理搬了只被快照引用的单元"
    );
    let destroyed = recorded
        .steps_of(CompactionFlowStepKind::DestroySnapshotThroughTheSession)
        .first()
        .and_then(|step| step.versions.last())
        .expect("删快照那一步的最后一版");
    assert!(
        common_snapshot_prefix::snapshot_trees_of(destroyed).is_empty()
            && destroyed.deadlist_entries.is_empty()
            && destroyed.mapping_entries_kept_for_the_snapshots.is_empty(),
        "删快照之后快照、deadlist 与留存的映射条目都空"
    );
    let discards = prepared.discards_in_the_stream();
    let region_segment = SlotNumber(REGION_START_SLOT_OF_THE_CRASH_FLOWS);
    assert_eq!(
        discards
            .iter()
            .map(|(_index, device, segment)| (*device, *segment))
            .collect::<Vec<_>>(),
        vec![
            (DeviceIdentity(0), region_segment),
            (DeviceIdentity(1), region_segment)
        ],
        "录制流里的 discard 恰好是 R 两块盘各一段（回收出来的整段）"
    );
    assert!(
        !region_is_empty_by_the_records(
            &recorded.pool.output.allocation_records,
            region_of_the_crash_flows()
        ),
        "最后一版在搬出来的空段 R 上再分配了"
    );
}

/// 快档那一趟：单元段不展开，每个状态交两个观察者。
fn enumerate_the_quick_tier(
    prepared: &PreparedCompactionCrashFlow,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
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
    let item_name = format!("lifecycle_closure::quick_tier::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(prepared),
        &expansion_of_the_segments_shorter_than_ten,
        &[landed_version_observer(prepared, &observe)],
        &[mount_continuation_observer(prepared, &observe_the_mount)],
    )
    .judged()
}

/// 平时跑的那一份：单元段不展开，其余每段任意真子集；每个状态读快照、抹掉源落点再读，再可写挂载一次核删除与整理续做。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_the_lifecycle_closure_recovers_reads_through_the_mapping_and_the_mount_continues_every_intent(
) {
    let prepared = prepare("lifecycle-closure-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_LIFECYCLE_CLOSURE_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_the_shape_of_the_flow(&prepared);
    assert_eq!(
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时展开的段按闭式数"
    );
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &expansion_of_the_segments_shorter_than_ten,
        ),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_lifecycle_closure_quick_tier_counts_the_same_when_rerun_from_the_library() {
    let prepared = prepare("lifecycle-closure-crash-point-replay-rerun");
    let Some(first_run) = enumerate_the_quick_tier(&prepared, "rerun") else {
        return;
    };
    let Some(second_run) = enumerate_the_quick_tier(&prepared, "rerun") else {
        return;
    };
    assert_eq!(second_run, first_run, "第二遍全照库里复用，计数逐项相同");
    assert_eq!(
        first_run.observed_states, first_run.states,
        "每个状态都看过"
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 读快照 + 抹掉源落点再读，并在工作线程上可写挂载一次核删除与整理续做。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_LIFECYCLE_CLOSURE` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 2.9 亿个状态、每个两遍恢复 + checker + 读快照与抹掉源落点再读 + 一次可写挂载续做；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_lifecycle_closure_stream_is_exhaustive_and_clean() {
    let prepared = prepare("lifecycle-closure-crash-point-replay-full");
    assert_eq!(
        closed_form_state_count(&prepared.segments),
        CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)"
    );
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
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
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "lifecycle_closure::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[landed_version_observer(&prepared, &observe)],
        &[mount_continuation_observer(&prepared, &observe_the_mount)],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_LIFECYCLE_CLOSURE states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    assert_eq!(tally.states, closed_form);
    assert_the_shape_of_the_flow(&prepared);
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// 判一条录制流两块盘上的 discard（设备日志用程序投到每块盘上的事件，可发区间由判器自己从流里的发布算）。
fn device_log_verdicts(
    prepared: &PreparedCompactionCrashFlow,
    operations: &[RetainedOperation],
) -> [Result<usize, MisplacedDiscard>; 2] {
    [DeviceIdentity(0), DeviceIdentity(1)].map(|device| {
        judge_discards_against_the_publishes(
            &prepared.base,
            operations,
            &common::geometry(),
            device,
            &expected_device_events(operations, device),
        )
    })
}

/// 设备日志判器（判决 G5）在这条流的录制流上：回收之后会话对 R 发的 discard 两块盘各一条，都落在那一刻判器自己算出的可发区间里、
/// 在使它可发的那次发布的最后一道屏障之后。
#[test]
fn the_discards_of_the_lifecycle_closure_land_in_the_ranges_the_device_log_judge_finds_discardable()
{
    let prepared = prepare("lifecycle-closure-device-log");
    assert_eq!(
        device_log_verdicts(&prepared, &prepared.operations_after_mkfs),
        [Ok(1), Ok(1)],
        "两块盘各判过一条 discard"
    );
}

/// 必红世界（判决 G5「discard 提前到根 FUA 之前」）：R 那两条 discard 挪到整理最后一批的根槽写之前——那时最近返回的是上一批，R 里还有近几代根
/// 引用的单元，要到之后回收那几轮才可发：判器判它提前到了使它可发的那次发布的最后一道屏障之前。
#[test]
fn the_discards_of_the_region_moved_before_the_root_record_write_of_the_final_batch_are_red() {
    let prepared = prepare("lifecycle-closure-device-log-moved-discard");
    let operations = &prepared.operations_after_mkfs;
    let mkfs_operations = prepared.recorded.pool.mkfs_operation_count;
    let final_batch = prepared
        .recorded
        .steps_of(CompactionFlowStepKind::CompactionFinalBatch)
        .first()
        .map(|step| step.operations.clone())
        .expect("最后一批");
    let geometry = common::geometry();
    let root_record_write_of_the_final_batch = (final_batch.start - mkfs_operations
        ..final_batch.end - mkfs_operations)
        .find(|index| geometry.classify(&operations[*index].operation) == StepKind::RootRecordFua)
        .expect("最后一批有一次根槽写");
    let discard_indexes: Vec<usize> = prepared
        .discards_in_the_stream()
        .iter()
        .map(|(index, _device, _segment)| *index)
        .collect();
    let mut moved: Vec<RetainedOperation> = Vec::with_capacity(operations.len());
    for (index, retained) in operations.iter().enumerate() {
        if index == root_record_write_of_the_final_batch {
            moved.extend(
                discard_indexes
                    .iter()
                    .map(|discard| operations[*discard].clone()),
            );
        }
        if !discard_indexes.contains(&index) {
            moved.push(retained.clone());
        }
    }
    let region_offset = DeviceOffsetInBytes(REGION_START_SLOT_OF_THE_CRASH_FLOWS * SLOT_BYTES);
    for verdict in device_log_verdicts(&prepared, &moved) {
        assert!(
            matches!(
                verdict,
                Err(MisplacedDiscard::BeforeTheLastBarrierOfThePublishThatMakesItDiscardable {
                    offset,
                    ..
                }) if offset == region_offset
            ),
            "R 的 discard 提前到最后一批的根槽写之前：判提前（之后回收那几轮才可发）：{verdict:?}"
        );
    }
}

/// 必红世界（判决 G5「discard 落在不可发的槽上」）：盘 0 上那条 discard 的段换成实例表第 0 片在盘 0 上的那一份所在的段——实例表是重挂写行时写的，
/// 从发 discard 的那一版到流尾每一版的根都指着同一份（之后没有挂载、没有清扫发布重写它），抹掉它那一段，每次发布返回时最新根的实例表那一份
/// 都读不对（I-2.1 逐份判校验和），在发它之前最近返回的那次发布之后不可发、之后也没有哪次发布让它可发：判器判它落在不可发的槽上；盘 1 那一条照旧判过。
/// 原先挪到发 discard 那一版树表所在的段上：那一版的树表之后几次发布就被换下、出了回退窗口，判器自己算出那一段之后会变成可发，报的是
/// 「提前到了使它可发的那次发布的最后一道屏障之前」（`MisplacedDiscard::BeforeTheLastBarrierOfThePublishThatMakesItDiscardable`），
/// 那一种已由上一条必红世界罩着；这一条要的是到流尾都不可发的段。
#[test]
fn a_discard_moved_onto_the_segment_of_the_instance_table_every_later_root_names_is_red() {
    let prepared = prepare("lifecycle-closure-device-log-misplaced-discard");
    let index_of_the_step_issuing_the_discards = prepared
        .recorded
        .steps
        .iter()
        .position(|step| !step.discards.is_empty())
        .expect("回收那一轮建 inode 之后发了 discard");
    let instance_table_when_the_discards_are_issued = prepared.recorded.steps
        [index_of_the_step_issuing_the_discards]
        .versions
        .last()
        .expect("那一步发成了一版")
        .root
        .instance_table;
    for later_version in prepared.recorded.steps[index_of_the_step_issuing_the_discards..]
        .iter()
        .flat_map(|step| step.versions.iter())
    {
        assert_eq!(
            later_version.root.instance_table, instance_table_when_the_discards_are_issued,
            "前提：从发 discard 的那一版到流尾，每一版的根都指着同一份实例表（txg {}）",
            later_version.root.checkpoint_txg.0
        );
    }
    let instance_table_slot_on_device_zero = instance_table_when_the_discards_are_issued
        .locations
        .iter()
        .find(|location| location.device == DeviceIdentity(0))
        .expect("实例表第 0 片在盘 0 上有一份")
        .slot;
    let misplaced_segment = SlotNumber(
        instance_table_slot_on_device_zero.0
            - instance_table_slot_on_device_zero.0 % singlefs_format::CLUSTER_SEGMENT_SLOTS,
    );
    assert_ne!(
        misplaced_segment.0, REGION_START_SLOT_OF_THE_CRASH_FLOWS,
        "前提：实例表那一份不在 R 里（R 刚回收）"
    );
    let mut misplaced = prepared.operations_after_mkfs.clone();
    let (discard_on_device_zero, _device, _segment) = *prepared
        .discards_in_the_stream()
        .iter()
        .find(|(_index, device, _segment)| *device == DeviceIdentity(0))
        .expect("盘 0 上的 discard");
    misplaced[discard_on_device_zero].operation.offset =
        DeviceOffsetInBytes(misplaced_segment.0 * SLOT_BYTES);
    let [on_device_zero, on_device_one] = device_log_verdicts(&prepared, &misplaced);
    assert!(
        matches!(
            on_device_zero,
            Err(MisplacedDiscard::OutsideTheDiscardableRanges { offset, .. })
                if offset == DeviceOffsetInBytes(misplaced_segment.0 * SLOT_BYTES)
        ),
        "盘 0：落在到流尾每一版的根都指着的实例表那一份所在的段上：{on_device_zero:?}"
    );
    assert_eq!(on_device_one, Ok(1), "盘 1 那一条照旧判过");
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；展开上限默认 6。崩溃点照公共模块 `common_crash_points` 设（`crash_points_of_a_flow_after_the_first_file`）。
/// 登记表挂在标准前缀节点上的观察者（可写挂载续做）与挂在整理那几步节点上的两个观察者（读、可写挂载续做，`common_compaction_crash_point_observers::OBSERVERS_ON_THE_COMPACTION_STEPS`）照表跑。录制流里有 discard：配了 GPU 时 GPU 判器建判表那一步认不认 discard 归 GPU 那一侧
/// （`crash_judge_tables` 的 `DiscardInTheEnumeratedWrites`）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_lifecycle_closure_stream_is_clean() {
    let prepared = prepare("crash-amplification-lifecycle-closure");
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    common_compaction_crash_point_observers::amplify_a_compaction_flow(
        "lifecycle-closure",
        &prepared,
        crash_points,
        6,
    );
}

/// 放量计划上整理那几步的观察者（`common_compaction_crash_point_observers::OBSERVERS_ON_THE_COMPACTION_STEPS`）：计划里 A 之后、标准前缀之外的每一个点
/// 都挂着读与可写挂载续做两个观察者，别的点不挂；两个观察者各挂的点数等于另按这条流每一步的入口名数出来的点数，不是 0。只排计划、不判。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_plan_of_the_lifecycle_closure_stream_hangs_both_compaction_observers_on_every_point_after_the_standard_prefix(
) {
    let prepared = prepare("crash-amplification-lifecycle-closure-observer-nodes");
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    let points = common_compaction_crash_point_observers::assert_the_compaction_observers_sit_on_every_planned_point_after_the_standard_prefix(
        "lifecycle-closure",
        &prepared,
        crash_points,
        6,
    );
    println!("CRASH_AMPLIFICATION_COMPACTION_OBSERVER_NODES flow=lifecycle-closure points_with_both_observers={points}");
}
