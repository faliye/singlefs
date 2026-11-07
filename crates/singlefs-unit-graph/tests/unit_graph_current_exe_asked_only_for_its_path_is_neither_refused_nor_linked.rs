//! `current_exe()` 拒不拒算看同一个单元里起没起子进程（`scripts/gate/lib_function_units.py` 文件头「测试二进制起自己」同一判法）：
//! 单元里没有 `Command`、`spawn` 这类词、只问路径的，不拒算也不连边；起了子进程的照旧按起自己判——折得出 `--exact` 后面点名的测试就连边，
//! 折不出（没有 `--exact`、路径先存进变量再交 `Command::new`、点名的测试不在）拒算 `current-exe`。
//! 夹具与那一份的 `CURRENT_EXE_PATH_ONLY_FIXTURE`、`CURRENT_EXE_SPAWN_FIXTURE` 逐字同形，每条断言与那边同名的格同一个结论；
//! `use std::{env::current_exe, process::Command};` 那一条是两边共有的「后面不跟括号的 `current_exe` 不算」，夹具只在这一份。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::output::closure_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;

const PATH_ONLY_MANIFEST: &str = "[package]\nname = \"fexp\"\n";
const PATH_ONLY_FILE: &str = "crates/fexp/src/lib.rs";

/// 只问路径的两种写法（`current_exe().is_ok()`、取路径读程序文件的大小）与一条无关的测试。
const PATH_ONLY_LIBRARY: &str = r#"#[cfg(test)]
mod tests {
    #[test]
    fn it_only_asks_whether_it_has_a_path() {
        assert!(std::env::current_exe().is_ok());
    }
    #[test]
    fn it_reads_the_size_of_the_file_it_was_started_from() {
        let started_from = std::env::current_exe().expect("路径");
        let size = std::fs::metadata(&started_from).expect("读得到").len();
        assert!(size > 0);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
}
"#;

const UNRELATED_BODY: &str = "        let note = 5;\n        assert_eq!(note, 5);";
const UNRELATED_BODY_CHANGED: &str = "        let note = 6;\n        assert_eq!(note, 6);";

const ASKS_WHETHER_IT_HAS_A_PATH_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fexp::tests::it_only_asks_whether_it_has_a_path",
    target: "lib",
    conditions: &["test"],
};
const ASKS_WHETHER_IT_HAS_A_PATH_BODY: &str = "        assert!(std::env::current_exe().is_ok());";
const ASKS_WHETHER_IT_HAS_A_PATH_BODY_CHANGED: &str =
    "        assert!(std::env::current_exe().is_err());";

const READS_THE_SIZE_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fexp::tests::it_reads_the_size_of_the_file_it_was_started_from",
    target: "lib",
    conditions: &["test"],
};
const READS_THE_SIZE_BODY: &str = "expect(\"读得到\").len();\n        assert!(size > 0);";
const READS_THE_SIZE_BODY_CHANGED: &str = "expect(\"读得到\").len();\n        assert!(size > 1);";

const SPAWN_MANIFEST: &str = "[package]\nname = \"fexs\"\n";
const SPAWN_FILE: &str = "crates/fexs/src/lib.rs";

