//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 中央映射树分裂到三层、再收缩回一片叶的流（压小节点容量的只供测试的开关下）：
//! 第一个文件那一版（中央映射树 `L5 L5 I2`，基镜像，不枚举）→ 空发布（叶容量 5、内部容量 2：右叶切开、根装不下也切开，树高 2 → 3，`L3 L3 L4 I2 I1 I2`，写 13 个单元）
//! → 建一个 inode 的发布（容量换回节点格式那一档：删空的叶与内部节点从父节点里摘掉，根只剩一个孩子时降高，树高 3 → 1，`L10`，写 10 个单元）。
//! 收缩照 `singlefs_core::code_two_tree` 文件头规则 ③：删到空的节点从父节点里摘掉，根只剩一个孩子时降高一层。
//!
//! 基镜像与空发布那 4 段（全量 67,108,875 个状态）与 `crash_enumeration_tree_split_streams.rs` 的 `CentralMappingTwoLevelsSplitInARow`
//! 那一格逐字节相同（基镜像摘要与四段写表摘要相同，审查探针现算）；崩溃判定库的块键只由内容定，两条流相同的块走库复用。
//! 这条流新增的是建 inode 那一次发布的 4 段（全量 1,048,587 个状态），整条流状态数写在 `the_recorded_stream_counts_its_crash_states`。
//!
//! 每个崩溃状态跑恢复（看 / 不看 journal）、多版本 oracle、池级 checker 每条已实现的不变量（含 I-1.1 码 2 树逐节点合法性）、记录核对器。
//! 每一段都展开任意真子集，与闭式比；登记成崩溃枚举用例，带断点续跑、可双机分片。

mod common;
mod common_crash_points;
mod common_tree_split;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS};
use common_tree_split::{
    capacities, shape_text, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
    CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
};
use singlefs_core::transaction::{
    publish_new_inodes, CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, PoolWriter,
};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_verification::crash_library::StageOutcome;
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_verification::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_verification::segments::StepKind;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_tree_split_and_shrink_stream.rs";

/// 崩溃点重放项名第一个 `::` 之前那一截（来历表里这条流的名字）。
const FLOW_NAME_AT_THE_ENUMERATION: &str = "tree_split_and_shrink";

/// 崩溃放量里这条流的名字（来历表里与之并列的一行）。
const FLOW_NAME_AT_THE_AMPLIFICATION: &str = "tree-split-and-shrink";

/// 录制流闭式计数（`crash_state_count_with_torn_in_place_overwrites` 按全量展开数）：1 + 空发布 4 段 67,108,875 + 建 inode 那次 4 段 1,048,587。
const FULL_STATE_COUNT: u64 = 68_157_463;

const BIG_ACCOUNTING: (usize, usize) = ACCOUNTING_OF_THE_NODE_FORMAT;
const BIG_CENTRAL_MAPPING: (usize, usize) = CENTRAL_MAPPING_OF_THE_NODE_FORMAT;

struct PreparedTreeStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn publish_one_new_inode(pool: &mut TreeSplitPool, tree_capacities: CodeTwoTreeNodeCapacities) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let operations_before = pool.stream.operations().len();
    let mut writer = PoolWriter::new(&publish_parameters, pool.devices.as_mut_slice());
    writer.set_code_two_tree_node_capacities(tree_capacities);
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS + 60,
        previous.root.instance,
    )
    .expect("发布新 inode");
    pool.earlier_versions.push(previous);
    pool.output = output;
    pool.operations_before_the_current_version = operations_before;
}

