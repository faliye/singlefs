//! checker 档模块：crash_point_replay_progress、crash_injection、crash_given_states、crash_library、crash_amplification、gpu_unit_checks
//! 随机历史每一步的镜像与故障注入每个注入点重开的盘，池级 checker 那一遍搬进 checker 档两段跑（用户 2026-10-13 定：harness 档随机历史每步与
//! 故障注入每个注入点的池级 checker「搬进 checker 档两段跑」，harness 只留与理想模型、RefFS 比对那一截）：
//! harness 用同一批种子跑随机历史与故障注入（`PerStepChecker::Skipped`，harness 一条不变量都不判），每一步的镜像经每步镜像那个钩子
//! （`singlefs_harness::history::execute_history_handing_out_every_step_image`）、每个注入点重开的盘经注入点那个钩子
//! （`singlefs_harness::fault_injection::inject_faults_into_history_handing_out_the_images`）交出来；这里把它们建成给定状态表
//! （基线是 mkfs 之后的池、写表是 mkfs 之后的录制流、每张镜像一个持久集合，掉电接缝照历史里的掉电断开记录核对器的发布表，
//! `crash_injection::HistoryAfterTheMakeFilesystem`），交崩溃点重放的给定状态引擎（`crash_amplification::GivenStatesEngine`）：
//! 每张表先写进给定状态表那一类的判定库（录入段）、判段从库里读回来判，配了 GPU 只由 GPU 判器判、没配只由 CPU 判器判，结论按表存进库、
//! 判过的复用（键：判法版本、工具链、表的输入摘要，引擎现有的给定状态键）。
//!
//! 判据与 harness 原先每一步就地跑池级 checker 时相同：随机历史每一步之后的镜像上池级 checker 一条都不红（连同记录核对器两条：
//! 这几个状态都是「掉电接缝之前全落、之后一个都没落」，在崩溃注入摆状态的域里）；故障注入每个注入点上，带着注入跑的那一遍里第一张判红的每步镜像
//! 与重开的那张盘上判红的，照 harness 的归类（`fault_injection::classify_the_pool_checker_verdicts_of_the_images_after_a_fault`）只许落进
//! 故障注入自己那条「已知红」或说谎的设备留下的不一致，清单外的一条都不许有。故障注入那几张盘上的记录核对器只记数、不判
//! （注入的写报错、被吞的写都不进录制流，记录核对器在这种流上的判据没有条款）。
//!
//! 快档（普通 `cargo test`）跑一小批（随机历史与故障注入各头两段、每段 8 步），经引擎照本机配置判；编进 `gpu` 特性另有两条：
//! 同一小批 GPU 一趟、CPU 一趟分开判（各写各的库），逐张表逐状态的对拍字相同，往 GPU 那一侧塞一个故障两趟就不同。
//! 全量（`#[ignore]`，门禁 checker-tier-crash-point-replay `--full` 按登记的 `crash-case:` 那一行跑）跑 harness 原先逐步跑 checker 的那几段取样：
//! 随机历史快档、复用、回退、读回全 0、越过原分配记录墙、单元区墙两段，故障注入快档；种子与规模与 harness 那几条用例逐项相同。
#![cfg(feature = "verdict-store")]

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker_tier::crash_amplification::{
    FaultPutOnTheGpuSide, GivenStatesEngine, GivenStatesItem, GivenStatesRun,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_injection::HistoryAfterTheMakeFilesystem;
use singlefs_checker_tier::crash_judge_conclusions::CrossCheckWords;
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;

#[cfg(feature = "gpu")]
mod common_gpu_cards_of_the_caller;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault,
    inject_faults_into_history_handing_out_the_images, FailureAfterAFault, ImagesAfterAFault,
    RecordedStreamOfAnImage, SharedFaultPlan,
};
use singlefs_harness::gpu_memory_ledger::gpu_not_run_line;
use singlefs_harness::heavy_threads::{
    own_variable_or_default_heavy_threads, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::history::{
    execute_history_handing_out_every_step_image, generate_history_with_weights, GenerationWeights,
    HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker, StartingPointStep,
    StepPosition, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, RecordStreamSeam};
use singlefs_harness::numeric_environment_variable::process_environment_variable;
use singlefs_harness::SharedStream;

/// 这份测试文件的仓内路径：给定状态表那一路的设点文件（CPU 结论的版本按它出发的闭包取）。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs";

/// 崩溃点重放合表里这一项的名字（`.claude/gate.d/crash-point-replay-items.tsv`）。
const CRASH_POINT_REPLAY_ITEM: &str = "random-histories-and-fault-injection-given-states";

/// 全量跑完打的那一行的头（`.claude/gate.d/stage-inputs.tsv` 那一行 `crash-case:` 登记的计数行）：只打一次，自己带 `worker_threads=` 与 `slices=`。
const FINISHED_LINE_PREFIX: &str = "RANDOM_HISTORIES_AND_FAULT_INJECTION_GIVEN_STATES_FINISHED";

/// 工作线程数从这个环境变量取（`scripts/hosts/capped.sh` 设它；登记那一行的 `threads-variable=` 也是它）；没设取本机并行度。
const WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_RANDOM_HISTORY_THREADS";

/// harness 不跑每一步的池级 checker（那一遍就在这里），盘宽与空间准入照取样那一段。
const fn without_the_pool_checker(
    device_width: HistoryDeviceWidth,
    space_admission: SpaceAdmission,
) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width,
        space_admission,
    }
}

/// 一段随机历史取样：种子区间、每段步数、比重、怎么跑（与 harness `tests/random_histories.rs` 里同名那一段逐项相同）。
#[derive(Clone, Copy, Debug)]
struct RandomHistorySampling {
    name: &'static str,
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    weights: GenerationWeights,
    execution: HistoryExecution,
}

/// 一段故障注入取样：种子区间、每段步数、每段注入几次、比重、怎么跑（与 harness `tests/fault_injection_fast_tier.rs` 的快档逐项相同）。
#[derive(Clone, Copy, Debug)]
struct FaultInjectionSampling {
    name: &'static str,
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    faults_per_history: usize,
    weights: GenerationWeights,
    execution: HistoryExecution,
}

