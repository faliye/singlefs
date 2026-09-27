//! 里程碑「第二个事务」增补 2 收口表第 28 行的层 0：记账树与中央映射树分裂的每一种写序进崩溃点（用户 2026-09-24 嘱咐「做分裂的时候切记
//! 需要加崩溃点」；三方 `m2-treesplit-r1` T4 的写法）。**起点镜像不枚举，只录分裂那一次发布**：分裂之前那一大截（mkfs、取号、暖机、
//! 第一个文件版本、铺垫的空发布）全部施加成起点镜像，录制流从那一次发布的第一个单元写切起——段长只随那一次发布写几个单元涨，
//! 不随树里有多少条涨。每条流枚举那一次发布的全部崩溃状态（D13（验证路线） 已定项 4 的段模型：单元写段、记录段、根槽 FUA、
//! 系统配置槽轮换），每个状态跑两遍恢复（看 / 不看 journal）+ 多版本 oracle + 池级 checker（多层码 2 树逐节点核 I-1.1）+ 记录核对器。
//!
//! 七条流，每条一次结构变化（节点容量用只供测试的开关压小，`CodeTwoTreeNodeCapacities::CappedForTests`）：
//! - 中央映射树：根分裂树高 +1、叶分裂（树高不变、叶多一片）、两层连着分裂（叶切开让根装不下、根也切开，树高 2 → 3）、
//!   摘空节点（一片叶里的 key 全被换掉、那片叶摘掉，叶少一片）、根降高（一片叶删空摘掉、根只剩一个非空的孩子，树高 2 → 1）；
//! - 记账树：根分裂树高 +1、叶分裂（树高不变、叶多一片）。
//!
//! 除根降高那一条是建一个 inode 的发布（inode 叶容器与 inode 根也换 key，右边那片叶才删得空；数据单元与 extent 根的 key 不动，
//! 左边那片叶留着这两条），其余都是一次空发布：只换分配记录树与记账树节点的映射 key，
//! 写的单元最少（分配记录树 + 记账树的节点 + 中央映射树改了的那条路径 + 树表），状态数最小。
//!
//! **比已有的流多罩了什么**：已有的五条层 0 流里记账树与中央映射树恒是根兼叶，一次发布写一个记账节点、一个映射节点；
//! 这七条里一次发布写出 1–6 个映射树节点或 3–4 个记账树节点（分裂出来的新节点与路径上的祖先），它们与别的单元同一段
//! （D8（核心索引结构） 已定项 11 ①：不加写序步骤、不加屏障），单元写段里的子集第一次包括「新叶落了、新根没落」「新根落了、
//! 某片新叶没落」「被照抄的叶之外全落了」这一类；根降高那一条里新根记录指着的映射树只有一个节点而上一版有三个。
//! 每次发布还带着分配记录树的五个节点（D8（核心索引结构） 已定项 14：两盘各叶 61、两盘各第 1 层、根），被录那一次写 9–13 个单元。
//! 多跑的一步只有恢复本身（这几条流上没有重开、挂载）。与已有流的基线镜像、写表、段序列都不相同，各自全量枚举。

mod common;
mod common_tree_split;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS};
use common_tree_split::{
    capacities, leaf_count, shape_text, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
    CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
};
use singlefs_core::transaction::{
    publish_new_inodes, CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, PoolWriter,
};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions,
    layer0_state_count_with_torn_in_place_overwrites, writes_and_segments, Layer0SegmentExpansion,
    Layer0Tally, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 接着现行那一版推一次建一个 inode 的发布（`publish_new_inodes`：inode 叶容器与 inode 根重写，数据单元与 extent 树照抄），
