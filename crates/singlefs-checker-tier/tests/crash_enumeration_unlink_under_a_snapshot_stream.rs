//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 快照下删文件流（里程碑四一·4，建快照之后直接 unlink 的那一条快照流，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S6）：
//! 「取号 → 暖机 × 2 → A（一个数据单元）→ S（建快照）→ unlink 意图 → 最后一批（批大小 1）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。
//! A 的四个用户可见单元（数据单元、extent 根、inode 叶容器、inode 根）都生在 S 之前、被快照引用：unlink 的意图与最后一批换下它们时写进 deadlist
//! （头 = 活头 inode 树、纪元 = S.txg）、映射条目留存，不释放——这是快照下覆盖写与删文件流走不到的那条路（那条流上覆盖写已经把它们换掉，
//! unlink 换下的都生在 S 之后）。每个状态跑恢复 + 多版本 oracle（A、S 有文件，意图那一版起没有）、池级 checker、记录核对器；
//! 另逐状态核两样（观察者在 `tests/common_snapshot_crash_point_observers/mod.rs`）：
//! - 恢复落到的那一版的活快照在崩溃镜像上读出建它那一刻（A）的内容：最后一批之后它的四个单元只被快照与留存的映射条目引用；
//! - 可写挂载一次：续做把 unlink 走到空（落到意图那一版的续做一步），快照、deadlist 与留存的映射条目与录流时最后一批那一版逐项相同；
//!   挂完的盘在判段过池级 checker。
//!
//! 流的搭建住在公共模块 `crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs`（四条快照流共用一份录法）。
//!
//! **比已有的流多罩了什么**：删除批经 unlink 的候选角色把被快照引用的块写进 deadlist（`deadlist_changes_of_this_publish` 那一段）第一次被走到，
//! 崩在意图与最后一批的单元、记录、根槽写之间的每个状态第一次被枚举到。前缀到 S 与建快照流逐字节相同（建快照节点共享）。
//! 多跑的两步同快照下覆盖写与删文件流。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例，带断点续跑、可双机分片。系统配置槽写是原地覆写，取三态，其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_crash_point_observers;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;

use common_snapshot_crash_point_observers::{
    mount_continuation_observer_under_a_snapshot, observe_the_mount_continuation_under_a_snapshot,
    observe_the_snapshot_of_the_landed_version, snapshot_observer_of_every_state,
};
use common_snapshot_prefix::{
    prepare_the_snapshot_crash_flow, PreparedSnapshotCrashFlow, SnapshotCrashFlow,
    SnapshotFlowStepKind, POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_under_a_snapshot_stream.rs";

/// 段形（录流按真写表数出来的）：前十六段同建快照流（取号两轮各一段）；unlink 意图 28 个单元写、最后一批 26 个单元写，各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 24] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 26, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（取号写一轮时 352_387_114，取号第二轮那一段 +3）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 352_387_117;

/// 快档展开的段每次写只取两态时的闭式（取号写一轮时 46）。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 49;

/// A 被快照引用的用户可见单元：数据单元、extent 根、inode 叶容器、inode 根（判决 S3 的六类角色里这个文件用到的四类）。
const UNITS_OF_THE_FIRST_FILE_KEPT_FOR_THE_SNAPSHOT: usize = 4;

/// unlink 意图那次发布换下、写进 deadlist 的单元数（录流按真数：意图重写 inode 那一侧的两个，最后一批再换下其余两个）。
const DEADLIST_ENTRIES_AFTER_THE_UNLINK_INTENT: usize = 2;

/// 恢复落到的那一版是哪一种（计数名），每一种都要有状态。
const LANDED_COUNTERS: [&str; 5] = [
    "landed_before_the_first_file",
    "landed_on_first_file",
    "landed_on_create_snapshot",
    "landed_on_unlink_intent",
    "landed_on_unlink_final_batch",
];

fn prepare(tag: &str) -> PreparedSnapshotCrashFlow {
    prepare_the_snapshot_crash_flow(tag, SnapshotCrashFlow::UnlinkUnderASnapshot)
}

fn version_table(prepared: &PreparedSnapshotCrashFlow) -> CrashPointReplayVersionTable<'_> {
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

/// 两个观察者都看过每个状态：恢复落到的每一版都有状态；落到 S 起的每一版都读了它那一个活快照；每个状态都可写挂载过一次，
/// 落到意图那一版的续做一步、别的不续做；挂完的盘都过了池级 checker。
fn assert_every_state_was_observed(tally: &CrashPointReplayTally) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    assert_eq!(
        LANDED_COUNTERS
            .iter()
            .map(|name| counts.count_named(name))
            .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一版或 A 之前：{counts:?}"
    );
    for name in LANDED_COUNTERS {
        assert!(
            counts.count_named(name) > 0,
            "{name} 的状态真跑到了：{counts:?}"
        );
    }
    assert_eq!(
        counts.count_named("live_snapshots_read_at_their_creation_content"),
        LANDED_COUNTERS[2..]
            .iter()
            .map(|name| counts.count_named(name))
            .sum::<u64>(),
        "落到 S 起的每一版的每个状态都读了它那一个活快照"
    );
    assert_eq!(
        counts.count_named("mount_continuation_steps_0")
            + counts.count_named("mount_continuation_steps_1"),
        tally.states,
        "每个状态都可写挂载过一次，续做至多一步"
    );
    assert_eq!(
        counts.count_named("mount_continuation_steps_1"),
        counts.count_named("landed_on_unlink_intent"),
        "落到意图那一版的续做一步（最后一批），别的不续做"
    );
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        tally.states,
        "每个状态挂完的盘都过了池级 checker（判段在挂完的盘上判，配了 GPU 上卡）"
    );
}

