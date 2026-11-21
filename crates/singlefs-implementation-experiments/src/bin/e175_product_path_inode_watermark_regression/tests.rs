use super::*;

/// 第一个 inode 号，抄自 `crates/singlefs-core/src/transaction.rs` 的 `FIRST_INODE_NUMBER`，同一条用例回比。
const FIRST_INODE_NUMBER_COPIED_FROM_THE_TRANSACTION_MODULE: u64 = 1;
/// D8（核心索引结构） 已定项 6「新池新建文件写」：一条记录（inode 1、出生代 3、……）；记账里一条「inode 号水位 = 2」。
/// 条款给的锚点，不读 `crates/`（登记第七节 A1、A2）。
pub(super) const FIRST_FILE_INODE_NUMBER_IN_THE_CLAUSE: u64 = 1;
pub(super) const FIRST_FILE_OBJECT_BIRTH_IN_THE_CLAUSE: u64 = 3;
pub(super) const WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE: u64 = 2;

/// 单测判哪一臂：没设 `E175_ARM` 是产品臂（仓里这棵树没施加任何补丁）；副本里施加了变体补丁的设它。
pub(super) fn arm_of_this_tree() -> Arm {
    match std::env::var("E175_ARM") {
        Ok(value) => Arm::from_variable_value(Some(&value)),
        Err(_) => Arm::Product,
    }
}

/// 自证那几条单测读 `E175_BREAK`（文件头那张表）。
pub(super) fn breakage_of_this_process() -> Breakage {
    Breakage::from_variable_value(std::env::var("E175_BREAK").ok().as_deref())
}

// ───────────── 常量回比、规模、纯函数、D 族钉值（A1–A7）、多线程与单线程对拍 ─────────────

#[test]
fn result_files_from_two_code_baselines_or_without_a_header_are_refused_by_the_summary() {
    let result_text_of_code_baseline_aaaa = code_baseline_header("aaaa")
        + "\nE7RESULT name=history family=F2 arm=P index=0 line_end=ok\n";
    let result_text_of_code_baseline_bbbb = code_baseline_header("bbbb")
        + "\nE7RESULT name=history family=F2 arm=P index=1 line_end=ok\n";
    let same = [
        ("a1".to_string(), result_text_of_code_baseline_aaaa.clone()),
        ("a2".to_string(), result_text_of_code_baseline_aaaa.clone()),
    ];
    assert_eq!(
        single_code_baseline_of_result_texts(&same),
        Ok("aaaa".to_string())
    );
    let mixed = [
        ("a1".to_string(), result_text_of_code_baseline_aaaa.clone()),
        ("b1".to_string(), result_text_of_code_baseline_bbbb),
    ];
    assert!(
        single_code_baseline_of_result_texts(&mixed).is_err(),
        "两份代码混用必须拒"
    );
    let headless = [
        ("a1".to_string(), result_text_of_code_baseline_aaaa),
        ("bare".to_string(), "E7RESULT name=history\n".to_string()),
    ];
    assert!(
        single_code_baseline_of_result_texts(&headless).is_err(),
        "没有文件头必须拒"
    );
}

#[test]
fn seed_base_and_first_inode_number_match_the_crates() {
    assert_eq!(
        SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE,
        singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        "本地抄的种子基与 history.rs 的同名常量不同"
    );
    assert_eq!(
        FIRST_INODE_NUMBER_COPIED_FROM_THE_TRANSACTION_MODULE,
        singlefs_core::transaction::FIRST_INODE_NUMBER,
        "本地抄的第一个 inode 号与 transaction.rs 的同名常量不同"
    );
}

#[test]
fn invariant_names_match_the_checker() {
    assert_eq!(
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER_NAME,
        singlefs_checker::image::ONE_OBJECT_BIRTH_PER_INODE_NUMBER
    );
}

// ───────────── 入口：命令行、环境变量怎么定范围、结果文件怎么写 ─────────────

/// 单测的读变量表：不动进程环境，单测并行时互不相扰。
fn variables_of(table: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let owned_table: BTreeMap<String, String> = table
        .iter()
        .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
        .collect();
    move |name: &str| owned_table.get(name).cloned()
}

/// 这条单测自己的临时文件：放系统临时目录、名字带进程号与用例名，用完删。
pub(super) struct ScratchFile(pub(super) PathBuf);

impl ScratchFile {
    pub(super) fn named_for_the_test(test_name: &str) -> ScratchFile {
        let path =
            std::env::temp_dir().join(format!("e175-{}-{test_name}.out", std::process::id()));
        std::fs::remove_file(&path).ok();
        ScratchFile(path)
    }

