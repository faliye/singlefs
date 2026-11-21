//! 实验装置 E202（GPU与CPU两种模式GC对照组）那条内联真卡测试对拍里参照一侧（在 CPU 上算、只录那一段写进判定库）的判法单元入口登记表：
//! 一组一行，从哪个编译目标里的哪个 fn 收闭包。构建脚本（`build.rs`）经 `#[path]` 编同一份，经 `singlefs-unit-graph` 逐组算键、写进 `OUT_DIR`；
//! 装置按组名读回编进去的那一把（`tests/common_pairing_input_through_the_library/mod.rs`），拼进对拍参照的录入键与对拍结论的键：
//! 参照的代码改了，录入键变，库里旧的参照不再被读到，旧的结论也不再被复用。GPU 一侧的入口登记在 `src/gpu_side_units_entries.rs`，两张表用这里的同一种行。
//! 这份文件只放数据，只用 `std`：构建脚本单独编它。

/// 对拍一侧的一组判法单元入口（参照一侧与 GPU 一侧两张登记表共用这一种行）。
pub struct PairingSideUnitsEntryGroup {
    /// 组名：取键时点的名字。
    pub group_name: &'static str,
    /// 编译目标：`bin:<名>`（`src/bin/` 下那份实验装置里的入口）。
    pub target: &'static str,
    /// 入口 fn 路径：`singlefs_background_gpu_tests::<模块路径>::<fn>`。
    pub entry_functions: &'static [&'static str],
}

/// 实验装置 E202（GPU与CPU两种模式GC对照组）里装置自数的每段活槽数。
pub const REFERENCE_UNITS_ENTRY_GROUPS: &[PairingSideUnitsEntryGroup] = &[
    PairingSideUnitsEntryGroup {
        group_name: "apparatus_count_of_the_live_slots_per_segment_in_the_gpu_cpu_gc_comparison",
        target: "bin:e202_gpu_cpu_gc_comparison",
        entry_functions: &[
            "singlefs_background_gpu_tests::reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment",
        ],
    },
];

/// 收闭包、算键时交的特性：要真显卡的内联测试只在开了它时编，不论这一次开没开都照开着收（两种编法同一把键）。两张登记表都照它收。
pub const FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH: &str = "tests-on-a-real-gpu";
