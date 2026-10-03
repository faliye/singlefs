//! r3 L2 探针：对一棵树（参数 1，带 crates/ 与 .claude/gate.d/stage-inputs.tsv）里每一条登记的崩溃枚举用例，
//! 复现 `crash_identity::tests::every_registered_crash_case_depends_on_…` 的取数与那条关于 history.rs 的断言（不起 checker 档测试二进制，只调库的公开函数）。
use std::path::Path;

use singlefs_checker_tier::crash_identity::judging_code_by_module_references;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;

fn main() {
    let root_text = std::env::args().nth(1).expect("参数 1：树根");
    let root = Path::new(&root_text);
    let registry = std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表");
    let targets: Vec<String> = registry
        .lines()
        .filter(|line| line.starts_with("crash-case:"))
        .map(|line| {
            let column = line.split('\t').find_map(|column| column.strip_prefix("test=")).expect("test=");
            column.split(':').nth(1).expect("目标").to_string()
        })
        .collect();
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    println!("registered targets: {}", targets.len());
    let mut violated = 0;
    for target in &targets {
        let entry = format!("crates/singlefs-checker-tier/tests/{target}.rs");
        let code = judging_code_by_module_references(root, &[entry.as_str()], &toolchain).expect("算得出");
        let numeric = "crates/singlefs-harness/src/numeric_environment_variable.rs";
        let history = "crates/singlefs-harness/src/history.rs";
        let history_reason = code.reason_of_each_file.get(history);
        let history_ok = history_reason.map_or(true, |reason| reason.starts_with("referenced by crates/singlefs-checker-tier/tests/"));
        if !history_ok {
            violated += 1;
        }
        if std::env::var("R3_LIST").as_deref() == Ok(target.as_str()) {
            for file in &code.files {
                println!("F {file}");
            }
        }
        println!(
            "T {target}: files={} numeric_in={} numeric_reason={:?} history_reason={:?} guard_ok={history_ok}",
            code.files.len(),
            code.files.iter().any(|f| f == numeric),
            code.reason_of_each_file.get(numeric),
            history_reason
        );
    }
    println!("guard-violations: {violated}");
}