/// 写入口装上给定的容量。与 `TreeSplitPool::empty_publish` 同一种接法：录制流在这次发布之前有几步记进
/// `operations_before_the_current_version`，上一版推进 `earlier_versions`。
fn publish_one_new_inode(
    pool: &mut TreeSplitPool,
    capacities_of_this_publish: CodeTwoTreeNodeCapacities,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let operations_before = pool.stream.operations().len();
    let mut writer = PoolWriter::new(&publish_parameters, pool.devices.as_mut_slice());
    writer.set_code_two_tree_node_capacities(capacities_of_this_publish);
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS + 60,
        previous.root.instance,
    )
    .expect("建一个 inode");
    pool.earlier_versions.push(previous);
    pool.output = output;
    pool.operations_before_the_current_version = operations_before;
}

/// 一条流：先按 `first_file_capacities` 发第一个文件版本，再按 `target_capacities` 发被录的那一次（空发布或建一个 inode）。
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

/// 被录的那一次是什么发布。两种都不碰第一个文件：录前录后两版读回同一份内容。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetPublish {
    Empty,
    OneNewInode,
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

    // 第一个文件版本的中央映射树有 10 条（十二个单元里除去树表与映射树自己的节点，D19（块指针的结构与宽度预算） 已定项 8），
    // 按 key（类标签 → 出生树 → 出生 txg → 尾段）排：数据单元（码 1）| extent 根、inode 根、分配记录树五个节点（D8（核心索引结构）
    // 已定项 14：两盘各叶 61、两盘各第 1 层、根）、记账树（码 2，出生树 11、12、13、14）| inode 叶容器（码 3）。
    // 空发布换掉的是分配记录树五个节点与记账树的 6 条（先按 key 升序删、再按 key 升序插，D8（核心索引结构） 已定项 11），
    // 新 key 的出生 txg 更大，落在 inode 根与 inode 叶容器之间；其余 4 条（数据单元、extent 根、inode 根、inode 叶容器）不动。
    // 每个单元两盘各一次写：被录那一次的单元写段是 2 × `units_written` 写。
    fn plan(self) -> StreamPlan {
        match self {
            // 第一个文件版本：映射 10 条根兼叶。空发布删掉 6 条剩 4 条，再插 6 条新 key，叶容量 9：
            // 第 10 条插进去时从中间切（5 + 5），长出新根。写：分配记录树五个节点、记账树、映射树三个节点、树表 = 10 个单元。
            TreeSplitStream::CentralMappingRootSplit => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                target_capacities: capacities(BIG_ACCOUNTING, (9, 3)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L10",
                shape_after: "L5 L5 I2",
                units_written: 10,
            },
            // 第一个文件版本按叶容量 9 长成两片叶（第 10 条切开，5 + 5）。空发布删掉的 6 条两片叶里都有（左叶剩 3 条、右叶剩 inode 叶容器
            // 那 1 条），新 key 按分隔 key 全落进右叶，叶容量压到 5：右叶到 6 条时从中间切一次（3 + 3），之后那条落进切出的右边那片，
            // 三片叶（3 + 3 + 4），根三个孩子装得下（内部容量 3）、树高不变。左叶删了 key，照样重写。
            // 写：分配记录树五个节点、记账树、映射树三片叶与根、树表 = 11 个单元。
            TreeSplitStream::CentralMappingLeafSplit => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (9, 3)),
                target_capacities: capacities(BIG_ACCOUNTING, (5, 3)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L5 L5 I2",
                shape_after: "L3 L3 L4 I3",
                units_written: 11,
            },
            // 同上，内部节点容量 2：右叶切开让根有三个孩子、装不下，根也从中间切（2 + 1），长出第三层，树高 2 → 3。
            // 写：分配记录树五个节点、记账树、映射树三片叶、两个层级 1 的节点与新根、树表 = 13 个单元。
            TreeSplitStream::CentralMappingTwoLevelsSplitInARow => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (9, 2)),
                target_capacities: capacities(BIG_ACCOUNTING, (5, 2)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L5 L5 I2",
                shape_after: "L3 L3 L4 I2 I1 I2",
                units_written: 13,
            },
            // 第一个文件版本按叶容量 5 长成三片叶：第 6 条切一次（3 + 3）、第 9 条又切一次，中间那片只装分配记录树的 3 条 key。
            // 空发布把它们全换掉：那片叶删空、摘掉；新 key 按分隔 key 全落进装着 inode 叶容器那一片（叶容量放到 8，7 条装得下不再切）。
            // 叶从三片变两片，最左那片（数据单元、extent 根、inode 根）一条没变、照抄。
            // 写：分配记录树五个节点、记账树、映射树一片叶与根、树表 = 9 个单元。
            TreeSplitStream::CentralMappingEmptyLeafDropped => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (5, 3)),
                target_capacities: capacities(BIG_ACCOUNTING, (8, 3)),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L3 L3 L4 I3",
                shape_after: "L3 L7 I2",
                units_written: 9,
            },
            // 第一个文件版本按叶容量 9 长成两片叶（5 + 5）：左叶是数据单元、extent 根、inode 根与分配记录树的两个节点，右叶是分配记录树
            // 另三个节点、记账树与 inode 叶容器。被录的那一次建一个 inode、按产品容量发：inode 叶容器、inode 根、分配记录树五个节点、
            // 记账树这 8 条换 key，数据单元与 extent 根的 2 条不动。先删：左叶删到只剩那 2 条；右叶 5 条全删、删空摘掉，根只剩一个
            // 非空的孩子，降高、左叶当根（收缩只摘空节点，根只剩一个孩子时降高，D8（核心索引结构） 已定项 11 ③）；再插：新的 8 条进这片叶，
            // 10 条装得下，树高 2 → 1。降高是走到 `L10` 的唯一一条路：不降高，根留成只有一个孩子的内部节点，新 key 照样插进那片叶，
            // 录之后是 `L10 I1`；左叶留着 2 条，「两片叶都删空、根变回空叶」那一臂走不到。
            // 写：inode 叶容器与根、分配记录树五个节点、记账树、映射树一个节点、树表 = 10 个单元。
            TreeSplitStream::CentralMappingRootLowered => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, (9, 3)),
                target_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                target: TargetPublish::OneNewInode,
                tree: MultiLevelCodeTwoTree::CentralMapping,
                shape_before: "L5 L5 I2",
                shape_after: "L10",
                units_written: 10,
            },
            // 记账树 15 行根兼叶；空发布整批换代，叶容量 14：第 15 行插进去时从中间切（8 + 7），长出新根。
            // 写：分配记录树五个节点、记账树两片叶与根、映射树一个节点、树表 = 10 个单元。
            TreeSplitStream::AccountingRootSplit => StreamPlan {
                first_file_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                target_capacities: capacities((14, 3), BIG_CENTRAL_MAPPING),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::Accounting,
                shape_before: "L15",
                shape_after: "L8 L7 I2",
                units_written: 10,
            },
            // 记账树按叶容量 14 两片叶加根；空发布叶容量压到 8：15 行插到第 9 行切一次、第 14 行再切一次，三片叶（5 + 5 + 5），
            // 树高不变。写：分配记录树五个节点、记账树三片叶与根、映射树一个节点、树表 = 11 个单元。
            TreeSplitStream::AccountingLeafSplit => StreamPlan {
                first_file_capacities: capacities((14, 3), BIG_CENTRAL_MAPPING),
                target_capacities: capacities((8, 3), BIG_CENTRAL_MAPPING),
                target: TargetPublish::Empty,
                tree: MultiLevelCodeTwoTree::Accounting,
                shape_before: "L8 L7 I2",
                shape_after: "L5 L5 L5 I3",
                units_written: 11,
            },
        }
    }

    /// 全量用例里这条流那一行计数行的前缀：门禁 54 号按前缀认计数行（每个前缀恰好一行、只许大写字母、数字、`_`），
    /// 七条流各一个，登记在 `.claude/gate.d/stage-inputs.tsv` 那一条崩溃枚举用例的 `count-line=` 里。
    fn full_enumeration_count_line_prefix(self) -> &'static str {
        match self {
            TreeSplitStream::CentralMappingRootSplit => {
                "LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT"
            }
            TreeSplitStream::CentralMappingLeafSplit => {
                "LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT"
            }
            TreeSplitStream::CentralMappingTwoLevelsSplitInARow => {
                "LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW"
            }
            TreeSplitStream::CentralMappingEmptyLeafDropped => {
                "LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED"
            }
            TreeSplitStream::CentralMappingRootLowered => {
                "LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED"
            }
            TreeSplitStream::AccountingRootSplit => "LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT",
            TreeSplitStream::AccountingLeafSplit => "LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT",
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
    match plan.target {
        TargetPublish::Empty => pool.empty_publish(plan.target_capacities),
        TargetPublish::OneNewInode => publish_one_new_inode(&mut pool, plan.target_capacities),
    }
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
            content: file_content(),
        },
        PublishedVersion {
            instance: after.root.instance,
            checkpoint_txg: after.root.checkpoint_txg,
            content: file_content(),
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

/// 枚举一条流：`expand` 为真的段展开（每次写各取它的几态、任意组合，系统配置槽写是原地覆写、取三态），
/// 别的段只以整段持久进入后面的状态。
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
    assert_eq!(
        tally.states,
        layer0_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &|segment_index, segment| {
                if expand(segment_index, segment) {
                    Layer0SegmentExpansion::EveryProperSubset
                } else {
                    Layer0SegmentExpansion::NotExpanded
                }
            },
        ),
        "{stream:?}：展开的段按层 0 的枚举域数（原地覆写取三态）"
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
            1 + 3 + 1 + (3 * 3 - 1),
            "{stream:?}：记录段 3 + 根槽 1 + 轮换 3² − 1（两次系统配置槽写各取三态；每次写只取两态时 3）+ 全部持久 1"
        );
    }
}

