//! 编译时按产品后台 GPU 包的入口登记表（[`PAIRING_UNITS_ENTRIES_PATH`]，制表符隔开的数据文件，按路径读，不编别的包的 `.rs`）
//! 逐组算判法单元键与组成清单，两侧各写进 `OUT_DIR` 的一份表：参照一侧 `reference_units_keys.tsv`、GPU 一侧 `gpu_side_units_keys.tsv`，
//! 两张表走同一个算法（[`keys_of_the_entry_groups`]）。库经 `include_str!(concat!(env!("OUT_DIR"), …))` 读回编进去的那两份，
//! 产品 GPU 包的测试跑时不读盘上的树。录入键不在这里算：它只由场景标识与输入摘要组成，代码一个单元都不进
//! （`crates/singlefs-background-gpu/tests/common_pairing_input_through_the_library/mod.rs` 的 `recording_key_of_the_scenario_and_the_inputs`）。
//! 做法照验证包的构建脚本（`crates/singlefs-verification/build.rs`）：切分、闭包、键、表的写法都在 `singlefs-unit-graph` 里，
//! 这里只取 cargo 给的（依赖包在这一次编译里开了哪些特性、`TARGET`）；工具链与编译档不读、不进键，`Cargo.lock` 不进键（照旧盯着；D13（验证路线） 已定项 17），
//! 表里 `toolchain` 那一行只记收闭包时照的目标。每一组收闭包时照登记表那一行的特性列请求特性（真卡测试目标只在开了 `tests-on-a-real-gpu` 时编，
//! 不论这一次开没开都照开着收：两种编法同一把键）。
//!
//! 登记表一行一组，五列制表符隔开：侧（`reference`、`gpu-side`）、组名、编译目标（`test:<测试目标>`）、
//! 请求的特性（逗号隔开）、入口 fn 路径（`<包的库名>::<模块路径>::<fn>`，几个时逗号隔开）。`#` 起头的行与空行是说明。
//! 写坏的行构建脚本当场停下、报出行号；组名重复、两组登记同一个入口这类不停（产品 GPU 包的测试逐组核，那里判红）。

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use singlefs_unit_graph::build_features::{
    features_enabled_on_the_dependencies, FeaturesEnabledByTheBuild,
};
use singlefs_unit_graph::entry_groups::{
    entry_group_key_with_the_features_of_the_build,
    entry_group_table_text_with_the_features_of_the_build, target_line_of_the_table, EntryGroupKey,
    EntryGroupRequest,
};
use singlefs_unit_graph::{load_workspace, RuleSwitches, TargetSelector, Workspace};

/// 入口登记表，从仓根起的路径。
const PAIRING_UNITS_ENTRIES_PATH: &str =
    "crates/singlefs-background-gpu/tests/pairing_units_entries.tsv";

/// 登记表一行的列数。
const COLUMNS_OF_A_REGISTERED_GROUP: usize = 5;

/// 登记表一行是对拍的哪一侧（决定写进哪一张键表）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairingSide {
    /// 在 CPU 上算、只录那一段写进判定库的参照。
    Reference,
    /// 判段持卡那一段（开卡、跑 GPU、与读回的参照比），组名就是真卡对拍用例的名字。
    GpuSide,
}

/// 两侧，按写表的次序。
const PAIRING_SIDES: [PairingSide; 2] = [PairingSide::Reference, PairingSide::GpuSide];

impl PairingSide {
    /// 登记表第一列的写法换成侧；认不得的是 `None`。
    fn of_the_registry_word(word: &str) -> Option<Self> {
        match word {
            "reference" => Some(Self::Reference),
            "gpu-side" => Some(Self::GpuSide),
            _ => None,
        }
    }

    /// 这一侧的键表写进 `OUT_DIR` 的文件名（库按同一个名字读回）。
    fn keys_file_name(self) -> &'static str {
        match self {
            Self::Reference => "reference_units_keys.tsv",
            Self::GpuSide => "gpu_side_units_keys.tsv",
        }
    }
}

