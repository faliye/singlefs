//! 子进程测试边（`current-exe-child`，批次 `records/2026-10-26-checker欠账C896-C897-C914.md#5` 的 5g 规则 A）：单元把 `current_exe()` 交给
//! `Command::new` 起本测试二进制，`--exact` 后面跟着折得出的测试名（字面串、同一 crate 的 `const …: &str`、全由它们组成的 `format!`；
//! 或起子进程的函数自己的一个参数，取闭包里每一处调用交的实参），就连到入口目标里那个 `#[test] fn`、不拒算；折不出、解不到恰好一个的
//! 照旧拒算 `current-exe`。夹具是一个只有库的 crate `fchild`，入口都在它的 `#[cfg(test)] mod tests` 里（带 `--cfg test` 收）。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::output::closure_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;

const CHILD_MANIFEST: &str =
    "[package]\nname = \"fchild\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

const LIBRARY_FILE: &str = "crates/fchild/src/lib.rs";

/// `fchild` 的库：产品函数一个，测试模块里起子进程的几种写法与被起的那条测试。
const CHILD_LIBRARY: &str = r#"pub fn product_value() -> u32 {
    7
}

#[cfg(test)]
mod tests {
    const CHILD_TEST: &str = "tests::child_measures_alone";
    const CHILD_FUNCTION: &str = "child_measures_alone";

    fn spawn_alone(test_name: &str) {
        let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .args(["--exact", test_name, "--test-threads=1"])
            .output()
            .expect("子进程起得来");
        assert!(output.status.success());
    }

    #[test]
    fn parent_spawns_the_child_named_by_a_constant() {
        let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .args(["--exact", CHILD_TEST, "--nocapture"])
            .output()
            .expect("子进程起得来");
        assert!(output.status.success());
    }

    #[test]
    fn parent_spawns_the_child_named_by_a_format() {
        let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .arg("--exact")
            .arg(format!("{}::{CHILD_FUNCTION}", "tests"))
            .output()
            .expect("子进程起得来");
        assert!(output.status.success());
    }

    #[test]
    fn parent_spawns_the_child_through_a_helper() {
        spawn_alone(CHILD_TEST);
    }

    #[test]
    fn parent_names_a_test_that_does_not_exist() {
        let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .args(["--exact", "tests::no_test_has_this_name"])
            .output()
            .expect("子进程起得来");
        assert!(output.status.success());
    }

    #[test]
    fn child_measures_alone() {
        assert_eq!(super::product_value(), 7);
    }

    #[test]
    fn unrelated_test_outside_every_closure() {
        assert_eq!(1 + 1, 2);
    }
}
"#;

const BY_A_CONSTANT_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fchild::tests::parent_spawns_the_child_named_by_a_constant",
    target: "lib",
    conditions: &["test"],
};

const BY_A_FORMAT_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fchild::tests::parent_spawns_the_child_named_by_a_format",
    target: "lib",
    conditions: &["test"],
};

const THROUGH_A_HELPER_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fchild::tests::parent_spawns_the_child_through_a_helper",
    target: "lib",
    conditions: &["test"],
};

const NONEXISTENT_TEST_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fchild::tests::parent_names_a_test_that_does_not_exist",
    target: "lib",
    conditions: &["test"],
};

/// 被起的那条测试的函数体（改它，起它的入口的键要变）。
const CHILD_BODY: &str = "assert_eq!(super::product_value(), 7);";
const CHILD_BODY_CHANGED: &str = "assert_eq!(super::product_value(), 8 - 1);";

/// 哪个闭包都不收的那条测试的函数体。
const UNRELATED_BODY: &str = "assert_eq!(1 + 1, 2);";
const UNRELATED_BODY_CHANGED: &str = "assert_eq!(2 + 2, 4);";

fn child_tree(label: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            ("crates/fchild/Cargo.toml", CHILD_MANIFEST),
            (LIBRARY_FILE, CHILD_LIBRARY),
            ("Cargo.lock", "# 夹具不解析依赖\n"),
        ],
    )
}

