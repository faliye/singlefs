//! 攻方腿原型：判法闭包（复用键里的判法版本）罩不罩得住各崩溃枚举用例的观察者与逐状态挂载那几截。
use std::collections::BTreeSet;
use std::path::PathBuf;

use singlefs_checker_tier::crash_identity::{judging_code_by_module_references, judging_code_of_the_judges};
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

#[test]
fn observer_files_outside_the_judge_closure() {
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    let judges = judging_code_of_the_judges(&root(), &toolchain).expect("判法闭包");
    let judged: BTreeSet<String> = judges.files.iter().cloned().collect();
    println!("JUDGE_CLOSURE files={} stopped_at={:?}", judged.len(), judges.stopped_at);
    for f in &judges.files {
        println!("JUDGE_FILE {f}");
    }
    for core in ["mount.rs", "unlink.rs", "instance_table_sweep.rs", "admission.rs", "recovery.rs", "transaction.rs"] {
        let path = format!("crates/singlefs-core/src/{core}");
        println!("CORE_IN_JUDGE_CLOSURE {path} {}", judged.contains(&path));
    }
    for target in [
        "crash_enumeration_new_pool_file_creation_stream",
        "crash_enumeration_unlink_after_the_multi_record_publish_stream",
        "crash_enumeration_floor_raise_pushed_by_admission",
        "crash_points_of_the_instance_table_sweep",
        "crash_enumeration_writable_mount_of_an_io_min_8192_pool",
        "crash_injection_campaign",
        "crash_injection_read_faulted_states_never_hold_the_witness_criterion",
    ] {
        let entry = format!("crates/singlefs-checker-tier/tests/{target}.rs");
        let code = judging_code_by_module_references(&root(), &[entry.as_str()], &toolchain).expect("用例闭包");
        let outside: Vec<&String> = code.files.iter().filter(|f| !judged.contains(*f)).collect();
        println!("CASE_CLOSURE target={target} files={} outside_judge_closure={}", code.files.len(), outside.len());
        for f in outside {
            println!("  OUTSIDE {target} {f}");
        }
    }
}
