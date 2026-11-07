//! 取号时补写两槽全废的盘（丙'，C463（一块盘的系统配置两份全废之后静默成功）；开例外，四件闸，闸三加单元区扫描、环槽读错就不补写）。
//! 闸都过才补写：本盘带所选那一版的全部单元；根槽那一半不缺；这块盘根环与 journal 环里没有比所选那一版新的根与记录；
//! 单元区里没有写序实例大于所选那一版实例的单元；根环与 journal 环里没有读错的槽；补写失败按甲' 每块盘回卷一次（取号写那一份，
//! `acquisition_writes_two_rounds_and_rolls_each_device_back_once.rs`）。有一件不过照今天拒：取号之前的逐盘核报
//! `SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`，盘上逐字节不变。
//!
//! 都在单元区 384 槽那一档的两块内存小盘上（`tests/common_admission_pool/mod.rs`），两槽全废的都是盘 1。所选那一版 V 多数用例是第一个文件之后
//! 覆盖写一次的 txg 4（根落区域 1 的盘 1）；空盘与别的池的盘那两条换掉整块盘 1，用覆盖写两次的 txg 5（根落区域 2 的盘 0），盘 1 换掉之后择根、
//! 重放与见证都不缺东西，拒才落在补写闸上。

//! 这里只核实现行为；对应场景函数（`run_the_scenario_where_<用例名>`）交出的镜像由给定状态引擎两段判。

mod common;
mod common_images_for_the_given_states_engine;
mod common_shared_by_harness_scenarios;
mod common_unwitnessed_publish;
#[path = "common_images_for_the_given_states_engine/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs"]
mod scenarios;

#[test]
fn a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green(
) {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green);
}

#[test]
fn a_blank_device_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write() {
    scenarios::run_the_scenario_where_a_blank_device_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write();
}

#[test]
fn a_device_of_another_pool_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write() {
    scenarios::run_the_scenario_where_a_device_of_another_pool_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write();
}

#[test]
fn a_cloned_device_carrying_a_newer_root_and_record_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write(
) {
    scenarios::run_the_scenario_where_a_cloned_device_carrying_a_newer_root_and_record_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write();
}

#[test]
fn a_device_whose_journal_ring_holds_a_record_newer_than_the_version_is_not_rewritten() {
    scenarios::run_the_scenario_where_a_device_whose_journal_ring_holds_a_record_newer_than_the_version_is_not_rewritten();
}

#[test]
fn a_cloned_device_holding_units_written_by_a_later_instance_is_not_rewritten() {
    scenarios::run_the_scenario_where_a_cloned_device_holding_units_written_by_a_later_instance_is_not_rewritten();
}

#[test]
fn a_device_with_a_journal_ring_slot_reporting_a_read_error_is_not_rewritten() {
    scenarios::run_the_scenario_where_a_device_with_a_journal_ring_slot_reporting_a_read_error_is_not_rewritten();
}

#[test]
fn a_device_whose_ring_slot_of_the_version_root_no_longer_holds_it_is_not_rewritten() {
    scenarios::run_the_scenario_where_a_device_whose_ring_slot_of_the_version_root_no_longer_holds_it_is_not_rewritten();
}
