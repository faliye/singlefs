//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 43 行取丁-defer（被攻过一轮；
//! `research/prompts/abandoned-floor-r1-main-verification.md` M1）：I-3.1（已分配统计对得上） 的「实际遍历」并上 F_生效 之下、没被抛弃的根
//! 引用着、候选集那一遍没走到、最新根的账里是已释放且释放代高于 F_生效 的槽（还在 defer 里）。
//!
//! 历史只靠崩溃恢复造被抛弃实例（调查员报告 `research/prompts/closeout-recheck-2026-09-28/row43-investigator-report.md` 的最小复现）：
//! A（txg 3）→ 覆盖写 B（4）、C（5）→ C 的系统配置轮换没落盘就崩（轮换写全部退回崩溃前的字节，两槽照样自证得过）→ 可写挂载时 C 的根槽与
//! 数据单元一时读不出，落到 B、实例 2 → 再可写挂载一次（影子账看见 C）→ 覆盖写四次 → 抬 F 到 5。每块盘差的 10 槽释放代 6、
//! 只被 B（txg 4，F 之下）与被抛弃的 C 引用、还在 defer 里：改之前只有 F = 5 那一格 I-3.1 红（记账多于遍历），并上之后全绿。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs"]
mod scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green() {
    scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_past_the_gap_keeps_every_invariant_green`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_past_the_gap_keeps_every_invariant_green() {
    scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_past_the_gap_keeps_every_invariant_green(&mut PoolCheckerImages::left_to_the_checker_tier());
}
