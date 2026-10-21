//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 第一个文件之外还有文件带数据单元时的删除（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1–M3；M7 末句主 agent 定
//! 另开一条流，不并进里程碑三一·1 那三条）：多记录流「取号 → 暖机 × 2 → A → B → C」（C 是第一个文件三个数据单元）之后建 149 个空 inode、
//! 按 inode 号写 150 两个单元（上段长到层级 1），再按只供测试的批大小 1 分两支删（正推腿报告第 7 节「崩溃流」那一行的两条新流）：
//! - 先删别的文件：删 150（意图 → 留 1 的一批 → 最后一批：去掉它的上段条目、上段降回层级 0）→ 删第一个文件（意图 → 两批 → 最后一批：
//!   有条目的号集合变空，上段成层级 0 的零条目根兼叶）；
//! - 先删第一个文件：删第一个文件（别的文件还在：最后一批之后上段留在层级 1、第 0 片叶摘掉，这一版仍带文件）→ 删 150（最后一批之后集合变空，降成零条目根）。
//!
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑两遍恢复 + 多版本 oracle（第一个文件：删它的意图那一版起答没有文件）+ 池级 checker +
//! 记录核对器；另挂两个观察者（`common_multiple_copy_on_write_crash_streams` 文件头）：记恢复落到哪一版；在每个状态上可写挂载一次，续做把删除走完、
//! 发的次数等于落到的那一版之后录流里这一个号还剩的发布数，读回第一个文件与 150 都等于那一版续做走完之后的样子（正推第 6.2 节幂等判据 ③ 的取样点：
//! 崩在 150 的意图之后、批之间、最后一批之前；崩在删第一个文件的最后一批之前，上段要不要降层；崩在删最后一个带数据的号的最后一批之前，集合变空），
//! 落到删除中那几版的同一张崩溃镜像在另一份副本上再续做一次、挂完的盘逐字节相同（判据 ①b 的取样点）；挂完的盘过池级 checker（判段判，配了 GPU 上卡）。
//!
//! **比已有的流多罩了什么**：已有的删除流只删第一个文件、上段恒是第 0 片根兼叶；这两支第一次枚举删第一个文件之外的文件的意图、批与最后一批
//! （批截的是 150 自己的单元、不是第一个文件的），上段降层那一次发布（层级 1 → 0：上一版第 0 片叶照抄成根、旧根与第 1 片叶换下），删第一个文件时
//! 别的文件还在的最后一批（第 0 片叶摘掉、上段不降层），与删到集合变空的那一次；可写挂载续做从这些版本上接着走完。
//! 前二十段与多记录流、删除流逐字节相同；建 inode 与写 150 两步与多 inode 交错覆盖写那一条（`crash_enumeration_several_inodes_overwritten_in_turn_stream.rs`）
//! 共用同一份录法、同一个设点文件（用例 [`the_two_unlink_streams_share_their_nodes_through_the_write_of_inode_one_hundred_and_fifty_with_the_several_inodes_stream`] 钉住），
//! 崩溃放量那一路在库里共享。多跑的步：每个状态上观察者那一次可写挂载（续做）、一次只读挂载读回两个文件，落到删除中的再一次可写挂载。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，两支各登记一条崩溃枚举用例，可双机分片。系统配置槽写是原地覆写，取三态，其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;
mod common_multiple_copy_on_write_crash_streams;

use common_multiple_copy_on_write_crash_streams::steps::{
    assert_the_shape_of_the_steps, prepare_the_crash_stream, MultipleCopyOnWriteCrashStream,
    PreparedMultipleCopyOnWriteCrashStream,
};
use common_multiple_copy_on_write_crash_streams::{
    assert_clean, assert_every_state_was_observed, crash_state_count_under,
    digest_of_the_versions_of_the_stream, full_tier_expansion, observe_the_landed_version,
    observe_the_mount_continuation_and_the_files_read_back, quick_tier_expansion,
    segments_left_whole_text, version_table, FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW,
    LANDED_VERSION_OBSERVER_NAME, MOUNT_CONTINUATION_OBSERVER_NAME,
    QUICK_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW,
};
use singlefs_checker_tier::crash::{
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
};
use singlefs_checker_tier::crash_amplification::ObserverDeduplication;
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, CrashImage};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs";

/// 两支。
const STREAMS: [MultipleCopyOnWriteCrashStream; 2] = [
    MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile,
    MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile,
];

