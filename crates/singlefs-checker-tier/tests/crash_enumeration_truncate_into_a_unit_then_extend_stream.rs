//! checker 档模块：crash、crash_state_enumeration
//! 缩到单元中间再变长流（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K9 第二条）：多记录流「取号 → 暖机 × 2 → A → B → C」之后
//! （C 是第一个文件三个数据单元）把第一个文件截到一个半净荷：截断意图（单元 1 重写成声明长度半个净荷，判决 K3；单元 2 在 size 之外挂着）
//! → 最后一批（批大小 1，放单元 2、删截断条目）→ 变长到三个半净荷（一次发布：单元 1 补零重写成一整个净荷，之后到 size 是洞，上段叶条目标签 3，判决 K5），
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle（A、B、C 与 C 之后的每一版有文件，各批的内容同意图那一版：
//! 意图那次已把 size 写成目标，判决 K2「崩溃后读者只读到截前或截后」）、池级 checker（截断那一批起 I-4.4 收截断条目、I-4.6 每条记录都判、
//! I-4.10 与 I-9.38 新立）、记录核对器；另逐状态记恢复落到哪一版（只记计数，`common_truncation_crash_streams::observe_the_landed_version`）。
//! 流的搭建与判法住在公共模块 `tests/common_truncation_crash_streams/mod.rs`（三条截断流共用一份录法）。
//!
//! **比已有的流多罩了什么**：截到单元中间那次的尾单元重写（声明长度半个净荷）、放一个单元的最后一批，与变长那一次（尾单元补零重写、上段叶条目换成标签 3）
//! 各自的单元段、记录段、根槽写与轮换第一次被枚举到；变长之后的状态上恢复读洞答 0、池级 checker 判标签 3 的单元数（I-4.6，判决第六节 S1）
//! 与 size 之内每个单元的声明长度（I-9.38）。前二十段与多记录流、删除流逐字节相同（共用 `common_crash_points` 的录法）。
//! 多跑的步：崩溃点重放那一路没有（每个状态两遍恢复、池级 checker、记录核对器，同多记录流；截断的续做走共用走空，判决 K1）；
//! 崩溃放量那一路在 C 之后截断意图、最后一批、变长那几步的每个点上另挂两个观察者（`common_truncation_crash_streams::OBSERVERS_ON_THE_TRUNCATION_STEPS`）：
//! 记落点，与每个状态上可写挂载一次、共用走空把截断走完、挂完的盘过池级 checker（判段判，配了 GPU 上卡）。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例，带断点续跑、可双机分片。系统配置槽写是原地覆写，取三态，其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;
mod common_truncation_crash_streams;

use common_truncation_crash_streams::{
    assert_clean, assert_every_state_was_observed, assert_the_shape_of_the_steps,
    expansion_of_the_segments_shorter_than_ten, landed_version_observer,
    observe_the_landed_version, prepare_the_truncation_crash_stream, PreparedTruncationCrashStream,
    TruncationCrashStream,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayTally,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_enumeration_truncate_into_a_unit_then_extend_stream.rs";

/// 这一条流。
const STREAM: TruncationCrashStream = TruncationCrashStream::IntoAUnitThenExtended;

/// 段形（录流按真写表数出来的）：前二十段同多记录流（取号两轮各一段）；截断意图 28 个单元写、最后一批 20、变长 26，各自记录 2、根 1、轮换 2
const SEGMENT_SIZES: [usize; 32] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 28, 2, 1, 2, 20, 2, 1, 2, 26, 2,
    1, 2,
];

/// 每次写只取两态时整条流的闭式（取号写一轮时 1_695_547_518，取号第二轮那一段 +3）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 1_695_547_521;

/// 快档展开的段每次写只取两态时的闭式（取号写一轮时 132）。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 135;

fn prepare(tag: &str) -> PreparedTruncationCrashStream {
    prepare_the_truncation_crash_stream(tag, STREAM)
}

/// 录流的形状（判器判完之后再核：改坏实现时先红在判器上）：段形，与每一步之后截断条目在不在、单元数、标签。
fn assert_the_shape_of_the_flow(prepared: &PreparedTruncationCrashStream) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前二十段同多记录流；截断意图 28 个单元写、最后一批 20、变长 26，各自记录 2、根 1、轮换 2"
    );
    assert_the_shape_of_the_steps(STREAM, &prepared.recorded);
}

/// 快档那一趟：写数不小于 10 的段不展开，每个状态交落点观察者。
fn enumerate_the_quick_tier(
    prepared: &PreparedTruncationCrashStream,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    let landed_versions = prepared.landed_versions.as_slice();
    let observe = |_ordinal: u64,
                   _image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version(landed_versions, report, counts);
    };
    let item_name = format!("truncate_into_a_unit_then_extend::quick_tier::{item_label}");
    enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &expansion_of_the_segments_shorter_than_ten,
        &[landed_version_observer(prepared, &observe)],
    )
    .judged()
}

