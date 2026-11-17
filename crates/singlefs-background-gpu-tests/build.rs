//! 编译时按对拍两侧的入口登记表逐组算判法单元键与组成清单：参照一侧（`src/reference_units_entries.rs`）写进
//! `OUT_DIR/reference_units_keys.tsv`，GPU 一侧（`src/gpu_side_units_entries.rs`）写进 `OUT_DIR/gpu_side_units_keys.tsv`，两张表经 `#[path]` 编同一份、
//! 走同一个算法（[`keys_of_the_entry_groups`]）。测试经 `include_str!(concat!(env!("OUT_DIR"), …))` 读回编进去的那两份，跑时不读盘上的树。
//! 做法照验证包的构建脚本（`crates/singlefs-verification/build.rs`）：切分、闭包、键、工具链行的拼法、表的写法都在 `singlefs-unit-graph` 里，
//! 这里只取 cargo 给的（依赖包在这一次编译里开了哪些特性、`$RUSTC -vV`、`$CARGO -V`、`TARGET`、`CARGO_ENCODED_RUSTFLAGS`、`PROFILE`，连同 `Cargo.lock`）。
//! 每一组收闭包时都照开着 `tests-on-a-real-gpu` 收（登记表的 `FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH`）。

#[path = "src/gpu_side_units_entries.rs"]
mod gpu_side_units_entries;
#[path = "src/reference_units_entries.rs"]
mod reference_units_entries;

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;

use singlefs_unit_graph::build_features::{
    features_enabled_on_the_dependencies, FeaturesEnabledByTheBuild,
};
use singlefs_unit_graph::build_toolchain::{
    effective_profile_lines, profile_environment_variable,
    profile_name_of_the_build_script_profile_variable, toolchain_lines_of_the_build,
    BuildToolchainInputs, PROFILE_SETTING_KEYS,
};
use singlefs_unit_graph::entry_groups::{
    entry_group_key_with_the_features_of_the_build,
    entry_group_table_text_with_the_features_of_the_build, EntryGroupKey, EntryGroupRequest,
};
use singlefs_unit_graph::{
    load_workspace, parse_lock_file, LockedPackage, RuleSwitches, TargetSelector, Workspace,
};

use reference_units_entries::PairingSideUnitsEntryGroup;

/// 参照一侧那份表写进 `OUT_DIR` 的文件名（测试按同一个名字读回）。
const REFERENCE_UNITS_KEYS_FILE_NAME: &str = "reference_units_keys.tsv";
/// GPU 一侧那份表写进 `OUT_DIR` 的文件名（测试按同一个名字读回）。
const GPU_SIDE_UNITS_KEYS_FILE_NAME: &str = "gpu_side_units_keys.tsv";

fn environment_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("cargo 起构建脚本时设 {name}"))
}

fn output_of(program: &str, argument: &str) -> String {
    let output = Command::new(program)
        .arg(argument)
        .output()
        .unwrap_or_else(|error| panic!("跑不了 {program} {argument}：{error}"));
    assert!(
        output.status.success(),
        "{program} {argument} 退 {:?}",
        output.status.code()
    );
    String::from_utf8(output.stdout)
        .unwrap_or_else(|_| panic!("{program} {argument} 的输出不是 UTF-8"))
}

/// 一张登记表算键时要的、两张表共用的那几样：工作区、依赖包这一次开的特性、工具链行、`Cargo.lock` 解出的包、规则开关。
struct InputsOfTheEntryGroupKeys<'build> {
    workspace: &'build Workspace,
    features_enabled_by_the_build: &'build FeaturesEnabledByTheBuild,
    toolchain_lines: &'build [String],
    locked_packages: &'build [LockedPackage],
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
                inputs.toolchain_lines,
                inputs.locked_packages,
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
    let profile_name =
        profile_name_of_the_build_script_profile_variable(&environment_variable("PROFILE"))
            .to_string();
    let root_manifest_text = std::fs::read_to_string(repository_root.join("Cargo.toml"))
        .unwrap_or_else(|error| panic!("读不了根 Cargo.toml：{error}"));
    let profile_lines = effective_profile_lines(&root_manifest_text, &profile_name, &|name| {
        std::env::var(name).ok()
    })
    .unwrap_or_else(|reason| panic!("profile 的生效值算不出：{reason}"));
    let toolchain_lines = toolchain_lines_of_the_build(&BuildToolchainInputs {
        rustc_version_text: &output_of(&environment_variable("RUSTC"), "-vV"),
        cargo_version_text: &output_of(&environment_variable("CARGO"), "-V"),
        target_triple: &environment_variable("TARGET"),
        encoded_rustflags: &std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
        profile_lines: &profile_lines,
    });
    let lock_text = std::fs::read_to_string(repository_root.join("Cargo.lock"))
        .unwrap_or_else(|error| panic!("读不了 Cargo.lock：{error}"));
    let locked_packages =
        parse_lock_file(&lock_text).unwrap_or_else(|error| panic!("Cargo.lock 读不了：{error}"));
    let inputs = InputsOfTheEntryGroupKeys {
        workspace: &workspace,
        features_enabled_by_the_build: &features_enabled_by_the_build,
        toolchain_lines: &toolchain_lines,
        locked_packages: &locked_packages,
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
                &toolchain_lines,
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
    for setting_key in PROFILE_SETTING_KEYS {
        println!(
            "cargo:rerun-if-env-changed={}",
            profile_environment_variable(&profile_name, setting_key)
        );
    }
}
