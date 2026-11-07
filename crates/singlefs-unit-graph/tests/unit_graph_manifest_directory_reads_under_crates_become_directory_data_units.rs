//! 目录数据单元（批次 `records/2026-10-26-checker欠账C896-C897-C914.md#5` 的 5g 规则 B）：按 `env!("CARGO_MANIFEST_DIR")` 拼出的是
//! `crates/` 之下（不是 `crates/` 本身）的一个目录时，整个目录（递归）记一个 `data-directory` 数据单元，不拒算；指纹是目录下每份文件
//! 按相对路径排好、逐份一行「相对路径 \t 内容的 SHA-256」、每行以换行收尾的整段 SHA-256。拼出的目录绑到 `let` 名字上时看名字的每一处
//! 用法（`.parent()` 这类往外走的照旧拒算）；私有的 `const 名字: &str = env!("CARGO_MANIFEST_DIR");` 按 `Path::new(名字)` 的用法折。
//! 仓根、`crates/` 之外的目录与 `crates/` 整个照旧拒算 `runtime-read-path-not-literal`。
//! 夹具：`fscan` 读自己的 `tables/` 与兄弟包 `fother` 的 `src/`，各种读法各是一个入口函数。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::output::closure_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::sha256::sha256_of;

const SCANNING_FILE: &str = "crates/fscan/src/lib.rs";

/// `fscan` 的库。行号进断言：改它要跟着改断言里的行号。
const SCANNING_LIBRARY: &str = r#"use std::path::Path;

const MANIFEST_DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");
pub const PUBLIC_MANIFEST_DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");

pub fn table_count() -> usize {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_dir(package_root.join("tables")).map(Iterator::count).unwrap_or(0)
}

pub fn sibling_source_count() -> usize {
    let manifest_directory = Path::new(MANIFEST_DIRECTORY);
    std::fs::read_dir(manifest_directory.join("../fother/src")).map(Iterator::count).unwrap_or(0)
}

pub fn public_constant_count() -> usize {
    std::fs::read_dir(Path::new(PUBLIC_MANIFEST_DIRECTORY).join("tables")).map(Iterator::count).unwrap_or(0)
}

pub fn repository_root_count() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).map(Iterator::count).unwrap_or(0)
}

pub fn crates_directory_count() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("..")).map(Iterator::count).unwrap_or(0)
}

pub fn litmus_count() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../litmus")).map(Iterator::count).unwrap_or(0)
}

pub fn repository_root_through_parent() -> usize {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    package_root.parent().map_or(0, |crates| crates.as_os_str().len())
}
"#;

const TABLE_FILE: &str = "crates/fscan/tables/one.txt";
const NESTED_TABLE_FILE: &str = "crates/fscan/tables/nested/two.txt";
const PACKAGE_NOTES_FILE: &str = "crates/fscan/notes.txt";
const SIBLING_SOURCE_FILE: &str = "crates/fother/src/lib.rs";

const TABLE_CONTENT: &str = "第一张表\n";
const NESTED_TABLE_CONTENT: &str = "嵌在子目录里的表\n";

fn scanning_tree(label: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            (
                "crates/fscan/Cargo.toml",
                "[package]\nname = \"fscan\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (SCANNING_FILE, SCANNING_LIBRARY),
            (TABLE_FILE, TABLE_CONTENT),
            (NESTED_TABLE_FILE, NESTED_TABLE_CONTENT),
            (PACKAGE_NOTES_FILE, "包里、tables 之外的一份说明\n"),
            (
                "crates/fother/Cargo.toml",
                "[package]\nname = \"fother\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (SIBLING_SOURCE_FILE, "pub fn sibling() -> u32 {\n    1\n}\n"),
            ("litmus/sample.litmus", "C sample\n"),
            ("Cargo.lock", "# 夹具不解析依赖\n"),
        ],
    )
}

const fn entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

const TABLE_ENTRY: EntrySpecification = entry("fscan::table_count");
const SIBLING_ENTRY: EntrySpecification = entry("fscan::sibling_source_count");

