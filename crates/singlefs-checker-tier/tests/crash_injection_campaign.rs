//! checker 档模块：crash、crash_injection
//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 3 件：崩溃注入。生成器、执行器与理想模型在第 1、2 件（`singlefs_harness::history`、
//! `singlefs_harness::model`），切段、摆崩溃状态、重建与判定在 `singlefs_checker_tier::crash_injection`；
//! 这里是快档（普通 `cargo test`）、多线程对拍、写死的那几段历史（段内真子集全枚举）与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 枚举域与层 0 同一个（屏障与 FUA 写切段、段内任意真子集）；差别在怎么取：层 0 在两条写死的流上全枚举（门禁 checker-tier-layer0-replay），
//! 这里在随机历史上抽样。种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史那个二进制用的是同一个），
//! 打在报告第一行与失败信息里（随机说的是一个测试周期与下一个之间，不是每次跑重抽）。

use std::collections::BTreeMap;
use std::io::Write as _;

use singlefs_checker::walk::InstanceTableOfRootRecord;
use singlefs_checker_tier::crash::{check_records, check_records_against};
use singlefs_checker_tier::crash_injection::{
    crash_injection_campaign_from_environment, inject_crashes_into_history,
    run_crash_injection_campaign, CrashInjectionAssertions, CrashInjectionCampaign,
    CrashInjectionReport, CrashInjectionTally, CrashInjectionWorkerThreads, CrashPoint,
    CrashPointDraw, CrashPointFinding, CrashStateStage, FailureImageRetention,
    HistoryCrashInjection,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader};
use singlefs_format::DATA_UNIT_BYTES;
use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    environment_assignments_in_line, execute_history_with, generate_history,
    generate_history_with_weights, ContentChoice, ContentLength, FailureObservation,
    FailureSignature, FloorTargetChoice, GeneratedHistory, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed,
    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome, StepPosition,
};
use singlefs_harness::memory_pool::{
    root_identity_written_by, writes_and_segments, CrashImage, MemoryPool, RecordCheck,
    RecordStreamContinuity, RetainedWrite,
};
use singlefs_harness::numeric_environment_variable::process_environment_variable;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 快档的规模：段数、每段步数与每段摆几个崩溃状态。规模与种子基都写死（种子基是这个测试周期的常量）。
/// 每个崩溃状态上起可写挂载、发一次布、摆三个二次崩溃之后（代码审阅第 2 条），这一档 debug 下单跑一百来秒，标了 ignore，
/// 用户要求时由主 agent 经门禁 checker-tier-layer0-replay `--full`（`crash-case:crash-injection-fast-tier`）在 release 下跑，提交时的快档只核它的全绿标记；普通 `cargo test` 由小快档守。
const FAST_TIER_SEEDS: u64 = 24;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 24;
const FAST_TIER_DRAW: CrashPointDraw = CrashPointDraw::Sampled {
    crash_points_per_history: 4,
};

/// 小快档的段数：快档的头几段（每段步数、每段几个崩溃状态、比重与快档相同），普通 `cargo test` 里几秒跑完。
/// 头几段里有种子基 + 2 那一段：调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条那一形
/// （恢复落到由 journal 记录重建的一版、它的根槽写在更晚的段）就摆在那一段上。
const SMALL_FAST_TIER_SEEDS: u64 = 4;
/// 崩溃注入这一段历史本身怎么跑：不跑每一步的池级 checker，两块 4 GiB 的盘。
/// 活盘面上每一步的 checker 由随机历史那五段（门禁 74 号）罩着，这一段的预算全给崩溃状态。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 判红时先把种子基说清楚：同一个构建加同一个种子基重放得出来，报告里不带它就没法复现。
/// 重放行由报告自己拼（`CrashInjectionReport::replay_line`：种子区间、崩溃状态怎么摆、照哪一组断言判、步数、比重、checker、盘宽、空间准入），
/// 大档那条入口照同一张变量表读回（`crash_injection_campaign_from_environment`），照同一组断言判（[`judge_crash_injection_report`]）。
fn how_to_replay(report: &CrashInjectionReport) -> String {
    format!("重放：{}", report.replay_line())
}

/// 种子基是一个测试周期的常量：开一个周期抽一次，抽完在这个周期之内写死。周期从「开一个新里程碑」与
/// 「用户显式说从零开始测试 / 重建测试」里先到的那一件开始（用户 2026-09-20 定案第 7 条与同日定的范围：
/// 「我说的随机是 多次实验的随机  比如 里程碑 1 和里程碑 2  而不是 每个里程碑里面都要随机」）。
///
/// 这一条钉两样：
/// 1. 常量就是当前这个周期（里程碑「覆盖写、释放、回退与复用」）2026-09-20 抽出来的那个数。改成别的数 = 把这个周期的实验数据整批换掉，
///    而 `crates/mutations.tsv` 第 146–178 行那些「这条变异必须红」与里程碑那几条验收，量的都是这一批历史；
/// 2. 从它起跑出来的报告把它带了回来（报告第一行与重放那一行都带着它），判红了才重放得出来。
///
/// 随机历史那个二进制的五段用的是同一个常量——两边同基，那一半由它自己那条用例钉
/// （`random_histories.rs` 的 `the_five_sampling_tiers_start_from_the_test_cycle_seed_base`）。
#[test]
fn the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle() {
    assert_eq!(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        7_463_871_032_432_355_113,
        "种子基不是 2026-09-20 抽的那个数：它永久固定，换了就让绑在这批历史上的判红、镜像与过程文件整批作废\
         （为什么固定、要多测历史怎么追加区间，写在 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE 的文档注释里）"
    );
    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    });
    assert_eq!(
        report.first_seed, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        "报告没把跑的那个种子基带回来"
    );
    let replay = how_to_replay(&report);
    assert!(
        replay.contains(&SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.to_string()),
        "重放那一行里没有种子基，判红了就重放不出来：{replay}"
    );
    assert!(
        report.render().contains("这个测试周期写死的"),
        "报告第一行没说清种子基是这个测试周期写死的：\n{}",
        report.render()
    );
}

/// 快档：这个测试周期的种子基起 24 段、每段 24 步、每段摆 4 个崩溃状态。每个崩溃状态重建镜像、跑恢复、跑池级 checker、
/// 跑记录核对器，并问模型「恢复到的这一版允不允许」；「已知红」清单里的形态照记不停，清单外的一条都不许有。
/// 计数照打，并核各条路径真的跑到了：四种落点都扣下过、段内摆出过洞、起点那一段摆到过、截在发布中间过、
/// 截回到最后一版之前过、恢复三种结局见过两种（失败一次都不许有）、journal 记录真的施加过、模型真的比过内容。
#[test]
#[ignore = "快档：每个崩溃状态上起可写挂载、发一次布、摆三个二次崩溃之后 debug 下单跑一百来秒；用户要求时由主 agent 经门禁 checker-tier-layer0-replay --full 按 crash-case:crash-injection-fast-tier 在 release 下跑、提交时的快档只核它的全绿标记，普通 cargo test 由 crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed 守"]
fn crash_injection_fast_tier_recovers_only_into_versions_the_model_committed() {
    let started = std::time::Instant::now();
    let report = run_crash_injection_campaign(&fast_tier_campaign(
        CrashInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    judge_crash_injection_report(&report);
}

/// 小快档：快档的头 [`SMALL_FAST_TIER_SEEDS`] 段（每段步数、每段几个崩溃状态、比重与快档相同），普通 `cargo test` 里守着崩溃注入这一路。
/// 清单外的失败一条都不许有；三截每一截在每个崩溃状态上都真的跑了（每个崩溃状态都问过模型、跑过池级 checker 与记录核对器，
/// 都起过可写挂载、之后的池都跑过 checker 与记录核对器；二次崩溃摆出来过，每个都问过模型、跑过 checker 与记录核对器）。
/// 只钉这些「每个崩溃状态都做了什么」的结构计数，不钉抽样摆出来的形态（那是快档的事，头几段里未必每一形都有）。
#[test]
fn crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed() {
    let started = std::time::Instant::now();
    let report = run_crash_injection_campaign(&small_fast_tier_campaign(
        CrashInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入小快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    judge_crash_injection_report(&report);
}

/// 小快档那一批：快档的头 [`SMALL_FAST_TIER_SEEDS`] 段，按小快档那一组断言判。
fn small_fast_tier_campaign(worker_threads: CrashInjectionWorkerThreads) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: SMALL_FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        draw: FAST_TIER_DRAW,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::SmallFastTier,
    }
}

/// 照这一批的那一组断言判（[`CrashInjectionAssertions`]）：出红的那条用例与照它的重放行跑的大档入口都经这一个函数，
/// 所以照抄重放行跑，红在同一条断言上（代码三方 test-script-align-code-r1 判决 S2）。
fn judge_crash_injection_report(report: &CrashInjectionReport) {
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{}\n{}",
        how_to_replay(report),
        report.render()
    );
    match report.assertions {
        CrashInjectionAssertions::NewFindingsOnly => {
            assert_at_least_one_crash_state_was_judged(report);
        }
        CrashInjectionAssertions::SmallFastTier => {
            assert_the_small_fast_tier_ran_each_stage_on_every_crash_state(report);
        }
        CrashInjectionAssertions::FastTier => {
            assert_every_crash_injection_path_was_exercised(report);
        }
        CrashInjectionAssertions::RollbackSampling => {
            assert_the_rollback_sampling_placed_crash_states_and_recovered_from_each(report);
        }
    }
}

/// 只判清单外失败的那一组也要判过至少一个崩溃状态：一个都没判过的一批没有清单外失败，只是因为什么都没判
/// （上游 `show-me-test.md`「扫到 0 项也不是通过」）。别的三组各自要的计数都不少于段数，已经含着这一条。
fn assert_at_least_one_crash_state_was_judged(report: &CrashInjectionReport) {
    assert!(
        report.tally.crash_points > 0,
        "这一批一个崩溃状态都没判过，扫到 0 项不算通过；{}",
        how_to_replay(report)
    );
}

/// 一段都不跑的一批照只判清单外失败的那一组判：判红，不因为没有清单外失败就判成绿。
#[test]
#[should_panic(expected = "这一批一个崩溃状态都没判过")]
fn a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_assertions() {
    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 0,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    });
    judge_crash_injection_report(&report);
}

