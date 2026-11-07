//! Rust 闭包读到的 shell 脚本（`shell_sources`）：脚本在它自己那个 shell 里（传递地）`source` / `.` 进来的仓内文件，各收成一个数据单元，
//! 落在读这份脚本的那一处、字节进键；折不出的拒算。规则与 `scripts/gate/lib_function_units.py` 的 `shell_sourced_files`（那份文件头
//! 「Rust 闭包读到的 shell 脚本」）同一份，那边自证里的 `SHELL_DATA_FIXTURE` 几格（字面路径以外的三格）这里各有同一个结论的一条。
//!
//! 两份对拍：样本脚本放在 `tests/fixtures/shell_sources_shared_with_python/` 下（以 `.fixture` 收尾，免得仓里的门禁把它们当真脚本），
//! 两份实现各自收出的文件与拒算要与同一目录里的 `expected.tsv` 逐行相同。这一份由这里的
//! `the_files_and_refusals_from_the_shared_sample_scripts_match_the_table_shared_with_the_python_implementation` 核；python 那一份的核法：
//! 把样本照同样的规则写进一棵临时树，对表里每个样本脚本调 `shell_sourced_files(树根, 脚本)`，照表头的写法拼行、排好、与表逐行比
//! （命令随批次报告交主 agent 跑）。表是两边的共同期望：哪一边对不上，先看是哪一边的规则偏了，不许只改表迁就一边。
//!
//! 夹具：`fshell` 的每个函数照 `crates/singlefs-verification/src/gpu_card_configuration.rs` 起配置判法的写法，按包目录拼出一份样本脚本的路径。

mod common_fixture_workspace;

use std::path::{Path, PathBuf};

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::output::closure_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::sha256::sha256_of;
use singlefs_unit_graph::shell_sources::{shell_sourced_files, ShellSourcedFilesCache};
use singlefs_unit_graph::EntryError;

const SHELL_DATA_FILE: &str = "crates/fshell/src/lib.rs";

/// `fshell` 的库。读脚本的那一行（`env!` 所在的行）进断言：改它要跟着改 `*_READ_LINE`。
const SHELL_DATA_LIBRARY: &str = r#"pub fn runs_the_literal_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/literal.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_script_sourcing_from_its_own_directory() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/directory-of-the-script.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_dotted_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dotted.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_unfolded_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/unfolded.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_plain_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/plain.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
"#;

const LITERAL_SCRIPT_READ_LINE: u32 = 2;
const DIRECTORY_SCRIPT_READ_LINE: u32 = 6;
const DOTTED_SCRIPT_READ_LINE: u32 = 10;

const LITERAL_ENTRY: EntrySpecification = library_entry("fshell::runs_the_literal_script");
const DIRECTORY_ENTRY: EntrySpecification =
    library_entry("fshell::runs_the_script_sourcing_from_its_own_directory");
const DOTTED_ENTRY: EntrySpecification = library_entry("fshell::runs_the_dotted_script");
const UNFOLDED_ENTRY: EntrySpecification = library_entry("fshell::runs_the_unfolded_script");
const PLAIN_ENTRY: EntrySpecification = library_entry("fshell::runs_the_plain_script");

const fn library_entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

/// 两份对拍的样本目录（按包目录拼的字面路径：这个目录整个进这几条用例的键）。
fn shared_sample_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shell_sources_shared_with_python")
}

/// 样本目录下每份以 `.fixture` 收尾的文件 →（去掉 `.fixture` 的相对路径，内容），按相对路径排好。
fn shared_sample_files() -> Vec<(String, String)> {
    let root = shared_sample_directory();
    let mut found = Vec::new();
    let mut directories_to_visit = vec![root.clone()];
    while let Some(directory) = directories_to_visit.pop() {
        let entries = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("读样本目录 {}：{error}", directory.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|error| panic!("读样本目录 {}：{error}", directory.display()))
                .path();
            if path.is_dir() {
                directories_to_visit.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(&root)
                .expect("走的是样本目录底下")
                .to_string_lossy()
                .into_owned();
            if let Some(materialized) = relative.strip_suffix(".fixture") {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("读样本 {}：{error}", path.display()));
                found.push((materialized.to_string(), content));
            }
        }
    }
    found.sort();
    assert!(
        found.len() >= 20,
        "样本目录里要读到全部样本（二十多份），实际 {}：{found:?}",
        found.len()
    );
    found
}

fn shared_sample_content(materialized: &str) -> String {
    shared_sample_files()
        .into_iter()
        .find(|(relative, _content)| relative == materialized)
        .unwrap_or_else(|| panic!("样本目录里有 {materialized}.fixture"))
        .1
}

