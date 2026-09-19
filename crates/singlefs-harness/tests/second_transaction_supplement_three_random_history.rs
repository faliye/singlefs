//! 里程碑「第二个事务」增补 3 第 1 件：随机历史。生成器、执行器、「已知红」清单与收缩在 `singlefs_harness::history`；
//! 这里是快档（普通 `cargo test`）、大档（`#[ignore]`，种子数与步数从环境变量取）与清单每一条的复现。

use std::io::Write as _;

use singlefs_core::address::CheckpointTxg;
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::history::{
    allocated_and_walked_bytes, classify_failure, execute_history, execute_history_observing,
    generate_history, raised_floor_lands_only_on_abandoned_roots, run_history_campaign,
    shrink_to_reproduction, AppliedEffect, ContentChoice, ContentLength, FailureObservation,
    FailureSignature, FindingShrinking, FloorTargetChoice, GeneratedHistory, GenerationWeights,
    HistoryEnding, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    HistoryTally, MountAllocationComparison, NewFindingReport, RecordReuse, RollbackTargetChoice,
    StepOutcome, StepPosition, KNOWN_RED_FORMS,
};
use singlefs_harness::SharedStream;

/// 快档的种子区间与每段步数：写死，门禁每次跑同一批。
const FAST_TIER_FIRST_SEED: u64 = 0;
const FAST_TIER_SEEDS: u64 = 96;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 30;

/// 第 121 行那一类的专门取样点：种子区间与每段步数，写死（判出率与窗口大小见那条用例的注释）。
const REUSE_SAMPLING_FIRST_SEED: u64 = 0;
const REUSE_SAMPLING_SEEDS: u64 = 48;
const REUSE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：快档通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

fn worker_threads_by_default() -> usize {
    std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(16)
}

fn count_of(counts: &std::collections::BTreeMap<String, u64>, key: &str) -> u64 {
    counts.get(key).copied().unwrap_or(0)
}

/// 各条路径都真的跑到了：每类操作至少一次 Ok，回退的目标候选集里外都试过，抬 F 撞过上限也回收过落点，复用改写过已释放的记录
/// （含跨度变了的与被删的），六种内容长度都进过入口，冷启动读回过文件，checker 真的判过 I-3.1 与 I-5.4，至少一段历史转过根环。
fn assert_every_path_was_exercised(tally: &HistoryTally) {
    for kind in HistoryOperationKind::ALL {
        let applied = tally
            .operations_by_kind
            .get(&kind)
            .map_or(0, |operation| operation.applied);
        assert!(applied >= 1, "{kind:?} 一次 Ok 都没有");
    }
    let refusals = &tally.refusals_by_member;
    for member in [
        "MountError::RollbackFloorAboveCeiling",
        "MountError::RollbackTargetNotInRing",
        "MountError::RollbackToVersionWithoutFileUnsupported",
        "PublishError::ContentExceedsDataUnit",
        "PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp",
    ] {
        assert!(count_of(refusals, member) >= 1, "没见过 {member}");
    }
    assert!(
        refusals
            .keys()
            .any(|member| member.starts_with("MountError::RollbackTargetNotACandidate")),
        "回退的目标没落到过被抛弃的根或 F 之下的根"
    );
    assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
    assert!(
        tally.records_rewritten_from_released >= 1,
        "一条已释放的记录都没被复用"
    );
    assert!(
        tally.records_rewritten_with_changed_span >= 1,
        "复用时跨度一次都没变过（Z1-d 那条路没跑到）"
    );
    assert!(
        tally.released_records_removed >= 1,
        "复用时一条被罩住的已释放记录都没删过"
    );
    for length in [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 0 },
        ContentLength::OneByteBelowDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitPayloadCapacity,
        ContentLength::OneByteAboveDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitBytes,
    ] {
        assert!(
            tally
                .content_lengths_attempted
                .get(length.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "内容长度「{}」没进过入口",
            length.name()
        );
    }
    // 装得下的四种长度各至少发成一次：只看「进过入口」的话，全被拒也算进过（代码三方第一轮攻方的 B1 / P3）。
    for length in [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 0 },
        ContentLength::OneByteBelowDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitPayloadCapacity,
    ] {
        assert!(
            tally
                .content_lengths_published
                .get(length.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "装得下的内容长度「{}」一次都没发成",
            length.name()
        );
    }
    assert!(
        count_of(&tally.recovery_outcomes, "FileRead") >= 1,
        "冷启动一次都没读回文件"
    );
    for invariant in ["I-3.1", "I-5.4"] {
        assert!(
            tally.invariant_holds.get(invariant).copied().unwrap_or(0) >= 1,
            "checker 一次都没判过 {invariant}（全是不适用）"
        );
    }
    assert!(
        tally.histories_that_turned_the_root_ring >= 1,
        "没有一段历史转过根环"
    );
}

