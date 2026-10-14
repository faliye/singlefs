//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口，代码三方第二轮判决（`research/prompts/m2-wave2-code-r1-main-verification.md`）第二节第 1 行（Z1-d）：
//! 复用一个已回收的落点时跨度变了——两槽的单元落在一条跨 1 的已回收记录上，下一槽另一条跨 1 的已回收记录留着 ⇒ 盘上两条分配记录
//! 罩住同一个槽，下一次可写挂载重建分配器时在 `mark_allocated` 的断言上 panic（取号之前），池从此挂不上可写。
//! 改法：一次分配写下的记录罩住的槽上，别的已回收记录随这次分配删掉（`allocator.rs` 的 `make_room_for_record_on_device`）；
//! checker 新接 I-5.4（分配记录罩住的槽互不相交），每次发布之后判。
//!
//! 历史照攻方原样探针（`research/prompts/m2-wave2-code-r1-opus-model/opus_probe_pristine.rs` 的
//! `pristine_reuse_with_a_different_span_leaves_an_overlapping_record_and_the_next_mount_panics`，日志 `pristine-run.log`）：
//! 两块 4 GiB 内存盘，mkfs → 第一个文件 →「可写挂载 → 覆盖写 2 次」× 7 → 第 8 次可写挂载 → 覆盖写 1 次（txg 38，inode 叶容器
//! 两槽落在 50320，50320 与 50321 上各一条跨 1 的已回收记录）→ 第 9、10 次可写挂载。覆盖写的内容与写入时刻照探针取。
//! 改之前：txg 38 那次之后 50320 跨 2 与 50321 跨 1 两条记录重叠，第 9 次挂载在 `allocator.rs` 的 `mark_allocated` 断言上 panic。
//! 分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）每次发布多写几个节点，落点整体后移：同一条历史里换跨度的复用仍在 txg 38，
//! 落在 50332（50332 与 50333 上各一条跨 1 的已回收记录，草稿副本上沿这条历史逐次覆盖写查出来的）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/reused_allocation_record_overlap.rs"]
mod scenarios_of_reused_allocation_record_overlap;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reused_allocation_record_overlap::reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_reused_allocation_record_overlap::reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds,
    );
}