    pub(super) fn path_text(&self) -> String {
        self.0.display().to_string()
    }
}

impl Drop for ScratchFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}

fn lines_with_prefix(text: &str, prefix: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with(prefix))
        .map(str::to_string)
        .collect()
}

#[test]
fn no_arguments_run_the_families_summarize_writes_the_summary_and_anything_else_is_refused() {
    let words = |list: &[&str]| {
        list.iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>()
    };
    assert_eq!(
        command_line_mode_of(&words(&[])),
        Some(CommandLineMode::RunTheFamilies)
    );
    assert_eq!(
        command_line_mode_of(&words(&["summarize"])),
        Some(CommandLineMode::SummarizeTheResultFiles)
    );
    assert_eq!(command_line_mode_of(&words(&["--exact"])), None);
    assert_eq!(
        command_line_mode_of(&words(&["summarize", "summarize"])),
        None
    );
}

#[test]
fn run_request_takes_its_whole_scope_from_the_variables() {
    let request = RunRequest::from_variables(&variables_of(&[
        ("E175_ARM", "P"),
        ("E175_FAMILY", "D"),
        ("E175_RESULT_FILE", "/result/of/the/run.out"),
        ("E175_WORKER_THREADS", "3"),
        ("E175_TIME_BUDGET_SECONDS", "7"),
        ("E175_INDICES", "1, 5"),
        ("E175_INDEX_LIMIT", "6"),
        ("E175_CODE_BASELINE", "baseline-text"),
    ]));
    assert_eq!(
        request,
        RunRequest {
            arm: Arm::Product,
            families: vec![Family::Directed],
            result_file: PathBuf::from("/result/of/the/run.out"),
            worker_threads: 3,
            time_budget: Some(Duration::from_secs(7)),
            explicit_indices: Some(vec![1, 5]),
            index_limit: Some(6),
            code_baseline: "baseline-text".to_string(),
        }
    );
}

#[test]
fn run_request_without_the_optional_variables_means_the_registered_whole_range() {
    let request = RunRequest::from_variables(&variables_of(&[
        ("E175_ARM", "V2"),
        ("E175_RESULT_FILE", "/result/of/the/run.out"),
        ("E175_WORKER_THREADS", "1"),
    ]));
    assert_eq!(request.arm, Arm::PublishDoesNotAddTheNewInodes);
    assert_eq!(request.time_budget, None);
    assert_eq!(request.explicit_indices, None);
    assert_eq!(request.index_limit, None);
    assert_eq!(request.code_baseline, "not-given");
    let labels: Vec<&str> = request
        .families
        .iter()
        .map(|family| family.label())
        .collect();
    assert_eq!(
        labels,
        [
            "D",
            "F3a",
            "F2",
            "F1-broad",
            "F1-reuse-after-raising-the-floor",
            "F1-deleting-files-of-several-lower-leaves",
            "F1-newest-system-configuration-slot-reading-zeros",
        ],
        "V2 臂上第八节几何四点都登记 0 段"
    );
}

#[test]
#[should_panic(expected = "E175_RESULT_FILE 没设")]
fn run_request_without_a_result_file_is_refused() {
    let _ = RunRequest::from_variables(&variables_of(&[("E175_ARM", "P")]));
}

#[test]
#[should_panic(expected = "E175_ARM 没设")]
fn run_request_without_an_arm_is_refused() {
    let _ = RunRequest::from_variables(&variables_of(&[(
        "E175_RESULT_FILE",
        "/result/of/the/run.out",
    )]));
}

#[test]
#[should_panic(expected = "大规模不许开弄坏开关")]
fn run_request_with_a_breakage_switch_is_refused() {
    let _ = RunRequest::from_variables(&variables_of(&[
        ("E175_ARM", "P"),
        ("E175_RESULT_FILE", "/result/of/the/run.out"),
        ("E175_BREAK", "ledger-ignores-birth"),
    ]));
}

