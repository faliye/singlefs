//! 里程碑「第二个事务」增补 3 第 3 件：崩溃注入。生成器、执行器与理想模型在第 1、2 件（`singlefs_harness::history`、
//! `singlefs_harness::model`），切段、摆崩溃状态、重建与判定在 `singlefs_harness::crash_injection`；
//! 这里是快档（普通 `cargo test`）、多线程对拍、写死的那几段历史（段内真子集全枚举）与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 枚举域与层 0 同一个（屏障与 FUA 写切段、段内任意真子集）；差别在怎么取：层 0 在两条写死的流上全枚举（门禁 54 号），
//! 这里在随机历史上抽样。种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史那个二进制用的是同一个），
//! 打在报告第一行与失败信息里（用户 2026-09-20 定案第 7 条，同日定的范围：随机说的是一个测试周期与下一个之间，不是每次跑重抽）。

use std::collections::BTreeMap;
use std::io::Write as _;

use singlefs_checker::walk::InstanceTableOfRootRecord;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader};
use singlefs_format::DATA_UNIT_BYTES;
use singlefs_harness::crash::{
    check_records, check_records_against, root_identity_written_by, writes_and_segments,
    CrashImage, MemoryPool, RecordCheck, RetainedWrite,
};
use singlefs_harness::crash_injection::{
    inject_crashes_into_history, run_crash_injection_campaign, CrashInjectionCampaign,
    CrashInjectionReport, CrashInjectionWorkerThreads, CrashPoint, CrashPointDraw,
    FailureImageRetention, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::history::{
    execute_history_with, generate_history, generate_history_with_weights, ContentChoice,
    ContentLength, FloorTargetChoice, GeneratedHistory, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed,
    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 快档的规模：段数、每段步数与每段摆几个崩溃状态。规模与种子基都写死（种子基是这个测试周期的常量）。
/// 每个崩溃状态上起可写挂载、发一次布、摆三个二次崩溃之后（代码审阅第 2 条），这一档 debug 下单跑一百来秒，标了 ignore，
/// 由提交时的崩溃验证员经门禁 54 号（`crash-case:crash-injection-fast-tier`）在 release 下跑；普通 `cargo test` 由小快档守。
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

fn number_from_environment(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| {
        text.parse()
            .unwrap_or_else(|error| panic!("{name}={text} 不是一个非负整数：{error}"))
    })
}

/// 判红时先把种子基说清楚：同一个构建加同一个种子基重放得出来，报告里不带它就没法复现。
fn how_to_replay(report: &CrashInjectionReport) -> String {
    format!(
        "重放：SINGLEFS_CRASH_INJECTION_FIRST_SEED={} SINGLEFS_CRASH_INJECTION_SEEDS={} SINGLEFS_CRASH_INJECTION_OPERATIONS={} 跑大档那条 #[ignore] 用例",
        report.first_seed, report.seed_count, report.operations_per_history
    )
}

/// 种子基是一个测试周期的常量：开一个周期抽一次，抽完在这个周期之内写死。周期从「开一个新里程碑」与
/// 「用户显式说从零开始测试 / 重建测试」里先到的那一件开始（用户 2026-09-20 定案第 7 条与同日定的范围：
/// 「我说的随机是 多次实验的随机  比如 里程碑 1 和里程碑 2  而不是 每个里程碑里面都要随机」）。
///
/// 这一条钉两样：
/// 1. 常量就是当前这个周期（里程碑「第二个事务」）2026-09-20 抽出来的那个数。改成别的数 = 把这个周期的实验数据整批换掉，
///    而 `crates/mutations.tsv` 第 146–178 行那些「这条变异必须红」与里程碑那几条验收，量的都是这一批历史；
/// 2. 从它起跑出来的报告把它带了回来（报告第一行与重放那一行都带着它），判红了才重放得出来。
///
/// 随机历史那个二进制的五段用的是同一个常量——两边同基，那一半由它自己那条用例钉
/// （`second_transaction_supplement_three_random_history.rs` 的 `the_five_sampling_tiers_start_from_the_test_cycle_seed_base`）。
#[test]
fn the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle() {
    assert_eq!(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        7_463_871_032_432_355_113,
        "种子基不是这个测试周期 2026-09-20 抽的那个数：周期之内不许换，换了这个周期量过的判红全部作废；\
         下一个周期才重抽（周期怎么算、抽法、周期开头还要做什么，写在 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE 的文档注释里）"
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
#[ignore = "快档：每个崩溃状态上起可写挂载、发一次布、摆三个二次崩溃之后 debug 下单跑一百来秒；交提交时的崩溃验证员经门禁 54 号按 crash-case:crash-injection-fast-tier 在 release 下跑，普通 cargo test 由 crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed 守"]
fn crash_injection_fast_tier_recovers_only_into_versions_the_model_committed() {
    let started = std::time::Instant::now();
    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        draw: FAST_TIER_DRAW,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::from_the_environment(),
        // 门禁这一档跑完把镜像目录删掉：临时目录里不留东西（门禁 77 号判跑完机器干不干净）。
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
    });
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_every_crash_injection_path_was_exercised(&report);
}

/// 小快档：快档的头 [`SMALL_FAST_TIER_SEEDS`] 段（每段步数、每段几个崩溃状态、比重与快档相同），普通 `cargo test` 里守着崩溃注入这一路。
/// 清单外的失败一条都不许有；三截每一截在每个崩溃状态上都真的跑了（每个崩溃状态都问过模型、跑过池级 checker 与记录核对器，
/// 都起过可写挂载、之后的池都跑过 checker；二次崩溃摆出来过，每个都问过模型、跑过 checker 与记录核对器）。
/// 只钉这些「每个崩溃状态都做了什么」的结构计数，不钉抽样摆出来的形态（那是快档的事，头几段里未必每一形都有）。
#[test]
fn crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed() {
    let started = std::time::Instant::now();
    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: SMALL_FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        draw: FAST_TIER_DRAW,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::from_the_environment(),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
    });
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入小快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    let replay = how_to_replay(&report);
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{replay}\n{rendered}"
    );
    let tally = &report.tally;
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

