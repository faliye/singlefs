//! 里程碑「第二个事务」增补 3 第 4 件：故障注入。通用的设备包装、注入点怎么摆、注入之后怎么判在
//! `singlefs_harness::fault_injection`；这里是快档（普通 `cargo test`）、多线程对拍、写死的那几条用例与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 被测的两条性质：注入之后 `singlefs-core` **返回错误而不是 panic**；出错之后**重开恢复到模型允许的版本**。
//! 种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史与崩溃注入那两个二进制用的是同一个）。

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, resend_the_frozen_publish, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::fault_injection::{
    inject_faults_into_history, inject_one_fault_into_the_segment, run_fault_injection_campaign,
    AcquiredInstanceLeft, AcquiredInstanceLeftAfterAFailedMount, DrawnFault, FaultCounting,
    FaultDeviceSelector, FaultInjectingBlockDevice, FaultInjectionCampaign, FaultInjectionReport,
    FaultInjectionWorkerThreads, FaultOccurrence, FaultOutcome, FaultPlacement, FaultSchedule,
    FaultedSegment, FaultedSegmentKind, FlippedBit, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::{
    execute_history_with_faults, generate_history_with_weights, GeneratedHistory,
    GenerationWeights, HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StartingPointStep,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 写死的那几条用例里两块内存盘各多大（与各步用例、随机历史的 4 GiB 档相同）。
const IMAGE_BYTES: u64 = 4 << 30;

/// 快档的规模：段数、每段步数、每段注入几次。每次注入都要把这段历史整个重跑一遍，所以规模按「别显著变慢」定。
const FAST_TIER_SEEDS: u64 = 24;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 20;
const FAST_TIER_FAULTS_PER_HISTORY: usize = 4;

/// 注入之后这段历史怎么跑：每一步之后跑池级 checker、判红就停，两块 4 GiB 的盘。
/// 跑 checker 是这一件的一半——注入之后盘面坏没坏，只有它看得见。
const CHECKED_AFTER_EVERY_STEP: HistoryExecution =
    HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;

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

/// 判红时先把种子基与规模说清楚：同一个构建加同一个种子基重放得出来。
fn how_to_replay(report: &FaultInjectionReport) -> String {
    format!(
        "重放：SINGLEFS_FAULT_INJECTION_FIRST_SEED={} SINGLEFS_FAULT_INJECTION_SEEDS={} SINGLEFS_FAULT_INJECTION_OPERATIONS={} SINGLEFS_FAULT_INJECTION_FAULTS={} 跑大档那条 #[ignore] 用例",
        report.first_seed, report.seed_count, report.operations_per_history, report.faults_per_history
    )
}

fn fast_tier_campaign(worker_threads: FaultInjectionWorkerThreads) -> FaultInjectionCampaign {
    FaultInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        faults_per_history: FAST_TIER_FAULTS_PER_HISTORY,
        weights: GenerationWeights::BROAD,
        execution: CHECKED_AFTER_EVERY_STEP,
        worker_threads,
    }
}

/// 快档：这个测试周期的种子基起 24 段、每段 20 步、每段摆 4 个注入点。每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」；「已知红」清单里的形态照记不停，清单外的一条都不许有。
///
/// 读出坏字节（`read_returns_corrupted_bytes`）只注入在一次读上：释放之前读盘核那一读撞上它时重读一次就对得上，不隔离
/// （D19（块指针的结构与宽度预算） 已定项 5：核出对不上也先重读一次）。只重读的是读不出那一半时，这一格在种子 7463871032432355113
/// 第 7 步（覆盖写）把一对好槽隔离掉、池级 checker 判 I-3.11 红。**每一读都给坏字节**（坏读一直在、盘上的字节是好的）时两次都对不上，
/// 照规则隔离那一对好槽——那是「两次都对不上才隔离」认下的形态，不是新发现；随机注入不摆这一形（一次注入只坏一次调用），
/// 它的样子钉在 `second_transaction_supplement_two_release_checksum_quarantine.rs` 的用例 13。
#[test]
fn fault_injection_fast_tier_returns_errors_instead_of_panicking() {
    let started = std::time::Instant::now();
    let report = run_fault_injection_campaign(&fast_tier_campaign(
        FaultInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了：这是被测的性质（返回错误而不是 panic）；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_every_fault_injection_path_was_exercised(&report);
}

/// 各条路径真的跑到了：六种注入都摆过、起点段与每一类会写盘的操作上都注入过、重开都跑过、四类写落点都注入过。
fn assert_every_fault_injection_path_was_exercised(report: &FaultInjectionReport) {
    let tally = &report.tally;
    let rendered = report.render();
    assert!(
        tally.faults >= FAST_TIER_SEEDS,
        "摆出来的注入点太少（{}）：每段至少该摆出一个\n{rendered}",
        tally.faults
    );
    // 六种全抽：前四种是里程碑逐字写的（写、读、刷盘返回块设备错，加上读回改坏的字节），
    // 后两种是设备说谎（判决第二节改法一把它们抽进样本；`draw_injected_fault` 的 `below(6)` 钉在这一条上）。
    for fault in [
        InjectedFault::WriteFails,
        InjectedFault::ReadFails,
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: 0,
                bit_index: 0,
            },
        },
        InjectedFault::BarrierFails,
        InjectedFault::WriteIsSwallowed,
        InjectedFault::BarrierIsSwallowed,
    ] {
        assert!(
            tally.faults_by_kind.contains_key(fault.name()),
            "一次都没注入过 {}：这一种注入在快档里没跑到\n{rendered}",
            fault.name()
        );
    }
    // 起点段（mkfs、取号、暖机、第一个文件）上注入过：判决第二节改法二的验收那一格，改之前结构性地恒为 0
    // （`draw_faults` 的候选从第 1 步起算）。
    assert!(
        tally.faults_on_segment(FaultedSegmentKind::TheStartingPoint) > 0,
        "起点段上一次都没注入过：`draw_faults` 的候选区间又把第 0 段排掉了\n{rendered}"
    );
    // 说谎的设备真的丢掉过内容：这一格是零的话，说谎那两种注得进去却什么也没碰着，
    // `InjectedFault::the_device_lies` 的那道豁免就成了一张白给的免罪符，判别力要重新量
    //（`mutation-sampling.md`「取样点不敏感」）。
    assert!(
        tally.faults_where_a_lying_device_left_the_image_inconsistent > 0,
        "说谎的设备一次都没把盘面弄脏：说谎那两种抽到了却什么也没碰着\n{rendered}"
    );
    assert!(
        tally.reopens_reading_a_file > 0 && tally.reopens_without_a_file > 0,
        "重开的两种结局没都见过：读回文件 {}、没有文件 {}\n{rendered}",
        tally.reopens_reading_a_file,
        tally.reopens_without_a_file
    );
    assert!(
        tally
            .faults_by_outcome
            .contains_key(FaultOutcome::SurfacedAsAnError.name()),
        "一次都没有「注入那一步返回了错误」：这一件要的就是这一格\n{rendered}"
    );
    // 会写盘的四类操作：发布（覆盖写）、零单元发布、可写挂载、回退挂载、抬 F。冷启动只读不写，不要求写落点上注入过。
    for operation in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::CloseAndMountRollback,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            tally.faults_on_segment(FaultedSegmentKind::Operation(operation)) > 0,
            "{operation:?} 上一次都没注入过（验收要「每个发布步骤与挂载步骤上各注入过至少一次」）\n{rendered}"
        );
    }
    assert!(
        tally.reopens >= tally.faults,
        "每次注入之后都要重开一次：重开 {} 次、注入 {} 次\n{rendered}",
        tally.reopens,
        tally.faults
    );
    assert_eq!(
        tally.reopens_outside_everything_the_model_allows,
        0,
        "重开走到了模型都不允许的版本：{}\n{rendered}",
        how_to_replay(report)
    );
    assert!(
        tally.checker_runs_on_the_image_after_the_fault >= tally.faults,
        "每次注入之后都要对镜像跑一次池级 checker\n{rendered}"
    );
}