/// harness 原先每一步就地跑池级 checker 的那几段随机历史取样（`tests/random_histories.rs`：快档、复用、回退、读回全 0、越过原分配记录墙、单元区墙两段）。
const RANDOM_HISTORY_SAMPLINGS_OF_THE_FULL_RUN: [RandomHistorySampling; 7] = [
    RandomHistorySampling {
        name: "fast-tier",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 96,
        operations_per_history: 30,
        weights: GenerationWeights::BROAD,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    },
    RandomHistorySampling {
        name: "reuse-after-raising-the-floor",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 48,
        operations_per_history: 30,
        weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    },
    RandomHistorySampling {
        name: "rollback-after-raising-the-floor",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 48,
        operations_per_history: 30,
        weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    },
    RandomHistorySampling {
        name: "newest-system-configuration-slot-of-each-device-reading-zeros",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 48,
        operations_per_history: 24,
        weights: GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            SpaceAdmission::JudgedByTheFormula,
        ),
    },
    RandomHistorySampling {
        name: "toward-the-allocation-record-wall",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 32,
        operations_per_history: 150,
        weights: GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    },
    RandomHistorySampling {
        name: "toward-the-unit-area-wall-with-the-space-admission-skipped",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 32,
        operations_per_history: 150,
        weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::UnitAreaOf256Slots,
            SpaceAdmission::SkippedByTheTestOnlySwitch,
        ),
    },
    RandomHistorySampling {
        name: "toward-the-unit-area-wall-with-the-space-admission-judged",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 32,
        operations_per_history: 150,
        weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::UnitAreaOf256Slots,
            SpaceAdmission::JudgedByTheFormula,
        ),
    },
];

/// harness 故障注入快档（`tests/fault_injection_fast_tier.rs`：种子基起 24 段、每段 20 步、每段 4 个注入点）。
const FAULT_INJECTION_SAMPLINGS_OF_THE_FULL_RUN: [FaultInjectionSampling; 1] =
    [FaultInjectionSampling {
        name: "fault-injection-fast-tier",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 24,
        operations_per_history: 20,
        faults_per_history: 4,
        weights: GenerationWeights::BROAD,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    }];

/// 快档那一小批：随机历史头两段、每段 8 步。
const RANDOM_HISTORY_SAMPLINGS_OF_THE_SMALL_RUN: [RandomHistorySampling; 1] =
    [RandomHistorySampling {
        name: "small-tier",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        weights: GenerationWeights::BROAD,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    }];

/// 快档那一小批：故障注入头两段、每段 8 步、每段 2 个注入点。
const FAULT_INJECTION_SAMPLINGS_OF_THE_SMALL_RUN: [FaultInjectionSampling; 1] =
    [FaultInjectionSampling {
        name: "fault-injection-small-tier",
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        faults_per_history: 2,
        weights: GenerationWeights::BROAD,
        execution: without_the_pool_checker(
            HistoryDeviceWidth::FourGibibytes,
            SpaceAdmission::JudgedByTheFormula,
        ),
    }];

/// 表名里标 mkfs 没做完的那几张（故障注入落在 mkfs 自己身上、它把错交回来）：盘上没有自证过的系统配置，GPU 判器建不出判表，
/// 引擎照 H6 记一条「GPU 判不了」、交 CPU 判器的结论；只有这几张许判不了，别的一张都不许。
const MAKE_FILESYSTEM_UNFINISHED_MARKER: &str = "make-filesystem-unfinished";

/// 一趟判下来的账：判了几段、几张表、几个状态，逐张表逐状态的对拍字（GPU 与 CPU 两趟比的就是它），哪几条不变量判成立过，
/// 清单外的失败，故障注入归类的几格。
#[derive(Clone, Debug, PartialEq, Eq)]
struct JudgedThroughTheLibrary {
    random_histories: u64,
    step_images: u64,
    fault_injection_histories: u64,
    injection_points: u64,
    tables: u64,
    states: u64,
    words_by_table: BTreeMap<String, Vec<CrossCheckWords>>,
    invariants_held_at_least_once: BTreeSet<&'static str>,
    findings: Vec<String>,
    known_red_after_a_fault: BTreeMap<String, u64>,
    lying_device_signatures: BTreeMap<String, u64>,
    record_checker_red_states_after_a_fault: u64,
    tables_left_to_another_host: u64,
    tables_recorded_only: u64,
    tables_the_gpu_may_not_judge: BTreeSet<String>,
}

impl JudgedThroughTheLibrary {
    /// 还一张表都没判的账：随机历史几段、故障注入几段。
    fn of_histories(random_histories: u64, fault_injection_histories: u64) -> Self {
        Self {
            random_histories,
            step_images: 0,
            fault_injection_histories,
            injection_points: 0,
            tables: 0,
            states: 0,
            words_by_table: BTreeMap::new(),
            invariants_held_at_least_once: BTreeSet::new(),
            findings: Vec::new(),
            known_red_after_a_fault: BTreeMap::new(),
            lying_device_signatures: BTreeMap::new(),
            record_checker_red_states_after_a_fault: 0,
            tables_left_to_another_host: 0,
            tables_recorded_only: 0,
            tables_the_gpu_may_not_judge: BTreeSet::new(),
        }
    }

    /// 把另一段的账按次序并进来（工作线程各判各的段，按段的次序并：结果与线程数无关）。
    fn absorb(&mut self, following: Self) {
        self.random_histories += following.random_histories;
        self.step_images += following.step_images;
        self.fault_injection_histories += following.fault_injection_histories;
        self.injection_points += following.injection_points;
        self.tables += following.tables;
        self.states += following.states;
        self.words_by_table.extend(following.words_by_table);
        self.invariants_held_at_least_once
            .extend(following.invariants_held_at_least_once);
        self.findings.extend(following.findings);
        for (form, count) in following.known_red_after_a_fault {
            *self.known_red_after_a_fault.entry(form).or_insert(0) += count;
        }
        for (signature, count) in following.lying_device_signatures {
            *self.lying_device_signatures.entry(signature).or_insert(0) += count;
        }
        self.record_checker_red_states_after_a_fault +=
            following.record_checker_red_states_after_a_fault;
        self.tables_left_to_another_host += following.tables_left_to_another_host;
        self.tables_recorded_only += following.tables_recorded_only;
        self.tables_the_gpu_may_not_judge
            .extend(following.tables_the_gpu_may_not_judge);
    }

