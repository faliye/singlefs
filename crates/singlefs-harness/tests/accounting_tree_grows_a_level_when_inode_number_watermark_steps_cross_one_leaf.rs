//! inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」，第 49 批 W3）多到一片记账树叶装不下的那次发布：
//! 记账树从一个节点长成根加两片叶。两件事各一条用例：
//! - ckpt_cost 记账树那一项（D28（挂载期承诺量） 已定项 4「记账树每发布的节点数」）按这次发布写完之后那棵树的节点数算，不是上一版的；
//! - 理想模型数记账行时把台阶行算进去、带记账树节点数的上界，在台阶数跨叶的历史上与实现在同一步长一层、那次记账树占的槽数相同。
//!
//! 池是两块 4 GiB 盘：每次写记账行的发布写池级 3 行、每块盘 6 行，加照抄的台阶行；第一个文件那次留第 1 级台阶，之后每建一个 inode 再留一级，
//! 第 k 个 inode 那次照抄 k 级、写 15 + k 行。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::admission::{
    accounting_tree_nodes_after_the_publish_built_on,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, MetadataBlocks,
    SpaceAdmission,
};
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::transaction::{
    publish_new_inodes, CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, PoolWriter,
    TransactionOutput, TransactionUnit,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    StepOutcome, StepPosition,
};
use singlefs_harness::model::ModelUnitRole;
use singlefs_harness::SharedStream;

/// 两块盘每次写记账行的发布写的池级 3 行加每块盘 6 行。
const ACCOUNTING_ROWS_WITHOUT_STEPS_ON_TWO_DEVICES: u64 = 15;

/// 产品容量下记账树一片叶装的行数（(16384 − 头 159) ÷ 34）。
const ACCOUNTING_ROWS_ONE_LEAF_HOLDS: u64 = 477;

/// 产品容量两块盘时第一个装不下一片叶的那次：照抄 463 级台阶、写 478 行（D5 已定项 2 射程「第 463 行起 +2」）。
const INODE_CREATIONS_UP_TO_THE_FIRST_SPLIT_OF_THE_ACCOUNTING_TREE: u64 =
    ACCOUNTING_ROWS_ONE_LEAF_HOLDS - ACCOUNTING_ROWS_WITHOUT_STEPS_ON_TWO_DEVICES + 1;

/// 在会话外经写入口建一个 size 0 的 inode（`publish_new_inodes`，count 1），节点容量按 `capacities` 装，接在 `pool.output` 后面。
fn create_one_inode(
    pool: &mut BuiltPool,
    capacities: CodeTwoTreeNodeCapacities,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_code_two_tree_node_capacities(capacities);
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("建空 inode");
    pool.output = output.clone();
    output
}

/// 一次发布重写的记账树节点数（根兼叶，或根与根之下的每个节点）。
fn accounting_tree_nodes_written_by(output: &TransactionOutput) -> usize {
    output
        .rewritten
        .iter()
        .filter(|role| {
            matches!(
                role,
                TransactionUnit::AccountingTree
                    | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            )
        })
        .count()
}