/// 快档：种子 [0, 96)、每段 30 步。每一步之后跑池级 checker；入口的 `Err` 算合法结局，panic 与违例算失败，撞到「已知红」清单里的形态
/// 照记、那段到此为止，清单外的一条都不许有（有就按签名归类、报出种子与失败在哪一步；收缩交给「收缩一个种子」那条 `#[ignore]` 用例，
/// debug 下收缩一类要两分多钟）。计数照打，并核各条路径真的跑到了。
#[test]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        FAST_TIER_FIRST_SEED,
        FAST_TIER_SEEDS,
        FAST_TIER_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    assert_every_path_was_exercised(&report.tally);
}

/// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：比重取 `GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`，
/// 种子 [0, 48)、每段 30 步（代码三方第一轮判决第二节第 7 条）。2026-09-18 在 release 下量：第 121 行的变异施加之后，这组比重在种子
/// [0, 6000) × 30 步上 799 个种子判出（13.3%），按 48 个种子一窗切 125 窗、每窗至少 2 个；门禁这一窗 [0, 48) 判出 7 个。
/// 先判清单外的失败（变异下红在这一条），再核这一路真的跑到了：罩住别的已释放记录的起点、复用时跨度变了、抬 F 回收到落点——
/// 这几个数不看罩住的那条删没删，变异下照样成立，门禁 59 号看到的红只会是分类判出来的。
#[test]
fn reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        REUSE_SAMPLING_FIRST_SEED,
        REUSE_SAMPLING_SEEDS,
        REUSE_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向抬 F 之后复用的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let tally = &report.tally;
    assert!(
        tally.released_records_covered >= 1,
        "新分配的记录一次都没罩住别的已释放记录的起点（第 121 行那条路没跑到）"
    );
    assert!(
        tally.records_rewritten_with_changed_span >= 1,
        "复用时跨度一次都没变过"
    );
    assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
}

/// 「已知红」清单第 0 条（增补 2 收口表第 ② 行）的复现：第一个文件之后可写挂载一次（写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写。
/// 根环 R × S = 24 槽：txg 24、25、26 依次盖掉第 0 代根与两条暖机根，txg 26 那次覆盖写之后再没有一条有效根引用 mkfs 的第 0 版树表单元
/// （1 槽），它在 txg 3 释放、没回收，记账仍算已分配 ⇒ checker 判 I-3.1 红（记账比遍历多 16384 字节）、别的不变量不红，分类成清单第 0 条；
/// 之前每一步之后 checker 全绿。
/// 这一条修好之后本用例要红——那时把清单第 0 条删掉、这里改成「跑完」。
#[test]
fn turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form() {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(overwrite, 25))
            .collect(),
    };
    let run = execute_history(&history);
    let HistoryEnding::KnownRed { form, observation } = &run.ending else {
        panic!("要以已知红收尾：{:?}", run.ending);
    };
    assert_eq!(*form, 0, "{}", KNOWN_RED_FORMS[*form].shape);
    assert_eq!(
        observation.position,
        StepPosition::Operation(21),
        "第 21 步是 txg 26 那次覆盖写（txg 4、5 是写行与暖机，覆盖写从 txg 6 起）"
    );
    assert_eq!(observation.newest_ring_root_txg, Some(26));
    assert_eq!(observation.root_ring_slot_count, Some(24));
    assert_eq!(
        observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        vec!["I-3.1"]
    );
    let (allocated, walked) = allocated_and_walked_bytes(&observation.violations[0].1)
        .expect("I-3.1 的违例文字带记账与遍历两个数");
    assert_eq!(
        allocated - walked,
        16384,
        "差的正是 mkfs 那 1 槽第 0 版树表单元"
    );
    assert_eq!(
        run.tally.checker_runs, 23,
        "起点、挂载、前 20 次覆盖写之后各跑一次都是绿的，第 21 次覆盖写之后那一次判红"
    );
}

