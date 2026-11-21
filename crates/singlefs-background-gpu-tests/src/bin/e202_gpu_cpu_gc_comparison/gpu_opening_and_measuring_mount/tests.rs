use super::*;

#[test]
fn every_opening_in_the_apparatus_is_handed_the_same_openings_table_of_this_process() {
    let handed_to_one_opening = openings_not_returned_in_this_process();
    let handed_to_another_opening = openings_not_returned_in_this_process();
    assert!(handed_to_one_opening.is_the_same_table_as(handed_to_another_opening));
}

/// 装置各份源文件（`main.rs` 与它声明的每份子模块）的原文连成一串，编进测试二进制：开卡的几处在没有真卡时走不到，「各处都取进程那一张、不现建」只能对着原文核。
const SOURCE_TEXT_OF_THE_APPARATUS: &str = concat!(
    include_str!("../main.rs"),
    include_str!("../arms_and_background_compute_settings.rs"),
    include_str!("../arms_and_background_compute_settings/tests.rs"),
    include_str!("../cards_of_the_command.rs"),
    include_str!("../cards_of_the_command/tests.rs"),
    include_str!("../clocks_and_host_load.rs"),
    include_str!("../clocks_and_host_load/tests.rs"),
    include_str!("../command_opening_and_construction_lines.rs"),
    include_str!("../command_opening_and_construction_lines/tests.rs"),
    include_str!("../concurrent_runs.rs"),
    include_str!("../concurrent_runs/tests.rs"),
    include_str!("../concurrent_subcommand.rs"),
    include_str!("../concurrent_subcommand/tests.rs"),
    include_str!("../construction_phases_and_timing.rs"),
    include_str!("../construction_phases_and_timing/tests.rs"),
    include_str!("../construction_registration_and_stops.rs"),
    include_str!("../construction_registration_and_stops/tests.rs"),
    include_str!("../construction_timing_subcommand.rs"),
    include_str!("../crates_digest_and_checks_before_the_run.rs"),
    include_str!("../crates_digest_and_checks_before_the_run/tests.rs"),
    include_str!("../forced_segment_fallback.rs"),
    include_str!("../gc_call_records_and_anchors.rs"),
    include_str!("../gc_call_records_and_anchors/tests.rs"),
    include_str!("../gc_call_windows.rs"),
    include_str!("../gc_call_windows/tests.rs"),
    include_str!("../gpu_opening_and_measuring_mount.rs"),
    include_str!("../gpu_opening_and_measuring_mount/tests.rs"),
    include_str!("../host_hardware_and_groups.rs"),
    include_str!("../host_hardware_and_groups/tests.rs"),
    include_str!("../judge_trend_subcommand.rs"),
    include_str!("../judge_trend_subcommand/tests.rs"),
    include_str!("../live_data_contents_and_reading.rs"),
    include_str!("../live_data_contents_and_reading/tests.rs"),
    include_str!("../lockstep_runs.rs"),
    include_str!("../lockstep_runs/tests.rs"),
    include_str!("../lockstep_subcommand.rs"),
    include_str!("../lockstep_tier_judgement_and_repeats.rs"),
    include_str!("../lockstep_tier_judgement_and_repeats/tests.rs"),
    include_str!("../pool_geometry_and_segment_choice.rs"),
    include_str!("../pool_geometry_and_segment_choice/tests.rs"),
    include_str!("../pools_cards_and_sessions_for_tests.rs"),
    include_str!("../recording_memory_devices.rs"),
    include_str!("../recording_memory_devices/tests.rs"),
    include_str!("../rejudging_arms.rs"),
    include_str!("../rejudging_arms/tests.rs"),
    include_str!("../result_lines_of_the_product.rs"),
    include_str!("../run_resumed_by_hardware_group.rs"),
    include_str!("../run_resumed_by_hardware_group/tests.rs"),
    include_str!("../sensitivity_and_geometry_check_subcommands.rs"),
    include_str!("../sensitivity_and_geometry_check_subcommands/tests.rs"),
    include_str!("../sha256_digest.rs"),
    include_str!("../sha256_digest/tests.rs"),
    include_str!("../stage_three_cost_subcommand.rs"),
    include_str!("../stage_three_cost_subcommand/tests.rs"),
    include_str!("../starting_image_construction.rs"),
    include_str!("../starting_image_construction/tests.rs"),
    include_str!("../statistics_and_judgement.rs"),
    include_str!("../statistics_and_judgement/tests.rs"),
    include_str!("../tests.rs"),
    include_str!("../unit_area_tiers_and_local_constants.rs"),
    include_str!("../unit_area_tiers_and_local_constants/tests.rs"),
);

#[test]
fn the_apparatus_builds_an_empty_openings_table_only_for_the_one_kept_for_this_process() {
    // 拼起来写：整串照原样写在这里，这一行自己就会被数进去。
    let building_an_empty_openings_table = concat!("with_none", "_not_returned");
    assert_eq!(
        SOURCE_TEXT_OF_THE_APPARATUS
            .matches(building_an_empty_openings_table)
            .count(),
        1,
        "装置里现建一张空的开卡表的只该有进程那一张（openings_not_returned_in_this_process）：别处现建的开卡不与别的开卡共用表"
    );
}