    /// 判成立的那几条记进「判成立过」：判到位而没违例。
    fn note_the_invariants_held(&mut self, conclusion: &GivenStateConclusion) {
        let held = conclusion.words.pool_checker_judged_bits
            & !conclusion.words.pool_checker_violated_bits;
        for (position, invariant) in singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .enumerate()
        {
            if held & (1u128 << position) != 0 {
                self.invariants_held_at_least_once.insert(invariant);
            }
        }
    }
}

/// 一张镜像的录制流的若干个前缀（`prefixes`：每个是录制流里的步数）建成给定状态表交引擎判：录制流里 mkfs 写的那几步进基线，
/// 每个前缀一个持久集合（前缀里的写全落、之后的一个都没落），掉电接缝相同的几个前缀一张表（记录核对器照每个前缀之前掉过的电断开）。
/// 交回与 `prefixes` 同序的结论。判不了就 panic：配了 GPU 卡用不上、判定库读写失败都判红，不退回 CPU。
fn judge_prefixes_of_a_recorded_stream(
    judge: &dyn GivenStatesJudge,
    table_name: &str,
    recorded: &RecordedStreamOfAnImage,
    execution: HistoryExecution,
    prefixes: &[usize],
    judged: &mut JudgedThroughTheLibrary,
) -> Vec<GivenStateConclusion> {
    let geometry = execution.device_width.fixed_geometry();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&recorded.operations, &geometry);
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        recorded.operations_written_by_make_filesystem,
        execution,
    );
    // 每次掉电之后第一个写在写表里的下标：取录制流下标不小于掉电之后第一步的第一个写（写表不算屏障），掉电之后一个写都没录下的取写表长度
    let first_write_after_each_power_loss: Vec<usize> = recorded
        .first_stream_index_after_each_power_loss
        .iter()
        .map(|first_stream_index_after_the_power_loss| {
            stream_indexes
                .iter()
                .position(|stream_index| *stream_index >= *first_stream_index_after_the_power_loss)
                .unwrap_or(stream_indexes.len())
        })
        .collect();
    let mut prefixes_by_power_losses: BTreeMap<Vec<usize>, Vec<usize>> = BTreeMap::new();
    let mut persisted_writes_of_each_prefix: Vec<usize> = Vec::with_capacity(prefixes.len());
    for (prefix_index, operations_recorded) in prefixes.iter().enumerate() {
        let persisted_writes = stream_indexes
            .iter()
            .filter(|stream_index| **stream_index < *operations_recorded)
            .count();
        persisted_writes_of_each_prefix.push(persisted_writes);
        let power_losses_before_the_prefix_ends: Vec<usize> = first_write_after_each_power_loss
            .iter()
            .copied()
            .filter(|first_write_after_the_power_loss| {
                *first_write_after_the_power_loss <= persisted_writes
            })
            .collect();
        prefixes_by_power_losses
            .entry(power_losses_before_the_prefix_ends)
            .or_default()
            .push(prefix_index);
    }
    let mut conclusions: Vec<Option<GivenStateConclusion>> = vec![None; prefixes.len()];
    for (power_losses, prefix_indexes) in prefixes_by_power_losses {
        let seams: Vec<RecordStreamSeam> = power_losses
            .iter()
            .map(
                |first_write_after_the_power_loss| RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss: history_after_the_make_filesystem
                        .index_in_the_table(*first_write_after_the_power_loss),
                },
            )
            .collect();
        let persisted_sets: Vec<Vec<bool>> = prefix_indexes
            .iter()
            .map(|prefix_index| {
                let persisted_writes = persisted_writes_of_each_prefix[*prefix_index];
                let persisted_over_the_whole_write_table: Vec<bool> = (0..writes.len())
                    .map(|write_index| write_index < persisted_writes)
                    .collect();
                history_after_the_make_filesystem
                    .persisted_set_of_the_table(&persisted_over_the_whole_write_table)
            })
            .collect();
        let table = GivenStatesOfATable {
            table_name: format!("{table_name}/power-losses={power_losses:?}"),
            base: history_after_the_make_filesystem.base(),
            writes: history_after_the_make_filesystem.writes(),
            segments: history_after_the_make_filesystem.segments(),
            seams: &seams,
            persisted_sets,
        };
        let judged_table = judge
            .judge_the_given_states(&table)
            .unwrap_or_else(|reason| {
                panic!(
                    "给定状态表 {} 判不了（不退回别处判）：{reason}",
                    table.table_name
                )
            });
        assert_eq!(
            judged_table.states.len(),
            prefix_indexes.len(),
            "判器逐状态交回结论，与表里的状态同序"
        );
        judged.tables += 1;
        judged.states += u64::try_from(prefix_indexes.len()).expect("装得进 u64");
        match judged_table.source {
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
            GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost => {
                judged.tables_left_to_another_host += 1;
            }
            GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
                judged.tables_recorded_only += 1;
            }
        }
        if table_name.contains(MAKE_FILESYSTEM_UNFINISHED_MARKER) {
            judged
                .tables_the_gpu_may_not_judge
                .insert(table.table_name.clone());
        }
        judged.words_by_table.insert(
            table.table_name.clone(),
            judged_table
                .states
                .iter()
                .map(|state| state.words)
                .collect(),
        );
        for (prefix_index, conclusion) in prefix_indexes.into_iter().zip(judged_table.states) {
            conclusions[prefix_index] = Some(conclusion);
        }
    }
    conclusions
        .into_iter()
        .map(|conclusion| conclusion.expect("每个前缀恰好落在一组掉电接缝里"))
        .collect()
}