/// 段形（草稿探针录流数出来的，只用 harness 档与实现录同一份步骤）：前二十段同多记录流（取号两轮各一段）；建 inode 20、写 150 30；
/// 删 150：意图 26、留 1 的一批 24、最后一批 24（降层）；删第一个文件：意图 30、两批 28、26、最后一批 26（集合变空）。各自记录、根、轮换。
const SEGMENT_SIZES_DELETING_THE_OTHER_FILE_FIRST: [usize; 56] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 20, 2, 1, 2, 30, 4, 1, 2, 26, 2,
    1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 30, 2, 1, 2, 28, 2, 1, 2, 26, 2, 1, 2, 26, 2, 1, 2,
];

/// 段形：前二十八段同上一支；删第一个文件：意图 26、两批 26、28、最后一批 26（第 0 片叶摘掉）；删 150：意图 30、留 1 的一批 28、最后一批 26（集合变空）。
const SEGMENT_SIZES_DELETING_THE_FIRST_FILE_FIRST: [usize; 56] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 20, 2, 1, 2, 30, 4, 1, 2, 26, 2,
    1, 2, 26, 2, 1, 2, 28, 2, 1, 2, 26, 2, 1, 2, 30, 2, 1, 2, 28, 2, 1, 2, 26, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（两支各一个；取号写一轮时 4_010_803_374 与 4_312_793_262，取号第二轮那一段各 +3）。
const CLOSED_FORM_DELETING_THE_OTHER_FILE_FIRST_WITH_TWO_STATES_PER_WRITE: u64 = 4_010_803_377;
const CLOSED_FORM_DELETING_THE_FIRST_FILE_FIRST_WITH_TWO_STATES_PER_WRITE: u64 = 4_312_793_265;

/// 快档展开的段每次写只取两态时的闭式（两支相同：段数相同，长段只差在写数上；取号写一轮时 186）。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 189;

/// 一支的段形与全量闭式。
fn segment_sizes_and_closed_form(
    stream: MultipleCopyOnWriteCrashStream,
) -> (&'static [usize], u64) {
    match stream {
        MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile => (
            &SEGMENT_SIZES_DELETING_THE_OTHER_FILE_FIRST,
            CLOSED_FORM_DELETING_THE_OTHER_FILE_FIRST_WITH_TWO_STATES_PER_WRITE,
        ),
        MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile => (
            &SEGMENT_SIZES_DELETING_THE_FIRST_FILE_FIRST,
            CLOSED_FORM_DELETING_THE_FIRST_FILE_FIRST_WITH_TWO_STATES_PER_WRITE,
        ),
        MultipleCopyOnWriteCrashStream::OneFileOverwrittenAcrossTwoWritableRemounts
        | MultipleCopyOnWriteCrashStream::SeveralInodesOverwrittenInTurn
        | MultipleCopyOnWriteCrashStream::LargeFileOverwrittenAcrossLowerLeaves => {
            panic!("{stream:?} 不是删除那两支，它住在自己的测试文件里")
        }
    }
}

/// 全量那条计数行的前缀（一支一个，各登记一条 `count-line=`）。
fn full_enumeration_count_line_prefix(stream: MultipleCopyOnWriteCrashStream) -> &'static str {
    match stream {
        MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile => {
            "CRASH_POINT_REPLAY_OTHER_FILE_DELETED_BEFORE_THE_FIRST_FILE"
        }
        MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile => {
            "CRASH_POINT_REPLAY_FIRST_FILE_DELETED_BEFORE_THE_OTHER_FILE"
        }
        MultipleCopyOnWriteCrashStream::OneFileOverwrittenAcrossTwoWritableRemounts
        | MultipleCopyOnWriteCrashStream::SeveralInodesOverwrittenInTurn
        | MultipleCopyOnWriteCrashStream::LargeFileOverwrittenAcrossLowerLeaves => {
            panic!("{stream:?} 不是删除那两支，它住在自己的测试文件里")
        }
    }
}

/// 录流的形状（判器判完之后再核：改坏实现时先红在判器上）：段形，与每一步之后的上段、有条目的号、两个文件的单元数、删除队列。
fn assert_the_shape_of_the_flow(prepared: &PreparedMultipleCopyOnWriteCrashStream) {
    let stream = prepared.recorded.stream;
    let (segment_sizes, _closed_form) = segment_sizes_and_closed_form(stream);
    assert_eq!(
        prepared.segment_sizes(),
        segment_sizes,
        "{stream:?}：前二十段同多记录流；之后建 inode、写 150、两次删除各自的意图、批与最后一批"
    );
    assert_the_shape_of_the_steps(&prepared.recorded);
}