/// 大档：规模从环境变量取，后台跑（`cargo test --release -p singlefs-harness --test
/// second_transaction_supplement_three_fault_injection -- --ignored --nocapture`）。
#[test]
#[ignore = "大档：规模从环境变量取，按里程碑「增补 3」后台跑"]
fn fault_injection_large_tier_from_the_environment() {
    let first_seed = number_from_environment(
        "SINGLEFS_FAULT_INJECTION_FIRST_SEED",
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
    );
    let seed_count = number_from_environment("SINGLEFS_FAULT_INJECTION_SEEDS", 512);
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_FAULT_INJECTION_OPERATIONS",
        30,
    ))
    .expect("步数");
    let faults_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_FAULT_INJECTION_FAULTS",
        6,
    ))
    .expect("注入次数");
    let device_width = match std::env::var("SINGLEFS_FAULT_INJECTION_DEVICE_WIDTH").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("4gib") => HistoryDeviceWidth::FourGibibytes,
        Ok("small") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok(other) => panic!("SINGLEFS_FAULT_INJECTION_DEVICE_WIDTH={other} 只认 4gib 与 small"),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_FAULT_INJECTION_DEVICE_WIDTH 不是 UTF-8：{raw:?}")
        }
    };
    let started = std::time::Instant::now();
    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights: GenerationWeights::BROAD,
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
    });
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入大档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
}