/// 登记表里的一组：哪一侧、组名、编译目标、请求的特性、入口 fn 路径。
struct RegisteredEntryGroup {
    side: PairingSide,
    group_name: String,
    target: String,
    features: Vec<String>,
    entry_functions: Vec<String>,
}

/// 逗号隔开的一列拆成各项（去掉每项两头的空白）。
fn items_of_a_comma_separated_column(column: &str) -> Vec<String> {
    column
        .split(',')
        .map(|item| item.trim().to_string())
        .collect()
}

/// 读登记表的原文，一行一组。
///
/// # Panics
/// 有一行不是五列、侧认不得、组名或目标是空的、特性或入口里有空项：登记表写坏了，构建停下、报出行号。
fn registered_entry_groups(registry_text: &str) -> Vec<RegisteredEntryGroup> {
    registry_text
        .lines()
        .enumerate()
        .filter(|(_line_index, line)| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|(line_index, line)| {
            let line_number = line_index + 1;
            let columns: Vec<&str> = line.split('\t').collect();
            assert!(
                columns.len() == COLUMNS_OF_A_REGISTERED_GROUP,
                "{PAIRING_UNITS_ENTRIES_PATH} 第 {line_number} 行不是 {COLUMNS_OF_A_REGISTERED_GROUP} 列（侧、组名、目标、特性、入口）：{line:?}"
            );
            let side = PairingSide::of_the_registry_word(columns[0]).unwrap_or_else(|| {
                panic!(
                    "{PAIRING_UNITS_ENTRIES_PATH} 第 {line_number} 行的侧「{}」认不得：只认 reference、gpu-side",
                    columns[0]
                )
            });
            let features = items_of_a_comma_separated_column(columns[3]);
            let entry_functions = items_of_a_comma_separated_column(columns[4]);
            assert!(
                !columns[1].is_empty()
                    && !columns[2].is_empty()
                    && features.iter().all(|feature| !feature.is_empty())
                    && entry_functions.iter().all(|entry| !entry.is_empty()),
                "{PAIRING_UNITS_ENTRIES_PATH} 第 {line_number} 行有空的组名、目标、特性或入口：{line:?}"
            );
            RegisteredEntryGroup {
                side,
                group_name: columns[1].to_string(),
                target: columns[2].to_string(),
                features,
                entry_functions,
            }
        })
        .collect()
}

fn environment_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("cargo 起构建脚本时设 {name}"))
}

/// 两张表算键时共用的那几样：工作区、依赖包这一次开的特性、收闭包照的目标三元组、规则开关（工具链与 `Cargo.lock` 不进键）。
struct InputsOfTheEntryGroupKeys<'build> {
    workspace: &'build Workspace,
    features_enabled_by_the_build: &'build FeaturesEnabledByTheBuild,
    target_triple: &'build str,
    switches: &'build RuleSwitches,
}

/// 登记表里 `side` 那一侧的组逐组收闭包、算键（每组照它那一行的特性列请求特性），按登记表里的次序。
///
/// # Panics
/// 有一组的目标写法认不得、入口写错了（找不到、不唯一、不在同一个包里）。
fn keys_of_the_entry_groups(
    inputs: &InputsOfTheEntryGroupKeys<'_>,
    groups: &[RegisteredEntryGroup],
    side: PairingSide,
) -> Vec<EntryGroupKey> {
    groups
        .iter()
        .filter(|group| group.side == side)
        .map(|group| {
            let request = EntryGroupRequest {
                target: TargetSelector::parse(&group.target).unwrap_or_else(|text| {
                    panic!("登记表里组 {} 的目标写法认不得：{text}", group.group_name)
                }),
                entry_paths: group.entry_functions.clone(),
                features: group.features.clone(),
                group_name: group.group_name.clone(),
            };
            entry_group_key_with_the_features_of_the_build(
                inputs.workspace,
                &request,
                inputs.features_enabled_by_the_build,
                inputs.target_triple,
                inputs.switches,
            )
            .unwrap_or_else(|error| {
                panic!("登记表里组 {} 的入口写错了：{error}", request.group_name)
            })
        })
        .collect()
}

