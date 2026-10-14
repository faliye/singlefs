//! 准入抬 F 那一遍在一圈窗口最旧端有洞的根环上（D16（发布语义） 已定项 1；`research/prompts/m3-root-ring-code-r1-main-verification.md`
//! 第二节 K2-b、K2-b2 两格，攻方 `raise_floor_hidden_slot.rs`）。
//!
//! 盘面（S = 4 的 12 槽环）：实例 1 覆盖写到 txg `last`；之后几次可写挂载都崩在写行那次根槽写之前（写行那次的单元与记录已持久、根槽没持久：
//! 那一槽里留着上一圈的根，出了窗口），每次一个洞；再一次可写挂载写行那一版的根 X 落盘、接着两次空发布，正常退出。
//! 第四次可写挂载之后推空发布到锚 = 第一个洞 + 3S − 1（洞成了窗口最旧的几格），准入抬 F。
//!
//! - 上限只取窗口里的有效根（B2，用户 2026-10-03 定）：洞那几槽里上一圈的旧根在窗口外，窗口已许可它们引用的单元被复用；
//!   改之前上限要读它们的树表、读到被复用的槽，准入抬 F 被拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（攻方 `rf-14-none.out`）。
//! - 回收门槛把挂载时读不出的那一槽并进窗口里最旧有效根（RR4，攻方 B1）：X 那一槽第四次挂载起一直读不出，根环表按它能住的最大 txg 记着；
//!   改之前抬 F 那一遍只在读得出的根上取门槛，门槛越过 X、回收了 X 引用的槽，记账比遍历少（攻方 `rfo3-17-persist.out`：抬完 I-3.1 红）。
//!
//! 用户 2026-10-04 定 RGall 之后（上限与回收门槛都取整个环）：洞那几槽里的旧根是有效根，上限与门槛都由它们定，
//! 分不出上面两样改之前的做法（B2、RR4 两行变异挪走或拿掉，实现员报告）。同日又定读坏的槽在回收门槛里钉在它可能住过的最旧 txg
//! （条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：X 那一槽一直读不出时钉在 9（它在 12 槽环上可能住过的最旧 txg），
//! 比盘上环里最旧的有效根旧，池级 checker 读盘上完好的 X、按它算门槛，I-3.1 在记账多的一边红，直到这个进程写掉那一槽（第二条用例钉这个代价）。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
#[path = "common_images_for_the_checker_tier/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs"]
mod scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window(
) {
    scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held(
) {
    scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held(&mut PoolCheckerImages::left_to_the_checker_tier());
}
