//! 全池没有可用的系统配置槽 0 时，池级 checker 与恢复找槽 1 的读法相同、结论相同（C651（全池没有可用的系统配置槽 0 时 checker
//! 与实现找槽 1 的读法不同），第一轮判决 `research/prompts/m3-c651-r1-main-verification.md` 第四节三条与「用户定案」W1）：
//! 在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过根环起点 × 16384 − 4096），只收自证过、且自述的固定结构槽距
//! 等于所在偏移的槽；几档都收得到时取世代号最大的，世代号相同取偏移小的；读到带读者不收之值的槽整池拒（读到可用的一档也先判
//! I-8.1 三个字段等式、再核偏移，代码轮第二轮判决 K3-1 `research/prompts/m3-formal-code-r2-main-verification.md`）；
//! mkfs 在写任何元数据之前把每块盘的 [0, 根环基址) 整段写 0，在用过的盘上换一个 io_min 重做 mkfs，旧池别的槽距上的槽 1 不再被择中。
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），两边读的是同一份 `MemoryPool`。
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs"]
mod scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::{MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING, MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING, chosen_by_recovery, chosen_by_the_checker, formatted_image_with_minimum_input_output_bytes, image_of, parameters_with_minimum_input_output_bytes, plant_slot, slot_at, slot_bytes, two_sparse_devices, wipe_the_slot_at_on_every_device};
use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::PoolReader;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192,
    );
}

/// 只收自述的固定结构槽距等于所在偏移的槽（第四节第 1 条）：io_min 8192 的池，4096 处放一份自证得过、世代号 9、记着槽距 8192 的槽
/// （它不在自己记的槽距上），两块盘的槽 0 都抹掉：恢复与 checker 都不收它，都择 8192 处世代号 1 的那一份。
#[test]
fn a_self_verified_slot_whose_recorded_spacing_is_not_its_own_offset_is_taken_by_neither_the_checker_nor_recovery(
) {
    let spacing = u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING);
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut misplaced = slot_at(&image, device, spacing);
        misplaced.quantities.slot_generation = 9;
        plant_slot(&mut image, device, 4096, &misplaced);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, spacing, 1),
        "4096 处那一份记的槽距是 8192，恢复不收；择到 8192 处世代号 1 的"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((spacing, 1))), (1, Some((spacing, 1)))],
        "checker 同样不收 4096 处那一份"
    );
}

/// 几档都收得到时取世代号最大的（第四节第 2 条）：槽距 4096 的池，8192 处放一份同 fsid、记着槽距 8192、世代号 5 的槽，
/// 两块盘的槽 0 都抹掉。4096 处世代号 1、8192 处世代号 5 都自证过、记的槽距都等于所在偏移：恢复与 checker 都择世代号 5 的那一份。
#[test]
fn slots_found_at_several_spacings_resolve_to_the_highest_generation_in_both_the_checker_and_recovery(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut at_8192 = slot_at(&image, device, 4096);
        at_8192.immutable.sizes.fixed_structure_slot_spacing = 8192;
        at_8192.quantities.slot_generation = 5;
        plant_slot(&mut image, device, 8192, &at_8192);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, 8192, 5),
        "恢复择到 8192 处世代号 5 的那一份"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((8192, 5))), (1, Some((8192, 5)))],
        "checker 与恢复择同一份：8192 处世代号 5"
    );
}

/// 世代号相同时取偏移小的（第四节第 2 条）：槽距 4096 的池，8192 处放一份同 fsid、记着槽距 8192、世代号同为 1 的槽，
/// 两块盘的槽 0 都抹掉：恢复与 checker 都择 4096 处本池的那一份。
#[test]
fn slots_of_equal_generation_found_at_several_spacings_resolve_to_the_smaller_offset_in_both_the_checker_and_recovery(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut at_8192 = slot_at(&image, device, 4096);
        at_8192.immutable.sizes.fixed_structure_slot_spacing = 8192;
        plant_slot(&mut image, device, 8192, &at_8192);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, 4096, 1),
        "世代号都是 1：恢复取偏移小的 4096 那一份"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((4096, 1))), (1, Some((4096, 1)))],
        "checker 与恢复择同一份：4096 处世代号 1"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery,
    );
}

/// mkfs 清 [0, 根环基址)（W1）：一块盘先 mkfs 一次槽距 4096 的池、可写挂载一次（取号与两次发布各写一次系统配置，槽 1 世代号到 3），
/// 再拿同一个 fsid mkfs 一次 io_min 8192 的池，之后两块盘的槽 0 都抹掉。旧池 4096 处的槽 1 记着槽距 4096、世代号比新池的 1 大，
/// mkfs 不清它时逐档读的两个读者都会择中它；清了之后 4096 处是 0，恢复与 checker 都择到新池 8192 处的槽 1。
#[test]
fn remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader(
) {
    let mut devices = two_sparse_devices();
    let earlier_parameters =
        parameters_with_minimum_input_output_bytes(MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING);
    make_filesystem(&earlier_parameters, &mut devices).expect("先 mkfs 一次槽距 4096 的池");
    mount_writable(&earlier_parameters, &mut devices).expect("旧池可写挂载一次");
    let earlier_image = image_of(&devices);
    let earlier_slot_one_generation = slot_at(&earlier_image, DeviceIdentity(0), 4096)
        .quantities
        .slot_generation;
    assert!(
        earlier_slot_one_generation > 1,
        "旧池 4096 处的槽 1 写过系统配置，世代号 {earlier_slot_one_generation} 比新池 mkfs 写下的 1 大"
    );

    make_filesystem(
        &parameters_with_minimum_input_output_bytes(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING),
        &mut devices,
    )
    .expect("同一个 fsid 再 mkfs 一次 io_min 8192 的池");
    let mut image = image_of(&devices);
    wipe_the_slot_at_on_every_device(&mut image, 0);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let bytes_where_the_earlier_slot_one_was =
            PoolReader::read(&image, device, DeviceOffsetInBytes(4096), slot_bytes())
                .expect("4096 处读得到");
        assert!(
            bytes_where_the_earlier_slot_one_was
                .iter()
                .all(|byte| *byte == 0),
            "盘 {device:?}：mkfs 之后旧池槽 1 所在的 4096 处整槽是 0"
        );
    }
    assert_eq!(
        chosen_by_recovery(&image),
        (0, 8192, 1),
        "恢复择到新池 8192 处世代号 1 的槽 1，不是旧池 4096 处世代号 {earlier_slot_one_generation} 的那一份"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((8192, 1))), (1, Some((8192, 1)))],
        "checker 与恢复择同一份：新池 8192 处世代号 1"
    );
}
