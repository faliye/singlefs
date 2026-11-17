//! 对拍里 GPU 一侧（判段持卡那一段：开卡、跑 GPU、与读回的参照比）的判法单元入口登记表：一条真卡对拍用例一组，组名就是用例名，
//! 入口是那条用例的持卡判 fn（`judged_on_the_cards_that_<用例名>`，在它自己的测试目标里；实验装置 E202（GPU与CPU两种模式GC对照组）那一条内联真卡测试的在那份 bin 里，目标写 `bin:<名>`）。闭包从它收：开卡与取卡、产品 GPU 包里它真走到的
//! Rust 与 WGSL、读回的输入与参照怎么解、怎么比。构建脚本（`build.rs`）经 `#[path]` 编同一份，照参照一侧那张表同一个算法逐组算键、写进
//! `OUT_DIR/gpu_side_units_keys.tsv`；测试按用例名读回，与参照一侧那一把、读回输入的摘要一起组成对拍结论的键
//! （`tests/common_pairing_input_through_the_library`）：GPU 一侧走的代码或比法改了，键变，库里旧的结论不再被复用。
//! 一条用例一组：同一份输入、同一个参照组的几条用例比法各不相同，各落在自己的键上。
//! 这份文件只放数据，只用 `std`：构建脚本单独编它（行的写法在 `src/reference_units_entries.rs`）。

use crate::reference_units_entries::PairingSideUnitsEntryGroup;

/// 每条真卡对拍用例的持卡判入口，按测试目标排。
pub const GPU_SIDE_UNITS_ENTRY_GROUPS: &[PairingSideUnitsEntryGroup] = &[
    PairingSideUnitsEntryGroup {
        group_name: "the_table_driven_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_table_driven_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_bitwise_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_bitwise_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_two_kernels_confirm_every_entry_on_a_healthy_gpu",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_two_kernels_confirm_every_entry_on_a_healthy_gpu"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_the_batch_cuts_it_into_chunks_with_the_same_outcomes",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_the_batch_cuts_it_into_chunks_with_the_same_outcomes"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_a_unit_cuts_it_into_pieces_with_the_same_outcomes",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_a_unit_cuts_it_into_pieces_with_the_same_outcomes"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_batch_over_one_named_real_gpu_gives_it_work_and_it_submits",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_batch_over_one_named_real_gpu_gives_it_work_and_it_submits"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "opening_the_handed_over_gpus_again_and_again_in_one_process_succeeds_every_round_on_one_vulkan_instance",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_opening_the_handed_over_gpus_again_and_again_in_one_process_succeeds_every_round_on_one_vulkan_instance"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits",
        target: "test:checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu",
        target: "test:checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_batch_over_three_named_real_gpus_gives_each_work_and_each_submits",
        target: "test:checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_batch_over_three_named_real_gpus_gives_each_work_and_each_submits"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_kernel_searching_the_sorted_records_matches_the_cpu_reference_on_both_unit_areas",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_kernel_searching_the_sorted_records_matches_the_cpu_reference_on_both_unit_areas"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_two_live_slot_kernels_confirm_every_segment_on_a_healthy_gpu",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_two_live_slot_kernels_confirm_every_segment_on_a_healthy_gpu"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_the_record_table_cuts_it_into_slices_with_the_same_live_slots",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_the_record_table_cuts_it_into_slices_with_the_same_live_slots"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_below_the_smallest_piece_leaves_every_gpu_out_without_refusing_the_opening",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_below_the_smallest_piece_leaves_every_gpu_out_without_refusing_the_opening"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu",
        target: "test:mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu"],
    },
    PairingSideUnitsEntryGroup {
        group_name: "the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool",
        target: "bin:e202_gpu_cpu_gc_comparison",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool"],
    },
];