/// 小快档那一组的结构计数：三截每一截在每个崩溃状态上都真的跑了，二次崩溃摆出来过、每个都判过，恢复没失败过。
fn assert_the_small_fast_tier_ran_each_stage_on_every_crash_state(report: &CrashInjectionReport) {
    let replay = how_to_replay(report);
    let tally = &report.tally;
    // 每段历史都摆出过崩溃状态（总数够放得过「几段一个都没有、别的段多摆」，判决 `m2-supp3-item3-code-r2-main-verification.md`
    // 第四节第 4 题，用户定「全补」；为什么每段都摆得出见快档那一组）。
    assert_eq!(
        tally.histories_without_any_crash_point, 0,
        "有历史段一个崩溃状态都没摆出来；{replay}"
    );
    assert!(
        tally.crash_points >= SMALL_FAST_TIER_SEEDS,
        "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
        tally.crash_points
    );
    for (stage, runs) in [
        ("问模型", tally.model_judgements),
        ("池级 checker", tally.checker_runs),
        ("记录核对器", tally.record_checks),
        (
            "崩溃后镜像上的可写挂载",
            tally.writable_mounts_after_the_crash,
        ),
        (
            "可写挂载之后的池上 checker",
            tally.checker_runs_after_the_writable_mount,
        ),
        (
            "可写挂载之后的池上记录核对器",
            tally.record_checks_after_the_writable_mount,
        ),
    ] {
        assert_eq!(
            runs, tally.crash_points,
            "每个崩溃状态都要跑过{stage}；{replay}"
        );
    }
    assert!(
        tally.second_crash_points >= 1,
        "挂载途中一个二次崩溃都没摆出来；{replay}"
    );
    for (stage, runs) in [
        ("问模型", tally.second_crash_model_judgements),
        ("池级 checker", tally.second_crash_checker_runs),
        ("记录核对器", tally.second_crash_record_checks),
    ] {
        assert_eq!(
            runs, tally.second_crash_points,
            "每个二次崩溃都要跑过{stage}；{replay}"
        );
    }
    assert_eq!(tally.recoveries_failed, 0, "崩溃之后恢复失败过；{replay}");
}

/// 快档那一批：这个测试周期的种子基起 [`FAST_TIER_SEEDS`] 段、每段 [`FAST_TIER_OPERATIONS_PER_HISTORY`] 步、[`FAST_TIER_DRAW`]、
/// 各类操作都抽。门禁这一档跑完把镜像目录删掉：临时目录里不留东西（门禁 77 号判跑完机器干不干净）。
fn fast_tier_campaign(worker_threads: CrashInjectionWorkerThreads) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        draw: FAST_TIER_DRAW,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::FastTier,
    }
}

/// 快档那一组：各条路径都跑到了（四种落点都扣下过、段内摆出过洞、起点那一段摆到过、截在发布中间过……）。
fn assert_every_crash_injection_path_was_exercised(report: &CrashInjectionReport) {
    let tally = &report.tally;
    let replay = how_to_replay(report);
    // 总数够不说明每段都摆到了：`crash_points >= FAST_TIER_SEEDS` 放得过「几段一个都没有、别的段多摆」（判决
    // `m2-supp3-item3-code-r2-main-verification.md` 第四节第 4 题，用户定「全补」）。从第一个文件起的历史，起点那一段（取号、暖机、
    // 第一个文件）就有写、都是候选段；从 mkfs 起的历史要等第一次做成的可写挂载，关着会话时只抽可写挂载与冷启动两种，
    // 一次可写挂载都没做成就收尾的（新发现或已知红收尾在那之前）这一条照判红。推的，没在快档上跑过。
    assert_eq!(
        tally.histories_without_any_crash_point, 0,
        "有历史段一个崩溃状态都没摆出来；{replay}"
    );
    assert!(
        tally.crash_points >= FAST_TIER_SEEDS,
        "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
        tally.crash_points
    );
    assert_eq!(
        tally.model_judgements, tally.crash_points,
        "每个崩溃状态都要问过模型；{replay}"
    );
    assert_eq!(
        tally.checker_runs, tally.crash_points,
        "每个崩溃状态都要跑过池级 checker；{replay}"
    );
    assert_eq!(
        tally.record_checks, tally.crash_points,
        "每个崩溃状态都要跑过记录核对器；{replay}"
    );
    for kind in [
        StepKind::UnitWrite,
        StepKind::JournalRecord,
        StepKind::RootRecordFua,
        StepKind::SystemConfigurationSlot,
    ] {
        assert!(
            tally
                .crash_points_withholding_write_kind
                .get(kind.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "一次都没扣下 {}：{:?}；{replay}",
            kind.name(),
            tally.crash_points_withholding_write_kind
        );
    }
    for kind in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
    ] {
        assert!(
            tally
                .crash_points_by_operation_kind
                .get(&kind)
                .copied()
                .unwrap_or(0)
                >= 1,
            "一个崩溃状态都没落在 {kind:?} 里：{:?}；{replay}",
            tally.crash_points_by_operation_kind
        );
    }
    assert!(
        tally.crash_points_withholding_a_write_before_a_persisted_one >= 1,
        "一个段内有洞的状态都没摆出来（等于还是只截前缀）；{replay}"
    );
    assert!(
        tally.crash_points_inside_the_starting_point >= 1,
        "起点那一段里一个崩溃状态都没摆到（2026-09-20 才放开）；{replay}"
    );
    assert!(
        tally.crash_points_inside_an_unfinished_publish >= 1,
        "一次都没把发布截在它的根落盘之前；{replay}"
    );
    assert!(
        tally.crash_points_that_land_before_the_last_committed_version >= 1,
        "一个崩溃状态都没截回到最后一版之前（全落在流的尾巴上）；{replay}"
    );
    assert_eq!(tally.recoveries_failed, 0, "崩溃之后恢复失败过；{replay}");
    assert!(
        tally.recoveries_reading_a_file >= 1,
        "崩溃之后一次都没读回文件；{replay}"
    );
    assert!(
        tally.recoveries_without_a_file >= 1,
        "崩溃之后一次都没落在树表 0 条的一版上；{replay}"
    );
    assert!(
        tally.recoveries_that_applied_journal_records >= 1,
        "恢复一次都没施加过 journal 记录前缀（journal 在这一段里没承重）；{replay}"
    );
    assert!(
        tally.model_contents_compared >= 1,
        "模型一次都没比过崩溃之后读回的内容；{replay}"
    );
    assert!(
        tally.model_versions_without_a_file_matched >= 1,
        "模型一次都没对上「这一版树表 0 条」；{replay}"
    );
    for invariant in ["I-3.1", "I-5.4"] {
        assert!(
            tally.invariant_holds.get(invariant).copied().unwrap_or(0) >= 1,
            "崩溃状态上 checker 一次都没判过 {invariant}（全是不适用）；{replay}"
        );
    }
}