/// 全量：每条流那一次发布的全部崩溃状态，单元写段 2^(2 × 单元数) − 1 个，轮换那一段 3² − 1 个（系统配置槽写取三态）。
/// 单元写段：9 个单元 262143（摘空）、10 个 1048575（中央映射根分裂、根降高、记账根分裂）、11 个 4194303（两条叶分裂）、
/// 13 个 67108863（两层连着分裂）；七条流各加记录段 3、根槽 1、轮换 8、全部持久 1，合计 78905428 个状态。
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv`），门禁 54 号 --full 在 release 下跑：每条流跑完打一行计数行
/// （前缀 `full_enumeration_count_line_prefix`，带 `exhaustive=`）。枚举不留进度文件、不认双机分片开关。
#[test]
#[ignore = "七条流合计 78905428 个状态、每个两遍恢复 + checker；登记成崩溃枚举用例，门禁 54 号 --full 在 release 下跑"]
fn full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean() {
    for stream in TreeSplitStream::ALL {
        let tally = enumerate(stream, &|_segment_index, _segment| true);
        let units_written = stream.plan().units_written;
        let closed_form = 1 + ((1u64 << (2 * units_written)) - 1) + 3 + 1 + (3 * 3 - 1);
        println!(
            "{} states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} root_persisted_states={} journal_differing_states={} verification_ran_states={} record_root_without_record={} record_claimed_state_missing_unit={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
            stream.full_enumeration_count_line_prefix(),
            tally.states,
            tally.states == closed_form,
            tally.violations,
            tally.ignored_violations,
            tally.failed_states,
            tally.root_persisted_states,
            tally.journal_differing_states,
            tally.verification_ran_states,
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit,
            tally.states_by_publish_text(),
            tally.checker_counts_by_invariant()
        );
        assert_eq!(
            tally.states, closed_form,
            "{stream:?}：闭式 1 + (2^(2 × {units_written}) − 1) + 3 + 1 + (3² − 1)"
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