/// 说谎的设备许可留下的盘面不一致里 `["I-3.1"]` 那一组的取样点（`fault_injection::INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE`；
/// `crates/mutations.tsv` 钉着把那一组去掉的变异）。快档那 24 段里说谎的设备留下的不一致全落在另一组（`["I-2.1", "I-4.8", "I-7.4"]`），
/// 那一组去掉了快档照样绿；大档（这个测试周期的种子基起 512 段、每段 30 步、注入 6 次）里 `["I-3.1"]` 那一组落在 13 个注入点上。
/// 这里只跑其中一段：种子 7463871032432355306，同一组规模下抽出的 6 个注入点里有一次被吞掉的写（`write_is_swallowed`，
/// 整池第 107 次写调用）落在 `step_index` 6 那一步（覆盖写）上，之后池级 checker 只判红 I-3.1。它要被白名单豁免、记进说谎那一格，
/// 不算新发现（这一格为什么只剩 I-3.1、丢的是哪一份，没有逐字节追过）。
#[test]
fn a_swallowed_write_after_which_the_checker_flags_only_i_3_1_is_excused_as_what_a_lying_device_may_leave(
) {
    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
        first_seed: 7_463_871_032_432_355_306,
        seed_count: 1,
        operations_per_history: 30,
        faults_per_history: 6,
        weights: GenerationWeights::BROAD,
        execution: CHECKED_AFTER_EVERY_STEP,
        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
    });
    let rendered = report.render();
    assert!(
        report.new_findings.is_empty(),
        "说谎的设备留下的 I-3.1 要被白名单豁免，不算新发现；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_eq!(
        report
            .tally
            .lying_device_signatures
            .get("CheckerViolations { invariants: [\"I-3.1\"] }"),
        Some(&2),
        "这一段里被吞掉的那一次写之后只判红 I-3.1，记进说谎那一格——带着注入跑的那一遍里每一步之后的池级 checker 判红那一处、\
         注入之后的镜像上再跑的那一遍，各记一次：\n{rendered}"
    );
}

/// 一段写死的历史上把注入点摆满：同一段历史注入 12 次，逐次判「返回错误而不是 panic」。
#[test]
fn one_fixed_history_injects_twelve_faults_and_none_of_them_panics() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        24,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, CHECKED_AFTER_EVERY_STEP, 12);
    let rendered = injection.tally.render();
    print_uncaptured(&format!("── 故障注入：一段写死的历史 ──\n{rendered}"));
    assert!(
        injection.drawn_faults.len() >= 8,
        "12 次抽里摆出来的注入点太少（{}）",
        injection.drawn_faults.len()
    );
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty(),
        "「已知红」清单外的失败\n{rendered}"
    );
}

/// 摆注入点的候选从第 0 段起算，起点段摆得到：`draw_faults` 的 `(0..marks.len())` 改回 `(1..marks.len())`，
/// 起点段的注入点恒为 0，这里必红（判决第二节改法二钉的那一格；`crates/mutations.tsv` 同名的那一条）。
#[test]
fn the_drawn_injection_points_reach_the_starting_segment() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        8,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, CHECKED_AFTER_EVERY_STEP, 12);
    let rendered_faults: Vec<String> = injection
        .drawn_faults
        .iter()
        .map(DrawnFault::render)
        .collect();
    assert!(
        injection
            .drawn_faults
            .iter()
            .any(|drawn| drawn.segment == FaultedSegment::TheStartingPoint),
        "12 次抽里一个起点段的注入点都没摆出来：{rendered_faults:?}"
    );
    assert_eq!(
        injection.tally.faults_that_panicked,
        0,
        "起点段上注入打出了 panic\n{}",
        injection.tally.render()
    );
}