/// 「已知红」清单第 1 条（增补 2 收口表第 43 行）的复现，2026-09-18 在快档种子 80 上撞到、收缩出来的那一段（种子号随生成器的比重变，
/// 这一段不随）：可写挂载（实例 2，txg 4、5）、覆盖写两次（6、7）、回退到 (2, 7)（实例 3，txg 8–10）、再回退到 (2, 7)（实例 4，
/// txg 11–13：实例 3 的三条根被抛弃）、覆盖写（14）、可写挂载（实例 5，txg 15、16）、覆盖写三次（17–19）、抬 F 到 8（txg 20–22）。
/// F = 8 那个 txg 上的根被抛弃了，(2, 7) 落到 F 之下出了候选集，而它的实例表与四个固定点单元（6 槽）在 txg 11 才释放、释放代 11 > 8
/// 不回收 ⇒ 记账比遍历多 6 × 16384 字节，I-3.1 红。这一条修好之后本用例要红——那时把清单第 1 条删掉、这里改成「跑完」。
#[test]
fn raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form() {
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
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
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
    let run = execute_history(&history);
    let HistoryEnding::KnownRed { form, observation } = &run.ending else {
        panic!("要以已知红收尾：{:?}", run.ending);
    };
    assert_eq!(*form, 1, "{}", KNOWN_RED_FORMS[*form].shape);
    assert_eq!(observation.position, StepPosition::Operation(10));
    assert_eq!(
        run.outcomes[10],
        StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(8),
            publishes: 3,
            reclaimed_placements: 26,
            reuse: RecordReuse::default(),
        })
    );
    let (allocated, walked) = allocated_and_walked_bytes(&observation.violations[0].1)
        .expect("I-3.1 的违例文字带记账与遍历两个数");
    assert_eq!(
        allocated - walked,
        6 * 16384,
        "(2, 7) 的实例表 2 槽与四个固定点单元各 1 槽"
    );
}

/// 不经回退的抬 F 之后 I-3.1 记账多算是新发现，不是清单第 1 条（代码三方第一轮判决第二节第 1 条）。历史照攻方的 `opus_z2_history.rs`：
/// 第一个文件之后可写挂载（实例 2，txg 4、5）、覆盖写三次（6、7、8）、抬 F（选择子 3 ⇒ F = 3），一次回退都没有。今天的代码上这段跑完，
/// 抬 F 之前的镜像上 txg 3 那条根是实例 1 的有效根 ⇒ 「F 那个 txg 上的根全属于被抛弃的实例」为假；拿它配上攻方在回收门槛差一
/// （A1）下量到的那一步违例（`盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040`），分类是新发现。同一个判定在第 1 条的复现
/// （种子 80 那一段，F = 8 落在被抛弃的实例 3 上）为真，那一条由上面的用例钉着。
#[test]
fn an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding() {
    let three_overwrites = std::iter::repeat_n(
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed: 7,
        }),
        3,
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(three_overwrites)
            .chain(std::iter::once(HistoryOperation::RaiseRollbackFloor(
                FloorTargetChoice {
                    steps_above_current_floor: 3,
                },
            )))
            .collect(),
    };
    let mut image_before_raising: Option<MemoryPool> = None;
    let run = execute_history_observing(&history, &SharedStream::new(), &mut |observation| {
        if observation.position == StepPosition::Operation(3) {
            image_before_raising = Some(observation.image.clone());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "今天的代码上这段跑完");
    let StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. }) = run.outcomes[4] else {
        panic!("第 4 步抬 F 要成：{:?}", run.outcomes[4]);
    };
    assert_eq!(new_floor, CheckpointTxg(3));
    let lands_only_on_abandoned = raised_floor_lands_only_on_abandoned_roots(
        &image_before_raising.expect("观察者见过第 3 步之后的镜像"),
        new_floor,
    );
    assert_eq!(
        lands_only_on_abandoned,
        Some(false),
        "txg 3 那条根是实例 1 的有效根，不在任何回退留下的空档里"
    );
    let observation = FailureObservation {
        position: StepPosition::Operation(4),
        operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
        violations: vec![(
            "I-3.1",
            "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040".to_string(),
        )],
        panic: None,
        newest_ring_root_txg: Some(10),
        root_ring_slot_count: Some(24),
        harness_judgement: None,
        raised_floor_lands_only_on_abandoned_roots: lands_only_on_abandoned,
    };
    let ending = classify_failure(observation);
    assert!(
        matches!(
            &ending,
            HistoryEnding::NewFinding {
                signature: FailureSignature::CheckerViolations { invariants },
                ..
            } if invariants == &vec!["I-3.1"]
        ),
        "不经回退的抬 F 之后 I-3.1 多算要是新发现：{ending:?}"
    );
}

