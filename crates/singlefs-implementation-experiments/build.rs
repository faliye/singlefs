//! 编译时按装置自己的判法单元入口登记表（`src/judging_units_entries.rs`，经 `#[path]` 编同一份）逐组算键与组成清单，写进
//! `OUT_DIR/judging_units_keys.tsv`；库经 `include_str!(concat!(env!("OUT_DIR"), …))` 读回编进去的那一份（`JUDGING_UNITS_KEYS_TABLE_TEXT`），
//! 跑时不读盘上的树。做法照验证包的构建脚本（`crates/singlefs-verification/build.rs`）：切分、闭包、键、表的写法、特性的取法
//! 都在 `singlefs-unit-graph` 里，这里只取 cargo 给的（本包开了哪些特性、依赖包在这一次编译里开了哪些特性、`TARGET`）；工具链与编译档不读、
//! 不进键，`Cargo.lock` 不进键（照旧盯着；D13（验证路线） 已定项 17），表里 `toolchain` 那一行只记收闭包时照的目标。参照那几组与观察者那几组写进同一张表，组名照登记表拼。

#[path = "src/judging_units_entries.rs"]
mod judging_units_entries;

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use singlefs_unit_graph::build_features::{
    features_enabled_on_the_dependencies, features_of_the_crate_being_built,
};
use singlefs_unit_graph::entry_groups::{
    entry_group_key_with_the_features_of_the_build,
    entry_group_table_text_with_the_features_of_the_build, target_line_of_the_table, EntryGroupKey,
    EntryGroupRequest,
};
use singlefs_unit_graph::{load_workspace, RuleSwitches, TargetSelector};

/// 写进 `OUT_DIR` 的那份表的文件名（库按同一个名字读回）。
const JUDGING_UNITS_KEYS_FILE_NAME: &str = "judging_units_keys.tsv";

/// 本包清单 `[features]` 里的每个特性（清单里的写法，按名字排好）与 cargo 这一次开没开它。与清单对不上（漏写、写错名字）的，
/// `features_of_the_crate_being_built` 交错误、构建脚本停下。
const FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS: [(&str, bool); 1] =
    [("gpu", cfg!(feature = "gpu"))];

fn environment_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("cargo 起构建脚本时设 {name}"))
}

/// 登记表里的每一组（参照那几组在前、观察者那几组在后，按登记的次序）：组名、目标、入口 fn。
fn requests_of_the_entries_table() -> Vec<(String, &'static str, &'static [&'static str])> {
    judging_units_entries::JUDGING_UNITS_ENTRY_GROUPS
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
        )
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
    let features = features_of_the_crate_being_built(
        &workspace,
        own_crate,
        &FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS,
    )
    .unwrap_or_else(|error| panic!("本包这一次开了哪些特性取不了：{error}"));
    let features_enabled_by_the_build =
        features_enabled_on_the_dependencies(&workspace, own_crate, &read_environment)
            .unwrap_or_else(|error| panic!("依赖包在这一次编译里开了哪些特性取不全：{error}"));
    let target_triple = environment_variable("TARGET");
    // 组数是登记表的行数（几组），一条线程逐组算，键表的次序就是登记的次序
    let keys: Vec<EntryGroupKey> = requests_of_the_entries_table()
        .into_iter()
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
                &target_triple,
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
            &[target_line_of_the_table(&target_triple)],
            &features_enabled_by_the_build,
            &keys,
        ),
    )
    .unwrap_or_else(|error| panic!("写不了 {}：{error}", output_path.display()));
    let mut watched: BTreeSet<String> = [
        "Cargo.lock",
        "Cargo.toml",
        "crates/singlefs-implementation-experiments/build.rs",
        "crates/singlefs-implementation-experiments/src/judging_units_entries.rs",
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
}
