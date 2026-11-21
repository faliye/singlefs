//! E175（产品路径上 inode 号水位退回与重发号）的装置。产品路径 = 一次都不调 `PoolWriter::set_inode_number_watermark_source`、不加 `SharedFaultPlan` 注入。
//!
//! 臂由跑的一方在副本里施加补丁之后用 `E175_ARM` 告诉装置（`P`、`V1`、`V2`、`V717`）；装置不读源码判臂，
//! 施加前后的 `sha256sum` 由跑的一方记进产物。
//!
//! 三样读法（跑前登记 `research/prompts/e175-preregistration.md` 第六节）：S1 账 = `recovery::readable_roots` 列可读根、`recovery::rebuild_version` 重建每条根那一版；
//! S2 = `singlefs_checker::walk::check_pool_image`；S3 被崩溃恢复抛弃的根两条路（路 1 现行实例表、路 2 步账）。
//!
//! 入口：不带参数跑大规模；`summarize` 写汇总；别的参数退 2。
//!
//! 跑大规模读这几个环境变量：`E175_ARM`（必给：它说的是这棵树施加的是哪一臂的补丁）、
//! `E175_RESULT_FILE`（必给：结果行写到哪）、
//! `E175_FAMILY`（不给就是这一臂登记了段的全部族；给了是指定范围。D、F3a、F2；F1 四组
//! `F1-broad`、`F1-reuse-after-raising-the-floor`、`F1-deleting-files-of-several-lower-leaves`、
//! `F1-newest-system-configuration-slot-reading-zeros` 与第八节几何点 `G-long`、`G-small`、`G-draw`、`G-long-F1`）、
//! `E175_WORKER_THREADS`（不给取每台默认重活线程数）、`E175_TIME_BUDGET_SECONDS`（不给就跑完；给了到点不再领新的历史，领了的跑完，同一条命令再跑就是续跑，这一趟是缩域）、
//! 可选 `E175_INDICES`（逗号分隔，只跑这几段：F6 复跑与试跑用）、可选 `E175_INDEX_LIMIT`（只跑序号小于它的：试跑用）。
//! `E175_INDICES`、`E175_INDEX_LIMIT`、`E175_TIME_BUDGET_SECONDS` 都不设就是登记的全部段；设了任一个就是缩域，缩过的那一趟不写 `family_complete` 行，汇总里这一族写 `complete=no`，缩过的结果不当整族的产物。
//! 汇总读 `E175_SUMMARY_INPUTS`（逗号分隔的结果文件；不给就先照上面那几个变量跑一趟大规模，汇总它的结果文件）写到 `E175_SUMMARY_FILE`（必给，已有就拒）。
//!
//! 代码标记（登记第十二节修订 2 第 2 条）：`E175_CODE_BASELINE` = 冻结副本清单 `baseline-frozen-crates.sha` 的 sha256。
//! 大规模在新结果文件的第一行写 `# code_baseline=<值>`（不是 `E7RESULT` 行，复跑比对不看它），续写已有文件时第一行
//! 必须是同一个值，不同就拒（两份代码混用）；没设时写 `not-given`（只给 `scripts/experiment-apparatus/replay.sh` 的 `driver_e175` 复跑用，
//! 产物一律设）。汇总要求每份输入的第一行都是同一个 `code_baseline`、且等于 `E175_CODE_BASELINE`，
//! 汇总文件第一行写同一个文件头，每一行汇总行末尾带 `code_baseline=<值>`。
//!
//! 弄坏开关 `E175_BREAK` 只给单测的自证用，跑大规模见到它就拒。没设时单测判产品臂，副本里施加了变体补丁的设 `E175_ARM` 说明是哪一臂。三个开关各弄坏什么、打开后哪条单测红：
//! `ledger-ignores-birth`：S1 账按号去重、不看出生代，V1 臂上 `rollback_one_step_then_create_matches_the_arm` 红；
//! `every-root-abandoned`：S3 给发号事件分类时每条持有旧出生代的根都判成被崩溃恢复抛弃（Q1 / Q3b 与 Q9 用的那一处；择现行那一版与两路比对不动），同一条红（Q1 由 1 变 0、Q3b 由 0 变 1）；
//! `q9-always-still-present`：Q9 分类一律判「还在」，`older_birth_candidacy_says_gone_when_no_holder_is_readable` 红。