/// 多线程只影响跑得多快：同一批种子，1 个线程与 4 个线程跑出来的报告逐字相同
/// （`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
/// 计数按片的次序相加、新发现按种子留第一个，所以结论与线程数、调度次序无关；进度行按到达次序打，不带判定，不在比对之内。
/// 两遍拿的是同一个种子基（这个测试周期写死的那一个），比的就是同一批历史上的同一批崩溃状态。
#[test]
fn one_thread_and_four_threads_render_the_same_report() {
    let first_seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    let campaign = |worker_threads: usize| {
        run_crash_injection_campaign(&CrashInjectionCampaign {
            first_seed,
            seed_count: 8,
            operations_per_history: 16,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 3,
            },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(worker_threads),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        })
        .render()
    };
    let on_one_thread = campaign(1);
    let on_four_threads = campaign(4);
    assert_eq!(
        on_one_thread, on_four_threads,
        "线程数不改结论（种子基 {first_seed}）"
    );
    assert!(
        on_one_thread.contains("崩溃状态 "),
        "报告里有崩溃状态的计数：\n{on_one_thread}"
    );
}

/// 写死的一段历史：mkfs → 可写挂载 → 第一个文件 → 覆盖写 → 可写挂载 → 覆盖写；写数不超过 4 的段整段枚举（全部真子集），
/// 更长的段各抽 8 个。今天的代码上每个崩溃状态都恢复到模型提交过的某一版、池级 checker 与记录核对器一条都不红。
///
/// 判别力靠的就是这一条：段内真子集全枚举，所以「记录写持久了而根槽没持久」与「根槽持久了而它那次发布的记录一条都不在」
/// 两类状态都摆得出来——少一道屏障（`crates/mutations.tsv`「步 3：零单元发布在记录与根之间少一道屏障」）就把两段并成一段，
/// 后一类立刻摆得出，记录核对器判红。种子写死不随机：这一条判的是枚举域罩不罩得住，不是抽样运气。
#[test]
fn every_crash_state_of_a_written_out_history_recovers_into_a_committed_version() {
    let content = |selector: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector },
            fill_seed: selector,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            content(5),
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::CloseAndMountWritable,
            content(7),
        ],
    };
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 4,
            sampled_in_longer_segments: 8,
        },
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：写死的一段历史，段内真子集全枚举 ──\n{}",
        injection.tally.render()
    ));
    assert!(
        injection.new_findings.is_empty(),
        "写死的这段历史上崩溃状态判红：{:#?}",
        injection.new_findings
    );
    // 代码审阅第 19 条之后取号写之前多一道屏障：第二次可写挂载那一处，上一次发布末尾的轮换与取号那 4 写一段拆成 2、2，
    // 真子集 15 → 3 + 3，这段历史摆得出的崩溃状态 104 → 95（屏障只拆段、不添状态）。
    assert!(
        injection.tally.crash_points >= 95,
        "这段历史摆得出的崩溃状态不少于 95 个：{}",
        injection.tally.crash_points
    );
    assert_eq!(
        injection.tally.recoveries_failed, 0,
        "每个崩溃状态都恢复得出来"
    );
    assert!(
        injection.tally.crash_points_inside_an_unfinished_publish >= 4,
        "每次发布的单元写与记录写都截在它的根落盘之前：{}",
        injection.tally.crash_points_inside_an_unfinished_publish
    );
    assert!(
        injection
            .tally
            .crash_points_withholding_write_kind
            .get(StepKind::JournalRecord.name())
            .copied()
            .unwrap_or(0)
            >= 1,
        "一次都没把 journal 记录写扣下（少一道屏障那条变异就判不出来）：{:?}",
        injection.tally.crash_points_withholding_write_kind
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "今天的代码上没有「根在而它那次发布的记录一条都不在」的状态"
    );
    assert_eq!(
        injection.tally.crash_states_ending_new_finding, 0,
        "一个崩溃状态都不许是新发现"
    );
}

/// 「已知红」清单那一条（增补 2 收口表第 43 行）原先在崩溃状态上的复现那一段历史（与随机历史那边
/// `the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback` 逐项相同），两次回退换成挂着时的向前回退：
/// 没有回退留下的空档，抬 F 那几步之后的崩溃状态上一条清单外的失败都不许有；回退与抬 F 的写上都摆得到崩溃状态。
/// 清单那一形（F 落在被抛弃实例留下的空档里）今天只剩崩溃恢复抛弃的时间线造得出，随机历史与崩溃注入都没有这一种操作，
/// 它在崩溃状态上认不认得出来（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阳那一半）这一段不再罩，交主 agent（实三报告）。
#[test]
fn crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback(
) {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(80),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 8,
            }),
        ],
    };
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 1,
        },
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：原先抬 F 落进回退空档那一段历史（向前回退） ──\n{}",
        injection.tally.render()
    ));
    assert!(
        injection.new_findings.is_empty(),
        "一条清单外的失败都不许有：{:#?}",
        injection.new_findings
    );
    // 「回退与抬 F 的写上也摆得到崩溃状态」这一条钉在这里，不钉在随机种子那一段上：
    // 这段历史是写死的，回退两次、抬 F 一次都在里面，段内真子集又是全枚举的，所以这个结论不靠抽样运气。
    for kind in [
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            injection
                .tally
                .crash_points_by_operation_kind
                .get(&kind)
                .copied()
                .unwrap_or(0)
                >= 1,
            "一个崩溃状态都没落在 {kind:?} 里：{:?}",
            injection.tally.crash_points_by_operation_kind
        );
    }
}

/// 各类操作的写上摆到几个崩溃状态（判决 `m2-supp3-item3-code-r2-main-verification.md` 第四节第 4 题，用户定「全补」：七类操作各钉一条；
/// 这之后又加了崩溃恢复抛弃根，共八类）。覆盖写与可写挂载由快档钉、回退与抬 F 由上一条写死历史钉，剩下四类钉在这一段写死的历史上：
/// mkfs → 可写挂载 → 零单元发布（树表 0 条的一版上才调得成）→ 冷启动 → 可写挂载 → 第一个文件 → 崩溃恢复抛弃根（C554 乙拒）→ 可写挂载。
/// 写数不超过 2 的段整段枚举（2^n − 1 个真子集），更长的段各抽 1 个，所以每一类摆到几个是定数，不靠抽样运气：
/// 零单元发布的段是 2、1、2 ⇒ 3 + 1 + 3 = 7；第一个文件的段是 24、2、1、2 ⇒ 1 + 3 + 1 + 3 = 8；
/// 冷启动只读盘、一个写都不发，崩溃恢复抛弃根在取号之前被拒、也不发写 ⇒ 两类都是 0（崩溃状态按第一个被扣下的写落在哪一步归类，
/// 不发写的那一步归不到）。段序列是 2026-10-03 在 harness 侧按同一段历史、同一个执行（`execute_history_with` 加
/// `writes_and_segments_with_stream_indexes`）现量的，这一条在 checker 档里没在交回之前跑过。
/// 比已有的两段写死历史多罩的：树表 0 条的一版上的零单元发布、冷启动之后与崩溃恢复被拒之后的可写挂载那几段的崩溃状态。
#[test]
fn crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert_eq!(
        crash_recovery_outcomes,
        vec![StepOutcome::Refused {
            member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
        }],
        "崩溃恢复抛弃根那一步被 C554 乙拒：它归到 0 个崩溃状态说的是「被拒的那一步不发写」"
    );
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 1,
        },
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：零单元发布、第一个文件、冷启动与崩溃恢复抛弃根各摆几个崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    let crash_points_in = |kind: HistoryOperationKind| {
        injection
            .tally
            .crash_points_by_operation_kind
            .get(&kind)
            .copied()
            .unwrap_or(0)
    };
    assert_eq!(
        crash_points_in(HistoryOperationKind::ColdStartRecover),
        0,
        "冷启动一个写都不发，崩溃状态归不到它：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot),
        0,
        "崩溃恢复抛弃根在取号之前被拒、一个写都不发，崩溃状态归不到它：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::PublishWithoutUnits),
        7,
        "零单元发布的段 2、1、2：3 + 1 + 3：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::PublishFirstFile),
        8,
        "第一个文件的段 24、2、1、2：1 + 3 + 1 + 3：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    // 计数先钉、判定后钉：计数只看段序列与归步，少一道屏障那类变异先红在上面的计数上（`crates/mutations.tsv`）。
    assert!(
        injection.new_findings.is_empty(),
        "一条清单外的失败都不许有：{:#?}",
        injection.new_findings
    );
}

