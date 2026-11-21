//! 实验包装置自己的判法单元入口登记表：装置先录后判的 CPU 参照那几组、装置挂在崩溃枚举上的观察者，每一组从哪几个入口 fn 收闭包。
//! 构建脚本（`build.rs`）经 `#[path]` 编同一份，经 `singlefs-unit-graph` 逐组算键与组成清单、写进 `OUT_DIR`（与验证包的构建脚本同一套写法），
//! 库按组名读回编进去的那一张（`crate::JUDGING_UNITS_KEYS_TABLE_TEXT`），跑时不读盘上的树。表里的文字（组名、路径、观察者名）只用来找入口，不进键。
//! 这份文件只放数据与拼组名的函数，只用 `std`：构建脚本单独编它。
//!
//! 一组的入口都在同一个编译目标里（这个包的装置都是 `bin:<名>`），入口 fn 路径的头一段是这个包的 crate 名。

/// 一组判法单元的入口：装置先录后判的参照那一侧。
pub struct JudgingUnitsEntryGroup {
    /// 组名：装置取键时交的那一个（`singlefs_verification::crash_identity::judging_units_key_of_the_group_in_the_table_text`）。
    pub group_name: &'static str,
    /// 编译目标：`bin:<名>`。
    pub target: &'static str,
    /// 入口 fn 路径：`singlefs_implementation_experiments::<模块路径>::<fn>`。
    pub entry_functions: &'static [&'static str],
}

/// 装置先录后判的参照那几组。
///
/// - E185（GC一批搬迁的挂钟拆分）装置先录后判的 CPU 参照（`e185_cpu_reference_crcs`）：只录那一段在 CPU 上算参照 CRC 的两个函数
///   （gpu-submission 一个数据单元两种段长、带卡单测按种子造的槽），键拼进录入键，参照的代码改了库里旧的那一份参照不再被读到。
pub const JUDGING_UNITS_ENTRY_GROUPS: &[JudgingUnitsEntryGroup] = &[JudgingUnitsEntryGroup {
    group_name: "e185_cpu_reference_crcs",
    target: "bin:e185_gc_batch_wall_clock_split",
    entry_functions: &[
        "singlefs_implementation_experiments::cpu_reference_of_the_submission_unit",
        "singlefs_implementation_experiments::cpu_reference_of_the_slots",
    ],
}];

/// 一个观察者的入口：观察者代码文件（装置交给崩溃枚举的项的设点文件）与观察者名对到它的入口 fn。观察者的身份就是这几个入口 fn 的闭包键。
pub struct ObserverEntry {
    pub observer_code_file: &'static str,
    pub observer_name: &'static str,
    /// 编译目标：`bin:<名>`。
    pub target: &'static str,
    pub entry_functions: &'static [&'static str],
}

/// 装置挂在崩溃枚举上的观察者。观察者代码写成闭包、闭包里只转调一个 fn 的，入口是被转调的那个 fn（闭包本身只是把另读的输入递过去，
/// 另读的输入另按摘要进它的版本）。装置经 `singlefs_verification::crash_state_enumeration::enumerate_crash_states_with_expansion_keying_the_observers_in`
/// 把编进来的那一张（`crate::OBSERVER_KEYS_OF_THE_EXPERIMENTS`）交给崩溃枚举；没登记的取键拒算（照判、不入库）。
pub const OBSERVER_ENTRIES: &[ObserverEntry] = &[
    // E161（崩溃放量的去重与分段耗时） 的 S3 观察者：装置里抽成具名 fn，跑到它时不走拒算
    ObserverEntry {
        observer_code_file: "crates/singlefs-implementation-experiments/src/bin/e161_crash_state_dedup_and_time_split.rs",
        observer_name: "e161_crates_comparison",
        target: "bin:e161_crash_state_dedup_and_time_split",
        entry_functions: &["singlefs_implementation_experiments::observe_the_crates_comparison"],
    },
];

/// 观察者那一组的组名（只当索引）：`observer_code:<观察者代码文件>:<观察者名>`。崩溃枚举照
/// `singlefs_verification::judging_units_entries::observer_group_name` 拼同一个名字去表里找；两边逐字相同由库的单测
/// `the_observer_group_names_are_the_ones_the_crash_enumeration_looks_up` 核。
#[must_use]
pub fn observer_group_name(observer_code_file: &str, observer_name: &str) -> String {
    format!("observer_code:{observer_code_file}:{observer_name}")
}
