//! 进键的那几组判法单元的入口登记表：挂得上的观察者每个一行（[`OBSERVER_ENTRIES`]），判器引到的被检测产品代码一组
//! （[`JUDGED_PRODUCT_CODE_ENTRIES`]）。构建脚本（`build.rs`）经 `#[path]` 编同一份，经 `singlefs-unit-graph` 逐组算键与组成清单、写进 `OUT_DIR`；
//! 库按组名读回编进去的那一把，跑时不读盘上的树（剪枝复查 design 路报告 3.8「build.rs 做什么」）。表里的文字（组名、路径、观察者名）只用来找入口，不进键。
//! 两种组都只留产品包（[`PRODUCT_PACKAGES_KEPT_IN_THE_KEYS`]）的单元：判器（池级 checker、记录核对器、CPU 参照判器、GPU 判器）、观察者、
//! 录制管线与测试服务的代码不进任何键，崩溃判定的键 = 块的输入数据 + 判器与观察者引到的被检测产品代码的指纹
//! （`crate::crash_identity::ConclusionVersion::of_the_crash_image_judgement` 一族与观察者的版本）。录入键不在这里：它只由录下的写流摘要与场景标识组成
//! （`crate::crash_library::recording_key_of_the_scenario_and_the_inputs`），代码一个单元都不进。
//! 这份文件只放数据与几个拼名字、取特性的函数，只用 `std`：构建脚本单独编它。
//!
//! 一组的入口都在同一个编译目标里（`lib`、`test:<测试目标>`）；只在某个特性下编进来的入口，这一趟没开那个特性就不收
//! （组成清单那份表里记 `not-compiled-entry`），一个都编不进来的组整组记「不编进来」，跑时取它就拒算。

/// 一个观察者的入口：观察者代码文件（`crate::crash_amplification::ObserverOnTheNodes::observer_code_file`）与观察者名对到它的入口 fn。
/// 观察者结论的键里分开同一块上几个观察者的，是它的判定项标识（观察者代码文件与观察者名拼的组名 [`observer_group_name`]），不是它的代码：
/// 构建脚本按入口 fn 收它那一组，键只留 [`PRODUCT_PACKAGES_KEPT_IN_THE_KEYS`] 的单元（观察者引到的被检测产品代码，进它的版本），
/// 观察者自己的代码不进；没登记、或那一组拒算的照判、不入库。
pub struct ObserverEntry {
    pub observer_code_file: &'static str,
    pub observer_name: &'static str,
    /// 编译目标：观察者写在测试目标里时是 `test:<测试目标>`（几个测试目标共用的搭建模块，挑一个编进它的）。
    pub target: &'static str,
    pub entry_functions: &'static [&'static str],
}