/// 一段历史上写出的每次发布：它写出的单元写（写表下标）与它的根身份 (实例代号, checkpoint_txg)。
/// 归法与记录核对器的相同：一次根槽写之前、上一次根槽写之后的单元写都归这一次。
fn unit_writes_of_each_publish(writes: &[RetainedWrite]) -> Vec<((u32, u64), Vec<usize>)> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    for (index, write) in writes.iter().enumerate() {
        match write.kind {
            StepKind::UnitWrite => units.push(index),
            StepKind::RootRecordFua => {
                let (instance, checkpoint_txg) =
                    root_identity_written_by(write.bytes().expect("根槽写带着字节"));
                publishes.push(((instance.0, checkpoint_txg.0), std::mem::take(&mut units)));
            }
            StepKind::JournalRecord
            | StepKind::ZeroFill
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    publishes
}

/// 这次单元写写下的字节还在不在 `image` 上。
fn unit_write_is_still_on_disk(image: &CrashImage<'_>, write: &RetainedWrite) -> bool {
    let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
    PoolReader::read(image, write.device, write.offset, length).as_deref() == write.bytes()
}

/// 记录核对器不要求被抛弃时间线上那次发布的单元还在（实七报告 (c)，实八；复现的随机种子是崩溃注入快档种子基 + 2）：
/// 第一个文件（txg 3）之后覆盖写一次（txg 4），崩溃恢复抛弃它（新实例 2 写行 (1, 3, 0)），再覆盖写几次——实例 2 看不见 (1, 4)，
/// 它的数据单元那一对槽被后来的发布复用。整条流都落了盘的那个状态上，恢复落到最新那一版，它的实例表判 (1, 4) 被抛弃
/// （行 (1, 3) 且 4 > 3，D23（journal 的角色与格式） 已定项 14），(1, 4) 写出的单元不算「该在」，记录核对器不判红。
/// 判别力的两半：同一条流上把没被抛弃的发布的一个单元的两份都扣下，照判红——一次是 (1, 3)（行 (1, 3) 的边上，T = Ti 不算被抛弃），
/// 一次是最后那次覆盖写。
///
/// **C554 乙之后「崩溃恢复抛弃根」这一步造不出被抛弃的根**（合入后验证一，主 agent 2026-09-27 定改钉「这一形不可达」）：(1, 4) 是这个会话发的，
/// 系统配置见证过它；这一步让它的根槽与单元读回全 0 之后，可写挂载读阶段判据 N-配置 为真、重读一次仍为真，按 C554 乙拒
/// （`MountError::NewerStateStillUnreadableAfterOneReread`，`history.rs` 的 `apply_crash_recovery_abandoning_the_newest_root` 文档写着），
/// 不开新实例、不写抛弃行，后面三次覆盖写没有可写会话、都不适用。第一轮快照（`refs/sop/m2-closeout-code-r1-snapshot`，C554 乙合入之前）上这一条绿，
/// 今天的底座上把 C577 那道屏障去掉它照样造不出（探针量过，合入后验证一报告）：不可达是 C554 乙定的，不是 C577。
/// 这一条保留原来的构造（同一串步），改钉今天的样子：整条流都落了盘的那个状态上恢复落到 (1, 4)（实例 1），流里没有实例 2 的发布，
/// 记录核对器不判红；判别力那两次扣单元照旧，扣的是 (1, 3) 与流里最后一次发布 (1, 4)。把 C554 乙那一判关掉，挂载照旧做成、落到实例 2，
/// 红在「落到实例 1」那一条（`crates/mutations.tsv`）。
#[test]
fn units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red(
) {
    let overwrite = |fill_seed: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite(1),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            overwrite(2),
            overwrite(3),
            overwrite(4),
        ],
    };
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let units_of = |identity: (u32, u64)| -> &Vec<usize> {
        &publishes
            .iter()
            .find(|(written, _)| *written == identity)
            .unwrap_or_else(|| panic!("流里有 {identity:?} 那次发布：{publishes:?}"))
            .1
    };
    let every_write_landed = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    let report = recover(&every_write_landed, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "C554 乙之后崩溃恢复抛弃根那一步的可写挂载被拒：落到实例 1 的 (1, 4)，没有实例 2（C554 乙之前落到实例 2、比 (1, 4) 新的一版）"
    );
    assert!(
        publishes.iter().all(|((instance, _), _)| *instance <= 1),
        "流里只有 mkfs（实例 0）与实例 1 的发布、没有实例 2 的：{publishes:?}"
    );
    assert_eq!(
        check_records(&every_write_landed, report.effective_root),
        RecordCheck::default(),
        "整条流都落了盘：记录核对器一条都不判红"
    );

    let last_overwrite = publishes.last().expect("流里至少一次发布").0;
    for kept in [(1, 3), last_overwrite] {
        let data_unit = *units_of(kept)
            .iter()
            .find(|unit| writes[**unit].length_in_bytes() == DATA_UNIT_BYTES)
            .unwrap_or_else(|| panic!("{kept:?} 那次发布写了数据单元"));
        // 这次发布写在这个偏移上的每一份（两块盘各一份）都扣下。
        let mut persisted = vec![true; writes.len()];
        for unit in units_of(kept) {
            if writes[*unit].offset == writes[data_unit].offset {
                persisted[*unit] = false;
            }
        }
        let a_kept_unit_withheld = CrashImage {
            base: &base,
            writes: &writes,
            persisted,
        };
        let recovery_with_the_unit_withheld =
            recover(&a_kept_unit_withheld, JournalPolicy::Consult);
        assert!(
            recovery_with_the_unit_withheld
                .effective_root
                .is_some_and(|(_, txg)| txg.0 >= kept.1),
            "恢复落到的那一版不早于 {kept:?}：{:?}",
            recovery_with_the_unit_withheld.effective_root
        );
        assert!(
            check_records(
                &a_kept_unit_withheld,
                recovery_with_the_unit_withheld.effective_root
            )
            .claimed_state_missing_unit,
            "{kept:?} 没被抛弃，它的数据单元两份都没落，照判红"
        );
    }
}

/// 一次根槽写写出的根身份 (实例代号, checkpoint_txg)；别的写是 None。
fn root_identity_of_a_root_slot_write(
    write: &RetainedWrite,
) -> Option<(InstanceGeneration, CheckpointTxg)> {
    match write.kind {
        StepKind::RootRecordFua => Some(root_identity_written_by(
            write.bytes().expect("根槽写带着字节"),
        )),
        StepKind::UnitWrite
        | StepKind::JournalRecord
        | StepKind::ZeroFill
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => None,
    }
}

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条那一形所在的历史：种子基 + 2，
/// 快档的步数与比重（快档与小快档都跑它，按 [`FAST_TIER_DRAW`] 摆的崩溃状态也与它们相同）。
fn history_with_a_crash_state_landing_on_a_version_rebuilt_from_records() -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 2),
        FAST_TIER_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    )
}

/// C554 乙之后崩溃恢复抛弃根那一步的结局：可写挂载读阶段判据 N-配置 为真、重读一次仍为真，拒可写（`history.rs` 的
/// `apply_crash_recovery_abandoning_the_newest_root`）。
const CRASH_RECOVERY_REFUSED_BY_THE_WITNESS: &str =
    "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)";

/// `history` 在崩溃注入那一种执行下，每一步「崩溃恢复抛弃根」的结局，按次序。
fn outcomes_of_the_crash_recovery_steps(history: &GeneratedHistory) -> Vec<StepOutcome> {
    let run = execute_history_with(
        history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    history
        .operations
        .iter()
        .zip(run.outcomes)
        .filter(|(operation, _)| {
            matches!(
                operation,
                HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
            )
        })
        .map(|(_, outcome)| outcome)
        .collect()
}

/// 与整条流（`write_count` 个写）逐条对应的持久集合：崩溃点所在段之前整段持久、
/// 这一段按崩溃点的子集、之后的一个都没持久——崩溃注入交给记录核对器的那一种。
fn persisted_over_the_whole_stream(
    write_count: usize,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: &[bool],
) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    persisted[..first_write_of_the_segment].fill(true);
    persisted[first_write_of_the_segment
        ..first_write_of_the_segment + persisted_within_the_segment.len()]
        .copy_from_slice(persisted_within_the_segment);
    persisted
}

/// 两块空盘上叠 `writes_before_the_segment` 里在持久集合 `persisted`（整条流那一份，逐条对应）里的写。
fn pool_of_the_persisted_writes_before_the_segment(
    writes_before_the_segment: &[RetainedWrite],
    persisted: &[bool],
    device_bytes: u64,
) -> MemoryPool {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], device_bytes);
    for (write, is_persisted) in writes_before_the_segment.iter().zip(persisted) {
        if *is_persisted {
            base.apply_writes(std::slice::from_ref(write));
        }
    }
    base
}