// admission: always 复跑登记要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: none 只读环境变量、写点名的结果文件，除了编好的程序本身没有环境要求

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::instance_table::InstanceTableRecords;
use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_of_root,
    publish_is_abandoned_by_the_instance_table, readable_roots, rebuild_version, scan_journal,
    RebuiltVersion,
};
use singlefs_core::root_record::RootRecord;
use singlefs_verification::history::{
    execute_history_with, AppliedEffect, ContentChoice, ContentLength, FloorTargetChoice,
    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    NamedGenerationWeights, OverwriteContentDraw, PerStepChecker, RollbackTargetChoice,
    RollbackTargetDraw, StepObservation, StepOutcome, StepPosition,
};
use singlefs_verification::memory_pool::MemoryPool;
use singlefs_verification::SharedStream;

mod arms_and_families;
mod evaluation_of_a_history;
mod histories_of_the_families;
mod history_account;
mod roots_abandoned_by_crash_recovery;
mod summary_of_the_result_files;

use arms_and_families::*;
use evaluation_of_a_history::*;
use histories_of_the_families::*;
use history_account::*;
use roots_abandoned_by_crash_recovery::*;
use summary_of_the_result_files::*;

/// 种子基，抄自 `crates/singlefs-verification/src/history.rs` 的 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`（登记第五节「装置要读 crates/ 的常量」）；
/// `seed_base_and_first_inode_number_match_the_crates` 拿它与实现侧同名常量回比。
const SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE: u64 = 7_463_871_032_432_355_113;
/// 被数的两条不变量的编号（`singlefs_checker::image` 里的名字；`invariant_names_match_the_checker` 回比 I-9.29）。
const ONE_OBJECT_BIRTH_PER_INODE_NUMBER_NAME: &str = "I-9.29";
const WATERMARK_ABOVE_BOTH_HIGHEST_NUMBERS_NAME: &str = "I-9.6";

/// F3a 的规模（登记第五节「历史族」）：中段长 0..=4（P 臂）、0..=3（V 臂），字母表 11 个。
const DIRECTED_EXHAUSTIVE_ALPHABET_SIZE: u64 = 11;
const DIRECTED_EXHAUSTIVE_LONGEST_MIDDLE_ON_THE_PRODUCT_ARM: u32 = 4;
const DIRECTED_EXHAUSTIVE_LONGEST_MIDDLE_ON_THE_VARIANT_ARMS: u32 = 3;
/// F2 的规模：P 臂 256 种子 × 60 步，V 臂前 64 个种子。
const RANDOM_WITH_ROLLBACKS_SEEDS_ON_THE_PRODUCT_ARM: u64 = 256;
const RANDOM_WITH_ROLLBACKS_SEEDS_ON_THE_VARIANT_ARMS: u64 = 64;
const RANDOM_WITH_ROLLBACKS_OPERATIONS_PER_HISTORY: usize = 60;
/// F1 的规模（登记第五节 F1 那一行）：每组 P 臂 128 种子 × 30 步，V 臂每组前 32 个种子。
const EXISTING_RANDOM_SEEDS_ON_THE_PRODUCT_ARM: u64 = 128;
const EXISTING_RANDOM_SEEDS_ON_THE_VARIANT_ARMS: u64 = 32;
const EXISTING_RANDOM_OPERATIONS_PER_HISTORY: usize = 30;
/// 第八节几何点的规模：G-long 与 G-long-F1 每段 150 步；G-long P 64 种子、V1 32 种子；G-small、G-draw P 各 64 种子；
/// G-long-F1 P 32 种子。表里写「不跑」的那几格登记 0 段。
const LONG_GEOMETRY_OPERATIONS_PER_HISTORY: usize = 150;
const LONGER_RANDOM_SEEDS_ON_THE_PRODUCT_ARM: u64 = 64;
const LONGER_RANDOM_SEEDS_ON_THE_ROLLBACK_VARIANT_ARM: u64 = 32;
const SMALL_DEVICE_RANDOM_SEEDS_ON_THE_PRODUCT_ARM: u64 = 64;
const RECENT_DRAW_RANDOM_SEEDS_ON_THE_PRODUCT_ARM: u64 = 64;
const LONGER_BROAD_SEEDS_ON_THE_PRODUCT_ARM: u64 = 32;