/// 只有起点段的一段历史：操作 0 步，跑起来只有 mkfs、取号、暖机、第一个文件。
/// 比重挑一律从第一个文件起的那一组，起点段四步才都走得到。
fn a_history_with_only_the_starting_point() -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        0,
        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
    )
}

/// 在整池第 `ordinal` 次写上注一次写报错，跑那一段历史。`ordinal` 为 `None` 时不注入。
fn run_the_starting_point(
    history: &GeneratedHistory,
    failing_write_ordinal: Option<u64>,
) -> (HistoryRun, SharedFaultPlan) {
    let geometry = CHECKED_AFTER_EVERY_STEP.device_width.fixed_geometry();
    let plan = match failing_write_ordinal {
        None => SharedFaultPlan::unarmed(geometry),
        Some(ordinal) => SharedFaultPlan::armed(
            geometry,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, ordinal),
        ),
    };
    let run = execute_history_with_faults(
        history,
        CHECKED_AFTER_EVERY_STEP,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    (run, plan)
}

/// 起点段失败时那条观察：没 panic，而且停在「起点段的入口返回了错误」上，交回是哪一步。
fn the_starting_point_step_that_returned_an_error(
    run: &HistoryRun,
    ordinal: u64,
) -> Option<StartingPointStep> {
    let observation = match &run.ending {
        HistoryEnding::Completed => return None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => observation,
    };
    assert!(
        observation.panic.is_none(),
        "起点段第 {ordinal} 次写上注入打出了 panic：{:?}",
        observation.panic
    );
    match &observation.harness_judgement {
        Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),
        Some(_) | None => {
            panic!("起点段第 {ordinal} 次写上注入之后停在别的东西上：{observation:?}")
        }
    }
}

/// 起点段（mkfs、取号、暖机、第一个文件）的四个入口逐个注入：一次 panic 都没有，四步各至少有一次把错交回来。
///
/// 判决第二节改法二钉的就是这一格：改之前这四步在执行器里是 `expect`，注入在起点段只会撞出执行器自己的 panic
/// （判定五：攻方在只改候选区间的探针上实测 49 次，全在 `history.rs` mkfs 那一处）。这条用例不抽样——
/// 起点段每一次写都注入一遍，四步谁少了一处 `expect` 没改，这里必红。
#[test]
fn every_entry_in_the_starting_segment_returns_an_error_instead_of_panicking() {
    let history = a_history_with_only_the_starting_point();
    // 一、不注入跑一遍，数起点段发了多少次读 / 写 / 刷盘（验收要的那个数，这里现量）。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    let calls = measurement.calls();
    print_uncaptured(&format!(
        "── 起点段逐次注入 ──\n起点段的设备调用：读 {}、写 {}、刷盘 {}，合计 {}\n",
        calls.reads,
        calls.writes,
        calls.barriers,
        calls.reads + calls.writes + calls.barriers
    ));
    // 二、起点段的每一次写各注入一遍：记下是哪一步把错交回来的，panic 一次都不许有。
    let mut failures_by_step: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut tolerated = 0u64;
    for ordinal in 1..=calls.writes {
        let (run, plan) = run_the_starting_point(&history, Some(ordinal));
        assert_eq!(plan.fired_count(), 1, "第 {ordinal} 次写没注上");
        match the_starting_point_step_that_returned_an_error(&run, ordinal) {
            None => tolerated += 1,
            Some(step) => *failures_by_step.entry(step.name()).or_insert(0) += 1,
        }
    }
    print_uncaptured(&format!(
        "把错交回来的：{failures_by_step:?}；报了成功、照样跑完的 {tolerated} 次\n"
    ));
    for step in [
        StartingPointStep::MakeFilesystem,
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert!(
            failures_by_step.contains_key(step.name()),
            "起点段的{}一次都没把错交回来（这一处的 `expect` 还在？）：{failures_by_step:?}",
            step.name()
        );
    }
}

