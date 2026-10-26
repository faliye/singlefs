//! checker 档模块：crash、crash_state_enumeration
//! 快照下缩别的 inode 流（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第五节「别的 inode 的截断崩溃流」，截断判决
//! `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K9 第三条推到第一个文件之外的 inode）：`common::build_pool` 那一段（取号 → 暖机 × 2 → A）
//! 之后在第一个文件旁边建 inode 2、按 inode 号把它顺序写成三个数据单元，建快照 S，再把 inode 2 缩到半个净荷：截断意图（单元 0 重写，被快照引用、换下的
//! 那一份进 deadlist、映射条目留存，判决 K4）→ 一批（批大小 1，放单元 2）→ 最后一批（放单元 1、删条目；放掉的单元被快照引用，同样进 deadlist、不释放），
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle（第一个文件从 A 起不动，每一版都在版本表里）、池级 checker、
//! 记录核对器；另一个观察者逐状态记恢复落到哪一版，并在崩溃镜像上只读挂载、按号读 inode 2，读出的要等于落到的那一版里 inode 2 的内容
//! （`common_truncation_of_an_inode_other_than_the_first_crash_streams::observe_the_landed_version_and_the_truncated_inode`；判决第二节 S5「崩溃读回按
//! inode 给允许集合」）。录法在 `tests/common_truncation_of_an_inode_other_than_the_first_recording/mod.rs`（三条流共用一份）。
//!
//! **比已有的流多罩了什么**：已有的快照下缩流（`crash_enumeration_truncate_under_a_snapshot_stream.rs`）截的是第一个文件；这一条截的是第一个文件之外的
//! inode：快照下换下的是 inode 2 的单元与它那一串上段（判决第二节 S3「快照下被换下的上一版角色进 deadlist 那条链」）、截断条目点名 inode 2，
//! 这几次的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；崩在这几次中间的状态上 inode 2 只读到截前或截后，池级 checker 对 inode 2
//! 判 I-4.10、I-4.6、I-4.4。前缀与已有的流不同：建 inode 2、按号写它两次发布代替多记录流的 B、C（第一个文件那两次顺序写），基线镜像相同、
//! 写表与段序列不同，所以另开全量枚举。
//! 多跑的步：每个状态多一次只读挂载（按号读 inode 2）。崩溃放量那一路这一条没接（只在崩溃点重放那一路枚举）。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例。系统配置槽写是原地覆写，取三态，其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;
mod common_truncation_of_an_inode_other_than_the_first_crash_streams;
mod common_truncation_of_an_inode_other_than_the_first_recording;

use common_truncation_of_an_inode_other_than_the_first_crash_streams::{
    assert_clean, assert_every_state_was_observed, expansion_of_the_segments_shorter_than_ten,
    observe_the_landed_version_and_the_truncated_inode, observer_of_the_truncated_inode,
    prepare_the_truncation_of_an_inode_other_than_the_first,
    PreparedTruncationOfAnInodeOtherThanTheFirst,
};
use common_truncation_of_an_inode_other_than_the_first_recording::TruncationOfAnInodeOtherThanTheFirstCrashStream;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayTally,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_truncate_an_inode_other_than_the_first_under_a_snapshot_stream.rs";

/// 这一条流。
const STREAM: TruncationOfAnInodeOtherThanTheFirstCrashStream =
    TruncationOfAnInodeOtherThanTheFirstCrashStream::UnderASnapshot;

/// 段形（录流按真写表数出来的）：前十二段同新池新建文件那一段（取号两轮、暖机两次、A）；建 inode 2 那次 20 个单元写、记录 2、根 1、轮换 2；
/// 按号写 inode 2 那次 30、6、1、2；建快照 16、截断意图 30、一批 22、最后一批 20，各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 36] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 30, 6, 1, 2, 16, 2, 1, 2, 30, 2, 1, 2, 22, 2,
    1, 2, 20, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 2_170_617_979;

/// 快档展开的段每次写只取两态时的闭式。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 130;

fn prepare(tag: &str) -> PreparedTruncationOfAnInodeOtherThanTheFirst {
    prepare_the_truncation_of_an_inode_other_than_the_first(tag, STREAM)
}

/// 录流的形状（判器判完之后再核：改坏实现时先红在判器上）：段形，与每一步之后截断条目在不在、inode 2 的单元数、它那一条上段叶条目的标签。
fn assert_the_shape_of_the_flow(prepared: &PreparedTruncationOfAnInodeOtherThanTheFirst) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前十二段同新池新建文件那一段；建 inode 2、按号写它、建快照、截断意图、一批、最后一批各自的单元、记录、根、轮换"
    );
    assert_eq!(
        prepared.recorded.shape_of_the_steps(),
        vec![
            (false, 3, Some(1)),
            (true, 3, Some(1)),
            (true, 2, Some(1)),
            (false, 1, Some(2))
        ],
        "建快照：不动 inode 2；意图、一批、最后一批同缩到 0 那一条的形状"
    );
}

/// 快档那一趟：写数不小于 10 的段不展开，每个状态交观察者。
fn enumerate_the_quick_tier(
    prepared: &PreparedTruncationOfAnInodeOtherThanTheFirst,
    item_label: &str,
) -> Option<CrashPointReplayTally> {
    let versions = prepared.versions.as_slice();
    let landed_versions = prepared.landed_versions.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_and_the_truncated_inode(
            versions,
            landed_versions,
            image,
            report,
            counts,
        );
    };
    let item_name = format!(
        "truncate_an_inode_other_than_the_first_under_a_snapshot::quick_tier::{item_label}"
    );
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
        &[observer_of_the_truncated_inode(prepared, &observe)],
    )
    .judged()
}

/// 平时跑的那一份：写数不小于 10 的段不展开，其余每段任意真子集；每个状态记恢复落到哪一版、按号读 inode 2。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_a_truncation_of_another_inode_under_a_snapshot_reads_it_before_or_after_and_stays_clean(
) {
    let prepared =
        prepare("truncate-an-inode-other-than-the-first-under-a-snapshot-crash-point-replay-fast");
    let Some(tally) = enumerate_the_quick_tier(&prepared, "fast") else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_TRUNCATE_AN_INODE_OTHER_THAN_THE_FIRST_UNDER_A_SNAPSHOT_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
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
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_truncation_of_another_inode_under_a_snapshot_quick_tier_counts_the_same_when_rerun_from_the_library(
) {
    let prepared =
        prepare("truncate-an-inode-other-than-the-first-under-a-snapshot-crash-point-replay-rerun");
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

/// 全量：每段都展开，每个状态两遍恢复 + checker + 观察者。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_TRUNCATE_AN_INODE_OTHER_THAN_THE_FIRST_UNDER_A_SNAPSHOT` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 2170617979 个状态（二十一亿多，两态闭式；崩溃点重放的枚举域以用例打出的为准）、每个两遍恢复 + checker + 只读挂载读 inode 2；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_truncate_an_inode_other_than_the_first_under_a_snapshot_stream_is_exhaustive_and_clean(
) {
    let prepared =
        prepare("truncate-an-inode-other-than-the-first-under-a-snapshot-crash-point-replay-full");
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
    let versions = prepared.versions.as_slice();
    let landed_versions = prepared.landed_versions.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_and_the_truncated_inode(
            versions,
            landed_versions,
            image,
            report,
            counts,
        );
    };
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "truncate_an_inode_other_than_the_first_under_a_snapshot::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        &full_expansion,
        &[observer_of_the_truncated_inode(&prepared, &observe)],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_TRUNCATE_AN_INODE_OTHER_THAN_THE_FIRST_UNDER_A_SNAPSHOT states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
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
}