fn assert_every_crash_injection_path_was_exercised(report: &CrashInjectionReport) {
    let tally = &report.tally;
    let replay = how_to_replay(report);
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
    let (landed_instance, landed_txg) = report.effective_root.expect("恢复落到了一版");
    assert_eq!(
        landed_instance,
        InstanceGeneration(2),
        "落到实例 2 的最新那一版"
    );
    assert!(
        landed_txg > CheckpointTxg(4),
        "落到的那一版比被抛弃的 (1, 4) 新"
    );
    let abandoned_units = units_of((1, 4));
    assert!(
        abandoned_units
            .iter()
            .any(|unit| !unit_write_is_still_on_disk(&every_write_landed, &writes[*unit])),
        "(1, 4) 写出的单元有被后来的发布复用盖掉的（这一条要罩的就是这一形）"
    );
    assert_eq!(
        check_records(&every_write_landed, report.effective_root),
        RecordCheck::default(),
        "被抛弃的 (1, 4) 的单元不算「该在」，记录核对器一条都不判红"
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

/// 一个恢复落到由 journal 记录重建的一版上的崩溃状态（调查报告第 1 条那一形），按崩溃注入重建镜像的那种摆法拆开：
/// 整条历史录制流的写表、崩溃点所在的段与段内持久了哪几个写、落到的那一版与它在整条流里的根槽写（在这一段之后）、
/// 被那一版的实例表抛弃而单元已被后来的发布复用盖掉的那次发布。
struct CrashStateLandingOnAVersionRebuiltFromRecords {
    writes: Vec<RetainedWrite>,
    device_bytes: u64,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: Vec<bool>,
    landed_version: (InstanceGeneration, CheckpointTxg),
    root_write_of_the_landed_version: usize,
    abandoned_publish_with_a_unit_overwritten: (u32, u64),
}

impl CrashStateLandingOnAVersionRebuiltFromRecords {
    fn end_of_the_segment(&self) -> usize {
        self.first_write_of_the_segment + self.persisted_within_the_segment.len()
    }

    /// 崩溃注入交给记录核对器的那一种持久集合，另扣下 `withheld_earlier_writes`（[`persisted_over_the_whole_stream`]）。
    fn persisted_over_the_whole_stream(&self, withheld_earlier_writes: &[usize]) -> Vec<bool> {
        persisted_over_the_whole_stream(
            self.writes.len(),
            self.first_write_of_the_segment,
            &self.persisted_within_the_segment,
            withheld_earlier_writes,
        )
    }

    fn base_pool(&self, persisted: &[bool]) -> MemoryPool {
        pool_of_the_persisted_writes_before_the_segment(
            &self.writes[..self.first_write_of_the_segment],
            persisted,
            self.device_bytes,
        )
    }

    /// 基线加上崩溃点所在段里持久了的写（崩溃注入重建镜像的同一种摆法）。
    fn crash_image<'base>(&'base self, base: &'base MemoryPool) -> CrashImage<'base> {
        CrashImage {
            base,
            writes: &self.writes[self.first_write_of_the_segment..self.end_of_the_segment()],
            persisted: self.persisted_within_the_segment.clone(),
        }
    }

    /// 落到的那一版的实例表，从 `image` 上按它在整条流里那次根槽写带的指针读。
    fn instance_table_of_the_landed_version(
        &self,
        image: &CrashImage<'_>,
    ) -> Option<InstanceTableOfRootRecord> {
        InstanceTableOfRootRecord::read(
            image,
            self.writes[self.root_write_of_the_landed_version]
                .bytes()
                .expect("根槽写带着字节"),
        )
    }
}

/// 与整条流（`write_count` 个写）逐条对应的持久集合：崩溃点所在段之前整段持久（`withheld_earlier_writes` 里的除外）、
/// 这一段按崩溃点的子集、之后的一个都没持久——崩溃注入交给记录核对器的那一种。
fn persisted_over_the_whole_stream(
    write_count: usize,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: &[bool],
    withheld_earlier_writes: &[usize],
) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    persisted[..first_write_of_the_segment].fill(true);
    for withheld in withheld_earlier_writes {
        assert!(
            *withheld < first_write_of_the_segment,
            "另扣下的只能是崩溃点所在段之前的写（写表下标 {withheld}）"
        );
        persisted[*withheld] = false;
    }
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
///
/// # Panics
/// 一个这样的崩溃状态都没有：种子基、切段或这段历史怎么跑变了，这一形不在这几个崩溃状态里，靠它的用例要换一段摆得出它的历史。
fn crash_state_landing_on_a_version_rebuilt_from_records(
    history: &GeneratedHistory,
    crash_points: &[CrashPoint],
) -> CrashStateLandingOnAVersionRebuiltFromRecords {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    execute_history_with(history, execution, &stream, &mut |_| {});
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let device_bytes = execution.device_width.device_bytes();
    let found = crash_points.iter().find_map(|crash_point| {
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
            &[],
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
        Some((
            first_write_of_the_segment,
            crash_point.persisted_within_the_segment.clone(),
            landed_version,
            root_write_of_the_landed_version,
            *abandoned_publish_with_a_unit_overwritten,
        ))
    });
    let (
        first_write_of_the_segment,
        persisted_within_the_segment,
        landed_version,
        root_write_of_the_landed_version,
        abandoned_publish_with_a_unit_overwritten,
    ) = found.unwrap_or_else(|| {
        panic!(
            "种子 {:?} 的崩溃状态 {crash_points:#?} 里没有「恢复落到由记录重建的一版、它的根槽写在更晚的段、它的实例表抛弃的发布有单元已被复用盖掉」那一形：\
             种子基、切段或这段历史怎么跑变了，要换一段摆得出这一形的历史",
            history.seed
        )
    });
    CrashStateLandingOnAVersionRebuiltFromRecords {
        writes,
        device_bytes,
        first_write_of_the_segment,
        persisted_within_the_segment,
        landed_version,
        root_write_of_the_landed_version,
        abandoned_publish_with_a_unit_overwritten,
    }
}

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条（装置错）：种子基 + 2 那段历史上摆出来的一个崩溃状态，
/// 扣下一份 journal 记录、留下另一份，恢复施加记录落到由记录重建的那一版，那一版的根槽写在更晚的段里（D23（journal 的角色与格式） 已定项 15）；
/// 那一版的实例表抛弃了更早的一次发布，那次发布的单元已被后来的发布复用盖掉。记录核对器按落到的那一版的实例表豁免它（实八），
/// 前提是交给它的写表里找得到那一版的根槽写：崩溃注入交整条流（与层 0 同一种交法），这段历史上的崩溃状态一条「恢复自称新态而单元缺席」都不判。
/// 只交到崩溃点所在段为止的前缀时，那一版的实例表读不出、豁免整条落空，这一条判红。
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
    let state =
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points);
    print_uncaptured(&format!(
        "── 崩溃注入：恢复落到由记录重建的 {:?}（根槽写在写表第 {} 项，崩溃点所在段 [{}, {})），它的实例表抛弃 {:?} ──\n{}",
        state.landed_version,
        state.root_write_of_the_landed_version,
        state.first_write_of_the_segment,
        state.end_of_the_segment(),
        state.abandoned_publish_with_a_unit_overwritten,
        injection.tally.render()
    ));
    assert_eq!(
        injection.tally.record_checks,
        u64::try_from(injection.crash_points.len()).expect("崩溃状态数装得进 u64"),
        "每个崩溃状态都跑过记录核对器"
    );
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 0,
        "被落到的那一版的实例表抛弃的 {:?} 的单元不算「该在」：记录核对器要在交给它的写表里找得到那一版的根槽写（写表第 {} 项）",
        state.abandoned_publish_with_a_unit_overwritten, state.root_write_of_the_landed_version
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "这段历史上没有「根在而它那次发布的记录一条都不在」的崩溃状态"
    );
}

