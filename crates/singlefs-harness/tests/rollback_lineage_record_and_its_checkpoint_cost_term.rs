//! 管理员回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14）：
//! 谱系容器（码 3 打包记录类型 5，容器号同墓碑规则：零根时 1）与谱系树根（码 2 层级 1）在回退那次写出、从盘上重建读得回，
//! 记录 (T_rb, T_old) 逐字段相等；写账另立一种 `LineageTreeNode`（用户 2026-10-02 定）；ckpt_cost 谱系那一项按一次谱系容器 COW
//! 连祖先算（容器 2 + 根 1 + 内部层数，用户 2026-10-02 定「乙」）。池是两块 4 GiB 盘。

mod common;

use common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, BuiltPool,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::{
    central_mapping_entries_of_one_lineage_container_cow,
    checkpoint_cost_of_one_lineage_container_cow, checkpoint_cost_of_the_version_to_build_on,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, MetadataBlocks,
};
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::lineage_tree::{LineageContainer, LineageContainerIndexInTree};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::LineageRecord;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, TransactionOutput, TransactionUnit};
use singlefs_core::unit::{PackedIdentity, PACKED_TYPE_LINEAGE};
use singlefs_core::write_accounting::{WriteCallsAndBytes, WrittenStructureKind};
use singlefs_format::TREE_IDENTIFIER_LINEAGE;

/// 第一个文件那一版之后覆盖写一次，交回第一个文件那一版（回退目标）。
fn pool_with_one_overwrite_after_the_first_file(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"overwritten content");
    (pool, first_file_version)
}

fn roll_back_to(pool: &mut BuiltPool, target: &TransactionOutput) {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
    )
    .expect("回退目标在候选集里");
}

fn rebuilt_from_disk(pool: &BuiltPool) -> TransactionOutput {
    match rebuild_version(
        &pool.memory_pool(),
        &pool.output.root,
        Some(pool.output.record.clone()),
    )
    .expect("回退之后那一版从盘上重建")
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("回退之后那一版有树"),
    }
}

fn violations(pool: &BuiltPool) -> Vec<String> {
    check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn lineage_records_of(version: &TransactionOutput) -> Vec<LineageContainer> {
    version
        .lineage_containers
        .iter()
        .map(|container| container.contents.clone())
        .collect()
}

/// 第一次回退（零根上）：谱系树开容器号 1、出生代 = 回退那次的 txg、类型 5，装一条 (T_rb = 回退那次的 txg, T_old = R_old 的 txg)；
/// 这次重写谱系容器与谱系树根各一个，写账记在 `LineageTreeNode` 那一种（两块盘各一份：4 次写、2 × (32768 + 16384) 字节）；
/// 从盘上重建读回的谱系容器逐字段相等；冷启动恢复的走读（映射条目数把谱系树的容器与根也算上）读回第一个文件；池级 checker 全绿。
#[test]
fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field() {
    let (mut pool, first_file_version) =
        pool_with_one_overwrite_after_the_first_file("rollback-appends-a-lineage-record");
    assert!(
        pool.output.lineage_containers.is_empty(),
        "回退之前谱系树零根"
    );
    roll_back_to(&mut pool, &first_file_version);
    let rollback_txg = pool.output.root.checkpoint_txg;
    let expected = vec![LineageContainer {
        identity: PackedIdentity {
            birth_tree: singlefs_core::address::TreeIdentifier(TREE_IDENTIFIER_LINEAGE),
            record_type: PACKED_TYPE_LINEAGE,
            container: 1,
            container_birth: rollback_txg,
        },
        records: vec![LineageRecord {
            rollback_txg,
            rollback_target_txg: first_file_version.root.checkpoint_txg,
        }],
    }];
    assert_eq!(
        (rollback_txg, lineage_records_of(&pool.output),),
        (
            CheckpointTxg(first_file_version.root.checkpoint_txg.0 + 2),
            expected.clone()
        ),
        "第一个文件那一版之后覆盖写一次、再回退：T_rb 比 T_old 大 2，容器号 1"
    );
    assert_eq!(
        pool.output
            .rewritten
            .iter()
            .filter(|role| matches!(
                role,
                TransactionUnit::LineageLeafContainer(_) | TransactionUnit::LineageRoot
            ))
            .copied()
            .collect::<Vec<_>>(),
        vec![
            TransactionUnit::LineageLeafContainer(LineageContainerIndexInTree(0)),
            TransactionUnit::LineageRoot,
        ],
        "回退那次重写谱系容器与谱系树根，先叶后根"
    );
    assert_eq!(
        pool.output.writes.of(WrittenStructureKind::LineageTreeNode),
        WriteCallsAndBytes {
            write_calls: 4,
            written_bytes: 2 * (32_768 + 16_384),
        },
        "谱系树那一种：两个单元各落两块盘"
    );
    assert_eq!(
        lineage_records_of(&rebuilt_from_disk(&pool)),
        expected,
        "从盘上重建读回的谱系容器与写出的逐字段相等"
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (pool.output.root.instance, rollback_txg),
            content: file_content(),
        },
        "冷启动恢复走到回退那一版，读回第一个文件"
    );
    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
}

