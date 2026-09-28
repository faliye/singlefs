//! E164（崩溃放量新设计的小范围功能测试）：三轮设计之后崩溃放量新设计的每一件改法，在入库装置上、小范围的域里，
//! 是不是把它对应的「红的世界」从非 0 变成 0，且不引入新的漏或假红。
//!
// admission: always 第一段的模型与编译每次重算；第二段读主仓 crates/ 与 git 历史的现状，产物每行带开跑时的 HEAD 与 crates/ 原文摘要
// run-condition: command cargo rustc git python3
//!
//! 跑前登记：`research/prompts/e164-preregistration.md`。问题单：`research/prompts/m3-function-test-questions.md`。
//! 装置不依赖、不链接 `crates/` 的任何包：新设计的每一件在这里独立写一份（`.claude/rules/implementation-first.md` 第 4 条）；
//! 观测 `crates/` 只经仓副本上的子进程（白名单在 `common` 模块的 `assert_subprocess_is_allowed`）。
//!
//! 子命令：`anchors`（第七节钉绝对值的断言）、`models`（F10、F11、F12，不读主仓现状）、
//! `repository`（W3、W5、F13，读主仓现状）、`variants <草稿目录>`（第二段）。


use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 这一趟打出的 E7RESULT 行数（replay.sh 的完整性闸：末行 `name=done emitted=N`，N 连 config 行与 done 行自己）。
static EMITTED_RESULT_LINES: AtomicUsize = AtomicUsize::new(0);

macro_rules! result_line {
    ($($argument:tt)*) => {{
        crate::EMITTED_RESULT_LINES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        println!($($argument)*);
    }};
}

use common::{run_allowed, sha256_hexadecimal, strings};
use pipeline::{claim_forms, explore, geometries, pipeline_forms, Geometry, ModelForm, ModelReport};
use write_tables::{
    append_with_barrier_merging, closed_form_state_count, segment_write_counts, write_table_hash_of_bytes,
    write_table_hash_of_final_sectors, write_table_hash_with_barriers, OperationKind, RecordedOperation,
};

const F12_WEAK_DESIGN: &str = include_str!("e164_crash_amplification_design_function_test/f12_design_weak.rs");
const F12_STRICT_DESIGN: &str = include_str!("e164_crash_amplification_design_function_test/f12_design_strict.rs");
const F12_WRITINGS: &str = include_str!("e164_crash_amplification_design_function_test/f12_writings.rs");

