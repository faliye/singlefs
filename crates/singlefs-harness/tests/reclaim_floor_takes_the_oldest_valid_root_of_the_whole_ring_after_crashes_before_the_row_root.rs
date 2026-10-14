//! 回收门槛取「环里最旧有效根」（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」；用户 2026-10-04 定 RGall，
//! `research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1：回收门槛与原来同取一圈窗口的另外五处——checker 的候选过滤、
//! checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛、影子账的候选过滤、回退候选集、抬 F 的上限——一起退回整个环）。
//!
//! 盘面（S = 4 的 12 槽环，攻方腿 `research/prompts/m3-reclaim-window-r1-opus-output.md` 那一形；原型
//! `research/prompts/m3-reclaim-window-r1-opus-model/probe/main.rs` 的 `crash_a_mount_before_its_row_root` 在 `common_root_ring_holes` 里另写了一份）：
//! 实例 1 覆盖写到 txg 24；连着几次可写挂载都崩在写行那次根槽写之前（K1 那一类洞：写行那次的单元与记录已持久、根槽没持久，每崩一次 txg 跳一个，
//! 洞那一槽里留着实例 1 上一圈的根）；再一次可写挂载做完（写行、暖机落满）。
//!
//! - 连崩 11 次：写行那次的根一记，进程内的锚跳过 11 个没有根的 txg，实例 1 留在洞里的根（txg 13–23）全落到一圈窗口外。照一圈窗口取门槛时
//!   它们引用、已释放的单元当场回收（攻方量到挂载之后每盘已分配 136 → 20）；取整个环时一个都不回收。
//! - 连崩 1 次、只改分配器那一臂（攻方 `arm-RGalloc.diff`）：分配器取整个环、checker 的候选过滤与丁-defer 门槛仍取窗口，
//!   挂载之后池级 checker 判 I-3.1 红（攻方 S = 4 每段 112 / 120）；六处一起改之后不红。
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
#[path = "common_images_for_the_checker_tier/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs"]
mod scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated(
) {
    scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring(
) {
    scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring(&mut PoolCheckerImages::left_to_the_checker_tier());
}