/// 一段随机历史：harness 跑一遍（不跑池级 checker），每一步的镜像经钩子交出来，按录制流前缀建成给定状态表交引擎判；
/// 哪一步的镜像上池级 checker 或记录核对器红，记一条失败。
fn judge_the_step_images_of_a_random_history(
    judge: &dyn GivenStatesJudge,
    sampling: &RandomHistorySampling,
    seed: HistorySeed,
) -> JudgedThroughTheLibrary {
    let mut judged = JudgedThroughTheLibrary::of_histories(1, 0);
    let history =
        generate_history_with_weights(seed, sampling.operations_per_history, &sampling.weights);
    let execution = sampling.execution;
    let stream = SharedStream::retaining_contents();
    let mut step_images: Vec<(StepPosition, usize)> = Vec::new();
    let run = execute_history_handing_out_every_step_image(
        &history,
        execution,
        &stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |step_image| {
            step_images.push((step_image.position, step_image.operations_recorded));
        },
    );
    assert_eq!(
        run.tally.checker_runs, 0,
        "{} 种子 {}：harness 不跑每一步的池级 checker（那一遍在这里）",
        sampling.name, seed.0
    );
    judged.step_images = u64::try_from(step_images.len()).expect("装得进 u64");
    let recorded = RecordedStreamOfAnImage {
        operations: stream.retained_operations(),
        operations_written_by_make_filesystem: run.operations_written_by_make_filesystem,
        first_stream_index_after_each_power_loss: run.first_stream_index_after_each_power_loss,
    };
    let prefixes: Vec<usize> = step_images
        .iter()
        .map(|(_, operations_recorded)| *operations_recorded)
        .collect();
    let conclusions = judge_prefixes_of_a_recorded_stream(
        judge,
        &format!("random-history/{}/seed={}", sampling.name, seed.0),
        &recorded,
        execution,
        &prefixes,
        &mut judged,
    );
    for ((position, _), conclusion) in step_images.iter().zip(conclusions) {
        judged.note_the_invariants_held(&conclusion);
        let red_items = conclusion.red_items();
        if !red_items.is_empty() {
            judged.findings.push(format!(
                "随机历史 {} 种子 {} {position:?} 之后的镜像红 {red_items:?}；池级 checker {:?}",
                sampling.name,
                seed.0,
                conclusion.pool_checker_violations()
            ));
        }
    }
    judged
}

/// 一段故障注入：harness 按种子摆注入点、每个注入点重跑一遍（不跑池级 checker），带着注入跑的那一遍每一步的镜像与重开的盘经钩子交出来；
/// 每个注入点重开那张盘的录制流的前缀（每步镜像与整条）一组表、被吞的写插回去之后那张盘一张表交引擎判，池级 checker 的结论交 harness 归类。
fn judge_the_images_after_the_faults_of_a_history(
    judge: &dyn GivenStatesJudge,
    sampling: &FaultInjectionSampling,
    seed: HistorySeed,
) -> JudgedThroughTheLibrary {
    let mut judged = JudgedThroughTheLibrary::of_histories(0, 1);
    let history =
        generate_history_with_weights(seed, sampling.operations_per_history, &sampling.weights);
    let mut handed_out: Vec<ImagesAfterAFault> = Vec::new();
    let injection = inject_faults_into_history_handing_out_the_images(
        &history,
        sampling.execution,
        sampling.faults_per_history,
        &mut |images| handed_out.push(images),
    );
    assert_eq!(
        (
            injection.tally.checker_runs_on_the_image_after_the_fault,
            injection.tally.images_after_the_fault_handed_out,
        ),
        (0, u64::try_from(handed_out.len()).expect("装得进 u64")),
        "{} 种子 {}：harness 不在注入之后的盘上跑池级 checker，每个注入点交出一份盘",
        sampling.name,
        seed.0
    );
    for (injection_point, images) in handed_out.iter().enumerate() {
        judged.injection_points += 1;
        let make_filesystem_unfinished = images
            .facts_after_the_fault
            .starting_point_step_that_returned_an_error
            == Some(StartingPointStep::MakeFilesystem);
        let table_name = format!(
            "fault-injection/{}/seed={}/injection-point={injection_point}{}",
            sampling.name,
            seed.0,
            if make_filesystem_unfinished {
                format!("/{MAKE_FILESYSTEM_UNFINISHED_MARKER}")
            } else {
                String::new()
            }
        );
        let mut prefixes: Vec<usize> = images
            .step_images_of_the_run_with_the_fault
            .iter()
            .map(|step_image| step_image.operations_recorded)
            .collect();
        prefixes.push(images.image_after_the_fault.operations.len());
        let mut conclusions = judge_prefixes_of_a_recorded_stream(
            judge,
            &table_name,
            &images.image_after_the_fault,
            sampling.execution,
            &prefixes,
            &mut judged,
        );
        let conclusion_after_the_fault = conclusions.pop().expect("整条录制流那一个前缀在最后");
        let invariants_still_violated_if_the_swallowed_writes_had_landed = images
            .image_with_the_swallowed_writes_landed
            .as_ref()
            .map_or_else(BTreeSet::new, |landed| {
                judge_prefixes_of_a_recorded_stream(
                    judge,
                    &format!("{table_name}/swallowed-writes-landed"),
                    landed,
                    sampling.execution,
                    &[landed.operations.len()],
                    &mut judged,
                )
                .into_iter()
                .flat_map(|conclusion| conclusion.pool_checker_violations())
                .map(|(invariant, _)| invariant)
                .collect()
            });
        judged.record_checker_red_states_after_a_fault += u64::try_from(
            conclusions
                .iter()
                .chain(std::iter::once(&conclusion_after_the_fault))
                .filter(|conclusion| {
                    !singlefs_harness::history::record_check_aspects(&conclusion.record_check())
                        .is_empty()
                })
                .count(),
        )
        .expect("装得进 u64");
        let failures = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
            images,
            conclusions
                .iter()
                .map(GivenStateConclusion::pool_checker_violations)
                .collect(),
            conclusion_after_the_fault.pool_checker_violations(),
            invariants_still_violated_if_the_swallowed_writes_had_landed,
        );
        for failure in failures {
            match failure {
                FailureAfterAFault::KnownRed(form) => {
                    *judged
                        .known_red_after_a_fault
                        .entry(form.render())
                        .or_insert(0) += 1;
                }
                FailureAfterAFault::LeftByTheLyingDevice { signature } => {
                    *judged
                        .lying_device_signatures
                        .entry(format!("{signature:?}"))
                        .or_insert(0) += 1;
                }
                FailureAfterAFault::NewFinding {
                    signature,
                    observation,
                } => judged.findings.push(format!(
                    "故障注入 {} 种子 {} 注入点 {}（{}）：新发现 {signature:?}，{observation:?}",
                    sampling.name,
                    seed.0,
                    injection_point,
                    images.drawn.render()
                )),
            }
        }
    }
    judged
}