fn key_text(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

fn assert_refused_as_runtime_path(entry_path: &'static str, line: u32) {
    let tree = scanning_tree("directory-refused");
    let reason = key_text(&tree, &entry(entry_path)).expect_err("这种读法照旧拒算");
    assert!(
        reason.contains(&format!(
            "refused\truntime-read-path-not-literal\t{SCANNING_FILE}:{line}\t"
        )),
        "{entry_path} 拒算 runtime-read-path-not-literal、落在第 {line} 行：{reason}"
    );
}

/// 场景：`let package_root = Path::new(env!(…)); read_dir(package_root.join("tables"))`。预期：不拒算，闭包里有一行
/// `data-directory\tcrates/fscan/tables\t<读它的那一行>\t<摘要>`，摘要照规则逐字算（两份文件按相对路径排好、每行换行收尾）。
#[test]
fn a_directory_under_crates_handed_to_read_dir_becomes_a_directory_data_unit_instead_of_refused() {
    let tree = scanning_tree("directory-unit");
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(&workspace, &TABLE_ENTRY, &switches, HOST_TARGET_TRIPLE)
        .expect("入口找得到");
    let lines = closure_lines(&workspace, &outcome, None);
    assert!(
        outcome.refusals.is_empty(),
        "crates/ 之下的目录：不拒算：{lines:?}"
    );
    let listing = format!(
        "nested/two.txt\t{}\none.txt\t{}\n",
        sha256_of(NESTED_TABLE_CONTENT.as_bytes()).hexadecimal(),
        sha256_of(TABLE_CONTENT.as_bytes()).hexadecimal()
    );
    let expected = format!(
        "data-directory\tcrates/fscan/tables\t{SCANNING_FILE}:7-7\t{}",
        sha256_of(listing.as_bytes()).hexadecimal()
    );
    assert!(
        lines.contains(&expected),
        "目录数据单元一行，写法与摘要照规则：要 {expected}，实际 {lines:?}"
    );
}

/// 场景：改目录里子目录下的一份文件。预期：读这个目录的入口的键变。
#[test]
fn changing_a_file_nested_in_the_read_directory_changes_the_key() {
    let tree = scanning_tree("directory-nested-change");
    let before = key_text(&tree, &TABLE_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(NESTED_TABLE_FILE, "嵌在子目录里的表", "改过的表");
    let after = key_text(&tree, &TABLE_ENTRY).expect("改后算得出键");
    assert_ne!(before, after, "目录里（递归）一份文件改了，键要变");
}

/// 场景：改包里、读的目录之外的一份文件。预期：键不变。
#[test]
fn changing_a_package_file_outside_the_read_directory_keeps_the_key() {
    let tree = scanning_tree("directory-outside-change");
    let before = key_text(&tree, &TABLE_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(PACKAGE_NOTES_FILE, "一份说明", "一份改过的说明");
    let after = key_text(&tree, &TABLE_ENTRY).expect("改后算得出键");
    assert_eq!(before, after, "读的目录之外的文件改了，键不变");
}

/// 场景：私有常量 `MANIFEST_DIRECTORY = env!(…)`，`Path::new(常量)` 绑到名字上再 `.join("../fother/src")` 交给 `read_dir`。
/// 预期：不拒算；兄弟包源码目录里的文件改了，键变。
#[test]
fn a_private_manifest_directory_constant_wrapped_in_a_path_reads_the_sibling_directory_it_joins() {
    let tree = scanning_tree("directory-constant");
    let before =
        key_text(&tree, &SIBLING_ENTRY).expect("私有常量按 Path::new 的用法折得出，不拒算");
    tree.replace_exactly_once(SIBLING_SOURCE_FILE, "    1\n", "    2\n");
    let after = key_text(&tree, &SIBLING_ENTRY).expect("改后算得出键");
    assert_ne!(before, after, "兄弟包源码目录里的文件改了，键要变");
}

/// 场景：`pub const` 的清单目录常量（别的文件里的用法看不见）。预期：照旧拒算，落在常量那一行。
#[test]
fn a_public_manifest_directory_constant_is_still_refused() {
    assert_refused_as_runtime_path("fscan::public_constant_count", 4);
}

/// 场景：`.join("../..")` 拼出仓根。预期：照旧拒算。
#[test]
fn the_repository_root_is_still_refused() {
    assert_refused_as_runtime_path("fscan::repository_root_count", 21);
}

/// 场景：`.join("..")` 拼出 `crates/` 整个。预期：照旧拒算。
#[test]
fn the_whole_crates_directory_is_still_refused() {
    assert_refused_as_runtime_path("fscan::crates_directory_count", 25);
}

/// 场景：拼出仓根下 `crates/` 之外的 `litmus/`。预期：照旧拒算。
#[test]
fn a_directory_outside_crates_is_still_refused() {
    assert_refused_as_runtime_path("fscan::litmus_count", 29);
}

/// 场景：包目录绑到 `package_root`，之后 `package_root.parent()`（读整棵仓的写法）。预期：照旧拒算，落在 `env!` 那一行。
#[test]
fn a_bound_package_root_that_goes_to_its_parent_is_still_refused() {
    assert_refused_as_runtime_path("fscan::repository_root_through_parent", 33);
}
