//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 快照下覆盖写与删文件流（里程碑四一·4 F2，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S6）：
//! 「取号 → 暖机 × 2 → A（一个数据单元）→ S（建快照）→ W（覆盖写：换下的数据单元、extent 根、inode 叶容器、inode 根都生在 S 之前，
//! birth ≤ 活头的 `previous_snapshot_txg` = S.txg，进 deadlist、不释放、映射条目留存）→ unlink 意图 → 最后一批（批大小 1）」
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle（A、S、W 有文件，意图那一版起没有）、池级 checker、
//! 记录核对器；另逐状态核两样（快档与全量都核）：
//! - 恢复落到的那一版的活快照在崩溃镜像上读得出、读出建它那一刻（A）的内容：W 之后它的四个单元只被快照与留存的映射条目引用；
//! - 可写挂载一次：续做把 unlink 走到空（落到意图那一版的续做一步），快照、deadlist 与留存的映射条目原样留着（与录流时最后一批那一版逐项相同）；
//!   挂完的盘在判段过池级 checker（一条不变量都不许判违例）。
//!
//! 流的搭建住在公共模块 `crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs`（三条快照流共用一份录法）。
//!
//! **比已有的流多罩了什么**：已有的流上没有快照，覆盖写与 unlink 换下的单元全进 defer；这里 W 那次发布第一次把被快照引用的块写进 deadlist
//! （deadlist 树从零根长出第一片节点、中央映射树多四条留存条目，W 的单元段 13 个单元 26 写），崩在 W 的单元、记录、根槽写之间的每个状态
//! 第一次被枚举到；unlink 的意图与最后一批第一次在带快照、带非空 deadlist 的版本上走（换下的单元生在 S 之后，照旧进 defer；deadlist 与留存条目
//! 随每一版带着）。多跑的两步是逐状态读快照与每个状态上的可写挂载（取号、写行、暖机、续做）、挂完的盘上那一遍池级 checker。
//! 第一个文件只有一个数据单元（判决 S6），W 把它被快照引用的四个单元全换掉，unlink 换下的都生在 S 之后：「unlink 把被快照引用的块放进 deadlist」
//! 这一条路这条流走不到，要另起一条删文件直接接在建快照之后的流才走得到。
//!
//! 快档只展开写数小于 10 的段（五个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored（崩溃点重放的枚举域 167837792 个状态，
//! 五个单元段占 167837691；每次写只取两态时的闭式 167837747），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

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
    UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
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
    "crates/singlefs-checker-tier/tests/crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs";

/// 段形：前十六段同建快照流（取号两轮各一段）；W 26 个单元写（数据单元、extent 根、inode 叶容器与根、分配记录树五个节点、记账树、deadlist 树根、中央映射树、
/// 树表，两盘各一份）、记录 2、根 1、轮换 2；意图 26 个单元写、最后一批 24 个单元写，各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 28] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2, 26, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2,
];

/// 恢复落到的那一版是哪一种（计数名），每一种都要有状态。
const LANDED_COUNTERS: [&str; 6] = [
    "landed_before_the_first_file",
    "landed_on_first_file",
    "landed_on_create_snapshot",
    "landed_on_overwrite_under_the_snapshot",
    "landed_on_unlink_intent",
    "landed_on_unlink_final_batch",
];

fn prepare(tag: &str) -> PreparedSnapshotCrashFlow {
    prepare_the_snapshot_crash_flow(tag, SnapshotCrashFlow::OverwriteAndUnlinkUnderASnapshot)
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
/// 落到意图那一版的续做一步、别的不续做，两种都跑到了；挂完的盘都过了池级 checker。
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

/// 录流的形状（判器判完之后再核）：W 把四个被快照引用、生在 S 之前的单元写进 deadlist（头 = 活头 inode 树、纪元 = S.txg，D5（快照 / 空间记账机制）
/// 已定项 12），映射条目留存四条；意图与最后一批原样带着它们，快照仍在、没有销毁中的。
fn assert_the_shape_of_the_flow(prepared: &PreparedSnapshotCrashFlow) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前十六段同建快照流；W 26、意图 26、最后一批 24 个单元写，各自记录 2、根 1、轮换 2"
    );
    let recorded = &prepared.recorded;
    let snapshot_txg = recorded
        .outputs_of(SnapshotFlowStepKind::CreateSnapshot)
        .first()
        .expect("建快照那一步")
        .root
        .checkpoint_txg;
    let overwritten = recorded
        .outputs_of(SnapshotFlowStepKind::OverwriteUnderTheSnapshot)
        .first()
        .copied()
        .expect("覆盖写那一步");
    assert_eq!(
        overwritten.deadlist_entries.len(),
        UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
        "W 把数据单元、extent 根、inode 叶容器、inode 根四个被快照引用的单元写进 deadlist"
    );
    for entry in &overwritten.deadlist_entries {
        assert_eq!(
            (entry.head, entry.epoch),
            (overwritten.tree_identifiers.inode, snapshot_txg),
            "deadlist 条目的头是活头 inode 树、纪元是 S.txg"
        );
        assert!(
            singlefs_core::records::birth_txg_of_a_mapping_key(&entry.mapping_key)
                .is_some_and(|birth| birth < snapshot_txg),
            "进 deadlist 的单元都生在 S 之前（birth ≤ S.txg 的那一侧）"
        );
    }
    assert_eq!(
        overwritten.mapping_entries_kept_for_the_snapshots.len(),
        UNITS_THE_OVERWRITE_KEEPS_FOR_THE_SNAPSHOT,
        "一条 deadlist 条目一条留存的映射条目"
    );
    for kind in [
        SnapshotFlowStepKind::UnlinkIntent,
        SnapshotFlowStepKind::UnlinkFinalBatch,
    ] {
        for output in recorded.outputs_of(kind) {
            assert_eq!(
                output.deadlist_entries, overwritten.deadlist_entries,
                "{kind:?}：deadlist 原样带着（unlink 换下的单元生在 S 之后，照旧进 defer）"
            );
            assert_eq!(
                common_snapshot_prefix::live_snapshot_trees_of(output),
                vec![recorded.snapshot()],
                "{kind:?}：快照仍在"
            );
        }
    }
}

