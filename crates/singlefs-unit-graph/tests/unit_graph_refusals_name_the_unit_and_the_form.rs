//! 拒算（设计 3.4）：认不出的形态是显式结果——闭包里有它就不交键，拒算行点名文件、行与形态，不静默跳过。
//! 每条用例一个只有一个 crate 的小夹具，入口是 `refusing::entry_point`。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::rule_switches::RuleSwitches;

const PLAIN_MANIFEST: &str =
    "[package]\nname = \"refusing\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

const ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "refusing::entry_point",
    target: "lib",
    conditions: &[],
};

/// 用这几份文件（`src/lib.rs` 之外的另给）装一个 crate，交入口的键或拒算说明。
fn entry_verdict(
    manifest: &str,
    files: &[(&str, &str)],
    entry: &EntrySpecification,
) -> Result<String, String> {
    let mut tree_files = vec![("crates/refusing/Cargo.toml", manifest)];
    tree_files.extend(files.iter().copied());
    let tree = FixtureTree::materialize("refusal", &tree_files);
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

fn assert_refused_with(form: &str, files: &[(&str, &str)]) {
    let verdict = entry_verdict(PLAIN_MANIFEST, files, &ENTRY);
    let reason = verdict.expect_err("这个形态应当拒算");
    assert!(
        reason.contains(&format!("refused\t{form}\tcrates/refusing/")),
        "拒算行应当点名形态 {form} 与文件：{reason}"
    );
}

#[test]
fn an_include_str_whose_argument_is_not_a_literal_is_refused() {
    assert_refused_with(
        "include-argument-form",
        &[(
            "crates/refusing/src/lib.rs",
            "const TABLE_PATH: &str = \"table.txt\";\npub fn entry_point() -> usize {\n    include_str!(TABLE_PATH).len()\n}\n",
        )],
    );
}

#[test]
fn a_runtime_read_whose_path_is_joined_from_a_variable_is_refused() {
    assert_refused_with(
        "runtime-read-path-not-literal",
        &[(
            "crates/refusing/src/lib.rs",
            "pub fn entry_point(file_name: &str) -> String {\n    std::fs::read_to_string(std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(file_name)).unwrap_or_default()\n}\n",
        )],
    );
}

#[test]
fn a_runtime_read_with_a_literal_path_becomes_a_data_unit_instead_of_a_refusal() {
    let files = [
        (
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> String {\n    std::fs::read_to_string(std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"table.txt\")).unwrap_or_default()\n}\n",
        ),
        ("crates/refusing/table.txt", "原来的内容\n"),
    ];
    let before = entry_verdict(PLAIN_MANIFEST, &files, &ENTRY).expect("字面路径拼得出，不拒算");
    let changed_files = [files[0], ("crates/refusing/table.txt", "改过的内容\n")];
    let after =
        entry_verdict(PLAIN_MANIFEST, &changed_files, &ENTRY).expect("字面路径拼得出，不拒算");
    assert_ne!(before, after, "运行期读的那份文件改了，键要变");
}

#[test]
fn a_test_that_starts_its_own_executable_is_refused() {
    assert_refused_with(
        "current-exe",
        &[(
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> bool {\n    std::env::current_exe().is_ok()\n}\n",
        )],
    );
}

#[test]
fn an_identifier_pasting_macro_is_refused() {
    assert_refused_with(
        "identifier-pasting-macro",
        &[(
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> u32 {\n    paste! { [<value_ one>]() }\n}\n",
        )],
    );
}

#[test]
fn a_cfg_name_nobody_recognizes_is_refused_unless_the_command_line_gives_it() {
    let files = [(
        "crates/refusing/src/lib.rs",
        "#[cfg(verus_keep_ghost)]\npub fn ghost_helper() -> u32 {\n    1\n}\npub fn entry_point() -> u32 {\n    ghost_helper()\n}\n",
    )];
    let reason =
        entry_verdict(PLAIN_MANIFEST, &files, &ENTRY).expect_err("认不出的 cfg 名字应当拒算");
    assert!(
        reason.contains(
            "refused\tunrecognized-cfg-predicate\tcrates/refusing/src/lib.rs:1\tverus_keep_ghost"
        ),
        "{reason}"
    );
    let given = EntrySpecification {
        entry_path: "refusing::entry_point",
        target: "lib",
        conditions: &["verus_keep_ghost"],
    };
    assert!(
        entry_verdict(PLAIN_MANIFEST, &files, &given).is_ok(),
        "命令行给了 --cfg verus_keep_ghost，这个名字就认得"
    );
}

#[test]
fn a_call_into_an_extern_block_is_refused() {
    assert_refused_with(
        "extern-function-interface",
        &[(
            "crates/refusing/src/lib.rs",
            "extern \"C\" {\n    fn external_symbol() -> u32;\n}\npub fn entry_point() -> u32 {\n    unsafe { external_symbol() }\n}\n",
        )],
    );
}

#[test]
fn a_linked_file_with_unbalanced_brackets_is_refused() {
    assert_refused_with(
        "unbalanced-brackets",
        &[
            (
                "crates/refusing/src/lib.rs",
                "mod broken;\npub fn entry_point() -> u32 {\n    1\n}\n",
            ),
            (
                "crates/refusing/src/broken.rs",
                "pub fn half_written() {\n    (\n",
            ),
        ],
    );
}

#[test]
fn a_linked_file_with_an_unrecognized_item_start_is_refused() {
    assert_refused_with(
        "unrecognized-item-start",
        &[
            (
                "crates/refusing/src/lib.rs",
                "mod odd;\npub fn entry_point() -> u32 {\n    1\n}\n",
            ),
            ("crates/refusing/src/odd.rs", "what is this;\n"),
        ],
    );
}

#[test]
fn an_item_level_include_is_refused() {
    assert_refused_with(
        "item-level-include",
        &[(
            "crates/refusing/src/lib.rs",
            "include!(\"generated.rs\");\npub fn entry_point() -> u32 {\n    1\n}\n",
        )],
    );
}

#[test]
fn a_declared_module_whose_file_is_missing_is_refused() {
    assert_refused_with(
        "module-file-missing",
        &[(
            "crates/refusing/src/lib.rs",
            "mod absent;\npub fn entry_point() -> u32 {\n    1\n}\n",
        )],
    );
}

#[test]
fn an_included_file_that_does_not_exist_is_refused() {
    assert_refused_with(
        "data-file-unreadable",
        &[(
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> usize {\n    include_str!(\"absent.txt\").len()\n}\n",
        )],
    );
}

#[test]
fn a_procedure_macro_crate_in_the_link_is_refused() {
    let manifest = "[package]\nname = \"refusing\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nproc-macro = true\n";
    let verdict = entry_verdict(
        manifest,
        &[(
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> u32 {\n    1\n}\n",
        )],
        &ENTRY,
    );
    let reason = verdict.expect_err("proc-macro crate 应当拒算");
    assert!(
        reason.contains("refused\tproc-macro-crate\tcrates/refusing/src/lib.rs:1"),
        "{reason}"
    );
}

#[test]
fn a_manifest_directory_path_canonicalized_after_its_literal_joins_is_refused() {
    let files = [
        (
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> std::path::PathBuf {\n    std::path::PathBuf::from(env!(\"CARGO_MANIFEST_DIR\"))\n        .join(\"table.txt\")\n        .canonicalize()\n        .expect(\"在\")\n}\n",
        ),
        ("crates/refusing/table.txt", "内容\n"),
    ];
    let reason = entry_verdict(PLAIN_MANIFEST, &files, &ENTRY)
        .expect_err("字面 join 之后又接了方法：拼出的不是交给读文件的那条路径，拒算");
    assert!(
        reason.contains("refused\truntime-read-path-not-literal\tcrates/refusing/src/lib.rs:2"),
        "{reason}"
    );
}

#[test]
fn a_path_buf_from_the_manifest_directory_with_a_literal_join_becomes_a_data_unit() {
    let files = [
        (
            "crates/refusing/src/lib.rs",
            "pub fn entry_point() -> String {\n    std::fs::read_to_string(std::path::PathBuf::from(env!(\"CARGO_MANIFEST_DIR\")).join(\"table.txt\")).unwrap_or_default()\n}\n",
        ),
        ("crates/refusing/table.txt", "原来的内容\n"),
    ];
    let before = entry_verdict(PLAIN_MANIFEST, &files, &ENTRY).expect("字面路径拼得出，不拒算");
    let changed_files = [files[0], ("crates/refusing/table.txt", "改过的内容\n")];
    let after =
        entry_verdict(PLAIN_MANIFEST, &changed_files, &ENTRY).expect("字面路径拼得出，不拒算");
    assert_ne!(
        before, after,
        "PathBuf::from 与 Path::new 同一种认法：读的那份文件改了，键要变"
    );
}

/// `crates/singlefs-unit-graph/src/heavy_threads.rs` 读登记文件的那一形：`include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "../../<仓根下的文件>"))`
/// 照清单目录拼出包外、仓根之内的字面路径（`RelativeToManifest`），收成数据单元、不拒算；那份文件改了，键跟着变。
/// 判别力（`crates/mutations.tsv`）：`env!("CARGO_MANIFEST_DIR")` 起头的 `include_*!` 参数当成认不得的形态（拒算），这条红。
#[test]
fn a_manifest_relative_include_of_a_file_above_the_package_becomes_a_data_unit_instead_of_a_refusal(
) {
    let source = "pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(\n    env!(\"CARGO_MANIFEST_DIR\"),\n    \"/\",\n    \"../../registered-thread-default\"\n));\npub fn entry_point() -> usize {\n    REGISTRATION_FILE_TEXT.len()\n}\n";
    let files = [
        ("crates/refusing/src/lib.rs", source),
        ("registered-thread-default", "31  # 理由\n"),
    ];
    let before = entry_verdict(PLAIN_MANIFEST, &files, &ENTRY)
        .expect("清单目录起头的字面路径拼得出，不拒算");
    let changed_files = [files[0], ("registered-thread-default", "16  # 理由\n")];
    let after = entry_verdict(PLAIN_MANIFEST, &changed_files, &ENTRY)
        .expect("清单目录起头的字面路径拼得出，不拒算");
    assert_ne!(before, after, "包外那份登记文件改了，键要变");
}
