//! 判法单元的入口登记表：每一组判法单元从哪几个入口 fn 收闭包（`crate::crash_identity::JudgingUnits` 每一种一行，挂得上的观察者每个一行）。
//! 构建脚本（`build.rs`）经 `#[path]` 编同一份，经 `singlefs-unit-graph` 逐组算键与组成清单、写进 `OUT_DIR`；库按组名读回编进去的那一把，
//! 跑时不读盘上的树（剪枝复查第二轮 design 路报告 3.8「build.rs 做什么」，4-2a）。表里的文字（组名、路径、观察者名）只用来找入口，不进键。
//! 这份文件只放数据与拼组名的一个函数，只用 `std`：构建脚本单独编它。
//!
//! 一组的入口都在同一个编译目标里（`lib`、`test:<测试目标>`、`bin:<名>`）；只在某个特性下编进来的入口，这一趟没开那个特性就不收
//! （组成清单那份表里记 `not-compiled-entry`），一个都编不进来的组整组记「不编进来」，跑时取它就拒算。

/// 一组判法单元的入口。
pub struct JudgingUnitsEntryGroup {
    /// 组名：与 `crate::crash_identity::JudgingUnits::group_name` 交的相同。
    pub group_name: &'static str,
    /// 编译目标：`lib`、`test:<测试目标>`、`bin:<名>`。
    pub target: &'static str,
    /// 入口 fn 路径：`singlefs_checker_tier::<模块路径>::<fn>`，impl 里的方法写 `…::<类型>::<方法>`。
    pub entry_functions: &'static [&'static str],
}