/// 记账树叶压到 16 行、内部节点 4 个孩子（中央映射树照产品容量）：第一个文件那一版（txg 3）15 行、自己那一行是第 1 级台阶；
/// 建一个 inode（txg 4）照抄 1 级、写 16 行，仍一个节点；再建一个（txg 5）照抄 2 级、写 17 行，一片叶装不下，长成根加两片叶 3 个节点、
/// 三个都是这次写的。接在 txg 4 那一版后面的 ckpt_cost 记账树那一项按这次发布写完之后那棵树算 3，不小于 txg 5 那次实际写的 3 个；
/// 整个 ckpt_cost 的绝对值钉住（分配记录树 529、中央映射树那一项、记账树 3、树表 1、谱系 3）。
/// 判别力：ckpt_cost 的记账树那一项退回拿 txg 4 那一版的节点数 1 算时，红在绝对值那一句（540，少 2）；规划下一次发布的行时不照抄台阶行
/// （只剩 15 行、一个节点），那一项算成 1，「不小于实际写的」那一句先红。
#[test]
fn the_checkpoint_cost_counts_the_accounting_nodes_after_the_publish_whose_watermark_step_crosses_one_leaf(
) {
    let capped = CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: CodeTwoTreeNodeCapacity {
            leaf_entries: 16,
            internal_entries: 4,
        },
        central_mapping: MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format(),
    };
    let mut pool =
        build_pool("checkpoint-cost-counts-the-accounting-nodes-after-a-step-crosses-a-leaf");
    let sixteen_rows = create_one_inode(&mut pool, capped);
    let accounting_term_built_on_sixteen_rows = accounting_tree_nodes_after_the_publish_built_on(
        &sixteen_rows,
        capped.of_tree(MultiLevelCodeTwoTree::Accounting),
    );
    let checkpoint_cost_built_on_sixteen_rows =
        checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
            Some(&sixteen_rows),
            &pool.allocator,
            capped,
        );
    let seventeen_rows = create_one_inode(&mut pool, capped);
    assert_eq!(
        (
            sixteen_rows.accounting_entries.len(),
            sixteen_rows.inode_number_watermark_steps().len(),
            sixteen_rows.accounting_tree.node_count(),
            seventeen_rows.accounting_entries.len(),
            seventeen_rows.accounting_tree.node_count(),
            accounting_tree_nodes_written_by(&seventeen_rows),
        ),
        (16, 2, 1, 17, 3, 3),
        "txg 4 那一版 16 行（2 级台阶）一个节点；txg 5 那次 17 行，根加两片叶 3 个节点都是这次写的"
    );
    assert!(
        accounting_term_built_on_sixteen_rows
            >= u64::try_from(accounting_tree_nodes_written_by(&seventeen_rows))
                .expect("节点数装得进 u64"),
        "ckpt_cost 记账树那一项 {accounting_term_built_on_sixteen_rows} 不小于台阶跨叶那次实际写的记账树节点数 {}",
        accounting_tree_nodes_written_by(&seventeen_rows)
    );
    assert_eq!(
        (
            accounting_term_built_on_sixteen_rows,
            checkpoint_cost_built_on_sixteen_rows
        ),
        (
            3,
            MetadataBlocks(
                CHECKPOINT_COST_BUILT_ON_SIXTEEN_ROWS_UNDER_A_SIXTEEN_ROW_ACCOUNTING_LEAF
            )
        ),
        "接在 16 行那一版后面：记账树那一项 3、整个 ckpt_cost 的绝对值"
    );
}

/// 副本上现跑的数（第 49 批 W3 补做）：529 + 中央映射树那一项 6 + 记账树 3 + 树表 1 + 谱系 3，第 50 批加退役 3 = 545；
/// 记账树那一项拿上一版的节点数 1 算时是 543。
const CHECKPOINT_COST_BUILT_ON_SIXTEEN_ROWS_UNDER_A_SIXTEEN_ROW_ACCOUNTING_LEAF: u64 = 545;

/// 一次发布写的记账树单元（根兼叶，或根与根之下的每个节点）每盘占几个槽：按交回的单元字节数数。
fn accounting_tree_slots_written_by(output: &TransactionOutput) -> u64 {
    output
        .units
        .iter()
        .filter(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::AccountingTree
                    | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            ) && output.rewritten.contains(&unit.identity)
        })
        .map(|unit| u64::try_from(unit.bytes.len()).expect("单元字节数装得进 u64") / SLOT_BYTES)
        .sum()
}

/// 产品容量两块盘、跨过第一次分裂之后再多建几个：对拍历史在分裂之后照常往下比。
const INODE_CREATIONS_PAST_THE_FIRST_SPLIT: u64 = 2;

/// 一版的记账树：行数、节点数（模型那一侧是上界）、写它的那次发布记账树每盘占几个槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AccountingTreeOfAVersion {
    rows: u64,
    nodes: u64,
    slots_written: u64,
}

