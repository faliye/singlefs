//! 编译时按判法单元的入口登记表（`src/judging_units_entries.rs`，经 `#[path]` 编同一份）逐组算判器键与组成清单，写进
//! `OUT_DIR/judging_units_keys.tsv`；库经 `include_str!(concat!(env!("OUT_DIR"), …))` 读回编进去的那一份，跑时不读盘上的树
//! （剪枝复查第二轮 design 路报告 3.8「build.rs 做什么」，4-2a）。
//!
//! 只做生成、不藏逻辑：切分、闭包、键、工具链行的拼法、表的写法、特性的取法都在 `singlefs-unit-graph` 里（`entry_groups`、
//! `build_toolchain`、`build_features`），命令行 `unit-graph key … --toolchain-of-the-build <profile> --lock-file Cargo.lock` 用的是同一份。
//! 这里只取 cargo 给的：本包开了哪些特性（cargo 编本构建脚本时带着本包这一次开了的特性的 cfg，[`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG`]；
//! 不读进程环境里的 `CARGO_FEATURE_*`，外面带进来的同名变量分不出，七·6 后续批代码轮第一轮判决第二节 T4）、依赖包在这一次编译里开了哪些特性（带 `links` 的依赖包构建脚本交出的
//! `DEP_<links>_ENABLED_FEATURES`：一起编 harness 档测试时它的 dev 依赖替 `singlefs-core` 并上的特性，本包的 `CARGO_FEATURE_*` 看不到）、
//! `$RUSTC -vV`、`$CARGO -V`、`TARGET`、`CARGO_ENCODED_RUSTFLAGS`、`PROFILE`（profile 的生效值另从工作区根 `Cargo.toml` 算，
//! `CARGO_PROFILE_<名>_<键>` 照盖），连同 `Cargo.lock`。

#[allow(
    dead_code,
    reason = "构建脚本只读登记表与拼组名；判器闭包不许够到的文件那两张表是库里的测试用的"
)]
#[path = "src/judging_units_entries.rs"]
mod judging_units_entries;

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;

use singlefs_unit_graph::build_features::{
    features_enabled_on_the_dependencies, features_of_the_crate_being_built,
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
use singlefs_unit_graph::{load_workspace, parse_lock_file, RuleSwitches, TargetSelector};

/// 写进 `OUT_DIR` 的那份表的文件名（库按同一个名字读回）。
const JUDGING_UNITS_KEYS_FILE_NAME: &str = "judging_units_keys.tsv";

/// 本包清单 `[features]` 里的每个特性（清单里的写法，按名字排好）与 cargo 这一次开没开它。与清单对不上（漏写、写错名字）的，
/// `features_of_the_crate_being_built` 交错误、构建脚本停下。
const FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG: [(&str, bool); 3] = [
    ("default", cfg!(feature = "default")),
    ("gpu", cfg!(feature = "gpu")),
    ("verdict-store", cfg!(feature = "verdict-store")),
];

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
    let features = features_of_the_crate_being_built(
        &workspace,
        own_crate,
        &FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG,
    )
    .unwrap_or_else(|error| panic!("本包这一次开了哪些特性取不了：{error}"));
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
    let requests = judging_units_entries::JUDGING_UNITS_ENTRY_GROUPS
        .iter()
        .map(|group| {
            (
                group.group_name.to_string(),
                group.target,
                group.entry_functions,
            )
        })
        .chain(
            judging_units_entries::OBSERVER_ENTRIES
                .iter()
                .map(|observer| {
                    (
                        judging_units_entries::observer_group_name(
                            observer.observer_code_file,
                            observer.observer_name,
                        ),
                        observer.target,
                        observer.entry_functions,
                    )
                }),
        );
    let keys: Vec<EntryGroupKey> = requests
        .map(|(group_name, target, entry_functions)| {
            let request = EntryGroupRequest {
                target: TargetSelector::parse(target).unwrap_or_else(|text| {
                    panic!("登记表里组 {group_name} 的目标写法认不得：{text}")
                }),
                entry_paths: entry_functions
                    .iter()
                    .map(|entry| (*entry).to_string())
                    .collect(),
                features: features.clone(),
                group_name,
            };
            entry_group_key_with_the_features_of_the_build(
                &workspace,
                &request,
                &features_enabled_by_the_build,
                &toolchain_lines,
                &locked_packages,
                &switches,
            )
            .unwrap_or_else(|error| {
                panic!("登记表里组 {} 的入口写错了：{error}", request.group_name)
            })
        })
        .collect();
    let output_path =
        PathBuf::from(environment_variable("OUT_DIR")).join(JUDGING_UNITS_KEYS_FILE_NAME);
    std::fs::write(
        &output_path,
        entry_group_table_text_with_the_features_of_the_build(
            &toolchain_lines,
            &features_enabled_by_the_build,
            &keys,
        ),
    )
    .unwrap_or_else(|error| panic!("写不了 {}：{error}", output_path.display()));
    let mut watched: BTreeSet<String> = [
        "Cargo.lock",
        "Cargo.toml",
        "crates/singlefs-checker-tier/build.rs",
        "crates/singlefs-checker-tier/src/judging_units_entries.rs",
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
    for key in &keys {
        watched.extend(key.watched_files.iter().cloned());
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