/// 判器、核对、录入、观察者结论拼块六组的入口。
///
/// - GPU 判器：三个内核的源码怎么拼（`JudgeKernelSources`，连同旋钮那一份；扫描内核源码里的第三个入口 `scan_rebuild_main` 在
///   `crash_judge_checker_scan_rebuild.wgsl`，经拼扫描内核源码那一个函数读进闭包，它的 `@compute` 入口照 WGSL 的收法进组）、上卡与派活
///   （编五条管线的 `compile_the_judge_kernels`）、回读归位（`GpuCrashJudge::judge_states` 调的 `judge_one_dispatch`：派活里决定结论落到哪个状态的那一段，
///   底稿 L7）、结论字怎么解成红项、恢复字、计数向量与扫描重建那一截的结局码（`scan_rebuild_outcome_of_the_gpu_verdict`）。
/// - 判镜像的定义（`judgement_of_the_crash_images`）：CPU 参照判器 `crash::judge_crash_image` 与结论的编码（红项、计数向量）；镜像判定的版本
///   就是这一把，不论在 CPU 还是 GPU 上判的（GPU 判器那一组只进块属性当出处）。
/// - 判给定状态的定义（`judgement_of_the_given_states`）：CPU 参照判器 `crash_given_states` 的判法与它的计数向量、红项编码；同上。
/// - 两种核对：第 ② 段的 CPU 参照与 GPU 内核。
/// - 录入：写进库的那几样怎么从流算出、怎么编成字节（事实表抽取、判表建表与打包、录入的流）。
/// - 观察者结论拼块：观察者交回的盘怎么建判表、判不了的怎么记、一块怎么拼（原来那个手写的拼块版本号管的那几处）。
/// - 两种对拍的比法：对拍测试里逐状态比两边判定、交回结论的那一个函数（键拼进对拍结论的输入摘要）。
pub const JUDGING_UNITS_ENTRY_GROUPS: &[JudgingUnitsEntryGroup] = &[
    JudgingUnitsEntryGroup {
        group_name: "gpu_judge",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash_judge_gpu::JudgeKernelSources::of_the_judge",
            "singlefs_checker_tier::crash_judge_gpu::JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs",
            "singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge::on_card_compiling_these_kernel_sources",
            "singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge::judge_states",
            "singlefs_checker_tier::crash_judge_gpu::gpu_verdict_items",
            "singlefs_checker_tier::crash_judge_gpu::recovery_words_of_the_gpu_verdict",
            "singlefs_checker_tier::crash_judge_gpu::cross_check_words_of_gpu_verdict",
            "singlefs_checker_tier::crash_judge_gpu::scan_rebuild_outcome_of_the_gpu_verdict",
            "singlefs_checker_tier::crash_judge_conclusions::StateCountsOfABlock::count_the_gpu_bits",
            "singlefs_checker_tier::crash_judge_conclusions::StateCountsOfABlock::into_count_vector",
            "singlefs_checker_tier::crash_judge_conclusions::encoded_verdict_of_the_red_items",
        ],
    },
    JudgingUnitsEntryGroup {
        group_name: "judgement_of_the_crash_images",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash::judge_crash_image",
            "singlefs_checker_tier::crash_judge_conclusions::StateCountsOfABlock::count_the_cpu_judgement",
            "singlefs_checker_tier::crash_judge_conclusions::StateCountsOfABlock::into_count_vector",
            "singlefs_checker_tier::crash_judge_conclusions::encoded_verdict_of_the_red_items",
        ],
    },
    JudgingUnitsEntryGroup {
        group_name: "judgement_of_the_given_states",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash_given_states::judge_the_given_states_on_the_cpu",
            "singlefs_checker_tier::crash_given_states::count_vector_of_the_given_states",
            "singlefs_checker_tier::crash_judge_conclusions::encoded_verdict_of_the_red_items",
        ],
    },
    JudgingUnitsEntryGroup {
        group_name: "cpu_facts_verifier",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash_facts::verify_states_from_facts",
            "singlefs_checker_tier::crash_facts::verify_states_from_facts_by_the_segment_expansions",
        ],
    },
    JudgingUnitsEntryGroup {
        group_name: "gpu_facts_verifier",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash_verify_gpu::GpuFactsVerifier::new",
            "singlefs_checker_tier::crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions",
            "singlefs_checker_tier::crash_verify_gpu::GpuFactsVerifier::verify_states",
        ],
    },
    JudgingUnitsEntryGroup {
        group_name: "recording",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash_facts::extract_flow_facts_with_slices",
            "singlefs_checker_tier::crash_facts::FlowFacts::to_bytes",
            "singlefs_checker_tier::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices",
            "singlefs_checker_tier::crash_judge_tables::build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions",
            "singlefs_checker_tier::crash_judge_tables::JudgeTables::words",
            "singlefs_checker_tier::crash_judge_dispatch::PackedJudgeTables::of",
            "singlefs_checker_tier::crash_judge_dispatch::PackedJudgeTables::to_bytes",
            "singlefs_checker_tier::crash_amplification::RecordedFlow::to_bytes",
        ],
    },
    JudgingUnitsEntryGroup {
        group_name: "gpu_and_cpu_judge_pairing_comparison",
        target: "test:crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state",
        entry_functions: &["singlefs_checker_tier::compare_the_judgements_state_by_state"],
    },
    JudgingUnitsEntryGroup {
        group_name: "gpu_and_cpu_facts_verifier_pairing_comparison",
        target: "test:crash_facts_gpu_verifier_agrees_with_the_cpu_reference",
        entry_functions: &["singlefs_checker_tier::verifier_bits_agreeing_on_every_state"],
    },
    JudgingUnitsEntryGroup {
        group_name: "observer_conclusion_composition",
        target: "lib",
        entry_functions: &[
            "singlefs_checker_tier::crash_amplification::judge_the_images_after_the_observers_on_the_cpu",
            "singlefs_checker_tier::crash_amplification::judge_the_images_after_the_observers_on_the_gpu",
            "singlefs_checker_tier::crash_amplification::plan_the_judge_tables_of_the_images_after_the_observers",
            "singlefs_checker_tier::crash_amplification::pool_after_the_observer",
        ],
    },
];

/// 一个观察者的入口：观察者代码文件（`crate::crash_amplification::ObserverOnTheNodes::observer_code_file`）与观察者名对到它的入口 fn。
/// 观察者的身份就是这几个入口 fn 的闭包键（同一份文件里两个观察者各自一把）；没登记的观察者取键拒算（照判、不入库）。
pub struct ObserverEntry {
    pub observer_code_file: &'static str,
    pub observer_name: &'static str,
    /// 编译目标：观察者写在测试目标里时是 `test:<测试目标>`（几个测试目标共用的搭建模块，挑一个编进它的）。
    pub target: &'static str,
    pub entry_functions: &'static [&'static str],
}

