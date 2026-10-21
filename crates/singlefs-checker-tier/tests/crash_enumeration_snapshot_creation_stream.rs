//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 只读快照的建快照流（里程碑四一·4 F1，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S6）：
//! 「取号 → 暖机 × 2 → A（新池新建文件，一个数据单元）→ S（建快照：独立发布，只重写树表、中央映射树、记账树与分配记录树五个节点，
//! 码 14、15 两条条目追加进树表，活头 inode 条目的 `previous_snapshot_txg` 换成 S.txg）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。
//! 每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；另逐状态核一样（快档与全量都核）：恢复落到 S 那一版时，它的活快照从那一版的树表读得出、
//! 读出的是建它那一刻（A）的内容（`read_the_first_file_of_a_snapshot`，判决 S1「读快照指着的根按根指针头部的出生树比」）；落到 A 与 A 之前的没有快照。
//! 流的搭建住在公共模块 `crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs`（三条快照流共用一份录法）。
//!
//! **比已有的流多罩了什么**：已有的崩溃点重放流上没有建快照那次发布；这里 S 的单元段（8 个单元两盘各一份、16 写）、记录段、根槽写与之后的
//! 系统配置槽轮换第一次被枚举到，池级 checker 第一次在带码 14、15 条目的树表上逐状态判（I-9.16、I-9.25、I-9.27、I-3.7）。前缀（取号、暖机、A）
//! 与新池新建文件流逐字节相同。多跑的一步只有逐状态读快照（这条流上没有重开、挂载：S 之后没有可续做的东西）。
//!
//! 快档只展开写数小于 10 的段（A、S 两个单元段 24 写、16 写不展开，只以整段持久进入后面的状态）；全量标 ignored（崩溃点重放的枚举域 16842815 个状态，
//! 两个单元段占 16842750；每次写只取两态时的闭式 16842785），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_crash_point_observers;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;

use common_snapshot_crash_point_observers::{
    observe_the_snapshot_of_the_landed_version, snapshot_observer_of_every_state,
};
use common_snapshot_prefix::{
    prepare_the_snapshot_crash_flow, PreparedSnapshotCrashFlow, SnapshotCrashFlow,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_creation_stream.rs";

/// 段形：取号两轮各 2、暖机两次（记录 2、根 1、轮换 2）、A 的 24 个单元写；A 的记录 2、根 1、轮换 2；
/// S 的 16 个单元写（树表、中央映射树、记账树、分配记录树五个节点，两盘各一份）、记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 16] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2];

fn prepare(tag: &str) -> PreparedSnapshotCrashFlow {
    prepare_the_snapshot_crash_flow(tag, SnapshotCrashFlow::SnapshotCreation)
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

/// 每个状态都看过，恢复落到的每一版都有状态，落到 S 的每个状态都读了它的快照（一个）。
fn assert_every_state_was_observed(tally: &CrashPointReplayTally) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    let landed = [
        "landed_before_the_first_file",
        "landed_on_first_file",
        "landed_on_create_snapshot",
    ];
    assert_eq!(
        landed
            .iter()
            .map(|name| counts.count_named(name))
            .sum::<u64>(),
        tally.states,
        "每个状态都落到 A 之前、A 或 S 之一：{counts:?}"
    );
    for name in landed {
        assert!(
            counts.count_named(name) > 0,
            "{name} 的状态真跑到了：{counts:?}"
        );
    }
    assert_eq!(
        counts.count_named("live_snapshots_read_at_their_creation_content"),
        counts.count_named("landed_on_create_snapshot"),
        "落到 S 的每个状态都读了它那一个快照"
    );
}

/// 快档那一趟：两个单元段不展开，每个状态交 [`observe_the_snapshot_of_the_landed_version`]。
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
    let item_name = format!("snapshot_creation::quick_tier::{item_label}");
    enumerate_crash_states_with_expansion(
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
    )
    .judged()
}