/// 复跑表登记的 `E175|@driver_e175` 那一行比的留存产物（`git show 59ce5a865^:research/results/e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out`），去掉驱动补的 `name=done` 行。
const REGISTERED_RESULT_LINES_OF_THE_DIRECTED_FAMILY_ON_THE_PRODUCT_ARM: &str = r#"E7RESULT name=history family=D arm=P index=0 label=D1 ending=completed steps_observed=1 issuances=1 issuances_at_operations=0 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=1 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=1 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=2 watermark_final=2 no_watermark_steps=0 q4=0 q5=0 q6=0 q6_judged=0 q7=0 q10_issuances_at_operations=0 q11=0 rollbacks_applied=0 rollback_targets_not_in_previous_ring=0 creations_applied=0 mounts_or_recoveries_applied=0 s1_root_observations=4 s1_unreadable=0 s1_rebuilds=4 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[] line_end=ok
E7RESULT name=history family=D arm=P index=1 label=D2(1) ending=completed steps_observed=6 issuances=4 issuances_at_operations=3 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=6 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=6 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=5 watermark_final=5 no_watermark_steps=0 q4=1 q5=1 q6=0 q6_judged=1 q7=0 q10_issuances_at_operations=3 q11=0 rollbacks_applied=1 rollback_targets_not_in_previous_ring=0 creations_applied=3 mounts_or_recoveries_applied=0 s1_root_observations=44 s1_unreadable=0 s1_rebuilds=10 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,create,rollback(RingRoot{index_from_newest:1}),create] line_end=ok
E7RESULT name=history family=D arm=P index=2 label=D2(2) ending=completed steps_observed=6 issuances=4 issuances_at_operations=3 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=6 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=6 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=5 watermark_final=5 no_watermark_steps=0 q4=1 q5=1 q6=0 q6_judged=1 q7=0 q10_issuances_at_operations=3 q11=0 rollbacks_applied=1 rollback_targets_not_in_previous_ring=0 creations_applied=3 mounts_or_recoveries_applied=0 s1_root_observations=44 s1_unreadable=0 s1_rebuilds=10 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,create,rollback(RingRoot{index_from_newest:2}),create] line_end=ok
E7RESULT name=history family=D arm=P index=3 label=D2(3) ending=completed steps_observed=6 issuances=4 issuances_at_operations=3 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=6 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=6 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=5 watermark_final=5 no_watermark_steps=0 q4=1 q5=1 q6=0 q6_judged=1 q7=0 q10_issuances_at_operations=3 q11=0 rollbacks_applied=1 rollback_targets_not_in_previous_ring=0 creations_applied=3 mounts_or_recoveries_applied=0 s1_root_observations=45 s1_unreadable=0 s1_rebuilds=11 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,create,rollback(RingRoot{index_from_newest:3}),create] line_end=ok
E7RESULT name=history family=D arm=P index=4 label=D2(4) ending=completed steps_observed=6 issuances=3 issuances_at_operations=2 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=6 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=6 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=4 watermark_final=4 no_watermark_steps=0 q4=1 q5=0 q6=0 q6_judged=1 q7=0 q10_issuances_at_operations=2 q11=0 rollbacks_applied=1 rollback_targets_not_in_previous_ring=0 creations_applied=2 mounts_or_recoveries_applied=0 s1_root_observations=43 s1_unreadable=0 s1_rebuilds=9 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,create,rollback(RingRoot{index_from_newest:4}),create] line_end=ok
E7RESULT name=history family=D arm=P index=5 label=D2(5) ending=completed steps_observed=6 issuances=4 issuances_at_operations=3 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=6 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=6 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=5 watermark_final=5 no_watermark_steps=0 q4=0 q5=0 q6=0 q6_judged=0 q7=0 q10_issuances_at_operations=3 q11=0 rollbacks_applied=0 rollback_targets_not_in_previous_ring=0 creations_applied=3 mounts_or_recoveries_applied=0 s1_root_observations=42 s1_unreadable=0 s1_rebuilds=9 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,create,rollback(RingRoot{index_from_newest:5}),create] line_end=ok
E7RESULT name=history family=D arm=P index=6 label=D2(6) ending=completed steps_observed=6 issuances=4 issuances_at_operations=3 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=6 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=6 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=5 watermark_final=5 no_watermark_steps=0 q4=0 q5=0 q6=0 q6_judged=0 q7=0 q10_issuances_at_operations=3 q11=0 rollbacks_applied=0 rollback_targets_not_in_previous_ring=0 creations_applied=3 mounts_or_recoveries_applied=0 s1_root_observations=42 s1_unreadable=0 s1_rebuilds=9 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,create,rollback(RingRoot{index_from_newest:6}),create] line_end=ok
E7RESULT name=history family=D arm=P index=7 label=D3 ending=completed steps_observed=8 issuances=4 issuances_at_operations=3 q1=0 q3b=0 q9_still=0 q9_gone=0 q2_one_birth_violated=0 i9_29_holds=8 i9_29_not_applicable=0 i9_29_partially_judged=0 i9_29_violated_steps=0 i9_29_missing=0 i9_6_holds=8 i9_6_not_applicable=0 i9_6_partially_judged=0 i9_6_violated_steps=0 i9_6_missing=0 q3_steps=0 q3_peak=0 q3_final=0 watermark_peak=5 watermark_final=5 no_watermark_steps=0 q4=0 q5=0 q6=0 q6_judged=0 q7=0 q10_issuances_at_operations=3 q11=0 rollbacks_applied=0 rollback_targets_not_in_previous_ring=0 creations_applied=3 mounts_or_recoveries_applied=3 s1_root_observations=71 s1_unreadable=0 s1_rebuilds=13 s1_identity_conflicts=0 s3_disagreements=0 s3_table_unreadable_steps=0 f3_still_but_holds=0 f3_still_but_partial=0 f3_violated_without_two_births=0 system_configuration_unchosen_steps=0 first_violation=none first_s3_disagreement=none first_q11=none operations=[unlink,create,mount,create,cold,mount,create] line_end=ok
E7RESULT name=family_complete family=D arm=P histories=8
"#;