/// 弄坏开关（文件头那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Breakage {
    ledger_ignores_birth: bool,
    every_root_abandoned: bool,
    older_birth_always_still_present: bool,
}

/// 一段要跑的历史：族、序号、给人看的标签（D 的名字、F3a 的中段字母、F2 的种子）与历史本身。
struct HistoryItem {
    family: Family,
    index: u64,
    label: String,
    history: GeneratedHistory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RootFacts {
    identity: RootIdentity,
    rollback_floor: u64,
    version: VersionFacts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HolderAtThisStep {
    readable_now: bool,
    abandoned_now: bool,
    txg: u64,
}

/// 一次发号碰上账里同号的另一个出生代（Q1 或 Q3b 的一个事件）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReissueEvent {
    position: String,
    inode: u64,
    new_birth: u64,
    older_births: Vec<u64>,
    /// 持有旧出生代的根里至少一条不被崩溃恢复抛弃（Q1）；否则是跨被抛弃时间线（Q3b）。
    same_timeline: bool,
    older_birth_candidacy: OlderBirthStillInTheCandidateSet,
    /// 同一步 S2 对 I-9.29 的判定（四栏之一）。
    one_birth_per_number_at_this_step: VerdictClass,
}

/// 四栏加「缺」的步数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct VerdictColumns {
    holds: u64,
    not_applicable: u64,
    partially_judged: u64,
    violated: u64,
    missing: u64,
}

/// 一段历史里各量的计数（登记第六节「量」那张表；名字照表里的量号）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HistoryCounts {
    steps_observed: u64,
    issuances: u64,
    issuances_at_operations: u64,
    watermark_regression_steps: u64,
    watermark_regression_peak: u64,
    watermark_regression_final: u64,
    watermark_peak: u64,
    watermark_final: u64,
    steps_whose_current_version_has_no_watermark: u64,
    rollbacks_across_issued_numbers: u64,
    windows_with_a_creation_after_such_a_rollback: u64,
    rollbacks_whose_ring_maximum_is_below_the_current: u64,
    rollbacks_judged_against_the_ring_maximum: u64,
    mounts_landing_on_a_lower_watermark: u64,
    creations_not_issuing_the_previous_watermark: u64,
    rollbacks_applied: u64,
    rollback_targets_not_in_the_previous_ring: u64,
    creations_applied: u64,
    mounts_or_recoveries_applied: u64,
    one_birth_per_number: VerdictColumns,
    watermark_above_both_highest_numbers: VerdictColumns,
    root_observations: u64,
    unreadable_root_observations: u64,
    version_rebuilds: u64,
    same_identity_with_different_facts: u64,
    abandonment_path_disagreements: u64,
    instance_table_unreadable_steps: u64,
    still_candidate_but_one_birth_holds: u64,
    still_candidate_but_one_birth_partially_judged: u64,
    one_birth_violated_without_two_births_in_the_ledger: u64,
    system_configuration_unchosen_steps: u64,
}

/// 每一步 S2 对两条被数的不变量的判定，D 族的钉值断言要按步看。
#[derive(Clone, Debug, PartialEq, Eq)]
struct StepVerdicts {
    position: String,
    one_birth_per_number: VerdictClass,
    watermark_above_both_highest_numbers: VerdictClass,
}