/// 判别力（调查报告第 1 条「推翻条件与造它的那一次」第 2 次）：同一个崩溃状态、同样交整条流，先不另扣——一条不判（对照）；
/// 再把落到的那一版在它实例里的前一版（没被抛弃）写在某个偏移上的单元两份都扣下，照判红：整条流让那一版的实例表读得出、豁免生效之后，
/// 没被抛弃的发布缺单元照样判。只算干净的扣法——扣下之后恢复仍落到同一版、那一版的实例表仍读得出且仍抛弃单元被复用的那次发布
/// （扣下实例表那一片会把豁免又关掉，那样判红说明不了判别力）；干净的扣法至少一个，每个都判红。
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
    let state =
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points);
    let (landed_instance, landed_txg) = state.landed_version;
    let (abandoned_instance, abandoned_txg) = state.abandoned_publish_with_a_unit_overwritten;

    let nothing_else_withheld = state.persisted_over_the_whole_stream(&[]);
    let base = state.base_pool(&nothing_else_withheld);
    let image = state.crash_image(&base);
    assert_eq!(
        check_records_against(
            &image,
            &state.writes,
            &nothing_else_withheld,
            Some(state.landed_version)
        ),
        RecordCheck::default(),
        "对照：整条流、不另扣，被抛弃的 {:?} 豁免，一条不判",
        state.abandoned_publish_with_a_unit_overwritten
    );

    let publishes = unit_writes_of_each_publish(&state.writes);
    let (kept_publish, kept_units) = publishes
        .iter()
        .rev()
        .find(|((instance, checkpoint_txg), units)| {
            *instance == landed_instance.0
                && *checkpoint_txg < landed_txg.0
                && !units.is_empty()
                && units
                    .iter()
                    .all(|unit| *unit < state.first_write_of_the_segment)
        })
        .unwrap_or_else(|| {
            panic!(
                "落到的那一版 {:?} 在它实例里有前一版，写过单元",
                state.landed_version
            )
        });
    let mut copies_by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for unit in kept_units {
        copies_by_offset
            .entry(state.writes[*unit].offset.0)
            .or_default()
            .push(*unit);
    }
    let mut clean_withholdings = 0usize;
    for (offset, copies) in &copies_by_offset {
        let persisted = state.persisted_over_the_whole_stream(copies);
        let base_with_the_unit_withheld = state.base_pool(&persisted);
        let image_with_the_unit_withheld = state.crash_image(&base_with_the_unit_withheld);
        let recovery = recover(&image_with_the_unit_withheld, JournalPolicy::Consult);
        let still_abandons_the_overwritten_publish = state
            .instance_table_of_the_landed_version(&image_with_the_unit_withheld)
            .is_some_and(|table| table.abandons(abandoned_instance, abandoned_txg));
        if recovery.effective_root != Some(state.landed_version)
            || !still_abandons_the_overwritten_publish
        {
            continue;
        }
        clean_withholdings += 1;
        assert!(
            check_records_against(
                &image_with_the_unit_withheld,
                &state.writes,
                &persisted,
                recovery.effective_root
            )
            .claimed_state_missing_unit,
            "{kept_publish:?} 没被落到的那一版 {:?} 抛弃，它写在偏移 {offset} 上的单元两份（写表第 {copies:?} 项）都扣下，照判红",
            state.landed_version
        );
    }
    print_uncaptured(&format!(
        "── 崩溃注入：落到 {:?}，扣下 {kept_publish:?} 的单元：{} 个偏移里干净的扣法 {clean_withholdings} 个，都判红 ──\n",
        state.landed_version,
        copies_by_offset.len()
    ));
    assert!(
        clean_withholdings >= 1,
        "{kept_publish:?} 写出的 {} 个偏移里一个干净的扣法都没有（扣哪一个恢复都换了一版，或那一版的实例表读不出）：判别力这一半没验到",
        copies_by_offset.len()
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
    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 12,
        operations_per_history: 20,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 6,
        },
        weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::from_the_environment(),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
    });
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入：偏向抬 F 之后回退的取样点 ──\n{rendered}"
    ));
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.tally.crash_points >= 1,
        "一个崩溃状态都没摆出来；{}",
        how_to_replay(&report)
    );
    assert_eq!(
        report.tally.recoveries_failed,
        0,
        "崩溃之后恢复失败过；{}",
        how_to_replay(&report)
    );
}