/// 起点段失败时跑不跑池级 checker：**mkfs 自己就报错那一次不跑**（盘上没有一个做完了的文件系统，
/// checker 判的每一条都没有对象），**取号 / 暖机 / 第一个文件报错那几次照跑**（mkfs 已经做完，盘上有池）。
///
/// 用户 2026-09-21 定的口径，分界与「重开走读失败算不算失败」同一条：mkfs 做没做完
/// （`history.rs` 的 `StartingPointStep::a_finished_filesystem_is_on_the_devices`）。
/// 起点段 44 次写逐次注入，两边都数出来——口径被改掉（两边都跑、或者两边都不跑），这里必红。
#[test]
fn the_pool_checker_runs_on_a_starting_point_failure_only_when_make_filesystem_finished() {
    let history = a_history_with_only_the_starting_point();
    // 不注入时起点段跑完，起点之后跑一次 checker：下面那两格拿它当对照。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    assert_eq!(
        measured.tally.checker_runs, 1,
        "起点段跑完之后该跑一次池级 checker"
    );
    let mut checker_runs_by_step: BTreeMap<&'static str, BTreeSet<u64>> = BTreeMap::new();
    for ordinal in 1..=measurement.calls().writes {
        let (run, _plan) = run_the_starting_point(&history, Some(ordinal));
        let Some(step) = the_starting_point_step_that_returned_an_error(&run, ordinal) else {
            continue;
        };
        checker_runs_by_step
            .entry(step.name())
            .or_default()
            .insert(run.tally.checker_runs);
        let expected_checker_runs = u64::from(step.a_finished_filesystem_is_on_the_devices());
        assert_eq!(
            run.tally.checker_runs,
            expected_checker_runs,
            "起点段第 {ordinal} 次写上注入、停在{}：mkfs 做完了才该跑 checker",
            step.name()
        );
    }
    print_uncaptured(&format!(
        "── 起点段失败时的 checker ──\n每一步报错时 checker 跑过几次：{checker_runs_by_step:?}\n"
    ));
    // 两边都要有样本，不然上面那条断言是空过的。
    assert_eq!(
        checker_runs_by_step.get(StartingPointStep::MakeFilesystem.name()),
        Some(&BTreeSet::from([0])),
        "mkfs 报错那几次要有样本，而且一次 checker 都没跑：{checker_runs_by_step:?}"
    );
    for step in [
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert_eq!(
            checker_runs_by_step.get(step.name()),
            Some(&BTreeSet::from([1])),
            "{}报错那几次要有样本，而且各跑过一次 checker：{checker_runs_by_step:?}",
            step.name()
        );
    }
}

/// 线程数换了，报告逐字相同：计数按片的次序相加、新发现按种子从小到大留第一个
/// （`implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
#[test]
fn the_report_is_the_same_text_with_one_worker_thread_and_with_four() {
    let campaign = |threads| FaultInjectionCampaign {
        seed_count: 6,
        operations_per_history: 12,
        faults_per_history: 2,
        ..fast_tier_campaign(FaultInjectionWorkerThreads::GivenByCaller(threads))
    };
    let one = run_fault_injection_campaign(&campaign(1));
    let four = run_fault_injection_campaign(&campaign(4));
    assert_eq!(
        one.render(),
        four.render(),
        "线程数变了报告就变了：并到一起的次序不确定"
    );
}

