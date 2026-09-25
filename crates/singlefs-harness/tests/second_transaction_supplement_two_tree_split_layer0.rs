//! 里程碑「第二个事务」增补 2 收口表第 28 行的层 0：记账树与中央映射树分裂的每一种写序进崩溃点（用户 2026-09-24 嘱咐「做分裂的时候切记
//! 需要加崩溃点」；三方 `m2-treesplit-r1` T4 的写法）。**起点镜像不枚举，只录分裂那一次发布**：分裂之前那一大截（mkfs、取号、暖机、
//! 第一个文件版本、铺垫的空发布）全部施加成起点镜像，录制流从那一次发布的第一个单元写切起——段长只随那一次发布写几个单元涨，
//! 不随树里有多少条涨。每条流枚举那一次发布的全部崩溃状态（D13（验证路线） 已定项 4 的段模型：单元写段、记录段、根槽 FUA、
//! 系统配置槽轮换），每个状态跑两遍恢复（看 / 不看 journal）+ 多版本 oracle + 池级 checker（多层码 2 树逐节点核 I-1.1）+ 记录核对器。
//!
//! 七条流，每条一次结构变化（节点容量用只供测试的开关压小，`CodeTwoTreeNodeCapacities::CappedForTests`）：
//! - 中央映射树：根分裂树高 +1、叶分裂（树高不变、叶多一片）、两层连着分裂（叶切开让根装不下、根也切开，树高 2 → 3）、
//!   摘空节点（一片叶里的 key 全被换掉、那片叶摘掉，叶少一片）、根降高（覆盖写换掉全部 key，树高 2 → 1）；
//! - 记账树：根分裂树高 +1、叶分裂（树高不变、叶多一片）。
//!
//! 除根降高那一条是覆盖写（换掉映射里的每一把 key 才降得下来），其余都是一次空发布：只换分配记录树与记账树节点的映射 key，
//! 写的单元最少（分配记录树 + 记账树的节点 + 中央映射树改了的那条路径 + 树表），状态数最小。
//!
//! **比已有的流多罩了什么**：已有的五条层 0 流里记账树与中央映射树恒是根兼叶，一次发布写一个记账节点、一个映射节点；
//! 这七条里一次发布写出 2–5 个映射树节点或 3–4 个记账树节点（分裂出来的新节点与路径上的祖先），它们与别的单元同一段
//! （D8（核心索引结构） 已定项 11 ①：不加写序步骤、不加屏障），单元写段里的子集第一次包括「新叶落了、新根没落」「新根落了、
//! 某片新叶没落」「被照抄的叶之外全落了」这一类；根降高那一条里新根记录指着的映射树只有一个节点而上一版有三个。
//! 多跑的一步只有恢复本身（这几条流上没有重开、挂载）。与已有流的基线镜像、写表、段序列都不相同，各自全量枚举。

mod common;
mod common_tree_split;

use common::{file_content, geometry};
use common_tree_split::{
    capacities, leaf_count, shape_text, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
    CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
};
use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, writes_and_segments, Layer0Tally,
    MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 覆盖写换上去的内容：一个数据单元装得下、与第一个文件不同。
fn second_content() -> Vec<u8> {
    (0..2500)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 一条流：先按 `capacities_of_each_publish_before_the_target` 发第一个文件版本与铺垫的空发布，再按 `target_capacities`
/// 发被录的那一次（空发布或覆盖写）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeSplitStream {
    CentralMappingRootSplit,
    CentralMappingLeafSplit,
    CentralMappingTwoLevelsSplitInARow,
    CentralMappingEmptyLeafDropped,
    CentralMappingRootLowered,
    AccountingRootSplit,
    AccountingLeafSplit,
}

/// 被录的那一次是什么发布。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetPublish {
    Empty,
    Overwrite,
}