/// 一段要判的历史：随机历史那几段取样里的一段，或故障注入那几段里的一段。
#[derive(Clone, Copy, Debug)]
enum HistoryToJudge {
    RandomHistory(RandomHistorySampling, HistorySeed),
    FaultInjection(FaultInjectionSampling, HistorySeed),
}

/// 这几段取样里每一段历史，按取样的次序、种子从小到大。
fn histories_of(
    random_histories: &[RandomHistorySampling],
    fault_injections: &[FaultInjectionSampling],
) -> Vec<HistoryToJudge> {
    let random = random_histories.iter().flat_map(|sampling| {
        (0..sampling.seed_count).map(move |offset| {
            HistoryToJudge::RandomHistory(*sampling, HistorySeed(sampling.first_seed + offset))
        })
    });
    let faults = fault_injections.iter().flat_map(|sampling| {
        (0..sampling.seed_count).map(move |offset| {
            HistoryToJudge::FaultInjection(*sampling, HistorySeed(sampling.first_seed + offset))
        })
    });
    random.chain(faults).collect()
}

/// 工作线程数：[`WORKER_THREADS_ENVIRONMENT_VARIABLE`]，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取
/// `.claude/heavy-thread-default` 的登记值；判法在 `singlefs_harness::heavy_threads`）。
fn worker_threads() -> usize {
    own_variable_or_default_heavy_threads(
        WORKER_THREADS_ENVIRONMENT_VARIABLE,
        process_environment_variable(WORKER_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
    .get()
}

/// 一批历史交引擎判：工作线程按段领（叫停了就不再领），一段的账按段的次序并（结果与线程数无关）。交回并好的账、跑了几段、起了几条线程。
fn judge_the_histories(
    judge: &dyn GivenStatesJudge,
    histories: &[HistoryToJudge],
    worker_threads: usize,
) -> (JudgedThroughTheLibrary, usize, usize) {
    let next_history = AtomicUsize::new(0);
    let judged_by_history: Mutex<Vec<Option<JudgedThroughTheLibrary>>> =
        Mutex::new(vec![None; histories.len()]);
    let spawned_worker_threads = worker_threads.clamp(1, histories.len().max(1));
    std::thread::scope(|scope| {
        for _ in 0..spawned_worker_threads {
            scope.spawn(|| loop {
                if judge.stop_is_requested() {
                    break;
                }
                let history_index = next_history.fetch_add(1, Ordering::SeqCst);
                let Some(history) = histories.get(history_index) else {
                    break;
                };
                let judged = match history {
                    HistoryToJudge::RandomHistory(sampling, seed) => {
                        judge_the_step_images_of_a_random_history(judge, sampling, *seed)
                    }
                    HistoryToJudge::FaultInjection(sampling, seed) => {
                        judge_the_images_after_the_faults_of_a_history(judge, sampling, *seed)
                    }
                };
                judged_by_history.lock().expect("判定线程不 panic")[history_index] = Some(judged);
            });
        }
    });
    let mut merged = JudgedThroughTheLibrary::of_histories(0, 0);
    let mut histories_judged = 0usize;
    for judged in judged_by_history
        .into_inner()
        .expect("判定线程不 panic")
        .into_iter()
        .flatten()
    {
        merged.absorb(judged);
        histories_judged += 1;
    }
    (merged, histories_judged, spawned_worker_threads)
}

fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../Cargo.lock")
        .canonicalize()
        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
        .parent()
        .expect("Cargo.lock 所在的目录就是仓根")
        .to_path_buf()
}

/// 这一批的种子与规模（进给定状态表的块键）。
fn seeds_and_scale_of(
    random_histories: &[RandomHistorySampling],
    fault_injections: &[FaultInjectionSampling],
) -> String {
    format!("random_histories={random_histories:?} fault_injections={fault_injections:?}")
}

/// 起引擎（照此刻的环境：多机配置、库目录、显卡先后；`fault` 只给证明 GPU 与 CPU 两趟会不同的那条用例）、判这一批、收尾。
fn judge_through_the_given_states_engine(
    random_histories: &[RandomHistorySampling],
    fault_injections: &[FaultInjectionSampling],
    fault: FaultPutOnTheGpuSide,
    worker_threads: usize,
) -> (JudgedThroughTheLibrary, usize, usize, GivenStatesRun) {
    let engine = GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side(
        GivenStatesItem {
            item_name: CRASH_POINT_REPLAY_ITEM.to_string(),
            setting_file: THIS_FILE.to_string(),
            seeds_and_scale: seeds_and_scale_of(random_histories, fault_injections),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        &repository_root(),
        fault,
    )
    .expect("崩溃点重放的引擎照本机配置起得来（配了 GPU 卡用不上、结论版本与编译时对不上都在这里红，不退回 CPU）");
    let (judged, histories_judged, spawned_worker_threads) = judge_the_histories(
        &engine,
        &histories_of(random_histories, fault_injections),
        worker_threads,
    );
    (
        judged,
        histories_judged,
        spawned_worker_threads,
        engine.finish(),
    )
}

/// 引擎那一趟判红的几样（GPU 判不了）里，除了 mkfs 没做完的那几张表（[`MAKE_FILESYSTEM_UNFINISHED_MARKER`]）之外的那些。
fn red_items_of_the_engine_beyond_the_tables_the_gpu_may_not_judge(
    run: &GivenStatesRun,
    judged: &JudgedThroughTheLibrary,
) -> Vec<String> {
    run.tally
        .states_the_gpu_could_not_judge
        .iter()
        .filter(|state| {
            !judged
                .tables_the_gpu_may_not_judge
                .iter()
                .any(|table_name| state.starts_with(table_name.as_str()))
        })
        .cloned()
        .collect()
}

/// 每一张表都过了引擎（先入库、判段从库里读回）：这一趟判了的加复用的就是建出的张数；GPU 建不出判表的那几张（只许是 mkfs 没做完的）
/// 交的是 CPU 判器的结论、不进「判了的」那一格，至多差这么多张。
fn assert_every_table_went_through_the_engine(
    run: &GivenStatesRun,
    judged: &JudgedThroughTheLibrary,
) {
    let through_the_engine = run.tally.tables_judged_here + run.tally.tables_reused;
    let tables_the_gpu_may_not_judge =
        u64::try_from(judged.tables_the_gpu_may_not_judge.len()).expect("装得进 u64");
    assert!(
        through_the_engine <= judged.tables
            && through_the_engine + tables_the_gpu_may_not_judge >= judged.tables,
        "每一张表都过了引擎：建出 {} 张，引擎判了 {}、复用 {}，mkfs 没做完的 {} 张",
        judged.tables,
        run.tally.tables_judged_here,
        run.tally.tables_reused,
        tables_the_gpu_may_not_judge
    );
}

/// 报告直接写进进程的标准输出，不经 libtest 的捕获。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 一趟作不作数：引擎被叫停、或两台分片而有表归别的机器判又还没导回来，打「本次未跑」那一行、交 `false`。
fn the_run_counts(run: &GivenStatesRun, judged: &JudgedThroughTheLibrary, test_name: &str) -> bool {
    if run.stopped_early {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE 指的文件在）：判过的表在库里，下一趟接着判"
            )
        );
        return false;
    }
    if run.recorded_only() || judged.tables_recorded_only > 0 {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                &format!(
                    "只跑了录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）：{} 张表写进了库、没判，判段那一趟从库里读回判才作数",
                    judged.tables_recorded_only
                )
            )
        );
        return false;
    }
    if run.left_tables_to_the_other_hosts() || judged.tables_left_to_another_host > 0 {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                &format!(
                    "两台分片：这一份是 {}/{}，{} 张表归别的机器判、还没导回来（导入那一台的库之后再跑一趟才作数）",
                    run.share.index, run.share.count, run.tally.tables_left_to_the_other_hosts
                )
            )
        );
        return false;
    }
    true
}

