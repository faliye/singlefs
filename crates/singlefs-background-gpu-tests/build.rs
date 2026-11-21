//! 编译时按实验装置 E202（GPU与CPU两种模式GC对照组）那条内联真卡测试对拍两侧的入口登记表逐组算判法单元键与组成清单：参照一侧
//! （`src/reference_units_entries.rs`）写进 `OUT_DIR/reference_units_keys.tsv`，GPU 一侧（`src/gpu_side_units_entries.rs`）写进
//! `OUT_DIR/gpu_side_units_keys.tsv`，两张表经 `#[path]` 编同一份、走同一个算法（[`keys_of_the_entry_groups`]）。装置经
//! `include_str!(concat!(env!("OUT_DIR"), …))` 读回编进去的那两份，跑时不读盘上的树。
//! 做法照验证包的构建脚本（`crates/singlefs-verification/build.rs`）：切分、闭包、键、表的写法都在 `singlefs-unit-graph` 里，
//! 这里只取 cargo 给的（依赖包在这一次编译里开了哪些特性、`TARGET`）；工具链与编译档不读、不进键，`Cargo.lock` 不进键（照旧盯着；D13（验证路线） 已定项 17），
//! 表里 `toolchain` 那一行只记收闭包时照的目标。
//! 每一组收闭包时都照开着 `tests-on-a-real-gpu` 收（登记表的 `FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH`）。

#[path = "src/gpu_side_units_entries.rs"]
mod gpu_side_units_entries;
#[path = "src/reference_units_entries.rs"]
mod reference_units_entries;

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

use reference_units_entries::PairingSideUnitsEntryGroup;

/// 参照一侧那份表写进 `OUT_DIR` 的文件名（测试按同一个名字读回）。
const REFERENCE_UNITS_KEYS_FILE_NAME: &str = "reference_units_keys.tsv";
/// GPU 一侧那份表写进 `OUT_DIR` 的文件名（测试按同一个名字读回）。
const GPU_SIDE_UNITS_KEYS_FILE_NAME: &str = "gpu_side_units_keys.tsv";

fn environment_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("cargo 起构建脚本时设 {name}"))
}

/// 一张登记表算键时要的、两张表共用的那几样：工作区、依赖包这一次开的特性、收闭包照的目标三元组、规则开关（工具链与 `Cargo.lock` 不进键）。
struct InputsOfTheEntryGroupKeys<'build> {
    workspace: &'build Workspace,
    features_enabled_by_the_build: &'build FeaturesEnabledByTheBuild,
    target_triple: &'build str,
    switches: &'build RuleSwitches,
}

/// 一张登记表逐组收闭包、算键（每组照开着 `FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH` 收）。
///
/// # Panics
/// 登记表里有一组的目标写法认不得、入口写错了。
fn keys_of_the_entry_groups(
    inputs: &InputsOfTheEntryGroupKeys<'_>,
    groups: &[PairingSideUnitsEntryGroup],
) -> Vec<EntryGroupKey> {
    groups
        .iter()
        .map(|group| {
            let request = EntryGroupRequest {
                target: TargetSelector::parse(group.target).unwrap_or_else(|text| {
                    panic!("登记表里组 {} 的目标写法认不得：{text}", group.group_name)
                }),
                entry_paths: group
                    .entry_functions
                    .iter()
                    .map(|entry| (*entry).to_string())
                    .collect(),
                features: vec![
                    reference_units_entries::FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH
                        .to_string(),
                ],
                group_name: group.group_name.to_string(),
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

fn main() {
    let manifest_directory = PathBuf::from(environment_variable("CARGO_MANIFEST_DIR"));
    // 包目录往上两层就是仓根（crates/<包>）：用 parent() 取，不写往上的相对路径字面量
    let repository_root = manifest_directory
        .parent()
        .and_then(Path::parent)
        .expect("包目录在 crates/ 下，往上两层是仓根")
        .to_path_buf();
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
        "crates/singlefs-background-gpu-tests/build.rs",
        "crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs",
        "crates/singlefs-background-gpu-tests/src/reference_units_entries.rs",
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
    for (groups, file_name) in [
        (
            reference_units_entries::REFERENCE_UNITS_ENTRY_GROUPS,
            REFERENCE_UNITS_KEYS_FILE_NAME,
        ),
        (
            gpu_side_units_entries::GPU_SIDE_UNITS_ENTRY_GROUPS,
            GPU_SIDE_UNITS_KEYS_FILE_NAME,
        ),
    ] {
        let keys = keys_of_the_entry_groups(&inputs, groups);
        let output_path = output_directory.join(file_name);
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