/// 一条流的布置：第一个文件版本用的容量、被录那一次用的容量与发布种类，与那一次前后这棵树的样子（`shape_text`）。
struct StreamPlan {
    first_file_capacities: CodeTwoTreeNodeCapacities,
    target_capacities: CodeTwoTreeNodeCapacities,
    target: TargetPublish,
    tree: MultiLevelCodeTwoTree,
    shape_before: &'static str,
    shape_after: &'static str,
    /// 被录那一次写出几个单元（每个两盘各一次写）。
    units_written: usize,
}

const BIG_ACCOUNTING: (usize, usize) = ACCOUNTING_OF_THE_NODE_FORMAT;
const BIG_CENTRAL_MAPPING: (usize, usize) = CENTRAL_MAPPING_OF_THE_NODE_FORMAT;

impl TreeSplitStream {
    const ALL: [TreeSplitStream; 7] = [
        TreeSplitStream::CentralMappingRootSplit,
        TreeSplitStream::CentralMappingLeafSplit,
        TreeSplitStream::CentralMappingTwoLevelsSplitInARow,
        TreeSplitStream::CentralMappingEmptyLeafDropped,
        TreeSplitStream::CentralMappingRootLowered,
        TreeSplitStream::AccountingRootSplit,
        TreeSplitStream::AccountingLeafSplit,
    ];

    fn plan(self) -> StreamPlan {
        match self {
            // 第一个文件版本：映射 6 条根兼叶。空发布删掉分配记录树与记账树的旧 key、插进新 key，叶容量 5：
            // 第 6 条插进去时从中间切（3 + 3），长出新根。写：分配记录树、记账树、映射树三个节点、树表 = 6 个单元。
            TreeSplitStream::CentralMappingRootSplit => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                target_capacities: capacities(BIG_ACCOUNTING, (5, 3)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L6",
                shape_after: "L3 L3 I2",
                units_written: 6,
            },
            // 第一个文件版本按叶容量 5 长成两片叶（3 + 3）。空发布换掉右叶里的分配记录树与记账树 key，叶容量压到 2：
            // 右叶从中间切（2 + 1），根多一个孩子、树高不变；左叶照抄。写：分配记录树、记账树、映射树两片叶与根、树表 = 6 个单元。
            TreeSplitStream::CentralMappingLeafSplit => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (5, 3)),
                target_capacities: capacities(BIG_ACCOUNTING, (2, 3)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L3 L3 I2",
                shape_after: "L3 L2 L1 I3",
                units_written: 6,
            },
            // 同上，内部节点容量 2：右叶切开让根有三个孩子、装不下，根也从中间切（2 + 1），长出第三层。
            // 写：分配记录树、记账树、映射树两片叶、两个层级 1 的节点与新根、树表 = 8 个单元。
            TreeSplitStream::CentralMappingTwoLevelsSplitInARow => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (5, 2)),
                target_capacities: capacities(BIG_ACCOUNTING, (2, 2)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L3 L3 I2",
                shape_after: "L3 L2 L1 I2 I1 I2",
                units_written: 8,
            },
            // 记账树叶容量 14：15 行长成两片叶加根（三个节点、三条映射条目）；映射叶容量 2：八条映射条目长成四片叶，
            // 其中一片只装记账树两片叶的 key。空发布把它们全换掉：那片叶删空、摘掉；新 key 按分隔 key 落进装着 inode 叶那一片
            // （叶容量放到 8，装得下不再切）。叶从四片变三片，最左那片照抄。
            // 写：分配记录树、记账树三个节点、映射树两片叶与根、树表 = 8 个单元。
            TreeSplitStream::CentralMappingEmptyLeafDropped => StreamPlan {
                first_file_capacities: capacities((14, 3), (2, 8)),
                target_capacities: capacities((14, 3), (8, 8)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L2 L2 L2 L2 I4",
                shape_after: "L2 L2 L4 I3",
                units_written: 8,
            },
            // 第一个文件版本按叶容量 5 长成两片叶。覆盖写换掉映射里的每一把 key（数据单元、extent 根、inode 叶与根、分配记录树、
            // 记账树），按产品容量发：先删的时候左叶删空摘掉、根只剩一个孩子降高，新 key 插回一个节点装得下，树高 2 → 1。
            // 写：数据单元、extent 根、inode 叶与根、分配记录树、记账树、映射树一个节点、树表 = 8 个单元。
            TreeSplitStream::CentralMappingRootLowered => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (5, 3)),
                target_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                target: TargetPublish::Overwrite,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L3 L3 I2",
                shape_after: "L6",
                units_written: 8,
            },
            // 记账树 15 行根兼叶；空发布整批换代，叶容量 14：第 15 行插进去时从中间切（8 + 7），长出新根。
            // 写：分配记录树、记账树两片叶与根、映射树一个节点、树表 = 6 个单元。
            TreeSplitStream::AccountingRootSplit => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                target_capacities: capacities((14, 3), BIG_CENTRAL_MAPPING),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::Accounting,
                shape_before: "L15",
                shape_after: "L8 L7 I2",
                units_written: 6,
            },
            // 记账树按叶容量 14 两片叶加根；空发布叶容量压到 8：15 行插到第 9 行切一次、第 14 行再切一次，三片叶（5 + 5 + 5），
            // 树高不变。写：分配记录树、记账树三片叶与根、映射树一个节点、树表 = 7 个单元。
            TreeSplitStream::AccountingLeafSplit => StreamPlan {
                first_file_capacities: capacities((14, 3), BIG_CENTRAL_MAPPING),
                target_capacities: capacities((8, 3), BIG_CENTRAL_MAPPING),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::Accounting,
                shape_before: "L8 L7 I2",
                shape_after: "L5 L5 L5 I3",
                units_written: 7,
            },
        }
    }
}