fn prepare_stream() -> PreparedTreeStream {
    // 初始：中央映射树按叶容量 9、内部容量 2 建立，形如 L5 L5 I2
    let mut pool =
        TreeSplitPool::with_the_first_file_version_under(capacities(BIG_ACCOUNTING, (9, 2)));
    assert_eq!(
        shape_text(
            pool.output
                .multi_level_tree(MultiLevelCodeTwoTree::CentralMapping)
        ),
        "L5 L5 I2",
        "第一个文件版本后中央映射树初始为两叶一内部根"
    );
    let base = pool.memory_pool();
    let operations_start = pool.stream.operations().len();

    // 空发布，叶容量 5、内部容量 2：右叶切开，根装不下也切开，长出第三层
    pool.empty_publish(capacities(BIG_ACCOUNTING, (5, 2)));
    assert_eq!(
        shape_text(
            pool.output
                .multi_level_tree(MultiLevelCodeTwoTree::CentralMapping)
        ),
        "L3 L3 L4 I2 I1 I2",
        "空发布之后右叶与根都切开，树高 2 → 3"
    );
    let version_middle = pool.output.clone();

    // 建一个 inode 的发布，容量换回节点格式那一档：删空的叶与内部节点摘掉，根只剩一个孩子时降高，直到单叶根
    publish_one_new_inode(&mut pool, capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING));
    assert_eq!(
        shape_text(
            pool.output
                .multi_level_tree(MultiLevelCodeTwoTree::CentralMapping)
        ),
        "L10",
        "建 inode 那一次删空的节点摘掉、根降高，树高 3 → 1"
    );
    let version_after = pool.output.clone();

    let operations = pool.retained_operations();
    let (writes, segments) = writes_and_segments(&operations[operations_start..], &geometry());

    let mut judged_root_index = 0;
    for (index, write) in writes.iter().enumerate() {
        if write.kind == StepKind::RootRecordFua {
            judged_root_index = index;
        }
    }

    let versions = vec![
        PublishedVersion {
            instance: pool.earlier_versions[0].root.instance,
            checkpoint_txg: pool.earlier_versions[0].root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: version_middle.root.instance,
            checkpoint_txg: version_middle.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: version_after.root.instance,
            checkpoint_txg: version_after.root.checkpoint_txg,
            content: file_content(),
        },
    ];

    PreparedTreeStream {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

fn enumerate(
    item_label: &str,
    prepared: &PreparedTreeStream,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> StageOutcome<CrashPointReplayTally> {
    let item_name = format!("{FLOW_NAME_AT_THE_ENUMERATION}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&prepared.versions),
        expansion,
        &[],
        &[],
    )
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

#[test]
fn full_enumeration_of_the_tree_split_and_shrink_stream_is_exhaustive_and_clean() {
    let prepared = prepare_stream();
    let Some(tally) =
        judged_tally_or_left_to_another_stage(enumerate("full", &prepared, &full_expansion))
    else {
        return;
    };
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    println!(
        "CRASH_POINT_REPLAY_TREE_SPLIT_AND_SHRINK states={} closed_form={} exhaustive={} violations={} checker {} first_violation={}",
        tally.states,
        closed_form,
        tally.states == closed_form,
        tally.violations,
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "tree_split_and_shrink_stream：用例自己的展开下的闭式",
    );
}

#[test]
fn crash_amplification_of_the_tree_split_and_shrink_stream_is_clean() {
    let prepared = prepare_stream();
    let crash_points = common_crash_points::one_crash_point_over_every_write(
        file!(),
        FLOW_NAME_AT_THE_AMPLIFICATION,
        prepared.writes.len(),
    );
    common_crash_points::amplify_prepared_flow(
        FLOW_NAME_AT_THE_AMPLIFICATION,
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        crash_points,
        singlefs_verification::crash_amplification::EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
    );
}

/// 只录流，按写流算全量状态数，不枚举、不开判定库：录法变了而 `FULL_STATE_COUNT` 没跟着改就红。
// crash-case-check:not-a-crash-case 只录流、按写流算全量状态数，不枚举、不开判定库（用例文档写明）；枚举在同文件登记的 crash-case:tree-split-and-shrink-stream
#[test]
fn the_recorded_stream_counts_its_crash_states() {
    let prepared = prepare_stream();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    println!(
        "CRASH_ENUMERATION_STATE_COUNT name={FLOW_NAME_AT_THE_ENUMERATION} states={states} writes={} segments={:?}",
        prepared.writes.len(),
        prepared.segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    assert_eq!(states, FULL_STATE_COUNT, "录制流的全量状态数");
}