/// 观察者登记表。观察者代码写成闭包、闭包里只转调一个 fn 的，入口是被转调的那个 fn（闭包本身只是把另读的输入递过去，另读的输入
/// 另按摘要进它的版本）；代码整段写在测试函数的闭包里的，要先挪成 fn 才登记得上，没登记的照判、不入库。
/// 实验包装置挂的观察者不登在这里：登在实验包自己的观察者登记表，那一张表由实验包的构建脚本写，装置经
/// `crate::crash_state_enumeration::enumerate_crash_states_with_expansion_keying_the_observers_in` 把那一张交给崩溃枚举
/// （`crate::crash_identity::ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage`）。
pub const OBSERVER_ENTRIES: &[ObserverEntry] = &[
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_crash_point_observers/mod.rs",
        observer_name: "writable_mount_and_continuation",
        target: "test:crash_enumeration_multi_record_publish_stream",
        entry_functions: &[
            "singlefs_verification::common_crash_point_observers::observe_the_mount_continuation",
        ],
    },
    // 多记录流与删除流的崩溃枚举用例挂的同一个发布边界观察者：两条用例的挂法都点名这一份代码文件（`crash_state_enumeration::CrashPointReplayObserverCodeFile::SharedFile`），
    // 只这一行，两条流同一个判定项标识
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_crash_point_observers/mod.rs",
        observer_name: "multi_record_publish_boundary",
        target: "test:crash_enumeration_multi_record_publish_stream",
        entry_functions: &[
            "singlefs_verification::common_crash_point_observers::observe_the_publish_boundary_of_the_multi_record_publishes",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_record_spill_over_stream.rs",
        observer_name: "publish_boundary",
        target: "test:crash_enumeration_record_spill_over_stream",
        entry_functions: &["singlefs_verification::observe_the_publish_boundary"],
    },
    // 删除流的崩溃枚举用例只挂在删除那几步的段上的删除停在中间观察者（它的可写挂载续做观察者与发布边界观察者点名公共模块，用上面那两行）
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
        observer_name: "deletion_cut_in_the_middle",
        target: "test:crash_enumeration_unlink_after_the_multi_record_publish_stream",
        entry_functions: &["singlefs_verification::observe_the_deletion_cut_in_the_middle"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs",
        observer_name: "crash_image_handed_back",
        target: "test:crash_amplification_observer_images_judged_on_the_gpu_name_the_card",
        entry_functions: &["singlefs_verification::hand_back_the_crash_image"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs",
        observer_name: "system_configuration_slot_rewritten_past_the_limit",
        target: "test:crash_amplification_observer_images_judged_on_the_gpu_name_the_card",
        entry_functions: &[
            "singlefs_verification::rewrite_the_first_system_configuration_slot_past_the_judge_table_limit",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs",
        observer_name: "residual_record_chain",
        target: "test:crash_enumeration_fixed_script_stream",
        entry_functions: &["singlefs_verification::observe_the_chain_to_the_residual_record"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs",
        observer_name: "stale_tail",
        target: "test:crash_enumeration_fixed_script_stream",
        entry_functions: &["singlefs_verification::observe_the_stale_tail"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs",
        observer_name: "torn_slots",
        target: "test:crash_segments_per_device_and_torn_in_place_overwrites",
        entry_functions: &["singlefs_verification::count_the_torn_system_configuration_slots"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_writable_mount_of_a_minimum_input_output_8192_pool.rs",
        observer_name: "slot_one_at_8192",
        target: "test:crash_enumeration_writable_mount_of_a_minimum_input_output_8192_pool",
        entry_functions: &["singlefs_verification::observe_slot_one_at_8192"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_points_of_the_instance_table_sweep.rs",
        observer_name: "f5_after_the_deletion",
        target: "test:crash_points_of_the_instance_table_sweep",
        entry_functions: &["singlefs_verification::count_the_states_where_the_deletion_took_effect"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_points_of_the_instance_table_sweep.rs",
        observer_name: "remount_after_the_sweep_phases",
        target: "test:crash_points_of_the_instance_table_sweep",
        entry_functions: &[
            "singlefs_verification::remount_after_the_sweep_phases_without_the_deleted_row",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_points_of_the_instance_table_sweep.rs",
        observer_name: "rebuilt_instance_table_against_the_allocation_records",
        target: "test:crash_points_of_the_instance_table_sweep",
        entry_functions: &[
            "singlefs_verification::judge_the_rebuilt_instance_table_against_the_allocation_records",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_formatted_pool_remounted_without_a_file_stream.rs",
        observer_name: "remount_and_publish_the_first_file",
        target: "test:crash_enumeration_formatted_pool_remounted_without_a_file_stream",
        entry_functions: &["singlefs_verification::remount_and_publish_the_first_file_after_the_crash"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs",
        observer_name: "remount_after_the_crash",
        target: "test:crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream",
        entry_functions: &[
            "singlefs_verification::remount_after_the_crash_with_an_instance_above_every_one_on_the_disks",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_acquisition_rewriting_a_device_without_a_self_verified_system_configuration_stream.rs",
        observer_name: "remount_after_the_crash_of_the_rewrite",
        target: "test:crash_enumeration_acquisition_rewriting_a_device_without_a_self_verified_system_configuration_stream",
        entry_functions: &[
            "singlefs_verification::remount_after_the_crash_of_the_rewrite_with_both_slots_of_every_device_self_verified",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_fixed_script_stream.rs",
        observer_name: "mount_continuation",
        target: "test:crash_enumeration_unlink_after_the_fixed_script_stream",
        entry_functions: &[
            "singlefs_verification::common_crash_point_observers::observe_the_mount_continuation",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs",
        observer_name: "count_each_state_once",
        target: "test:crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume",
        entry_functions: &["singlefs_verification::count_each_state_once"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs",
        observer_name: "allocation_generation_read_sets",
        target: "test:crash_enumeration_new_pool_file_creation_stream",
        entry_functions: &["singlefs_verification::count_the_allocation_generation_read_set"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_floor_raise_pushed_by_admission.rs",
        observer_name: "writable_mount_after_the_crash",
        target: "test:crash_enumeration_floor_raise_pushed_by_admission",
        entry_functions: &["singlefs_verification::mount_writable_after_the_crash_of_the_floor_raise"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs",
        observer_name: "writable_mount_after_the_crash",
        target: "test:crash_enumeration_frozen_publish_resent_stream",
        entry_functions: &["singlefs_verification::mount_writable_after_a_crash_in_the_resend"],
    },
    // 截断、紧缩那几条流的观察者，崩溃点重放那一路：观察者代码文件是设点的用例文件（`crash_state_enumeration` 的 observers_on_the_nodes_of_the_item），用例里的闭包只转调
    // 公共模块里的一个 fn，入口是被转调的那个。
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_to_zero_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_to_zero_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_into_a_unit_then_extend_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_into_a_unit_then_extend_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_under_a_snapshot_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocation_stream_on_the_gpu_path",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs",
        observer_name: "mount_continuing_the_compaction_on_the_gpu_path",
        target: "test:crash_enumeration_compaction_relocation_stream_on_the_gpu_path",
        entry_functions: &["singlefs_verification::observe_the_mount_continuing_the_compaction_on_the_gpu_path"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_switching_backends_between_batches.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocation_stream_switching_backends_between_batches",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_switching_backends_between_batches.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocation_stream_switching_backends_between_batches",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_lifecycle_closure_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_lifecycle_closure_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_lifecycle_closure_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_lifecycle_closure_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_snapshot_creation_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_snapshot_creation_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_unlink_under_a_snapshot_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_unlink_under_a_snapshot_stream.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    // 崩溃放量那一路：观察者代码文件是观察者所在的公共模块（`common_snapshot_crash_point_observers::amplify_a_snapshot_flow`、
    // `common_compaction_crash_point_observers::amplify_a_compaction_flow`），几个测试目标共用，挑一个编进它的。
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_snapshot_crash_point_observers/mod.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_snapshot_creation_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_snapshot_crash_point_observers/mod.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_snapshot_creation_stream",
        entry_functions: &[
            "singlefs_verification::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_compaction_crash_point_observers/mod.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_compaction_crash_point_observers/mod.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_truncation_crash_streams/mod.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_to_zero_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_truncation_crash_streams/mod.rs",
        observer_name: "mount_continuing_the_truncation",
        target: "test:crash_enumeration_truncate_to_zero_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_crash_streams::observe_the_mount_continuing_the_truncation",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_several_inodes_overwritten_in_turn_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_several_inodes_overwritten_in_turn_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_several_inodes_overwritten_in_turn_stream.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_several_inodes_overwritten_in_turn_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_large_file_overwritten_across_lower_leaves_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_large_file_overwritten_across_lower_leaves_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_large_file_overwritten_across_lower_leaves_stream.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_large_file_overwritten_across_lower_leaves_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_unlink_of_two_files_holding_data_streams",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_unlink_of_two_files_holding_data_streams",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_multiple_copy_on_write_crash_streams/mod.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/common_multiple_copy_on_write_crash_streams/mod.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_verification::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs",
        observer_name: "make_filesystem_clause",
        target: "test:crash_enumeration_make_filesystem_over_a_used_pool_stream",
        entry_functions: &["singlefs_verification::observe_the_outcome_allowed_by_the_clause"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs",
        observer_name: "landed_version_reading_both_files_through_the_mapping",
        target: "test:crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream",
        entry_functions: &[
            "singlefs_verification::observe_the_landed_version_reading_both_files_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs",
        observer_name: "mount_continuing_the_deletion_and_the_compaction",
        target: "test:crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream",
        entry_functions: &[
            "singlefs_verification::observe_the_mount_continuing_the_deletion_and_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_deadlist_reclamation_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_deadlist_reclamation_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_deadlist_reclamation_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_deadlist_reclamation_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_three_generation_mount_fork_stream.rs",
        observer_name: "observe_root_fork_isolation",
        target: "test:crash_enumeration_three_generation_mount_fork_stream",
        entry_functions: &["singlefs_verification::observe_root_fork_isolation"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_three_generation_mount_fork_stream.rs",
        observer_name: "observe_mount_continuation",
        target: "test:crash_enumeration_three_generation_mount_fork_stream",
        entry_functions: &["singlefs_verification::observe_mount_continuation"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream.rs",
        observer_name: "observe_the_read_only_mount_reading_back_the_recovered_content",
        target: "test:crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream",
        entry_functions: &[
            "singlefs_verification::observe_the_read_only_mount_reading_back_the_recovered_content",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream.rs",
        observer_name: "observe_the_writable_mount_writing_one_more_unit",
        target: "test:crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream",
        entry_functions: &["singlefs_verification::observe_the_writable_mount_writing_one_more_unit"],
    },
    // 按来历统一挂的观察者（G4 丙-置「按流登记+相对基镜像」与「你做你的 GPU版让GPU去做」）：跑流的两处
    // （崩溃枚举用例的枚举入口、崩溃放量）按来历表挂，观察者代码文件就是来历表那一份，入口是那里的 fn（判段看），编在库里。
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/src/crash_point_replay_provenance.rs",
        observer_name: "devices_without_a_self_verified_system_configuration_slot",
        target: "lib",
        entry_functions: &[
            "singlefs_verification::crash_point_replay_provenance::observe_the_devices_without_a_self_verified_system_configuration_slot",
        ],
    },
    // 崩溃点重放那一路挂在用例文件上的观察者：用例里的闭包只转调一个 fn，入口是被转调的那个（公共模块里的写模块路径，用例文件里的写测试目标根上的名字）。
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream",
        entry_functions: &[
            "singlefs_verification::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_automatic_publish_interrupted_stream.rs",
        observer_name: "writable_mount_after_the_crash",
        target: "test:crash_enumeration_automatic_publish_interrupted_stream",
        entry_functions: &[
            "singlefs_verification::mount_writable_after_a_crash_in_the_automatic_publishes",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_batched_requests_not_yet_published_stream.rs",
        observer_name: "writable_mount_after_the_crash",
        target: "test:crash_enumeration_batched_requests_not_yet_published_stream",
        entry_functions: &["singlefs_verification::mount_writable_after_a_crash_in_the_drain"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_mounts_with_read_faults_stream.rs",
        observer_name: "where_the_new_instance_records_are_applied",
        target: "test:crash_enumeration_mounts_with_read_faults_stream",
        entry_functions: &[
            "singlefs_verification::observe_where_the_new_instance_records_are_applied",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_mounts_with_read_faults_stream.rs",
        observer_name: "a_new_tree_after_the_writable_mount",
        target: "test:crash_enumeration_mounts_with_read_faults_stream",
        entry_functions: &["singlefs_verification::observe_a_new_tree_after_the_writable_mount"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_partial_write_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_partial_write_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_to_zero_on_a_nearly_full_pool_stream.rs",
        observer_name: "mount_continuing_the_truncation_at_the_edge",
        target: "test:crash_enumeration_truncate_to_zero_on_a_nearly_full_pool_stream",
        entry_functions: &[
            "singlefs_verification::observe_the_mount_continuing_the_truncation_at_the_edge",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_an_inode_other_than_the_first_to_zero_stream.rs",
        observer_name: "landed_version_and_the_truncated_inode_read_back",
        target: "test:crash_enumeration_truncate_an_inode_other_than_the_first_to_zero_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_of_an_inode_other_than_the_first_crash_streams::observe_the_landed_version_and_the_truncated_inode",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_an_inode_other_than_the_first_into_a_unit_then_extend_stream.rs",
        observer_name: "landed_version_and_the_truncated_inode_read_back",
        target: "test:crash_enumeration_truncate_an_inode_other_than_the_first_into_a_unit_then_extend_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_of_an_inode_other_than_the_first_crash_streams::observe_the_landed_version_and_the_truncated_inode",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-verification/tests/crash_enumeration_truncate_an_inode_other_than_the_first_under_a_snapshot_stream.rs",
        observer_name: "landed_version_and_the_truncated_inode_read_back",
        target: "test:crash_enumeration_truncate_an_inode_other_than_the_first_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_verification::common_truncation_of_an_inode_other_than_the_first_crash_streams::observe_the_landed_version_and_the_truncated_inode",
        ],
    },
];

/// 判器引到的被检测产品代码那一组的组名（只当索引）。
pub const JUDGED_PRODUCT_CODE_GROUP_NAME: &str = "judged_product_code";

/// 判器引到的被检测产品代码那一组的编译目标。
pub const JUDGED_PRODUCT_CODE_TARGET: &str = "lib";

/// 判器引到的被检测产品代码那一组从哪几个入口收：CPU 参照判器判一个崩溃镜像（恢复看与不看 journal 两遍、oracle、池级 checker、记录核对器）、
/// CPU 路线判一块的那一段、给定状态的 CPU 判器、观察者交回的盘上跑池级 checker 的 CPU 那一路、第 ② 段核对的 CPU 参照。闭包里只留
/// [`PRODUCT_PACKAGES_KEPT_IN_THE_KEYS`] 的单元：入口与它们够到的判器代码不进，判器调到的产品恢复、挂载与格式代码进；GPU 判器的 WGSL
/// 是照 Rust 另写的一份，不在这几个入口的闭包里。一组的键进判器判出的三种结论的版本与对拍结论的键。
pub const JUDGED_PRODUCT_CODE_ENTRIES: &[&str] = &[
    "singlefs_verification::crash::judge_crash_image",
    "singlefs_verification::crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu",
    "singlefs_verification::crash_given_states::judge_the_given_states_on_the_cpu",
    "singlefs_verification::crash_amplification::images_after_the_observers::judge_the_images_after_the_observers_on_the_cpu",
    "singlefs_verification::crash_facts::verify_states_from_facts",
    "singlefs_verification::crash_facts::verify_states_from_facts_by_the_segment_expansions",
];

/// 进键的几组留哪几个包的单元：`.claude/gate.d/crate-roles.tsv` 里角色是「产品」的那五个（清单 `[package] name` 的写法）。
pub const PRODUCT_PACKAGES_KEPT_IN_THE_KEYS: &[&str] = &[
    "singlefs-format",
    "singlefs-core",
    "singlefs-replay-prefix",
    "singlefs-mount-runtime",
    "singlefs-background-gpu",
];

/// 清单 `[features]` 里 GPU 那个特性的名字。
pub const GPU_FEATURE: &str = "gpu";

/// 判器引到的被检测产品代码那一组收闭包、算键时交的本包特性：固定是 [`GPU_FEATURE`] 这一个，不照这一次编译开了哪些。
/// 依赖包那一侧同样固定：一个都不当开着（构建脚本交空的「依赖包被并上来的特性」，不照 `links` 交来的那一份）。
/// 同一棵树带不带 `lookup-work-counters`（验证包的 dev 依赖替核心层并上的只给测试计数的特性）的两种构建算出同一把键：
/// 两种构建不各判一遍同一批（D13（验证路线） 已定项 17「两份库重复判同一批，同样算违反」）。
pub const JUDGED_PRODUCT_CODE_FEATURES: &[&str] = &[GPU_FEATURE];

/// 观察者那几组收闭包、算键时交的本包特性：与判器那一组同一组固定的 cfg（[`JUDGED_PRODUCT_CODE_FEATURES`]，只开 [`GPU_FEATURE`]），
/// 依赖包那一侧同样一个都不当开着，不照这一次编译开了哪些、cargo 替依赖包并上了哪些。录入段与判段可以各在一份二进制里跑
/// （只录的那一趟不带 `gpu` 编、判段带着编；测试构建另替核心层并上只给测试计数的特性），几种构建都算出同一把观察者的版本，
/// 同一批块不判两遍（D13（验证路线） 已定项 17「两份库重复判同一批，同样算违反」）。
pub const OBSERVER_FEATURES: &[&str] = JUDGED_PRODUCT_CODE_FEATURES;

/// 观察者那一组的组名，也是它在结论键里的判定项标识：`observer_code:<观察者代码文件>:<观察者名>`。
#[must_use]
pub fn observer_group_name(observer_code_file: &str, observer_name: &str) -> String {
    format!("observer_code:{observer_code_file}:{observer_name}")
}
