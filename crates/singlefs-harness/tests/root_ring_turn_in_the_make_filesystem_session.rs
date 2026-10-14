//! mkfs 同一个进程那条会话（`common::build_pool`：取号、暖机、第一个文件，之后一次挂载都不做）里连着覆盖写、转过根环：
//! 分配器装着 mkfs 刚写下的根环（C551（`build_pool` 的分配器没有根环表）；用户 2026-10-05 定 core 在没装根环表而记根时断言），
//! 环转过之后按可再分配谓词回收（D16（发布语义） 已定项 1：已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)），池级 checker 一条不红——
//! I-3.1（已分配统计对得上）是分配器不装根环表、一次不回收时红的那一条（E171 流三与 C551 的连续覆盖写都是这一形）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/root_ring_turn_in_the_make_filesystem_session.rs"]
mod scenarios_of_root_ring_turn_in_the_make_filesystem_session;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_turn_in_the_make_filesystem_session::overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_turn_in_the_make_filesystem_session::overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant,
    );
}
