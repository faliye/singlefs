//! WGSL 名字供给不收测试代码（批次 `records/2026-10-26-checker欠账C896-C897-C914.md#5` 的 5g 规则 C）：闭包读到的 `.wgsl` 里用到、
//! 闭包里又没有着落的外名，链接集里供它的单元并进闭包（`wgsl-name-supplier`）；带 `#[test]` 的函数与 `#[cfg(test)]` 模块里的单元
//! 不当供名字的单元收（入口本身在闭包里，照旧算着落）。真仓的形状：`crash_identity::tests::the_readback_placement_in_the_dispatch_changes_the_gpu_judge_key`
//! 因串里有 `VERDICT_WORDS` 被拉进别的入口的闭包。夹具：`fsupplied` 的内核用到 `LIMIT`，产品函数 `limit_definition` 与测试
//! `tests::kernel_mentions_the_limit` 的串里都有这个词。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::output::closure_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;

const SUPPLIED_FILE: &str = "crates/fsupplied/src/lib.rs";

const SUPPLIED_LIBRARY: &str = r#"pub const KERNEL_SOURCE: &str = include_str!("kernel.wgsl");

pub fn kernel_text() -> String {
    KERNEL_SOURCE.to_string()
}

pub fn limit_definition() -> &'static str {
    "const LIMIT: u32 = 4u;"
}

#[cfg(test)]
mod tests {
    #[test]
    fn kernel_mentions_the_limit() {
        assert!(super::kernel_text().contains("LIMIT"), "内核用到 LIMIT");
    }
}
"#;

const KERNEL: &str = "@group(0) @binding(0) var<storage, read_write> words: array<u32>;\n\
@compute @workgroup_size(64)\n\
fn main(@builtin(global_invocation_id) id: vec3<u32>) {\n\
    words[id.x] = LIMIT;\n\
}\n";

/// 产品入口：只读内核，`LIMIT` 的着落要靠闭包外的供名字单元。
const KERNEL_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsupplied::kernel_text",
    target: "lib",
    conditions: &["test"],
};

/// 测试函数自己当入口：它的串在闭包里，给 `LIMIT` 着落。
const TEST_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsupplied::tests::kernel_mentions_the_limit",
    target: "lib",
    conditions: &["test"],
};

fn supplied_tree(label: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            (
                "crates/fsupplied/Cargo.toml",
                "[package]\nname = \"fsupplied\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (SUPPLIED_FILE, SUPPLIED_LIBRARY),
            ("crates/fsupplied/src/kernel.wgsl", KERNEL),
            ("Cargo.lock", "# 夹具不解析依赖\n"),
        ],
    )
}

fn key_text(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

/// 场景：产品入口的内核缺 `LIMIT`，产品函数与测试函数的串里都有它。预期：并进来的是产品函数，测试函数不进闭包。
#[test]
fn a_test_function_naming_the_external_name_in_a_string_is_not_pulled_in_as_a_supplier() {
    let tree = supplied_tree("wgsl-supplier-not-test");
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(&workspace, &KERNEL_ENTRY, &switches, HOST_TARGET_TRIPLE)
        .expect("入口找得到");
    let lines = closure_lines(&workspace, &outcome, None);
    assert!(
        outcome.refusals.is_empty(),
        "LIMIT 有产品函数供着：{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("fn\tfsupplied::limit_definition\t")),
        "产品函数照旧当供名字的单元并进来：{lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("kernel_mentions_the_limit")),
        "测试函数不当供名字的单元收：{lines:?}"
    );
}

/// 场景：改那条测试函数的函数体。预期：产品入口的键不变（它不在闭包里）。
#[test]
fn changing_a_test_function_that_names_the_external_name_keeps_the_kernel_key() {
    let tree = supplied_tree("wgsl-supplier-test-change");
    let before = key_text(&tree, &KERNEL_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(SUPPLIED_FILE, "\"内核用到 LIMIT\"", "\"内核里有 LIMIT\"");
    let after = key_text(&tree, &KERNEL_ENTRY).expect("改后算得出键");
    assert_eq!(before, after, "闭包外的测试函数改了，产品入口的键不变");
}

/// 场景：改供名字的产品函数。预期：产品入口的键变（它照旧在闭包里）。
#[test]
fn changing_the_product_supplier_changes_the_kernel_key() {
    let tree = supplied_tree("wgsl-supplier-product-change");
    let before = key_text(&tree, &KERNEL_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(SUPPLIED_FILE, "= 4u;", "= 5u;");
    let after = key_text(&tree, &KERNEL_ENTRY).expect("改后算得出键");
    assert_ne!(before, after, "供名字的产品函数改了，键要变");
}

/// 场景：测试函数自己当入口。预期：它的串在闭包里给 `LIMIT` 着落，不拒算，也不再去并产品的供名字函数。
#[test]
fn the_entry_test_function_still_supplies_the_external_name_from_its_own_strings() {
    let tree = supplied_tree("wgsl-supplier-entry-itself");
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(&workspace, &TEST_ENTRY, &switches, HOST_TARGET_TRIPLE)
        .expect("入口找得到");
    let lines = closure_lines(&workspace, &outcome, None);
    assert!(outcome.refusals.is_empty(), "入口自己的串给着落：{lines:?}");
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("fn\tfsupplied::limit_definition\t")),
        "已有着落，不再并供名字的单元：{lines:?}"
    );
}