/// 起子进程的四种写法与被起的那条测试。
const SPAWN_LIBRARY: &str = r#"#[cfg(test)]
mod tests {
    use std::process::Command;
    #[test]
    fn it_starts_itself_and_names_the_child() {
        let output = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::the_child_runs_alone"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_starts_itself_without_naming_a_test() {
        let output = Command::new(std::env::current_exe().expect("路径")).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_takes_the_path_first_and_hands_it_to_command() {
        let program = std::env::current_exe().expect("路径");
        let output = Command::new(program).args(["--exact", "tests::the_child_runs_alone"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_names_a_test_nobody_defines() {
        let _ = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).output();
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
}
"#;

const CHILD_BODY: &str = "        let value = 1;\n        assert_eq!(value, 1);";
const CHILD_BODY_CHANGED: &str = "        let value = 2;\n        assert_eq!(value, 2);";

const NAMES_THE_CHILD_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fexs::tests::it_starts_itself_and_names_the_child",
    target: "lib",
    conditions: &["test"],
};

/// 起了子进程而折不出点名的那一条测试的三种写法。
const SPAWNING_ENTRIES_WITHOUT_A_FOLDED_CHILD: [EntrySpecification; 3] = [
    EntrySpecification {
        entry_path: "fexs::tests::it_starts_itself_without_naming_a_test",
        target: "lib",
        conditions: &["test"],
    },
    EntrySpecification {
        entry_path: "fexs::tests::it_takes_the_path_first_and_hands_it_to_command",
        target: "lib",
        conditions: &["test"],
    },
    EntrySpecification {
        entry_path: "fexs::tests::it_names_a_test_nobody_defines",
        target: "lib",
        conditions: &["test"],
    },
];

const IMPORTED_MANIFEST: &str = "[package]\nname = \"fexu\"\n";
const IMPORTED_FILE: &str = "crates/fexu/src/lib.rs";

/// 一条 `use` 同时引进 `current_exe` 与 `Command`：那个 `use` 单元里有起子进程的词，而它里面的 `current_exe` 后面不跟括号。
const IMPORTED_LIBRARY: &str = r#"#[cfg(test)]
mod tests {
    use std::{env::current_exe, process::Command};
    #[test]
    fn it_asks_through_the_imported_name() {
        assert!(current_exe().is_ok());
    }
}
"#;

const ASKS_THROUGH_THE_IMPORTED_NAME_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fexu::tests::it_asks_through_the_imported_name",
    target: "lib",
    conditions: &["test"],
};

fn tree_of(
    label: &str,
    manifest_path: &str,
    manifest: &str,
    file: &str,
    library: &str,
) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            (manifest_path, manifest),
            (file, library),
            ("Cargo.lock", "# 夹具不解析依赖\n"),
        ],
    )
}