/// 挂载写出的写行与暖机那几次发布也比分配代（代码三方第二轮判决第二节第 1 条，照攻方的改法，被攻过零轮）。历史是攻方的形态 a
/// （只在挂载里复用改写已回收记录时不改分配代）在专门取样点的比重下种子 2 收缩出来的 11 步：从 mkfs 起，可写挂载、第一个文件、
/// 可写挂载、覆盖写、抬 F（到 0）、覆盖写四次、抬 F 到 6（回收 18 个落点）、再可写挂载——最后这次挂载的写行与暖机复用改写了已回收的记录。
/// 今天的代码上这段跑完；最后一步的挂载逐次比了两次带文件的发布、复用改写 14 条。形态 a / b 下最后一步是执行器判出的失败，本用例红。
#[test]
fn row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked(
) {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(2),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 0,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let run = execute_history(&history);
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "今天的代码上这段跑完：挂载写出的发布里复用改写的记录，代都是那一次的 txg"
    );
    let StepOutcome::Applied(AppliedEffect::Mounted {
        allocation_records_compared,
        reuse,
        ..
    }) = &run.outcomes[10]
    else {
        panic!("最后一步挂载要成：{:?}", run.outcomes[10]);
    };
    assert_eq!(
        *allocation_records_compared,
        MountAllocationComparison::Compared { publishes: 2 },
        "写行与一次暖机都带文件，逐次比过"
    );
    assert!(
        reuse.rewritten_from_released >= 1,
        "这次挂载的发布真的复用改写了已回收的记录：{reuse:?}"
    );
}

/// 同一个种子跑两次，每一步的结局、收尾、计数逐项相同，每一步之后的整份镜像逐字节相同（随机源手写、写入时间是参数）。
#[test]
fn the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice() {
    let history = generate_history(HistorySeed(7), 24);
    let run_keeping_every_image = || {
        let mut images: Vec<MemoryPool> = Vec::new();
        let run = execute_history_observing(&history, &SharedStream::new(), &mut |observation| {
            images.push(observation.image.clone());
        });
        (run, images)
    };
    let (first, first_images) = run_keeping_every_image();
    let (second, second_images) = run_keeping_every_image();
    assert_eq!(first, second);
    assert!(
        first.tally.checker_runs >= 2,
        "起点之后与至少一步写之后都跑了 checker：{}",
        first.tally.checker_runs
    );
    assert!(
        first_images.len() >= 2,
        "观察者至少见到起点与一步之后的镜像：{}",
        first_images.len()
    );
    assert!(first_images == second_images, "每一步之后的镜像逐字节相同");
}