/// 录流的形状（判器判完之后再核）：意图那一版 deadlist 两条、最后一批那一版四条（A 被快照引用的四个单元），每一条的头是活头 inode 树、
/// 纪元是 S.txg、birth 在 S 之前，一条 deadlist 条目一条留存的映射条目；两版快照都在、没有销毁中的，最后一批之后删除队列空。
fn assert_the_shape_of_the_flow(prepared: &PreparedSnapshotCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前十六段同建快照流；unlink 意图 28、最后一批 26 个单元写，各自记录 2、根 1、轮换 2"
    );
    let recorded = &prepared.recorded;
    let snapshot_txg = recorded
        .outputs_of(SnapshotFlowStepKind::CreateSnapshot)
        .first()
        .expect("建快照那一步")
        .root
        .checkpoint_txg;
    for (kind, deadlist_entries) in [
        (
            SnapshotFlowStepKind::UnlinkIntent,
            DEADLIST_ENTRIES_AFTER_THE_UNLINK_INTENT,
        ),
        (
            SnapshotFlowStepKind::UnlinkFinalBatch,
            UNITS_OF_THE_FIRST_FILE_KEPT_FOR_THE_SNAPSHOT,
        ),
    ] {
        let output = recorded
            .outputs_of(kind)
            .first()
            .copied()
            .unwrap_or_else(|| panic!("{kind:?} 那一步"));
        assert_eq!(
            output.deadlist_entries.len(),
            deadlist_entries,
            "{kind:?}：unlink 换下的、被快照引用的单元写进 deadlist"
        );
        for entry in &output.deadlist_entries {
            assert_eq!(
                (entry.head, entry.epoch),
                (output.tree_identifiers.inode, snapshot_txg),
                "{kind:?}：deadlist 条目的头是活头 inode 树、纪元是 S.txg"
            );
            assert!(
                singlefs_core::records::birth_txg_of_a_mapping_key(&entry.mapping_key)
                    .is_some_and(|birth| birth < snapshot_txg),
                "{kind:?}：进 deadlist 的单元都生在 S 之前"
            );
        }
        assert_eq!(
            output.mapping_entries_kept_for_the_snapshots.len(),
            deadlist_entries,
            "{kind:?}：一条 deadlist 条目一条留存的映射条目"
        );
        assert_eq!(
            common_snapshot_prefix::live_snapshot_trees_of(output),
            vec![recorded.snapshot()],
            "{kind:?}：快照仍在"
        );
    }
    assert!(
        recorded
            .outputs_of(SnapshotFlowStepKind::UnlinkFinalBatch)
            .first()
            .is_some_and(|output| output.deleted_inodes.is_empty()),
        "最后一批之后删除队列空"
    );
}