/// 第二次回退追加进同一片开放容器（一片装 2039 条）：容器号仍 1、出生代仍是第一次回退那次，两条记录按先后排；之间的发布照抄谱系树。
#[test]
fn a_second_rollback_appends_to_the_same_open_lineage_container() {
    let (mut pool, first_file_version) =
        pool_with_one_overwrite_after_the_first_file("second-rollback-appends-to-the-container");
    roll_back_to(&mut pool, &first_file_version);
    let first_rollback = pool.output.clone();
    let after_the_overwrite =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"again");
    assert_eq!(
        lineage_records_of(&after_the_overwrite),
        lineage_records_of(&first_rollback),
        "不回退的发布照抄谱系树"
    );
    roll_back_to(&mut pool, &first_rollback);
    let second_rollback_txg = pool.output.root.checkpoint_txg;
    let containers = lineage_records_of(&rebuilt_from_disk(&pool));
    assert_eq!(
        (
            containers.len(),
            containers
                .first()
                .map(|container| container.identity.container),
            containers
                .first()
                .map(|container| container.identity.container_birth),
            containers
                .first()
                .map(|container| container.records.clone())
                .unwrap_or_default(),
        ),
        (
            1,
            Some(1),
            Some(first_rollback.root.checkpoint_txg),
            vec![
                LineageRecord {
                    rollback_txg: first_rollback.root.checkpoint_txg,
                    rollback_target_txg: first_file_version.root.checkpoint_txg,
                },
                LineageRecord {
                    rollback_txg: second_rollback_txg,
                    rollback_target_txg: first_rollback.root.checkpoint_txg,
                },
            ],
        ),
        "一片容器两条记录：(第一次回退, 第一个文件)、(第二次回退, 第一次回退)"
    );
    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
}