/// 大档：种子数、每段步数、第一个种子、线程数、比重（`broad` / `reuse`）、收不收缩（`every` / `none`）从环境变量取。
#[test]
#[ignore = "大档：SINGLEFS_RANDOM_HISTORY_SEEDS 段、每段 SINGLEFS_RANDOM_HISTORY_OPERATIONS 步，从 SINGLEFS_RANDOM_HISTORY_FIRST_SEED 起，SINGLEFS_RANDOM_HISTORY_THREADS 个线程，比重 SINGLEFS_RANDOM_HISTORY_WEIGHTS，收缩 SINGLEFS_RANDOM_HISTORY_SHRINK；release 下后台跑"]
fn random_histories_large_tier_seeds_and_length_from_the_environment() {
    let first_seed = number_from_environment("SINGLEFS_RANDOM_HISTORY_FIRST_SEED", 0);
    let seed_count = number_from_environment("SINGLEFS_RANDOM_HISTORY_SEEDS", 1000);
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_OPERATIONS",
        60,
    ))
    .expect("步数装得进 usize");
    let worker_threads = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_THREADS",
        u64::try_from(std::thread::available_parallelism().map_or(1, usize::from)).expect("线程数"),
    ))
    .expect("线程数装得进 usize");
    let weights = match std::env::var("SINGLEFS_RANDOM_HISTORY_WEIGHTS").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("broad") => GenerationWeights::BROAD,
        Ok("reuse") => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        Ok(other) => panic!("SINGLEFS_RANDOM_HISTORY_WEIGHTS={other}：只认 broad 与 reuse"),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_RANDOM_HISTORY_WEIGHTS 不是 UTF-8：{raw:?}")
        }
    };
    let shrinking = match std::env::var("SINGLEFS_RANDOM_HISTORY_SHRINK").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("every") => FindingShrinking::EveryFindingClass,
        Ok("none") => FindingShrinking::ReportSeedsOnly,
        Ok(other) => panic!("SINGLEFS_RANDOM_HISTORY_SHRINK={other}：只认 every 与 none"),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_RANDOM_HISTORY_SHRINK 不是 UTF-8：{raw:?}")
        }
    };
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        first_seed,
        seed_count,
        operations_per_history,
        &weights,
        worker_threads,
        shrinking,
    );
    let rendered = report.render();
    let every_new_finding_seed: Vec<(u64, &FailureSignature)> = report
        .new_findings
        .iter()
        .flat_map(|finding| {
            finding
                .seeds
                .iter()
                .map(move |seed| (seed.0, &finding.signature))
        })
        .collect();
    print_uncaptured(&format!(
        "── 随机历史大档 ──\n{rendered}每个新发现的种子：{every_new_finding_seed:?}\n用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
}

/// 收缩一个种子：快档只报种子，拿这条把它收到最短复现（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED`，步数默认同快档）。
#[test]
#[ignore = "收缩一个种子：SINGLEFS_RANDOM_HISTORY_SHRINK_SEED 必给，SINGLEFS_RANDOM_HISTORY_OPERATIONS 默认 30，SINGLEFS_RANDOM_HISTORY_THREADS 默认全部核"]
fn shrink_one_failing_seed_from_the_environment() {
    let seed = HistorySeed(
        std::env::var("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED")
            .expect("要给 SINGLEFS_RANDOM_HISTORY_SHRINK_SEED")
            .parse()
            .expect("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED 是一个非负整数"),
    );
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_OPERATIONS",
        u64::try_from(FAST_TIER_OPERATIONS_PER_HISTORY).expect("步数"),
    ))
    .expect("步数装得进 usize");
    let worker_threads = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_THREADS",
        u64::try_from(std::thread::available_parallelism().map_or(1, usize::from)).expect("线程数"),
    ))
    .expect("线程数装得进 usize");
    let history = generate_history(seed, operations_per_history);
    let HistoryEnding::NewFinding {
        signature,
        observation,
    } = execute_history(&history).ending
    else {
        panic!(
            "种子 {} 的 {operations_per_history} 步没有撞到新发现",
            seed.0
        );
    };
    let shrunk = shrink_to_reproduction(&history, &signature, worker_threads);
    print_uncaptured(
        &NewFindingReport {
            signature,
            first_seed: seed,
            observation,
            seeds: vec![seed],
            shrunk: Some(shrunk),
        }
        .render(),
    );
}

fn number_from_environment(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| {
        text.parse()
            .unwrap_or_else(|error| panic!("{name}={text} 不是一个非负整数：{error}"))
    })
}
