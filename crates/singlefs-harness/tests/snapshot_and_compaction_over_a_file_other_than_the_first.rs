//! 快照与整理搬迁对第一个文件之外的文件（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 2 条；
//! 正推腿 `m3-close-w5-code-r1-sonnet-output.md` W5 节的缺口 C、D）：
//! - 快照下覆盖写别的号：被活快照引用的那几个单元（它的数据单元与下段节点，生在建快照之前）进 deadlist、映射条目留存，不进这次的释放
//!   （D5（快照 / 空间记账机制） 已定项 19「birth ≤ p：进 deadlist」；`transaction::asks_the_snapshot_lifetime_judgement` 加的两个成员）。
//! - 整理的一批搬别的号的数据单元：内存里那一版的指针换成映射位置（`transaction::take_the_mapping_positions_over_the_location_hints` 新加的那一段，
//!   C417 第一版政策「随重写刷新」），源落点两盘都清零之后照样读得回，之后一次写第一个文件重写上段时装的是映射位置。
//!
//! 都在同一个进程里直调发布入口；期望值从被测之前那一版现算（映射 key 与它的落点）。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/snapshot_and_compaction_over_a_file_other_than_the_first.rs"]
mod scenarios;

#[test]
fn overwriting_another_file_under_a_live_snapshot_puts_its_replaced_units_on_the_deadlist() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::overwriting_another_file_under_a_live_snapshot_puts_its_replaced_units_on_the_deadlist);
}

#[test]
fn a_relocation_batch_moving_the_data_unit_of_another_file_takes_its_mapping_position_over_the_hint(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::a_relocation_batch_moving_the_data_unit_of_another_file_takes_its_mapping_position_over_the_hint);
}