/// 平时跑的那一份：A、S 两个单元段不展开，其余每段任意真子集；每个状态核恢复落到的那一版的活快照读出建它那一刻的内容。
/// 判器（oracle、池级 checker、记录核对器）先判，段形与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_the_snapshot_creation_recovers_to_a_version_whose_live_snapshot_reads_its_creation_content(
) {
    let prepared = prepare("snapshot-creation-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_SNAPSHOT_CREATION_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "取号、暖机两次、A 同新池新建文件流；S 16 个单元写、记录 2、根 1、轮换 2"
    );
    assert_eq!(
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        1 + 6 * 3 + 4 * 3 + 4,
        "每次写只取两态时展开的段按闭式数：1 + 六个系统配置槽 2 写段（取号两轮、四次轮换）各 3 + 四个记录 2 写段各 3 + 四个根槽 1 写段各 1"
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
        1 + 6 * (3 * 3 - 1) + 4 * 3 + 4,
        "1 + 六个系统配置槽 2 写段各 3² − 1 + 四个记录 2 写段各 3 + 四个根槽 1 写段各 1"
    );
    assert_every_state_was_observed(&tally);
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_snapshot_creation_stream_quick_tier_counts_the_same_when_rerun_from_the_library() {
    let prepared = prepare("snapshot-creation-crash-point-replay-rerun");
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

/// 全量：16 段。A、S 两个单元段各 2^24 − 1、2^16 − 1，每个状态两遍恢复 + checker + 读快照。登记成崩溃枚举用例，checker-tier-crash-point-replay
/// 认下面打印的 `CRASH_POINT_REPLAY_SNAPSHOT_CREATION` 行里 `exhaustive=true`；带断点续跑、可双机分片（同多记录流）。
#[test]
#[ignore = "全量 16842815 个状态（一千六百多万）、每个两遍恢复 + checker + 读快照；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_snapshot_creation_stream_is_exhaustive_and_clean() {
    let prepared = prepare("snapshot-creation-crash-point-replay-full");
    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form_with_two_states_per_write,
        1 + 6 * 3 + 4 * 3 + 4 + ((1 << 24) - 1) + ((1 << 16) - 1),
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，十六段"
    );
    assert_eq!(closed_form_with_two_states_per_write, 16_842_785);
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 6 * (3 * 3 - 1) + 4 * 3 + 4 + ((1 << 24) - 1) + ((1 << 16) - 1),
        "枚举域的闭式：六个系统配置槽 2 写段（取号两轮、四次轮换）各 3² − 1、四个记录 2 写段各 3、四个根槽 1 写段、A / S 的单元段各 2^n − 1"
    );
    assert_eq!(closed_form, 16_842_815);
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_snapshot_of_the_landed_version(versions_of_the_flow, image, report, counts);
    };
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "snapshot_creation::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[snapshot_observer_of_every_state(&prepared, &observe)],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_SNAPSHOT_CREATION states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
    assert_eq!(prepared.segment_sizes(), SEGMENT_SIZES);
    assert_every_state_was_observed(&tally);
}

/// 录制器那一份的崩溃点（节点按 `<流程>.<周期>.<步>` 编码，`crash_amplification::CrashPointRecorder`）：取号加暖机、新池新建文件与建快照那一步
/// 都照公共模块 `common_crash_points` 设（`crash_points_of_a_flow_after_the_first_file`）：标准前缀与别的流共享，建快照那一步在四条快照流与生命周期
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
/// 两张登记表上的观察者照表跑：标准前缀节点上的可写挂载续做，建快照节点上的读快照与可写挂载续做
/// （`common_snapshot_crash_point_observers::amplify_a_snapshot_flow`）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_snapshot_creation_stream_is_clean() {
    let prepared = prepare("crash-amplification-snapshot-creation");
    let crash_points = crash_points_of_the_flow(&prepared);
    common_snapshot_crash_point_observers::amplify_a_snapshot_flow(
        "snapshot-creation",
        &prepared,
        crash_points,
        6,
    );
}