/// 一次做成的建 inode 发了哪几个号（D 族 A3–A6 按它钉）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CreationIssue {
    position: String,
    issued_numbers: Vec<u64>,
    previous_watermark: Option<u64>,
}

/// 一段历史的账（S1 账、S3 两路、S2 判定与各量）：执行器每一步之后的观察者喂它。
struct HistoryAccount {
    breakage: Breakage,
    /// 根键（根记录的全部字段）→ 读到的那一版：同一个根键只重建一次。
    root_cache: HashMap<String, RootFacts>,
    /// 号 → 出生代（弄坏开关 ledger-ignores-birth 打开时一律记 0）→ 持有过它的根。
    births: BTreeMap<u64, BTreeMap<u64, BTreeSet<RootIdentity>>>,
    /// 见过的每条根的水位（Q3 的 M 按它取）。
    watermarks_seen: BTreeMap<RootIdentity, Option<u64>>,
    /// S3 路 2 的步账（[`abandoned_by_the_step_ledger`]）。
    rows_by_steps: BTreeMap<u32, u64>,
    previous: Option<StepView>,
    previous_model_newest: Option<RootIdentity>,
    open_windows_after_a_rollback_across_issued_numbers: u64,
    counts: HistoryCounts,
    events: Vec<ReissueEvent>,
    first_violation: Option<String>,
    first_abandonment_path_disagreement: Option<String>,
    step_verdicts: Vec<StepVerdicts>,
    creation_issues: Vec<CreationIssue>,
    first_creation_not_issuing_the_previous_watermark: Option<String>,
    /// 每一步之后现行那一版的水位与 inode 树（D 族 A1、A2 按起点那一格钉）。
    current_trajectory: Vec<CurrentVersionAfterAStep>,
    /// 每次做成的回退：目标那一版的身份与 S1 读到的水位（D 族 A5 k ≥ 2 那几格按它钉）。
    rollback_targets: Vec<(RootIdentity, Option<u64>)>,
}

/// 一段历史跑完的结果：账、结局与挂钟（挂钟在装置进程里自己计，登记第五节「预计挂钟」）。
struct HistoryEvaluation {
    family: Family,
    index: u64,
    label: String,
    operations: Vec<HistoryOperation>,
    ending: String,
    account: HistoryAccount,
    elapsed: Duration,
}

/// 读一个环境变量：`main` 交进程环境，单测交一张表（单测并行，不动进程环境）。
type VariableReader<'reader> = &'reader dyn Fn(&str) -> Option<String>;

/// 没设返回 `None`；设了要是正整数。
fn optional_positive_number(name: &str, read_variable: VariableReader<'_>) -> Option<u64> {
    let value = read_variable(name)?;
    let number: u64 = value
        .parse()
        .unwrap_or_else(|_| panic!("{name}=「{value}」不是正整数"));
    assert!(number > 0, "{name} 要是正整数，收到 0");
    Some(number)
}

/// 大规模那一趟的范围：环境变量定下来，之后不再读环境。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RunRequest {
    arm: Arm,
    families: Vec<Family>,
    result_file: PathBuf,
    worker_threads: usize,
    time_budget: Option<Duration>,
    explicit_indices: Option<Vec<u64>>,
    index_limit: Option<u64>,
    code_baseline: String,
}