/// 大档／探索档：种子数、每段步数、每段几个崩溃状态、比重与盘宽从环境变量取；种子基没给就用这个测试周期写死的那一个。
/// 探索档要跑别的一批历史时，把种子基显式给出来（给了什么就跑什么，报告第一行照打）。
/// 判红的崩溃镜像留在临时目录里、路径打进报告（探索档要能回头看现场）。release 下后台跑。
#[test]
#[ignore = "大档：SINGLEFS_CRASH_INJECTION_SEEDS 段、每段 SINGLEFS_CRASH_INJECTION_OPERATIONS 步、每段 SINGLEFS_CRASH_INJECTION_POINTS 个崩溃状态，种子基 SINGLEFS_CRASH_INJECTION_FIRST_SEED（没给就用这个测试周期写死的那一个），线程数 SINGLEFS_CRASH_INJECTION_THREADS，比重 SINGLEFS_CRASH_INJECTION_WEIGHTS，盘宽 SINGLEFS_CRASH_INJECTION_DEVICES"]
fn crash_injection_large_tier_from_the_environment() {
    let first_seed = std::env::var("SINGLEFS_CRASH_INJECTION_FIRST_SEED").map_or_else(
        |_not_given| SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        |text| {
            text.parse()
                .unwrap_or_else(|error| panic!("种子基 {text} 不是一个非负整数：{error}"))
        },
    );
    let seed_count = number_from_environment("SINGLEFS_CRASH_INJECTION_SEEDS", 500);
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_CRASH_INJECTION_OPERATIONS",
        40,
    ))
    .expect("步数装得进 usize");
    let crash_points_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_CRASH_INJECTION_POINTS",
        8,
    ))
    .expect("崩溃状态数装得进 usize");
    let weights = match std::env::var("SINGLEFS_CRASH_INJECTION_WEIGHTS").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("broad") => GenerationWeights::BROAD,
        Ok("reuse") => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        Ok("rollback") => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        Ok("wall") => GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        Ok("unit-area-wall") => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        Ok(other) => panic!(
            "SINGLEFS_CRASH_INJECTION_WEIGHTS={other}：只认 broad、reuse、rollback、wall 与 unit-area-wall"
        ),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_CRASH_INJECTION_WEIGHTS 不是 UTF-8：{raw:?}")
        }
    };
    let device_width = match std::env::var("SINGLEFS_CRASH_INJECTION_DEVICES").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("4gib") => HistoryDeviceWidth::FourGibibytes,
        Ok("small") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok(other) => panic!("SINGLEFS_CRASH_INJECTION_DEVICES={other}：只认 4gib 与 small"),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_CRASH_INJECTION_DEVICES 不是 UTF-8：{raw:?}")
        }
    };
    let started = std::time::Instant::now();
    let report = run_crash_injection_campaign(&CrashInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history,
        },
        weights,
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        worker_threads: CrashInjectionWorkerThreads::from_the_environment(),
        image_retention: FailureImageRetention::KeepTheImageFiles,
    });
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入大档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
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