/// 共用样本连同 `fshell` 包写进一棵临时树。
fn shell_data_tree(label: &str) -> FixtureTree {
    let samples = shared_sample_files();
    let mut files: Vec<(&str, &str)> = vec![
        (
            "crates/fshell/Cargo.toml",
            "[package]\nname = \"fshell\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        (SHELL_DATA_FILE, SHELL_DATA_LIBRARY),
        ("Cargo.lock", "# 夹具不解析依赖\n"),
    ];
    files.extend(
        samples
            .iter()
            .map(|(relative, content)| (relative.as_str(), content.as_str())),
    );
    FixtureTree::materialize(label, &files)
}

fn key_text(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

fn closure_text_lines(tree: &FixtureTree, entry: &EntrySpecification) -> Vec<String> {
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(&workspace, entry, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("{} 的闭包收得起来：{error}", entry.entry_path));
    closure_lines(&workspace, &outcome, None)
}

/// 一个经 source 收进来的文件在闭包里的那一行：种类 `data`，限定名是被 source 的文件，位置是读脚本的那一处，指纹是它的字节 SHA-256。
fn sourced_data_line(sourced_file: &str, read_line: u32) -> String {
    format!(
        "data\t{sourced_file}\t{SHELL_DATA_FILE}:{read_line}-{read_line}\t{}",
        sha256_of(shared_sample_content(sourced_file).as_bytes()).hexadecimal()
    )
}

/// 场景：脚本里 `source scripts/lib/plain-library.sh`（相对路径，按根找到）。预期：不拒算，闭包里有一行被 source 的那份文件的数据单元，
/// 落在读脚本的那一行、指纹是它的字节。
#[test]
fn a_literal_source_path_brings_the_sourced_file_into_the_closure_as_a_data_unit_at_the_site_reading_the_script(
) {
    let tree = shell_data_tree("shell-literal");
    let lines = closure_text_lines(&tree, &LITERAL_ENTRY);
    let expected = sourced_data_line("scripts/lib/plain-library.sh", LITERAL_SCRIPT_READ_LINE);
    assert!(
        lines.contains(&expected),
        "字面路径 source 的文件收成数据单元：要 {expected}，实际 {lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line.starts_with("refused\t")),
        "字面路径折得出、文件在：不拒算：{lines:?}"
    );
}

/// 场景：`source "$(dirname "${BASH_SOURCE[0]}")/lib/first.sh"`，改 first.sh 的一行。预期：改动前后都算得出键，键变。
#[test]
fn changing_a_file_sourced_through_the_directory_of_bash_source_changes_the_key() {
    let tree = shell_data_tree("shell-directory-of-the-script");
    let before = key_text(&tree, &DIRECTORY_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(
        "scripts/lib/first.sh",
        "  echo first\n",
        "  echo first, changed\n",
    );
    let after = key_text(&tree, &DIRECTORY_ENTRY).expect("改后算得出键");
    assert_ne!(
        before, after,
        "经 dirname 写法 source 进来的文件改了，键要变"
    );
}

/// 场景：脚本 source 的 first.sh 再经 `$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)` 赋的变量 source second.sh；改 second.sh。
/// 预期：闭包里两层的文件都在，改里层 second.sh 键变。
#[test]
fn a_library_sourced_by_a_sourced_library_is_followed_layer_by_layer_and_changing_it_changes_the_key(
) {
    let tree = shell_data_tree("shell-nested");
    let lines = closure_text_lines(&tree, &DIRECTORY_ENTRY);
    for sourced_file in ["scripts/lib/first.sh", "scripts/lib/second.sh"] {
        let expected = sourced_data_line(sourced_file, DIRECTORY_SCRIPT_READ_LINE);
        assert!(
            lines.contains(&expected),
            "两层都收：要 {expected}，实际 {lines:?}"
        );
    }
    let before = key_text(&tree, &DIRECTORY_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(
        "scripts/lib/second.sh",
        "  echo second\n",
        "  echo second, changed\n",
    );
    let after = key_text(&tree, &DIRECTORY_ENTRY).expect("改后算得出键");
    assert_ne!(before, after, "里层 second.sh 改了，键要变");
}

/// 场景：`. "$SCRIPT_DIRECTORY/lib/dot-only.sh"`（变量由 `$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)` 赋）。预期：`.` 照 source 收。
#[test]
fn a_dot_source_through_a_variable_assigned_from_cd_and_pwd_is_followed() {
    let tree = shell_data_tree("shell-dotted");
    let lines = closure_text_lines(&tree, &DOTTED_ENTRY);
    let expected = sourced_data_line("scripts/lib/dot-only.sh", DOTTED_SCRIPT_READ_LINE);
    assert!(
        lines.contains(&expected),
        "`.` 引进来的文件收成数据单元：要 {expected}，实际 {lines:?}"
    );
}

/// 场景：`library="$1"; source "$library"`。预期：拒算 `shell-path-unfolded`，落在样本脚本 source 的那一行。
#[test]
fn a_source_path_that_does_not_fold_refuses_the_key_as_shell_path_unfolded() {
    let tree = shell_data_tree("shell-unfolded");
    let reason = key_text(&tree, &UNFOLDED_ENTRY).expect_err("source 的路径折不出要拒算");
    assert!(
        reason.contains("refused\tshell-path-unfolded\tscripts/unfolded.sh:2\tsource \"$library\""),
        "拒算行点名样本脚本第 2 行：{reason}"
    );
}

/// 场景：改 first.sh（plain.sh 不 source 它）。预期：读 plain.sh 的入口改动前后都算得出键，键不变。
#[test]
fn changing_a_sourced_file_keeps_the_key_of_an_entry_reading_a_script_that_does_not_source_it() {
    let tree = shell_data_tree("shell-plain");
    let before = key_text(&tree, &PLAIN_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(
        "scripts/lib/first.sh",
        "  echo first\n",
        "  echo first, changed\n",
    );
    let after = key_text(&tree, &PLAIN_ENTRY).expect("改后算得出键");
    assert_eq!(
        before, after,
        "plain.sh 不 source first.sh，改 first.sh 不动它的键"
    );
}

/// 场景：共用样本写进临时树，对 `expected.tsv` 里每个样本脚本调 `shell_sourced_files`，照表的写法拼行。预期：与表逐行相同
/// （字面路径、dirname 写法、`cd … && pwd`、`.`、两层、函数里的 local 与 `:-` 缺省、被 source 的文件里赋的变量、写出来的脚本、
/// `$目录/<字面尾巴>`、`%/*` 与 `git rev-parse --git-common-dir` 两处赋值、`$CLAUDE_PROJECT_DIR`、注释与 `bash -c` 与 here-doc 正文不收、
/// 折不出、指到不在的文件、落在根外、bash 规范化不出来、下标带空格与展开的数组下标赋值后面同一行的赋值照认）。
#[test]
fn the_files_and_refusals_from_the_shared_sample_scripts_match_the_table_shared_with_the_python_implementation(
) {
    let tree = FixtureTree::materialize(
        "shell-shared-samples",
        &shared_sample_files()
            .iter()
            .map(|(relative, content)| (relative.as_str(), content.as_str()))
            .collect::<Vec<_>>(),
    );
    let table = std::fs::read_to_string(shared_sample_directory().join("expected.tsv"))
        .expect("样本目录里有 expected.tsv");
    let mut expected: Vec<String> = table
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect();
    expected.sort();
    let mut scripts: Vec<String> = expected
        .iter()
        .map(|line| {
            line.split('\t')
                .next()
                .expect("每行有样本脚本那一段")
                .to_string()
        })
        .collect();
    scripts.dedup();
    assert!(
        scripts.len() >= 15,
        "表里要有全部样本脚本（十五个以上），实际 {}：{scripts:?}",
        scripts.len()
    );
    let mut actual = Vec::new();
    for script in &scripts {
        let collected = shell_sourced_files(tree.root(), script)
            .unwrap_or_else(|failure| panic!("这台机器上 bash 规范化跑得起来：{failure}"));
        actual.extend(
            collected
                .sourced_files
                .iter()
                .map(|file| format!("{script}\tsourced\t{file}")),
        );
        actual.extend(collected.refusals.iter().map(|refusal| {
            format!(
                "{script}\trefused\t{}\t{}:{}\t{}",
                refusal.form.output_name(),
                refusal.file,
                refusal.line,
                refusal.snippet
            )
        }));
        if collected.sourced_files.is_empty() && collected.refusals.is_empty() {
            actual.push(format!("{script}\tnothing"));
        }
    }
    actual.sort();
    let missing: Vec<&String> = expected
        .iter()
        .filter(|line| !actual.contains(line))
        .collect();
    let extra: Vec<&String> = actual
        .iter()
        .filter(|line| !expected.contains(line))
        .collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "与两份共用的期望表逐行相同：表里有而收出来没有的 {missing:?}；收出来有而表里没有的 {extra:?}"
    );
    assert_eq!(actual, expected, "行数与次序也相同");
}

/// 场景：工作区的 shell 脚本表换成起不来的 bash（临时树里不在的一个路径），收读 `scripts/literal.sh` 的入口的闭包。
/// 预期：整趟报错 `EntryError::ShellNormalizationFailed`，点名 literal.sh、原因是 bash 起不来；不交闭包、不记成拒算
/// （python 那一份在这里抛 `FunctionUnitsError`）。
#[test]
fn bash_that_cannot_start_fails_the_whole_closure_instead_of_refusing() {
    let tree = shell_data_tree("shell-bash-cannot-start");
    let switches = RuleSwitches::every_rule_intact();
    let mut workspace = workspace_of(tree.root(), &switches);
    workspace.shell_sourced_files = ShellSourcedFilesCache::with_the_bash_program(
        tree.root()
            .join("no-bash-in-this-tree/bash")
            .into_os_string(),
    );
    match closure_with_triple(&workspace, &LITERAL_ENTRY, &switches, HOST_TARGET_TRIPLE) {
        Err(EntryError::ShellNormalizationFailed(failure)) => {
            assert_eq!(
                failure.script, "scripts/literal.sh",
                "报错点名折不下去的那份脚本：{failure}"
            );
            assert!(
                failure.reason.contains("起不来"),
                "原因是 bash 起不来：{failure}"
            );
        }
        Err(other) => panic!("bash 起不来要交 ShellNormalizationFailed，实际交了别的错：{other}"),
        Ok(outcome) => panic!(
            "bash 起不来要整趟报错，实际交了闭包（{} 条拒算）：{:?}",
            outcome.refusals.len(),
            closure_lines(&workspace, &outcome, None)
        ),
    }
}