/// 增补 2 收口表第 40 行（C381（根已落盘之后发布失败，分配器仍退回））那一格，按 D23（journal 的角色与格式） 已定项 14
/// 「这一版的失败处置」（用户 2026-09-24 定：发布不接受失败，失败的那次冻结，下一次发布之前逐字节原样重发它）：
/// 发布 B 在最后一步（系统配置槽）失败时根已 FUA 落盘、分配器退回——改之前同一个写入口拿同一个上一版再发一次 C，就把那条根指着的单元
/// 原地盖掉，那一次再断电，盘上留下「最新的那条根指着一份内容已经换掉的单元」（池级 checker 判 I-7.4 / I-7.2 红、冷启动走不到那条根）。
/// 现在 B 冻结在分配器上：再发 C 在任何写之前被拒；原样重发 B 之后那条根指着的单元就是它自己写的那一份，checker 不红、冷启动读回 B；
/// 之后 C 接在 B 上照常发布。
#[test]
fn a_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &first_file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个事务");
    assert_eq!(first.root.checkpoint_txg, CheckpointTxg(3));

    // 一、发布 B：系统配置槽那两次写全报错。根槽 FUA 写在它们之前，已经落盘。
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(parameters.geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let refused = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &second_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect_err("系统配置槽写报错，这次发布必须失败");
    assert!(
        matches!(refused, PublishError::BlockDevice(_)),
        "报的应当是块设备错：{refused:?}"
    );
    let fired = plan.fired();
    assert_eq!(fired.len(), 1, "系统配置槽第一次写就报错、发布到此为止");
    assert_eq!(
        fired[0].written_structure,
        Some(StepKind::SystemConfigurationSlot),
        "注入的那一次要落在系统配置槽上"
    );
    let image_after_the_failed_publish = image_from(&stream);
    let roots_after: Vec<u64> = readable_root_txgs(&image_after_the_failed_publish);
    assert!(
        roots_after.contains(&4),
        "C381 的前提：这次发布报了失败，txg 4 那条根却已经 FUA 落盘、在环里读得出来；读到的是 {roots_after:?}"
    );

    // 二、同一个写入口、同一个上一版再发一次：B 冻结着，第一道就拒，一个写都不发。
    let fired_before_the_refused_publish = plan.fired_count();
    let operations_before_the_refused_publish = stream.operations().len();
    let refused_again = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect_err("B 冻结着，另建一次发布必须被拒");
    assert!(
        matches!(
            refused_again,
            PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: CheckpointTxg(4),
                ..
            }
        ),
        "报的应当是「B 冻结着、还没重发」：{refused_again:?}"
    );
    assert_eq!(
        (plan.fired_count(), stream.operations().len()),
        (
            fired_before_the_refused_publish,
            operations_before_the_refused_publish
        ),
        "被拒的那一次一个写、一道屏障都没发"
    );

    // 三、盘好了，原样重发 B；之后 C 接在 B 上照常发布。
    plan.disarm();
    let second = resend_the_frozen_publish(&mut writer, &mut allocator)
        .expect("盘好了，重发这一遍不再报错")
        .and_then(PoolVersion::into_file_version)
        .expect("冻结着的是带文件的发布 B");
    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    let image_after_the_resend = image_from(&stream);
    let violations_after_the_resend: Vec<&'static str> = check_pool_image(&image_after_the_resend)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert!(
        violations_after_the_resend.is_empty(),
        "重发之后最新那条根（B 的 txg 4）指着的单元就是它自己写的那一份：checker 不红，判红的是 {violations_after_the_resend:?}"
    );
    let recovery = recover(&image_after_the_resend, JournalPolicy::Consult);
    assert!(
        matches!(
            &recovery.outcome,
            RecoveryOutcome::FileRead { root, content }
                if *root == (instance, CheckpointTxg(4)) && *content == second_content()
        ),
        "冷启动读回 B：{:?}",
        recovery.outcome
    );
    let third = publish_overwrite(
        &mut writer,
        &mut allocator,
        &second,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect("重发之后 C 照常发布");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));
}

/// 起点（mkfs、取号 1、暖机、第一个文件）之后只做一步：关掉会话、可写挂载（取号 2 → 写行 → 暖机）。
fn a_history_that_mounts_once_after_the_first_file() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::CloseAndMountWritable],
    }
}

/// 那一步可写挂载里的第几次写：取号是头两次（两块盘各一次系统配置槽写），写行那次发布的第一个单元写是第 3 次；
/// 写行那次发布一共 15 次写（实例表与四个固定点单元各两盘 10 次、记录两盘 2 次、根槽 1 次、系统配置槽轮换 2 次），
/// 暖机第一次空发布的第一个单元写是第 2 + 15 + 1 = 18 次。
const FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT: u64 = 3;
const FIRST_WARM_UP_WRITE_OF_THE_MOUNT: u64 = 18;