/// 仓根：`E164_REPOSITORY_ROOT` 设了就用它（mutate.sh 只拷 research/，副本里没有 .git 与 crates/），否则取装置往上两层。
pub fn repository_root() -> PathBuf {
    if let Ok(root) = std::env::var("E164_REPOSITORY_ROOT") {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("仓根在 research/e7-index-bench 往上两层")
}

pub fn scratch_directory() -> PathBuf {
    let directory = std::env::var("E164_SCRATCH_DIRECTORY").map(PathBuf::from).unwrap_or_else(|_| {
        std::env::temp_dir().join("e164-crash-amplification-design-function-test")
    });
    std::fs::create_dir_all(&directory).expect("建得了草稿目录");
    directory
}

// ============================== 第七节：钉绝对值的断言 ==============================

fn byte_write(device: u32, kind: OperationKind) -> RecordedOperation {
    RecordedOperation::write(device, kind, 0, &[1u8; 512])
}

fn stream_of(steps: &[(char, u32)]) -> Vec<RecordedOperation> {
    let mut stream = Vec::new();
    for (letter, device) in steps {
        let operation = match letter {
            'W' => byte_write(*device, OperationKind::Write),
            'F' => byte_write(*device, OperationKind::WriteForceUnitAccess),
            _ => RecordedOperation::barrier(*device),
        };
        append_with_barrier_merging(&mut stream, operation);
    }
    stream
}

/// 7.1 与 7.2 的每一格：（名字，量到的，应为）。
pub fn anchor_rows() -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    let mut push = |name: &str, measured: String, expected: &str| rows.push((name.to_string(), measured, expected.to_string()));
    let one_disk_two_segments = stream_of(&[('W', 0), ('W', 0), ('B', 0), ('W', 0), ('B', 0)]);
    push("clause_w_w_b_w_b_states", closed_form_state_count(&segment_write_counts(&one_disk_two_segments)).to_string(), "5");
    let merged = stream_of(&[('W', 0), ('B', 0), ('B', 0), ('W', 0), ('B', 0)]);
    push("clause_w_b_b_w_b_states", closed_form_state_count(&segment_write_counts(&merged)).to_string(), "3");
    let barriers = merged.iter().filter(|operation| operation.kind == OperationKind::Barrier).count();
    push("clause_w_b_b_w_b_barriers_recorded", barriers.to_string(), "2");
    let two_disks = stream_of(&[('W', 0), ('W', 1), ('B', 0), ('W', 0), ('B', 1), ('B', 0)]);
    push("clause_two_disks_states", closed_form_state_count(&segment_write_counts(&two_disks)).to_string(), "8");
    let fua = stream_of(&[('W', 0), ('F', 0)]);
    push("clause_w_fua_states", closed_form_state_count(&segment_write_counts(&fua)).to_string(), "4");
    // 修订（跑前登记第十二节）：M3 在 `W FUA` 上取样点不敏感，补 FUA 后面还有写的一格。
    let fua_then_write = stream_of(&[('W', 0), ('F', 0), ('W', 0), ('B', 0)]);
    push("revision_w_fua_w_b_states", closed_form_state_count(&segment_write_counts(&fua_then_write)).to_string(), "5");
    push(
        "clause_identity_text",
        repository_scans::identity_text("crates/singlefs-harness/tests/e164_probe.rs", "records_h_long", 3, "ab"),
        "crates/singlefs-harness/tests/e164_probe.rs::records_h_long::3::ab",
    );
    let anchor_digest = "e7587d8d4f76653ed394fefe683bb459d4976354ee1366bf987fcd16750a418c";
    push("lexical_const", lexical::lexical_digest("const A: u8 = 1;"), anchor_digest);
    push("lexical_line_comment", lexical::lexical_digest("const A: u8 = 1; // x"), anchor_digest);
    push("lexical_doc_comment", lexical::lexical_digest("/// doc\nconst A: u8 = 1;"), anchor_digest);
    push("lexical_moved_line", lexical::lexical_digest("\nconst A: u8 = 1;"), anchor_digest);
    push("lexical_literal_changed", lexical::lexical_digest("const A: u8 = 2;"), "1869717063fbdfadfa523427dd8bd33001d9070e3bab056a405409a4adb11eae");
    push("lexical_comment_only", lexical::lexical_digest("// only\n/* block /* nested */ */\n"), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    let file_text = lexical::token_text(&lexical::lexical_tokens("fn f() {}"));
    push("lexical_combined", lexical::combined_digest(&[("x/a.rs".to_string(), file_text)]), "7bddf7d478debd3b5d05e81ffb4cafc154c9f273543173f00149c77bcf0821df");
    push("sha256_512_ones", sha256_hexadecimal(&[1u8; 512]), "6caf38d537984e261527b8caef5f990fb91415a1db917198821a79ed28997973");
    let plain = stream_of(&[('W', 0), ('B', 0)]);
    push("write_table_with_barriers", write_table_hash_with_barriers(&plain), "85f76441b8e6cf40f2e7676d1a26fe28995bf3ae12e44d46f1c511099f628eeb");
    // 修订（跑前登记第十二节）：M1 在只有段 0 的锚点上取样点不敏感，补第二次写落在段 1 的一格（命令另算）。
    let two_segments = stream_of(&[('W', 0), ('B', 0), ('W', 0), ('B', 0)]);
    push("revision_write_table_with_barriers_second_segment", write_table_hash_with_barriers(&two_segments), "8f1a4e96284976c59a4f3ee4d25d03f1170199fb98dbf94f5153472f4c48a7a5");
    let as_fua = stream_of(&[('F', 0), ('B', 0)]);
    push("write_table_with_barriers_fua", write_table_hash_with_barriers(&as_fua), "b0016e7f021a373bae734cf73650c7468ada8c75d1342f6856f75fb90f229123");
    push("write_table_of_bytes", write_table_hash_of_bytes(&plain), "036682e3e5240c354c145dba7b2df3856731feb6202eea1fb0fcb3244e9450ae");
    push("write_table_of_final_sectors", write_table_hash_of_final_sectors(&plain), "53b3145f50e52c782c6481d430fdc5a316ea03c2be088a6dceb8d46e0e81e088");
    for (segments, expected) in [(vec![2usize, 1], "5"), (vec![1, 1], "3"), (vec![2], "4"), (vec![3], "8")] {
        push(&format!("closed_form_{}", write_tables::segment_sizes_text(&segments)), closed_form_state_count(&segments).to_string(), expected);
    }
    rows
}

fn command_anchors() -> bool {
    let mut all_hold = true;
    for (name, measured, expected) in anchor_rows() {
        let holds = measured == expected;
        all_hold &= holds;
        result_line!("E7RESULT name=anchor anchor={name} measured={measured} expected={expected} holds={holds}");
    }
    let one_block = pipeline::explore(
        &ModelForm { allow_kills: false, allow_judging_change: false, ..pipeline::STRONGEST_FORM },
        &Geometry { blocks: 1, cards: 1, backlog_cap: 1, batch: 1 },
    );
    let errors = one_block.missing_result + one_block.wrong_input + one_block.double_tally + one_block.stale_after_change + one_block.stuck;
    let derived_holds = one_block.terminal_states == 1 && errors == 0;
    all_hold &= derived_holds;
    result_line!("E7RESULT name=anchor anchor=derived_one_block_strongest terminal_states={} errors={errors} holds={derived_holds}", one_block.terminal_states);
    result_line!("E7RESULT name=anchors_summary all_hold={all_hold}");
    all_hold
}

// ============================== 第一段：F10、F11（模型）与 F12（编译） ==============================

fn report_fields(report: &ModelReport) -> String {
    let set_text = |values: &std::collections::BTreeSet<usize>| values.iter().map(usize::to_string).collect::<Vec<String>>().join(",");
    format!(
        "reachable_states={} terminal_states={} missing_result={} wrong_input={} double_tally={} stale_after_change={} stuck={} dead_block_terminals={} kill_transitions={} judging_change_transitions={} card_drop_transitions={} takeover_after_lease_transitions={} backlog_peak={} backlog_positive_states={} terminal_backlog_values={} held_peak={} held_positive_states={} terminal_held_values={} stopped_over_limit={}",
        report.reachable_states, report.terminal_states, report.missing_result, report.wrong_input, report.double_tally,
        report.stale_after_change, report.stuck, report.dead_block_terminals, report.kill_transitions,
        report.judging_change_transitions, report.card_drop_transitions, report.takeover_after_lease_transitions,
        report.backlog_peak, report.backlog_positive_states, set_text(&report.terminal_backlog_values), report.held_peak,
        report.held_positive_states, set_text(&report.terminal_held_values), report.stopped_over_limit
    )
}

fn geometry_text(geometry: &Geometry) -> String {
    format!("blocks={} cards={} backlog_cap={} batch={}", geometry.blocks, geometry.cards, geometry.backlog_cap, geometry.batch)
}

/// 弱形各自那一种错误（跑前登记 5.8「F10 五种弱形」）。
fn own_error_of_weak_form(name: &str, report: &ModelReport) -> u64 {
    match name {
        "record_flag_before_data" => report.wrong_input,
        "checked_flag_before_result" => report.missing_result,
        "result_then_flag_with_appended_tally" => report.double_tally,
        "checked_flag_is_a_boolean" => report.stale_after_change,
        "batch_larger_than_backlog_cap_without_flush" => report.stuck,
        _ => panic!("认不出的弱形 {name}"),
    }
}

fn command_pipeline_models() {
    let mut strongest_errors = 0u64;
    let mut strongest_positive = (0u64, 0u64, 0u64);
    let mut strongest_backlog_over_cap_points = 0usize;
    for (name, form) in pipeline_forms() {
        let mut own_error_points = 0usize;
        for geometry in geometries() {
            let report = explore(&form, &geometry);
            result_line!("E7RESULT name=q10_model form={name} {} {}", geometry_text(&geometry), report_fields(&report));
            let errors = report.missing_result + report.wrong_input + report.double_tally + report.stale_after_change + report.stuck;
            if name == "strongest" {
                strongest_errors += errors;
                strongest_positive.0 += report.kill_transitions;
                strongest_positive.1 += report.judging_change_transitions;
                strongest_positive.2 += report.terminal_states;
                strongest_backlog_over_cap_points += usize::from(report.backlog_peak > geometry.backlog_cap);
            } else if own_error_of_weak_form(name, &report) > 0 {
                own_error_points += 1;
            }
        }
        if name != "strongest" {
            result_line!("E7RESULT name=q10_weak_form_summary form={name} points_with_its_own_error={own_error_points} of={} reproduced={}", geometries().len(), own_error_points > 0);
        }
    }
    let without_change = ModelForm { allow_judging_change: false, ..pipeline::STRONGEST_FORM };
    let mut points_over_cap_without_change = 0usize;
    for geometry in geometries() {
        let report = explore(&without_change, &geometry);
        result_line!("E7RESULT name=q10_traj_supplement form=strongest_without_judging_change {} backlog_peak={} backlog_positive_states={}", geometry_text(&geometry), report.backlog_peak, report.backlog_positive_states);
        points_over_cap_without_change += usize::from(report.backlog_peak > geometry.backlog_cap);
    }
    result_line!("E7RESULT name=q10_traj_supplement_summary points_with_backlog_peak_over_cap_without_judging_change={points_over_cap_without_change}");
    result_line!(
        "E7RESULT name=q10_strongest_summary errors_over_all_points={strongest_errors} kill_transitions={} judging_change_transitions={} terminal_states={} points_with_backlog_peak_over_cap={strongest_backlog_over_cap_points} holds={}",
        strongest_positive.0, strongest_positive.1, strongest_positive.2, strongest_errors == 0
    );
    for (name, form) in claim_forms() {
        let mut dead = 0u64;
        let mut other_four = 0u64;
        let mut drops = 0u64;
        let mut takeovers = 0u64;
        for geometry in geometries() {
            let report = explore(&form, &geometry);
            result_line!("E7RESULT name=q11_model form={name} {} {}", geometry_text(&geometry), report_fields(&report));
            dead += report.dead_block_terminals;
            other_four += report.missing_result + report.wrong_input + report.double_tally + report.stale_after_change;
            drops += report.card_drop_transitions;
            takeovers += report.takeover_after_lease_transitions;
        }
        result_line!("E7RESULT name=q11_summary form={name} dead_block_terminals={dead} other_four_errors={other_four} card_drop_transitions={drops} takeover_after_lease_transitions={takeovers}");
    }
}

/// F12 的五种写法，按「// writing N:」切开。
fn f12_writings() -> Vec<(u32, String)> {
    let mut writings: Vec<(u32, String)> = Vec::new();
    for line in F12_WRITINGS.lines() {
        if let Some(rest) = line.strip_prefix("// writing ") {
            let number: u32 = rest.split(':').next().and_then(|digits| digits.trim().parse().ok()).expect("写法编号");
            writings.push((number, String::new()));
        } else if let Some((_, body)) = writings.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    writings
}

fn command_label_compilation() {
    let directory = scratch_directory().join("f12");
    std::fs::create_dir_all(&directory).expect("建得了 F12 目录");
    let mut weak_bypasses = 0usize;
    let mut strict_bypasses = 0usize;
    let mut strict_legal_compiles = false;
    for (form, design) in [("weak", F12_WEAK_DESIGN), ("strict", F12_STRICT_DESIGN)] {
        for (number, body) in f12_writings() {
            let source = format!("#![forbid(unsafe_code)]\n{design}\n{body}");
            let source_path = directory.join(format!("writing_{number}_{form}.rs"));
            std::fs::write(&source_path, &source).expect("写得了用例");
            let arguments = strings(&["--edition", "2021", "--crate-type", "lib", "--emit=metadata", "-o"])
                .into_iter()
                .chain([directory.join(format!("writing_{number}_{form}.rmeta")).display().to_string(), source_path.display().to_string()])
                .collect::<Vec<String>>();
            let output = run_allowed("rustc", &arguments, &directory, &[]);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let first_error = stderr
                .lines()
                .find_map(|line| line.strip_prefix("error[").and_then(|rest| rest.split(']').next()))
                .unwrap_or("none");
            let compiles = output.status.success();
            result_line!("E7RESULT name=q12_compile form={form} writing={number} compiles={compiles} first_error={first_error}");
            match (form, number, compiles) {
                ("weak", 1..=4, true) => weak_bypasses += 1,
                ("strict", 1..=4, true) => strict_bypasses += 1,
                ("strict", 5, true) => strict_legal_compiles = true,
                _ => {}
            }
        }
    }
    result_line!("E7RESULT name=q12_summary weak_writings_one_to_four_compiling={weak_bypasses} strict_writings_one_to_four_compiling={strict_bypasses} strict_writing_five_compiles={strict_legal_compiles}");
}

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().filter(|argument| argument != "--force").collect();
    result_line!("E7RESULT name=config experiment=E164 command={}", arguments.get(1).map_or("none", String::as_str));
    match arguments.get(1).map(String::as_str) {
        Some("anchors") => {
            if !command_anchors() {
                std::process::exit(1);
            }
        }
        Some("models") => {
            if !command_anchors() {
                std::process::exit(1);
            }
            command_pipeline_models();
            command_label_compilation();
        }
        Some("repository") => repository_scans::command_repository(),
        Some("variants") => variants::command_variants(Path::new(arguments.get(2).map_or("", String::as_str))),
        other => {
            eprintln!("  ✗ 认不出的子命令 {other:?}");
            eprintln!("  → 怎么办：用 anchors / models / repository / variants <草稿目录> 之一");
            std::process::exit(2);
        }
    }
    println!("E7RESULT name=done emitted={}", EMITTED_RESULT_LINES.load(Ordering::Relaxed) + 1);
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("research/e7-index-bench/src/bin/e164_crash_amplification_design_function_test.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn frozen_file_name_scan_covers_every_commit() {
        let root = repository_root();
        let output = common::run_allowed("git", &strings(&["rev-list", "--all", "--count"]), &root, &[]);
        let independent: usize = String::from_utf8_lossy(&output.stdout).trim().parse().expect("git 打出一个数");
        assert_eq!(repository_scans_w3::all_commits(&root).len(), independent, "W3 扫到的提交数等于 git rev-list --all --count");
    }

    #[test]
    fn a_node_reruns_when_its_own_target_file_changes() {
        let closure = std::collections::BTreeSet::from(["singlefs-format".to_string()]);
        let untouched = std::collections::BTreeSet::new();
        let changed = vec!["crates/singlefs-harness/tests/a.rs".to_string()];
        assert!(repository_scans_w5::node_reruns(&closure, &untouched, Some("crates/singlefs-harness/tests/a.rs"), &changed));
        assert!(!repository_scans_w5::node_reruns(&closure, &untouched, Some("crates/singlefs-harness/tests/b.rs"), &changed));
        let touched = std::collections::BTreeSet::from(["singlefs-format".to_string()]);
        assert!(repository_scans_w5::node_reruns(&closure, &touched, None, &[]));
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
}

// ============================== 模块（内联：mutate.sh 只改这一份源文件） ==============================

mod arms {
    //! E164 第二段：各变体上各臂的复用判定与漏 / 多余计数（跑前登记 5.5、第六节「第二段」）。

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    use crate::node_code::{
        closure_source_files, digest_by_crate_closure, digest_by_function, digest_by_walked_files, own_source_files, package_source_files,
        packages_lexical_digest, rustc_version, target_files,
    };
    use crate::repository_scans::{dependency_graph, ProductSuffix};
    use crate::variants::{cargo_test, parse_history, prepare_copy, replacement_of, stop, variant_directory, variant_sources, ProbeHistory, PROBE_TEST_FILE};
    use crate::write_tables::{
        closed_form_state_count, overlay_sectors, sector_table_hash, segment_sizes_text, segment_write_counts, write_table_hash_of_bytes,
        write_table_hash_of_final_sectors, write_table_hash_with_barriers, RecordedOperation,
    };

    const V5_TEST: (&str, &str, &str) = ("singlefs-harness", "crates/singlefs-harness/tests/new_pool_file_creation_publish.rs", "root_slots_system_configurations_and_journal_ring_hold_the_published_state");
    const V11_TEST: &str = "pointer_record_and_entry_width_literals";
    const F6_TEST: &str = "the_model_module_uses_only_the_standard_library_and_the_format_constants";

    /// 输出比对（做）去掉的位置字段：`location=<文件>:<行>[:<列>]` 只留文件。只给变异 M12 改。
    fn strip_positions(output: &str) -> String {
        output
            .split(' ')
            .map(|field| match field.strip_prefix("location=") {
                Some(location) => format!("location={}", location.split(':').next().unwrap_or(location)),
                None => field.to_string(),
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// 一个变体上量到的全部东西。
    struct VariantObservation {
        histories: BTreeMap<String, Result<ProbeHistory, String>>,
        panic_line: String,
        pid_lines: Vec<String>,
        recording_code: String,
        judging_not_doing: String,
        judging_doing: String,
        v5_node: [String; 3],
        v11_node: [String; 3],
        harness_lib_not_doing: String,
        harness_lib_doing: String,
        probe_node: String,
        named_exit: BTreeMap<&'static str, i32>,
        harness_lib_outcomes: BTreeMap<String, String>,
    }

    fn three_digests(copy: &Path, files: &[String], rustc_text: &str) -> [String; 3] {
        [digest_by_function(copy, files), digest_by_walked_files(copy, files), digest_by_crate_closure(copy, files, &BTreeSet::new(), rustc_text)]
    }

    fn test_outcomes(stdout: &str) -> BTreeMap<String, String> {
        stdout
            .lines()
            .filter_map(|line| line.strip_prefix("test "))
            .filter_map(|rest| rest.rsplit_once(" ... ").map(|(name, outcome)| (name.to_string(), outcome.trim().to_string())))
            .collect()
    }

    fn observe(root: &Path, scratch: &Path, name: &str, source: Option<crate::variants::VariantSource>) -> VariantObservation {
        let copy = variant_directory(scratch, name);
        let replacement = source.map(|source| replacement_of(root, source));
        prepare_copy(root, &copy, replacement.as_ref());
        let graph = dependency_graph(&copy, &[]).unwrap_or_else(|message| stop(&format!("副本上 cargo metadata --offline 起不来（S-offline）：{message}")));
        let rustc_text = rustc_version(&copy);
        let harness_files = closure_source_files(&copy, &graph, "singlefs-harness");
        let mut v5_files = harness_files.clone();
        v5_files.extend(target_files(&copy, V5_TEST.1));
        let format_files = closure_source_files(&copy, &graph, "singlefs-format");
        let own_source = own_source_files(&copy, &package_source_files(&copy, "crates/singlefs-harness"));
        let mut probe_files = harness_files.clone();
        probe_files.extend(target_files(&copy, PROBE_TEST_FILE));
        let probe_run = cargo_test(&copy, &["-p", "singlefs-harness", "--test", "e164_probe", "--", "--nocapture", "--test-threads=1"]);
        let mut histories = BTreeMap::new();
        for probe in ["h_short", "h_long"] {
            let parsed = if probe_run.exit_code == 0 { parse_history(&probe_run.stdout, probe) } else { Err(format!("探针退出码 {}", probe_run.exit_code)) };
            histories.insert(probe.to_string(), parsed);
        }
        let result_line = |stdout: &str, probe: &str| {
            stdout.lines().find_map(|line| line.find(&format!("E7RESULT probe={probe} ")).map(|start| line[start..].to_string())).unwrap_or_else(|| "missing".to_string())
        };
        let mut pid_lines = vec![result_line(&probe_run.stdout, "pid")];
        if name == "V0" {
            let second = cargo_test(&copy, &["-p", "singlefs-harness", "--test", "e164_probe", "--", "--nocapture", "--test-threads=1", "pid_probe"]);
            pid_lines.push(result_line(&second.stdout, "pid"));
        }
        let mut named_exit = BTreeMap::new();
        if matches!(name, "V0" | "V5") {
            named_exit.insert("v5_test", cargo_test(&copy, &["-p", V5_TEST.0, "--test", "new_pool_file_creation_publish", "--", V5_TEST.2]).exit_code);
        }
        if matches!(name, "V0" | "V11") {
            named_exit.insert("v11_test", cargo_test(&copy, &["-p", "singlefs-format", "--lib", "--", V11_TEST]).exit_code);
        }
        let model_source = std::fs::read_to_string(copy.join("crates/singlefs-harness/src/model_comparison.rs")).unwrap_or_default();
        if !model_source.contains(F6_TEST) {
            stop(&format!("F6 那一条测试 {F6_TEST} 在副本里找不到（S-probe）"));
        }
        let mut harness_lib_outcomes = BTreeMap::new();
        if matches!(name, "V0" | "V4" | "V8") {
            let run = cargo_test(&copy, &["-p", "singlefs-harness", "--lib"]);
            harness_lib_outcomes = test_outcomes(&run.stdout);
            named_exit.insert("harness_lib", run.exit_code);
            named_exit.insert("f6_test", cargo_test(&copy, &["-p", "singlefs-harness", "--lib", "--", F6_TEST]).exit_code);
        }
        VariantObservation {
            histories,
            panic_line: result_line(&probe_run.stdout, "panic"),
            pid_lines,
            recording_code: digest_by_crate_closure(&copy, &closure_source_files(&copy, &graph, "singlefs-core"), &BTreeSet::new(), &rustc_text),
            judging_not_doing: packages_lexical_digest(&copy, &["crates/singlefs-checker", "crates/singlefs-format"]),
            judging_doing: packages_lexical_digest(&copy, &["crates/singlefs-checker", "crates/singlefs-format", "crates/singlefs-harness", "crates/singlefs-checker-tier"]),
            v5_node: three_digests(&copy, &v5_files, &rustc_text),
            v11_node: three_digests(&copy, &format_files, &rustc_text),
            harness_lib_not_doing: digest_by_crate_closure(&copy, &harness_files, &BTreeSet::new(), &rustc_text),
            harness_lib_doing: digest_by_crate_closure(&copy, &harness_files, &own_source, &rustc_text),
            probe_node: digest_by_crate_closure(&copy, &probe_files, &BTreeSet::new(), &rustc_text),
            named_exit,
            harness_lib_outcomes,
        }
    }

    /// 一条历史上每一步（base 之后）的属性。
    struct StepAttributes {
        label: String,
        final_sectors: String,
        bytes: String,
        with_barriers: String,
        writes_without_barriers: String,
        parent_output: String,
        closed_form: u128,
    }

    fn step_attributes(history: &ProbeHistory) -> (String, Vec<StepAttributes>) {
        let mkfs = sector_table_hash(&history.mkfs_sectors);
        let mut image = history.mkfs_sectors.clone();
        let mut steps = Vec::new();
        for (label, start, end) in history.steps.iter().filter(|(label, _, _)| label != "base") {
            let operations: &[RecordedOperation] = &history.operations[*start..*end];
            let parent_output = sector_table_hash(&image);
            let writes: Vec<RecordedOperation> = operations.iter().filter(|operation| operation.kind != crate::write_tables::OperationKind::Barrier).cloned().collect();
            let kinds_and_bytes: String = writes.iter().map(|operation| format!("{} {} ", operation.kind.serialized_name(), operation.content_sha256)).collect();
            steps.push(StepAttributes {
                label: label.clone(),
                final_sectors: write_table_hash_of_final_sectors(operations),
                bytes: write_table_hash_of_bytes(operations),
                with_barriers: write_table_hash_with_barriers(operations),
                writes_without_barriers: format!("{}|{}", write_table_hash_of_bytes(&writes), crate::common::sha256_hexadecimal(kinds_and_bytes.as_bytes())),
                parent_output,
                closed_form: closed_form_state_count(&segment_write_counts(operations)),
            });
            overlay_sectors(&mut image, operations);
        }
        (mkfs, steps)
    }

    /// S-seg：装置独立切出的每一步段序列与探针打出的 harness 切段逐步相同。
    fn segment_crosscheck(history: &ProbeHistory) -> Vec<String> {
        let mut mismatches = Vec::new();
        for (label, start, end) in &history.steps {
            let counts = segment_write_counts(&history.operations[*start..*end]);
            let (harness_sizes, harness_closed_form) = history.harness_segments.get(label).cloned().unwrap_or_default();
            let closed_form_matches = counts.iter().any(|count| *count >= 64) || closed_form_state_count(&counts).to_string() == harness_closed_form;
            if segment_sizes_text(&counts) != harness_sizes || !closed_form_matches {
                mismatches.push(format!("{label}:{}!={harness_sizes}", segment_sizes_text(&counts)));
            }
        }
        mismatches
    }

    /// 一条臂在一个变体、一条历史上的逐步判定：（步，复用没有，真值「输入变了」没有，这一步的状态数）。
    type StepDecisions = Vec<(String, bool, bool, u128)>;

    #[derive(Clone, Copy)]
    enum RecordingArm {
        FinalSectorsChained,
        BytesChained,
        WithBarriersChained,
        WithBarriersChainedAndRecordingCode,
        WithBarriersAndParentIdentity,
        WithBarriersAndParentOutput,
        FullDesign,
    }

    impl RecordingArm {
        fn name(self) -> &'static str {
            match self {
                RecordingArm::FinalSectorsChained => "f1_final_sectors",
                RecordingArm::BytesChained => "f1_bytes",
                RecordingArm::WithBarriersChained => "f1_with_barriers",
                RecordingArm::WithBarriersChainedAndRecordingCode => "f2_with_recording_code",
                RecordingArm::WithBarriersAndParentIdentity => "f7_parent_identity",
                RecordingArm::WithBarriersAndParentOutput => "f7_parent_output",
                RecordingArm::FullDesign => "full_design",
            }
        }
    }

    fn decide(arm: RecordingArm, base: &(String, Vec<StepAttributes>), variant: &(String, Vec<StepAttributes>), recording_code_same: bool) -> StepDecisions {
        let mut decisions = Vec::new();
        let mut chain_holds = base.0 == variant.0;
        for (index, variant_step) in variant.1.iter().enumerate() {
            let base_step = base.1.get(index);
            let same = |pick: fn(&StepAttributes) -> &String| base_step.is_some_and(|step| pick(step) == pick(variant_step));
            let with_barriers_same = same(|step| &step.with_barriers);
            let parent_same = same(|step| &step.parent_output);
            let this_step_same = match arm {
                RecordingArm::FinalSectorsChained => same(|step| &step.final_sectors),
                RecordingArm::BytesChained => same(|step| &step.bytes),
                RecordingArm::WithBarriersChained
                | RecordingArm::WithBarriersChainedAndRecordingCode
                | RecordingArm::WithBarriersAndParentIdentity
                | RecordingArm::WithBarriersAndParentOutput
                | RecordingArm::FullDesign => with_barriers_same,
            };
            chain_holds &= this_step_same;
            let reused = match arm {
                RecordingArm::FinalSectorsChained | RecordingArm::BytesChained | RecordingArm::WithBarriersChained => chain_holds,
                RecordingArm::WithBarriersChainedAndRecordingCode => chain_holds && recording_code_same,
                RecordingArm::WithBarriersAndParentIdentity => with_barriers_same,
                RecordingArm::WithBarriersAndParentOutput => with_barriers_same && parent_same,
                RecordingArm::FullDesign => chain_holds && recording_code_same && parent_same,
            };
            let input_changed = !(with_barriers_same && parent_same);
            decisions.push((variant_step.label.clone(), reused, input_changed, variant_step.closed_form));
        }
        decisions
    }

    struct Tally {
        missed_steps: usize,
        missed_states: u128,
        rerecorded_steps: usize,
    }

    fn tally(decisions: &StepDecisions) -> Tally {
        Tally {
            missed_steps: decisions.iter().filter(|(_, reused, changed, _)| *reused && *changed).count(),
            missed_states: decisions.iter().filter(|(_, reused, changed, _)| *reused && *changed).map(|(_, _, _, states)| *states).sum(),
            rerecorded_steps: decisions.iter().filter(|(_, reused, _, _)| !reused).count(),
        }
    }

    pub fn run_segment_two(root: &Path, scratch: &Path) {
        let suffix = ProductSuffix::current(root);
        let mut observations: BTreeMap<String, VariantObservation> = BTreeMap::new();
        for (name, source) in variant_sources() {
            let observation = observe(root, scratch, name, source);
            for (probe, history) in &observation.histories {
                match history {
                    Ok(history) => {
                        let mismatches = segment_crosscheck(history);
                        suffix.emit(&format!("E7RESULT name=probe variant={name} history={probe} operations={} steps={} segment_mismatches={}", history.operations.len(), history.steps.len(), if mismatches.is_empty() { "none".to_string() } else { mismatches.join(",") }));
                        if !mismatches.is_empty() {
                            stop(&format!("{name} {probe} 装置切段与 harness 切段不同（S-seg）：{}", mismatches.join(",")));
                        }
                    }
                    Err(reason) => suffix.emit(&format!("E7RESULT name=probe variant={name} history={probe} failed={}", reason.replace(' ', "_"))),
                }
            }
            observations.insert(name.to_string(), observation);
        }
        let base = &observations["V0"];
        if !base.panic_line.contains("name=panic ") {
            stop("DeviceFreeMap::new 在小盘上没 panic（S-probe）");
        }
        report_recording_arms(&observations, &suffix);
        report_code_attributes(root, &observations, &suffix);
    }

    fn attributes_of(observation: &VariantObservation, probe: &str) -> Option<(String, Vec<StepAttributes>)> {
        observation.histories.get(probe).and_then(|history| history.as_ref().ok()).map(step_attributes)
    }

    fn report_recording_arms(observations: &BTreeMap<String, VariantObservation>, suffix: &ProductSuffix) {
        let base = &observations["V0"];
        let v0b_identical = ["h_short", "h_long"].iter().all(|probe| {
            let left = attributes_of(base, probe).map(|(mkfs, steps)| (mkfs, steps.iter().map(|step| step.with_barriers.clone()).collect::<Vec<_>>()));
            let right = attributes_of(&observations["V0b"], probe).map(|(mkfs, steps)| (mkfs, steps.iter().map(|step| step.with_barriers.clone()).collect::<Vec<_>>()));
            left.is_some() && left == right
        });
        suffix.emit(&format!("E7RESULT name=determinism v0_and_v0b_write_tables_identical={v0b_identical}"));
        if !v0b_identical {
            stop("V0 与 V0b 的写表不逐字相同（V-det：录制不确定，第二段整段作废）");
        }
        let arms = [
            RecordingArm::FinalSectorsChained,
            RecordingArm::BytesChained,
            RecordingArm::WithBarriersChained,
            RecordingArm::WithBarriersChainedAndRecordingCode,
            RecordingArm::WithBarriersAndParentIdentity,
            RecordingArm::WithBarriersAndParentOutput,
            RecordingArm::FullDesign,
        ];
        for (variant_name, observation) in observations {
            for probe in ["h_short", "h_long"] {
                let (Some(base_attributes), Some(variant_attributes)) = (attributes_of(base, probe), attributes_of(observation, probe)) else {
                    suffix.emit(&format!("E7RESULT name=recording_arm variant={variant_name} history={probe} probe_unavailable=true"));
                    continue;
                };
                let recording_code_same = observation.recording_code == base.recording_code;
                for arm in arms {
                    let decisions = decide(arm, &base_attributes, &variant_attributes, recording_code_same);
                    let counts = tally(&decisions);
                    let trace: Vec<String> = decisions.iter().map(|(label, reused, changed, _)| format!("{label}:{}{}", if *reused { "reuse" } else { "rerecord" }, if *changed { "+changed" } else { "" })).collect();
                    suffix.emit(&format!(
                        "E7RESULT name=recording_arm arm={} variant={variant_name} history={probe} missed_steps={} missed_states={} rerecorded_steps={} recording_code_same={recording_code_same} trace={}",
                        arm.name(), counts.missed_steps, counts.missed_states, counts.rerecorded_steps, trace.join(",")
                    ));
                }
                let only_segments_differ = base_attributes
                    .1
                    .iter()
                    .zip(&variant_attributes.1)
                    .filter(|(left, right)| left.with_barriers != right.with_barriers && left.writes_without_barriers == right.writes_without_barriers)
                    .count();
                let steps_with_barriers_differ = base_attributes.1.iter().zip(&variant_attributes.1).filter(|(left, right)| left.with_barriers != right.with_barriers).count();
                suffix.emit(&format!(
                    "E7RESULT name=write_table_difference variant={variant_name} history={probe} steps_with_barrier_table_changed={steps_with_barriers_differ} steps_where_only_barriers_or_segments_differ={only_segments_differ} mkfs_baseline_same={}",
                    base_attributes.0 == variant_attributes.0
                ));
            }
        }
    }

    /// 全仓「读自己原文」的目标：每个工作区包的库目标与 `tests/*.rs` 各一个。
    fn own_source_targets(root: &Path) -> Vec<(String, BTreeSet<String>)> {
        let mut targets = Vec::new();
        let Ok(entries) = std::fs::read_dir(root.join("crates")) else { return targets };
        let mut packages: Vec<String> = entries.flatten().filter(|entry| entry.path().join("Cargo.toml").exists()).map(|entry| format!("crates/{}", entry.file_name().to_string_lossy())).collect();
        packages.sort();
        for package in packages {
            let library = own_source_files(root, &package_source_files(root, &package));
            if !library.is_empty() {
                targets.push((format!("{package}:--lib"), library));
            }
            let mut tests: Vec<String> = crate::repository_scans::files_under(root, &format!("{package}/tests")).into_iter().filter(|path| path.ends_with(".rs") && path.matches('/').count() == 3).collect();
            tests.sort();
            for test in tests {
                let read = own_source_files(root, &target_files(root, &test));
                if !read.is_empty() {
                    targets.push((test, read));
                }
            }
        }
        targets
    }

    fn changed(left: &str, right: &str) -> bool {
        left != right
    }

    fn report_code_attributes(root: &Path, observations: &BTreeMap<String, VariantObservation>, suffix: &ProductSuffix) {
        let base = &observations["V0"];
        for name in ["V4", "V8", "V6", "V0b"] {
            let observation = &observations[name];
            suffix.emit(&format!(
                "E7RESULT name=q3_digest variant={name} not_doing_changed={} doing_changed={}",
                changed(&base.judging_not_doing, &observation.judging_not_doing),
                changed(&base.judging_doing, &observation.judging_doing)
            ));
        }
        let v4 = &observations["V4"];
        let differing_tests: Vec<&String> = base.harness_lib_outcomes.iter().filter(|(test, outcome)| v4.harness_lib_outcomes.get(*test) != Some(outcome)).map(|(test, _)| test).collect();
        suffix.emit(&format!(
            "E7RESULT name=q3_lib tests_in_v0={} tests_in_v4={} differing_outcomes={} differing={}",
            base.harness_lib_outcomes.len(), v4.harness_lib_outcomes.len(), differing_tests.len(),
            if differing_tests.is_empty() { "none".to_string() } else { differing_tests.iter().map(|test| test.as_str()).collect::<Vec<&str>>().join(",") }
        ));
        let methods = ["by_function", "by_walked_files", "by_crate_closure"];
        let mut misses = [0usize; 3];
        for (name, key) in [("V5", "v5_test"), ("V11", "v11_test")] {
            let observation = &observations[name];
            let (base_node, variant_node) = if name == "V5" { (&base.v5_node, &observation.v5_node) } else { (&base.v11_node, &observation.v11_node) };
            let base_exit = base.named_exit.get(key).copied().unwrap_or(-1);
            let variant_exit = observation.named_exit.get(key).copied().unwrap_or(-1);
            let flipped = base_exit == 0 && variant_exit != 0;
            suffix.emit(&format!("E7RESULT name=q4_flip variant={name} base_exit={base_exit} variant_exit={variant_exit} green_to_red={flipped}"));
            for (index, method) in methods.iter().enumerate() {
                let digest_changed = changed(&base_node[index], &variant_node[index]);
                misses[index] += usize::from(flipped && !digest_changed);
                suffix.emit(&format!("E7RESULT name=q4_digest variant={name} method={method} digest_changed={digest_changed}"));
            }
        }
        for (index, method) in methods.iter().enumerate() {
            suffix.emit(&format!("E7RESULT name=q4_miss method={method} missed_variants={}", misses[index]));
        }
        for (name, left, right) in [
            ("V6", &base.panic_line, &observations["V6"].panic_line),
            ("V6b", &base.panic_line, &observations["V6b"].panic_line),
            ("V10", &base.pid_lines[0], base.pid_lines.get(1).unwrap_or(&base.pid_lines[0])),
        ] {
            let node_same = name == "V10" || observations[name].probe_node == base.probe_node;
            let not_doing_red = node_same && left != right;
            let doing_red = node_same && strip_positions(left) != strip_positions(right);
            suffix.emit(&format!("E7RESULT name=q5_red variant={name} node_code_same={node_same} not_doing_red={} doing_red={} left={} right={}", u8::from(not_doing_red), u8::from(doing_red), left.replace(' ', "|"), right.replace(' ', "|")));
        }
        let v8 = &observations["V8"];
        let base_exit = base.named_exit.get("f6_test").copied().unwrap_or(-1);
        let v8_exit = v8.named_exit.get("f6_test").copied().unwrap_or(-1);
        let exit_changed = base_exit != v8_exit;
        let not_doing_changed = changed(&base.harness_lib_not_doing, &v8.harness_lib_not_doing);
        let doing_changed = changed(&base.harness_lib_doing, &v8.harness_lib_doing);
        suffix.emit(&format!(
            "E7RESULT name=q6 base_exit={base_exit} v8_exit={v8_exit} exit_changed={exit_changed} not_doing_digest_changed={not_doing_changed} doing_digest_changed={doing_changed} not_doing_miss={} doing_miss={} doing_changed_on_v6={}",
            u8::from(exit_changed && !not_doing_changed), u8::from(exit_changed && !doing_changed), changed(&base.harness_lib_doing, &observations["V6"].harness_lib_doing)
        ));
        let targets = own_source_targets(root);
        for (target, files) in &targets {
            suffix.emit(&format!("E7RESULT name=q6_own_source_target target={target} reads={}", files.iter().cloned().collect::<Vec<String>>().join(",")));
        }
        suffix.emit(&format!("E7RESULT name=q6_own_source_targets count={}", targets.len()));
        for (name, observation) in observations {
            suffix.emit(&format!(
                "E7RESULT name=full_design_code variant={name} judging_changed={} v5_node_changed={} v11_node_changed={} harness_lib_node_changed={} probe_node_changed={} recording_code_changed={}",
                changed(&base.judging_doing, &observation.judging_doing),
                changed(&base.v5_node[2], &observation.v5_node[2]),
                changed(&base.v11_node[2], &observation.v11_node[2]),
                changed(&base.harness_lib_doing, &observation.harness_lib_doing),
                changed(&base.probe_node, &observation.probe_node),
                changed(&base.recording_code, &observation.recording_code)
            ));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::strip_positions;

        fn step(parent_output: &str, with_barriers: &str) -> super::StepAttributes {
            super::StepAttributes {
                label: "0".to_string(),
                final_sectors: "f".to_string(),
                bytes: "b".to_string(),
                with_barriers: with_barriers.to_string(),
                writes_without_barriers: "w".to_string(),
                parent_output: parent_output.to_string(),
                closed_form: 4,
            }
        }

        #[test]
        fn parent_output_arm_rerecords_when_only_the_parent_output_changed() {
            let base = ("m".to_string(), vec![step("p1", "t")]);
            let variant = ("m".to_string(), vec![step("p2", "t")]);
            let doing = super::decide(super::RecordingArm::WithBarriersAndParentOutput, &base, &variant, true);
            let not_doing = super::decide(super::RecordingArm::WithBarriersAndParentIdentity, &base, &variant, true);
            assert_eq!(doing, vec![("0".to_string(), false, true, 4)], "输入取父输出：父输出变了就重录");
            assert_eq!(not_doing, vec![("0".to_string(), true, true, 4)], "输入取父身份：漏重录");
        }

        #[test]
        fn stripping_positions_keeps_the_file_and_the_message() {
            assert_eq!(strip_positions("location=crates/a.rs:371:5 message_hex=41"), "location=crates/a.rs message_hex=41");
            assert_ne!(strip_positions("location=crates/a.rs:371 message_hex=41"), strip_positions("location=crates/a.rs:372 message_hex=42"), "消息里的差别要判得出（M12）");
            assert_ne!(strip_positions("pid=1"), strip_positions("pid=2"), "pid 不是位置字段");
        }
    }
}

mod node_code {
    //! E164 第二段：三种节点代码摘要、录入代码摘要、判法摘要（跑前登记 5.4），都在仓副本上算。

    use std::collections::BTreeSet;
    use std::path::Path;

    use crate::common::{run_allowed, sha256_hexadecimal, strings};
    use crate::lexical::{combined_digest, function_items_text, has_non_test_function, lexical_tokens, own_source_reads, token_text};
    use crate::repository_scans::{files_under, DependencyGraph};

    /// 「读自己原文」的认法认不认 `include_str!`。只给变异 M13 翻。
    const OWN_SOURCE_READS_INCLUDE_STR: bool = true;

    pub fn read_text(root: &Path, path: &str) -> String {
        std::fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("读不了 {path}：{error}"))
    }

    /// 一个包 `src/` 下全部 `.rs`（仓内路径）。
    pub fn package_source_files(root: &Path, package_directory: &str) -> Vec<String> {
        files_under(root, &format!("{package_directory}/src")).into_iter().filter(|path| path.ends_with(".rs")).collect()
    }

    /// 节点所在包的依赖闭包里全部工作区成员的 `src/` 文件。
    pub fn closure_source_files(root: &Path, graph: &DependencyGraph, package: &str) -> Vec<String> {
        let identifier = graph.identifier_of(package).unwrap_or_else(|| panic!("{package} 是工作区成员"));
        let mut files = Vec::new();
        for member in graph.closure(&identifier, true) {
            let information = &graph.packages[&member];
            if information.is_workspace_member {
                let directory = information.manifest_directory.strip_prefix(root).expect("成员在副本里").display().to_string();
                files.extend(package_source_files(root, &directory));
            }
        }
        files.sort();
        files.dedup();
        files
    }

    /// 目标文件连同它经 `mod x;` 与 `#[path = "…"] mod x;` 带进来的文件（按 crate 根的规则解到同一层目录）。
    pub fn target_files(root: &Path, target_file: &str) -> Vec<String> {
        let mut found = BTreeSet::new();
        let mut pending = vec![target_file.to_string()];
        while let Some(path) = pending.pop() {
            if !found.insert(path.clone()) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(root.join(&path)) else { continue };
            let tokens = lexical_tokens(&text);
            let directory = Path::new(&path).parent().expect("文件有上层").to_path_buf();
            let mut explicit_path: Option<String> = None;
            for (position, token) in tokens.iter().enumerate() {
                let texts: Vec<&str> = tokens[position..tokens.len().min(position + 6)].iter().map(|item| item.text.as_str()).collect();
                if texts.starts_with(&["#", "[", "path", "="]) {
                    explicit_path = texts.get(4).map(|literal| literal.trim_matches('"').to_string());
                }
                if token.text == "mod" && tokens.get(position + 2).is_some_and(|end| end.text == ";") {
                    let name = &tokens[position + 1].text;
                    let resolved = match explicit_path.take() {
                        Some(relative) => normalized(&directory.join(relative)),
                        None => {
                            let flat = directory.join(format!("{name}.rs"));
                            if root.join(&flat).exists() { normalized(&flat) } else { normalized(&directory.join(name).join("mod.rs")) }
                        }
                    };
                    pending.push(resolved);
                }
            }
        }
        found.into_iter().collect()
    }

    fn normalized(path: &Path) -> String {
        let mut parts: Vec<String> = Vec::new();
        for component in path.components() {
            match component.as_os_str().to_string_lossy().as_ref() {
                "." => {}
                ".." => {
                    parts.pop();
                }
                other => parts.push(other.to_string()),
            }
        }
        parts.join("/")
    }

    /// 目标的文件里「读自己原文」读到的仓内文件。
    pub fn own_source_files(root: &Path, files: &[String]) -> BTreeSet<String> {
        let mut read = BTreeSet::new();
        for path in files {
            let text = read_text(root, path);
            let directory = Path::new(path).parent().expect("文件有上层");
            for literal in own_source_reads(&text) {
                if literal == "<file!>" {
                    read.insert(path.clone());
                } else if OWN_SOURCE_READS_INCLUDE_STR || !text.contains("include_str") {
                    let candidate = normalized(&directory.join(&literal));
                    if root.join(&candidate).exists() {
                        read.insert(candidate);
                    }
                }
            }
        }
        read
    }

    pub fn digest_by_function(root: &Path, files: &[String]) -> String {
        combined_digest(&files.iter().map(|path| (path.clone(), function_items_text(&read_text(root, path)))).collect::<Vec<_>>())
    }

    pub fn digest_by_walked_files(root: &Path, files: &[String]) -> String {
        let kept: Vec<(String, String)> = files
            .iter()
            .filter_map(|path| {
                let text = read_text(root, path);
                has_non_test_function(&text).then(|| (path.clone(), token_text(&lexical_tokens(&text))))
            })
            .collect();
        combined_digest(&kept)
    }

    pub fn rustc_version(root: &Path) -> String {
        String::from_utf8_lossy(&run_allowed("rustc", &strings(&["-vV"]), root, &[]).stdout).into_owned()
    }

    /// 按 crate 闭包：闭包 `src/` 与目标文件取词法摘要，`raw_files` 里的取原文摘要，另拼 `Cargo.lock` 原文与 `rustc -vV`。
    pub fn digest_by_crate_closure(root: &Path, files: &[String], raw_files: &BTreeSet<String>, rustc_text: &str) -> String {
        let mut entries: Vec<(String, String)> = files
            .iter()
            .map(|path| {
                let text = read_text(root, path);
                let content = if raw_files.contains(path) { format!("RAW:{}", sha256_hexadecimal(text.as_bytes())) } else { token_text(&lexical_tokens(&text)) };
                (path.clone(), content)
            })
            .collect();
        entries.push(("Cargo.lock".to_string(), sha256_hexadecimal(read_text(root, "Cargo.lock").as_bytes())));
        entries.push(("<rustc -vV>".to_string(), rustc_text.to_string()));
        combined_digest(&entries)
    }

    /// 几个包 `src/` 的词法摘要（判法摘要用）。
    pub fn packages_lexical_digest(root: &Path, package_directories: &[&str]) -> String {
        let mut entries = Vec::new();
        for directory in package_directories {
            for path in package_source_files(root, directory) {
                entries.push((path.clone(), token_text(&lexical_tokens(&read_text(root, &path)))));
            }
        }
        combined_digest(&entries)
    }
}

mod common {
    //! E164 装置的公共部分：SHA-256（手写，不加依赖）、十六进制、E7RESULT 行、子进程白名单（跑前登记 5.1）。

    use std::path::Path;
    use std::process::{Command, Output};

    const SHA256_INITIAL_HASH: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    const SHA256_ROUND_CONSTANTS: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    pub fn sha256(bytes: &[u8]) -> [u8; 32] {
        let mut hash = SHA256_INITIAL_HASH;
        let bit_length = u64::try_from(bytes.len()).expect("长度装得进 u64") * 8;
        let mut message = bytes.to_vec();
        message.push(0x80);
        while message.len() % 64 != 56 {
            message.push(0);
        }
        message.extend_from_slice(&bit_length.to_be_bytes());
        for chunk in message.chunks_exact(64) {
            let mut schedule = [0u32; 64];
            for (word_index, word) in schedule.iter_mut().take(16).enumerate() {
                *word = u32::from_be_bytes(chunk[word_index * 4..word_index * 4 + 4].try_into().expect("切了 4 字节"));
            }
            for word_index in 16..64 {
                let sigma_zero = schedule[word_index - 15].rotate_right(7)
                    ^ schedule[word_index - 15].rotate_right(18)
                    ^ (schedule[word_index - 15] >> 3);
                let sigma_one = schedule[word_index - 2].rotate_right(17)
                    ^ schedule[word_index - 2].rotate_right(19)
                    ^ (schedule[word_index - 2] >> 10);
                schedule[word_index] = schedule[word_index - 16]
                    .wrapping_add(sigma_zero)
                    .wrapping_add(schedule[word_index - 7])
                    .wrapping_add(sigma_one);
            }
            let mut registers = hash;
            for round in 0..64 {
                let big_sigma_one = registers[4].rotate_right(6) ^ registers[4].rotate_right(11) ^ registers[4].rotate_right(25);
                let choose = (registers[4] & registers[5]) ^ ((!registers[4]) & registers[6]);
                let temporary_one = registers[7]
                    .wrapping_add(big_sigma_one)
                    .wrapping_add(choose)
                    .wrapping_add(SHA256_ROUND_CONSTANTS[round])
                    .wrapping_add(schedule[round]);
                let big_sigma_zero = registers[0].rotate_right(2) ^ registers[0].rotate_right(13) ^ registers[0].rotate_right(22);
                let majority = (registers[0] & registers[1]) ^ (registers[0] & registers[2]) ^ (registers[1] & registers[2]);
                let temporary_two = big_sigma_zero.wrapping_add(majority);
                registers = [
                    temporary_one.wrapping_add(temporary_two),
                    registers[0],
                    registers[1],
                    registers[2],
                    registers[3].wrapping_add(temporary_one),
                    registers[4],
                    registers[5],
                    registers[6],
                ];
            }
            for (slot, value) in hash.iter_mut().zip(registers) {
                *slot = slot.wrapping_add(value);
            }
        }
        let mut digest = [0u8; 32];
        for (word_index, word) in hash.iter().enumerate() {
            digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    pub fn hexadecimal(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn sha256_hexadecimal(bytes: &[u8]) -> String {
        hexadecimal(&sha256(bytes))
    }

    /// 一行 `E7RESULT … key=value …` 拆成 key → value（值里不带空白）。
    pub fn parse_result_fields(line: &str) -> std::collections::BTreeMap<String, String> {
        line.split_whitespace()
            .filter_map(|token| token.split_once('='))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    /// 跑前登记 5.1 的子进程白名单：起之前逐条核参数，碰到别的就整轮作废（退 3）。
    pub fn assert_subprocess_is_allowed(program: &str, arguments: &[String]) {
        let joined = arguments.join(" ");
        let forbidden_words = ["singlefs-checker-tier", "layer0", "gate.d", "mutate.sh"];
        if program == "cargo" && forbidden_words.iter().any(|word| joined.contains(word)) {
            refuse_subprocess(program, &joined, "参数里有 checker 档、layer0、门禁阶段或 mutate.sh");
        }
        let allowed = match program {
            "cargo" => match arguments.first().map(String::as_str) {
                Some("build" | "test") => {
                    let package_position = arguments.iter().position(|argument| argument == "-p");
                    let package = package_position.and_then(|position| arguments.get(position + 1)).map(String::as_str);
                    matches!(package, Some("singlefs-harness" | "singlefs-format"))
                }
                Some("metadata") => arguments.iter().any(|argument| argument == "--offline"),
                _ => false,
            },
            "rustc" => {
                arguments == ["-vV"] || arguments.iter().any(|argument| argument == "--emit=metadata")
                    && arguments.windows(2).any(|pair| pair[0] == "--crate-type" && pair[1] == "lib")
            }
            "git" => {
                let subcommand = arguments.iter().find(|argument| !argument.starts_with('-') && !argument.contains('='));
                matches!(subcommand.map(String::as_str), Some("log" | "rev-list" | "ls-files" | "show" | "grep" | "diff" | "rev-parse" | "ls-tree"))
            }
            "python3" => arguments.first().is_some_and(|script| script.ends_with("research/scripts/admission.py"))
                && arguments.get(1).is_some_and(|subcommand| subcommand == "crash-case-manifest"),
            _ => false,
        };
        if !allowed {
            refuse_subprocess(program, &joined, "不在跑前登记 5.1 的子进程白名单里");
        }
    }

    fn refuse_subprocess(program: &str, joined: &str, reason: &str) -> ! {
        eprintln!("  ✗ 装置要起白名单之外的子进程（{reason}）：{program} {joined}");
        eprintln!("  → 怎么办：这一轮作废（跑前登记第十一节 V-tier）；改装置，不放宽白名单");
        std::process::exit(3)
    }

    /// 核过白名单之后起子进程，在 `directory` 里跑，环境按 `environment`（值为 None 的删掉）。
    pub fn run_allowed(
        program: &str,
        arguments: &[String],
        directory: &Path,
        environment: &[(String, Option<String>)],
    ) -> Output {
        assert_subprocess_is_allowed(program, arguments);
        let mut command = Command::new(program);
        command.args(arguments).current_dir(directory);
        for (key, value) in environment {
            match value {
                Some(text) => {
                    command.env(key, text);
                }
                None => {
                    command.env_remove(key);
                }
            }
        }
        command.output().unwrap_or_else(|error| {
            eprintln!("  ✗ 起不了 {program}：{error}");
            eprintln!("  → 怎么办：确认 {program} 在 PATH 上");
            std::process::exit(3)
        })
    }

    pub fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }
}

mod fingerprints {
    //! E164 F13：用例指纹的两臂（跑前登记 5.7「F13」）与「会不会改编出的代码」的重编对照。
    //! 不做臂 = `admission.py crash-case-manifest --judging-digest --toolchain --build-environment` 今天的算法；
    //! 做臂 = admission 今天收的源文件清单（带 --judging-digest，不带后两样）+ 只收会改编出代码的配置键。

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use crate::common::{run_allowed, sha256_hexadecimal, strings};
    use crate::repository_scans::{dependency_graph, ProductSuffix};

    const CASE_KEY: &str = "crash-case:layer0-first-stream";
    const CASE_PACKAGE: &str = "singlefs-checker-tier";

    /// 做臂收的配置表与键：`build` 表里这几个键，`target.*` 下这几个键，`profile` 整张。`[env]` 另按闭包判。只给变异 M19 加键。
    const CONFIGURATION_KEYS_THAT_CHANGE_CODE: &[&str] = &["build.rustflags", "build.rustc", "build.rustc-wrapper", "build.rustc-workspace-wrapper", "build.target"];
    const TARGET_KEYS_THAT_CHANGE_CODE: &[&str] = &["rustflags", "runner", "linker"];
    /// 做臂收的环境变量（名字相等或以 `*` 前的一段起头、以 `*` 后的一段收尾）。只给变异 M20 删键。
    const ENVIRONMENT_VARIABLES_THAT_CHANGE_CODE: &[&str] = &[
        "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_*", "CARGO_TARGET_*_RUSTFLAGS", "CARGO_TARGET_*_RUNNER",
        "CARGO_TARGET_*_LINKER", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_TARGET",
    ];
    /// 闭包里有带 build 脚本的包时另收的环境变量（`[env]` 表里的键另外全收）。
    const BUILD_SCRIPT_ENVIRONMENT_VARIABLES: &[&str] = &["CC", "CXX", "CFLAGS", "CXXFLAGS", "LDFLAGS", "BINDGEN_EXTRA_CLANG_ARGS", "LIBCLANG_PATH", "PKG_CONFIG_PATH"];

    fn variable_matches(pattern: &str, name: &str) -> bool {
        match pattern.split_once('*') {
            Some((head, tail)) => name.starts_with(head) && name.ends_with(tail) && name.len() >= head.len() + tail.len(),
            None => pattern == name,
        }
    }

    /// 拷一份仓（带 .git，跳过名叫 target 的目录）；不带旧修改时刻。
    pub fn copy_tree(source: &Path, destination: &Path) {
        std::fs::create_dir_all(destination).expect("建得了目标目录");
        for entry in std::fs::read_dir(source).expect("读得了源目录").flatten() {
            let file_type = entry.file_type().expect("读得了文件类型");
            let target = destination.join(entry.file_name());
            if file_type.is_dir() {
                if entry.file_name() != "target" {
                    copy_tree(&entry.path(), &target);
                }
            } else if file_type.is_symlink() {
                let link = std::fs::read_link(entry.path()).expect("读得了链接");
                std::os::unix::fs::symlink(link, &target).expect("建得了链接");
            } else {
                std::fs::copy(entry.path(), &target).expect("拷得了文件");
            }
        }
    }

    /// 一份 cargo 配置文件里的（表.键，值原文）。只认单行的 `键 = 值` 与 `[表]`，够判这几种设置。
    fn configuration_entries(text: &str) -> Vec<(String, String)> {
        let mut table = String::new();
        let mut entries = Vec::new();
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(header) = line.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
                table = header.trim().to_string();
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                entries.push((format!("{table}.{}", key.trim()), value.trim().to_string()));
            }
        }
        entries
    }

    /// cargo 读配置的层：从 `directory` 往上每一层的 `.cargo/config.toml` 与 `.cargo/config`，再加 CARGO_HOME 的。
    fn configuration_layers(directory: &Path, environment: &BTreeMap<String, String>) -> Vec<String> {
        let mut texts = Vec::new();
        let mut current = Some(directory.to_path_buf());
        while let Some(level) = current {
            for name in ["config.toml", "config"] {
                if let Ok(text) = std::fs::read_to_string(level.join(".cargo").join(name)) {
                    texts.push(text);
                }
            }
            current = level.parent().map(Path::to_path_buf);
        }
        let cargo_home = environment.get("CARGO_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(environment.get("HOME").cloned().unwrap_or_default()).join(".cargo"));
        for name in ["config.toml", "config"] {
            if let Ok(text) = std::fs::read_to_string(cargo_home.join(name)) {
                texts.push(text);
            }
        }
        texts
    }

    fn program_content_line(value: &str, environment: &BTreeMap<String, String>) -> String {
        let program = value.trim_matches(|character| character == '"' || character == '[' || character == ']').split([' ', ',']).next().unwrap_or("").trim_matches('"');
        let candidates: Vec<PathBuf> = if program.contains('/') {
            vec![PathBuf::from(program)]
        } else {
            environment.get("PATH").map(|path| path.split(':').map(|directory| Path::new(directory).join(program)).collect()).unwrap_or_default()
        };
        match candidates.iter().find_map(|candidate| std::fs::read(candidate).ok()) {
            Some(bytes) => format!("program {program} {}", sha256_hexadecimal(&bytes)),
            None => format!("program {program} missing"),
        }
    }

    /// 做臂里配置那一半：只收会改编出代码的键；闭包里有带 build 脚本的包时另收 `[env]` 与那几个环境变量。
    pub fn code_changing_configuration_lines(directory: &Path, environment: &BTreeMap<String, String>, closure_has_build_script: bool) -> Vec<String> {
        let mut lines = BTreeSet::new();
        for text in configuration_layers(directory, environment) {
            for (key, value) in configuration_entries(&text) {
                let is_target_key = key.starts_with("target.") && TARGET_KEYS_THAT_CHANGE_CODE.iter().any(|name| key.ends_with(&format!(".{name}")));
                let takes = CONFIGURATION_KEYS_THAT_CHANGE_CODE.contains(&key.as_str())
                    || is_target_key
                    || key.starts_with("profile.")
                    || (closure_has_build_script && key.starts_with("env."));
                if takes {
                    if key.ends_with("runner") || key.ends_with("wrapper") {
                        lines.insert(program_content_line(&value, environment));
                    }
                    lines.insert(format!("config {key}={value}"));
                }
                if closure_has_build_script && key.starts_with("env.") {
                    let variable = key.trim_start_matches("env.");
                    if let Some(value) = environment.get(variable) {
                        lines.insert(format!("environment {variable}={value}"));
                    }
                }
            }
        }
        for (name, value) in environment {
            let changes_code = ENVIRONMENT_VARIABLES_THAT_CHANGE_CODE.iter().any(|pattern| variable_matches(pattern, name))
                || (closure_has_build_script && BUILD_SCRIPT_ENVIRONMENT_VARIABLES.contains(&name.as_str()));
            if changes_code {
                if name.ends_with("RUNNER") || name.ends_with("WRAPPER") {
                    lines.insert(program_content_line(value, environment));
                }
                lines.insert(format!("environment {name}={value}"));
            }
        }
        lines.into_iter().collect()
    }

    fn environment_list(environment: &BTreeMap<String, String>) -> Vec<(String, Option<String>)> {
        let current: BTreeMap<String, String> = std::env::vars().collect();
        let mut list: Vec<(String, Option<String>)> = current.keys().filter(|name| !environment.contains_key(*name)).map(|name| (name.clone(), None)).collect();
        list.extend(environment.iter().map(|(name, value)| (name.clone(), Some(value.clone()))));
        list
    }

    fn rustc_version_text(directory: &Path, environment: &BTreeMap<String, String>) -> String {
        let output = run_allowed("rustc", &strings(&["-vV"]), directory, &environment_list(environment));
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// admission 的用例清单：交回（指纹，清单文件的行）。
    fn admission_manifest(copy_root: &Path, environment: &BTreeMap<String, String>, with_environment_lines: bool, scratch: &Path) -> (String, Vec<String>) {
        let manifest_path = scratch.join(format!("manifest-{}.txt", u8::from(with_environment_lines)));
        let mut arguments = vec![
            copy_root.join("research/scripts/admission.py").display().to_string(),
            "crash-case-manifest".to_string(),
            copy_root.display().to_string(),
            CASE_KEY.to_string(),
            manifest_path.display().to_string(),
            "--judging-digest".to_string(),
        ];
        if with_environment_lines {
            arguments.extend(strings(&["--toolchain", "--build-environment"]));
        }
        let output = run_allowed("python3", &arguments, copy_root, &environment_list(environment));
        assert!(output.status.success(), "admission crash-case-manifest 失败：{}", String::from_utf8_lossy(&output.stderr));
        let fingerprint = String::from_utf8_lossy(&output.stdout).split_whitespace().next().expect("admission 打出指纹").to_string();
        let lines = std::fs::read_to_string(&manifest_path).expect("读得了清单").lines().map(str::to_string).collect();
        (fingerprint, lines)
    }

    fn closure_has_build_script(directory: &Path, package: &str, extra_arguments: &[&str]) -> Result<bool, String> {
        let graph = dependency_graph(directory, extra_arguments)?;
        let identifier = graph.identifier_of(package).ok_or_else(|| format!("{package} 不是工作区成员"))?;
        Ok(graph.closure(&identifier, true).iter().any(|member| graph.packages[member].has_build_script))
    }

    struct ArmFingerprints {
        not_doing: String,
        doing: String,
    }

    fn both_fingerprints(copy_root: &Path, environment: &BTreeMap<String, String>, build_script_in_closure: bool, scratch: &Path) -> ArmFingerprints {
        let (not_doing, _) = admission_manifest(copy_root, environment, true, scratch);
        let (_, source_lines) = admission_manifest(copy_root, environment, false, scratch);
        let mut doing_text = source_lines.join("\n");
        doing_text.push('\n');
        doing_text.push_str(&rustc_version_text(copy_root, environment));
        for line in code_changing_configuration_lines(copy_root, environment, build_script_in_closure) {
            doing_text.push_str(&line);
            doing_text.push('\n');
        }
        ArmFingerprints { not_doing, doing: sha256_hexadecimal(doing_text.as_bytes()) }
    }

    /// 重编对照：同一份副本、同一个编译目录里编一次，交回 cargo 对工作区包报了 Compiling / Dirty 没有。
    fn builds_workspace_packages(copy_root: &Path, environment: &BTreeMap<String, String>) -> bool {
        let arguments = strings(&["build", "--release", "--offline", "-p", "singlefs-harness", "--lib", "-v"]);
        let output = run_allowed("cargo", &arguments, copy_root, &environment_list(environment));
        assert!(output.status.success(), "重编对照的 cargo build 失败：{}", String::from_utf8_lossy(&output.stderr).lines().rev().take(5).collect::<Vec<&str>>().join(" / "));
        String::from_utf8_lossy(&output.stderr).lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("Compiling singlefs-") || trimmed.starts_with("Dirty singlefs-")
        })
    }

    struct Setting {
        name: &'static str,
        /// 这一种设置改了编译环境之外什么：记在产物里，只为读的人。
        description: &'static str,
    }

    const SETTINGS: [Setting; 6] = [
        Setting { name: "S1", description: "personal_cargo_home_with_term_color" },
        Setting { name: "S2", description: "upper_directory_cargo_config_with_net_offline" },
        Setting { name: "S3", description: "repository_cargo_config_gcc_13_to_14" },
        Setting { name: "S4", description: "rustflags_target_cpu_native" },
        Setting { name: "S5", description: "profile_release_overflow_checks_environment" },
        Setting { name: "S6", description: "other_path_other_target_directory_other_jobs" },
    ];

    const GCC_THIRTEEN_INCLUDE: &str = "gcc/x86_64-linux-gnu/13/include";
    const GCC_FOURTEEN_INCLUDE: &str = "gcc/x86_64-linux-gnu/14/include";

    fn replace_in_file(path: &Path, old: &str, new: &str) {
        let text = std::fs::read_to_string(path).expect("读得了要改的文件");
        assert_eq!(text.matches(old).count(), 1, "{} 里 {old} 应恰好命中一次", path.display());
        std::fs::write(path, text.replace(old, new)).expect("写得回去");
    }

    /// 施加一种设置：交回这一种设置下的环境；改了文件的由 `revert_setting` 换回。
    fn apply_setting(name: &str, copy_root: &Path, upper: &Path, scratch: &Path, base: &BTreeMap<String, String>) -> BTreeMap<String, String> {
        let mut environment = base.clone();
        match name {
            "S1" => {
                let real_home = base.get("CARGO_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(base.get("HOME").cloned().unwrap_or_default()).join(".cargo"));
                let personal = scratch.join("cargo-home-s1");
                std::fs::create_dir_all(&personal).expect("建得了个人 CARGO_HOME");
                for shared in ["registry", "git", "bin"] {
                    let link = personal.join(shared);
                    if !link.exists() {
                        std::os::unix::fs::symlink(real_home.join(shared), link).expect("链得上共用的缓存");
                    }
                }
                std::fs::write(personal.join("config.toml"), "[term]\ncolor = \"always\"\n").expect("写得了个人配置");
                environment.insert("CARGO_HOME".to_string(), personal.display().to_string());
            }
            "S2" => {
                std::fs::create_dir_all(upper.join(".cargo")).expect("建得了上层 .cargo");
                std::fs::write(upper.join(".cargo/config.toml"), "[net]\noffline = true\n").expect("写得了上层配置");
            }
            "S3" => replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_THIRTEEN_INCLUDE, GCC_FOURTEEN_INCLUDE),
            "S4" => {
                environment.insert("RUSTFLAGS".to_string(), "-C target-cpu=native".to_string());
            }
            "S5" => {
                environment.insert("CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS".to_string(), "true".to_string());
            }
            other => panic!("apply_setting 不管 {other}"),
        }
        environment
    }

    fn revert_setting(name: &str, copy_root: &Path, upper: &Path) {
        match name {
            "S2" => std::fs::remove_file(upper.join(".cargo/config.toml")).expect("删得了上层配置"),
            "S3" => replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_FOURTEEN_INCLUDE, GCC_THIRTEEN_INCLUDE),
            _ => {}
        }
    }

    pub fn scan_fingerprints(root: &Path, suffix: &ProductSuffix) {
        let scratch = crate::scratch_directory().join("f13");
        if scratch.exists() {
            std::fs::remove_dir_all(&scratch).expect("清得掉上一趟的 F13 草稿");
        }
        let upper = scratch.join("upper");
        let copy_root = upper.join("repository");
        copy_tree(root, &copy_root);
        let base: BTreeMap<String, String> = std::env::vars().collect();
        let layer0_build_script = match closure_has_build_script(&copy_root, CASE_PACKAGE, &[]) {
            Ok(answer) => answer,
            Err(message) => {
                suffix.emit(&format!("E7RESULT name=q13_offline_failure closure=layer0 message={}", message.replace(' ', "_")));
                return;
            }
        };
        suffix.emit(&format!("E7RESULT name=q13_closure case={CASE_KEY} closure_has_build_script={layer0_build_script}"));
        let default_fingerprints = both_fingerprints(&copy_root, &base, layer0_build_script, &scratch);
        builds_workspace_packages(&copy_root, &base);
        let mut rows: Vec<(String, bool, bool, String)> = Vec::new();
        for setting in &SETTINGS {
            let (fingerprints, rebuild) = if setting.name == "S6" {
                let other_root = scratch.join("elsewhere/level/repository");
                copy_tree(root, &other_root);
                let mut environment = base.clone();
                environment.insert("CARGO_TARGET_DIR".to_string(), scratch.join("target-s6").display().to_string());
                environment.insert("CARGO_BUILD_JOBS".to_string(), "3".to_string());
                (both_fingerprints(&other_root, &environment, layer0_build_script, &scratch), "not_applicable".to_string())
            } else {
                let environment = apply_setting(setting.name, &copy_root, &upper, &scratch, &base);
                let fingerprints = both_fingerprints(&copy_root, &environment, layer0_build_script, &scratch);
                let rebuild = if builds_workspace_packages(&copy_root, &environment) { "rebuilds" } else { "does_not_rebuild" };
                revert_setting(setting.name, &copy_root, &upper);
                builds_workspace_packages(&copy_root, &base);
                (fingerprints, rebuild.to_string())
            };
            let not_doing_changed = fingerprints.not_doing != default_fingerprints.not_doing;
            let doing_changed = fingerprints.doing != default_fingerprints.doing;
            suffix.emit(&format!(
                "E7RESULT name=q13_setting setting={} description={} not_doing_fingerprint_changed={not_doing_changed} doing_fingerprint_changed={doing_changed} rebuild_control={rebuild}",
                setting.name, setting.description
            ));
            rows.push((setting.name.to_string(), not_doing_changed, doing_changed, rebuild));
        }
        let doing_missed = rows.iter().filter(|(_, _, doing, rebuild)| rebuild == "rebuilds" && !doing).count();
        let doing_false = rows.iter().filter(|(_, _, doing, rebuild)| rebuild == "does_not_rebuild" && *doing).count();
        let not_doing_false = rows.iter().filter(|(_, not_doing, _, rebuild)| rebuild == "does_not_rebuild" && *not_doing).count();
        let not_doing_positive = rows.iter().filter(|(name, not_doing, _, _)| ["S1", "S2", "S3"].contains(&name.as_str()) && *not_doing).count();
        let doing_positive = rows.iter().filter(|(name, _, doing, _)| ["S4", "S5"].contains(&name.as_str()) && *doing).count();
        suffix.emit(&format!(
            "E7RESULT name=q13_summary doing_missed={doing_missed} doing_false={doing_false} not_doing_false={not_doing_false} not_doing_positive_settings_among_s1_to_s3={not_doing_positive} doing_positive_settings_among_s4_s5={doing_positive}"
        ));
        let research = copy_root.join("research");
        match closure_has_build_script(&research, "e7-index-bench", &["--features", "e162-block-stores"]) {
            Ok(e162_build_script) => {
                let before = code_changing_configuration_lines(&research, &base, e162_build_script);
                replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_THIRTEEN_INCLUDE, GCC_FOURTEEN_INCLUDE);
                let after = code_changing_configuration_lines(&research, &base, e162_build_script);
                replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_FOURTEEN_INCLUDE, GCC_THIRTEEN_INCLUDE);
                suffix.emit(&format!("E7RESULT name=q13_geometry closure=e162_block_stores closure_has_build_script={e162_build_script} doing_configuration_changed_by_s3={}", before != after));
            }
            Err(message) => suffix.emit(&format!("E7RESULT name=q13_offline_failure closure=e162 message={}", message.replace(' ', "_"))),
        }
    }
}

mod json {
    //! 读 `cargo metadata --format-version 1` 用的最小 JSON 读取器（不加依赖）。只读，不写。

    use std::collections::BTreeMap;

    #[derive(Clone, Debug, PartialEq)]
    pub enum JsonValue {
        Null,
        Boolean(bool),
        Number(String),
        Text(String),
        Array(Vec<JsonValue>),
        Object(BTreeMap<String, JsonValue>),
    }

    impl JsonValue {
        pub fn field(&self, name: &str) -> &JsonValue {
            match self {
                JsonValue::Object(fields) => fields.get(name).unwrap_or(&JsonValue::Null),
                JsonValue::Null | JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Text(_) | JsonValue::Array(_) => &JsonValue::Null,
            }
        }

        pub fn items(&self) -> &[JsonValue] {
            match self {
                JsonValue::Array(items) => items,
                JsonValue::Null | JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Text(_) | JsonValue::Object(_) => &[],
            }
        }

        pub fn text(&self) -> Option<&str> {
            match self {
                JsonValue::Text(text) => Some(text),
                JsonValue::Null | JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Array(_) | JsonValue::Object(_) => None,
            }
        }
    }

    struct JsonReader<'text> {
        characters: std::iter::Peekable<std::str::Chars<'text>>,
    }

    impl JsonReader<'_> {
        fn skip_whitespace(&mut self) {
            while self.characters.peek().is_some_and(|character| character.is_whitespace()) {
                self.characters.next();
            }
        }

        fn expect_character(&mut self, expected: char) {
            self.skip_whitespace();
            let found = self.characters.next();
            assert_eq!(found, Some(expected), "JSON 这里应是 {expected:?}");
        }

        fn read_text(&mut self) -> String {
            self.expect_character('"');
            let mut text = String::new();
            loop {
                let character = self.characters.next().expect("JSON 串没闭合");
                match character {
                    '"' => return text,
                    '\\' => {
                        let escaped = self.characters.next().expect("JSON 转义没写完");
                        match escaped {
                            'n' => text.push('\n'),
                            't' => text.push('\t'),
                            'r' => text.push('\r'),
                            'b' => text.push('\u{8}'),
                            'f' => text.push('\u{c}'),
                            'u' => {
                                let hexadecimal_digits: String = (0..4).map(|_| self.characters.next().expect("\\u 后面四位")).collect();
                                let code = u32::from_str_radix(&hexadecimal_digits, 16).expect("\\u 后面是十六进制");
                                text.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                            }
                            other => text.push(other),
                        }
                    }
                    other => text.push(other),
                }
            }
        }

        fn read_value(&mut self) -> JsonValue {
            self.skip_whitespace();
            match self.characters.peek().copied() {
                Some('{') => {
                    self.characters.next();
                    let mut fields = BTreeMap::new();
                    self.skip_whitespace();
                    if self.characters.peek() == Some(&'}') {
                        self.characters.next();
                        return JsonValue::Object(fields);
                    }
                    loop {
                        let name = self.read_text();
                        self.expect_character(':');
                        let value = self.read_value();
                        fields.insert(name, value);
                        self.skip_whitespace();
                        match self.characters.next() {
                            Some(',') => continue,
                            Some('}') => return JsonValue::Object(fields),
                            other => panic!("JSON 对象里遇到 {other:?}"),
                        }
                    }
                }
                Some('[') => {
                    self.characters.next();
                    let mut items = Vec::new();
                    self.skip_whitespace();
                    if self.characters.peek() == Some(&']') {
                        self.characters.next();
                        return JsonValue::Array(items);
                    }
                    loop {
                        items.push(self.read_value());
                        self.skip_whitespace();
                        match self.characters.next() {
                            Some(',') => continue,
                            Some(']') => return JsonValue::Array(items),
                            other => panic!("JSON 数组里遇到 {other:?}"),
                        }
                    }
                }
                Some('"') => JsonValue::Text(self.read_text()),
                Some('t') | Some('f') | Some('n') => {
                    let mut word = String::new();
                    while self.characters.peek().is_some_and(|character| character.is_ascii_alphabetic()) {
                        word.push(self.characters.next().expect("刚看过有"));
                    }
                    match word.as_str() {
                        "true" => JsonValue::Boolean(true),
                        "false" => JsonValue::Boolean(false),
                        "null" => JsonValue::Null,
                        other => panic!("认不出的 JSON 字 {other}"),
                    }
                }
                _ => {
                    let mut number = String::new();
                    while self.characters.peek().is_some_and(|character| character.is_ascii_digit() || "+-.eE".contains(*character)) {
                        number.push(self.characters.next().expect("刚看过有"));
                    }
                    assert!(!number.is_empty(), "JSON 这里应是一个值");
                    JsonValue::Number(number)
                }
            }
        }
    }

    pub fn parse_json(text: &str) -> JsonValue {
        let mut reader = JsonReader { characters: text.chars().peekable() };
        reader.read_value()
    }
}

