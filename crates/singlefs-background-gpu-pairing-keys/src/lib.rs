//! 产品后台 GPU 包真卡对拍两侧（参照一侧、GPU 一侧）的判法单元键表：本包构建脚本照入口登记表
//! （`crates/singlefs-background-gpu/tests/pairing_units_entries.tsv`）逐组算好、写进 `OUT_DIR`，这里只交编进来的那两份原文。
//! 录入键不用这里的表：它只由场景标识与输入摘要组成，代码一个单元都不进。
//! 表的写法在 `singlefs-unit-graph` 的 `entry_groups` 文件头，读回用那里的 `parse_entry_group_table`。

#![deny(unsafe_code)]

/// 参照一侧那张表（在 CPU 上算、只录那一段写进判定库的参照）。
pub const REFERENCE_UNITS_KEYS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/reference_units_keys.tsv"));

/// GPU 一侧那张表（组名就是真卡对拍用例的名字）。
pub const GPU_SIDE_UNITS_KEYS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/gpu_side_units_keys.tsv"));