/// 录好的一条流：起点镜像、那一次发布的写表与段、被判的根槽写、oracle 认的两版。
struct PreparedStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn prepare(stream: TreeSplitStream) -> PreparedStream {
    let plan = stream.plan();
    let mut pool = TreeSplitPool::with_the_first_file_version_under(plan.first_file_capacities);
    assert_eq!(
        shape_text(pool.output.multi_level_tree(plan.tree)),
        plan.shape_before,
        "{stream:?}：被录那一次之前这棵树的样子"
    );
    let content_before = file_content();
    let content_after = match plan.target {
        TargetPublish::Empty => {
            pool.empty_publish(plan.target_capacities);
            content_before.clone()
        }
        TargetPublish::Overwrite => {
            let content = second_content();
            pool.overwrite(&content, plan.target_capacities);
            content
        }
    };
    let before = pool.version_before_the_current_one();
    let after = &pool.output;
    assert_eq!(
        shape_text(after.multi_level_tree(plan.tree)),
        plan.shape_after,
        "{stream:?}：被录那一次之后这棵树的样子"
    );
    assert_eq!(
        after.rewritten.len(),
        plan.units_written,
        "{stream:?}：被录那一次写出的单元：{:?}",
        after.rewritten
    );
    let operations = pool.retained_operations();
    let (writes, segments) = writes_and_segments(
        &operations[pool.operations_before_the_current_version..],
        &geometry(),
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 1, "只录了一次发布：一次根槽写");
    let versions = vec![
        PublishedVersion {
            instance: before.root.instance,
            checkpoint_txg: before.root.checkpoint_txg,
            content: content_before,
        },
        PublishedVersion {
            instance: after.root.instance,
            checkpoint_txg: after.root.checkpoint_txg,
            content: content_after,
        },
    ];
    PreparedStream {
        base: pool.memory_pool_before_the_current_version(),
        writes,
        segments,
        judged_root_index: root_indexes[0],
        versions,
    }
}