mod repository_scans_w3 {
    //! E164 W3：登记时冻结文件名在全部 git 历史上撞不撞（跑前登记 5.7「W3」）。只读 git。

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    use crate::repository_scans::{git_text, ProductSuffix};

    struct RenameEvent {
        commit: String,
        commit_time: u64,
        old_path: String,
        new_path: String,
    }

    /// `git log --all -M --name-status --diff-filter=<过滤> --format=commit:%H:%ct` 按提交拆成（提交，时刻，名字状态行）。
    fn name_status_by_commit(root: &Path, diff_filter: &str) -> Vec<(String, u64, Vec<Vec<String>>)> {
        let filter_argument = format!("--diff-filter={diff_filter}");
        let text = git_text(root, &["log", "--all", "-M", "--name-status", &filter_argument, "--format=commit:%H:%ct"]);
        let mut commits: Vec<(String, u64, Vec<Vec<String>>)> = Vec::new();
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("commit:") {
                let (hash, time) = rest.split_once(':').expect("commit:<哈希>:<时刻>");
                commits.push((hash.to_string(), time.parse().expect("时刻是数"), Vec::new()));
            } else if !line.is_empty() {
                let fields: Vec<String> = line.split('\t').map(str::to_string).collect();
                commits.last_mut().expect("名字状态行前面有提交行").2.push(fields);
            }
        }
        commits
    }

    fn has_test_attribute_at(root: &Path, commit: &str, path: &str) -> bool {
        if !path.ends_with(".rs") {
            return false;
        }
        let specification = format!("{commit}:{path}");
        git_text(root, &["show", &specification]).contains("#[test]")
    }

    /// 一次提交的树里 `#[test]` 函数：（仓内路径，函数名）。取 `#[test]` 之后同一份文件里第一行带 `fn ` 的。
    fn test_functions_at(root: &Path, commit: &str) -> Vec<(String, String)> {
        let arguments = crate::common::strings(&["grep", "-z", "-n", "-A", "6", "-e", "#[test]", "-F", commit, "--", "*.rs"]);
        let output = crate::common::run_allowed("git", &arguments, root, &[]);
        let no_match = output.status.code() == Some(1) && output.stderr.is_empty();
        assert!(output.status.success() || no_match, "git grep 在 {commit} 上失败：{}", String::from_utf8_lossy(&output.stderr));
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        let mut functions = Vec::new();
        let mut waiting_file: Option<String> = None;
        for line in text.lines() {
            if line == "--" {
                continue;
            }
            let without_commit = line.strip_prefix(commit).and_then(|rest| rest.strip_prefix(':')).unwrap_or(line);
            let mut parts = without_commit.splitn(3, '\0');
            let (Some(path), Some(_line_number), Some(content)) = (parts.next(), parts.next(), parts.next()) else { continue };
            if content.trim_start().starts_with("#[test]") {
                waiting_file = Some(path.to_string());
                continue;
            }
            if waiting_file.as_deref() == Some(path) {
                if let Some(after) = content.split("fn ").nth(1) {
                    let name: String = after.chars().take_while(|character| character.is_alphanumeric() || *character == '_').collect();
                    if !name.is_empty() {
                        functions.push((path.to_string(), name));
                        waiting_file = None;
                    }
                }
            }
        }
        functions
    }

    /// W3 的域：全部引用可达的提交、全部路径。
    pub fn all_commits(root: &Path) -> Vec<String> {
        git_text(root, &["rev-list", "--all"]).lines().map(str::to_string).collect()
    }

    pub fn scan_frozen_file_names(root: &Path, suffix: &ProductSuffix) {
        let commits = all_commits(root);
        let mut renames = Vec::new();
        for (commit, commit_time, entries) in name_status_by_commit(root, "R") {
            for fields in entries {
                if fields.first().is_some_and(|status| status.starts_with('R')) && fields.len() == 3 {
                    renames.push(RenameEvent { commit: commit.clone(), commit_time, old_path: fields[1].clone(), new_path: fields[2].clone() });
                }
            }
        }
        let mut creations: Vec<(String, u64, String, Option<String>)> = Vec::new();
        for (commit, commit_time, entries) in name_status_by_commit(root, "AR") {
            for fields in entries {
                match (fields.first().map(String::as_str), fields.len()) {
                    (Some("A"), 2) => creations.push((commit.clone(), commit_time, fields[1].clone(), None)),
                    (Some(status), 3) if status.starts_with('R') => creations.push((commit.clone(), commit_time, fields[2].clone(), Some(fields[1].clone()))),
                    _ => {}
                }
            }
        }
        let mut reoccupied_all = 0usize;
        let mut reoccupied_test_files = 0usize;
        let mut test_file_renames = 0usize;
        for rename in &renames {
            let is_test_file = has_test_attribute_at(root, &rename.commit, &rename.new_path);
            test_file_renames += usize::from(is_test_file);
            let reoccupied = creations.iter().any(|(commit, commit_time, path, source)| {
                *commit != rename.commit
                    && *commit_time > rename.commit_time
                    && *path == rename.old_path
                    && source.as_deref() != Some(rename.new_path.as_str())
            });
            if reoccupied {
                reoccupied_all += 1;
                reoccupied_test_files += usize::from(is_test_file);
                suffix.emit(&format!("E7RESULT name=w3_reoccupied commit={} old={} new={} test_file={is_test_file}", rename.commit, rename.old_path, rename.new_path));
            }
        }
        suffix.emit(&format!(
            "E7RESULT name=w3_scan commits={} renames={} test_file_renames={test_file_renames}",
            commits.len(),
            renames.len()
        ));
        suffix.emit(&format!("E7RESULT name=w3_reoccupied_summary all_paths={reoccupied_all} test_rs_files={reoccupied_test_files}"));
        let mut commits_with_duplicate_by_file_name = 0usize;
        let mut commits_with_duplicate_by_path = 0usize;
        let mut most_duplicates_by_file_name = 0usize;
        let mut most_duplicates_by_path = 0usize;
        let mut example_by_path: Option<String> = None;
        let mut distinct_duplicates_by_path: BTreeSet<String> = BTreeSet::new();
        for commit in &commits {
            let functions = test_functions_at(root, commit);
            let mut by_file_name: BTreeMap<String, usize> = BTreeMap::new();
            let mut by_path: BTreeMap<String, usize> = BTreeMap::new();
            for (path, name) in &functions {
                let file_name = path.rsplit('/').next().unwrap_or(path);
                *by_file_name.entry(format!("{file_name}::{name}")).or_default() += 1;
                *by_path.entry(format!("{path}::{name}")).or_default() += 1;
            }
            let duplicated_file_names = by_file_name.values().filter(|count| **count > 1).count();
            let duplicated_paths: BTreeSet<&String> = by_path.iter().filter(|(_, count)| **count > 1).map(|(identity, _)| identity).collect();
            commits_with_duplicate_by_file_name += usize::from(duplicated_file_names > 0);
            commits_with_duplicate_by_path += usize::from(!duplicated_paths.is_empty());
            most_duplicates_by_file_name = most_duplicates_by_file_name.max(duplicated_file_names);
            most_duplicates_by_path = most_duplicates_by_path.max(duplicated_paths.len());
            distinct_duplicates_by_path.extend(duplicated_paths.iter().map(|identity| (*identity).clone()));
            if example_by_path.is_none() {
                example_by_path = duplicated_paths.iter().next().map(|identity| format!("{commit}:{identity}"));
            }
        }
        suffix.emit(&format!(
            "E7RESULT name=w3_uniqueness commits_scanned={} commits_with_duplicate_by_file_name={commits_with_duplicate_by_file_name} most_duplicates_in_one_commit_by_file_name={most_duplicates_by_file_name} commits_with_duplicate_by_path={commits_with_duplicate_by_path} most_duplicates_in_one_commit_by_path={most_duplicates_by_path} first_duplicate_by_path={}",
            commits.len(),
            example_by_path.unwrap_or_else(|| "none".to_string())
        ));
        for identity in &distinct_duplicates_by_path {
            suffix.emit(&format!("E7RESULT name=w3_duplicate_identity_by_path identity={identity}"));
        }
        suffix.emit(&format!("E7RESULT name=w3_duplicate_identities_by_path distinct={}", distinct_duplicates_by_path.len()));
    }
}

