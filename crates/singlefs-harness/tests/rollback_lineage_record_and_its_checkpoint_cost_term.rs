//! 管理员回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14）：
//! 谱系容器（码 3 打包记录类型 5，容器号同墓碑规则：零根时 1）与谱系树根（码 2 层级 1）在回退那次写出、从盘上重建读得回，
//! 记录 (T_rb, T_old, H_cur) 逐字段相等（H_cur 是 cur 那一版的 inode 号水位，W3 第 49 批）；写账另立一种 `LineageTreeNode`（用户 2026-10-02 定）；ckpt_cost 谱系那一项按一次谱系容器 COW
//! 连祖先算（容器 2 + 根 1 + 内部层数，用户 2026-10-02 定「乙」）。池是两块 4 GiB 盘。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rollback_lineage_record_and_its_checkpoint_cost_term.rs"]
mod scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::{
    pool_with_one_overwrite_after_the_first_file, roll_back_to,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::admission::{
    central_mapping_entries_of_one_lineage_container_cow,
    checkpoint_cost_of_one_lineage_container_cow, checkpoint_cost_of_the_version_to_build_on,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, MetadataBlocks,
};
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree};

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_appends_one_lineage_record_that_reads_back_field_by_field`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_appends_one_lineage_record_that_reads_back_field_by_field,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_second_rollback_appends_to_the_same_open_lineage_container`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_second_rollback_appends_to_the_same_open_lineage_container() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_second_rollback_appends_to_the_same_open_lineage_container,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target,
    );
}

/// ckpt_cost 谱系那一项：谱系树零根（回退之前）与只有根（回退之后）都是 3 块（容器 2 + 根 1，内部 0 层），长高之后多一层多一块
/// （第一版的谱系树还不长高，长高那一格只在 `checkpoint_cost_of_one_lineage_container_cow` 的参数上取）。
/// 第一个文件那一版整个 ckpt_cost 的绝对值 = 分配记录树 529 + 中央映射树那一项 + 记账树节点 + 树表 1 + 谱系 3 + 退役 3
/// （退役那一项是判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2「两样都做」加的）。
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
        (1, MetadataBlocks(529 + 6 + 1 + 1 + 3 + 3)),
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
/// 记账树照产品容量（ckpt_cost 记账树那一项按传进去的容量规划这次发布之后的树，第 49 批 W3 补做）：压到叶 4 时两版各 15 行的记账树
/// 那一项跟着变成多片叶，量的就不只是谱系那几把了。
#[test]
fn the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root_and_on_a_root_only_tree(
) {
    let (mut pool, first_file_version) = pool_with_one_overwrite_after_the_first_file(
        "central-mapping-term-counts-the-lineage-entries",
    );
    let capped = CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: MultiLevelCodeTwoTree::Accounting.node_capacity_of_the_node_format(),
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

/// 草稿探针现跑的数（三·2 第 8 位实现员副本上）：叶 4、内部 4 时两版都是 1062（叶 3 / 6 / 10 时 1064 / 803 / 670，产品容量 540）；
/// 第 50 批加了退役那一项（容器 2 + 根 1 块，它在中央映射树里多删插的 2 把在叶 4 下又多切出 3 个节点），第 50 批实现员副本上现跑是 1068。
const CHECKPOINT_COST_OF_THE_ZERO_ROOT_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1068;

const CHECKPOINT_COST_OF_THE_ROOT_ONLY_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1068;