/// 快档那一趟：单元段不展开，每个状态交两个观察者。
fn enumerate_the_quick_tier(
    prepared: &PreparedSnapshotCrashFlow,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_snapshot_of_the_landed_version(versions_of_the_flow, image, report, counts);
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_under_a_snapshot(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let item_name = format!("unlink_under_a_snapshot::quick_tier::{item_label}");
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
        &[snapshot_observer_of_every_state(prepared, &observe)],
        &[mount_continuation_observer_under_a_snapshot(
            prepared,
            &observe_the_mount,
        )],
    )
    .judged()
}

/// 平时跑的那一份：两个单元段不展开，其余每段任意真子集；每个状态核活快照读出建它那一刻的内容，再可写挂载一次核续做走到空、快照与 deadlist 留着。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_an_unlink_under_a_snapshot_keeps_the_units_of_the_snapshot_on_the_deadlist_and_readable(
) {
    let prepared = prepare("unlink-under-a-snapshot-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_UNLINK_UNDER_A_SNAPSHOT_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
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
    assert_every_state_was_observed(&tally);
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_unlink_under_a_snapshot_quick_tier_counts_the_same_when_rerun_from_the_library() {
    let prepared = prepare("unlink-under-a-snapshot-crash-point-replay-rerun");
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

/// 全量：每段都展开，每个状态两遍恢复 + checker + 读快照，并在工作线程上可写挂载一次核续做。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_UNLINK_UNDER_A_SNAPSHOT` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 3.5 亿个状态、每个两遍恢复 + checker + 读快照 + 一次可写挂载；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_unlink_under_a_snapshot_stream_is_exhaustive_and_clean() {
    let prepared = prepare("unlink-under-a-snapshot-crash-point-replay-full");
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
        observe_the_snapshot_of_the_landed_version(versions_of_the_flow, image, report, counts);
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_under_a_snapshot(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "unlink_under_a_snapshot::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[snapshot_observer_of_every_state(&prepared, &observe)],
        &[mount_continuation_observer_under_a_snapshot(
            &prepared,
            &observe_the_mount,
        )],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_UNLINK_UNDER_A_SNAPSHOT states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
    assert_every_state_was_observed(&tally);
}

/// 录制器那一份的崩溃点：照公共模块 `common_crash_points` 设（`crash_points_of_a_flow_after_the_first_file`）：标准前缀与别的流共享，
/// 建快照与 unlink 两步在四条快照流与生命周期闭环流之间共享。
#[cfg(feature = "verdict-store")]
fn crash_points_of_the_flow(
    prepared: &PreparedSnapshotCrashFlow,
) -> Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan> {
    common_crash_points::crash_points_of_a_flow_after_the_first_file(
        &prepared.recorded.pool,
        &prepared.recorded.crash_point_entries_of_the_steps(),
        &prepared.writes,
        &prepared.segments,
        &prepared.stream_indexes,
    )
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；展开上限默认 6。两张登记表上的观察者照表跑：标准前缀节点上的可写挂载续做，
/// 快照那几步节点上的读快照与可写挂载续做（`common_snapshot_crash_point_observers::amplify_a_snapshot_flow`）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_unlink_under_a_snapshot_stream_is_clean() {
    let prepared = prepare("crash-amplification-unlink-under-a-snapshot");
    let crash_points = crash_points_of_the_flow(&prepared);
    common_snapshot_crash_point_observers::amplify_a_snapshot_flow(
        "unlink-under-a-snapshot",
        &prepared,
        crash_points,
        6,
    );
}