impl RunRequest {
    fn from_variables(read_variable: VariableReader<'_>) -> RunRequest {
        let breakage = Breakage::from_variable_value(read_variable("E175_BREAK").as_deref());
        assert_eq!(
            breakage,
            Breakage::NONE,
            "大规模不许开弄坏开关（E175_BREAK 只给自证用）"
        );
        let arm = Arm::from_variable_value(read_variable("E175_ARM").as_deref());
        RunRequest {
            arm,
            families: Family::selected_by_the_variable_value(
                read_variable("E175_FAMILY").as_deref(),
                arm,
            ),
            result_file: PathBuf::from(
                read_variable("E175_RESULT_FILE")
                    .expect("E175_RESULT_FILE 没设：结果行要写到点名的文件"),
            ),
            worker_threads: optional_positive_number("E175_WORKER_THREADS", read_variable)
                .map_or_else(
                    || singlefs_verification::heavy_threads::default_heavy_threads().get(),
                    |threads| usize::try_from(threads).expect("线程数装得进 usize"),
                ),
            time_budget: optional_positive_number("E175_TIME_BUDGET_SECONDS", read_variable)
                .map(Duration::from_secs),
            explicit_indices: read_variable("E175_INDICES").map(|list| {
                list.split(',')
                    .map(|index| index.trim().parse().expect("E175_INDICES 是逗号分隔的序号"))
                    .collect()
            }),
            index_limit: read_variable("E175_INDEX_LIMIT")
                .map(|limit| limit.parse::<u64>().expect("E175_INDEX_LIMIT 是正整数")),
            code_baseline: code_baseline_from_the_variables(read_variable),
        }
    }
}

/// 结果文件里已有的、写完整了的结果行（带 `line_end=ok`）的序号：续跑跳过它们。
fn indices_already_in(path: &std::path::Path, family: Family, arm: Arm) -> BTreeSet<u64> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    let family_field = format!(" family={} ", family.label());
    let arm_field = format!(" arm={} ", arm.label());
    text.lines()
        .filter(|line| line.starts_with("E7RESULT name=history ") && line.ends_with(" line_end=ok"))
        .filter(|line| line.contains(&family_field) && line.contains(&arm_field))
        .filter_map(|line| {
            line.split(' ')
                .find_map(|field| field.strip_prefix("index="))
                .and_then(|index| index.parse().ok())
        })
        .collect()
}

/// 结果文件与汇总文件的第一行（文件头），不是 `E7RESULT` 行。
fn code_baseline_header(code_baseline: &str) -> String {
    format!("# code_baseline={code_baseline}")
}

/// 一份结果文件第一行记的代码标记；第一行不是文件头时交回 `None`。
fn code_baseline_of_result_text(text: &str) -> Option<&str> {
    text.lines()
        .next()
        .and_then(|first_line| first_line.strip_prefix("# code_baseline="))
}

/// 几份结果文件的代码标记必须全有、全相同（登记第十一节「两份代码混用」）；交回那一个值，否则交回说明。
fn single_code_baseline_of_result_texts(texts: &[(String, String)]) -> Result<String, String> {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, text) in texts {
        let Some(code_baseline) = code_baseline_of_result_text(text) else {
            return Err(format!("{path} 第一行不是 `# code_baseline=` 文件头"));
        };
        seen.entry(code_baseline.to_string())
            .or_default()
            .push(path.clone());
    }
    match seen.len() {
        1 => Ok(seen.into_keys().next().expect("len 是 1")),
        0 => Err("没有输入".to_string()),
        different_code_count => Err(format!(
            "输入来自 {different_code_count} 份不同的代码：{seen:?}"
        )),
    }
}

