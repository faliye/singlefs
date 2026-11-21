//! 崩溃放量节点的身份与复用键（里程碑二「增补 4」第一项 `research/prompts/m3-prune-gpu-r3-main-verification.md`
//! 第四节的设计表认了身份形态；E164（崩溃放量新设计的小范围功能测试） 第一、二段量过各件）。
//!
//! - 键只由内容定（键里不许有路径、名字、种子、行号、手写版本号、整文件或整表指纹、按提交算的东西）：
//!   块键 = 这一步带屏障、FUA、盘、偏移、段号的写表与它起步的镜像（扇区终值）的摘要，加段展开、判器另读的输入与段内序号
//!   （`verdict_store::VerdictBlockKey`）。崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹：判器（GPU 判器、CPU 判器、
//!   池级 checker、记录核对器）、观察者与录制管线自己的代码都不进键，判器判出的那几种结论的版本由结论种类与判器引到的被检测产品代码定
//!   （[`ConclusionVersion::of_the_crash_image_judgement`] 一族，[`judged_product_code_key`]），同一块上几个观察者的结论按各自的判定项标识与
//!   各自引到的被检测产品代码分开（[`observer_product_code_key_in`]）。这几组都只经 [`judging_units_key`] 一族入口取（[`JudgingUnitsKey`]，
//!   判法单元指纹）。录入键不是判法单元：只由录下的写流摘要与场景标识组成，代码一个单元都不进
//!   （`crate::crash_library::recording_key_of_the_scenario_and_the_inputs`）。
//! - 身份 `<设这个崩溃点的文件的仓内路径>::<崩溃点名称>::<复用键>` 只作索引：进路径索引表与覆盖报告，不进任何键。
//!   标签类型不给格式化、不给比较（没有 `Display`、`Debug`、`Clone`、`Hash`、`PartialEq`、`Ord`），不能拿去当键（E164 F12）。
//! - 判法单元指纹：构建脚本按入口登记表（`crate::judging_units_entries`）经 `singlefs-unit-graph` 逐组从入口 fn 收函数单元闭包、
//!   算键（单元去注释的指纹与闭包真引到的 cfg 原子；工具链、`Cargo.lock`、编译档与线程数都不进），连同组成清单写进 `OUT_DIR`；
//!   这里只读编进二进制的那一份，跑时不读盘上的树。二进制比盘上的树旧时它的键对不上新树算出来的键，查库不中就现判。

use std::collections::{BTreeMap, BTreeSet};

use crate::hexadecimal::hexadecimal_text;
use crate::memory_pool::{MemoryPool, RetainedWrite, WrittenContents, SECTOR_BYTES};
use crate::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_unit_graph::entry_groups::{
    parse_entry_group_table, ReadBackEntryGroup, ReadBackEntryGroupTable, ReadBackEntryGroupVerdict,
};
use singlefs_unit_graph::FeaturesEnabledByTheBuild;

use crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity;

mod compiled_judging_units_key_table;
mod conclusion_kinds_and_versions;
mod crash_point_labels_and_reuse_keys;
mod judging_units_key_and_composition;
mod step_write_table_and_parent_image;

pub use compiled_judging_units_key_table::*;
pub use conclusion_kinds_and_versions::*;
pub use crash_point_labels_and_reuse_keys::*;
pub use judging_units_key_and_composition::*;
pub use step_write_table_and_parent_image::*;
