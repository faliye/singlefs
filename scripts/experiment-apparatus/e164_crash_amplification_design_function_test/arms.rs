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

const PAYLOAD_CHECKSUM_OFFSET_VARIANT_TEST: (&str, &str, &str) = ("singlefs-verification", "crates/singlefs-verification/tests/new_pool_file_creation_publish.rs", "root_slots_system_configurations_and_journal_ring_hold_the_published_state");
const FORMAT_CONSTANT_LITERAL_VARIANT_TEST: &str = "pointer_record_and_entry_width_literals";
const MODEL_MODULE_STANDARD_LIBRARY_ONLY_TEST: &str = "the_model_module_uses_only_the_standard_library_and_the_format_constants";

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
    payload_checksum_offset_variant_node: [String; 3],
    format_constant_literal_variant_node: [String; 3],
    harness_library_not_doing: String,
    harness_library_doing: String,
    probe_node: String,
    named_exit: BTreeMap<&'static str, i32>,
    harness_library_outcomes: BTreeMap<String, String>,
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
    let harness_files = closure_source_files(&copy, &graph, "singlefs-verification");
    let mut payload_checksum_offset_variant_files = harness_files.clone();
    payload_checksum_offset_variant_files.extend(target_files(&copy, PAYLOAD_CHECKSUM_OFFSET_VARIANT_TEST.1));
    let format_files = closure_source_files(&copy, &graph, "singlefs-format");
    let own_source = own_source_files(&copy, &package_source_files(&copy, "crates/singlefs-verification"));
    let mut probe_files = harness_files.clone();
    probe_files.extend(target_files(&copy, PROBE_TEST_FILE));
    let probe_run = cargo_test(&copy, &["-p", "singlefs-verification", "--test", "e164_probe", "--", "--nocapture", "--test-threads=1"]);
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
        let second = cargo_test(&copy, &["-p", "singlefs-verification", "--test", "e164_probe", "--", "--nocapture", "--test-threads=1", "pid_probe"]);
        pid_lines.push(result_line(&second.stdout, "pid"));
    }
    let mut named_exit = BTreeMap::new();
    if matches!(name, "V0" | "V5") {
        named_exit.insert("v5_test", cargo_test(&copy, &["-p", PAYLOAD_CHECKSUM_OFFSET_VARIANT_TEST.0, "--test", "new_pool_file_creation_publish", "--", PAYLOAD_CHECKSUM_OFFSET_VARIANT_TEST.2]).exit_code);
    }
    if matches!(name, "V0" | "V11") {
        named_exit.insert("v11_test", cargo_test(&copy, &["-p", "singlefs-format", "--lib", "--", FORMAT_CONSTANT_LITERAL_VARIANT_TEST]).exit_code);
    }
    let model_source = std::fs::read_to_string(copy.join("crates/singlefs-verification/src/model_comparison.rs")).unwrap_or_default();
    if !model_source.contains(MODEL_MODULE_STANDARD_LIBRARY_ONLY_TEST) {
        stop(&format!("F6 那一条测试 {MODEL_MODULE_STANDARD_LIBRARY_ONLY_TEST} 在副本里找不到（S-probe）"));
    }
    let mut harness_library_outcomes = BTreeMap::new();
    if matches!(name, "V0" | "V4" | "V8") {
        let run = cargo_test(&copy, &["-p", "singlefs-verification", "--lib"]);
        harness_library_outcomes = test_outcomes(&run.stdout);
        named_exit.insert("harness_lib", run.exit_code);
        named_exit.insert("f6_test", cargo_test(&copy, &["-p", "singlefs-verification", "--lib", "--", MODEL_MODULE_STANDARD_LIBRARY_ONLY_TEST]).exit_code);
    }
    VariantObservation {
        histories,
        panic_line: result_line(&probe_run.stdout, "panic"),
        pid_lines,
        recording_code: digest_by_crate_closure(&copy, &closure_source_files(&copy, &graph, "singlefs-core"), &BTreeSet::new(), &rustc_text),
        judging_not_doing: packages_lexical_digest(&copy, &["crates/singlefs-checker", "crates/singlefs-format"]),
        judging_doing: packages_lexical_digest(&copy, &["crates/singlefs-checker", "crates/singlefs-format", "crates/singlefs-verification"]),
        payload_checksum_offset_variant_node: three_digests(&copy, &payload_checksum_offset_variant_files, &rustc_text),
        format_constant_literal_variant_node: three_digests(&copy, &format_files, &rustc_text),
        harness_library_not_doing: digest_by_crate_closure(&copy, &harness_files, &BTreeSet::new(), &rustc_text),
        harness_library_doing: digest_by_crate_closure(&copy, &harness_files, &own_source, &rustc_text),
        probe_node: digest_by_crate_closure(&copy, &probe_files, &BTreeSet::new(), &rustc_text),
        named_exit,
        harness_library_outcomes,
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
    let harness_library_patch_observation = &observations["V4"];
    let differing_tests: Vec<&String> = base.harness_library_outcomes.iter().filter(|(test, outcome)| harness_library_patch_observation.harness_library_outcomes.get(*test) != Some(outcome)).map(|(test, _)| test).collect();
    suffix.emit(&format!(
        "E7RESULT name=q3_lib tests_in_v0={} tests_in_v4={} differing_outcomes={} differing={}",
        base.harness_library_outcomes.len(), harness_library_patch_observation.harness_library_outcomes.len(), differing_tests.len(),
        if differing_tests.is_empty() { "none".to_string() } else { differing_tests.iter().map(|test| test.as_str()).collect::<Vec<&str>>().join(",") }
    ));
    let methods = ["by_function", "by_walked_files", "by_crate_closure"];
    let mut misses = [0usize; 3];
    for (name, key) in [("V5", "v5_test"), ("V11", "v11_test")] {
        let observation = &observations[name];
        let (base_node, variant_node) = if name == "V5" { (&base.payload_checksum_offset_variant_node, &observation.payload_checksum_offset_variant_node) } else { (&base.format_constant_literal_variant_node, &observation.format_constant_literal_variant_node) };
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
    let comment_only_model_patch_observation = &observations["V8"];
    let base_exit = base.named_exit.get("f6_test").copied().unwrap_or(-1);
    let comment_only_model_patch_exit = comment_only_model_patch_observation.named_exit.get("f6_test").copied().unwrap_or(-1);
    let exit_changed = base_exit != comment_only_model_patch_exit;
    let not_doing_changed = changed(&base.harness_library_not_doing, &comment_only_model_patch_observation.harness_library_not_doing);
    let doing_changed = changed(&base.harness_library_doing, &comment_only_model_patch_observation.harness_library_doing);
    suffix.emit(&format!(
        "E7RESULT name=q6 base_exit={base_exit} v8_exit={comment_only_model_patch_exit} exit_changed={exit_changed} not_doing_digest_changed={not_doing_changed} doing_digest_changed={doing_changed} not_doing_miss={} doing_miss={} doing_changed_on_v6={}",
        u8::from(exit_changed && !not_doing_changed), u8::from(exit_changed && !doing_changed), changed(&base.harness_library_doing, &observations["V6"].harness_library_doing)
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
            changed(&base.payload_checksum_offset_variant_node[2], &observation.payload_checksum_offset_variant_node[2]),
            changed(&base.format_constant_literal_variant_node[2], &observation.format_constant_literal_variant_node[2]),
            changed(&base.harness_library_doing, &observation.harness_library_doing),
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
