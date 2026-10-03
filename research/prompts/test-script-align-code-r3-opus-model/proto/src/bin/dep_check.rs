//! L2 / L4 第 1741 行：`crash_identity` 的依赖用例（每条登记的崩溃枚举用例的判法代码不许经 impl 头带进随机历史脚手架）照它的断言重算，
//! 用例本身（checker 档）不跑。参数是一个或几个仓根（冻结副本、按变异行改过原文的副本：判法代码按原文收，不用编译）。
//! 每个仓根、每条登记的用例一行：收到的文件数、history.rs 在不在与原因、numeric_environment_variable.rs 在不在与原因、这一条断言判 PASS / FAIL。
use std::path::PathBuf;

use singlefs_checker_tier::crash_identity::judging_code_by_module_references;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;

fn main() {
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    for root in std::env::args().skip(1) {
        let root = PathBuf::from(root).canonicalize().expect("仓根");
        let registry = std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表在");
        let targets: Vec<String> = registry
            .lines()
            .filter(|line| line.starts_with("crash-case:"))
            .map(|line| line.split('\t').find_map(|c| c.strip_prefix("test=")).expect("test= 列").split(':').nth(1).expect("目标").to_string())
            .collect();
        let mut failing = 0;
        for target in &targets {
            let entry = format!("crates/singlefs-checker-tier/tests/{target}.rs");
            let code = judging_code_by_module_references(&root, &[entry.as_str()], &toolchain).expect("算得出");
            let history = code.reason_of_each_file.get("crates/singlefs-harness/src/history.rs").cloned();
            let numeric = code.reason_of_each_file.get("crates/singlefs-harness/src/numeric_environment_variable.rs").cloned();
            let amplification = code.files.iter().any(|f| f == "crates/singlefs-checker-tier/src/crash_amplification.rs");
            let ok = history.as_deref().is_none_or(|reason| reason.starts_with("referenced by crates/singlefs-checker-tier/tests/"));
            if !ok {
                failing += 1;
            }
            println!(
                "DEP root={} target={target} files={} crash_amplification_in={amplification} history={:?} numeric_environment_variable={:?} assertion={}",
                root.display(), code.files.len(), history, numeric, if ok { "PASS" } else { "FAIL" }
            );
        }
        println!("DEP_TOTAL root={} targets={} failing={failing} test_would_be={}", root.display(), targets.len(), if failing > 0 { "red" } else { "green" });
    }
}