fn key_text(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

/// 场景：入口用 `--exact 常量` 起子进程。预期：不拒算，闭包里有被点名的那条测试与它调的产品函数，没有别的测试。
#[test]
fn a_child_named_by_a_constant_after_exact_is_linked_into_the_closure_instead_of_refused() {
    let tree = child_tree("current-exe-constant");
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(
        &workspace,
        &BY_A_CONSTANT_ENTRY,
        &switches,
        HOST_TARGET_TRIPLE,
    )
    .expect("入口找得到");
    let lines = closure_lines(&workspace, &outcome, None);
    assert!(
        outcome.refusals.is_empty(),
        "测试名折得出、解得到恰好一条 #[test] fn：不拒算 current-exe：{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("fn\tfchild::tests::child_measures_alone\t")),
        "子进程跑的那条测试连进闭包：{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("fn\tfchild::product_value\t")),
        "子进程测试调的产品函数随它进闭包：{lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("unrelated_test_outside_every_closure")),
        "没被点名的测试不进闭包：{lines:?}"
    );
}

/// 场景：改被起的那条测试的函数体。预期：起它的入口的键变。
#[test]
fn changing_the_body_of_the_child_test_changes_the_key_of_the_test_that_spawns_it() {
    let tree = child_tree("current-exe-child-body");
    let before = key_text(&tree, &BY_A_CONSTANT_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(LIBRARY_FILE, CHILD_BODY, CHILD_BODY_CHANGED);
    let after = key_text(&tree, &BY_A_CONSTANT_ENTRY).expect("改后算得出键");
    assert_ne!(before, after, "子进程跑的测试改了，起它的入口的键要变");
}

/// 场景：改闭包外的一条测试的函数体。预期：键不变。
#[test]
fn changing_a_test_outside_the_closure_keeps_the_key_of_the_test_that_spawns_the_child() {
    let tree = child_tree("current-exe-outside");
    let before = key_text(&tree, &BY_A_CONSTANT_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(LIBRARY_FILE, UNRELATED_BODY, UNRELATED_BODY_CHANGED);
    let after = key_text(&tree, &BY_A_CONSTANT_ENTRY).expect("改后算得出键");
    assert_eq!(before, after, "闭包外的测试改了，键不变");
}

/// 场景：测试名写成 `format!("{}::{CHILD_FUNCTION}", "tests")`（字面串与常量拼的），经 `.arg` 一个一个交。
/// 预期：不拒算；改被起的测试，键变。
#[test]
fn a_child_named_by_a_format_of_literals_and_constants_is_linked_and_keys_follow_its_body() {
    let tree = child_tree("current-exe-format");
    let before = key_text(&tree, &BY_A_FORMAT_ENTRY).expect("format! 拼的测试名折得出，不拒算");
    tree.replace_exactly_once(LIBRARY_FILE, CHILD_BODY, CHILD_BODY_CHANGED);
    let after = key_text(&tree, &BY_A_FORMAT_ENTRY).expect("改后算得出键");
    assert_ne!(before, after, "子进程跑的测试改了，键要变");
}

/// 场景：起子进程的是一个辅助函数，测试名是它的参数，入口交一个常量。预期：不拒算；改被起的测试，键变。
#[test]
fn a_child_named_by_the_argument_a_caller_passes_to_the_spawning_helper_is_linked() {
    let tree = child_tree("current-exe-helper");
    let before = key_text(&tree, &THROUGH_A_HELPER_ENTRY).expect("调用处交的实参折得出，不拒算");
    tree.replace_exactly_once(LIBRARY_FILE, CHILD_BODY, CHILD_BODY_CHANGED);
    let after = key_text(&tree, &THROUGH_A_HELPER_ENTRY).expect("改后算得出键");
    assert_ne!(
        before, after,
        "子进程跑的测试改了，经辅助函数起它的入口的键要变"
    );
}

/// 场景：`--exact` 后面的名字在这个目标里没有对得上的 `#[test] fn`。预期：照旧拒算 `current-exe`，落在 `current_exe` 那一行。
#[test]
fn a_child_name_that_matches_no_test_function_is_still_refused_as_current_exe() {
    let tree = child_tree("current-exe-unmatched");
    let reason = key_text(&tree, &NONEXISTENT_TEST_ENTRY).expect_err("解不到 #[test] fn：照旧拒算");
    assert!(
        reason.contains("refused\tcurrent-exe\tcrates/fchild/src/lib.rs:44\t"),
        "拒算行点名 current-exe 与 current_exe 那一行：{reason}"
    );
}
