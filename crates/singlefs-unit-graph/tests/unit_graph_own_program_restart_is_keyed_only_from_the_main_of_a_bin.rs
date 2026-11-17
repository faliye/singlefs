//! 程序起自己（`src/closure.rs` 文件头「程序起自己」，`scripts/gate/lib_function_units.py` 文件头同一判法）：`current_exe()` 的路径一个字不改地交给
//! `Command::new`、入口恰好是一个 bin 目标的 `fn main` 又不按测试编的，子进程跑的还是这个 `main`，不拒算、不连边，键跟闭包里的代码变。
//! 路径改过（起的可能是别的程序）或入口不是那个 `main`（按测试编的进程里 `main` 是 libtest 的，参数选中哪条测试说不清）的拒算 `current-exe`。
//! 夹具只切词、不编译。

mod common_fixture_workspace;

use common_fixture_workspace::{
    key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::{compute_closure_of_entries, ClosureRequest, TargetSelector};

const MANIFEST_PATH: &str = "crates/frestart/Cargo.toml";
const MANIFEST: &str = "[package]\nname = \"frestart\"\n";
const MAIN_FILE: &str = "crates/frestart/src/main.rs";
const LIBRARY_FILE: &str = "crates/frestart/src/lib.rs";

const MAIN_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "frestart::main",
    target: "bin:frestart",
    conditions: &[],
};

/// `main` 分两个角色：带参数的那个角色跑 `role_body`，不带参数的起自己。起自己的写法由各条用例给。
fn program_with_restart(restart_function: &str) -> String {
    format!(
        "fn role_body() -> u32 {{\n    1\n}}\n\
         {restart_function}\n\
         fn main() {{\n    if std::env::args().count() > 1 {{\n        role_body();\n    }} else {{\n        restart();\n    }}\n}}\n"
    )
}

const RESTART_WRITTEN_IN_COMMAND_NEW: &str = r#"fn restart() {
    let mut command = std::process::Command::new(std::env::current_exe().expect("路径"));
    command.arg("role");
    command.status().expect("起得来");
}"#;

const RESTART_BOUND_WITH_A_DIVERGING_FALLBACK: &str = r#"fn restart() {
    let this_program = std::env::current_exe().unwrap_or_else(|error| {
        eprintln!("取不到路径：{error}");
        std::process::exit(78)
    });
    let status = std::process::Command::new(&this_program).arg("role").status().unwrap_or_else(|error| {
        eprintln!("起不了 {}：{error}", this_program.display());
        std::process::exit(78)
    });
    drop(status);
}"#;

const RESTART_BOUND_BY_A_MATCH_RETURNING_ON_ERROR: &str = r#"fn restart() -> Result<(), String> {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => return Err(error.to_string()),
    };
    let child = std::process::Command::new(executable).arg("role").spawn();
    child.map(drop).map_err(|error| error.to_string())
}"#;

const RESTART_WITH_QUESTION_MARK: &str = r#"fn restart() -> std::io::Result<()> {
    let program = std::env::current_exe()?;
    std::process::Command::new(program).arg("role").status().map(drop)
}"#;

fn tree_of(label: &str, files: &[(&str, &str)]) -> FixtureTree {
    let mut all_files = vec![
        (MANIFEST_PATH, MANIFEST),
        ("Cargo.lock", "# 夹具不解析依赖\n"),
    ];
    all_files.extend_from_slice(files);
    FixtureTree::materialize(label, &all_files)
}

