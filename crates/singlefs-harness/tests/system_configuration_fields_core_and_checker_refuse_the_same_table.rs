//! core 与池级 checker 收同一张系统配置几何字段表（代码三方 m2-closeout-code-r2 Y4-a「读字段，不等于常量就整池拒」）：
//! 核心层读者（`singlefs-core` 的 `recovery::system_configuration_values_this_reader_accepts`，改法 B）与池级 checker 的
//! I-7.13（系统配置池级字段在读者收的范围里）（`singlefs-checker` 的 `image::geometry_of`，改法 A）各写一份，这里在同一份镜像上两边一起判。
//! 攻方 Y4 那五格（`research/prompts/m2-closeout-code-r2-opus-model/opus_r2_y4_system_configuration_fields_core_and_checker_read_differently.rs`，
//! 只取它造镜像的写法）：新池新建文件之后那份合法镜像，两块盘四个系统配置槽里改同一个几何字段、整槽校验和重封。每一格：
//! core 的只读挂载与可写挂载都整池拒、拒成同一个值、两块盘逐字节不变；池级 checker 报 I-7.13 违例、说明里点名它拒的成员，
//! 别的每一条报不适用（池挂不上）。改之前（第二轮快照）core 不读 325 与 371：B、C、E 三格 core 照挂而 checker 报 I-7.13
//! 「实现整池拒绝挂载」，A、D 两格 core 拒而 I-7.13 不红。
//! E 格两边拒的理由不同：core 先判槽距上界（按同一槽自述的根环起点 0 算，槽 1 不在它之前），checker 先判根环起点等不等第一版常量；
//! 这里照今天两边各自的判定次序钉。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/system_configuration_fields_core_and_checker_refuse_the_same_table.rs"]
mod scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base,
    );
}