/// 副本里施加了变体补丁的树不是这份产物的来源，这一条只在产品臂的树上成立。
#[test]
fn directed_family_on_the_product_arm_writes_the_registered_result_lines_byte_for_byte() {
    let result_file = ScratchFile::named_for_the_test("directed-family-registered-lines");
    let path_text = result_file.path_text();
    run_families_into_the_result_file(&RunRequest::from_variables(&variables_of(&[
        ("E175_ARM", "P"),
        ("E175_FAMILY", "D"),
        ("E175_RESULT_FILE", path_text.as_str()),
        ("E175_WORKER_THREADS", "3"),
    ])));
    let text = std::fs::read_to_string(&result_file.0).expect("结果文件写出来了");
    assert_eq!(text.lines().next(), Some("# code_baseline=not-given"));
    let result_lines = lines_with_prefix(&text, "E7RESULT ").join("\n") + "\n";
    assert_eq!(
        result_lines,
        REGISTERED_RESULT_LINES_OF_THE_DIRECTED_FAMILY_ON_THE_PRODUCT_ARM
    );
}

#[test]
fn second_run_on_the_same_result_file_only_evaluates_what_is_missing() {
    let result_file = ScratchFile::named_for_the_test("resume-on-the-same-file");
    let path_text = result_file.path_text();
    let variables_with_the_limit = [
        ("E175_ARM", "P"),
        ("E175_FAMILY", "D"),
        ("E175_RESULT_FILE", path_text.as_str()),
        ("E175_WORKER_THREADS", "2"),
        ("E175_INDEX_LIMIT", "1"),
    ];
    run_families_into_the_result_file(&RunRequest::from_variables(&variables_of(
        &variables_with_the_limit,
    )));
    let text_after_the_narrowed_run =
        std::fs::read_to_string(&result_file.0).expect("结果文件写出来了");
    assert_eq!(
        lines_with_prefix(&text_after_the_narrowed_run, "E7RESULT name=history ").len(),
        1
    );
    assert_eq!(
        lines_with_prefix(
            &text_after_the_narrowed_run,
            "E7RESULT name=family_complete "
        )
        .len(),
        0,
        "缩域那一趟不写整族完成"
    );

    run_families_into_the_result_file(&RunRequest::from_variables(&variables_of(
        &variables_with_the_limit[..4],
    )));
    let text = std::fs::read_to_string(&result_file.0).expect("结果文件还在");
    assert_eq!(
        lines_with_prefix(&text, "# code_baseline=").len(),
        1,
        "续写不再写文件头"
    );
    assert_eq!(
        lines_with_prefix(&text, "E7RESULT name=history ").len(),
        8,
        "序号 0 不重跑，整族一共 8 段"
    );
    assert_eq!(
        lines_with_prefix(&text, "E7RESULT name=family_complete "),
        ["E7RESULT name=family_complete family=D arm=P histories=8"]
    );
    let chunk_lines = lines_with_prefix(&text, "E7TIMING name=chunk ");
    assert_eq!(chunk_lines.len(), 2);
    for (chunk_line, evaluated_field) in chunk_lines.iter().zip(["evaluated=1", "evaluated=7"]) {
        assert!(
            chunk_line.contains(" worker_threads=2 "),
            "切片并行照点名的线程数：{chunk_line}"
        );
        assert!(chunk_line.contains(evaluated_field), "{chunk_line}");
    }
}