/// 在 `history` 的整条录制流上，按 `crash_points`（这段历史上崩溃注入摆出来的那几个）逐个重建崩溃镜像、跑恢复，
/// 挑出第一个这样的崩溃状态：恢复施加过 journal 记录、落到的那一版的根槽写不在到这一段为止的前缀里（在更晚的段），
/// 那一版的实例表读得出、抛弃了更早的一次发布，而那次发布写在某个偏移上的单元每一份都已不在盘上（被后来的发布复用盖掉）。
/// 交回落到的那一版与被它的实例表抛弃的那次发布；一个这样的崩溃状态都没有时 `None`（C554 乙之后这一形不可达，见用到它的两条用例）。
fn crash_state_landing_on_a_version_rebuilt_from_records(
    history: &GeneratedHistory,
    crash_points: &[CrashPoint],
) -> Option<((InstanceGeneration, CheckpointTxg), (u32, u64))> {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    execute_history_with(history, execution, &stream, &mut |_| {});
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let device_bytes = execution.device_width.device_bytes();
    crash_points.iter().find_map(|crash_point| {
        let segment = &segments[crash_point.segment_index];
        assert_eq!(
            segment.len(),
            crash_point.persisted_within_the_segment.len(),
            "这里切出的第 {} 段与崩溃注入那一路切出的写数相同",
            crash_point.segment_index
        );
        let first_write_of_the_segment = segment[0];
        let end_of_the_segment = first_write_of_the_segment + segment.len();
        let persisted = persisted_over_the_whole_stream(
            writes.len(),
            first_write_of_the_segment,
            &crash_point.persisted_within_the_segment,
        );
        let base = pool_of_the_persisted_writes_before_the_segment(
            &writes[..first_write_of_the_segment],
            &persisted,
            device_bytes,
        );
        let image = CrashImage {
            base: &base,
            writes: &writes[first_write_of_the_segment..end_of_the_segment],
            persisted: crash_point.persisted_within_the_segment.clone(),
        };
        let report = recover(&image, JournalPolicy::Consult);
        let landed_version = report.effective_root?;
        if report.journal.prefix_applied == 0
            || writes[..end_of_the_segment]
                .iter()
                .any(|write| root_identity_of_a_root_slot_write(write) == Some(landed_version))
        {
            return None;
        }
        let root_write_of_the_landed_version = writes
            .iter()
            .rposition(|write| root_identity_of_a_root_slot_write(write) == Some(landed_version))?;
        let table = InstanceTableOfRootRecord::read(
            &image,
            writes[root_write_of_the_landed_version]
                .bytes()
                .expect("根槽写带着字节"),
        )?;
        let (abandoned_publish_with_a_unit_overwritten, _) =
            publishes
                .iter()
                .find(|((instance, checkpoint_txg), units)| {
                    let mut copies_by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
                    for unit in units {
                        copies_by_offset
                            .entry(writes[*unit].offset.0)
                            .or_default()
                            .push(*unit);
                    }
                    *checkpoint_txg <= landed_version.1 .0
                        && table.abandons(*instance, *checkpoint_txg)
                        && units.iter().all(|unit| *unit < first_write_of_the_segment)
                        && copies_by_offset.values().any(|copies| {
                            copies
                                .iter()
                                .all(|copy| !unit_write_is_still_on_disk(&image, &writes[*copy]))
                        })
                })?;
        Some((landed_version, *abandoned_publish_with_a_unit_overwritten))
    })
}

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条（装置错）那一形：种子基 + 2 那段历史上摆出来的一个崩溃状态，
/// 扣下一份 journal 记录、留下另一份，恢复施加记录落到由记录重建的那一版，那一版的根槽写在更晚的段里（D23（journal 的角色与格式） 已定项 15）；
/// 那一版的实例表抛弃了更早的一次发布，那次发布的单元已被后来的发布复用盖掉。那时记录核对器按落到的那一版的实例表豁免它（实八），
/// 前提是交给它的写表里找得到那一版的根槽写（崩溃注入交整条流）。
///
/// **C554 乙之后这一形不可达**（合入后验证一，主 agent 2026-09-27 定改钉「这一形不可达」）：实例表抛弃一次发布要崩溃恢复抛弃根
/// （`CrashRecoveryAbandoningTheNewestRoot`），而那条根是这个会话发的、系统配置见证过它，下一次可写挂载读不出它时按 C554 乙拒
/// （`MountError::NewerStateStillUnreadableAfterOneReread`），没有新实例、没有抛弃行。第一轮快照（C554 乙合入之前）上这一条绿；
/// 今天的底座上把 C577 那道屏障去掉照样摆不出来，不可达是 C554 乙定的。
/// 这一条保留原来的构造（同一段历史、同一批崩溃状态），改钉：这批崩溃状态里一个这样的都没有，记录核对器一条都不判。
/// 把 C554 乙那一判关掉，这一形回来，第一条断言红（`crates/mutations.tsv`）。
#[test]
fn crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons(
) {
    let history = history_with_a_crash_state_landing_on_a_version_rebuilt_from_records();
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        FAST_TIER_DRAW,
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：种子基 + 2 那段历史，C554 乙之后找「恢复落到由记录重建的一版、它的实例表抛弃了单元被复用的发布」那一形 ──\n{}",
        injection.tally.render()
    ));
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert!(
        !crash_recovery_outcomes.is_empty()
            && crash_recovery_outcomes.iter().all(|outcome| *outcome
                == StepOutcome::Refused {
                    member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
                }),
        "这段历史里的崩溃恢复抛弃根每一步都被 C554 乙拒、一个被抛弃的根都没造出来：{crash_recovery_outcomes:?}"
    );
    assert_eq!(
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points),
        None,
        "C554 乙之后崩溃恢复抛弃根那一步被拒、造不出被抛弃的根：这批崩溃状态里不该有落到由记录重建、实例表抛弃了单元被复用的发布的那一版"
    );
    assert_eq!(
        injection.tally.record_checks,
        u64::try_from(injection.crash_points.len()).expect("崩溃状态数装得进 u64"),
        "每个崩溃状态都跑过记录核对器"
    );
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 0,
        "这段历史上的崩溃状态一条「恢复自称新态而单元缺席」都不判"
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "这段历史上没有「根在而它那次发布的记录一条都不在」的崩溃状态"
    );
}

/// 判别力那一半（调查报告第 1 条「推翻条件与造它的那一次」第 2 次）原来在上一条那一形上做：落到的那一版在它实例里的前一版（没被抛弃）
/// 写在某个偏移上的单元两份都扣下，照判红。**C554 乙之后那一形不可达**（见上一条），这一条改钉：用快档的崩溃状态与同一段历史，
/// 一个那样的崩溃状态都摆不出来。「没被抛弃的发布缺单元照样判红」那一半由
/// [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 的扣单元那两次守着。
/// 把 C554 乙那一判关掉，这一形回来，这一条红（`crates/mutations.tsv`）。
#[test]
fn on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red(
) {
    let history = history_with_a_crash_state_landing_on_a_version_rebuilt_from_records();
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        FAST_TIER_DRAW,
        None,
    );
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert!(
        !crash_recovery_outcomes.is_empty()
            && crash_recovery_outcomes.iter().all(|outcome| *outcome
                == StepOutcome::Refused {
                    member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
                }),
        "这段历史里的崩溃恢复抛弃根每一步都被 C554 乙拒、一个被抛弃的根都没造出来：{crash_recovery_outcomes:?}"
    );
    assert_eq!(
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points),
        None,
        "C554 乙之后这一形不可达：判别力那一半没有可做的崩溃状态"
    );
}

/// 偏向抬 F 之后回退那一组比重的取样点：这个测试周期的种子基起 12 段、每段 20 步、每段摆 6 个崩溃状态，一条清单外的失败都不许有。
///
/// 这一段**不**断言「某一类操作里至少摆到一个崩溃状态」：那是抽样摆出来的形态，下一个测试周期重抽种子基就可能一次都没有
/// （前提不成立就不调入口；2026-09-20 在随机种子基上实测撞到过一次：49 个崩溃状态里 `RaiseRollbackFloor` 是 0），
/// 那样的断言判的是抽样运气，不是代码。回退与抬 F 罩得到这一条由上面那条写死历史的用例钉着，那里是全枚举。
/// 各类操作各摆到几个照打进报告，给人看。
#[test]
fn crash_states_land_inside_rollbacks_and_floor_raises() {
    let report = run_crash_injection_campaign(&rollback_sampling_campaign(
        CrashInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入：偏向抬 F 之后回退的取样点 ──\n{rendered}"
    ));
    judge_crash_injection_report(&report);
}

/// 回退取样那一组：摆出过崩溃状态、恢复没失败过（不钉抽样摆出来的形态，见用到它的那条用例的文档注释）。
fn assert_the_rollback_sampling_placed_crash_states_and_recovered_from_each(
    report: &CrashInjectionReport,
) {
    assert!(
        report.tally.crash_points >= 1,
        "一个崩溃状态都没摆出来；{}",
        how_to_replay(report)
    );
    assert_eq!(
        report.tally.recoveries_failed,
        0,
        "崩溃之后恢复失败过；{}",
        how_to_replay(report)
    );
}

/// 偏向抬 F 之后回退的那一批：种子基起 12 段、每段 20 步、每段 6 个崩溃状态。
fn rollback_sampling_campaign(
    worker_threads: CrashInjectionWorkerThreads,
) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 12,
        operations_per_history: 20,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 6,
        },
        weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::RollbackSampling,
    }
}