/// 按 `expansion` 枚举一支，挂两个观察者（落点、可写挂载续做读回每个文件）；只录那一趟交回 `None`。
fn enumerate_the_stream(
    prepared: &PreparedMultipleCopyOnWriteCrashStream,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let stream = prepared.recorded.stream;
    let versions = prepared.versions_of_the_stream.as_slice();
    let inodes_read_back = stream.inodes_read_back();
    let observe_the_landing =
        |_ordinal: u64,
         _image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_landed_version(versions, report, counts);
        };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_and_the_files_read_back(
                versions,
                &inodes_read_back,
                image,
                report,
                counts,
            )
        };
    let digest = digest_of_the_versions_of_the_stream(versions);
    let item_name = format!("{}::{item_label}", stream.flow_name());
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
        expansion,
        &[CrashPointReplayObserver {
            name: LANDED_VERSION_OBSERVER_NAME,
            extra_inputs_digest: digest,
            observe: &observe_the_landing,
        }],
        &[CrashPointReplayObserverHandingBackAnImage {
            name: MOUNT_CONTINUATION_OBSERVER_NAME,
            extra_inputs_digest: digest,
            observe: &observe_the_mount,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

/// 快档一支：写数不小于 10 的段不展开，其余每段任意真子集；判器先判，录流的形状与计数在它之后核。
fn quick_tier_of(stream: MultipleCopyOnWriteCrashStream, tag: &str) {
    let prepared = prepare_the_crash_stream(tag, stream);
    let Some(tally) = enumerate_the_stream(&prepared, "quick_tier::fast", &quick_tier_expansion)
    else {
        return;
    };
    println!(
        "{}_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        full_enumeration_count_line_prefix(stream),
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(stream, &tally);
    assert_the_shape_of_the_flow(&prepared);
    assert_eq!(
        closed_form_state_count(
            &prepared.segments_shorter_than(QUICK_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW)
        ),
        QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "{stream:?}：每次写只取两态时展开的段按闭式数"
    );
    assert_eq!(
        tally.states,
        crash_state_count_under(&prepared, &quick_tier_expansion),
        "{stream:?}：展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_every_state_was_observed(&prepared, &tally);
}

/// 先删别的文件那一支的快档：每个状态记落点、可写挂载一次续做走完删除、读回第一个文件与 150，落到删除中的再续做一遍逐字节相同。
#[test]
fn every_crash_state_of_deleting_the_other_file_before_the_first_file_continues_the_deletion_to_the_same_files_and_stays_clean(
) {
    quick_tier_of(
        MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile,
        "unlink-other-file-first-crash-point-replay-fast",
    );
}

/// 先删第一个文件那一支的快档（判法同上一条）。
#[test]
fn every_crash_state_of_deleting_the_first_file_before_the_other_file_continues_the_deletion_to_the_same_files_and_stays_clean(
) {
    quick_tier_of(
        MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile,
        "unlink-first-file-first-crash-point-replay-fast",
    );
}

/// 复用不改判定：先删别的文件那一支的快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_quick_tier_of_deleting_the_other_file_before_the_first_file_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare_the_crash_stream(
        "unlink-other-file-first-crash-point-replay-rerun",
        MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile,
    );
    let Some(run_judging_the_blocks) =
        enumerate_the_stream(&prepared, "quick_tier::rerun", &quick_tier_expansion)
    else {
        return;
    };
    let Some(run_reusing_the_library) =
        enumerate_the_stream(&prepared, "quick_tier::rerun", &quick_tier_expansion)
    else {
        return;
    };
    assert_eq!(
        run_reusing_the_library, run_judging_the_blocks,
        "第二遍全照库里复用，计数逐项相同"
    );
    assert_eq!(
        run_judging_the_blocks.observed_states, run_judging_the_blocks.states,
        "每个状态都看过"
    );
}

/// 全量一支：每段都展开，每个状态两遍恢复 + checker + 两个观察者；打这一支的计数行（`exhaustive=true` 要状态数等于闭式）。
fn full_enumeration_of(stream: MultipleCopyOnWriteCrashStream, tag: &str) {
    let prepared = prepare_the_crash_stream(tag, stream);
    let (_segment_sizes, closed_form_with_two_states_per_write) =
        segment_sizes_and_closed_form(stream);
    assert_eq!(
        closed_form_state_count(
            &prepared.segments_shorter_than(FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW)
        ),
        closed_form_with_two_states_per_write,
        "{stream:?}：每次写只取两态时的闭式：1 + Σ(2^|段| − 1)"
    );
    let segments_left_whole = segments_left_whole_text(&prepared);
    assert_eq!(
        segments_left_whole, "-",
        "{stream:?}：每一段都装得下全量展开"
    );
    let closed_form = crash_state_count_under(&prepared, &full_tier_expansion);
    let Some(tally) = enumerate_the_stream(&prepared, "full", &full_tier_expansion) else {
        return;
    };
    println!(
        "{} states={} closed_form={closed_form} exhaustive={} segments_left_whole={segments_left_whole} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        full_enumeration_count_line_prefix(stream),
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(stream, &tally);
    assert_eq!(tally.states, closed_form);
    assert_the_shape_of_the_flow(&prepared);
    assert_every_state_was_observed(&prepared, &tally);
}

/// 全量：先删别的文件那一支。登记成崩溃枚举用例，checker-tier-crash-point-replay 认 `CRASH_POINT_REPLAY_OTHER_FILE_DELETED_BEFORE_THE_FIRST_FILE` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 40 亿个状态（两态闭式 4010803377；原地覆写取三态的枚举域以用例打出的为准）、每个两遍恢复 + checker + 一次可写挂载（按恢复镜像去重，落到删除中的两次）；门禁 checker-tier-crash-point-replay --full 在 release 下跑，可双机分片"]
fn full_enumeration_of_deleting_the_other_file_before_the_first_file_is_exhaustive_and_clean() {
    full_enumeration_of(
        MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile,
        "unlink-other-file-first-crash-point-replay-full",
    );
}

/// 全量：先删第一个文件那一支。登记成崩溃枚举用例，checker-tier-crash-point-replay 认 `CRASH_POINT_REPLAY_FIRST_FILE_DELETED_BEFORE_THE_OTHER_FILE` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 43 亿个状态（两态闭式 4312793265；原地覆写取三态的枚举域以用例打出的为准）、每个两遍恢复 + checker + 一次可写挂载（按恢复镜像去重，落到删除中的两次）；门禁 checker-tier-crash-point-replay --full 在 release 下跑，可双机分片"]
fn full_enumeration_of_deleting_the_first_file_before_the_other_file_is_exhaustive_and_clean() {
    full_enumeration_of(
        MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile,
        "unlink-first-file-first-crash-point-replay-full",
    );
}

/// 两份造法写表相同（两支各核一遍）：崩溃点重放那一份（不录崩溃点）与录制器那一份（C 之后每一步设一个点）录下的起点镜像、整张写表、段、
/// 被判的根槽写、两样版本表与流里每一版逐项相同——崩溃放量与崩溃点重放判的是同一条流。
#[cfg(feature = "verdict-store")]
#[test]
fn both_unlink_streams_write_the_same_table_through_the_crash_point_recorder_as_without_it() {
    for stream in STREAMS {
        let prepared = prepare_the_crash_stream(
            &format!("{}-write-table-without-the-recorder", stream.flow_name()),
            stream,
        );
        let recorded =
            common_multiple_copy_on_write_crash_streams::record_through_the_crash_point_recorder(
                &format!("{}-write-table-through-the-recorder", stream.flow_name()),
                stream,
            );
        let through_the_recorder = &recorded.prepared;
        assert!(
            through_the_recorder.base == prepared.base,
            "{stream:?}：起点镜像逐字节相同"
        );
        assert!(
            through_the_recorder.writes == prepared.writes,
            "{stream:?}：写表逐项相同"
        );
        assert_eq!(
            through_the_recorder.segments, prepared.segments,
            "{stream:?}：段一样切"
        );
        assert_eq!(
            through_the_recorder.judged_root_index, prepared.judged_root_index,
            "{stream:?}：被判的根槽写相同"
        );
        assert_eq!(
            (
                &through_the_recorder.versions_with_a_file,
                &through_the_recorder.versions_without_a_file
            ),
            (
                &prepared.versions_with_a_file,
                &prepared.versions_without_a_file
            ),
            "{stream:?}：两样版本表相同"
        );
        assert_eq!(
            through_the_recorder.versions_of_the_stream, prepared.versions_of_the_stream,
            "{stream:?}：流里每一版相同"
        );
        assert_eq!(
            recorded.crash_points.len(),
            prepared.segments.len(),
            "{stream:?}：每个屏障段一个崩溃点"
        );
    }
}

/// 共用前缀：多 inode 交错覆盖写那一条与删除这两支到「写 150」为止的崩溃点逐个相同（名字、设点文件、罩的写），那一段写表逐项相同——
/// 前缀节点在崩溃判定库里只判一次，别的流直接复用；写 150 之后的第一个点各走各的（删除这两支是 150 的意图或第一个文件的意图，那一条是写 2）。
#[cfg(feature = "verdict-store")]
#[test]
fn the_two_unlink_streams_share_their_nodes_through_the_write_of_inode_one_hundred_and_fifty_with_the_several_inodes_stream(
) {
    let flows = [
        MultipleCopyOnWriteCrashStream::SeveralInodesOverwrittenInTurn,
        MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile,
        MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile,
    ];
    let mut shared_prefixes = Vec::new();
    for stream in flows {
        let recorded =
            common_multiple_copy_on_write_crash_streams::record_through_the_crash_point_recorder(
                &format!("{}-shared-nodes", stream.flow_name()),
                stream,
            );
        let prepared = &recorded.prepared;
        let pool = &prepared.recorded.pool;
        // 共用前缀的最后一步是写 150（C 之后第二步）：它发完时录制流的长度，换成 mkfs 之后那一段的下标，数出前缀罩了几次写。
        let operations_through_the_shared_prefix =
            prepared.recorded.steps[1].operations_after_the_step - pool.mkfs_operation_count;
        let writes_through_the_shared_prefix = prepared
            .stream_indexes
            .iter()
            .filter(|stream_index| **stream_index < operations_through_the_shared_prefix)
            .count();
        let points_through_the_shared_prefix: Vec<(String, String, std::ops::Range<usize>)> =
            recorded
                .crash_points
                .iter()
                .take_while(|point| point.writes.end <= writes_through_the_shared_prefix)
                .map(|point| {
                    (
                        point.name.clone(),
                        point.code_file_in_repository.clone(),
                        point.writes.clone(),
                    )
                })
                .collect();
        let point_right_after_the_shared_prefix = recorded
            .crash_points
            .get(points_through_the_shared_prefix.len())
            .map(|point| point.name.clone())
            .expect("共用前缀之后还有点");
        shared_prefixes.push((
            stream,
            points_through_the_shared_prefix,
            prepared.writes[..writes_through_the_shared_prefix].to_vec(),
            point_right_after_the_shared_prefix,
        ));
    }
    let (_stream, reference_points, reference_writes, _first_point) = &shared_prefixes[0];
    let last_shared_point = reference_points.last().expect("共用前缀至少一个点");
    assert!(
        last_shared_point
            .0
            .starts_with("publish_sequential_write_of_an_inode.")
            && last_shared_point.1 == common_multiple_copy_on_write_crash_streams::THIS_FILE,
        "共用前缀的最后一个点是公共模块设的写 150：{last_shared_point:?}"
    );
    for (stream, points, writes, point_right_after) in &shared_prefixes {
        assert_eq!(
            points, reference_points,
            "{stream:?}：到写 150 为止的崩溃点与多 inode 那一条逐个相同"
        );
        assert!(
            writes == reference_writes,
            "{stream:?}：到写 150 为止的写表与多 inode 那一条逐项相同"
        );
        let expected_entry = match stream {
            MultipleCopyOnWriteCrashStream::SeveralInodesOverwrittenInTurn => {
                "publish_sequential_write_of_an_inode."
            }
            MultipleCopyOnWriteCrashStream::OtherFileDeletedBeforeTheFirstFile
            | MultipleCopyOnWriteCrashStream::FirstFileDeletedBeforeTheOtherFile => {
                "publish_unlink_intent_of_an_inode."
            }
            MultipleCopyOnWriteCrashStream::OneFileOverwrittenAcrossTwoWritableRemounts
            | MultipleCopyOnWriteCrashStream::LargeFileOverwrittenAcrossLowerLeaves => {
                panic!("{stream:?} 不走这一段共用前缀")
            }
        };
        assert!(
            point_right_after.starts_with(expected_entry),
            "{stream:?}：共用前缀之后的第一个点是 {expected_entry}…，实际 {point_right_after}"
        );
    }
}

/// 崩溃放量的形态：两支各按环境跑一趟，健康的流零红；展开上限默认 6。挂两张登记表上的观察者
/// （`common_multiple_copy_on_write_crash_streams::amplify_a_multiple_copy_on_write_flow`）：标准前缀节点上的可写挂载续做（与多记录流、删除流在 C 之前的块上共享），
/// 与 C 之后那几步节点上的落点、可写挂载续做读回每个文件两个（建 inode、写 150 两个节点上的块与多 inode 那一条共享）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_both_unlink_streams_is_clean() {
    for stream in STREAMS {
        let recorded =
            common_multiple_copy_on_write_crash_streams::record_through_the_crash_point_recorder(
                &format!("crash-amplification-{}", stream.flow_name()),
                stream,
            );
        common_multiple_copy_on_write_crash_streams::amplify_a_multiple_copy_on_write_flow(
            recorded, 6,
        );
    }
}