/// ckpt_cost 谱系那一项：谱系树零根（回退之前）与只有根（回退之后）都是 3 块（容器 2 + 根 1，内部 0 层），长高之后多一层多一块
/// （第一版的谱系树还不长高，长高那一格只在 `checkpoint_cost_of_one_lineage_container_cow` 的参数上取）。
/// 第一个文件那一版整个 ckpt_cost 的绝对值 = 分配记录树 529 + 中央映射树那一项 + 记账树节点 + 树表 1 + 谱系 3。
#[test]
fn the_lineage_term_of_the_checkpoint_cost_is_three_blocks_on_a_zero_root_and_on_a_root_only_tree()
{
    let (mut pool, first_file_version) =
        pool_with_one_overwrite_after_the_first_file("lineage-term-of-the-checkpoint-cost");
    let lineage_term_before = checkpoint_cost_of_one_lineage_container_cow(
        first_file_version.lineage_tree_internal_levels(),
    );
    roll_back_to(&mut pool, &first_file_version);
    let lineage_term_after =
        checkpoint_cost_of_one_lineage_container_cow(pool.output.lineage_tree_internal_levels());
    assert_eq!(
        (
            lineage_term_before,
            lineage_term_after,
            checkpoint_cost_of_one_lineage_container_cow(1),
        ),
        (MetadataBlocks(3), MetadataBlocks(3), MetadataBlocks(4)),
        "零根 3、只有根 3、长高一层 4"
    );
    assert_eq!(
        (
            first_file_version.accounting_tree.node_count(),
            checkpoint_cost_of_the_version_to_build_on(Some(&first_file_version), &pool.allocator),
        ),
        (1, MetadataBlocks(529 + 6 + 1 + 1 + 3)),
        "第一个文件那一版：分配记录树 529、中央映射树那一项 6、记账树 1、树表 1、谱系 3"
    );
    assert_eq!(
        pool.output.root.instance,
        InstanceGeneration(1),
        "回退在同一个实例里"
    );
}

/// ckpt_cost 中央映射树那一项的删插数连回退那次谱系容器与谱系树根进映射的删插一起算（用户 2026-10-02 定 Q-U「乙 一并算上」）：
/// 谱系零根（回退之前）与只有根（回退之后）两版都多算容器 1 把、根 1 把（[`central_mapping_entries_of_one_lineage_container_cow`]）。
/// 产品容量（叶 294、内部 143）下这两版那一项的切分次数不随删插数多 2 把而变（529 + 1 + 2 把插进一片叶，切 4 次都装得下），量不出这几把；
/// 中央映射树节点容量按只供测试的开关压到叶 4、内部节点 4（每切一次至少再插 2 把），两版的 ckpt_cost 绝对值钉在草稿探针现跑的数上。
#[test]
fn the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root_and_on_a_root_only_tree(
) {
    let (mut pool, first_file_version) = pool_with_one_overwrite_after_the_first_file(
        "central-mapping-term-counts-the-lineage-entries",
    );
    let capped = CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: CodeTwoTreeNodeCapacity {
            leaf_entries: 4,
            internal_entries: 4,
        },
        central_mapping: CodeTwoTreeNodeCapacity {
            leaf_entries: 4,
            internal_entries: 4,
        },
    };
    let zero_root = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        Some(&first_file_version),
        &pool.allocator,
        capped,
    );
    roll_back_to(&mut pool, &first_file_version);
    let root_only = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        Some(&pool.output),
        &pool.allocator,
        capped,
    );
    assert_eq!(
        (
            central_mapping_entries_of_one_lineage_container_cow(
                first_file_version.lineage_tree_internal_levels()
            ),
            central_mapping_entries_of_one_lineage_container_cow(
                pool.output.lineage_tree_internal_levels()
            ),
        ),
        (2, 2),
        "零根与只有根：谱系容器 1 把、谱系树根 1 把"
    );
    assert_eq!(
        (zero_root, root_only),
        (
            MetadataBlocks(CHECKPOINT_COST_OF_THE_ZERO_ROOT_VERSION_UNDER_THE_CAPPED_MAPPING),
            MetadataBlocks(CHECKPOINT_COST_OF_THE_ROOT_ONLY_VERSION_UNDER_THE_CAPPED_MAPPING)
        ),
        "叶 4、内部 4 的中央映射树：两版 ckpt_cost 的绝对值（删插数含谱系那 2 把）"
    );
}

/// 草稿探针现跑的数（三·2 第 8 位实现员副本上）：叶 4、内部 4 时两版都是 1062（叶 3 / 6 / 10 时 1064 / 803 / 670，产品容量 540）。
const CHECKPOINT_COST_OF_THE_ZERO_ROOT_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1062;
const CHECKPOINT_COST_OF_THE_ROOT_ONLY_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1062;