/// 大档那条入口读环境：变量与判红时打的重放行同一张表（`crash_injection_campaign_from_environment`），没设的取大档的默认
/// （种子基起 500 段、每段 40 步、每段 8 个崩溃状态、各类操作都抽、不跑每一步的 checker、两块 4 GiB 的盘、判空间准入、只判清单外的失败），
/// 线程数从 `SINGLEFS_CRASH_INJECTION_THREADS` 取，判红的崩溃镜像留着。入口与「重放行喂回入口」那两条自证都经这一个函数。
fn large_tier_campaign_from(lookup: &dyn Fn(&str) -> Option<String>) -> CrashInjectionCampaign {
    crash_injection_campaign_from_environment(
        lookup,
        &CrashInjectionCampaign {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 500,
            operations_per_history: 40,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 8,
            },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::from_the_environment(),
            image_retention: FailureImageRetention::KeepTheImageFiles,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        },
    )
}

/// 大档／探索档：种子数、每段步数、每段几个崩溃状态、比重与盘宽从环境变量取；种子基没给就用这个测试周期写死的那一个。
/// 探索档要跑别的一批历史时，把种子基显式给出来（给了什么就跑什么，报告第一行照打）。
/// 判红的崩溃镜像留在临时目录里、路径打进报告（探索档要能回头看现场）。release 下后台跑。
// crash-case-check:not-a-crash-case 照重放行复现一批崩溃注入的入口，规模全由环境变量给、没有固定的全量域；它调的两个辅助函数只对计数断言、不造崩溃状态
#[test]
#[ignore = "大档：照判红时打的重放行设环境变量；SINGLEFS_CRASH_INJECTION_FIRST_SEED 起 SINGLEFS_CRASH_INJECTION_SEEDS 段、每段 SINGLEFS_CRASH_INJECTION_OPERATIONS 步、每段 SINGLEFS_CRASH_INJECTION_POINTS 个崩溃状态（SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES 给数时是写数不超过它的段整段枚举、长段各抽这么多个，none 是只抽样），比重 SINGLEFS_CRASH_INJECTION_WEIGHTS，checker SINGLEFS_CRASH_INJECTION_CHECKER，盘宽 SINGLEFS_CRASH_INJECTION_DEVICES，空间准入 SINGLEFS_CRASH_INJECTION_SPACE_ADMISSION，照哪一组断言判 SINGLEFS_CRASH_INJECTION_ASSERTIONS（new-findings、small-fast-tier、fast-tier、rollback-sampling），线程数 SINGLEFS_CRASH_INJECTION_THREADS；没设的取大档默认（种子基、500 段、40 步、8 个、只判清单外的失败）"]
fn crash_injection_large_tier_from_the_environment() {
    let started = std::time::Instant::now();
    let report =
        run_crash_injection_campaign(&large_tier_campaign_from(&process_environment_variable));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入大档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    judge_crash_injection_report(&report);
}

/// 生成器那一路没被这一件碰过：同一个种子生成的历史与随机历史那一段逐项相同（摆崩溃状态的随机源与它岔开）。
#[test]
fn drawing_crash_points_does_not_change_the_generated_history() {
    let history = generate_history(HistorySeed(5), 18);
    let without_any = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 0,
        },
        None,
    );
    let with_five = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 5,
        },
        None,
    );
    assert_eq!(with_five.seed, HistorySeed(5));
    assert_eq!(
        history,
        generate_history(HistorySeed(5), 18),
        "生成的历史逐项相同"
    );
    // 名副其实的那一半：摆崩溃状态不改变这段历史自己怎么跑。
    assert_eq!(
        without_any.tally.histories_completed, with_five.tally.histories_completed,
        "摆了崩溃状态之后这段历史的收尾变了"
    );
    assert_eq!(
        without_any.tally.histories_ended_known_red, with_five.tally.histories_ended_known_red,
        "摆了崩溃状态之后这段历史落到的已知红变了"
    );
    assert_eq!(
        without_any.tally.most_committed_versions_in_one_history,
        with_five.tally.most_committed_versions_in_one_history,
        "摆了崩溃状态之后模型提交过的版本数变了"
    );
    // 判别力：不摆就一个崩溃状态都没有，摆了就真摆出来了；两句都不成立时上面那三条恒真。
    assert_eq!(
        without_any.tally.crash_points, 0,
        "要 0 个崩溃状态却摆出了 {}",
        without_any.tally.crash_points
    );
    assert!(
        with_five.crash_points.len() <= 5 && !with_five.crash_points.is_empty(),
        "摆出来的崩溃状态要落在 1..=5：{}",
        with_five.crash_points.len()
    );
}

/// 判红时对一批崩溃注入会打的两行重放命令（`CrashInjectionReport::render` 里新发现之后的「复现这一段：」与末尾的「重放这一整批：」；
/// 造一个假发现，不必真红），原样喂回大档那条入口（`large_tier_campaign_from`，入口用的同一个函数）：
/// 种子区间是那一段（种子数 1）或那一整批，崩溃状态怎么摆、每段步数、比重、跑法与原来那一批逐项相同，那一段生成的历史逐步相同；
/// 「重放这一整批」照原来那一批的那一组断言判，「复现这一段」只判清单外的失败（一段的新发现与整批的结构计数无关）。
/// 不跑崩溃状态：同一组参数下摆出来的崩溃状态由种子定（`crash_points_are_reproducible`），这里只核参数一项不漏地回来了。
fn the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_large_tier_entry(
    campaign: &CrashInjectionCampaign,
) {
    let finding_seed = HistorySeed(campaign.first_seed + 5);
    let report = CrashInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        draw: campaign.draw,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: CrashInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: vec![CrashPointFinding {
            signature: FailureSignature::HarnessJudgement {
                judgement: "自证用的假发现",
            },
            seed: finding_seed,
            crash_point: CrashPoint {
                segment_index: 3,
                persisted_within_the_segment: vec![true, false],
                step: StepPosition::Operation(2),
                operation_kind: None,
            },
            stage: CrashStateStage::CrashImage,
            observation: FailureObservation {
                position: StepPosition::Operation(2),
                operation_kind: None,
                violations: Vec::new(),
                panic: None,
                newest_ring_root_txg: None,
                root_ring_slot_count: None,
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            },
            read_back: "自证不读回".to_string(),
            image_files: None,
        }],
        image_directory: None,
        image_retention: campaign.image_retention,
        assertions: campaign.assertions,
    };
    let rendered = report.render();
    let replay_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains("SINGLEFS_CRASH_INJECTION_FIRST_SEED="))
        .collect();
    assert_eq!(
        replay_lines.len(),
        2,
        "判红的报告里有两行重放命令（那一段、那一整批）：\n{rendered}"
    );
    let replayed_parameters = |line: &str| {
        let assignments = environment_assignments_in_line(line);
        let replayed =
            large_tier_campaign_from(&|variable: &str| assignments.get(variable).cloned());
        (
            replayed.first_seed,
            replayed.seed_count,
            replayed.operations_per_history,
            replayed.draw,
            replayed.weights,
            replayed.execution,
            replayed.assertions,
        )
    };
    let original_batch = (
        campaign.first_seed,
        campaign.seed_count,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution,
        campaign.assertions,
    );
    let original_finding = (
        finding_seed.0,
        1,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution,
        CrashInjectionAssertions::NewFindingsOnly,
    );
    let replayed_finding = replayed_parameters(replay_lines[0]);
    assert_eq!(
        replayed_finding, original_finding,
        "照「复现这一段」那一行跑大档，跑的不是原来那一段（种子、种子数、步数、摆法、比重、跑法、照哪一组断言判）：{}",
        replay_lines[0]
    );
    assert_eq!(
        replayed_parameters(replay_lines[1]),
        original_batch,
        "照「重放这一整批」那一行跑大档，跑的不是原来那一批、或不照原来那一组断言判：{}",
        replay_lines[1]
    );
    assert_eq!(
        generate_history_with_weights(
            HistorySeed(replayed_finding.0),
            replayed_finding.2,
            &replayed_finding.4
        ),
        generate_history_with_weights(
            finding_seed,
            campaign.operations_per_history,
            &campaign.weights
        ),
        "那一段生成的历史逐步相同"
    );
}

#[test]
fn replay_lines_of_the_small_fast_tier_run_the_same_campaign_on_the_large_tier_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_large_tier_entry(
        &small_fast_tier_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
}