/// 快档：随机历史与故障注入各头两段（每段 8 步、故障注入每段 2 个注入点），每一步的镜像与每个注入点重开的盘经引擎判（照本机配置走 GPU 或 CPU，
/// 表先入库、判段从库里读回）。随机历史每一张镜像上池级 checker 与记录核对器都不红，故障注入归类之后清单外的一条都没有，
/// 引擎判不了的只许是 mkfs 没做完的那几张；每一张表都过了引擎（这一趟判了的加复用的就是建出的张数），状态与表都不为 0。
/// 判别力：harness 偷回逐步池级 checker（交出去之前就地判）时这里的「checker 跑过 0 次」红；判段不经引擎从库里读、
/// 拿内存里的表就地判时引擎一张表都没见过，张数对账红。
#[test]
fn the_small_tier_of_random_histories_and_fault_injection_judged_through_the_library_has_no_finding(
) {
    const TEST_NAME: &str =
        "the_small_tier_of_random_histories_and_fault_injection_judged_through_the_library_has_no_finding";
    let _one_test_at_a_time =
        singlefs_checker_tier::crash_amplification::one_test_changes_the_environment_at_a_time();
    let (judged, histories_judged, _spawned_worker_threads, run) =
        judge_through_the_given_states_engine(
            &RANDOM_HISTORY_SAMPLINGS_OF_THE_SMALL_RUN,
            &FAULT_INJECTION_SAMPLINGS_OF_THE_SMALL_RUN,
            FaultPutOnTheGpuSide::NoFault,
            worker_threads(),
        );
    if !the_run_counts(&run, &judged, TEST_NAME) {
        return;
    }
    print_uncaptured(&format!(
        "── 随机历史与故障注入小批经库判 ──\n随机历史 {} 段、每步镜像 {} 张；故障注入 {} 段、注入点 {} 个；表 {} 张、状态 {} 个；引擎 {:?}\n",
        judged.random_histories,
        judged.step_images,
        judged.fault_injection_histories,
        judged.injection_points,
        judged.tables,
        judged.states,
        run.tally
    ));
    assert_eq!(
        histories_judged,
        histories_of(
            &RANDOM_HISTORY_SAMPLINGS_OF_THE_SMALL_RUN,
            &FAULT_INJECTION_SAMPLINGS_OF_THE_SMALL_RUN
        )
        .len(),
        "这一批每一段都判了"
    );
    assert!(
        judged.findings.is_empty(),
        "清单外的失败：\n{}",
        judged.findings.join("\n")
    );
    assert_eq!(
        red_items_of_the_engine_beyond_the_tables_the_gpu_may_not_judge(&run, &judged),
        Vec::<String>::new(),
        "引擎判不了的只许是 mkfs 没做完的那几张表"
    );
    assert_every_table_went_through_the_engine(&run, &judged);
    assert!(
        judged.step_images > 0 && judged.injection_points > 0 && judged.states > 0,
        "每步镜像、注入点与状态都不为 0：{judged:?}"
    );
}

