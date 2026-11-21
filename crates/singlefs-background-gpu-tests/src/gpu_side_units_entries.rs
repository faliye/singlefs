//! 实验装置 E202（GPU与CPU两种模式GC对照组）那条内联真卡测试对拍里 GPU 一侧（判段持卡那一段：开卡、跑 GPU、与读回的参照比）的判法单元入口登记表：
//! 组名就是那条测试的名字，入口是它的持卡判 fn（`judged_on_the_cards_that_<测试名>`，在那份 bin 里，目标写 `bin:<名>`）。闭包从它收：开卡与取卡、
//! 产品 GPU 包里它真走到的 Rust 与 WGSL、读回的输入与参照怎么解、怎么比。构建脚本（`build.rs`）经 `#[path]` 编同一份，照参照一侧那张表同一个算法
//! 逐组算键、写进 `OUT_DIR/gpu_side_units_keys.tsv`；装置按测试名读回，与参照一侧那一把、读回输入的摘要一起组成对拍结论的键
//! （`tests/common_pairing_input_through_the_library/mod.rs`）：GPU 一侧走的代码或比法改了，键变，库里旧的结论不再被复用。
//! 这份文件只放数据，只用 `std`：构建脚本单独编它（行的写法在 `src/reference_units_entries.rs`）。

use crate::reference_units_entries::PairingSideUnitsEntryGroup;

/// 实验装置 E202 那条内联真卡测试的持卡判入口。
pub const GPU_SIDE_UNITS_ENTRY_GROUPS: &[PairingSideUnitsEntryGroup] = &[
    PairingSideUnitsEntryGroup {
        group_name: "the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool",
        target: "bin:e202_gpu_cpu_gc_comparison",
        entry_functions: &["singlefs_background_gpu_tests::judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool"],
    },
];
