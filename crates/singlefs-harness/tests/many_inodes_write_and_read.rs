//! 里程碑「覆盖写、释放、回退与复用」并行线三（多个文件：多个 inode，元数据按「个」量、没有目录）的验收。
//!
//! 一次发布建 N 个 inode（N 跨过一容器 233 条那个门槛）：inode 号从记账里的水位起连着发
//! （D5（快照 / 空间记账机制） 已定项 4 第 12 项），记录落在最右那片叶、满 233 条就在末尾分裂
//! （D8（核心索引结构） 已定项 6，`singlefs_core::inode_tree`）。
//!
//! **这一份是 C116（叶容器的分裂 / 合并纪律没有会失败的检查） 的「恢复加 oracle」那一路**：
//! [09-inode树结构.md](../../singlefs-checker/invariants/09-inode树结构.md) 逐字把分裂的三条写路径纪律排除在 I-9（inode 树结构） 之外
//! （「分裂 / 合并的身份传递……是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放」），
//! 所以这里的期望态**带着分裂前的身份**：分裂之前那片容器的四段身份（出生树、打包记录类型、容器号、容器出生代）
//! 与它那 233 条记录，是从新池新建文件那一版与建 inode 那次发布之前的内存态里取的，
//! 冷启动重建出来的左半要与它逐项相同。纯函数那一路的单测在 `singlefs-core` 的 `inode_tree` 模块里。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/many_inodes_write_and_read.rs"]
mod scenarios_of_many_inodes_write_and_read;

use common::{build_pool, disk_snapshot, FIXED_WRITE_TIME_SECONDS};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_many_inodes_write_and_read::create_inodes;
use singlefs_core::inode_tree::leaf_containers_one_root_node_holds;
use singlefs_core::transaction::PublishError;
use singlefs_format::INODE_LEAF_RECORDS;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_many_inodes_write_and_read::creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold(
) {
    scenarios_of_many_inodes_write_and_read::creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_many_inodes_write_and_read::creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one() {
    scenarios_of_many_inodes_write_and_read::creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 一次建的 inode 多到叶容器数超过一个码 2 根装得下的 135 片：inode 树要在根之下再长一层内部节点，
/// 而「内部节点溢出之后树怎么长高」仓里没有条款（里程碑并行线三把「扇出 135 ⇒ 树高 2」整句标成**预想**）⇒
/// 在**任何落盘动作之前**返回说清哪条条款没定的错误成员，盘上逐字节不变。
#[test]
fn more_leaf_containers_than_one_root_node_holds_is_refused_before_anything_reaches_the_disk() {
    let mut pool = build_pool("parallel-line-three-too-many-containers");
    let capacity = u64::try_from(leaf_containers_one_root_node_holds()).expect("135");
    // 树里已有 1 条 ⇒ 再建 capacity × 233 条正好多出一片容器。
    let new_inode_count = capacity * INODE_LEAF_RECORDS;

    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let allocation_records_before = pool.allocator.records().len();
    let refusal = create_inodes(&mut pool, new_inode_count, FIXED_WRITE_TIME_SECONDS + 120)
        .expect_err("第 136 片容器今天装不下");

    let PublishError::InodeTreeWriteRefused(refused) = refusal else {
        panic!("该报 inode 树写入被拒，实际 {refusal:?}");
    };
    assert_eq!(
        refused,
        singlefs_core::inode_tree::InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
            containers: usize::try_from(capacity).expect("135") + 1,
            capacity: usize::try_from(capacity).expect("135"),
        }
    );
    let after = disk_snapshot(&pool.memory_pool(), &pool.stream);
    assert!(
        after == before,
        "盘上逐字节不变：系统配置槽、根环里的根、录制流步数都没动"
    );
    assert_eq!(
        pool.allocator.records().len(),
        allocation_records_before,
        "分配器一条记录都没动"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_many_inodes_write_and_read::publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish(
) {
    scenarios_of_many_inodes_write_and_read::publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish(&mut PoolCheckerImages::left_to_the_checker_tier());
}
