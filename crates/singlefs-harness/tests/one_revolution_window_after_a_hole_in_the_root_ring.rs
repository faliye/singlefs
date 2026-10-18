//! 根环上有洞之后的回收门槛（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：2026-10-02 起取一圈窗口，
//! 用户 2026-10-04 定 RGall 退回整个环（`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。文件名留着窗口那一轮的叫法，
//! 盘面就是当初定窗口用的那一形。
//!
//! 盘面（与崩溃注入种子 0 第 135 段那一形同形，调查员交回那一条见 `records/2026-10-01-里程碑三形式化接入.md`「根环上的「后洞」」那一句）：实例 1 在 24 槽的根环上发到 txg 30，
//! txg 31 那次发布崩在「记录已持久、根槽没持久」（根槽与之后的系统配置轮换都没落：那一槽里留着上一圈的 txg 7 的根）。
//! 可写挂载（实例 2）恢复由记录施加出 txg 31 那一版，写行 txg 32、暖机 txg 33、34 盖掉 txg 8、9、10 的槽；之后覆盖写 txg 35、36、37。
//! txg 7 的根一直留在 txg 31 那一槽里，是环里最旧的有效根，而 txg 8–11 已经离开根环。
//!
//! - 分配器（进程内每记一条根）：门槛卡在 txg 7，txg 7 那一版引用的单元一个都不回收。照一圈窗口取时 txg 35 起 txg 7 在窗口外，
//!   门槛跟着窗口里最旧的有效根往前走，txg 7 引用、已释放的单元当场回收。
//! - 重新挂载（重建分配器）与抬 F（抬 F 那一遍的回收）同一个门槛：txg 7 的单元仍不回收。
//! - 池级 checker：txg 7 是候选、照走；只被 txg 8–11 那几版引用的单元由「由记录施加出来、根槽从没落盘的那一版」那一路走到
//!   （它们的记录还在 journal 环里），记账与遍历对得上。
//! - 洞里 txg 7 的根所在那一槽再挂那一刻读不出（txg 7 比 N − 3S 还旧）：读坏的槽在回收门槛里钉在它可能住过的最旧 txg 7
//!   （用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`），挂载时、进程内与抬 F 那一遍都不越过它。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/one_revolution_window_after_a_hole_in_the_root_ring.rs"]
mod scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(
) {
    scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated() {
    scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated()
{
    scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated(
) {
    scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated(
) {
    scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated(
) {
    scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated(&mut PoolCheckerImages::left_to_the_checker_tier());
}
