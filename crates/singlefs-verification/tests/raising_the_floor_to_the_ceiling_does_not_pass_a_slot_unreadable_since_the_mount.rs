//! 准入抬 F 抬到上限时，挂载那一刻读不出或读得出而自证不过、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
//! （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行；RS3 与 K2 两类都并：
//! `research/prompts/m3-root-ring-code-r2-main-verification.md` 第四节 RS3 与第六节、
//! `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K2 与第六节，`k2-zeros` 那一形）。
//! 上限本身取环里全部有效根（RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。
//! 盘上读回全零的根槽同读坏的槽一样并进上限（D16（发布语义） 已定项 1「抬 F 的上限」那一行
//! ）：一槽从挂载起读回全零、而盘上住着一条非空有效根时 F 不越过它（没洞的那一份钉着）。回收门槛里它照「自证不过」那一类
//! 钉在可能住过的最旧 txg：抬 F 之前那一槽里那条根引用的单元不回收。
//!
//! 两份盘面（S = 4 的 12 槽环）：
//! - 有洞的那一份（攻方原型那一形）：实例 1 覆盖写到 txg 17；三次可写挂载崩在写行那次根槽写之前（洞 18、19、20：那几槽里留着实例 1 上一圈的
//!   txg 6、7、8）；再一次可写挂载写下 X = 21（写行那一版的根）再空发布两次，正常退出；第四次可写挂载之后空发布推到 29，准入抬 F 抬到上限。
//!   RGall 之后洞里那三条旧根是有效根、非空：第 4 新的不同状态落在 txg 6，上限 6，X 读不读得出都一样。
//! - 没洞的那一份：实例 1 覆盖写到 X = 17，进程退出；可写挂载起 X 那一槽读坏，写行、暖机之后空发布推到 28（X 成了环里最旧的根）。
//!   读得出的有效根是 18–28：最旧的 18 跟「没有这两棵树」比、算非空，其余都空，照读得出的根算上限是 18；X 那一槽读不出或读回全零时按它能住的
//!   最大 txg 17 并进来，上限 17。X 读得出时上限也是 17（X 是最旧的有效根）。
mod common;
mod common_images_for_the_given_states_engine;
mod common_root_ring_holes;
mod common_shared_by_harness_scenarios;
#[path = "common_images_for_the_given_states_engine/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs"]
mod scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount;

use common_images_for_the_given_states_engine::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount() {
    scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x() {
    scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes(
) {
    scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount() {
    scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_reading_back_zeros_since_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn admission_raising_the_floor_without_holes_stops_at_a_slot_reading_back_zeros_since_the_mount() {
    scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_reading_back_zeros_since_the_mount(&mut PoolCheckerImages::left_to_the_given_states_engine());
}