mod repository_scans_w5 {
    //! E164 W5：节点代码按 crate 闭包取时，一次改动让多少节点重跑（跑前登记 5.7「W5」）。只读 git 与 `cargo metadata --offline`。

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    use crate::repository_scans::{dependency_graph, git_text, DependencyGraph, ProductSuffix};

    /// 闭包算不算 dev-dependencies：节点是测试目标，dev 依赖进它的编译。只给变异 M22 翻。
    const CLOSURE_INCLUDES_DEVELOPMENT_DEPENDENCIES: bool = true;

    struct Node {
        name: String,
        is_crash_enumeration: bool,
        package: String,
        /// 节点自己的目标文件（仓内路径）；库节点没有单独的目标文件，它就是 `src/`。
        target_file: Option<String>,
    }

    fn package_directory(graph: &DependencyGraph, root: &Path, name: &str) -> String {
        let identifier = graph.identifier_of(name).unwrap_or_else(|| panic!("{name} 是工作区成员"));
        graph.packages[&identifier].manifest_directory.strip_prefix(root).expect("工作区成员在仓里").display().to_string()
    }

    fn nodes_of(root: &Path, graph: &DependencyGraph) -> Vec<Node> {
        let mut nodes = Vec::new();
        let stage_inputs = std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("读得了 stage-inputs.tsv");
        for line in stage_inputs.lines().filter(|line| line.starts_with("crash-case:")) {
            let columns: Vec<&str> = line.split('\t').collect();
            let test_column = columns.iter().find_map(|column| column.strip_prefix("test=")).expect("crash-case 行有 test= 列");
            let mut parts = test_column.split(':');
            let package = parts.next().expect("test= 的包").to_string();
            let target = parts.next().expect("test= 的测试目标").to_string();
            let directory = package_directory(graph, root, &package);
            nodes.push(Node {
                name: columns[0].to_string(),
                is_crash_enumeration: true,
                target_file: Some(format!("{directory}/tests/{target}.rs")),
                package,
            });
        }
        for package in ["singlefs-harness", "singlefs-checker", "singlefs-core", "singlefs-format"] {
            let directory = package_directory(graph, root, package);
            nodes.push(Node { name: format!("{package}:--lib"), is_crash_enumeration: false, package: package.to_string(), target_file: None });
            let mut test_files: Vec<String> = std::fs::read_dir(root.join(&directory).join("tests"))
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
                        .map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .filter(|name| name.ends_with(".rs"))
                        .collect()
                })
                .unwrap_or_default();
            test_files.sort();
            for file in test_files {
                nodes.push(Node {
                    name: format!("{package}:{}", file.trim_end_matches(".rs")),
                    is_crash_enumeration: false,
                    package: package.to_string(),
                    target_file: Some(format!("{directory}/tests/{file}")),
                });
            }
        }
        nodes
    }

    /// 节点要重跑当且仅当它的闭包里有包的 `src/` 被改了，或这次提交改了它自己的目标文件。
    pub fn node_reruns(closure: &BTreeSet<String>, touched_packages: &BTreeSet<String>, target_file: Option<&str>, changed_files: &[String]) -> bool {
        !closure.is_disjoint(touched_packages) || target_file.is_some_and(|file| changed_files.iter().any(|changed| changed == file))
    }

    fn commit_class(touched: &BTreeSet<String>) -> String {
        match touched.iter().collect::<Vec<&String>>().as_slice() {
            [only] => only.trim_start_matches("singlefs-").to_string(),
            _ => "multi".to_string(),
        }
    }

    fn ratio_summary(values: &mut [f64]) -> String {
        if values.is_empty() {
            return "none".to_string();
        }
        values.sort_by(|left, right| left.partial_cmp(right).expect("比例不是 NaN"));
        let median = values[values.len() / 2];
        format!("{:.3}/{median:.3}/{:.3}", values[0], values[values.len() - 1])
    }

    pub fn scan_rerun_surface(root: &Path, suffix: &ProductSuffix) {
        let graph = match dependency_graph(root, &[]) {
            Ok(graph) => graph,
            Err(message) => {
                suffix.emit(&format!("E7RESULT name=w5_offline_failure message={}", message.replace(' ', "_")));
                return;
            }
        };
        let nodes = nodes_of(root, &graph);
        let directory_to_package: BTreeMap<String, String> = graph
            .packages
            .values()
            .filter(|package| package.is_workspace_member)
            .filter_map(|package| package.manifest_directory.strip_prefix(root).ok().map(|directory| (directory.display().to_string(), package.name.clone())))
            .collect();
        let closures: Vec<BTreeSet<String>> = nodes
            .iter()
            .map(|node| {
                let identifier = graph.identifier_of(&node.package).expect("节点的包是工作区成员");
                graph.member_names(&graph.closure(&identifier, CLOSURE_INCLUDES_DEVELOPMENT_DEPENDENCIES))
            })
            .collect();
        let nodes_whose_closure_needs_development_dependencies = nodes
            .iter()
            .zip(&closures)
            .filter(|(node, closure)| {
                let identifier = graph.identifier_of(&node.package).expect("节点的包是工作区成员");
                graph.member_names(&graph.closure(&identifier, false)) != **closure
            })
            .count();
        let crash_total = nodes.iter().filter(|node| node.is_crash_enumeration).count();
        let test_total = nodes.len() - crash_total;
        for (node, closure) in nodes.iter().zip(&closures) {
            suffix.emit(&format!("E7RESULT name=w5_node node={} crash_enumeration={} closure={}", node.name, node.is_crash_enumeration, closure.iter().cloned().collect::<Vec<String>>().join(",")));
        }
        suffix.emit(&format!("E7RESULT name=w5_nodes total={} crash_enumeration={crash_total} test={test_total} nodes_whose_closure_needs_development_dependencies={nodes_whose_closure_needs_development_dependencies}", nodes.len()));
        for window in [10usize, 30] {
            let count_argument = format!("-n{window}");
            let commits: Vec<String> = git_text(root, &["log", &count_argument, "--format=%H", "--", "crates/*/src/*"]).lines().map(str::to_string).collect();
            let mut by_class: BTreeMap<String, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
            for commit in &commits {
                let changed: Vec<String> = git_text(root, &["show", "--name-only", "--format=", commit]).lines().filter(|line| !line.is_empty()).map(str::to_string).collect();
                let touched: BTreeSet<String> = changed
                    .iter()
                    .filter_map(|path| {
                        directory_to_package.iter().find(|(directory, _)| path.starts_with(&format!("{directory}/src/"))).map(|(_, package)| package.clone())
                    })
                    .collect();
                let mut crash_rerun = 0usize;
                let mut test_rerun = 0usize;
                for (node, closure) in nodes.iter().zip(&closures) {
                    let reruns = node_reruns(closure, &touched, node.target_file.as_deref(), &changed);
                    if reruns {
                        if node.is_crash_enumeration {
                            crash_rerun += 1;
                        } else {
                            test_rerun += 1;
                        }
                    }
                }
                let class = commit_class(&touched);
                let crash_ratio = crash_rerun as f64 / crash_total.max(1) as f64;
                let test_ratio = test_rerun as f64 / test_total.max(1) as f64;
                let entry = by_class.entry(class.clone()).or_default();
                entry.0.push(crash_ratio);
                entry.1.push(test_ratio);
                if window == 30 {
                    suffix.emit(&format!(
                        "E7RESULT name=w5_commit commit={commit} class={class} touched_packages={} crash_enumeration_rerun={crash_rerun}/{crash_total} test_rerun={test_rerun}/{test_total}",
                        touched.iter().cloned().collect::<Vec<String>>().join(",")
                    ));
                }
            }
            for (class, (crash_ratios, test_ratios)) in &mut by_class {
                suffix.emit(&format!(
                    "E7RESULT name=w5_class_summary window={window} commits_in_window={} class={class} commits={} crash_enumeration_rerun_ratio_min_median_max={} test_rerun_ratio_min_median_max={}",
                    commits.len(),
                    crash_ratios.len(),
                    ratio_summary(crash_ratios),
                    ratio_summary(test_ratios)
                ));
            }
        }
    }
}