/// 产品容量（记账树叶 477 行）、同一组台阶数，模型与实现在同一步判记账树长一层、那次记账树占的槽数相同：
/// - 实现：第一个文件之后经写入口一次建一个 inode，第 462 个那次写 477 行、记账树一个节点占 1 槽；第 463 个那次写 478 行、
///   长成根加两片叶，3 个节点都是这次写的、占 3 槽；
/// - 模型：对拍历史（起点第一个文件，unlink 之后一次建一个 inode；unlink 不留台阶，第 k 个 inode 那次同样照抄 k 级）每一步都与实现对上、
///   跑完不停；第 462 个那一版 477 行、记账树节点数上界 1，第 463 个那一版 478 行、上界 3（`model::accounting_tree_nodes_upper_bound`：
///   一片叶切一次、根切一次），写记账树占 3 槽。分裂之后再建 2 个，对拍照常往下比（胶水不再在根之下的记账树节点上停）。
///
/// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是模型的行数、节点数与占槽，不判盘上。
/// 判别力：模型数记账行时不算台阶行，第 463 个那一版模型 15 行、一个节点；多数一级，第 462 个那一版就判长层；
/// 记账树节点数的上界不随行数长（照抄上一版的），第 463 个那一版模型 1 个节点、占 1 槽；切叶次数的上界不减一，第 462 个那一版就是 3 个节点。
#[test]
fn the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step() {
    let inode_creations = INODE_CREATIONS_UP_TO_THE_FIRST_SPLIT_OF_THE_ACCOUNTING_TREE
        + INODE_CREATIONS_PAST_THE_FIRST_SPLIT;
    let mut pool = build_pool("model-and-implementation-grow-the-accounting-tree-at-the-same-step");
    let mut implementation_after_each_creation = Vec::new();
    for _ in 0..inode_creations {
        let created = create_one_inode(&mut pool, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
        implementation_after_each_creation.push(AccountingTreeOfAVersion {
            rows: u64::try_from(created.accounting_entries.len()).expect("行数装得进 u64"),
            nodes: u64::try_from(created.accounting_tree.node_count()).expect("节点数装得进 u64"),
            slots_written: accounting_tree_slots_written_by(&created),
        });
    }

    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(std::iter::repeat_n(
        HistoryOperation::CreateInode,
        usize::try_from(inode_creations).expect("几百装得进 usize"),
    ));
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let mut model_after_each_creation = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &SharedStream::new(),
        &mut |observation| {
            let is_an_inode_creation = matches!(
                observation.position,
                StepPosition::Operation(index) if index >= 1
            );
            if let (true, Some(current)) = (
                is_an_inode_creation,
                observation.model.current_version_in_the_session(),
            ) {
                model_after_each_creation.push(AccountingTreeOfAVersion {
                    rows: current.accounting_rows,
                    nodes: current.accounting_tree_nodes_upper_bound,
                    slots_written: current.accounting_tree_nodes_upper_bound
                        * ModelUnitRole::AccountingTree.span_in_slots(),
                });
            }
        },
    );
    let published_steps = run
        .outcomes
        .iter()
        .filter(|outcome| {
            matches!(
                outcome,
                StepOutcome::Applied(AppliedEffect::Published { .. })
            )
        })
        .count();
    assert_eq!(
        (
            &run.ending,
            published_steps,
            model_after_each_creation.len()
        ),
        (
            &HistoryEnding::Completed,
            1 + usize::try_from(inode_creations).expect("几百装得进 usize"),
            usize::try_from(inode_creations).expect("几百装得进 usize"),
        ),
        "模型：unlink 与每一次建 inode 都与实现对上，跨过记账树分裂照常跑完"
    );

    let first_split = INODE_CREATIONS_UP_TO_THE_FIRST_SPLIT_OF_THE_ACCOUNTING_TREE;
    let at_creation = |after_each_creation: &[AccountingTreeOfAVersion], creation: u64| {
        after_each_creation[usize::try_from(creation - 1).expect("几百装得进 usize")]
    };
    let one_node = AccountingTreeOfAVersion {
        rows: ACCOUNTING_ROWS_ONE_LEAF_HOLDS,
        nodes: 1,
        slots_written: 1,
    };
    let root_and_two_leaves = AccountingTreeOfAVersion {
        rows: ACCOUNTING_ROWS_ONE_LEAF_HOLDS + 1,
        nodes: 3,
        slots_written: 3,
    };
    assert_eq!(
        (
            at_creation(&implementation_after_each_creation, first_split - 1),
            at_creation(&implementation_after_each_creation, first_split),
        ),
        (one_node, root_and_two_leaves),
        "实现：第 462 个 inode 那次 477 行一个节点占 1 槽，第 463 个那次 478 行、根加两片叶都是这次写的、占 3 槽"
    );
    assert_eq!(
        (
            at_creation(&model_after_each_creation, first_split - 1),
            at_creation(&model_after_each_creation, first_split),
        ),
        (
            at_creation(&implementation_after_each_creation, first_split - 1),
            at_creation(&implementation_after_each_creation, first_split),
        ),
        "模型：第 462、463 个 inode 那两版的行数、记账树节点数（上界）与写记账树占的槽数都与实现相同"
    );
    let first_growth = |after_each_creation: &[AccountingTreeOfAVersion]| {
        after_each_creation
            .iter()
            .position(|tree| tree.nodes > 1)
            .map(|index| index + 1)
    };
    assert_eq!(
        (
            first_growth(&model_after_each_creation),
            first_growth(&implementation_after_each_creation)
        ),
        (
            Some(usize::try_from(first_split).expect("几百装得进 usize")),
            Some(usize::try_from(first_split).expect("几百装得进 usize"))
        ),
        "模型与实现都是第 463 个 inode 那次第一次长层"
    );
}
