//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 多个 inode 交错覆盖写（里程碑三一·1 第二条流，并行线三的形态）：多记录流「取号 → 暖机 × 2 → A → B → C」（C 是第一个文件三个数据单元）之后
//! 建 149 个空 inode（号 2..=150，一次发布）、按 inode 号写 150 两个单元（号 150 落在 extent 上段第 1 片叶上，上段从层级 0 长到层级 1）、
//! 写 2 两个单元（与第一个文件同在第 0 片叶上）、覆盖写第一个文件成一个单元、覆盖写 150 成一个单元、覆盖写 2 成一个单元。
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑两遍恢复 + 多版本 oracle（第一个文件每一版的内容）+ 池级 checker + 记录核对器；
//! 另挂两个观察者（`common_multiple_copy_on_write_crash_streams` 文件头）：记恢复落到哪一版，与在每个状态上可写挂载一次、读回第一个文件、
//! inode 2 与 inode 150 都等于落到的那一版（oracle 只读第一个文件，别的两个由它核）、挂完的盘过池级 checker（判段判，配了 GPU 上卡）。
//!
//! **比已有的流多罩了什么**：已有的崩溃流里只有第一个文件写数据，extent 上段恒是第 0 片根兼叶（`crash_enumeration_position_addressed_trees.rs`
//! 断言的就是这一形）；这一条第一次枚举按 inode 号写别的文件的那几次发布：上段长到层级 1 那一次（第 0、1 片叶与根同一次写）、同一片叶里两个文件的
//! 条目一条重写一条照抄、写一个文件时别的文件的数据单元与下段照抄、别的文件整份覆盖写时它上一版的单元经映射释放（下段根换回内联）。
//! 前二十段与多记录流逐字节相同；建 inode 与写 150 两步与多文件删除那两支（`crash_enumeration_unlink_of_two_files_holding_data_streams.rs`）
//! 共用同一份录法、同一个设点文件，崩溃放量那一路在库里共享。多跑的步：每个状态上观察者那一次可写挂载、一次只读挂载读回三个文件。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例，可双机分片。系统配置槽写是原地覆写，取三态，其余写取两态。

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
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_enumeration_several_inodes_overwritten_in_turn_stream.rs";

/// 这一条流。
const STREAM: MultipleCopyOnWriteCrashStream =
    MultipleCopyOnWriteCrashStream::SeveralInodesOverwrittenInTurn;

/// 段形（草稿探针录流数出来的，只用 harness 档与实现录同一份步骤）：前二十段同多记录流（取号两轮各一段）；建 inode 的单元段 20、写 150 的 30（上段长到层级 1）、
/// 写 2 的 30、覆盖写第一个文件的 26、覆盖写 150 的 30、覆盖写 2 的 30，各自记录、根、轮换。
const SEGMENT_SIZES: [usize; 44] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 20, 2, 1, 2, 30, 4, 1, 2, 30, 4,
    1, 2, 26, 2, 1, 2, 30, 2, 1, 2, 30, 2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（取号写一轮时 5_722_079_400，取号第二轮那一段 +3）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 5_722_079_403;

/// 快档展开的段每次写只取两态时的闭式（取号写一轮时 177）。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 180;

fn prepare(tag: &str) -> PreparedMultipleCopyOnWriteCrashStream {
    prepare_the_crash_stream(tag, STREAM)
}

/// 录流的形状（判器判完之后再核：改坏实现时先红在判器上）：段形，与每一步之后的上段、第一个文件的单元数与下段。
fn assert_the_shape_of_the_flow(prepared: &PreparedMultipleCopyOnWriteCrashStream) {
    assert_eq!(
        prepared.segment_sizes(),
        SEGMENT_SIZES,
        "前二十段同多记录流；之后建 inode、五次按 inode 号写"
    );
    assert_the_shape_of_the_steps(&prepared.recorded);
}

