//! 函数单元图：Rust 与 WGSL 源码按条目切成单元、取去注释指纹，从入口按名字与编译目标收闭包、算复用键。
//!
//! 设计是剪枝复查第二轮 design 路报告第三节（3.1 键、3.2 切法、3.3 收法、3.4 拒算、3.5 WGSL、3.8 放法），
//! 由 `records/2026-10-13-复用键与全仓改名.md` 的「2026-10-15 批4」4-1a 落进来。只用 std，不依赖工作区任何 crate：
//! `singlefs-checker-tier` 的库与构建脚本以后都经 Cargo 依赖它（4-2a）。
//!
//! 用法：`load_workspace` 装一个工作区（仓根或 `crates/` 目录），`compute_closure` 从一个入口收闭包，`reuse_key` 算键；
//! 依赖行可以由调用方收窄好交进来，或用 `lock_file::narrowed_lock_lines` 从整份 `Cargo.lock` 只取闭包真引到的外部包；
//! `output` 里是 `unit-graph` 命令行打的那几种行。切分按文件切片并行，线程数由调用方给，多线程与单线程输出逐字相同。
//! `entry_groups` 按一组入口算键与组成清单、写成构建输出目录里的那份表（`singlefs-checker-tier` 的构建脚本用，4-2a）；
//! `build_toolchain` 拼一次构建的工具链行（`rustc -vV`、`cargo -V`、目标、RUSTFLAGS、profile 的生效值）。
//! 弄坏开关在 `rule_switches`，只给测试用。
#![forbid(unsafe_code)]

pub mod build_toolchain;
pub mod closure;
pub mod conditional_compilation;
pub mod entry_groups;
pub mod heavy_threads;
pub mod lexer;
pub mod lock_file;
pub mod manifest;
pub mod output;
pub mod rule_switches;
pub mod rust_items;
pub mod sha256;
pub mod units;
pub mod wgsl;
pub mod workspace;

pub use build_toolchain::{
    effective_profile_lines, profile_name_of_the_build_script_profile_variable,
    toolchain_lines_of_the_build, BuildToolchainInputs,
};
pub use closure::{
    compute_closure, compute_closure_of_entries, reuse_key, ClosureOutcome, ClosureRequest,
    EntryError, ExternalKeyInputs, RequestedCondition,
};
pub use conditional_compilation::target_triple_of_toolchain_lines;
pub use entry_groups::{
    entry_group_key, entry_group_table_text, parse_entry_group_table, EntryGroupKey,
    EntryGroupRequest, EntryGroupVerdict, ReadBackEntryGroup, ReadBackEntryGroupTable,
    ReadBackEntryGroupVerdict,
};
pub use lock_file::{narrowed_lock_lines, parse_lock_file, LockFileError, LockedPackage};
pub use rule_switches::{BrokenRule, RuleSwitches, BREAK_ENVIRONMENT_VARIABLE};
pub use workspace::{load_workspace, TargetSelector, Workspace, WorkspaceError};