/// 观察者登记表。观察者代码写成闭包、闭包里只转调一个 fn 的，入口是被转调的那个 fn（闭包本身只是把另读的输入递过去，另读的输入
/// 另按摘要进它的版本）；代码整段写在测试函数的闭包里的，要先挪成 fn 才登记得上，没登记的取键拒算。
pub const OBSERVER_ENTRIES: &[ObserverEntry] = &[
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs",
        observer_name: "writable_mount_and_continuation",
        target: "test:crash_enumeration_multi_record_publish_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_crash_point_observers::observe_the_mount_continuation",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs",
        observer_name: "publish_boundary",
        target: "test:crash_enumeration_multi_record_publish_stream",
        entry_functions: &["singlefs_checker_tier::observe_the_publish_boundary"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs",
        observer_name: "publish_boundary",
        target: "test:crash_enumeration_record_spill_over_stream",
        entry_functions: &["singlefs_checker_tier::observe_the_publish_boundary"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
        observer_name: "every_state",
        target: "test:crash_enumeration_unlink_after_the_multi_record_publish_stream",
        entry_functions: &["singlefs_checker_tier::observe_every_state"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
        observer_name: "mount_continuation",
        target: "test:crash_enumeration_unlink_after_the_multi_record_publish_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_crash_point_observers::observe_the_mount_continuation",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs",
        observer_name: "crash_image_handed_back",
        target: "test:crash_amplification_planned_import_names_the_gpu_conclusions",
        entry_functions: &["singlefs_checker_tier::hand_back_the_crash_image"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs",
        observer_name: "system_configuration_slot_rewritten_past_the_limit",
        target: "test:crash_amplification_planned_import_names_the_gpu_conclusions",
        entry_functions: &[
            "singlefs_checker_tier::rewrite_the_first_system_configuration_slot_past_the_judge_table_limit",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs",
        observer_name: "import_probe",
        target: "test:crash_amplification_pipeline_records_checks_and_reuses",
        entry_functions: &["singlefs_checker_tier::count_each_state_for_the_import_probe"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs",
        observer_name: "residual_record_chain",
        target: "test:crash_enumeration_fixed_script_stream",
        entry_functions: &["singlefs_checker_tier::observe_the_chain_to_the_residual_record"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs",
        observer_name: "stale_tail",
        target: "test:crash_enumeration_fixed_script_stream",
        entry_functions: &["singlefs_checker_tier::observe_the_stale_tail"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs",
        observer_name: "torn_slots",
        target: "test:crash_segments_per_device_and_torn_in_place_overwrites",
        entry_functions: &["singlefs_checker_tier::count_the_torn_system_configuration_slots"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs",
        observer_name: "slot_one_at_8192",
        target: "test:crash_enumeration_writable_mount_of_an_io_min_8192_pool",
        entry_functions: &["singlefs_checker_tier::observe_slot_one_at_8192"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs",
        observer_name: "f5_after_the_deletion",
        target: "test:crash_points_of_the_instance_table_sweep",
        entry_functions: &["singlefs_checker_tier::count_the_states_where_the_deletion_took_effect"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs",
        observer_name: "remount_after_the_sweep_phases",
        target: "test:crash_points_of_the_instance_table_sweep",
        entry_functions: &[
            "singlefs_checker_tier::remount_after_the_sweep_phases_without_the_deleted_row",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs",
        observer_name: "rebuilt_instance_table_against_the_allocation_records",
        target: "test:crash_points_of_the_instance_table_sweep",
        entry_functions: &[
            "singlefs_checker_tier::judge_the_rebuilt_instance_table_against_the_allocation_records",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_formatted_pool_remounted_without_a_file_stream.rs",
        observer_name: "remount_and_publish_the_first_file",
        target: "test:crash_enumeration_formatted_pool_remounted_without_a_file_stream",
        entry_functions: &["singlefs_checker_tier::remount_and_publish_the_first_file_after_the_crash"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs",
        observer_name: "remount_after_the_crash",
        target: "test:crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream",
        entry_functions: &[
            "singlefs_checker_tier::remount_after_the_crash_with_an_instance_above_every_one_on_the_disks",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_fixed_script_stream.rs",
        observer_name: "mount_continuation",
        target: "test:crash_enumeration_unlink_after_the_fixed_script_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_crash_point_observers::observe_the_mount_continuation",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs",
        observer_name: "count_each_state_once",
        target: "test:crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume",
        entry_functions: &["singlefs_checker_tier::count_each_state_once"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs",
        observer_name: "allocation_generation_read_sets",
        target: "test:crash_enumeration_new_pool_file_creation_stream",
        entry_functions: &["singlefs_checker_tier::count_the_allocation_generation_read_set"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs",
        observer_name: "writable_mount_after_the_crash",
        target: "test:crash_enumeration_floor_raise_pushed_by_admission",
        entry_functions: &["singlefs_checker_tier::mount_writable_after_the_crash_of_the_floor_raise"],
    },
    // 里程碑四那几条流的观察者（截断第二轮截壬补登：同步主线那一批立了这张表，里程碑四的这几条没登进来，跑到它们时取键拒算）。
    // 崩溃点重放那一路：观察者代码文件是设点的用例文件（`crash_state_enumeration` 的 observers_on_the_nodes_of_the_item），用例里的闭包只转调
    // 公共模块里的一个 fn，入口是被转调的那个。
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_truncate_to_zero_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_to_zero_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_truncate_into_a_unit_then_extend_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_into_a_unit_then_extend_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_truncate_under_a_snapshot_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_compaction_relocation_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_compaction_relocation_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_lifecycle_closure_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_lifecycle_closure_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_creation_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_snapshot_creation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_under_a_snapshot_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_under_a_snapshot_stream.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_unlink_under_a_snapshot_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_snapshot_destruction_resumed_after_a_crash_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    // 崩溃放量那一路：观察者代码文件是观察者所在的公共模块（`common_snapshot_crash_point_observers::amplify_a_snapshot_flow`、
    // `common_compaction_crash_point_observers::amplify_a_compaction_flow`），几个测试目标共用，挑一个编进它的。
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_snapshot_crash_point_observers/mod.rs",
        observer_name: "snapshot_of_the_landed_version",
        target: "test:crash_enumeration_snapshot_creation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_snapshot_of_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_snapshot_crash_point_observers/mod.rs",
        observer_name: "mount_continuation_under_a_snapshot",
        target: "test:crash_enumeration_snapshot_creation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_snapshot_crash_point_observers::observe_the_mount_continuation_under_a_snapshot",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_compaction_crash_point_observers/mod.rs",
        observer_name: "landed_version_read_through_the_mapping",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_landed_version_reading_through_the_mapping",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_compaction_crash_point_observers/mod.rs",
        observer_name: "mount_continuing_the_compaction",
        target: "test:crash_enumeration_compaction_relocation_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_compaction_crash_point_observers::observe_the_mount_continuing_the_compaction",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_truncation_crash_streams/mod.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_truncate_to_zero_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_truncation_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_truncation_crash_streams/mod.rs",
        observer_name: "mount_continuing_the_truncation",
        target: "test:crash_enumeration_truncate_to_zero_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_truncation_crash_streams::observe_the_mount_continuing_the_truncation",
        ],
    },
    // E161 的 S3 观察者（4-2a 报告第四节第 6 条）：装置里抽成具名 fn，跑到它时不再走拒算
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs",
        observer_name: "e161_crates_comparison",
        target: "bin:e161_crash_state_dedup_and_time_split",
        entry_functions: &["singlefs_checker_tier::observe_the_crates_comparison"],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_several_inodes_overwritten_in_turn_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_several_inodes_overwritten_in_turn_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_several_inodes_overwritten_in_turn_stream.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_several_inodes_overwritten_in_turn_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_large_file_overwritten_across_lower_leaves_stream.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_large_file_overwritten_across_lower_leaves_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_large_file_overwritten_across_lower_leaves_stream.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_large_file_overwritten_across_lower_leaves_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_unlink_of_two_files_holding_data_streams",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_unlink_of_two_files_holding_data_streams",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_multiple_copy_on_write_crash_streams/mod.rs",
        observer_name: "landed_version",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_landed_version",
        ],
    },
    ObserverEntry {
        observer_code_file: "crates/singlefs-checker-tier/tests/common_multiple_copy_on_write_crash_streams/mod.rs",
        observer_name: "mount_continuation_and_files_read_back",
        target: "test:crash_enumeration_one_file_overwritten_across_two_writable_remounts_stream",
        entry_functions: &[
            "singlefs_checker_tier::common_multiple_copy_on_write_crash_streams::observe_the_mount_continuation_and_the_files_read_back",
        ],
    },
];

/// 观察者那一组的组名（只当索引）：`observer_code:<观察者代码文件>:<观察者名>`。
#[must_use]
pub fn observer_group_name(observer_code_file: &str, observer_name: &str) -> String {
    format!("observer_code:{observer_code_file}:{observer_name}")
}

/// 判器的闭包里不许有的流水线文件（停止表退役之后留下的那条设计约束，只给测试断言用，不进键）：录写表、排计划、领块、存库、对账、
/// 工具链身份、几类库与两段协议、崩溃枚举用例的两段。录入与观察者结论拼块两组本来就在流水线里，不受这一条管。
pub const PIPELINE_FILES_THE_JUDGES_MUST_NOT_REACH: &[&str] = &[
    "crates/singlefs-checker-tier/src/crash_amplification.rs",
    "crates/singlefs-checker-tier/src/verdict_store.rs",
    "crates/singlefs-checker-tier/src/crash_point_replay_progress.rs",
    "crates/singlefs-checker-tier/src/crash_library.rs",
    "crates/singlefs-checker-tier/src/crash_state_enumeration.rs",
];

/// 只有 GPU 那两组（GPU 判器、GPU 核对）许够到的卡上那一套：派活与起卡、读显存。CPU 那几组的闭包里一个都不许有。
pub const GPU_FILES_ONLY_THE_GPU_GROUPS_REACH: &[&str] = &[
    "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs",
    "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
];