/// 按 `expansion` 枚举这条流，挂两个观察者（落点、可写挂载续做读回每个文件）；只录那一趟交回 `None`。
fn enumerate_the_stream(
    prepared: &PreparedMultipleCopyOnWriteCrashStream,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let versions = prepared.versions_of_the_stream.as_slice();
    let inodes_read_back = STREAM.inodes_read_back();
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
    let item_name = format!("{}::{item_label}", STREAM.flow_name());
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

/// 平时跑的那一份：写数不小于 10 的段不展开，其余每段任意真子集；每个状态记落点、可写挂载一次读回第一个文件、inode 2 与 inode 150。
/// 判器（oracle、池级 checker、记录核对器）先判，录流的形状与计数在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_several_inodes_overwritten_in_turn_reads_every_file_of_a_published_version_and_stays_clean(
) {
    let prepared = prepare("several-inodes-in-turn-crash-point-replay-fast");
    let Some(tally) = enumerate_the_stream(&prepared, "quick_tier::fast", &quick_tier_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_SEVERAL_INODES_OVERWRITTEN_IN_TURN_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(STREAM, &tally);
    assert_the_shape_of_the_flow(&prepared);
    assert_eq!(
        closed_form_state_count(
            &prepared.segments_shorter_than(QUICK_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW)
        ),
        QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时展开的段按闭式数"
    );
    assert_eq!(
        tally.states,
        crash_state_count_under(&prepared, &quick_tier_expansion),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_every_state_was_observed(&prepared, &tally);
}

/// 复用不改判定：同一趟快档跑两遍，第二遍的块与观察者结论全照库里的，两份计数逐项相同。
#[test]
fn the_quick_tier_of_several_inodes_overwritten_in_turn_counts_the_same_when_rerun_from_the_library(
) {
    let prepared = prepare("several-inodes-in-turn-crash-point-replay-rerun");
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

/// 全量：每段都展开，每个状态两遍恢复 + checker + 两个观察者。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_SEVERAL_INODES_OVERWRITTEN_IN_TURN` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 57 亿个状态（两态闭式 5722079403；原地覆写取三态的枚举域以用例打出的为准）、每个两遍恢复 + checker + 一次可写挂载（按恢复镜像去重）；门禁 checker-tier-crash-point-replay --full 在 release 下跑，可双机分片"]
fn full_enumeration_of_several_inodes_overwritten_in_turn_is_exhaustive_and_clean() {
    let prepared = prepare("several-inodes-in-turn-crash-point-replay-full");
    assert_eq!(
        closed_form_state_count(
            &prepared.segments_shorter_than(FULL_TIER_WRITES_OF_AN_EXPANDED_SEGMENT_BELOW)
        ),
        CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)"
    );
    let segments_left_whole = segments_left_whole_text(&prepared);
    assert_eq!(segments_left_whole, "-", "这条流的每一段都装得下全量展开");
    let closed_form = crash_state_count_under(&prepared, &full_tier_expansion);
    let Some(tally) = enumerate_the_stream(&prepared, "full", &full_tier_expansion) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_SEVERAL_INODES_OVERWRITTEN_IN_TURN states={} closed_form={closed_form} exhaustive={} segments_left_whole={segments_left_whole} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(STREAM, &tally);
    assert_eq!(tally.states, closed_form);
    assert_the_shape_of_the_flow(&prepared);
    assert_every_state_was_observed(&prepared, &tally);
}

/// 两份造法写表相同：崩溃点重放那一份（[`prepare`]，不录崩溃点）与录制器那一份（C 之后每一步设一个点）录下的起点镜像、整张写表、段、
/// 被判的根槽写、版本表与流里每一版逐项相同——崩溃放量与崩溃点重放判的是同一条流。
#[cfg(feature = "verdict-store")]
#[test]
fn several_inodes_overwritten_in_turn_write_the_same_table_through_the_crash_point_recorder_as_without_it(
) {
    let prepared = prepare("several-inodes-in-turn-write-table-without-the-recorder");
    let recorded =
        common_multiple_copy_on_write_crash_streams::record_through_the_crash_point_recorder(
            "several-inodes-in-turn-write-table-through-the-recorder",
            STREAM,
        );
    let through_the_recorder = &recorded.prepared;
    assert!(
        through_the_recorder.base == prepared.base,
        "起点镜像逐字节相同"
    );
    assert!(
        through_the_recorder.writes == prepared.writes,
        "写表逐项相同"
    );
    assert_eq!(through_the_recorder.segments, prepared.segments, "段一样切");
    assert_eq!(
        through_the_recorder.judged_root_index, prepared.judged_root_index,
        "被判的根槽写相同"
    );
    assert_eq!(
        through_the_recorder.versions_with_a_file, prepared.versions_with_a_file,
        "版本表相同"
    );
    assert_eq!(
        through_the_recorder.versions_of_the_stream, prepared.versions_of_the_stream,
        "流里每一版相同"
    );
    assert_eq!(
        recorded.crash_points.len(),
        prepared.segments.len(),
        "每个屏障段一个崩溃点"
    );
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；展开上限默认 6。挂两张登记表上的观察者（`common_multiple_copy_on_write_crash_streams::amplify_a_multiple_copy_on_write_flow`）：
/// 标准前缀节点上的可写挂载续做（与多记录流、删除流、截断流在 C 之前的块上共享），与 C 之后那几步节点上的落点、可写挂载续做读回每个文件两个。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_several_inodes_overwritten_in_turn_is_clean() {
    let recorded =
        common_multiple_copy_on_write_crash_streams::record_through_the_crash_point_recorder(
            "crash-amplification-several-inodes-in-turn",
            STREAM,
        );
    common_multiple_copy_on_write_crash_streams::amplify_a_multiple_copy_on_write_flow(recorded, 6);
}
