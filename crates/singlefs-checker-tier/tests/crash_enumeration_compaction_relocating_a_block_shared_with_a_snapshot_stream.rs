//! checker 档模块：crash、crash_state_enumeration
//! 整理搬被快照共享的块（里程碑四二·3「搬被共享的块一条」，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「被快照引用的块照样能搬」、G6）：
//! 「取号 → 暖机 × 2 → A（一个数据单元）→ S（建快照）→ W（快照下覆盖写：A 的数据单元、extent 根、inode 叶容器、inode 根进 deadlist、映射条目留存）
//! → 重挂 → 整理签发 → 三批 → 最后一批（批大小 2）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。R 强制取第一个文件那次发布的提交内生块所在的段：
//! A 的 extent 根、inode 叶容器、inode 根在里面，W 之后它们只被快照引用（留存的映射条目，`RelocatedUnitHolder::KeptForTheSnapshots`），
//! 整理把它们原样拷到目的地、留存的映射条目跟着换位置；同一段里活头的几个单元一起搬。每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；
//! 另逐状态核两样（观察者在 `tests/common_compaction_crash_point_observers/mod.rs`）：
//! - 恢复落到的那一版的活快照读出建它那一刻（A）的内容；再把那一版与它之前搬走的单元的源落点整槽抹成 0，第一个文件（W 的内容）与活快照照样读得回：
//!   快照一侧的树里还是旧提示，读路径要照留存的映射条目走到目的地（判决 G6 的必红世界「只更新活头一侧的位置、快照一侧读旧提示」在这一样与池级 checker 上红）；
//! - 可写挂载一次（整理开关同录流）：落到签发与各批的状态上整理续做推到意图删了，挂完快照照样读得回、挂完的盘在判段过池级 checker。
//!
//! 流的搭建住在公共模块 `crates/singlefs-harness/tests/common_compaction_prefix/mod.rs`（建快照与快照下覆盖写照 `common_snapshot_prefix` 那一份）。
//!
//! **比已有的流多罩了什么**：整理搬迁流上没有快照，这里整理第一次搬只被快照引用的单元（中央映射树里留存的映射条目换到目的地、deadlist 条目不动），
//! 崩在这几批的单元、记录、根槽写之间的每个状态第一次被枚举到。前缀到 W 与快照下覆盖写与删文件流逐字节相同（建快照、覆盖写两个节点共享），
//! 多跑的两步同整理搬迁流。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例，带断点续跑、可双机分片。系统配置槽写是原地覆写，取三态，其余写取两态。

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
    CompactionFlowStepKind, PreparedCompactionCrashFlow,
};
use common_snapshot_prefix::{
    live_snapshot_trees_of, SnapshotFlowStepKind, UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs";

/// 段形（录流按真写表数出来的）：前二十段同快照下覆盖写与删文件流到 W 为止（取号两轮各一段）；重挂（取号两轮系统配置各 2、写行 18、暖机 16 个单元写）；
/// 签发 18、三批 20、24、24、最后一批 22 个单元写——各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 50] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2, 26, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    18, 2, 1, 2, 20, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 22, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（取号写一轮时 123_338_833；两次取号各多一段 2，各 +3）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 123_338_839;

/// 快档展开的段每次写只取两态时的闭式（取号写一轮时 91）。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 97;

fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(
        tag,
        CompactionCrashFlow::CompactionRelocatingABlockSharedWithASnapshot,
        &[],
    )
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

/// 录流的形状（判器判完之后再核）：步的次序；搬走的单元里有只被快照引用的（留存的映射条目跟着搬），源落点都在 R 里、目的落点都在 R 外；
/// 最后一批那一版留存的映射条目与 deadlist 条目仍各四条、快照仍在，留存的映射条目没有一条还指着 R。
fn assert_the_shape_of_the_flow(prepared: &PreparedCompactionCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前二十段同快照下覆盖写与删文件流到 W；重挂、签发、三批、最后一批"
    );
    let recorded = &prepared.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded.steps.iter().map(|step| step.kind).collect();
    assert_eq!(
        kinds[..4],
        [
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
            CompactionFlowStepKind::MountWritable,
            CompactionFlowStepKind::CompactionIssue,
        ],
        "S → W → 重挂 → 签发"
    );
    assert_eq!(
        kinds.last(),
        Some(&CompactionFlowStepKind::CompactionFinalBatch),
        "最后一步是最后一批"
    );
    let region = region_of_the_crash_flows();
    let relocated = recorded.relocated_units();
    assert!(
        relocated
            .iter()
            .any(|unit| unit.is_kept_only_for_the_snapshots),
        "整理搬了只被快照引用的单元：{relocated:?}"
    );
    for unit in &relocated {
        assert!(
            unit.source
                .iter()
                .all(|(_device, slot)| region.contains_slot(*slot))
                && unit
                    .destination
                    .iter()
                    .all(|(_device, slot)| !region.contains_slot(*slot)),
            "源落点在 R 里、目的落点在 R 外：{unit:?}"
        );
    }
    let final_batch = recorded
        .steps_of(CompactionFlowStepKind::CompactionFinalBatch)
        .first()
        .and_then(|step| step.versions.last())
        .expect("最后一批那一版");
    assert_eq!(
        (
            final_batch.mapping_entries_kept_for_the_snapshots.len(),
            final_batch.deadlist_entries.len()
        ),
        (
            UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
            UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT
        ),
        "搬完之后留存的映射条目与 deadlist 条目仍各四条（只换位置）"
    );
    assert!(
        final_batch
            .mapping_entries_kept_for_the_snapshots
            .iter()
            .all(|(_key, locations)| locations
                .iter()
                .all(|location| !region.contains_slot(location.slot))),
        "留存的映射条目没有一条还指着 R"
    );
    assert_eq!(
        live_snapshot_trees_of(final_batch),
        vec![recorded.snapshot.expect("这条流建过快照")],
        "快照仍在"
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
    let item_name =
        format!("compaction_relocating_a_block_shared_with_a_snapshot::quick_tier::{item_label}");
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

/// 平时跑的那一份：单元段不展开，其余每段任意真子集；每个状态读快照、抹掉源落点再读第一个文件与快照，再可写挂载一次核整理续做。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_relocating_a_block_shared_with_a_snapshot_keeps_the_snapshot_readable_through_the_kept_mapping_entries(
) {
    let prepared = prepare("compaction-relocating-a-shared-block-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATING_A_BLOCK_SHARED_WITH_A_SNAPSHOT_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
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
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "compaction_relocating_a_block_shared_with_a_snapshot",
        &tally,
    );
    assert!(
        tally
            .observer_counts
            .count_named("live_snapshots_read_at_their_creation_content")
            > 0,
        "落到 S 起的状态读过快照：{:?}",
        tally.observer_counts
    );
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_compaction_relocating_a_block_shared_with_a_snapshot_quick_tier_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("compaction-relocating-a-shared-block-crash-point-replay-rerun");
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

/// 全量：每段都展开，每个状态两遍恢复 + checker + 读快照 + 抹掉源落点再读，并在工作线程上可写挂载一次核整理续做。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_COMPACTION_RELOCATING_A_BLOCK_SHARED_WITH_A_SNAPSHOT` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 1.2 亿个状态、每个两遍恢复 + checker + 读快照 + 抹掉源落点再读 + 一次可写挂载续做；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_compaction_relocating_a_block_shared_with_a_snapshot_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-relocating-a-shared-block-crash-point-replay-full");
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
            name: "compaction_relocating_a_block_shared_with_a_snapshot::full",
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
        "CRASH_POINT_REPLAY_COMPACTION_RELOCATING_A_BLOCK_SHARED_WITH_A_SNAPSHOT states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "compaction_relocating_a_block_shared_with_a_snapshot",
        &tally,
    );
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；展开上限默认 6。崩溃点照公共模块 `common_crash_points` 设：建快照、快照下覆盖写与快照流共享，
/// 重挂与整理那几步与整理搬迁流、生命周期闭环流共享入口名。登记表挂在标准前缀节点上的观察者（可写挂载续做）与挂在整理那几步节点上的两个观察者（读、可写挂载续做，`common_compaction_crash_point_observers::OBSERVERS_ON_THE_COMPACTION_STEPS`）照表跑。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_compaction_relocating_a_block_shared_with_a_snapshot_stream_is_clean()
{
    let prepared = prepare("crash-amplification-compaction-relocating-a-shared-block");
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    common_compaction_crash_point_observers::amplify_a_compaction_flow(
        "compaction-relocating-a-block-shared-with-a-snapshot",
        &prepared,
        crash_points,
        6,
    );
}

/// 放量计划上整理那几步的观察者（`common_compaction_crash_point_observers::OBSERVERS_ON_THE_COMPACTION_STEPS`）：计划里 A 之后、标准前缀之外的每一个点
/// 都挂着读与可写挂载续做两个观察者，别的点不挂；两个观察者各挂的点数等于另按这条流每一步的入口名数出来的点数，不是 0。只排计划、不判。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_plan_of_the_compaction_relocating_a_block_shared_with_a_snapshot_stream_hangs_both_compaction_observers_on_every_point_after_the_standard_prefix(
) {
    let prepared =
        prepare("crash-amplification-compaction-relocating-a-shared-block-observer-nodes");
    let crash_points = common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    );
    let points = common_compaction_crash_point_observers::assert_the_compaction_observers_sit_on_every_planned_point_after_the_standard_prefix(
        "compaction-relocating-a-block-shared-with-a-snapshot",
        &prepared,
        crash_points,
        6,
    );
    println!("CRASH_AMPLIFICATION_COMPACTION_OBSERVER_NODES flow=compaction-relocating-a-block-shared-with-a-snapshot points_with_both_observers={points}");
}