/// 照小快档的重放行跑的大档入口按小快档那一组断言判：一批二次崩溃一个都没摆出来的报告（造的计数，不跑崩溃状态；
/// 攻方的改坏就是这一形：`draw_second_crash_points` 一个都不摆），判红在小快档那一条「挂载途中一个二次崩溃都没摆出来」上，
/// 不是只判新发现而判绿（代码三方 test-script-align-code-r1 判决 S2）。
#[test]
#[should_panic(expected = "挂载途中一个二次崩溃都没摆出来")]
fn a_small_fast_tier_replay_without_second_crashes_is_judged_red_by_the_small_fast_tier_assertions()
{
    let campaign = small_fast_tier_campaign(CrashInjectionWorkerThreads::GivenByCaller(1));
    let every_crash_state = SMALL_FAST_TIER_SEEDS;
    let tally = CrashInjectionTally {
        crash_points: every_crash_state,
        model_judgements: every_crash_state,
        checker_runs: every_crash_state,
        record_checks: every_crash_state,
        writable_mounts_after_the_crash: every_crash_state,
        checker_runs_after_the_writable_mount: every_crash_state,
        record_checks_after_the_writable_mount: every_crash_state,
        second_crash_points: 0,
        // 计数袋没有不变量，其余计数从零起（与 `crash_injection.rs` 里汇总一段历史的计数同一种写法）。
        ..CrashInjectionTally::default()
    };
    let replay_line = CrashInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        draw: campaign.draw,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: CrashInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
        image_directory: None,
        image_retention: campaign.image_retention,
        assertions: campaign.assertions,
    }
    .replay_line();
    let assignments = environment_assignments_in_line(&replay_line);
    let replayed = large_tier_campaign_from(&|variable: &str| assignments.get(variable).cloned());
    judge_crash_injection_report(&CrashInjectionReport {
        first_seed: replayed.first_seed,
        seed_count: replayed.seed_count,
        operations_per_history: replayed.operations_per_history,
        draw: replayed.draw,
        weights: replayed.weights,
        execution: replayed.execution,
        tally,
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
        image_directory: None,
        image_retention: replayed.image_retention,
        assertions: replayed.assertions,
    });
}

#[test]
fn replay_lines_of_the_fast_tier_run_the_same_campaign_on_the_large_tier_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_large_tier_entry(
        &fast_tier_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
}

#[test]
fn replay_lines_of_the_rollback_sampling_run_the_same_campaign_on_the_large_tier_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_large_tier_entry(
        &rollback_sampling_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
}

/// 记录核对器两条判据的靶向用例（代码三方 m2-supp3-item3-code-r2 判决第四节第 1 题，用户 2026-09-28 选「靶向用例 + 变异行」；
/// 攻方报告第四节改法甲、丙）用的那段历史：起点发完第一个文件 (1, 3)，再覆盖写两次 (1, 4)、(1, 5)。
/// 两块 4 GiB 的盘上这段历史的录制流切成 24 段（2026-10-03 在 harness 侧现量的）：第 12 段起是 (1, 3) 的单元写
/// （段内第 0、1 个是它的数据单元在两块盘上的两份），第 21 段是 (1, 5) 的两份 journal 记录，第 22 段是 (1, 5) 的根槽写；
/// 从第 12 段的第一个写数到 (1, 4) 的根槽写共 56 个写。两条用例都先核这几处，段形变了先在那里红，不悄悄判别的状态。
fn history_of_a_first_file_and_two_overwrites() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![overwrite_choosing(5), overwrite_choosing(7)],
    }
}

/// 一次覆盖写：内容落在一个数据单元里，长度与字节都随 `selector` 变（每一次都换下上一版的数据单元）。
fn overwrite_choosing(selector: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed: selector,
    })
}

/// 这段历史在崩溃注入那一种执行下的写表与切段（与 `inject_crashes_into_history` 自己切的同一份：同一条执行、同一份几何）。
fn writes_and_segments_of(history: &GeneratedHistory) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    )
}

/// 崩溃镜像那一截（只读恢复、模型、池级 checker、记录核对器）判红记下的那一条新发现；没有时 panic，带上全部新发现。
fn finding_on_the_crash_image(injection: &HistoryCrashInjection) -> &CrashPointFinding {
    injection
        .new_findings
        .iter()
        .find(|finding| finding.stage == CrashStateStage::CrashImage)
        .unwrap_or_else(|| panic!("崩溃镜像那一截没有判红：{:#?}", injection.new_findings))
}

/// 记录核对器第一条判据（根在而那次发布的记录一条都不在）接进崩溃注入之后真的判得出东西（K3-④）：
/// 第 21 段起数 3 个写（(1, 5) 的两份 journal 记录与它的根槽写），两份记录扣下、根槽写持久——越过了记录与根之间那道屏障，
/// 屏障守住时摆不出来。恢复落到 (1, 5)、读回它的内容，模型认、池级 checker 不红（2026-10-03 在 harness 侧照同一个持久集合现量的），
/// 这个崩溃状态只有记录核对器判得出。把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉、
/// 计数照记（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
#[test]
fn a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone()
{
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[21][0];
    assert_eq!(
        writes[first_write..first_write + 3]
            .iter()
            .map(|write| write.kind)
            .collect::<Vec<_>>(),
        vec![
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::RootRecordFua
        ],
        "第 21 段起的 3 个写是一次发布的两份 journal 记录与它的根槽写"
    );
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[first_write + 2]),
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "那个根槽写是第二次覆盖写 (1, 5) 的"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 21,
            writes_from_the_segment_on: 3,
            withheld_among_them: &[0, 1],
        },
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：记录核对器第一条判据的靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_root_without_record, 1,
        "记录核对器在这个崩溃状态上判出「根在而它那次发布的记录一条都不在」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["root_without_record"]
        },
        "崩溃镜像那一截的新发现记在记录核对器头上：{finding:#?}"
    );
    assert!(
        finding.observation.violations.is_empty()
            && finding.observation.model_disagreement.is_none(),
        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
    );
}

/// 记录核对器第二条判据（恢复自称的那一版不早于某次发布、而那次发布写出的某个单元两份都不在）接进崩溃注入之后真的判得出东西（K3-⑨）：
/// 第 12 段起数 56 个写（(1, 3) 的单元写起、到 (1, 4) 的根槽写为止），扣下 (1, 3) 的数据单元两份、别的都持久——
/// 越过了 (1, 3) 自己单元与记录之间、记录与根之间两道屏障。恢复落到 (1, 4)（它的根槽写持久了），读回它的内容，模型认。
/// (1, 3) 的数据单元已被 (1, 4) 换下、还没被复用，盘上那两处一直是空的，记录核对器判「恢复自称新态而单元缺席」。
/// 照常点名验证的恢复下，只扣单元不越过根的状态落回上一版，这一条判据判不到（K3-⑨ 调查），所以这里让根也持久。
/// 池级 checker 也红（根环里还留着 (1, 3) 的根，从它出发的遍历读不到那个数据单元），签名照「记录核对器先于 checker」取记录核对器；
/// 只有记录核对器红的那一个状态是下一条用例。
/// 把崩溃镜像那一截交给记录核对器的「恢复落到的那一版」换成 None（`crates/mutations.tsv`），第二条判据的前置不成立，这一条红。
#[test]
fn a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker(
) {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[12][0];
    let withheld_copies = &writes[first_write..first_write + 2];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == DATA_UNIT_BYTES
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 12 段的头两个写是同一个数据单元在两块盘上的两份：{withheld_copies:?}"
    );
    assert_eq!(
        writes[first_write + 2..first_write + 56]
            .iter()
            .filter_map(root_identity_of_a_root_slot_write)
            .collect::<Vec<_>>(),
        vec![
            (InstanceGeneration(1), CheckpointTxg(3)),
            (InstanceGeneration(1), CheckpointTxg(4))
        ],
        "从第 12 段起数的 56 个写里有 (1, 3) 与 (1, 4) 的根槽写，最后一个是 (1, 4) 的"
    );
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[first_write + 55]),
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "从第 12 段起数的第 56 个写是 (1, 4) 的根槽写"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 12,
            writes_from_the_segment_on: 56,
            withheld_among_them: &[0, 1],
        },
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：记录核对器第二条判据的靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 1,
        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["claimed_state_missing_unit"]
        },
        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
    );
    assert!(
        finding.observation.model_disagreement.is_none(),
        "恢复落到 (1, 4) 并读回它的内容，模型认：{finding:#?}"
    );
}