/// 快档那一趟：五个单元段不展开，每个状态交两个观察者。
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
    let item_name = format!("overwrite_and_unlink_under_a_snapshot::quick_tier::{item_label}");
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

/// 平时跑的那一份：五个单元段不展开，其余每段任意真子集；每个状态核活快照读出建它那一刻的内容，再可写挂载一次核续做走到空、快照与 deadlist 留着。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_an_overwrite_and_unlink_under_a_snapshot_keeps_the_snapshot_readable_and_the_mount_continues_to_empty(
) {
    let prepared = prepare("overwrite-and-unlink-under-a-snapshot-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_OVERWRITE_AND_UNLINK_UNDER_A_SNAPSHOT_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_the_shape_of_the_flow(&prepared);
    assert_eq!(
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        1 + 9 * 3 + 7 * 3 + 7,
        "每次写只取两态时展开的段按闭式数：1 + 九个系统配置槽 2 写段（取号两轮、七次轮换）各 3 + 七个记录 2 写段各 3 + 七个根槽 1 写段各 1"
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
    assert_eq!(
        tally.states,
        1 + 9 * (3 * 3 - 1) + 7 * 3 + 7,
        "1 + 九个系统配置槽 2 写段各 3² − 1 + 七个记录 2 写段各 3 + 七个根槽 1 写段各 1"
    );
    assert_every_state_was_observed(&tally);
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_overwrite_and_unlink_under_a_snapshot_quick_tier_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("overwrite-and-unlink-under-a-snapshot-crash-point-replay-rerun");
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

/// 全量：28 段。五个单元段各 2^n − 1（n = 24、16、26、26、24：A、S、W、意图、最后一批），每个状态两遍恢复 + checker + 读快照，
/// 并在工作线程上可写挂载一次核续做。登记成崩溃枚举用例，checker-tier-crash-point-replay 认下面打印的
/// `CRASH_POINT_REPLAY_OVERWRITE_AND_UNLINK_UNDER_A_SNAPSHOT` 行里 `exhaustive=true`；带断点续跑、可双机分片（同多记录流）。
#[test]
#[ignore = "全量 167837792 个状态（一亿六千多万）、每个两遍恢复 + checker + 读快照 + 一次可写挂载；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_overwrite_and_unlink_under_a_snapshot_stream_is_exhaustive_and_clean() {
    let prepared = prepare("overwrite-and-unlink-under-a-snapshot-crash-point-replay-full");
    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form_with_two_states_per_write,
        1 + 9 * 3
            + 7 * 3
            + 7
            + ((1 << 24) - 1)
            + ((1 << 16) - 1)
            + ((1 << 26) - 1)
            + ((1 << 26) - 1)
            + ((1 << 24) - 1),
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，二十八段"
    );
    assert_eq!(closed_form_with_two_states_per_write, 167_837_747);
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 9 * (3 * 3 - 1)
            + 7 * 3
            + 7
            + ((1 << 24) - 1)
            + ((1 << 16) - 1)
            + ((1 << 26) - 1)
            + ((1 << 26) - 1)
            + ((1 << 24) - 1),
        "枚举域的闭式：九个系统配置槽 2 写段（取号两轮、七次轮换）各 3² − 1、七个记录 2 写段各 3、七个根槽 1 写段、五个单元段各 2^n − 1"
    );
    assert_eq!(closed_form, 167_837_792);
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
            name: "overwrite_and_unlink_under_a_snapshot::full",
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
        "CRASH_POINT_REPLAY_OVERWRITE_AND_UNLINK_UNDER_A_SNAPSHOT states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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

/// 录制器那一份的崩溃点（节点按 `<流程>.<周期>.<步>` 编码，`crash_amplification::CrashPointRecorder`）：取号加暖机、新池新建文件与之后的每一步
/// 都照公共模块 `common_crash_points` 设（`crash_points_of_a_flow_after_the_first_file`）：标准前缀与别的流共享，快照那几步在四条快照流与生命周期
/// 闭环流之间共享。流已经录完，按每一步记下的录制流区间补记（记录器只认区间首尾相接、罩住整条流）。
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

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判、CPU 判器同时对照。展开上限默认 6。
/// 版本表带「没有文件的那几版」（意图那一版起）。两张登记表上的观察者照表跑：标准前缀节点上的可写挂载续做，快照那几步节点上的读快照与
/// 可写挂载续做（`common_snapshot_crash_point_observers::amplify_a_snapshot_flow`）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_overwrite_and_unlink_under_a_snapshot_stream_is_clean() {
    let prepared = prepare("crash-amplification-overwrite-and-unlink-under-a-snapshot");
    let crash_points = crash_points_of_the_flow(&prepared);
    common_snapshot_crash_point_observers::amplify_a_snapshot_flow(
        "overwrite-and-unlink-under-a-snapshot",
        &prepared,
        crash_points,
        6,
    );
}