fn run_one_family_on_one_arm(family: Family, request: &RunRequest) {
    let arm = request.arm;
    let path = request.result_file.as_path();
    let explicit_indices = request.explicit_indices.as_deref();
    let limit = request.index_limit;
    let registered = family.registered_history_count(arm);
    assert!(
        registered > 0,
        "{} 在 {} 臂上登记 0 段（登记第八节那张表写「不跑」的格）",
        family.label(),
        arm.label()
    );
    let wanted: Vec<u64> = match explicit_indices {
        Some(indices) => indices.to_vec(),
        None => (0..limit.map_or(registered, |limit| limit.min(registered))).collect(),
    };
    assert!(
        wanted.iter().all(|index| *index < registered),
        "序号越过这一族在这一臂登记的 {registered} 段"
    );
    let done = indices_already_in(path, family, arm);
    let items: Vec<HistoryItem> = wanted
        .iter()
        .filter(|index| !done.contains(index))
        .map(|index| history_item(family, *index))
        .collect();
    let existing_text = std::fs::read_to_string(path).unwrap_or_default();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|error| panic!("开不了结果文件 {}：{error}", path.display()));
    let code_baseline = request.code_baseline.as_str();
    if existing_text.is_empty() {
        file.write_all(format!("{}\n", code_baseline_header(code_baseline)).as_bytes())
            .expect("写结果文件");
    } else {
        assert_eq!(
            code_baseline_of_result_text(&existing_text),
            Some(code_baseline),
            "续写的结果文件 {} 第一行的代码标记与这一次的 E175_CODE_BASELINE 不同（两份代码混用）",
            path.display()
        );
    }
    if !existing_text.is_empty() && !existing_text.ends_with('\n') {
        // 上一趟写到一半被打断的那一行没有 line_end=ok，续跑不认它；补一个换行把它隔开。
        file.write_all(b"\n").expect("写结果文件");
    }
    let started = Instant::now();
    let mut evaluated = 0_u64;
    evaluate_items(
        &items,
        request.worker_threads,
        request.time_budget.map(|budget| started + budget),
        Breakage::NONE,
        |evaluation| {
            let mut block = String::new();
            for line in evaluation.event_lines(arm) {
                block.push_str(&line);
                block.push('\n');
            }
            block.push_str(&evaluation.history_line(arm));
            block.push('\n');
            file.write_all(block.as_bytes()).expect("写结果文件");
            file.flush().expect("刷结果文件");
            evaluated += 1;
        },
    );
    let now_done = indices_already_in(path, family, arm);
    let remaining = wanted
        .iter()
        .filter(|index| !now_done.contains(index))
        .count();
    let chunk_line = format!(
        "E7TIMING name=chunk family={} arm={} worker_threads={} evaluated={evaluated} remaining={remaining} elapsed_ns={}\n",
        family.label(),
        arm.label(),
        request.worker_threads,
        started.elapsed().as_nanos()
    );
    file.write_all(chunk_line.as_bytes()).expect("写结果文件");
    let complete_marker = format!(
        "E7RESULT name=family_complete family={} arm={} ",
        family.label(),
        arm.label()
    );
    let whole_family = explicit_indices.is_none() && limit.is_none_or(|limit| limit >= registered);
    if whole_family && remaining == 0 && !existing_text.contains(&complete_marker) {
        let line = format!("{complete_marker}histories={}\n", now_done.len());
        file.write_all(line.as_bytes()).expect("写结果文件");
    }
    println!(
        "E175 {} {} evaluated={evaluated} remaining={remaining}",
        family.label(),
        arm.label()
    );
}

/// 大规模：点名的每一族依次跑进同一份结果文件。
fn run_families_into_the_result_file(request: &RunRequest) {
    for family in &request.families {
        run_one_family_on_one_arm(*family, request);
    }
}

/// 命令行选的那条入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandLineMode {
    RunTheFamilies,
    SummarizeTheResultFiles,
}

/// 不带参数跑大规模，`summarize` 写汇总，别的认不得交 `None`。
fn command_line_mode_of(arguments: &[String]) -> Option<CommandLineMode> {
    match arguments {
        [] => Some(CommandLineMode::RunTheFamilies),
        [word] if word == "summarize" => Some(CommandLineMode::SummarizeTheResultFiles),
        [_, ..] => None,
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let read_process_variable = |name: &str| std::env::var(name).ok();
    match command_line_mode_of(&arguments) {
        Some(CommandLineMode::RunTheFamilies) => {
            run_families_into_the_result_file(&RunRequest::from_variables(&read_process_variable));
        }
        Some(CommandLineMode::SummarizeTheResultFiles) => {
            summarize_the_result_files_from_the_variables(&read_process_variable);
        }
        None => {
            eprintln!("用法：e175_product_path_inode_watermark_regression [summarize]；范围由环境变量定，见文件头");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests;