fn key_text(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

/// 只问路径的入口：键算得出；改无关的测试键不变；改它自己的函数体键变。
fn assert_a_path_only_entry_is_keyed_and_follows_only_its_own_body(
    label: &str,
    entry: &EntrySpecification,
    own_body: &str,
    own_body_changed: &str,
) {
    let tree = tree_of(
        label,
        "crates/fexp/Cargo.toml",
        PATH_ONLY_MANIFEST,
        PATH_ONLY_FILE,
        PATH_ONLY_LIBRARY,
    );
    let kept = key_text(&tree, entry)
        .unwrap_or_else(|reason| panic!("只问路径、不起子进程：不拒算 current-exe：{reason}"));
    tree.replace_exactly_once(PATH_ONLY_FILE, UNRELATED_BODY, UNRELATED_BODY_CHANGED);
    let after_the_unrelated_edit = key_text(&tree, entry).expect("改无关的测试之后照样算得出键");
    assert_eq!(
        kept, after_the_unrelated_edit,
        "改无关的测试，只问路径的入口键不变"
    );
    tree.replace_exactly_once(PATH_ONLY_FILE, own_body, own_body_changed);
    let after_its_own_edit = key_text(&tree, entry).expect("改它自己的函数体之后照样算得出键");
    assert_ne!(
        kept, after_its_own_edit,
        "改只问路径的入口自己的函数体，键变"
    );
}

/// 场景：`assert!(std::env::current_exe().is_ok())`，单元里没有起子进程的词。预期：不拒算、键只跟它自己的函数体变。
#[test]
fn a_test_asking_only_whether_it_has_a_path_is_keyed_and_follows_only_its_own_body() {
    assert_a_path_only_entry_is_keyed_and_follows_only_its_own_body(
        "current-exe-asks-whether-it-has-a-path",
        &ASKS_WHETHER_IT_HAS_A_PATH_ENTRY,
        ASKS_WHETHER_IT_HAS_A_PATH_BODY,
        ASKS_WHETHER_IT_HAS_A_PATH_BODY_CHANGED,
    );
}

/// 场景：取 `current_exe()` 的路径读程序文件的大小，单元里没有起子进程的词。预期：不拒算、键只跟它自己的函数体变。
#[test]
fn a_test_reading_the_size_of_its_own_program_file_is_keyed_and_follows_only_its_own_body() {
    assert_a_path_only_entry_is_keyed_and_follows_only_its_own_body(
        "current-exe-reads-the-size",
        &READS_THE_SIZE_ENTRY,
        READS_THE_SIZE_BODY,
        READS_THE_SIZE_BODY_CHANGED,
    );
}

/// 场景：同一单元里 `Command::new(current_exe())` 起子进程、`--exact` 后面点名同一目标里的那条测试。预期：不拒算，
/// 被点名的测试连进闭包，改它的函数体，起它的入口的键变。
#[test]
fn a_test_starting_itself_and_naming_the_child_links_the_child_and_follows_its_body() {
    let tree = tree_of(
        "current-exe-spawn-named",
        "crates/fexs/Cargo.toml",
        SPAWN_MANIFEST,
        SPAWN_FILE,
        SPAWN_LIBRARY,
    );
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(
        &workspace,
        &NAMES_THE_CHILD_ENTRY,
        &switches,
        HOST_TARGET_TRIPLE,
    )
    .expect("入口找得到");
    let lines = closure_lines(&workspace, &outcome, None);
    assert!(
        outcome.refusals.is_empty(),
        "点名的测试折得出、解得到：不拒算：{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("fn\tfexs::tests::the_child_runs_alone\t")),
        "子进程跑的那条测试连进闭包：{lines:?}"
    );
    let kept = key_text(&tree, &NAMES_THE_CHILD_ENTRY).expect("改前算得出键");
    tree.replace_exactly_once(SPAWN_FILE, CHILD_BODY, CHILD_BODY_CHANGED);
    let after = key_text(&tree, &NAMES_THE_CHILD_ENTRY).expect("改后算得出键");
    assert_ne!(kept, after, "子进程跑的测试改了，起它的入口的键变");
}

/// 场景：同一单元里起了子进程而折不出点名的那一条测试——没有 `--exact`、路径先存进变量再交 `Command::new`、点名的测试不在
/// （同一棵夹具树上三个入口，各判一次，不改树）。预期：三个都拒算 `current-exe`，落在夹具文件里。
#[test]
fn a_test_starting_itself_without_a_folded_child_name_is_still_refused_as_current_exe() {
    let tree = tree_of(
        "current-exe-spawn-unfolded",
        "crates/fexs/Cargo.toml",
        SPAWN_MANIFEST,
        SPAWN_FILE,
        SPAWN_LIBRARY,
    );
    for entry in &SPAWNING_ENTRIES_WITHOUT_A_FOLDED_CHILD {
        let reason = key_text(&tree, entry).expect_err("起了子进程、折不出点名的测试：拒算");
        assert!(
            reason.contains("refused\tcurrent-exe\tcrates/fexs/src/lib.rs:"),
            "{}：拒算行点名 current-exe 与夹具文件：{reason}",
            entry.entry_path
        );
    }
}

/// 场景：一条 `use` 同时引进 `current_exe` 与 `Command`，入口只问路径（`current_exe().is_ok()`）。那条 `use` 随用到的
/// `current_exe` 进闭包，它里面有 `Command` 这个词，而 `current_exe` 后面跟的是逗号不是括号。预期：不拒算、键算得出。
#[test]
fn a_use_importing_current_exe_together_with_command_does_not_refuse_a_test_asking_only_for_its_path(
) {
    let tree = tree_of(
        "current-exe-imported",
        "crates/fexu/Cargo.toml",
        IMPORTED_MANIFEST,
        IMPORTED_FILE,
        IMPORTED_LIBRARY,
    );
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(
        &workspace,
        &ASKS_THROUGH_THE_IMPORTED_NAME_ENTRY,
        &switches,
        HOST_TARGET_TRIPLE,
    )
    .expect("入口找得到");
    let lines = closure_lines(&workspace, &outcome, None);
    assert!(
        lines.iter().any(|line| line.starts_with("use\t")),
        "引进 current_exe 的那条 use 在闭包里（不然这一格判不出后面不跟括号的 current_exe 算不算）：{lines:?}"
    );
    assert!(
        key_text(&tree, &ASKS_THROUGH_THE_IMPORTED_NAME_ENTRY).is_ok(),
        "use 里后面不跟括号的 current_exe 不算问路径、也不算起自己：不拒算：{lines:?}"
    );
}
