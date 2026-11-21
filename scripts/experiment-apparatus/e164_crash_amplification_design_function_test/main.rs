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

// ============================== 模块（内联：mutate.sh 只改这一份源文件） ==============================
mod arms;
mod common;
mod fingerprints;
mod json;
mod lexical;
mod node_code;
mod pipeline;
mod repository_scans;
mod repository_scans_crate_closure_reruns;
mod repository_scans_frozen_file_names;
#[cfg(test)]
mod tests;
mod variants;
mod write_tables;


use common::{run_allowed, sha256_hexadecimal, strings};
use pipeline::{claim_forms, explore, geometries, pipeline_forms, Geometry, ModelForm, ModelReport};
use write_tables::{
    append_with_barrier_merging, closed_form_state_count, segment_write_counts, write_table_hash_of_bytes,
    write_table_hash_of_final_sectors, write_table_hash_with_barriers, OperationKind, RecordedOperation,
};

const LABEL_COMPILATION_WEAK_DESIGN: &str = include_str!("label_compilation_design_weak.rs");
const LABEL_COMPILATION_STRICT_DESIGN: &str = include_str!("label_compilation_design_strict.rs");
const LABEL_COMPILATION_WRITINGS: &str = include_str!("label_compilation_writings.rs");

/// 仓根：`E164_REPOSITORY_ROOT` 设了就用它（mutate.sh 只拷 research/，副本里没有 .git 与 crates/），否则取装置往上两层。
/// 没设环境变量时从本源文件（清单目录里点名的一份文件，字节进单元键）往上取仓根：`repository`、`variants` 与单测读的是真仓，
/// 但同一个二进制的 `models` 行要有单元键（目录直接拼出仓根会被当运行期路径拒算）。
pub fn repository_root() -> PathBuf {
    if let Ok(root) = std::env::var("E164_REPOSITORY_ROOT") {
        return PathBuf::from(root);
    }
    let this_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e164_crash_amplification_design_function_test/main.rs");
    this_source.ancestors().nth(4).expect("本源文件在 <根>/scripts/experiment-apparatus/ 下").canonicalize().expect("仓根在 scripts/experiment-apparatus 往上两层")
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
        repository_scans::identity_text("crates/singlefs-verification/tests/e164_probe.rs", "records_h_long", 3, "ab"),
        "crates/singlefs-verification/tests/e164_probe.rs::records_h_long::3::ab",
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
fn label_compilation_writings() -> Vec<(u32, String)> {
    let mut writings: Vec<(u32, String)> = Vec::new();
    for line in LABEL_COMPILATION_WRITINGS.lines() {
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
    for (form, design) in [("weak", LABEL_COMPILATION_WEAK_DESIGN), ("strict", LABEL_COMPILATION_STRICT_DESIGN)] {
        for (number, body) in label_compilation_writings() {
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
    // 准入脚本与本源文件：设了 E164_REPOSITORY_ROOT（mutate.sh 的副本）取那边，否则从本源文件（单元键里点名的一份已入库文件）往上取：
    // 准入脚本在 .gitignore 的 .claude/singlefs-ai-sop 下，不直接拼进 env! 那一处（临时树里没有它，单元键会拒算）。
    let own_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e164_crash_amplification_design_function_test/main.rs");
    let (source, script) = match std::env::var("E164_REPOSITORY_ROOT") {
        Ok(root) => (
            PathBuf::from(&root).join("scripts/experiment-apparatus/e164_crash_amplification_design_function_test/main.rs"),
            PathBuf::from(&root).join(".claude/singlefs-ai-sop/scripts/preflight.py"),
        ),
        Err(_) => {
            let script = own_source.parent().expect("源文件有父目录").join("../../../.claude/singlefs-ai-sop/scripts/preflight.py");
            (own_source, script)
        }
    };
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
