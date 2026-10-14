//! 回退候选集取环里全部按现行那一版的实例表判仍然有效、txg ≥ F_生效、带文件的根（D16（发布语义） 已定项 1「回退候选集」；D23（journal 的角色与格式）
//! 已定项 14 的挂着时回退取同一个集合）。原来另有「txg 落在一圈窗口里」一格（用户 2026-10-03 定照 7-2 报告第四节做），用户 2026-10-04 定 RGall
//! 与回收门槛一起退回整个环（`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：回收门槛不再越过窗口外的有效根，
//! 它引用的单元都还在，它就是候选。
//!
//! 盘面同 `one_revolution_window_after_a_hole_in_the_root_ring.rs`：实例 1 在 24 槽的根环上发到 txg 30，txg 31 那次发布崩在「记录已持久、根槽没持久」，
//! 那一槽里留着上一圈的 txg 7 的根；可写挂载（实例 2）恢复由记录施加出 txg 31，写行 32、暖机 33、34。txg 7 的根读得出、按实例表有效、
//! txg ≥ F_生效（F 没抬过，是 0）、带文件；现行那一版 txg 34 的一圈窗口是 txg > 10，txg 7 在窗口外。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs"]
mod scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate::rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version()
{
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate::rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version,
    );
}