/// 读登记表、两侧逐组算键、各写进 `OUT_DIR` 的一份表，报要看的文件。
///
/// # Panics
/// cargo 该给的环境变量没给；登记表读不了或写坏了；工作区装不起；依赖包的特性取不全；
/// 表写不进 `OUT_DIR`。
fn write_the_keys_of_the_registered_entry_groups() {
    let manifest_directory = PathBuf::from(environment_variable("CARGO_MANIFEST_DIR"));
    // 包目录往上两层就是仓根（crates/<包>）：用 parent() 取，不写往上的相对路径字面量
    let repository_root = manifest_directory
        .parent()
        .and_then(Path::parent)
        .expect("包目录在 crates/ 下，往上两层是仓根")
        .to_path_buf();
    let registry_text = std::fs::read_to_string(repository_root.join(PAIRING_UNITS_ENTRIES_PATH))
        .unwrap_or_else(|error| panic!("读不了 {PAIRING_UNITS_ENTRIES_PATH}：{error}"));
    let groups = registered_entry_groups(&registry_text);
    let threads = std::env::var("NUM_JOBS")
        .or_else(|_| std::env::var("SINGLEFS_THREAD_CAP"))
        .ok()
        .and_then(|text| text.parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .unwrap_or(NonZeroUsize::MIN);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = load_workspace(&repository_root, threads, &switches)
        .unwrap_or_else(|error| panic!("装不起工作区：{error}"));
    let own_crate = workspace
        .crate_named(&environment_variable("CARGO_PKG_NAME"))
        .expect("工作区里有正在编的这个包");
    let read_environment = |name: &str| std::env::var(name).ok();
    let features_enabled_by_the_build =
        features_enabled_on_the_dependencies(&workspace, own_crate, &read_environment)
            .unwrap_or_else(|error| panic!("依赖包在这一次编译里开了哪些特性取不全：{error}"));
    let target_triple = environment_variable("TARGET");
    let inputs = InputsOfTheEntryGroupKeys {
        workspace: &workspace,
        features_enabled_by_the_build: &features_enabled_by_the_build,
        target_triple: &target_triple,
        switches: &switches,
    };
    let output_directory = PathBuf::from(environment_variable("OUT_DIR"));
    let mut watched: BTreeSet<String> = [
        "Cargo.lock",
        "Cargo.toml",
        "crates/singlefs-background-gpu-pairing-keys/build.rs",
        PAIRING_UNITS_ENTRIES_PATH,
    ]
    .iter()
    .map(|path| (*path).to_string())
    .collect();
    watched.extend(
        workspace
            .crates
            .iter()
            .map(|workspace_crate| workspace_crate.directory.join("Cargo.toml"))
            .filter_map(|path| {
                path.strip_prefix(&repository_root)
                    .ok()
                    .map(|relative| relative.display().to_string())
            }),
    );
    for side in PAIRING_SIDES {
        let keys = keys_of_the_entry_groups(&inputs, &groups, side);
        let output_path = output_directory.join(side.keys_file_name());
        std::fs::write(
            &output_path,
            entry_group_table_text_with_the_features_of_the_build(
                &[target_line_of_the_table(&target_triple)],
                &features_enabled_by_the_build,
                &keys,
            ),
        )
        .unwrap_or_else(|error| panic!("写不了 {}：{error}", output_path.display()));
        for key in &keys {
            watched.extend(key.watched_files.iter().cloned());
        }
    }
    for path in &watched {
        println!(
            "cargo:rerun-if-changed={}",
            repository_root.join(path).display()
        );
    }
}

fn main() {
    write_the_keys_of_the_registered_entry_groups();
}