/// 全量：harness 原先每一步就地跑池级 checker 的那几段随机历史取样与故障注入快档（种子、规模逐项同 harness 那几条用例），
/// 每一张镜像经引擎判。判据同快档那一条，另核随机历史上 I-3.1、I-5.4、I-3.11、I-7.9、I-9.15 都真被判成立过（原先 harness 快档那一格）、
/// 故障注入里说谎的设备至少一次留下过盘面不一致（原先 harness 故障注入快档那一格：说谎那两种抽到了却什么也没碰着时判别力要重新量）。
/// 跑完打一行 [`FINISHED_LINE_PREFIX`]（只打一次，带 `worker_threads=` 与 `slices=`，一段历史一片）。
/// 标了 ignore：随机历史 336 段（越过原分配记录墙与单元区墙那三段每段 150 步）加故障注入 24 段 × 4 个注入点、每个注入点重跑整段历史，
/// 单跑以十分钟计；由主 agent 经门禁 checker-tier-crash-point-replay `--full` 按 `crash-case:random-histories-and-fault-injection-given-states` 在 release 下跑。
#[test]
#[ignore = "全量：harness 原先逐步跑池级 checker 的那几段取样与故障注入快档全部经库判，单跑以十分钟计；由主 agent 经门禁 checker-tier-crash-point-replay --full 按 crash-case:random-histories-and-fault-injection-given-states 在 release 下跑"]
fn every_step_image_and_every_image_after_a_fault_of_the_harness_samplings_judged_through_the_library_has_no_finding(
) {
    const TEST_NAME: &str = "every_step_image_and_every_image_after_a_fault_of_the_harness_samplings_judged_through_the_library_has_no_finding";
    let started = std::time::Instant::now();
    let (judged, histories_judged, spawned_worker_threads, run) = {
        let _one_test_at_a_time =
            singlefs_checker_tier::crash_amplification::one_test_changes_the_environment_at_a_time(
            );
        judge_through_the_given_states_engine(
            &RANDOM_HISTORY_SAMPLINGS_OF_THE_FULL_RUN,
            &FAULT_INJECTION_SAMPLINGS_OF_THE_FULL_RUN,
            FaultPutOnTheGpuSide::NoFault,
            worker_threads(),
        )
    };
    if !the_run_counts(&run, &judged, TEST_NAME) {
        return;
    }
    let histories = histories_of(
        &RANDOM_HISTORY_SAMPLINGS_OF_THE_FULL_RUN,
        &FAULT_INJECTION_SAMPLINGS_OF_THE_FULL_RUN,
    );
    println!(
        "{FINISHED_LINE_PREFIX} random_histories={} step_images={} fault_injection_histories={} injection_points={} tables={} states={} tables_judged_here={} tables_reused={} states_the_gpu_could_not_judge={} known_red_after_a_fault={} lying_device_signatures={} record_checker_red_states_after_a_fault={} worker_threads={spawned_worker_threads} slices={} elapsed_seconds={:.1}",
        judged.random_histories,
        judged.step_images,
        judged.fault_injection_histories,
        judged.injection_points,
        judged.tables,
        judged.states,
        run.tally.tables_judged_here,
        run.tally.tables_reused,
        run.tally.states_the_gpu_could_not_judge.len(),
        judged.known_red_after_a_fault.values().sum::<u64>(),
        judged.lying_device_signatures.values().sum::<u64>(),
        judged.record_checker_red_states_after_a_fault,
        histories.len(),
        started.elapsed().as_secs_f64()
    );
    // 计数行只带数（门禁按空白切 key=value）；故障注入归类的逐格明细另打一行
    print_uncaptured(&format!(
        "── 故障注入归类 ──\n已知红 {:?}\n说谎的设备留下的 {:?}\n",
        judged.known_red_after_a_fault, judged.lying_device_signatures
    ));
    assert_eq!(histories_judged, histories.len(), "这一批每一段都判了");
    assert!(
        judged.findings.is_empty(),
        "清单外的失败（{} 条）：\n{}",
        judged.findings.len(),
        judged.findings.join("\n")
    );
    assert_eq!(
        red_items_of_the_engine_beyond_the_tables_the_gpu_may_not_judge(&run, &judged),
        Vec::<String>::new(),
        "引擎判不了的只许是 mkfs 没做完的那几张表"
    );
    assert_every_table_went_through_the_engine(&run, &judged);
    for invariant in ["I-3.1", "I-5.4", "I-3.11", "I-7.9", "I-9.15"] {
        assert!(
            judged.invariants_held_at_least_once.contains(invariant),
            "池级 checker 一次都没判成立过 {invariant}（全是不适用或判不到）"
        );
    }
    assert!(
        !judged.lying_device_signatures.is_empty(),
        "说谎的设备一次都没把盘面弄脏：说谎那两种抽到了却什么也没碰着"
    );
}

/// 开 GPU 的多机配置，只写开关：显卡先后照调用方给这个进程的那一份、经环境变量交给引擎（清场之下要清场之后那一份显卡先后；
/// [`small_tier_judged_under`] 设，`common_gpu_cards_of_the_caller` 文件头写了为什么不抄进配置）。
#[cfg(feature = "gpu")]
fn write_a_configuration_with_the_gpu_enabled(configuration: &std::path::Path) {
    std::fs::write(configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
}

/// 这一趟在环境守卫里判：多机配置指到 `configuration`、库在 `library`，开 GPU 时这台机器的角色与显卡先后照调用方给这个进程的那一份
/// （守卫把一趟读的变量连同机器角色一起清掉，不设回去第二台上的引擎会认成本机；显卡先后经环境变量交，配置只写开关）。
#[cfg(feature = "gpu")]
fn small_tier_judged_under(
    configuration: &std::path::Path,
    library: &std::path::Path,
    gpu_enabled: bool,
    fault: FaultPutOnTheGpuSide,
) -> (JudgedThroughTheLibrary, GivenStatesRun) {
    use singlefs_checker_tier::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_checker_tier::gpu_card_configuration::{
        GpuConfigurationOfTheCaller, CARDS_BY_PRIORITY_KEY, HOST_ROLE_ENVIRONMENT_VARIABLE,
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    };
    let host = GpuConfigurationOfTheCaller::of_this_process()
        .host()
        .expect("调用方给的机器角色是 local 或 peer");
    let cards =
        common_gpu_cards_of_the_caller::card_priority_value_of_the_caller(&repository_root());
    let mut settings: Vec<(&str, &std::ffi::OsStr)> = vec![
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        ("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", library.as_os_str()),
    ];
    if gpu_enabled {
        settings.push((
            HOST_ROLE_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new(host.name()),
        ));
        settings.push((CARDS_BY_PRIORITY_KEY, std::ffi::OsStr::new(&cards)));
    }
    let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    let (judged, _histories_judged, _spawned_worker_threads, run) =
        judge_through_the_given_states_engine(
            &RANDOM_HISTORY_SAMPLINGS_OF_THE_SMALL_RUN,
            &FAULT_INJECTION_SAMPLINGS_OF_THE_SMALL_RUN,
            fault,
            2,
        );
    drop(environment);
    (judged, run)
}

/// 同一小批的两趟：配了 GPU 的那一趟与没开 GPU 的那一趟，各自的账与引擎那一趟。
#[cfg(feature = "gpu")]
struct GpuRunAndCpuRun {
    on_the_gpu: (JudgedThroughTheLibrary, GivenStatesRun),
    on_the_cpu: (JudgedThroughTheLibrary, GivenStatesRun),
}

/// 同一小批一趟配了 GPU、一趟没开 GPU，各写一份自己的库（家目录下流水线单元测试那一类里以用例标签命名、起跑先清空、跑完留着）。
#[cfg(feature = "gpu")]
fn small_tier_judged_in_a_gpu_run_and_in_a_cpu_run(
    tag: &str,
    fault: FaultPutOnTheGpuSide,
) -> GpuRunAndCpuRun {
    let directory_of = |run: &str| {
        singlefs_checker_tier::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
            &format!("random-histories-and-fault-injection-given-states-{tag}-{run}"),
        )
        .expect("流水线单元测试那一类里这条用例的目录")
    };
    let gpu_directory = directory_of("on-the-gpu");
    let gpu_configuration = gpu_directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&gpu_configuration);
    let on_the_gpu = small_tier_judged_under(
        &gpu_configuration,
        &gpu_directory.join("library"),
        true,
        fault,
    );
    let cpu_directory = directory_of("on-the-cpu");
    let cpu_configuration = cpu_directory.join("multi-host-on-the-cpu.env");
    std::fs::write(&cpu_configuration, "ENABLE_GPU=0\n").expect("写得了配置");
    let on_the_cpu = small_tier_judged_under(
        &cpu_configuration,
        &cpu_directory.join("library"),
        false,
        FaultPutOnTheGpuSide::NoFault,
    );
    GpuRunAndCpuRun {
        on_the_gpu,
        on_the_cpu,
    }
}