mod lexical {
    //! E164 的词法摘要与四种节点代码取法（跑前登记 5.4「词法摘要」「节点代码：按函数 / 按走到的文件」「读自己原文的目标」）。
    //! 不链接 rustc 的词法器：照 5.4 的字面手写一份，锚点在跑前登记 7.2 前五行。

    use crate::common::sha256_hexadecimal;

    /// 一个词法单元：文本取源码原文。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LexicalToken {
        pub text: String,
    }

    fn is_identifier_start(character: char) -> bool {
        character == '_' || character.is_alphabetic()
    }

    fn is_identifier_continue(character: char) -> bool {
        character == '_' || character.is_alphanumeric()
    }

    /// 从 `start` 起（指向开引号）读一个带转义的串，交回结束之后的下标。
    fn end_of_escaped_quoted(characters: &[char], start: usize, quote: char) -> usize {
        let mut position = start + 1;
        while position < characters.len() {
            if characters[position] == '\\' {
                position += 2;
                continue;
            }
            if characters[position] == quote {
                return position + 1;
            }
            position += 1;
        }
        characters.len()
    }

    /// 从 `start` 起（指向 `r`）读原始串 `r#…#"…"#…#`，交回结束之后的下标；不是原始串交回 None。
    fn end_of_raw_string(characters: &[char], start: usize) -> Option<usize> {
        let mut position = start + 1;
        let mut hashes = 0usize;
        while position < characters.len() && characters[position] == '#' {
            hashes += 1;
            position += 1;
        }
        if position >= characters.len() || characters[position] != '"' {
            return None;
        }
        position += 1;
        while position < characters.len() {
            if characters[position] == '"'
                && (0..hashes).all(|offset| characters.get(position + 1 + offset) == Some(&'#'))
            {
                return Some(position + 1 + hashes);
            }
            position += 1;
        }
        Some(characters.len())
    }

    /// 从 `start` 起跳过可嵌套的块注释，交回结束之后的下标。
    fn end_of_block_comment(characters: &[char], start: usize) -> usize {
        let mut depth = 0usize;
        let mut position = start;
        while position + 1 < characters.len() {
            if characters[position] == '/' && characters[position + 1] == '*' {
                depth += 1;
                position += 2;
                continue;
            }
            if characters[position] == '*' && characters[position + 1] == '/' {
                depth -= 1;
                position += 2;
                if depth == 0 {
                    return position;
                }
                continue;
            }
            position += 1;
        }
        characters.len()
    }

    /// 5.4「词法摘要」：去空白、行注释、块注释（可嵌套）与全部文档注释；串、字节串、原始串、字符、生命周期各算一个单元。
    pub fn lexical_tokens(source: &str) -> Vec<LexicalToken> {
        let characters: Vec<char> = source.chars().collect();
        let mut tokens = Vec::new();
        let mut position = 0usize;
        while position < characters.len() {
            let current = characters[position];
            let next = characters.get(position + 1).copied();
            if current.is_whitespace() {
                position += 1;
                continue;
            }
            if current == '/' && next == Some('/') {
                while position < characters.len() && characters[position] != '\n' {
                    position += 1;
                }
                continue;
            }
            if current == '/' && next == Some('*') {
                position = end_of_block_comment(&characters, position);
                continue;
            }
            let token_start = position;
            if (current == 'r' || (current == 'b' && next == Some('r')))
                && end_of_raw_string(&characters, if current == 'b' { position + 1 } else { position }).is_some()
            {
                position = end_of_raw_string(&characters, if current == 'b' { position + 1 } else { position })
                    .expect("上一行判过是原始串");
            } else if current == 'b' && (next == Some('"') || next == Some('\'')) {
                position = end_of_escaped_quoted(&characters, position + 1, next.expect("判过是引号"));
            } else if current == '"' {
                position = end_of_escaped_quoted(&characters, position, '"');
            } else if current == '\'' {
                position = end_of_character_or_lifetime(&characters, position);
            } else if current == 'r' && next == Some('#') && characters.get(position + 2).is_some_and(|character| is_identifier_start(*character)) {
                position += 2;
                while position < characters.len() && is_identifier_continue(characters[position]) {
                    position += 1;
                }
            } else if is_identifier_start(current) {
                while position < characters.len() && is_identifier_continue(characters[position]) {
                    position += 1;
                }
            } else if current.is_ascii_digit() {
                position = end_of_number(&characters, position);
            } else {
                position += 1;
            }
            tokens.push(LexicalToken {
                text: characters[token_start..position].iter().collect(),
            });
        }
        tokens
    }

    /// `'` 起头：`'\…'` 与 `'x'` 是字符，其余是生命周期。
    fn end_of_character_or_lifetime(characters: &[char], start: usize) -> usize {
        if characters.get(start + 1) == Some(&'\\') {
            return end_of_escaped_quoted(characters, start, '\'');
        }
        if characters.get(start + 2) == Some(&'\'') {
            return start + 3;
        }
        let mut position = start + 1;
        while position < characters.len() && is_identifier_continue(characters[position]) {
            position += 1;
        }
        position
    }

    /// 数字字面量：数字起头，接字母数字下划线，`.` 后面紧跟数字时连上。
    fn end_of_number(characters: &[char], start: usize) -> usize {
        let mut position = start;
        while position < characters.len() {
            let character = characters[position];
            let continues_with_fraction = character == '.'
                && characters.get(position + 1).is_some_and(char::is_ascii_digit);
            if is_identifier_continue(character) || continues_with_fraction {
                position += 1;
            } else {
                break;
            }
        }
        position
    }

    pub fn token_text(tokens: &[LexicalToken]) -> String {
        tokens.iter().map(|token| token.text.as_str()).collect::<Vec<&str>>().join(" ")
    }

    /// 一份源码的词法摘要：单元串用一个空格连起来取 SHA-256。
    pub fn lexical_digest(source: &str) -> String {
        sha256_hexadecimal(token_text(&lexical_tokens(source)).as_bytes())
    }

    /// 多份文件的拼法：按仓内相对路径字典序，每份拼「路径 + 换行 + 内容串 + 换行」，整串取 SHA-256。
    /// `per_file` 给每份文件交回它那一段内容串（词法单元串、原文摘要，或别的取法）。
    pub fn combined_digest(files: &[(String, String)]) -> String {
        let mut sorted: Vec<&(String, String)> = files.iter().collect();
        sorted.sort_by(|left, right| left.0.cmp(&right.0));
        let mut joined = String::new();
        for (path, content_text) in sorted {
            joined.push_str(path);
            joined.push('\n');
            joined.push_str(content_text);
            joined.push('\n');
        }
        sha256_hexadecimal(joined.as_bytes())
    }

    /// 从 `start`（一个开括号 `{`）找到配对的 `}`，交回它的下标。
    fn matching_close_brace(tokens: &[LexicalToken], start: usize) -> usize {
        let mut depth = 0usize;
        for (position, token) in tokens.iter().enumerate().skip(start) {
            match token.text.as_str() {
                "{" => depth += 1,
                "}" => {
                    depth -= 1;
                    if depth == 0 {
                        return position;
                    }
                }
                _ => {}
            }
        }
        tokens.len() - 1
    }

    /// 从 `start` 起找一个项的末尾：先遇到 `;` 就到 `;`，先遇到 `{` 就到配对的 `}`。
    fn end_of_item(tokens: &[LexicalToken], start: usize) -> usize {
        for position in start..tokens.len() {
            match tokens[position].text.as_str() {
                ";" => return position,
                "{" => return matching_close_brace(tokens, position),
                _ => {}
            }
        }
        tokens.len() - 1
    }

    /// `#[cfg(test)]` 或 `#[test]` 这一个属性从 `position` 起，交回属性末尾（`]`）的下标。
    fn test_attribute_end(tokens: &[LexicalToken], position: usize) -> Option<usize> {
        let texts: Vec<&str> = tokens[position..tokens.len().min(position + 7)].iter().map(|token| token.text.as_str()).collect();
        if texts.starts_with(&["#", "[", "cfg", "(", "test", ")", "]"]) {
            return Some(position + 6);
        }
        if texts.starts_with(&["#", "[", "test", "]"]) {
            return Some(position + 3);
        }
        None
    }

    /// 5.4「节点代码：按函数」：非测试 fn 项（从 `fn` 到体的配对 `}`）的单元，按次序；
    /// 带 `#[cfg(test)]` 或 `#[test]` 的项（测试模块、测试函数）整个跳过；`fn(` 这类函数指针类型不算项。
    pub fn non_test_function_items(tokens: &[LexicalToken]) -> Vec<Vec<LexicalToken>> {
        let mut items = Vec::new();
        let mut position = 0usize;
        let mut next_item_is_test = false;
        while position < tokens.len() {
            if let Some(attribute_end) = test_attribute_end(tokens, position) {
                next_item_is_test = true;
                position = attribute_end + 1;
                continue;
            }
            let text = tokens[position].text.as_str();
            if text == "#" {
                position += 1;
                continue;
            }
            if next_item_is_test && (text == "mod" || text == "fn") {
                position = end_of_item(tokens, position) + 1;
                next_item_is_test = false;
                continue;
            }
            let names_a_function = text == "fn"
                && tokens.get(position + 1).is_some_and(|name| name.text != "(");
            if names_a_function {
                let end = end_of_item(tokens, position);
                items.push(tokens[position..=end].to_vec());
                position = end + 1;
                continue;
            }
            position += 1;
        }
        items
    }

    /// 按函数取时一份文件的内容串：各非测试 fn 项的单元串按次序用空格连起来。
    pub fn function_items_text(source: &str) -> String {
        non_test_function_items(&lexical_tokens(source))
            .iter()
            .map(|item| token_text(item))
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// 按走到的文件取时这份文件进不进：至少含一个非测试 fn 项。
    pub fn has_non_test_function(source: &str) -> bool {
        !non_test_function_items(&lexical_tokens(source)).is_empty()
    }

    /// 5.4「读自己原文的目标」：源码里出现 `include_str!`、`include_bytes!`、`file!()`，或以字面路径读仓内 `.rs`，
    /// 交回它读的字面路径（`file!()` 交回 `<file!>`，由调用方换成这份文件自己）。
    pub fn own_source_reads(source: &str) -> Vec<String> {
        let tokens = lexical_tokens(source);
        let mut reads = Vec::new();
        for (position, token) in tokens.iter().enumerate() {
            let literal_argument = tokens.get(position + 3).map(|argument| argument.text.clone());
            match token.text.as_str() {
                "include_str" | "include_bytes" if tokens.get(position + 1).is_some_and(|bang| bang.text == "!") => {
                    if let Some(argument) = literal_argument.filter(|argument| argument.starts_with('"')) {
                        reads.push(argument.trim_matches('"').to_string());
                    }
                }
                "file" if tokens.get(position + 1).is_some_and(|bang| bang.text == "!") => reads.push("<file!>".to_string()),
                "read_to_string" | "read" => {
                    let argument = tokens.get(position + 2).map(|argument| argument.text.clone());
                    if let Some(literal) = argument.filter(|literal| literal.starts_with('"') && literal.ends_with(".rs\"")) {
                        reads.push(literal.trim_matches('"').to_string());
                    }
                }
                _ => {}
            }
        }
        reads
    }
}