/// 崩溃注入第一截交给记录核对器的写表取整条历史录制流（`crash_injection.rs` 第一截那一次 `check_records_against`），
/// 与只取到崩溃点所在段为止的前缀，判出来不同只在两种崩溃状态上（推的：前缀之后的写一个都没持久，缺席判定与回收谓词只用得到
/// 持久的后写与更早的写；整条流多出来的只有根槽写在前缀之后的那几次发布，与恢复落到的那一版的根槽写）：
/// - 恢复落到由记录重建的一版、它的实例表抛弃了某次单元已被复用的发布：前缀里没有那一版的根槽写，实例表读不出，豁免落空。
///   C554 乙之后这一形摆不出来（[`crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons`]、
///   [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 钉着）。
/// - 恢复落到由记录重建的一版、那一版自己的某个单元两份都不在：前缀里没有它的根槽写，那次发布归不出来，第二条判据罩不到它。
///   这一条造的就是它：第二次覆盖写 (1, 5) 的第一个数据单元两份（第 20 段头两个写）扣下，第 20 段别的单元写与第 21 段两份 journal 记录都持久，
///   第 22 段 (1, 5) 的根槽写在窗口之后、没落。验点名单元的恢复（`JournalPolicy::Consult`，崩溃注入用的就是它）不施加那条记录、落回 (1, 4)，
///   两种交法同判；只有不验点名单元的恢复（只供测试强制进入的 `JournalPolicy::ConsultWithoutNamedVerification`）落到 (1, 5)，整条流判红、前缀不判。
///
/// 所以把第一截的写表退回前缀那一条变异不是等价变异：这一条自己就造出了两种交法不同值的输入（不验点名单元的恢复落到 (1, 5)，整条流判红、前缀不判）。
/// 它是第三类「取样点不敏感」（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」）：崩溃注入的取值域里摆不出能分辨两种交法的盘面——
/// 崩溃注入恢复只用验点名单元的那一种（`JournalPolicy::Consult`），C554 乙那一判又让另一形摆不出来；补不上敏感的取样点，是因为要给崩溃注入
/// 加换恢复策略的开关（改被测代码，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第五节「崩溃注入换恢复策略的开关」那一行），
/// 不是因为两种交法在所有输入上同值。这一条钉住取值域外那一支的两种交法确实分开、取值域里那一形两种交法同判；C554 乙那一判或点名验证哪天不在了，
/// 崩溃注入里两种交法也会分开。
/// 第二条判据不罩恰好是恢复落到的那一版的那次发布（`crates/mutations.tsv`），整条流那一判不红，这一条红。
#[test]
fn a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it(
) {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let root_of_the_second_overwrite = segments[22][0];
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[root_of_the_second_overwrite]),
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "第 22 段的头一个写是第二次覆盖写 (1, 5) 的根槽写"
    );
    assert_eq!(
        segments[21]
            .iter()
            .map(|index| writes[*index].kind)
            .collect::<Vec<_>>(),
        vec![StepKind::JournalRecord, StepKind::JournalRecord],
        "第 21 段是 (1, 5) 的两份 journal 记录"
    );
    let first_write = segments[20][0];
    let withheld_copies = &writes[first_write..first_write + 2];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == DATA_UNIT_BYTES
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 20 段的头两个写是 (1, 5) 一个数据单元在两块盘上的两份：{withheld_copies:?}"
    );
    let end_of_the_window = segments[21][0] + segments[21].len();
    assert_eq!(
        end_of_the_window, root_of_the_second_overwrite,
        "窗口（第 20、21 段）截在 (1, 5) 的根槽写之前"
    );

    let persisted_within_the_window: Vec<bool> = (first_write..end_of_the_window)
        .map(|index| index >= first_write + 2)
        .collect();
    let persisted =
        persisted_over_the_whole_stream(writes.len(), first_write, &persisted_within_the_window);
    let base = pool_of_the_persisted_writes_before_the_segment(
        &writes[..first_write],
        &persisted,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .device_bytes(),
    );
    let image = CrashImage {
        base: &base,
        writes: &writes[first_write..end_of_the_window],
        persisted: persisted_within_the_window,
    };
    // 第一截的交法：两份镜像同一份、一条录制流（这段历史没掉过电），写表与持久集合取到 `end_of_table` 为止。
    let record_check_on_the_table_up_to =
        |end_of_table: usize, effective_root: Option<(InstanceGeneration, CheckpointTxg)>| {
            check_records_against(
                &image,
                &image,
                &writes[..end_of_table],
                &persisted[..end_of_table],
                RecordStreamContinuity::OneRecording,
                effective_root,
            )
        };

    let verifying = recover(&image, JournalPolicy::Consult).effective_root;
    assert_eq!(
        verifying,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "验点名单元的恢复不施加 (1, 5) 的记录（它点名的数据单元两份都不在），落回 (1, 4)"
    );
    assert_eq!(
        record_check_on_the_table_up_to(writes.len(), verifying),
        RecordCheck::default(),
        "落回 (1, 4)：整条流上记录核对器一条都不判"
    );
    assert_eq!(
        record_check_on_the_table_up_to(end_of_the_window, verifying),
        RecordCheck::default(),
        "落回 (1, 4)：前缀上也一条都不判，两种交法同值"
    );

    let unverifying =
        recover(&image, JournalPolicy::ConsultWithoutNamedVerification).effective_root;
    assert_eq!(
        unverifying,
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "不验点名单元的恢复施加 (1, 5) 的记录，落到由记录重建的 (1, 5)"
    );
    assert!(
        record_check_on_the_table_up_to(writes.len(), unverifying).claimed_state_missing_unit,
        "整条流里有 (1, 5) 的根槽写，那次发布归得出来，它的数据单元两份都不在：判「恢复自称新态而单元缺席」"
    );
    assert_eq!(
        record_check_on_the_table_up_to(end_of_the_window, unverifying),
        RecordCheck::default(),
        "前缀里没有 (1, 5) 的根槽写，那次发布归不出来，第二条判据罩不到它"
    );
}

/// 第一个文件之后接着覆盖写的次数：根环三个区域各 8 槽，第 24 次覆盖写 (1, 27) 的根槽写盖掉 (1, 3) 的那一槽。
const OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE: u64 = 24;

/// 第一个文件 (1, 3) 之后覆盖写 24 次：根环转过一圈，(1, 3) 的根槽已被 (1, 27) 盖掉。
fn history_of_a_first_file_and_overwrites_that_turn_the_root_ring() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (0..OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE)
            .map(|overwrite_index| overwrite_choosing(5 + 2 * overwrite_index))
            .collect(),
    }
}

/// 记录核对器第二条判据只有它自己判得出的那一个状态（第二批报告 `research/prompts/m2-rest-batch2/report.md`「停下交主 agent 的设计问题」
/// 第 2 条说找不到：缺单元的发布只要它的根还在环里，从那条根出发的 checker 遍历就会红）。根环转过一圈之后 (1, 3) 的根不在环里了：
/// 第 12 段起到流尾的每一个写都持久，只扣下第 12 段段内第 4、5 个写——(1, 3) 那一片 inode 树叶容器在两块盘上的两份（它已被 (1, 4)
/// 换下，之后没有一个写落在那两处）。恢复落到 (1, 27)、读回它的内容，模型认，池级 checker 不红（2026-10-03 在 harness 侧照同一个持久集合现量：
/// 覆盖写 16 次时 (1, 3) 的根还在环里，checker 红 I-2.1、I-4.8、I-7.4；24 次时不红）；记录核对器判 (1, 3) 那个单元两份都缺席。
/// RefFS 的崩溃时刻取持久窗口最后一个写所在那一段（流尾那一次覆盖写），读回最后一版的内容合它。
/// 把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
#[test]
fn a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone(
) {
    let history = history_of_a_first_file_and_overwrites_that_turn_the_root_ring();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[12][0];
    let withheld_copies = [&writes[first_write + 4], &writes[first_write + 5]];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 12 段段内第 4、5 个写是同一个单元在两块盘上的两份：{withheld_copies:?}"
    );
    let first_file_root = writes
        .iter()
        .position(|write| {
            root_identity_of_a_root_slot_write(write)
                == Some((InstanceGeneration(1), CheckpointTxg(3)))
        })
        .expect("流里有第一个文件 (1, 3) 的根槽写");
    assert!(
        first_write + 5 < first_file_root,
        "扣下的两份是 (1, 3) 那次发布的单元写（在它的根槽写之前）"
    );
    assert!(
        writes[first_file_root + 1..].iter().any(|write| {
            write.kind == StepKind::RootRecordFua
                && write.device == writes[first_file_root].device
                && write.offset == writes[first_file_root].offset
        }),
        "(1, 3) 的根槽之后被别的根盖掉了：根环转过一圈"
    );
    assert!(
        writes[first_write + 6..].iter().all(|later| withheld_copies
            .iter()
            .all(|withheld| later.device != withheld.device || later.offset != withheld.offset)),
        "扣下的那两处之后没有一个写落上去：缺席没有更晚的写来解释"
    );
    let writes_from_the_segment_to_the_end = writes.len() - first_write;
    assert_eq!(
        writes_from_the_segment_to_the_end, 793,
        "从第 12 段的第一个写数到流尾共 793 个写"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 12,
            writes_from_the_segment_on: 793,
            withheld_among_them: &[4, 5],
        },
        None,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：只有记录核对器判得出的第二条判据靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 1,
        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["claimed_state_missing_unit"]
        },
        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
    );
    assert!(
        finding.observation.violations.is_empty()
            && finding.observation.model_disagreement.is_none(),
        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
    );
}
