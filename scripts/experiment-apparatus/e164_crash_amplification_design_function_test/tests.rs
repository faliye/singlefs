use super::*;

#[test]
fn every_anchor_of_section_seven_holds() {
    for (name, measured, expected) in anchor_rows() {
        assert_eq!(measured, expected, "第七节锚点 {name}");
    }
}

#[test]
fn strongest_form_has_no_error_on_two_blocks_and_two_cards() {
    let report = explore(&pipeline::STRONGEST_FORM, &Geometry { blocks: 2, cards: 2, backlog_cap: 1, batch: 1 });
    assert!(report.kill_transitions > 0 && report.judging_change_transitions > 0 && report.terminal_states > 0, "被杀、换判法、终局都走到了");
    assert_eq!(report.missing_result + report.wrong_input + report.double_tally + report.stale_after_change + report.stuck, 0, "最强形 0 错误");
}

#[test]
fn claims_with_a_lease_leave_no_dead_block_and_no_other_error() {
    let (_, form) = claim_forms().into_iter().find(|(name, _)| *name == "claims_persisted_with_lease").expect("有带租期那一形");
    let report = explore(&form, &Geometry { blocks: 2, cards: 2, backlog_cap: 1, batch: 1 });
    assert!(report.takeover_after_lease_transitions > 0, "租期到期后另一张卡接手的迁移走到了");
    assert_eq!(report.dead_block_terminals, 0, "带租期不留死块");
    assert_eq!(report.missing_result + report.wrong_input + report.double_tally + report.stale_after_change, 0, "带租期其余四种错误 = 0");
}

#[test]
fn function_items_ignore_items_outside_functions() {
    assert_eq!(lexical::function_items_text("const A: u8 = 1; fn f() { g() }"), lexical::function_items_text("const A: u8 = 2; fn f() { g() }"));
    assert_ne!(lexical::function_items_text("fn f() { g(1) }"), lexical::function_items_text("fn f() { g(2) }"));
    assert_eq!(lexical::function_items_text("#[cfg(test)] mod tests { fn t() {} } fn f() {}"), "fn f ( ) { }");
}

#[test]
fn walked_files_skip_a_file_without_non_test_functions() {
    assert!(!lexical::has_non_test_function("pub const A: u8 = 1;\n#[cfg(test)]\nmod tests { #[test] fn t() {} }"));
    assert!(lexical::has_non_test_function("pub fn f() {}"));
}