/// C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）按用户 2026-09-23 定的那一边（认了，写成已知行为；
/// D23（journal 的角色与格式） 已定项 16 的射程）断言：取号之后写行或暖机那一次写报块设备错，挂载返回错误，
/// 重开的盘上系统配置的实例代号**还是新号 2**（没回卷成 1），这一次记进故障注入的失败账
/// （`FaultInjectionTally::acquired_instances_left_after_a_failed_mount`）、报告里点名。两格各一条：
/// 写行那次的第一个单元写报错 ⇒ 新号一条根都没有（烧掉）；暖机第一次的第一个单元写报错 ⇒ 新号已有写行那次的根。
///
/// 判别力自证：把挂载改成「发布报错之后把系统配置的实例代号写回旧号」（回卷，`crates/mutations.tsv` 里那一条），
/// 重开的盘上系统配置是 1，这一格记不上，这里红。
#[test]
fn a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it(
) {
    let history = a_history_that_mounts_once_after_the_first_file();
    let mount_step = FaultedSegment::Operation {
        step_index: 0,
        operation_kind: HistoryOperationKind::CloseAndMountWritable,
    };
    for (call_within_the_mount, expected_kind, expected_highest_root_instance) in [
        (
            FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT,
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            InstanceGeneration(1),
        ),
        (
            FIRST_WARM_UP_WRITE_OF_THE_MOUNT,
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
            InstanceGeneration(2),
        ),
    ] {
        let injection = inject_one_fault_into_the_segment(
            &history,
            CHECKED_AFTER_EVERY_STEP,
            InjectedFault::WriteFails,
            mount_step,
            call_within_the_mount,
        );
        let rendered = injection.tally.render();
        print_uncaptured(&format!(
            "── C378：可写挂载第 {call_within_the_mount} 次写报错 ──\n{rendered}"
        ));
        assert_eq!(
            injection.acquired_instances_left,
            vec![AcquiredInstanceLeft {
                drawn: injection.drawn_faults[0],
                kind: expected_kind,
                system_configuration_instance_before_the_step: InstanceGeneration(1),
                system_configuration_instance_after_the_reopen: InstanceGeneration(2),
                highest_root_instance_after_the_reopen: Some(expected_highest_root_instance),
            }],
            "取号之后第 {call_within_the_mount} 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1\n{rendered}"
        );
        assert_eq!(
            injection.tally.acquired_instances_left_after_a_failed_mount,
            BTreeMap::from([(expected_kind.name(), 1)]),
            "失败账记下这一次\n{rendered}"
        );
        assert!(
            rendered.contains("取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）")
                && rendered.contains(&format!("  {}：1 次", expected_kind.name())),
            "报告里点名这一格\n{rendered}"
        );
        // 注入真打在那一步可写挂载的单元写上，那一步返回了错误：不是别处先停了。
        assert_eq!(
            injection.tally.faults_by_injection_point,
            BTreeMap::from([("CloseAndMountWritable/unit_write".to_string(), 1)]),
            "注入点\n{rendered}"
        );
        assert_eq!(
            injection
                .tally
                .faults_by_outcome
                .get(FaultOutcome::SurfacedAsAnError.name())
                .copied(),
            Some(1),
            "可写挂载返回了错误\n{rendered}"
        );
        // 记下的错误成员恰好一次、而且是发布那一步的：空表上「每一个都是」恒真，所以先钉条数。
        assert_eq!(
            injection.tally.refusal_members.values().sum::<u64>(),
            1,
            "注入那一步返回的错误成员记了一次\n{rendered}"
        );
        assert!(
            injection
                .tally
                .refusal_members
                .keys()
                .all(|member| member.starts_with("MountError::Publish(")),
            "返回的是发布那一步的错误成员\n{rendered}"
        );
        assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
        assert!(
            injection.new_findings.is_empty(),
            "认下的行为不算新发现，别的也不许有\n{rendered}"
        );
    }
}

/// 发布 B 的内容（4100 字节，与虚机二进制 `second-transaction` 模式相同）。
fn second_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 发布 C 的内容：与发布 B 不同，单元字节才会真的变。
fn third_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 11 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 录制流里真正落到盘上的那些写重建出来的镜像。
fn image_from(stream: &SharedStream) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&stream.retained_operations());
    image
}

/// 环里自证过的根的 txg，从小到大。
fn readable_root_txgs(image: &MemoryPool) -> Vec<u64> {
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let mut txgs: Vec<u64> = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg.0)
    .collect();
    txgs.sort_unstable();
    txgs
}
