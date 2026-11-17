//! 对拍里参照一侧（在 CPU 上算、只录那一段写进判定库）的判法单元入口登记表：一组一行，从哪个测试目标里的哪个 fn 收闭包。
//! 构建脚本（`build.rs`）经 `#[path]` 编同一份，经 `singlefs-unit-graph` 逐组算键、写进 `OUT_DIR`；测试按组名读回编进去的那一把
//! （`tests/common_pairing_input_through_the_library`），拼进对拍参照的录入键与对拍结论的键：参照的代码改了，录入键变，库里旧的参照不再被读到，
//! 旧的结论也不再被复用。GPU 一侧的入口登记在 `src/gpu_side_units_entries.rs`，两张表用这里的同一种行。
//! 这份文件只放数据，只用 `std`：构建脚本单独编它。

/// 对拍一侧的一组判法单元入口（参照一侧与 GPU 一侧两张登记表共用这一种行）。
pub struct PairingSideUnitsEntryGroup {
    /// 组名：测试取键时点的名字。
    pub group_name: &'static str,
    /// 编译目标：`test:<测试目标>`，或 `bin:<名>`（`src/bin/` 下那份实验装置里的入口）。
    pub target: &'static str,
    /// 入口 fn 路径：`singlefs_background_gpu_tests::<模块路径>::<fn>`。
    pub entry_functions: &'static [&'static str],
}

/// 校验和核、活槽核、区域记录核三组的 CPU 参照，真卡整理与挂载运行库两组里选 `cpu` 的那一趟，交一批校验和那一条的 CPU 参照，
/// 点名不在本机的卡那一条在 CPU 上写下的拒法，实验装置 E202（GPU与CPU两种模式GC对照组）里装置自数的每段活槽数。
pub const REFERENCE_UNITS_ENTRY_GROUPS: &[PairingSideUnitsEntryGroup] = &[
    PairingSideUnitsEntryGroup {
        group_name: "cpu_reference_of_the_checksum_kernels",
        target: "test:checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &[
            "singlefs_background_gpu_tests::common_checksum_batches_over_the_handed_over_gpus::reference_bytes_of_the_checksum_pairing_input",
        ],
    },
    PairingSideUnitsEntryGroup {
        group_name: "cpu_reference_of_the_live_slot_kernels",
        target: "test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_live_slot_pairing_input",
        ],
    },
    PairingSideUnitsEntryGroup {
        group_name: "cpu_reference_of_the_region_record_kernels",
        target: "test:region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_region_pairing_input",
        ],
    },
    PairingSideUnitsEntryGroup {
        group_name: "cpu_path_of_the_compaction_run",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_compaction_run_on_the_cpu",
            "singlefs_background_gpu_tests::reference_bytes_of_the_units_narrowed_to_one_card",
        ],
    },
    PairingSideUnitsEntryGroup {
        group_name: "cpu_refusal_of_the_card_not_on_this_host",
        target: "test:compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_refusal_of_the_card_not_on_this_host",
        ],
    },
    PairingSideUnitsEntryGroup {
        group_name: "cpu_path_of_the_mount_runtime",
        target: "test:mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_runtime_on_the_cpu",
        ],
    },
    PairingSideUnitsEntryGroup {
        group_name: "apparatus_count_of_the_live_slots_per_segment_in_the_gpu_cpu_gc_comparison",
        target: "bin:e202_gpu_cpu_gc_comparison",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment",
        ],
    },
];

/// 收闭包、算键时交的特性：这些测试目标只在开了它时编，不论这一次开没开都照开着收（两种编法同一把键）。两张登记表都照它收。
pub const FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH: &str = "tests-on-a-real-gpu";
