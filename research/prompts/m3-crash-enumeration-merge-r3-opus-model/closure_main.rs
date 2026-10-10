
// ============================== 模型的驱动（这一段是攻方腿写的，不是抽出来的） ==============================

const SETTING_FILES: &[&str] = &[
    "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs",
    "crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs",
    "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
    "crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs",
    "crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs",
    "crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs",
    "crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs",
];
const WATCHED: &[&str] = &[
    "crates/singlefs-checker-tier/src/crash_amplification.rs",
    "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs",
    "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
    "crates/singlefs-checker-tier/src/verdict_store.rs",
    "crates/singlefs-checker-tier/src/crash_identity.rs",
    "crates/singlefs-checker-tier/src/layer0_progress.rs",
    "crates/singlefs-checker-tier/src/crash_injection.rs",
    "crates/singlefs-checker-tier/src/bad_disk_input.rs",
    "crates/singlefs-harness/tests/common/mod.rs",
    "crates/singlefs-harness/src/history.rs",
];

fn stub() -> Layer0ToolchainIdentity {
    Layer0ToolchainIdentity { rustc_version_lines: String::new(), cargo_version: String::new(), target_triple: String::new() }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = PathBuf::from(&args[1]);
    let out_dir = PathBuf::from(&args[2]);
    let toolchain = stub();
    let judges = judging_code_of_the_judges(&root, &toolchain).expect("判器闭包");
    let has = |code: &JudgingCode, path: &str| code.files.iter().any(|file| file == path);
    // 自证一：抽出来的算法在今天的树上复现 crash_identity.rs 自己的测试里钉的判器闭包成员与非成员（同一张表抄过来）。
    let required = [
        "crates/singlefs-checker-tier/src/crash.rs", "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
        "crates/singlefs-checker-tier/src/crash_judge_tables.rs", "crates/singlefs-checker-tier/src/crash_facts.rs",
        "crates/singlefs-checker/src/walk.rs", "crates/singlefs-checker/src/image.rs", "crates/singlefs-core/src/recovery.rs",
        "crates/singlefs-replay-prefix/src/lib.rs", "crates/singlefs-format/src/lib.rs", "crates/singlefs-harness/src/memory_pool.rs",
    ];
    let excluded = [
        "crates/singlefs-checker-tier/src/crash_amplification.rs", "crates/singlefs-checker-tier/src/verdict_store.rs",
        "crates/singlefs-checker-tier/src/crash_identity.rs", "crates/singlefs-checker-tier/src/layer0_progress.rs",
        "crates/singlefs-checker-tier/src/crash_verify_gpu.rs", "crates/singlefs-checker-tier/src/lib.rs", "crates/singlefs-core/src/lib.rs",
        "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs", "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
    ];
    let mut mismatched: Vec<String> = Vec::new();
    for file in required { if !has(&judges, file) { mismatched.push(format!("missing:{file}")); } }
    for file in excluded { if has(&judges, file) { mismatched.push(format!("unexpected:{file}")); } }
    // 自证二（会红）：不设停止表时进度模块 layer0_progress.rs 就进了判器闭包——停止表这一刀真在起作用，模型分得出。
    let unstopped = judging_code_by_module_references(&root, JUDGE_ENTRY_FILES, &toolchain).expect("不停的闭包");
    let mutant_red = has(&unstopped, "crates/singlefs-checker-tier/src/layer0_progress.rs");
    println!("MODEL judges files={} stopped={} unstopped_files={} unstopped_has_layer0_progress={mutant_red}",
        judges.files.len(), judges.stopped_at.len(), unstopped.files.len());
    let stop_files: Vec<&str> = JUDGE_CLOSURE_STOP_FILES.iter().map(|(file, _)| *file).collect();
    std::fs::create_dir_all(&out_dir).expect("输出目录");
    std::fs::write(out_dir.join("judges.txt"), judges.files.iter().map(|file| format!("{file}\n")).collect::<String>()).expect("写判器闭包清单");
    for setting in SETTING_FILES {
        for (label, stops) in [("no_stop", &[][..]), ("judge_stops", &stop_files[..])] {
            let code = judging_code_by_module_references_stopping_at(&root, &[setting], stops, &toolchain).expect("设点文件的闭包");
            let watched: Vec<String> = WATCHED.iter().map(|file| {
                let short = file.rsplit('/').next().unwrap();
                let short = if file.contains("harness/tests/common") { "harness_tests_common_mod.rs" } else { short };
                format!("{short}={}", if has(&code, file) { 1 } else { 0 })
            }).collect();
            let extra: Vec<&String> = code.files.iter().filter(|file| !has(&judges, file)).collect();
            let union: BTreeSet<&String> = code.files.iter().chain(judges.files.iter()).collect();
            let name = setting.rsplit('/').nth(if setting.ends_with("mod.rs") { 1 } else { 0 }).unwrap().trim_end_matches(".rs");
            println!("MODEL setting={name} closure={label} files={} union_with_judges={} not_in_judges={} {}",
                code.files.len(), union.len(), extra.len(), watched.join(" "));
            if label == "no_stop" {
                let list: String = extra.iter().map(|file| format!("{file}\n")).collect();
                std::fs::write(out_dir.join(format!("extra-{name}.txt")), list).expect("写清单");
            }
        }
    }
    if !mutant_red { mismatched.push("mutant_unstopped_not_red".to_string()); }
    println!("MODEL_SELFCHECK compared={} mismatched={} {:?}", required.len() + excluded.len() + 1, mismatched.len(), mismatched);
    if !mismatched.is_empty() { std::process::exit(1); }
}