mod pipeline {
    //! E164 F10、F11 的流水线与领块模型（跑前登记 5.6）：录入与核对两个进程经库交接，穷举交错、被杀、换判法、卡掉线、租期到期。
    //! 显式状态、深度优先、按状态去重；终局 = 没有后继的可达状态；门禁在终局读一次。

    use std::collections::{BTreeSet, HashSet};

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum DataState {
        Absent,
        Half,
        Full,
    }

    /// 「已核对」标记：没有、布尔、或记下核它的判法摘要（0 = D1，1 = D2）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum CheckedFlag {
        Unset,
        Boolean,
        Digest(u8),
    }

    /// 领块记录（只在落盘的两形里进库）：哪张卡、租期还有效没有。
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum PersistedClaim {
        Unclaimed,
        Held { card: u8, lease_valid: bool },
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct StoreBlock {
        pub data: DataState,
        pub recorded: bool,
        pub checked: CheckedFlag,
        /// 结果：算它时读到的数据、算它用的判法。
        pub result: Option<(DataState, u8)>,
        pub tally: u8,
        pub claim: PersistedClaim,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum RecordOrder {
        /// 数据与「已录入」标记同一批原子写。
        Atomic,
        /// 标记先于数据写（record_flag_before_data）：标记、半块、整块三次写。
        FlagBeforeData,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum CheckOrder {
        /// 一批的结果与标记同一次原子写，结果按块键覆盖。
        AtomicBatchKeyed,
        /// 逐块先写「已核对」标记、再写结果（checked_flag_before_result）。
        FlagBeforeResult,
        /// 逐块先写结果（按追加计）、再写标记（result_then_flag_with_appended_tally）。
        ResultThenFlagAppended,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum ClaimForm {
        /// 领块只在内存：库里不记。
        InMemory,
        /// 领块落盘、不带租期。
        PersistedWithoutLease,
        /// 领块落盘、带租期：租期可在任何时刻到期，到期后别的卡可以接。
        PersistedWithLease,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ModelForm {
        pub record_order: RecordOrder,
        pub check_order: CheckOrder,
        pub flag_records_digest: bool,
        pub flush_when_blocked: bool,
        pub claim_form: ClaimForm,
        pub cards_can_drop: bool,
        pub allow_kills: bool,
        pub allow_judging_change: bool,
        /// 带租期那一形里，结果写回时领块记录已不是自己的（租期到了被接走）：按追加计结果。只给变异 M17 翻。
        pub late_writer_appends: bool,
        /// 租期会不会到期。只给变异 M18 翻。
        pub lease_can_expire: bool,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Geometry {
        pub blocks: usize,
        pub cards: usize,
        pub backlog_cap: usize,
        pub batch: usize,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum CardState {
        Dropped,
        Alive {
            held: Vec<usize>,
            computed: Vec<(DataState, u8)>,
            writes_done: usize,
            killed_once: bool,
        },
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct ModelState {
        pub store: Vec<StoreBlock>,
        pub recorder_next_block: usize,
        pub recorder_step: usize,
        pub recorder_killed_once: bool,
        pub cards: Vec<CardState>,
        pub judging_phase: u8,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct ModelReport {
        pub reachable_states: u64,
        pub terminal_states: u64,
        pub missing_result: u64,
        pub wrong_input: u64,
        pub double_tally: u64,
        pub stale_after_change: u64,
        pub stuck: u64,
        pub dead_block_terminals: u64,
        pub kill_transitions: u64,
        pub judging_change_transitions: u64,
        pub card_drop_transitions: u64,
        pub takeover_after_lease_transitions: u64,
        pub backlog_peak: usize,
        pub backlog_positive_states: u64,
        pub terminal_backlog_values: BTreeSet<usize>,
        pub held_peak: usize,
        pub held_positive_states: u64,
        pub terminal_held_values: BTreeSet<usize>,
        pub stopped_over_limit: bool,
    }

    pub const REACHABLE_STATE_LIMIT: u64 = 10_000_000;

    fn checked_now(block: &StoreBlock, form: &ModelForm, phase: u8) -> bool {
        match block.checked {
            CheckedFlag::Unset => false,
            CheckedFlag::Boolean => true,
            CheckedFlag::Digest(digest) => form.flag_records_digest && digest == phase,
        }
    }

    fn pending_blocks(state: &ModelState, form: &ModelForm) -> Vec<usize> {
        (0..state.store.len())
            .filter(|index| state.store[*index].recorded && !checked_now(&state.store[*index], form, state.judging_phase))
            .collect()
    }

    fn held_block_count(state: &ModelState) -> usize {
        state
            .store
            .iter()
            .filter(|block| matches!(block.claim, PersistedClaim::Held { lease_valid: true, .. }))
            .count()
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TransitionKind {
        Ordinary,
        Kill,
        JudgingChange,
        CardDrop,
        TakeoverAfterLease,
    }

    fn initial_state(geometry: &Geometry) -> ModelState {
        ModelState {
            store: vec![
                StoreBlock {
                    data: DataState::Absent,
                    recorded: false,
                    checked: CheckedFlag::Unset,
                    result: None,
                    tally: 0,
                    claim: PersistedClaim::Unclaimed,
                };
                geometry.blocks
            ],
            recorder_next_block: 0,
            recorder_step: 0,
            recorder_killed_once: false,
            cards: vec![
                CardState::Alive { held: Vec::new(), computed: Vec::new(), writes_done: 0, killed_once: false };
                geometry.cards
            ],
            judging_phase: 0,
        }
    }

    fn drop_already_used(state: &ModelState) -> bool {
        state.cards.iter().any(|card| matches!(card, CardState::Dropped))
    }

    fn recorder_successors(state: &ModelState, form: &ModelForm, geometry: &Geometry, output: &mut Vec<(ModelState, TransitionKind)>) {
        let block_index = state.recorder_next_block;
        if block_index >= geometry.blocks {
            return;
        }
        let backlog = pending_blocks(state, form).len();
        if state.recorder_step == 0 && state.store[block_index].recorded {
            let mut next = state.clone();
            next.recorder_next_block += 1;
            output.push((next, TransitionKind::Ordinary));
        } else if state.recorder_step > 0 || backlog < geometry.backlog_cap {
            let mut next = state.clone();
            let block = &mut next.store[block_index];
            let writes_in_this_order = match form.record_order {
                RecordOrder::Atomic => {
                    block.data = DataState::Full;
                    block.recorded = true;
                    1
                }
                RecordOrder::FlagBeforeData => {
                    match state.recorder_step {
                        0 => block.recorded = true,
                        1 => block.data = DataState::Half,
                        _ => block.data = DataState::Full,
                    }
                    3
                }
            };
            if state.recorder_step + 1 == writes_in_this_order {
                next.recorder_next_block += 1;
                next.recorder_step = 0;
            } else {
                next.recorder_step += 1;
            }
            output.push((next, TransitionKind::Ordinary));
        }
        if form.allow_kills && !state.recorder_killed_once {
            let mut next = state.clone();
            next.recorder_next_block = 0;
            next.recorder_step = 0;
            next.recorder_killed_once = true;
            output.push((next, TransitionKind::Kill));
        }
    }

    fn claim_allows(claim: PersistedClaim, form: &ModelForm, card: u8) -> bool {
        match (form.claim_form, claim) {
            (ClaimForm::InMemory, _) | (_, PersistedClaim::Unclaimed) => true,
            (ClaimForm::PersistedWithoutLease | ClaimForm::PersistedWithLease, PersistedClaim::Held { card: holder, lease_valid }) => {
                holder == card || !lease_valid
            }
        }
    }

    fn flag_for(form: &ModelForm, digest: u8) -> CheckedFlag {
        if form.flag_records_digest {
            CheckedFlag::Digest(digest)
        } else {
            CheckedFlag::Boolean
        }
    }

    /// 一次结果写回：自己的领块记录还在（或不落盘）时按块键覆盖；带租期、被接走之后迟到的写回按 `late_writer_appends` 办。
    fn write_result(block: &mut StoreBlock, computed: (DataState, u8), form: &ModelForm, card: u8, appended: bool) {
        let own_claim = matches!(block.claim, PersistedClaim::Held { card: holder, lease_valid: true } if holder == card);
        let late = form.claim_form == ClaimForm::PersistedWithLease && !own_claim;
        block.result = Some(computed);
        if appended || (late && form.late_writer_appends) {
            block.tally = block.tally.saturating_add(1);
        } else {
            block.tally = 1;
        }
    }

    fn release_own_claim(block: &mut StoreBlock, card: u8) {
        if matches!(block.claim, PersistedClaim::Held { card: holder, .. } if holder == card) {
            block.claim = PersistedClaim::Unclaimed;
        }
    }

    fn card_successors(state: &ModelState, form: &ModelForm, geometry: &Geometry, output: &mut Vec<(ModelState, TransitionKind)>) {
        for card_index in 0..state.cards.len() {
            let card = u8::try_from(card_index).expect("卡数不超过 255");
            let CardState::Alive { held, computed, writes_done, killed_once } = &state.cards[card_index] else {
                continue;
            };
            if held.is_empty() {
                let pending = pending_blocks(state, form);
                let free: Vec<usize> = pending.iter().copied().filter(|index| claim_allows(state.store[*index].claim, form, card)).collect();
                let recorder_done = state.recorder_next_block >= geometry.blocks;
                let may_take = free.len() >= geometry.batch
                    || (!free.is_empty() && (recorder_done || (form.flush_when_blocked && pending.len() >= geometry.backlog_cap)));
                if may_take {
                    let chosen: Vec<usize> = free.iter().copied().take(geometry.batch).collect();
                    let mut next = state.clone();
                    let mut kind = TransitionKind::Ordinary;
                    if form.claim_form != ClaimForm::InMemory {
                        for block_index in &chosen {
                            if matches!(next.store[*block_index].claim, PersistedClaim::Held { card: holder, lease_valid: false } if holder != card) {
                                kind = TransitionKind::TakeoverAfterLease;
                            }
                            next.store[*block_index].claim = PersistedClaim::Held { card, lease_valid: true };
                        }
                    }
                    next.cards[card_index] = CardState::Alive { held: chosen, computed: Vec::new(), writes_done: 0, killed_once: *killed_once };
                    output.push((next, kind));
                }
            } else if computed.len() < held.len() {
                let mut next = state.clone();
                let read = (state.store[held[computed.len()]].data, state.judging_phase);
                if let CardState::Alive { computed: next_computed, .. } = &mut next.cards[card_index] {
                    next_computed.push(read);
                }
                output.push((next, TransitionKind::Ordinary));
            } else {
                let mut next = state.clone();
                let total_writes = match form.check_order {
                    CheckOrder::AtomicBatchKeyed => {
                        for (block_index, result) in held.iter().zip(computed) {
                            let block = &mut next.store[*block_index];
                            write_result(block, *result, form, card, false);
                            block.checked = flag_for(form, result.1);
                            release_own_claim(block, card);
                        }
                        1
                    }
                    CheckOrder::FlagBeforeResult | CheckOrder::ResultThenFlagAppended => {
                        let position = writes_done / 2;
                        let block = &mut next.store[held[position]];
                        let result = computed[position];
                        let flag_first = form.check_order == CheckOrder::FlagBeforeResult;
                        let writing_flag = (writes_done % 2 == 0) == flag_first;
                        if writing_flag {
                            block.checked = flag_for(form, result.1);
                        } else {
                            write_result(block, result, form, card, !flag_first);
                        }
                        if writes_done % 2 == 1 {
                            release_own_claim(block, card);
                        }
                        held.len() * 2
                    }
                };
                next.cards[card_index] = if writes_done + 1 == total_writes {
                    CardState::Alive { held: Vec::new(), computed: Vec::new(), writes_done: 0, killed_once: *killed_once }
                } else {
                    CardState::Alive { held: held.clone(), computed: computed.clone(), writes_done: writes_done + 1, killed_once: *killed_once }
                };
                output.push((next, TransitionKind::Ordinary));
            }
            if form.allow_kills && !killed_once {
                let mut next = state.clone();
                next.cards[card_index] = CardState::Alive { held: Vec::new(), computed: Vec::new(), writes_done: 0, killed_once: true };
                output.push((next, TransitionKind::Kill));
            }
            if form.cards_can_drop && !drop_already_used(state) {
                let mut next = state.clone();
                next.cards[card_index] = CardState::Dropped;
                output.push((next, TransitionKind::CardDrop));
            }
        }
    }

    fn environment_successors(state: &ModelState, form: &ModelForm, output: &mut Vec<(ModelState, TransitionKind)>) {
        if form.allow_judging_change && state.judging_phase == 0 {
            let mut next = state.clone();
            next.judging_phase = 1;
            output.push((next, TransitionKind::JudgingChange));
        }
        if form.claim_form == ClaimForm::PersistedWithLease && form.lease_can_expire {
            for block_index in 0..state.store.len() {
                if let PersistedClaim::Held { card, lease_valid: true } = state.store[block_index].claim {
                    let mut next = state.clone();
                    next.store[block_index].claim = PersistedClaim::Held { card, lease_valid: false };
                    output.push((next, TransitionKind::Ordinary));
                }
            }
        }
    }

    fn tally_terminal(state: &ModelState, form: &ModelForm, report: &mut ModelReport) {
        report.terminal_states += 1;
        let phase = state.judging_phase;
        let mut missing_result = false;
        let mut wrong_input = false;
        let mut double_tally = false;
        let mut stale_after_change = false;
        let mut stuck = false;
        let mut dead_block = false;
        for block in &state.store {
            let checked = checked_now(block, form, phase);
            if checked && block.result.is_none() {
                missing_result = true;
            }
            if block.result.is_some_and(|(data, _)| data != DataState::Full) {
                wrong_input = true;
            }
            if block.tally > 1 {
                double_tally = true;
            }
            if checked && block.result.is_some_and(|(_, digest)| digest != phase) {
                stale_after_change = true;
            }
            if !checked {
                stuck = true;
                if let PersistedClaim::Held { card, lease_valid: true } = block.claim {
                    if matches!(state.cards[usize::from(card)], CardState::Dropped) {
                        dead_block = true;
                    }
                }
            }
        }
        report.missing_result += u64::from(missing_result);
        report.wrong_input += u64::from(wrong_input);
        report.double_tally += u64::from(double_tally);
        report.stale_after_change += u64::from(stale_after_change);
        report.stuck += u64::from(stuck);
        report.dead_block_terminals += u64::from(dead_block);
        report.terminal_backlog_values.insert(pending_blocks(state, form).len());
        report.terminal_held_values.insert(held_block_count(state));
    }

    /// 穷举一种形在一个几何点上的全部可达状态。
    pub fn explore(form: &ModelForm, geometry: &Geometry) -> ModelReport {
        let mut report = ModelReport::default();
        let mut seen: HashSet<ModelState> = HashSet::new();
        let mut stack = vec![initial_state(geometry)];
        while let Some(state) = stack.pop() {
            if !seen.insert(state.clone()) {
                continue;
            }
            if seen.len() as u64 > REACHABLE_STATE_LIMIT {
                report.stopped_over_limit = true;
                break;
            }
            let backlog = pending_blocks(&state, form).len();
            report.backlog_peak = report.backlog_peak.max(backlog);
            report.backlog_positive_states += u64::from(backlog > 0);
            let held = held_block_count(&state);
            report.held_peak = report.held_peak.max(held);
            report.held_positive_states += u64::from(held > 0);
            let mut successors = Vec::new();
            recorder_successors(&state, form, geometry, &mut successors);
            card_successors(&state, form, geometry, &mut successors);
            environment_successors(&state, form, &mut successors);
            if successors.is_empty() {
                tally_terminal(&state, form, &mut report);
                continue;
            }
            for (next, kind) in successors {
                match kind {
                    TransitionKind::Ordinary => {}
                    TransitionKind::Kill => report.kill_transitions += 1,
                    TransitionKind::JudgingChange => report.judging_change_transitions += 1,
                    TransitionKind::CardDrop => report.card_drop_transitions += 1,
                    TransitionKind::TakeoverAfterLease => report.takeover_after_lease_transitions += 1,
                }
                stack.push(next);
            }
        }
        report.reachable_states = seen.len() as u64;
        report
    }

    /// 跑前登记 5.6 的最强形：录入原子写、核对一批原子写、标记记判法摘要、结果按块键覆盖、领块只在内存、中途换一次判法、可被杀。
    pub const STRONGEST_FORM: ModelForm = ModelForm {
        record_order: RecordOrder::Atomic,
        check_order: CheckOrder::AtomicBatchKeyed,
        flag_records_digest: true,
        flush_when_blocked: true,
        claim_form: ClaimForm::InMemory,
        cards_can_drop: false,
        allow_kills: true,
        allow_judging_change: true,
        late_writer_appends: false,
        lease_can_expire: true,
    };

    /// F10 的六种形：最强形与五种弱形，每种弱形只改最强形的一样。
    pub fn pipeline_forms() -> Vec<(&'static str, ModelForm)> {
        vec![
            ("strongest", STRONGEST_FORM),
            ("record_flag_before_data", ModelForm { record_order: RecordOrder::FlagBeforeData, ..STRONGEST_FORM }),
            ("checked_flag_before_result", ModelForm { check_order: CheckOrder::FlagBeforeResult, ..STRONGEST_FORM }),
            ("result_then_flag_with_appended_tally", ModelForm { check_order: CheckOrder::ResultThenFlagAppended, ..STRONGEST_FORM }),
            ("checked_flag_is_a_boolean", ModelForm { flag_records_digest: false, ..STRONGEST_FORM }),
            ("batch_larger_than_backlog_cap_without_flush", ModelForm { flush_when_blocked: false, ..STRONGEST_FORM }),
        ]
    }

    /// F11 的三形：都开卡掉线，其余取最强形。
    pub fn claim_forms() -> Vec<(&'static str, ModelForm)> {
        let with_drops = ModelForm { cards_can_drop: true, ..STRONGEST_FORM };
        vec![
            ("claims_persisted_without_lease", ModelForm { claim_form: ClaimForm::PersistedWithoutLease, ..with_drops }),
            ("claims_in_memory", ModelForm { claim_form: ClaimForm::InMemory, ..with_drops }),
            ("claims_persisted_with_lease", ModelForm { claim_form: ClaimForm::PersistedWithLease, ..with_drops }),
        ]
    }

    /// 跑前登记 5.6 的几何：块数 2 与 3、卡数 1 与 2、积压上限 1 与 2、批 1 与 2。
    pub fn geometries() -> Vec<Geometry> {
        let mut points = Vec::new();
        for blocks in [2usize, 3] {
            for cards in [1usize, 2] {
                for backlog_cap in [1usize, 2] {
                    for batch in [1usize, 2] {
                        points.push(Geometry { blocks, cards, backlog_cap, batch });
                    }
                }
            }
        }
        points
    }
}

mod repository_scans {
    //! E164 读主仓现状的几件：W3（冻结文件名）、W5（按 crate 闭包的重跑面）、F13（指纹两臂与重编对照）。
    //! 产物每行带开跑时的 HEAD 与主工作区 `crates/` 的原文摘要（跑前登记 5.1）。

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use crate::common::{run_allowed, sha256_hexadecimal, strings};
    use crate::json::{parse_json, JsonValue};
    use crate::lexical::combined_digest;

    /// 身份串（第三轮判决第四节）：`<登记时冻结的代码文件名>::<验证目标名>::<流程步号>::<哈希串>`。
    pub fn identity_text(code_file_name: &str, target_name: &str, step_number: u32, hash_text: &str) -> String {
        format!("{code_file_name}::{target_name}::{step_number}::{hash_text}")
    }

    pub fn git_text(root: &Path, arguments: &[&str]) -> String {
        let output = run_allowed("git", &strings(arguments), root, &[]);
        assert!(output.status.success(), "git {} 失败：{}", arguments.join(" "), String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// 目录下全部文件（跳过名叫 `target` 与 `.git` 的目录），交回相对 `base` 的路径。
    pub fn files_under(base: &Path, relative_directory: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut pending = vec![base.join(relative_directory)];
        while let Some(directory) = pending.pop() {
            let Ok(entries) = std::fs::read_dir(&directory) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let file_type = entry.file_type().expect("读得了文件类型");
                if file_type.is_dir() {
                    if name != "target" && name != ".git" {
                        pending.push(path);
                    }
                } else if file_type.is_file() {
                    found.push(path.strip_prefix(base).expect("在 base 下").display().to_string());
                }
            }
        }
        found.sort();
        found
    }

    /// 主工作区 `crates/` 的原文摘要（5.4「原文摘要」的多文件拼法）。
    pub fn crates_raw_digest(root: &Path) -> String {
        let files: Vec<(String, String)> = files_under(root, "crates")
            .into_iter()
            .map(|path| {
                let bytes = std::fs::read(root.join(&path)).expect("读得了 crates/ 下的文件");
                (path, sha256_hexadecimal(&bytes))
            })
            .collect();
        combined_digest(&files)
    }

    pub struct ProductSuffix(pub String);

    impl ProductSuffix {
        pub fn current(root: &Path) -> Self {
            let head = git_text(root, &["rev-parse", "HEAD"]).trim().to_string();
            Self(format!("head={head} crates_raw_digest={}", crates_raw_digest(root)))
        }
        pub fn emit(&self, line: &str) {
            result_line!("{line} {}", self.0);
        }
    }

    // ============================== cargo metadata 的依赖图 ==============================

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum DependencyKind {
        Normal,
        Development,
        Build,
    }

    pub struct PackageInformation {
        pub name: String,
        pub manifest_directory: PathBuf,
        pub has_build_script: bool,
        pub is_workspace_member: bool,
    }

    pub struct DependencyGraph {
        pub packages: BTreeMap<String, PackageInformation>,
        pub dependencies: BTreeMap<String, Vec<(String, Vec<DependencyKind>)>>,
    }

    pub fn dependency_graph(directory: &Path, extra_arguments: &[&str]) -> Result<DependencyGraph, String> {
        let mut arguments = strings(&["metadata", "--offline", "--format-version", "1"]);
        arguments.extend(strings(extra_arguments));
        let output = run_allowed("cargo", &arguments, directory, &[]);
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).lines().take(3).collect::<Vec<&str>>().join(" / "));
        }
        let metadata = parse_json(&String::from_utf8_lossy(&output.stdout));
        let members: BTreeSet<String> = metadata.field("workspace_members").items().iter().filter_map(|member| member.text().map(str::to_string)).collect();
        let mut packages = BTreeMap::new();
        for package in metadata.field("packages").items() {
            let identifier = package.field("id").text().expect("包有 id").to_string();
            let manifest_path = PathBuf::from(package.field("manifest_path").text().expect("包有 manifest_path"));
            let has_build_script = package.field("targets").items().iter().any(|target| {
                target.field("kind").items().iter().any(|kind| kind.text() == Some("custom-build"))
            });
            packages.insert(
                identifier.clone(),
                PackageInformation {
                    name: package.field("name").text().expect("包有名字").to_string(),
                    manifest_directory: manifest_path.parent().expect("manifest 有上层").to_path_buf(),
                    has_build_script,
                    is_workspace_member: members.contains(&identifier),
                },
            );
        }
        let mut dependencies = BTreeMap::new();
        for node in metadata.field("resolve").field("nodes").items() {
            let identifier = node.field("id").text().expect("节点有 id").to_string();
            let edges = node
                .field("deps")
                .items()
                .iter()
                .map(|edge| {
                    let kinds = edge
                        .field("dep_kinds")
                        .items()
                        .iter()
                        .map(|kind| match kind.field("kind") {
                            JsonValue::Text(text) if text == "dev" => DependencyKind::Development,
                            JsonValue::Text(text) if text == "build" => DependencyKind::Build,
                            _ => DependencyKind::Normal,
                        })
                        .collect();
                    (edge.field("pkg").text().expect("边有 pkg").to_string(), kinds)
                })
                .collect();
            dependencies.insert(identifier, edges);
        }
        Ok(DependencyGraph { packages, dependencies })
    }

    impl DependencyGraph {
        pub fn identifier_of(&self, name: &str) -> Option<String> {
            self.packages.iter().find(|(_, package)| package.name == name && package.is_workspace_member).map(|(identifier, _)| identifier.clone())
        }

        /// 节点所在包的依赖闭包：包自己的 normal、build（`include_development` 时加 dev）依赖，再往下只走 normal 与 build。
        pub fn closure(&self, root_identifier: &str, include_development: bool) -> BTreeSet<String> {
            let mut closure = BTreeSet::from([root_identifier.to_string()]);
            let mut pending = Vec::new();
            for (dependency, kinds) in self.dependencies.get(root_identifier).into_iter().flatten() {
                let follows = kinds.iter().any(|kind| *kind != DependencyKind::Development || include_development);
                if follows {
                    pending.push(dependency.clone());
                }
            }
            while let Some(identifier) = pending.pop() {
                if !closure.insert(identifier.clone()) {
                    continue;
                }
                for (dependency, kinds) in self.dependencies.get(&identifier).into_iter().flatten() {
                    if kinds.iter().any(|kind| *kind != DependencyKind::Development) {
                        pending.push(dependency.clone());
                    }
                }
            }
            closure
        }

        pub fn member_names(&self, closure: &BTreeSet<String>) -> BTreeSet<String> {
            closure.iter().filter_map(|identifier| self.packages.get(identifier)).filter(|package| package.is_workspace_member).map(|package| package.name.clone()).collect()
        }
    }

    pub fn command_repository() {
        let root = crate::repository_root();
        let suffix = ProductSuffix::current(&root);
        crate::repository_scans_w3::scan_frozen_file_names(&root, &suffix);
        crate::repository_scans_w5::scan_rerun_surface(&root, &suffix);
        crate::fingerprints::scan_fingerprints(&root, &suffix);
    }
}

mod variants {
    //! E164 第二段：仓副本、施加变体（跑前登记 5.3）、harness 档探针与点名测试，读回录制流。

    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use crate::common::{run_allowed, sha256_hexadecimal, strings};
    use crate::fingerprints::copy_tree;
    use crate::lexical::combined_digest;
    use crate::repository_scans::files_under;
    use crate::write_tables::{OperationKind, RecordedOperation};

    pub const PROBE_SOURCE: &str = include_str!("e164_crash_amplification_design_function_test/probe_recording.rs");
    const VARIANT_PATCHES: &str = include_str!("e164_crash_amplification_design_function_test/variant_patches.tsv");
    pub const PROBE_TEST_FILE: &str = "crates/singlefs-harness/tests/e164_probe.rs";

    /// 变体表：名字 → 怎么施加（`crates/mutations.tsv` 的变异名，或 `variant_patches.tsv` 的一行，或不改）。
    pub fn variant_sources() -> Vec<(&'static str, Option<VariantSource>)> {
        vec![
            ("V0", None),
            ("V0b", None),
            ("V1", Some(VariantSource::MutationRow("步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）"))),
            ("V2", Some(VariantSource::Patch("V2"))),
            ("V3", Some(VariantSource::MutationRow("步 3：前缀跨实例边界（把别的实例的记录也接上）"))),
            ("V4", Some(VariantSource::Patch("V4"))),
            ("V5", Some(VariantSource::MutationRow("记录头 311：池级 checker 的载荷校验和偏移没跟着后挪 4 字节"))),
            ("V6", Some(VariantSource::Patch("V6"))),
            ("V6b", Some(VariantSource::Patch("V6b"))),
            ("V7", Some(VariantSource::Patch("V7"))),
            ("V8", Some(VariantSource::Patch("V8"))),
            ("V11", Some(VariantSource::MutationRow("格式常量字面量改掉一个数：DATA_POINTER_BYTES"))),
        ]
    }

    #[derive(Clone, Copy)]
    pub enum VariantSource {
        MutationRow(&'static str),
        Patch(&'static str),
    }

    pub struct Replacement {
        pub file: String,
        pub original: String,
        pub replacement: String,
    }

    fn unescape(text: &str) -> String {
        text.replace("\\n", "\n")
    }

    pub fn replacement_of(root: &Path, source: VariantSource) -> Replacement {
        let (table, key) = match source {
            VariantSource::MutationRow(name) => (std::fs::read_to_string(root.join("crates/mutations.tsv")).expect("读得了 crates/mutations.tsv"), name),
            VariantSource::Patch(name) => (VARIANT_PATCHES.to_string(), name),
        };
        let row = table
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.split('\t').collect::<Vec<&str>>())
            .find(|columns| columns[0] == key)
            .unwrap_or_else(|| stop(&format!("变体 {key} 在表里找不到（S-anchor）")));
        Replacement { file: row[1].to_string(), original: unescape(row[2]), replacement: unescape(row[3]) }
    }

    pub fn stop(reason: &str) -> ! {
        eprintln!("  ✗ 停机：{reason}");
        eprintln!("  → 怎么办：两边都查（跑前登记第十一节），装置与被测代码都不默认对；写进报告交主 agent");
        std::process::exit(4)
    }

    /// 副本 `crates/` 的原文摘要；`replacement` 给了就在内存里把它施加到主仓原文上再算（「原样 + 这一个变体」）。
    pub fn crates_raw_digest_of(root: &Path, replacement: Option<&Replacement>) -> String {
        let entries: Vec<(String, String)> = files_under(root, "crates")
            .into_iter()
            .filter(|path| path != PROBE_TEST_FILE)
            .map(|path| {
                let mut bytes = std::fs::read(root.join(&path)).expect("读得了文件");
                if let Some(change) = replacement.filter(|change| change.file == path) {
                    let text = String::from_utf8(bytes).expect("源码是 UTF-8");
                    bytes = text.replacen(&change.original, &change.replacement, 1).into_bytes();
                }
                (path, sha256_hexadecimal(&bytes))
            })
            .collect();
        combined_digest(&entries)
    }

    /// 建一份变体副本：拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`（不带 .git、不带 target、不带旧修改时刻），施加变体，放探针。
    pub fn prepare_copy(root: &Path, directory: &Path, replacement: Option<&Replacement>) {
        if directory.exists() {
            std::fs::remove_dir_all(directory).expect("清得掉上一趟的副本");
        }
        std::fs::create_dir_all(directory).expect("建得了副本目录");
        copy_tree(&root.join("crates"), &directory.join("crates"));
        copy_tree(&root.join(".cargo"), &directory.join(".cargo"));
        for file in ["Cargo.toml", "Cargo.lock"] {
            std::fs::copy(root.join(file), directory.join(file)).expect("拷得了工作区文件");
        }
        if let Some(change) = replacement {
            let path = directory.join(&change.file);
            let text = std::fs::read_to_string(&path).expect("读得了要改的文件");
            if text.matches(&change.original).count() != 1 {
                stop(&format!("{} 里变体原文不是恰好命中一次（S-anchor）", change.file));
            }
            std::fs::write(&path, text.replacen(&change.original, &change.replacement, 1)).expect("写得回去");
        }
        let expected = crates_raw_digest_of(root, replacement);
        let actual = crates_raw_digest_of(directory, None);
        if expected != actual {
            eprintln!("  ✗ 副本 crates/ 的原文摘要不等于「原样 + 这一个变体」（V-build）");
            eprintln!("  → 怎么办：这一格作废；查副本是不是拷全了、变体是不是施加对了");
            std::process::exit(5);
        }
        for forbidden in ["CrashImage", "enumerate_layer0", "check_pool_image"] {
            if PROBE_SOURCE.contains(forbidden) {
                eprintln!("  ✗ 探针源码里出现 {forbidden}（V-tier）");
                eprintln!("  → 怎么办：探针只录制，不造崩溃状态；改探针");
                std::process::exit(5);
            }
        }
        std::fs::write(directory.join(PROBE_TEST_FILE), PROBE_SOURCE).expect("放得下探针");
    }

    pub struct TestRun {
        pub exit_code: i32,
        pub stdout: String,
    }

    pub fn cargo_test(directory: &Path, arguments: &[&str]) -> TestRun {
        let mut full = strings(&["test", "--offline"]);
        full.extend(strings(arguments));
        let target_directory = directory.join("target").display().to_string();
        let output = run_allowed("cargo", &full, directory, &[("CARGO_TARGET_DIR".to_string(), Some(target_directory))]);
        TestRun { exit_code: output.status.code().unwrap_or(-1), stdout: String::from_utf8_lossy(&output.stdout).into_owned() }
    }

    /// 探针读回的一条历史。
    #[derive(Default)]
    pub struct ProbeHistory {
        pub steps: Vec<(String, usize, usize)>,
        pub operations: Vec<RecordedOperation>,
        pub mkfs_sectors: BTreeMap<(u32, u64), String>,
        pub harness_segments: BTreeMap<String, (String, String)>,
    }

    fn result_fields(line: &str) -> Option<BTreeMap<String, String>> {
        line.find("E7RESULT ").map(|start| crate::common::parse_result_fields(&line[start..]))
    }

    /// 从探针输出里读一条历史（`probe` 取 h_short 或 h_long）；完整性闸 V-count 不过就交回 Err。
    pub fn parse_history(stdout: &str, probe: &str) -> Result<ProbeHistory, String> {
        let mut history = ProbeHistory::default();
        let mut counted_lines = 0usize;
        let mut end_fields: Option<BTreeMap<String, String>> = None;
        for fields in stdout.lines().filter_map(result_fields).filter(|fields| fields.get("probe").map(String::as_str) == Some(probe)) {
            let field = |name: &str| fields.get(name).cloned().unwrap_or_default();
            match field("name").as_str() {
                "step" => history.steps.push((field("step"), field("start").parse().expect("start"), field("end").parse().expect("end"))),
                "op" => {
                    let kind = OperationKind::from_serialized_name(&field("kind")).expect("认得的种类");
                    let sectors = field("sectors");
                    history.operations.push(RecordedOperation {
                        device: field("device").parse().expect("device"),
                        kind,
                        offset_in_bytes: field("offset").parse().expect("offset"),
                        length_in_bytes: field("length").parse().expect("length"),
                        content_sha256: field("sha256"),
                        sector_sha256s: if sectors == "none" { Vec::new() } else { sectors.split(',').map(str::to_string).collect() },
                    });
                }
                "mkfs_sector" => {
                    history.mkfs_sectors.insert((field("device").parse().expect("device"), field("sector").parse().expect("sector")), field("sha256"));
                }
                "harness_segments" => {
                    history.harness_segments.insert(field("step"), (field("sizes"), field("closed_form")));
                }
                "probe_end" => {
                    end_fields = Some(fields.clone());
                    continue;
                }
                _ => {}
            }
            counted_lines += 1;
        }
        let end = end_fields.ok_or_else(|| format!("{probe} 没有 probe_end 行"))?;
        let emitted: usize = end["emitted_before_this_line"].parse().expect("emitted");
        let operations: usize = end["operations"].parse().expect("operations");
        if emitted != counted_lines || operations != history.operations.len() {
            return Err(format!("{probe} 完整性闸：读到 {counted_lines} 行 / {} 条操作，探针报 {emitted} 行 / {operations} 条（V-count）", history.operations.len()));
        }
        Ok(history)
    }

    pub fn variant_directory(scratch: &Path, name: &str) -> PathBuf {
        scratch.join("variants").join(name)
    }

    pub fn command_variants(scratch: &Path) {
        if scratch.as_os_str().is_empty() {
            eprintln!("  ✗ variants 要一个草稿目录参数");
            eprintln!("  → 怎么办：variants <草稿目录>，放得下十二份 crates/ 副本与各自的编译目录");
            std::process::exit(2);
        }
        crate::arms::run_segment_two(&crate::repository_root(), scratch);
    }
}

mod write_tables {
    //! E164 的切段、两态闭式与三种写表哈希（跑前登记 5.4「写表哈希甲 / 乙 / 丙」「mkfs 基线属性」「输入：父输出」、第七节序列化）。
    //! 切段照 D13（验证路线） 已定项 4 的字面独立写：一次写只被它自己那块盘上之后的屏障（或那块盘上的 FUA）放行；
    //! 当前段里每块有写的盘都放行了才关段；段里还没有写时的屏障并进下一段；流尾只有屏障的并进上一段。

    use std::collections::{BTreeMap, BTreeSet};

    use crate::common::sha256_hexadecimal;

    pub const SECTOR_BYTES: u64 = 512;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum OperationKind {
        Write,
        WriteForceUnitAccess,
        WriteZeroes,
        Barrier,
    }

    impl OperationKind {
        pub fn serialized_name(self) -> &'static str {
            match self {
                OperationKind::Write => "write",
                OperationKind::WriteForceUnitAccess => "write_fua",
                OperationKind::WriteZeroes => "write_zeroes",
                OperationKind::Barrier => "barrier",
            }
        }

        pub fn from_serialized_name(name: &str) -> Option<Self> {
            match name {
                "write" => Some(OperationKind::Write),
                "write_fua" => Some(OperationKind::WriteForceUnitAccess),
                "write_zeroes" => Some(OperationKind::WriteZeroes),
                "barrier" => Some(OperationKind::Barrier),
                _ => None,
            }
        }
    }

    /// 录制流里的一条操作（探针打出的 `name=op` 行）。清零写的字节摘要记成 `zeroes`，扇区摘要为空。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RecordedOperation {
        pub device: u32,
        pub kind: OperationKind,
        pub offset_in_bytes: u64,
        pub length_in_bytes: u64,
        pub content_sha256: String,
        pub sector_sha256s: Vec<String>,
    }

    impl RecordedOperation {
        pub fn barrier(device: u32) -> Self {
            Self {
                device,
                kind: OperationKind::Barrier,
                offset_in_bytes: 0,
                length_in_bytes: 0,
                content_sha256: "none".to_string(),
                sector_sha256s: Vec::new(),
            }
        }

        pub fn write(device: u32, kind: OperationKind, offset_in_bytes: u64, bytes: &[u8]) -> Self {
            Self {
                device,
                kind,
                offset_in_bytes,
                length_in_bytes: u64::try_from(bytes.len()).expect("长度装得进 u64"),
                content_sha256: sha256_hexadecimal(bytes),
                sector_sha256s: bytes.chunks(512).map(sha256_hexadecimal).collect(),
            }
        }
    }

    /// D13 已定项 4「同一块盘上连着的屏障并成一道」：尾部那串屏障里这块盘已有一道时，新的一道不记。
    pub fn append_with_barrier_merging(stream: &mut Vec<RecordedOperation>, operation: RecordedOperation) {
        let this_device_already_in_the_trailing_barriers = stream
            .iter()
            .rev()
            .take_while(|previous| previous.kind == OperationKind::Barrier)
            .any(|previous| previous.device == operation.device);
        if operation.kind == OperationKind::Barrier && this_device_already_in_the_trailing_barriers {
            return;
        }
        stream.push(operation);
    }

    /// 切段：交回每条操作落在第几段（从 0 起）。
    pub fn segment_index_of_each_operation(operations: &[RecordedOperation]) -> Vec<usize> {
        let mut indexes = Vec::with_capacity(operations.len());
        let mut current_segment = 0usize;
        let mut unreleased_devices: BTreeSet<u32> = BTreeSet::new();
        let mut writes_in_current_segment = 0usize;
        for operation in operations {
            indexes.push(current_segment);
            let closes = match operation.kind {
                OperationKind::Write | OperationKind::WriteZeroes => {
                    writes_in_current_segment += 1;
                    unreleased_devices.insert(operation.device);
                    false
                }
                OperationKind::WriteForceUnitAccess => {
                    writes_in_current_segment += 1;
                    unreleased_devices.remove(&operation.device);
                    unreleased_devices.is_empty()
                }
                OperationKind::Barrier => {
                    unreleased_devices.remove(&operation.device);
                    writes_in_current_segment > 0 && unreleased_devices.is_empty()
                }
            };
            if closes {
                current_segment += 1;
                writes_in_current_segment = 0;
            }
        }
        let trailing_has_no_writes = writes_in_current_segment == 0;
        if trailing_has_no_writes && current_segment > 0 {
            for index in indexes.iter_mut() {
                if *index == current_segment {
                    *index = current_segment - 1;
                }
            }
        }
        indexes
    }

    /// 每段的写数（屏障不算）。
    pub fn segment_write_counts(operations: &[RecordedOperation]) -> Vec<usize> {
        let indexes = segment_index_of_each_operation(operations);
        let segment_count = indexes.iter().max().map_or(0, |last| last + 1);
        let mut counts = vec![0usize; segment_count];
        for (operation, segment) in operations.iter().zip(&indexes) {
            if operation.kind != OperationKind::Barrier {
                counts[*segment] += 1;
            }
        }
        counts
    }

    pub fn segment_sizes_text(counts: &[usize]) -> String {
        counts.iter().map(usize::to_string).collect::<Vec<String>>().join("+")
    }

    /// 两态闭式：1 + Σ(2^|段| − 1)。
    pub fn closed_form_state_count(counts: &[usize]) -> u128 {
        1 + counts.iter().map(|writes| (1u128 << writes) - 1).sum::<u128>()
    }

    /// 写表哈希丙：写记 `W <设备> <种类> <偏移> <长度> <字节 SHA-256> <步内段号>`，屏障记 `B <设备>`。
    pub fn write_table_hash_with_barriers(operations: &[RecordedOperation]) -> String {
        let indexes = segment_index_of_each_operation(operations);
        let mut text = String::new();
        for (operation, segment) in operations.iter().zip(&indexes) {
            match operation.kind {
                OperationKind::Barrier => text.push_str(&format!("B {}\n", operation.device)),
                OperationKind::Write | OperationKind::WriteForceUnitAccess | OperationKind::WriteZeroes => {
                    text.push_str(&format!(
                        "W {} {} {} {} {} {}\n",
                        operation.device,
                        operation.kind.serialized_name(),
                        operation.offset_in_bytes,
                        operation.length_in_bytes,
                        operation.content_sha256,
                        segment
                    ));
                }
            }
        }
        sha256_hexadecimal(text.as_bytes())
    }

    /// 写表哈希乙：每次写 `<设备> <偏移> <长度> <字节 SHA-256>`，屏障跳过，种类与 FUA 不进。
    pub fn write_table_hash_of_bytes(operations: &[RecordedOperation]) -> String {
        let mut text = String::new();
        for operation in operations.iter().filter(|operation| operation.kind != OperationKind::Barrier) {
            text.push_str(&format!(
                "{} {} {} {}\n",
                operation.device, operation.offset_in_bytes, operation.length_in_bytes, operation.content_sha256
            ));
        }
        sha256_hexadecimal(text.as_bytes())
    }

    /// 扇区终值表：（设备，扇区号）→ 内容摘要，按录制次序叠。清零写在这一步里出现时停机（跑前登记 S-probe），
    /// 只有 mkfs 那一段有清零，它的终值由探针的 mkfs 扇区表给。
    pub fn overlay_sectors(table: &mut BTreeMap<(u32, u64), String>, operations: &[RecordedOperation]) {
        for operation in operations {
            match operation.kind {
                OperationKind::Barrier => {}
                OperationKind::WriteZeroes => panic!("步里出现清零写：扇区终值表只收 mkfs 之后的普通写与 FUA 写（S-probe）"),
                OperationKind::Write | OperationKind::WriteForceUnitAccess => {
                    let first_sector = operation.offset_in_bytes / SECTOR_BYTES;
                    for (sector_offset, digest) in operation.sector_sha256s.iter().enumerate() {
                        table.insert((operation.device, first_sector + sector_offset as u64), digest.clone());
                    }
                }
            }
        }
    }

    pub fn sector_table_hash(table: &BTreeMap<(u32, u64), String>) -> String {
        let mut text = String::new();
        for ((device, sector), digest) in table {
            text.push_str(&format!("{device} {sector} {digest}\n"));
        }
        sha256_hexadecimal(text.as_bytes())
    }

    /// 写表哈希甲：这一步的写叠完后每个被写过的扇区的终值，按（设备，扇区）排序取 SHA-256。
    pub fn write_table_hash_of_final_sectors(operations: &[RecordedOperation]) -> String {
        let mut table = BTreeMap::new();
        overlay_sectors(&mut table, operations);
        sector_table_hash(&table)
    }
}
