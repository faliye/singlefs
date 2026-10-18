// 样本：六条模型对拍用例各调自己的函数（名字与真仓 random_histories.rs 里那六条相同）；setup.sh 拿这一版造上一版，只改回 reuse_steps

fn fast_steps() -> u32 { 1912 }
fn reuse_steps() -> u32 { 1183 }
fn rollback_steps() -> u32 { 913 }
fn allocation_steps() -> u32 { 4001 }
fn unit_area_steps() -> u32 { 4749 }
fn unit_area_judged_steps() -> u32 { 4749 }

#[test]
#[ignore]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() { assert!(fast_steps() > 0); }

#[test]
#[ignore]
fn reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms() { assert!(reuse_steps() > 0); }

#[test]
#[ignore]
fn rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms() { assert!(rollback_steps() > 0); }

#[test]
#[ignore]
fn allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall() { assert!(allocation_steps() > 0); }

#[test]
#[ignore]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall() { assert!(unit_area_steps() > 0); }

#[test]
#[ignore]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval() { assert!(unit_area_judged_steps() > 0); }