/// GPU 与 CPU 对拍（用户 2026-10-11 定：独立的两趟，一段里 CPU 不既跑数据又对拍；用户 2026-10-13 定：两趟都从库里读）：
/// 同一小批一趟配了 GPU、由 GPU 判器判，一趟没开 GPU、由 CPU 判器判，各写各的库、从空库起；逐张表逐状态的对拍字相同，
/// 两趟建出的表与状态一样多，GPU 那一趟判不了的只许是 mkfs 没做完的那几张（它们交的是 CPU 判器的结论，对拍里不算 GPU 判过）。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
#[cfg(feature = "gpu")]
#[test]
fn the_small_tier_judged_in_a_gpu_run_and_in_a_cpu_run_gives_the_same_words_on_every_state() {
    use singlefs_checker_tier::crash_amplification::JudgedBy;
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_small_tier_judged_in_a_gpu_run_and_in_a_cpu_run_gives_the_same_words_on_every_state";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let GpuRunAndCpuRun {
        on_the_gpu: (gpu_judged, gpu_run),
        on_the_cpu: (cpu_judged, cpu_run),
    } = small_tier_judged_in_a_gpu_run_and_in_a_cpu_run("agrees", FaultPutOnTheGpuSide::NoFault);
    assert_eq!(
        gpu_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU 的那一趟：GPU 判器判"
    );
    assert_eq!(
        cpu_run.judged_by,
        JudgedBy::CpuJudge,
        "没开 GPU 的那一趟：CPU 判器判"
    );
    assert!(
        gpu_run.tally.states_judged_by_the_gpu_here > 0 && gpu_run.tally.states_reused == 0,
        "GPU 那一趟从空库起、状态都是这一趟在卡上判的：{:?}",
        gpu_run.tally
    );
    assert!(
        cpu_run.tally.states_judged_by_the_cpu_here > 0 && cpu_run.tally.states_reused == 0,
        "CPU 那一趟从空库起、状态都是这一趟判的：{:?}",
        cpu_run.tally
    );
    assert_eq!(
        red_items_of_the_engine_beyond_the_tables_the_gpu_may_not_judge(&gpu_run, &gpu_judged),
        Vec::<String>::new(),
        "GPU 那一趟判不了的只许是 mkfs 没做完的那几张表"
    );
    print_uncaptured(&format!(
        "RANDOM_HISTORIES_AND_FAULT_INJECTION_GPU_PAIRING tables={} states={} gpu_could_not_judge={}\n",
        gpu_judged.tables,
        gpu_judged.states,
        gpu_run.tally.states_the_gpu_could_not_judge.len()
    ));
    assert_eq!(
        (gpu_judged.tables, gpu_judged.states),
        (cpu_judged.tables, cpu_judged.states),
        "两趟建出的表与状态一样多"
    );
    let mismatches: Vec<String> = gpu_judged
        .words_by_table
        .iter()
        .filter(|(table_name, gpu_words)| {
            cpu_judged.words_by_table.get(*table_name) != Some(gpu_words)
        })
        .map(|(table_name, gpu_words)| {
            format!(
                "{table_name}：GPU {gpu_words:?}；CPU {:?}",
                cpu_judged.words_by_table.get(table_name)
            )
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} 张表上 GPU 一趟与 CPU 一趟的对拍字不同：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        gpu_judged.findings, cpu_judged.findings,
        "两趟的失败逐条相同"
    );
}

/// 上一条的判别力：往 GPU 那一侧塞一个已知故障（每张表第一个状态的第一条不变量违例位翻过来），GPU 那一趟的对拍字与 CPU 那一趟不同。
#[cfg(feature = "gpu")]
#[test]
fn a_fault_put_on_the_gpu_side_of_the_small_tier_makes_the_gpu_run_differ_from_the_cpu_run() {
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "a_fault_put_on_the_gpu_side_of_the_small_tier_makes_the_gpu_run_differ_from_the_cpu_run";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let GpuRunAndCpuRun {
        on_the_gpu: (gpu_judged, _gpu_run),
        on_the_cpu: (cpu_judged, _cpu_run),
    } = small_tier_judged_in_a_gpu_run_and_in_a_cpu_run(
        "fault",
        FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
    );
    assert_ne!(
        gpu_judged.words_by_table, cpu_judged.words_by_table,
        "GPU 那一侧塞了故障，两趟的对拍字却相同"
    );
}