#[test]
fn own_source_reads_are_found_through_include_str() {
    let directory = std::env::temp_dir().join(format!("e164-own-source-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("pkg/src")).expect("建得了临时目录");
    std::fs::write(directory.join("pkg/src/reader.rs"), "const TEXT: &str = include_str!(\"model.rs\");").expect("写得了");
    std::fs::write(directory.join("pkg/src/model.rs"), "pub fn m() {}").expect("写得了");
    let found = node_code::own_source_files(&directory, &["pkg/src/reader.rs".to_string(), "pkg/src/model.rs".to_string()]);
    std::fs::remove_dir_all(&directory).expect("删得掉临时目录");
    assert_eq!(found.into_iter().collect::<Vec<String>>(), vec!["pkg/src/model.rs".to_string()]);
}

#[test]
fn code_changing_configuration_takes_rustflags_and_ignores_term_and_net() {
    let directory = std::env::temp_dir().join(format!("e164-configuration-{}", std::process::id()));
    std::fs::create_dir_all(directory.join(".cargo")).expect("建得了临时目录");
    std::fs::write(directory.join(".cargo/config.toml"), "[term]\ncolor = \"always\"\n[net]\noffline = true\n").expect("写得了");
    let empty = std::collections::BTreeMap::from([("HOME".to_string(), "/nonexistent-e164".to_string())]);
    let mut with_rustflags = empty.clone();
    with_rustflags.insert("RUSTFLAGS".to_string(), "-C target-cpu=native".to_string());
    let quiet = fingerprints::code_changing_configuration_lines(&directory, &empty, false);
    let loud = fingerprints::code_changing_configuration_lines(&directory, &with_rustflags, false);
    std::fs::remove_dir_all(&directory).expect("删得掉临时目录");
    assert!(quiet.is_empty(), "[term] 与 [net] 不进指纹：{quiet:?}");
    assert_eq!(loud, vec!["environment RUSTFLAGS=-C target-cpu=native".to_string()]);
}

/// 在临时仓里起 git 造样本历史：不读用户与系统的 git 配置，不继承外层的 GIT_DIR 这类变量，打出标准输出。
/// 样本历史要 `init`、`switch`、`commit`，不在装置对真仓起子进程的白名单里（`common::assert_subprocess_is_allowed`），所以这里直接起 git；只写传进来的临时目录。
fn git_in_sample_repository(directory: &std::path::Path, arguments: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(["-c", "user.name=e164", "-c", "user.email=e164@example.invalid", "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null"])
        .args(arguments)
        .current_dir(directory)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("PATH 上有 git");
    assert!(output.status.success(), "git {} 失败：{}", arguments.join(" "), String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// W3 的域要罩住全部引用可达的提交，不只是当前分支：样本仓里主干 2 次提交，另一条分支再带 1 次主干到不了的提交，一共 3 次。
/// 3 是照造法数出来的绝对值，不从被测函数或同一条 git 命令读；只扫当前分支（`rev-list HEAD`）会得 2。
#[test]
fn frozen_file_name_scan_covers_every_commit() {
    let directory = std::env::temp_dir().join(format!("e164-frozen-file-name-scan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("建得了临时目录");
    git_in_sample_repository(&directory, &["init", "--quiet", "--initial-branch=main"]);
    git_in_sample_repository(&directory, &["commit", "--quiet", "--allow-empty", "-m", "主干一"]);
    git_in_sample_repository(&directory, &["switch", "--quiet", "--create", "side"]);
    git_in_sample_repository(&directory, &["commit", "--quiet", "--allow-empty", "-m", "只在旁支"]);
    let side_only = git_in_sample_repository(&directory, &["rev-parse", "side"]);
    git_in_sample_repository(&directory, &["switch", "--quiet", "main"]);
    git_in_sample_repository(&directory, &["commit", "--quiet", "--allow-empty", "-m", "主干二"]);
    let scanned = repository_scans_frozen_file_names::all_commits(&directory);
    std::fs::remove_dir_all(&directory).expect("删得掉临时目录");
    assert_eq!(scanned.len(), 3, "W3 扫到的提交数 = 样本仓造出的 3 次提交：{scanned:?}");
    assert!(scanned.contains(&side_only), "只在旁支上的提交也扫到了：{scanned:?}");
}

#[test]
fn node_reruns_when_its_own_target_file_changes() {
    let closure = std::collections::BTreeSet::from(["singlefs-format".to_string()]);
    let untouched = std::collections::BTreeSet::new();
    let changed = vec!["crates/singlefs-verification/tests/a.rs".to_string()];
    assert!(repository_scans_crate_closure_reruns::node_reruns(&closure, &untouched, Some("crates/singlefs-verification/tests/a.rs"), &changed));
    assert!(!repository_scans_crate_closure_reruns::node_reruns(&closure, &untouched, Some("crates/singlefs-verification/tests/b.rs"), &changed));
    let touched = std::collections::BTreeSet::from(["singlefs-format".to_string()]);
    assert!(repository_scans_crate_closure_reruns::node_reruns(&closure, &touched, None, &[]));
}

#[test]
fn one_block_one_card_strongest_without_kills_has_one_clean_terminal() {
    let report = explore(
        &ModelForm { allow_kills: false, allow_judging_change: false, ..pipeline::STRONGEST_FORM },
        &Geometry { blocks: 1, cards: 1, backlog_cap: 1, batch: 1 },
    );
    assert_eq!(report.terminal_states, 1, "7.3：终局只有 1 个");
    assert_eq!(report.missing_result + report.wrong_input + report.double_tally + report.stale_after_change + report.stuck, 0, "7.3：五种错误都 = 0");
}
