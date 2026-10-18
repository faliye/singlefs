//! cfg 集取法：目标平台的 cfg（`target_os`、`target_pointer_width`、`unix` 这类）从工具链的目标三元组查表，不写死本机平台；
//! 同一份源码换三元组，`#[cfg(target_os = …)]` 两支进闭包的不同；表里查不出的三元组拒算。

mod common_fixture_workspace;

use std::collections::BTreeSet;

use common_fixture_workspace::{
    closure_with_triple, key_of_outcome, workspace_of, EntrySpecification, FixtureTree,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::{ClosureOutcome, ExternalKeyInputs, Workspace};

/// 一个入口调 `platform_name`，它按 `target_os` 分两支；`unix` 下另有一支 `separator`。
const PLATFORM_BRANCH_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fplatform/Cargo.toml",
        "[package]\nname = \"fplatform\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fplatform/src/lib.rs",
        "#[cfg(target_os = \"linux\")]\n\
         pub fn platform_name() -> &'static str {\n\
         \x20   \"linux\"\n\
         }\n\
         #[cfg(target_os = \"macos\")]\n\
         pub fn platform_name() -> &'static str {\n\
         \x20   \"macos\"\n\
         }\n\
         #[cfg(unix)]\n\
         pub fn separator() -> char {\n\
         \x20   '/'\n\
         }\n\
         #[cfg(windows)]\n\
         pub fn separator() -> char {\n\
         \x20   '\\\\'\n\
         }\n\
         pub fn describe() -> String {\n\
         \x20   format!(\"{}{}\", platform_name(), separator())\n\
         }\n",
    ),
];

const DESCRIBE_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fplatform::describe",
    target: "lib",
    conditions: &[],
};

fn closure_under(workspace: &Workspace, target_triple: &str) -> ClosureOutcome {
    closure_with_triple(
        workspace,
        &DESCRIBE_ENTRY,
        &RuleSwitches::every_rule_intact(),
        target_triple,
    )
    .expect("入口找得到")
}

/// 闭包里函数单元的起行（两支同名，按起行分得开）。
fn function_start_lines(workspace: &Workspace, outcome: &ClosureOutcome) -> BTreeSet<u32> {
    outcome
        .units
        .iter()
        .map(|unit_index| &workspace.units[unit_index.0])
        .filter(|unit| unit.kind == singlefs_unit_graph::units::UnitKind::Function)
        .map(|unit| unit.start_line)
        .collect()
}

#[test]
fn the_same_source_under_two_triples_closes_over_the_two_target_os_branches() {
    let tree = FixtureTree::materialize("platform-branch", &PLATFORM_BRANCH_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let linux = closure_under(&workspace, "x86_64-unknown-linux-gnu");
    let macos = closure_under(&workspace, "aarch64-apple-darwin");
    assert!(linux.refusals.is_empty(), "{:?}", linux.refusals);
    assert!(macos.refusals.is_empty(), "{:?}", macos.refusals);
    assert_eq!(
        function_start_lines(&workspace, &linux),
        BTreeSet::from([1, 9, 17]),
        "Linux 下收 target_os = linux 那一支（第 1 行）与 unix 那一支（第 9 行）"
    );
    assert_eq!(
        function_start_lines(&workspace, &macos),
        BTreeSet::from([5, 9, 17]),
        "macOS 下收 target_os = macos 那一支（第 5 行）与 unix 那一支"
    );
    let windows = closure_under(&workspace, "x86_64-pc-windows-msvc");
    assert_eq!(
        function_start_lines(&workspace, &windows),
        BTreeSet::from([13, 17]),
        "Windows 下两支 target_os 都不收，收 windows 那一支（第 13 行）"
    );
    assert_ne!(
        key_of_outcome(&workspace, &linux, &ExternalKeyInputs::default()),
        key_of_outcome(&workspace, &macos, &ExternalKeyInputs::default()),
        "两个三元组的键不同"
    );
}

#[test]
fn a_triple_missing_from_the_table_is_refused() {
    let tree = FixtureTree::materialize("platform-unknown", &PLATFORM_BRANCH_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_under(&workspace, "sparc64-sun-solaris");
    let reason = key_of_outcome(&workspace, &outcome, &ExternalKeyInputs::default())
        .expect_err("表里没有的三元组判不了平台谓词，拒算");
    assert!(
        reason.contains(
            "refused\ttarget-triple-unrecognized\tcrates/fplatform/src/lib.rs:17\tsparc64-sun-solaris"
        ),
        "拒算行落在入口那一行、片段是三元组：{reason}"
    );
}