/// 一次发布的段：单元写（每个单元两盘各一次）→ 屏障 → 一条记录两盘各一份 → 屏障 → 根槽 FUA（它自己一段）→ 两盘系统配置槽轮换。
fn expected_segment_sizes(units_written: usize) -> Vec<usize> {
    vec![2 * units_written, 2, 1, 2]
}

fn assert_clean(stream: TreeSplitStream, tally: &Layer0Tally) {
    assert_eq!(
        tally.violations, 0,
        "{stream:?} oracle：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "{stream:?} 不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "{stream:?} 走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "{stream:?} 记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{stream:?} {invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 枚举一条流：`expand` 为真的段展开任意子集，别的段只以整段持久进入后面的状态。
fn enumerate(stream: TreeSplitStream, expand: &dyn Fn(usize, &[usize]) -> bool) -> Layer0Tally {
    let prepared = prepare(stream);
    let plan = stream.plan();
    let sizes: Vec<usize> = prepared.segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        expected_segment_sizes(plan.units_written),
        "{stream:?}：只录那一次发布，四段"
    );
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .enumerate()
        .filter(|(index, segment)| expand(*index, segment))
        .map(|(_, segment)| segment.clone())
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "{stream:?}：展开的段按闭式数"
    );
    println!(
        "LAYER0_TREE_SPLIT stream={stream:?} states={} closed_form_of_every_segment={} root_persisted_states={} journal_differing_states={} verification_ran_states={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        closed_form_state_count(&prepared.segments),
        tally.root_persisted_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(stream, &tally);
    tally
}

/// 平时跑的那一份：每条流的单元写段只以整段持久进入后面的状态（不展开），记录段、根槽、系统配置槽轮换那几段全展开——
/// 「单元全落了、记录落了一份、根没落」这一类每条流都跑到。
#[test]
fn every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes() {
    for stream in TreeSplitStream::ALL {
        let tally = enumerate(stream, &|_segment_index, segment| segment.len() < 4);
        assert_eq!(
            tally.states,
            1 + 3 + 1 + 3,
            "{stream:?}：记录段 3 + 根槽 1 + 轮换 3 + 全部持久 1"
        );
    }
}

/// 全量：每条流那一次发布的全部崩溃状态，单元写段 2^(2 × 单元数) − 1 个。
/// 七条流合计见各自的 `LAYER0_TREE_SPLIT` 行：6 个单元的 4095、7 个 16383、8 个 65535。
#[test]
#[ignore = "七条流合计约 22 万个状态、每个两遍恢复 + checker，debug 下太慢；交 crash-verifier 在 release 下跑"]
fn full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean() {
    for stream in TreeSplitStream::ALL {
        let tally = enumerate(stream, &|_segment_index, _segment| true);
        let units_written = stream.plan().units_written;
        assert_eq!(
            tally.states,
            1 + ((1u64 << (2 * units_written)) - 1) + 3 + 1 + 3,
            "{stream:?}：闭式 1 + (2^(2 × {units_written}) − 1) + 3 + 1 + 3"
        );
    }
}

/// 两条记账树的流各自的树高从根节点头现读，与形状相符（D28（挂载期承诺量） 已定项 4）。
#[test]
fn the_accounting_streams_read_the_tree_height_from_the_root_node_header() {
    for (stream, height_before, height_after) in [
        (TreeSplitStream::AccountingRootSplit, 1, 2),
        (TreeSplitStream::AccountingLeafSplit, 2, 2),
    ] {
        let plan = stream.plan();
        let mut pool = TreeSplitPool::with_the_first_file_version_under(plan.first_file_capacities);
        assert_eq!(
            pool.height(MultiLevelCodeTwoTree::Accounting),
            height_before
        );
        pool.empty_publish(plan.target_capacities);
        assert_eq!(pool.height(MultiLevelCodeTwoTree::Accounting), height_after);
        assert_eq!(
            leaf_count(&pool.output.accounting_tree),
            shape_text(&pool.output.accounting_tree)
                .matches('L')
                .count()
        );
    }
}