fn key_text(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

fn assert_refused_as_current_exe(reason: &str) {
    assert!(
        reason.contains("refused\tcurrent-exe\tcrates/frestart/src/"),
        "拒算行点名 current-exe 与夹具文件：{reason}"
    );
}

/// 起自己的写法从 `main` 入口算得出键，改子进程那个角色跑的函数键变。
fn assert_keyed_from_main_and_follows_the_role_of_the_child(label: &str, restart_function: &str) {
    let source = program_with_restart(restart_function);
    let tree = tree_of(label, &[(MAIN_FILE, source.as_str())]);
    let kept = key_text(&tree, &MAIN_ENTRY)
        .unwrap_or_else(|reason| panic!("bin 的 main 起自己不拒算：{reason}"));
    tree.replace_exactly_once(
        MAIN_FILE,
        "fn role_body() -> u32 {\n    1",
        "fn role_body() -> u32 {\n    2",
    );
    let after_the_edit = key_text(&tree, &MAIN_ENTRY).expect("改了角色的函数体照样算得出键");
    assert_ne!(kept, after_the_edit, "子进程那个角色跑的函数改了，键要变");
}

fn assert_refused_from_main(label: &str, restart_function: &str) {
    let source = program_with_restart(restart_function);
    let tree = tree_of(label, &[(MAIN_FILE, source.as_str())]);
    let reason =
        key_text(&tree, &MAIN_ENTRY).expect_err("路径改过或交给了别处，起的可能是别的程序，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：`Command::new(current_exe().expect("…"))`（E162 块存储 `child_command` 的形状），入口是 bin 的 `main`。预期：算得出键，键跟角色函数变。
#[test]
fn bin_main_restarting_itself_inside_command_new_is_keyed_and_follows_the_role_of_the_child() {
    assert_keyed_from_main_and_follows_the_role_of_the_child(
        "restart-inside-command-new",
        RESTART_WRITTEN_IN_COMMAND_NEW,
    );
}

/// 场景：路径绑给 `let`、失败时闭包 `process::exit`，用法只有 `Command::new(&名字)` 与 `名字.display()`（E16 第五跑的形状）。预期：同上。
#[test]
fn bin_main_restarting_itself_through_a_binding_with_a_diverging_fallback_is_keyed() {
    assert_keyed_from_main_and_follows_the_role_of_the_child(
        "restart-bound-diverging-fallback",
        RESTART_BOUND_WITH_A_DIVERGING_FALLBACK,
    );
}

/// 场景：`let 名字 = match current_exe() { Ok(名字) => 名字, Err(…) => return … };`（E162 断电装置的形状）。预期：同上。
#[test]
fn bin_main_restarting_itself_through_a_match_returning_on_error_is_keyed() {
    assert_keyed_from_main_and_follows_the_role_of_the_child(
        "restart-bound-by-match",
        RESTART_BOUND_BY_A_MATCH_RETURNING_ON_ERROR,
    );
}

/// 场景：`let 名字 = current_exe()?;`。预期：同上。
#[test]
fn bin_main_restarting_itself_through_a_question_mark_binding_is_keyed() {
    assert_keyed_from_main_and_follows_the_role_of_the_child(
        "restart-question-mark",
        RESTART_WITH_QUESTION_MARK,
    );
}

/// 场景：起自己的函数在 lib 里，由 bin 的 `main` 调到。预期：子进程还是这个 bin，算得出键。
#[test]
fn library_function_restarting_the_program_is_keyed_when_reached_from_the_main_of_a_bin() {
    let tree = tree_of(
        "restart-in-library-from-main",
        &[
            (
                LIBRARY_FILE,
                "pub fn restart_from_library() {\n    let _ = std::process::Command::new(std::env::current_exe().expect(\"路径\")).status();\n}\n",
            ),
            (MAIN_FILE, "fn main() {\n    frestart::restart_from_library();\n}\n"),
        ],
    );
    let kept = key_text(&tree, &MAIN_ENTRY)
        .unwrap_or_else(|reason| panic!("从 main 调到 lib 里起自己不拒算：{reason}"));
    tree.replace_exactly_once(LIBRARY_FILE, "expect(\"路径\")", "expect(\"程序路径\")");
    let after_the_edit = key_text(&tree, &MAIN_ENTRY).expect("改了 lib 函数照样算得出键");
    assert_ne!(
        kept, after_the_edit,
        "lib 里起自己的函数在 main 的闭包里，改了键要变"
    );
}

/// 场景：同一个 lib 函数，入口是 lib 单测（进程是 libtest）。预期：拒算。
#[test]
fn the_same_library_function_is_refused_when_reached_from_a_test_of_the_library() {
    let tree = tree_of(
        "restart-in-library-from-test",
        &[(
            LIBRARY_FILE,
            "pub fn restart_from_library() {\n    let _ = std::process::Command::new(std::env::current_exe().expect(\"路径\")).status();\n}\n\
             #[cfg(test)]\nmod tests {\n    #[test]\n    fn it_restarts() {\n        super::restart_from_library();\n    }\n}\n",
        )],
    );
    let entry = EntrySpecification {
        entry_path: "frestart::tests::it_restarts",
        target: "lib",
        conditions: &["test"],
    };
    let reason = key_text(&tree, &entry).expect_err("lib 单测里起自己，子进程是 libtest，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：bin 里的单测调同一个起自己的函数（`cargo test --bin`）。预期：拒算：进程的 `main` 是 libtest 的，不带 `--exact` 的参数选中哪条测试说不清。
#[test]
fn test_inside_the_bin_calling_the_restart_function_is_refused() {
    let source = format!(
        "{}\n#[cfg(test)]\nmod tests {{\n    #[test]\n    fn it_restarts() {{\n        super::restart();\n    }}\n}}\n",
        program_with_restart(RESTART_WRITTEN_IN_COMMAND_NEW)
    );
    let tree = tree_of("restart-from-test-in-bin", &[(MAIN_FILE, source.as_str())]);
    let entry = EntrySpecification {
        entry_path: "frestart::tests::it_restarts",
        target: "bin:frestart",
        conditions: &["test"],
    };
    let reason = key_text(&tree, &entry).expect_err("入口是 bin 里的单测，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：bin 里的单测调 `main`（闭包里有 `main`，可进程仍是 libtest）。预期：拒算：判的是入口，不是闭包里有没有 `main`。
#[test]
fn test_inside_the_bin_calling_main_is_refused_although_main_is_in_its_closure() {
    let source = format!(
        "{}\n#[cfg(test)]\nmod tests {{\n    #[test]\n    fn it_calls_main() {{\n        super::main();\n    }}\n}}\n",
        program_with_restart(RESTART_WRITTEN_IN_COMMAND_NEW)
    );
    let tree = tree_of("restart-test-calls-main", &[(MAIN_FILE, source.as_str())]);
    let entry = EntrySpecification {
        entry_path: "frestart::tests::it_calls_main",
        target: "bin:frestart",
        conditions: &["test"],
    };
    let reason = key_text(&tree, &entry).expect_err("入口不是 main，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：入口写 `main`，但这一趟按测试编（`--cfg test`）。预期：拒算。
#[test]
fn the_main_of_a_bin_compiled_as_a_test_is_refused() {
    let source = program_with_restart(RESTART_WRITTEN_IN_COMMAND_NEW);
    let tree = tree_of("restart-main-as-test", &[(MAIN_FILE, source.as_str())]);
    let entry = EntrySpecification {
        entry_path: "frestart::main",
        target: "bin:frestart",
        conditions: &["test"],
    };
    let reason = key_text(&tree, &entry).expect_err("按测试编的 main 不是这个进程的入口，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：入口是 bin 里起自己的那个辅助函数本身（闭包里没有 `main`）。预期：拒算。
#[test]
fn helper_function_of_the_bin_as_the_entry_is_refused() {
    let source = program_with_restart(RESTART_WRITTEN_IN_COMMAND_NEW);
    let tree = tree_of("restart-helper-entry", &[(MAIN_FILE, source.as_str())]);
    let entry = EntrySpecification {
        entry_path: "frestart::restart",
        target: "bin:frestart",
        conditions: &[],
    };
    let reason = key_text(&tree, &entry).expect_err("子进程的 main 不在这个入口的闭包里，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：入口是模块里另一个也叫 `main` 的函数。预期：拒算：它不是这个程序的入口。
#[test]
fn function_named_main_inside_a_module_as_the_entry_is_refused() {
    let source = format!(
        "{}\nmod inner {{\n    pub fn main() {{\n        super::restart();\n    }}\n}}\n",
        program_with_restart(RESTART_WRITTEN_IN_COMMAND_NEW)
    );
    let tree = tree_of("restart-nested-main", &[(MAIN_FILE, source.as_str())]);
    let entry = EntrySpecification {
        entry_path: "frestart::inner::main",
        target: "bin:frestart",
        conditions: &[],
    };
    let reason = key_text(&tree, &entry).expect_err("模块里的 main 不是程序入口，拒算");
    assert_refused_as_current_exe(&reason);
}

/// 场景：一组入口里除了 `main` 还有别的函数。预期：拒算：这一组的判定不全从 `main` 起。
#[test]
fn an_entry_group_with_main_and_another_function_is_refused() {
    let source = program_with_restart(RESTART_WRITTEN_IN_COMMAND_NEW);
    let tree = tree_of("restart-entry-group", &[(MAIN_FILE, source.as_str())]);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let request = ClosureRequest {
        entry_path: "frestart::main".to_string(),
        target: TargetSelector::parse("bin:frestart").expect("测试里写的目标认得"),
        requested_conditions: Vec::new(),
        uses_default_features: true,
        target_triple: HOST_TARGET_TRIPLE.to_string(),
    };
    let outcome = compute_closure_of_entries(
        &workspace,
        &request,
        &["frestart::role_body".to_string()],
        &switches,
    )
    .expect("两个入口都找得到");
    assert!(
        outcome
            .refusals
            .iter()
            .any(|refusal| refusal.form.output_name() == "current-exe"),
        "一组入口里另有别的入口，拒算 current-exe：{:?}",
        outcome.refusals
    );
}

/// 场景：`Command::new(current_exe().expect("…").with_file_name("other"))`：起的是同目录的另一个程序。预期：拒算。
#[test]
fn main_starting_a_sibling_program_written_inside_command_new_is_refused() {
    assert_refused_from_main(
        "sibling-inside-command-new",
        r#"fn restart() {
    let _ = std::process::Command::new(std::env::current_exe().expect("路径").with_file_name("other")).status();
}"#,
    );
}

/// 场景：路径绑给名字，`Command::new(名字.with_file_name("other"))`。预期：拒算。
#[test]
fn main_starting_a_sibling_program_through_a_binding_is_refused() {
    assert_refused_from_main(
        "sibling-through-binding",
        r#"fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new(program.with_file_name("other")).status();
}"#,
    );
}

/// 场景：`let 名字 = current_exe().expect("…").parent()…`：绑给名字之前就改了。预期：拒算。
#[test]
fn binding_of_the_parent_directory_of_the_program_is_refused() {
    assert_refused_from_main(
        "binding-of-parent",
        r#"fn restart() {
    let program = std::env::current_exe().expect("路径").parent().expect("有上级").join("other");
    let _ = std::process::Command::new(program).status();
}"#,
    );
}

/// 场景：`.unwrap_or_else` 的闭包交回另一个路径。预期：拒算：失败时起的是别的程序。
#[test]
fn fallback_handing_another_path_is_refused() {
    assert_refused_from_main(
        "fallback-another-path",
        r#"fn restart() {
    let program = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("/bin/other"));
    let _ = std::process::Command::new(program).status();
}"#,
    );
}

/// 场景：`match current_exe()` 的 `Err` 分支交回另一个路径。预期：拒算。
#[test]
fn match_whose_error_arm_hands_another_path_is_refused() {
    assert_refused_from_main(
        "match-error-another-path",
        r#"fn restart() {
    let program = match std::env::current_exe() {
        Ok(program) => program,
        Err(_) => std::path::PathBuf::from("/bin/other"),
    };
    let _ = std::process::Command::new(program).status();
}"#,
    );
}

/// 场景：`match current_exe()` 的 `Ok` 分支交的不是绑出来的那个路径。预期：拒算。
#[test]
fn match_whose_ok_arm_hands_another_path_is_refused() {
    assert_refused_from_main(
        "match-ok-another-path",
        r#"fn restart() -> Option<()> {
    let other = std::path::PathBuf::from("/bin/other");
    let program = match std::env::current_exe() {
        Ok(_program) => other,
        Err(_) => return None,
    };
    std::process::Command::new(program).status().ok().map(drop)
}"#,
    );
}

/// 场景：路径是交给别的程序的参数（`Command::new("sh").arg(名字)`）。预期：拒算。
#[test]
fn the_program_path_handed_as_an_argument_to_another_program_is_refused() {
    assert_refused_from_main(
        "path-as-argument",
        r#"fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new("sh").arg(&program).status();
}"#,
    );
}

/// 场景：路径交给 `Command::new` 起了一次，后面又交给一个辅助函数。预期：拒算：辅助函数拿它做什么这里看不到。
#[test]
fn the_program_path_also_handed_to_a_helper_is_refused() {
    assert_refused_from_main(
        "path-to-helper",
        r#"fn helper(_path: &std::path::Path) {}
fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new(&program).status();
    helper(&program);
}"#,
    );
}

/// 场景：路径交给 `Command::new` 起了一次，再 `let` 同一个名字绑成别的路径又起一次。预期：拒算。
#[test]
fn name_bound_again_to_another_path_is_refused() {
    assert_refused_from_main(
        "rebound-name",
        r#"fn restart() {
    let program = std::env::current_exe().expect("路径");
    let _ = std::process::Command::new(&program).status();
    let program = program.with_file_name("other");
    let _ = std::process::Command::new(program).status();
}"#,
    );
}

/// 场景：`--exact` 后面点名的测试在这个 bin 里没有。预期：走子进程测试边那一支，照旧拒算（不当起自己放过）。
#[test]
fn main_naming_a_test_nobody_defines_after_exact_is_still_refused() {
    assert_refused_from_main(
        "exact-names-nobody",
        r#"fn restart() {
    let _ = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).status();
}"#,
    );
}