/// 平时跑的那一份：写数不小于 10 的段不展开，其余每段任意真子集；每个状态记恢复落到哪一版。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_a_truncation_into_a_unit_then_an_extension_reads_the_size_before_or_after_and_stays_clean(
) {
    let prepared = prepare("truncate-into-a-unit-then-extend-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_TRUNCATE_INTO_A_UNIT_THEN_EXTEND_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
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
    assert_every_state_was_observed(&prepared, &tally);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "truncate_into_a_unit_then_extend",
        &tally,
    );
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_truncation_into_a_unit_then_extension_quick_tier_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("truncate-into-a-unit-then-extend-crash-point-replay-rerun");
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

/// 全量：每段都展开，每个状态两遍恢复 + checker + 记落点。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_TRUNCATE_INTO_A_UNIT_THEN_EXTEND` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 1695547521 个状态（十六亿多，两态闭式；崩溃点重放的枚举域以用例打出的为准）、每个两遍恢复 + checker；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_truncate_into_a_unit_then_extend_stream_is_exhaustive_and_clean() {
    let prepared = prepare("truncate-into-a-unit-then-extend-crash-point-replay-full");
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
    let landed_versions = prepared.landed_versions.as_slice();
    let observe = |_ordinal: u64,
                   _image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version(landed_versions, report, counts);
    };
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "truncate_into_a_unit_then_extend::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &full_expansion,
        &[landed_version_observer(&prepared, &observe)],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_TRUNCATE_INTO_A_UNIT_THEN_EXTEND states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
    assert_every_state_was_observed(&prepared, &tally);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "truncate_into_a_unit_then_extend",
        &tally,
    );
}

/// 两份造法写表相同：崩溃点重放那一份（[`prepare`]，不录崩溃点）与录制器那一份（C 之后每一步设一个点）录下的起点镜像、整张写表、段、
/// 被判的根槽写与版本表逐项相同——崩溃放量与崩溃点重放判的是同一条流。
#[cfg(feature = "verdict-store")]
#[test]
fn the_truncate_into_a_unit_then_extend_stream_writes_the_same_table_through_the_crash_point_recorder_as_without_it(
) {
    let prepared = prepare("truncate-into-a-unit-then-extend-write-table-without-the-recorder");
    let recorded = common_truncation_crash_streams::record_through_the_crash_point_recorder(
        "truncate-into-a-unit-then-extend-write-table-through-the-recorder",
        STREAM,
    );
    assert!(recorded.base == prepared.base, "起点镜像逐字节相同");
    assert!(recorded.writes == prepared.writes, "写表逐项相同");
    assert_eq!(recorded.segments, prepared.segments, "段一样切");
    assert_eq!(
        recorded.judged_root_index, prepared.judged_root_index,
        "被判的根槽写相同"
    );
    assert_eq!(recorded.versions, prepared.versions, "版本表相同");
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；展开上限默认 6。挂两张登记表上的观察者（`common_truncation_crash_streams::amplify_a_truncation_flow`）：
/// 标准前缀节点上的可写挂载续做（与多记录流、删除流在 C 之前的块上共享），与 C 之后截断意图、最后一批、变长那几步节点上的落点、可写挂载续做截断两个。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_truncate_into_a_unit_then_extend_stream_is_clean() {
    let recorded = common_truncation_crash_streams::record_through_the_crash_point_recorder(
        "crash-amplification-truncate-into-a-unit-then-extend",
        STREAM,
    );
    common_truncation_crash_streams::amplify_a_truncation_flow(
        "truncate-into-a-unit-then-extend",
        recorded,
        6,
    );
}

/// 放量计划上截断那几步的观察者（`common_truncation_crash_streams::OBSERVERS_ON_THE_TRUNCATION_STEPS`）：计划里 C 之后截断意图、最后一批、变长那几步的每一个点
/// 都挂着落点与可写挂载续做截断两个观察者，别的点不挂；两个观察者各挂的点数等于另按这条流每一步的入口名数出来的点数，不是 0。只排计划、不判。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_plan_of_the_truncate_into_a_unit_then_extend_stream_hangs_both_truncation_observers_on_every_point_after_the_standard_prefix(
) {
    let recorded = common_truncation_crash_streams::record_through_the_crash_point_recorder(
        "crash-amplification-truncate-into-a-unit-then-extend-observer-nodes",
        STREAM,
    );
    let points = common_truncation_crash_streams::assert_the_truncation_observers_sit_on_every_planned_point_after_the_standard_prefix(
        "truncate-into-a-unit-then-extend",
        recorded,
        6,
    );
    println!("CRASH_AMPLIFICATION_TRUNCATION_OBSERVER_NODES flow=truncate-into-a-unit-then-extend points_with_both_observers={points}");
}
