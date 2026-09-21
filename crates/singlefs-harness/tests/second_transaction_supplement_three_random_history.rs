//! 里程碑「第二个事务」增补 3 第 1 件：随机历史。生成器、执行器、「已知红」清单与收缩在 `singlefs_harness::history`；
//! 这里是快档（普通 `cargo test`）、大档（`#[ignore]`，种子数与步数从环境变量取）与清单每一条的复现。
//!
//! 五段取样的种子基都是 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`：这个测试周期开头抽一次、之后写死
//! （用户 2026-09-20 定案第 7 条），崩溃注入那个二进制用的是同一个；种子数照旧写死，里程碑的验收说的是「写死的种子数之内判红」。

use std::io::Write as _;

use singlefs_checker::image::{chosen_system_configurations, valid_roots};
use singlefs_core::address::CheckpointTxg;
use singlefs_harness::crash::{MemoryPool, RecordCheck};
use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    allocated_and_walked_bytes, allocation_records_on_the_image_under, classify_failure,
    execute_history, execute_history_observing, execute_history_with, generate_history,
    raised_floor_lands_only_on_abandoned_roots, run_history_campaign, shrink_to_reproduction,
    AppliedEffect, ContentChoice, ContentLength, FailureObservation, FailureSignature,
    FindingShrinking, FloorTargetChoice, GeneratedHistory, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind, HistoryRun,
    HistorySeed, HistoryStartingPoint, HistoryTally, MountAllocationComparison, NewFindingReport,
    PerStepChecker, RecordReuse, RollbackTargetChoice, StepOutcome, StepPosition, KNOWN_RED_FORMS,
};
use singlefs_harness::model::{ModelCheckpointTxg, ModelInstanceGeneration, ModelRootKey};
use singlefs_harness::SharedStream;

// 下面五段的种子基都是同一个：这个测试周期开头抽一次、抽完在这个周期之内写死的那个数
// （`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，崩溃注入那个二进制用的是同一个；用户 2026-09-20 定案第 7 条）。
// 种子**数**（96 / 48 / 48 / 32 / 32）与每段步数照旧写死：里程碑的验收说的是「写死的种子数之内判红」，管的是数不是基。
// 下一个测试周期重抽种子基之后，这五段跑的就是另一批历史，那时量过的判出率要重量。

/// 快档的种子区间与每段步数：门禁每次跑同一批。
const FAST_TIER_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
const FAST_TIER_SEEDS: u64 = 96;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 30;

/// 第 121 行那一类的专门取样点：种子区间与每段步数，写死（判出率与窗口大小见那条用例的注释）。
const REUSE_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
const REUSE_SAMPLING_SEEDS: u64 = 48;
const REUSE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;

/// 理想模型 B2 那一格的专门取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
const ROLLBACK_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
const ROLLBACK_SAMPLING_SEEDS: u64 = 48;
const ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;

/// 分配记录墙那一格的取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
const WALL_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
const WALL_SAMPLING_SEEDS: u64 = 32;
const WALL_SAMPLING_OPERATIONS_PER_HISTORY: usize = 150;

/// 单元区墙那一格的取样点（两块小盘）：种子区间与每段步数，写死（判出率见那条用例的注释）。
const UNIT_AREA_WALL_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
const UNIT_AREA_WALL_SAMPLING_SEEDS: u64 = 32;
const UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY: usize = 150;

/// 五段的种子基都是这个测试周期的常量：两个二进制（随机历史与崩溃注入）同基，
/// `crates/mutations.tsv` 第 146–178 行那些「这条变异必须红」与里程碑的验收才是可判的命题——
/// 哪一段被改回写死的别的数，那一段跑的就是另一批历史，那些判红不再是同一个命题。
/// 常量本身是不是这个周期 2026-09-20 抽出来的那个数，由崩溃注入那个二进制的
/// `the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle` 钉。
#[test]
fn the_five_sampling_tiers_start_from_the_test_cycle_seed_base() {
    for (tier, first_seed) in [
        ("快档", FAST_TIER_FIRST_SEED),
        ("偏向抬 F 之后复用的取样点", REUSE_SAMPLING_FIRST_SEED),
        ("偏向抬 F 之后回退的取样点", ROLLBACK_SAMPLING_FIRST_SEED),
        ("逼近分配记录墙的取样点", WALL_SAMPLING_FIRST_SEED),
        (
            "小盘上逼近单元区墙的取样点",
            UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        ),
    ] {
        assert_eq!(
            first_seed, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            "{tier}的种子基是 {first_seed}，不是这个测试周期的种子基：这一段跑的是另一批历史，变异表与验收在它上面量过的判红都不算数"
        );
    }
}

/// 只看准入与模型的写死用例怎么跑：不跑池级 checker（它们要在根环转过之后接着连发，checker 在已知红第 0 条那一形上会先停下），
/// 两块 4 GiB 的盘。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
};

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
        "MountError::RollbackTargetNotACandidate(NotInRing)",
        "MountError::RollbackToVersionWithoutFileUnsupported",
        "PublishError::ContentExceedsDataUnit",
        "PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp",
    ] {
        assert!(count_of(refusals, member) >= 1, "没见过 {member}");
    }
    assert!(
        count_of(
            refusals,
            "MountError::RollbackTargetNotACandidate(BelowEffectiveFloor)"
        ) + count_of(
            refusals,
            "MountError::RollbackTargetNotACandidate(OnAbandonedTimeline)"
        ) >= 1,
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
    assert_the_model_judged_every_kind_of_answer(tally);
}

/// 理想模型真的判过：该拒而拒、该成而成都有，比过根、分配记录、冷启动读回的内容、抬 F 的上限（第 2 件；阴性结果要能和「模型没跑到」分开）。
fn assert_the_model_judged_every_kind_of_answer(tally: &HistoryTally) {
    let counts = &tally.model_counts;
    assert!(tally.model_judged_steps >= 1, "模型一步都没判");
    assert!(
        counts.required_refusals_matched >= 1,
        "模型一次「该拒而拒」都没判过"
    );
    assert!(
        counts.successes_matched >= 1,
        "模型一次「该成而成」都没判过"
    );
    assert!(counts.roots_compared >= 1, "模型一条根都没比过");
    assert!(
        counts.allocation_records_compared >= 1,
        "模型一条分配记录的分配代都没比过"
    );
    assert!(
        counts.cold_start_contents_compared >= 1,
        "模型一次冷启动读回的内容都没比过"
    );
    assert!(counts.ceilings_compared >= 1, "模型一次抬 F 的上限都没比过");
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
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
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
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
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

/// 理想模型那一格 B2（回退到 txg = F_生效 的根被拒，`research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv`）的专门取样点：
/// 比重取 `GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR`（回退目标一半取候选集的下沿），种子 [0, 48)、每段 30 步。快档那组比重与抽法下
/// 回退落到 F 那条根上一次都没有。先判清单外的失败（模型对不上也在其内），再核这一格真的跑到了：回退到 txg = F_生效 > 0 的根做成过——
/// 这个数只在做成时加，变异下被拒、判红的是分类，不是这条计数。
#[test]
fn rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        ROLLBACK_SAMPLING_FIRST_SEED,
        ROLLBACK_SAMPLING_SEEDS,
        ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向抬 F 之后回退的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    assert!(
        report
            .tally
            .model_counts
            .rollbacks_accepted_at_the_effective_floor
            >= 1,
        "回退到 txg = F_生效 > 0 的根一次都没做成（B2 那一格没跑到）"
    );
}

/// 分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：比重取 `GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`，
/// 种子与步数见常量。墙拒时执行器按 checker 的解析从镜像上数准入基数，真条数 ≤ 812 而实现拒了，模型判对不上。每一步之后照跑池级 checker，
/// 「已知红」清单第 0 条那一形只记不停（`PerStepChecker::RunContinuingPastTheRingTurnForm`，第二轮判决第三节第 3 条：第一轮这一段不跑
/// checker，走到 812 条要在根环转过之后连发几十次，而攻方两条只有 checker 看得见的变异——根环转过之后回收门槛多一代、分配记录过 600 条
/// 之后「已分配」少记一槽——在它上面一段都不红）。先判没有新发现（checker 的别的判红、模型对不上、执行器判出的、panic 都算），再核这一路
/// 真的跑到了：checker 真的跑过、分配记录墙拒过且模型按真条数放行过——这个数只在放行时加，变异下被拒、判红的是分类，不是这条计数。
#[test]
fn allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count()
{
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        WALL_SAMPLING_FIRST_SEED,
        WALL_SAMPLING_SEEDS,
        WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        HistoryExecution {
            per_step_checker: PerStepChecker::RunContinuingPastTheRingTurnForm,
            device_width: HistoryDeviceWidth::FourGibibytes,
        },
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：逼近分配记录墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 除已知红第 0 条那一形之外的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    assert!(
        report.tally.checker_runs > 0,
        "这一段每一步之后跑池级 checker"
    );
    assert!(
        report
            .tally
            .model_counts
            .allocation_record_wall_refusals_over_one_node
            >= 1,
        "分配记录墙一次都没按真条数放行过（墙那一格没跑到）"
    );
}

/// 落点拒绝那一格（增补 3 第 2 件代码三方第二轮判决第三节第 2 条）的取样点：两块单元区 384 槽的小盘（`HistoryDeviceWidth::UnitAreaOf384Slots`），
/// 比重取 `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`，每一步之后照跑池级 checker、已知红第 0 条那一形只记不停，种子与步数见常量。
/// 4 GiB 的盘上前四段一次落点拒绝都走不到，攻方把分配器「每块盘上都没有」报成「小盘写满」（只换原因）四段全绿；这里单元区几十次发布就写满，
/// 胶水只把「每块盘上都没有」映射成单元区墙，别的原因映射成模型没有的理由、判对不上。先判没有新发现，再核这一路真的跑到了：
/// 发布与挂载里都见过「每块盘上都没有」、模型在单元区墙的区间里放行过。
#[test]
fn unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        HistoryExecution {
            per_step_checker: PerStepChecker::RunContinuingPastTheRingTurnForm,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        },
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：小盘上逼近单元区墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 除已知红第 0 条那一形之外的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    let refused_on_every_device = |member_prefix: &str| {
        count_of(
            &report.tally.refusals_by_member,
            &format!("{member_prefix}PlacementRefused(NoFreeSlotOnAnyDevice)"),
        )
    };
    assert!(
        refused_on_every_device("PublishError::") >= 1,
        "覆盖写、第一个文件一次都没被「每块盘上都没有」拒过（用户数据那一处没跑到）"
    );
    assert!(
        report
            .tally
            .model_counts
            .unit_area_wall_refusals_in_the_interval
            >= 1,
        "模型一次都没在单元区墙的区间里放行过（落点拒绝那一格没判过）"
    );
    assert!(
        report.tally.checker_runs > 0,
        "这一段每一步之后跑池级 checker"
    );
}

/// 镜像上最新那条根（按 checker 的读法，(txg, 实例) 最大）的身份。
fn newest_root_on_the_image(image: &MemoryPool) -> ModelRootKey {
    let geometry = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        .expect("历史里的盘上至少一块系统配置自证过");
    let (_, _, newest) = valid_roots(image, &geometry)
        .into_iter()
        .max_by_key(|(_, _, view)| (view.checkpoint_txg, view.instance))
        .expect("根环里至少有 mkfs 的第 0 代根");
    ModelRootKey {
        checkpoint_txg: ModelCheckpointTxg(newest.checkpoint_txg),
        instance: ModelInstanceGeneration(u64::from(newest.instance)),
    }
}

/// 分配记录墙的基数按 checker 的解析从镜像上数（`allocation_records_on_the_image_under`，不看分配器）：一条记录记一个单元、每盘一条、
/// 释放只改写不删（D3（空间分配） 已定项 7）。从 mkfs 起：第 0 代根与零单元写行、暖机那几版树表 0 条、没有分配记录树，数出 0 条；
/// 第一个文件那一版 20 条（八个单元加 mkfs 的实例表与第 0 版树表，每盘各一条）。从第一个文件起连着覆盖写（回收之前不复用），
/// 每次每盘加 8 条：36、52、68。
#[test]
fn allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file() {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 5,
    });
    let count_after_each_step = |history: &GeneratedHistory| {
        let mut counted: Vec<Option<u64>> = Vec::new();
        let run = execute_history_observing(history, &SharedStream::new(), &mut |observation| {
            counted.push(allocation_records_on_the_image_under(
                observation.image,
                newest_root_on_the_image(observation.image),
            ));
        });
        assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
        counted
    };
    let from_make_filesystem = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 2999 },
                fill_seed: 3,
            }),
        ],
    };
    assert_eq!(
        count_after_each_step(&from_make_filesystem),
        vec![Some(0), Some(0), Some(20)],
        "起点（txg 0）、挂载的零单元写行与暖机（txg 1、2）、第一个文件（txg 3）"
    );
    let from_the_first_file = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![overwrite; 3],
    };
    assert_eq!(
        count_after_each_step(&from_the_first_file),
        vec![Some(20), Some(36), Some(52), Some(68)]
    );
}

/// 分配记录墙的边沿（增补 3 第 2 件代码三方第一轮判决第三节第 1 条，攻方变异 W1 的形态：墙的 `>` 写成 `>=`，正好 812 条也拒）：
/// 从第一个文件（txg 3，20 条）起连着可写挂载四次——写行与暖机按根环落点每次加 18、18、26、26 条（txg 4–13），108 条——再连着覆盖写。
/// 回收只在挂载与抬 F 时做，挂载都在根环转圈（txg 24）之前、那时环里最旧的有效根还是 txg 0，一个落点都不回收，所以每次覆盖写正好加 16 条：
/// 第 44 次覆盖写之后正好 812 条（一个节点装满，条款说装得下），第 45 次要 828 条、被墙拒。不跑池级 checker（根环转圈之后已知红第 0 条
/// 会先停下）。今天的代码上这段跑完：812 条那一次做成、镜像上数得 812 条；828 条那一次被拒、模型按镜像上的真条数（812 + 16）放行。
/// W1 下 812 条那一次被拒，模型判「模型说该成、实现拒了」。
#[test]
fn an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused(
) {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 11,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(HistoryOperation::CloseAndMountWritable, 4)
            .chain(std::iter::repeat_n(overwrite, 45))
            .collect(),
    };
    let mut counted_after_each_step: Vec<Option<u64>> = Vec::new();
    let run = execute_history_with(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            counted_after_each_step.push(allocation_records_on_the_image_under(
                observation.image,
                newest_root_on_the_image(observation.image),
            ));
        },
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        counted_after_each_step[..6],
        [Some(20), Some(38), Some(56), Some(82), Some(108), Some(124)],
        "起点、四次挂载、第一次覆盖写之后镜像上数的条数"
    );
    assert!(
        matches!(
            run.outcomes[47],
            StepOutcome::Applied(AppliedEffect::Published { .. })
        ),
        "第 44 次覆盖写正好装满一个节点，要做成：{:?}",
        run.outcomes[47]
    );
    assert_eq!(
        counted_after_each_step[48],
        Some(812),
        "第 44 次覆盖写之后镜像上正好 812 条"
    );
    assert_eq!(
        run.outcomes[48],
        StepOutcome::Refused {
            member: "PublishError::AllocationRecordsExceedOneNode".to_string()
        },
        "第 45 次要 828 条"
    );
    assert_eq!(
        run.tally
            .model_counts
            .allocation_record_wall_refusals_over_one_node,
        1,
        "模型按镜像上的真条数放行了那一次"
    );
}

/// 一段写死的历史在 4 GiB 的盘上不跑 checker 跑完，交回每一步之后（含起点）镜像上最新那条根下的分配记录条数。
fn run_counting_allocation_records_after_each_step(
    history: &GeneratedHistory,
) -> (HistoryRun, Vec<Option<u64>>) {
    let mut counted_after_each_step: Vec<Option<u64>> = Vec::new();
    let run = execute_history_with(
        history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            counted_after_each_step.push(allocation_records_on_the_image_under(
                observation.image,
                newest_root_on_the_image(observation.image),
            ));
        },
    );
    (run, counted_after_each_step)
}

/// 抬 F 那一路逼近分配记录墙（增补 3 第 2 件代码三方第二轮判决第三节第 4 条：攻方变异 m1c「只在抬 F 路径的准入里多算一个角色」在门禁
/// 32 个种子里只红 1 段，按 32 个一窗切 15 窗有 5 窗一段都不红）。从第一个文件（txg 3，20 条）起可写挂载四次（txg 4–13，108 条）、
/// 覆盖写 43 次（txg 14–56，每次 16 条，796 条）、抬 F 到现行的 F（0；推 txg 57 落盘 0、txg 58 落盘 1 两次空发布，每次每盘 4 条）。
/// 回收只在挂载与抬 F 时做，四次挂载都在根环转过之前、一个落点都不回收；抬 F 回收的槽在带新 F 的根落满两盘之前扣住、这两次空发布用不上。
/// 所以两次空发布的准入基数正好 796、804 条，第二次之后 812 条——一个节点正好装满，条款说装得下。今天的代码上这段跑完、抬 F 做成两次发布；
/// m1c 下第二次空发布按 804 + 10 = 814 条被拒，模型按镜像上数的真条数（804 + 8）判「模型说该成、实现拒了」。
/// 一段历史每一步接着上一步的盘面，次序本身就是被测对象，不切片并行。
#[test]
fn raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds(
) {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 13,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(HistoryOperation::CloseAndMountWritable, 4)
            .chain(std::iter::repeat_n(overwrite, 43))
            .chain(std::iter::once(HistoryOperation::RaiseRollbackFloor(
                FloorTargetChoice {
                    steps_above_current_floor: 0,
                },
            )))
            .collect(),
    };
    let (run, counted_after_each_step) = run_counting_allocation_records_after_each_step(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        counted_after_each_step[47],
        Some(796),
        "抬 F 之前（第 43 次覆盖写之后）镜像上 796 条"
    );
    assert!(
        matches!(
            run.outcomes[47],
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor: CheckpointTxg(0),
                publishes: 2,
                ..
            })
        ),
        "抬 F 推两次空发布、第二次之后正好 812 条，要做成：{:?}",
        run.outcomes[47]
    );
    assert_eq!(
        counted_after_each_step[48],
        Some(812),
        "抬 F 之后镜像上正好 812 条"
    );
}

/// 回退那一路逼近分配记录墙（同一判决第三节第 4 条：攻方变异 m1d「只在回退路径的准入里多算一个角色」在门禁 32 个种子里只红 1 段，
/// 15 窗里 7 窗一段都不红）。从第一个文件（txg 3，20 条）起可写挂载两次（txg 4–7，56 条）、覆盖写一次（txg 8，72 条）、再可写挂载一次
/// （写行 txg 9 落盘 0、暖机 txg 10 落盘 1，90 条）、覆盖写 44 次（txg 11–54，794 条），回退到最新那条根 (54, 实例 4)：它在回退候选集里、
/// 带文件；回退的写行 txg 55 落盘 1、暖机 txg 56 落盘 0，取号之前一串算完的准入是 794 + 10 = 804、804 + 8 = 812 条——一个节点正好装满，
/// 条款说装得下。今天的代码上这段跑完、回退做成两次发布；m1d 下暖机那一次按 804 + 10 = 814 条在取号之前被拒，模型按镜像上数的回退目标
/// 那一版的真条数（794 + 10 + 8）判「模型说该成、实现拒了」。一段历史每一步接着上一步的盘面，不切片并行。
#[test]
fn rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds()
{
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 17,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: [
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CloseAndMountWritable,
            overwrite,
            HistoryOperation::CloseAndMountWritable,
        ]
        .into_iter()
        .chain(std::iter::repeat_n(overwrite, 44))
        .chain(std::iter::once(HistoryOperation::CloseAndMountRollback(
            RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            },
        )))
        .collect(),
    };
    let (run, counted_after_each_step) = run_counting_allocation_records_after_each_step(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        counted_after_each_step[..5],
        [Some(20), Some(38), Some(56), Some(72), Some(90)],
        "起点、两次挂载、一次覆盖写、第三次挂载之后镜像上数的条数"
    );
    assert_eq!(
        counted_after_each_step[48],
        Some(794),
        "回退之前（第 44 次覆盖写之后）最新那条根下 794 条"
    );
    assert!(
        matches!(
            run.outcomes[48],
            StepOutcome::Applied(AppliedEffect::Mounted { publishes: 2, .. })
        ),
        "回退写行与一次暖机、准入到 812 条，要做成：{:?}",
        run.outcomes[48]
    );
}

/// 回退到被抛弃时间线上的根（F 还是 0，不低于 F）要拒，理由是「被抛弃」；模型按判别字段比理由（增补 3 第 2 件代码三方第一轮判决
/// 第三节第 2 条：攻方变异 R1——被抛弃的根报成「txg 低于 F」、只换理由——此前三组套件三段全绿）。第一个文件（实例 1，txg 3）之后
/// 可写挂载（实例 2：写行 txg 4、暖机 5）、覆盖写两次（6、7）、回退到根环从新到旧第 2 条 (5, 2)（实例 3：写行 txg 8、暖机 9、10）、
/// 再回退到从新到旧第 3 条 (7, 2)：最新根的实例表里有回退行 (2, 5)，7 > 5 ⇒ 被抛弃。今天的代码上这段跑完、最后一步报
/// `OnAbandonedTimeline`；R1 下模型判「拒绝的理由」对不上。
#[test]
fn rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned() {
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
            HistoryOperation::CloseAndMountWritable,
            overwrite(6),
            overwrite(7),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
        ],
    };
    let run = execute_history(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[3],
            StepOutcome::Applied(AppliedEffect::Mounted { .. })
        ),
        "回退到 (5, 2) 要做成：{:?}",
        run.outcomes[3]
    );
    assert_eq!(
        run.outcomes[4],
        StepOutcome::Refused {
            member: "MountError::RollbackTargetNotACandidate(OnAbandonedTimeline)".to_string()
        }
    );
    assert_eq!(
        run.tally.model_counts.required_refusals_matched, 1,
        "模型要求拒、理由对上了"
    );
}

/// 回退到 txg 低于 F_生效 的根要拒，理由是「低于 F」（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）；它是候选排除，不是
/// 「树表 0 条、第一版不支持」（增补 3 第 2 件代码三方第二轮判决第三节第 1 条：两者互报的变异里，「低于 F」报成「树表 0 条」那一条在门禁
/// 五段里各只红 1 段，这一条把它钉死）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6，
/// 推 txg 10、11）、回退到根环从新到旧第 6 条 (5, 2)：它带文件、实例 2 还没有行（不是被抛弃的），只因 5 < 6 被挡。今天的代码上这段跑完、
/// 最后一步报 `BelowEffectiveFloor`，模型要求拒、理由对上。一段历史每一步接着上一步的盘面，不切片并行。
#[test]
fn rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor() {
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
            HistoryOperation::CloseAndMountWritable,
            overwrite(6),
            overwrite(7),
            overwrite(8),
            overwrite(9),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 6,
            }),
        ],
    };
    let run = execute_history(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[5],
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor: CheckpointTxg(6),
                ..
            })
        ),
        "抬到 6：{:?}",
        run.outcomes[5]
    );
    assert_eq!(
        run.outcomes[6],
        StepOutcome::Refused {
            member: "MountError::RollbackTargetNotACandidate(BelowEffectiveFloor)".to_string()
        }
    );
    assert_eq!(
        run.tally.model_counts.required_refusals_matched, 1,
        "模型要求拒、理由对上"
    );
}

/// 回退到 txg = F_生效 的根要做成（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效），模型按 B2 那一格判（攻方变异「回退到
/// txg = F_生效 的根也拒」）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6：上限是
/// 第 4 新的非空根 6 与盘 1 上最新的有效根 7 取小，推 txg 10、11 两次）、回退到候选集的下沿 (6, 2)（实例 3：txg 12、13）、冷启动读回。
/// 今天的代码上这段跑完：回退做成一次、落在 F 上，冷启动读回的是 txg 6 那次覆盖写的内容（模型比过）。取样点里这一格 48 个种子一窗只有
/// 0–5 段（2026-09-19 在 [0, 960) 上逐窗数），这一条把它钉死。
#[test]
fn rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version() {
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
            HistoryOperation::CloseAndMountWritable,
            overwrite(6),
            overwrite(7),
            overwrite(8),
            overwrite(9),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRootAtTheNewestFloor),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[5],
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor: CheckpointTxg(6),
                publishes: 2,
                ..
            })
        ),
        "抬到 6、推 txg 10（盘 1）与 11（盘 0）：{:?}",
        run.outcomes[5]
    );
    assert!(
        matches!(
            run.outcomes[6],
            StepOutcome::Applied(AppliedEffect::Mounted { .. })
        ),
        "回退到 (6, 2) 要做成：{:?}",
        run.outcomes[6]
    );
    assert_eq!(
        run.tally
            .model_counts
            .rollbacks_accepted_at_the_effective_floor,
        1,
        "模型数到一次落在 F_生效 上的回退"
    );
    assert_eq!(
        run.tally.model_counts.cold_start_contents_compared, 1,
        "冷启动读回的内容模型比过"
    );
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
            // checker 在 I-3.1 的说明文字里带的机理标识：这里是「F 把 4 条读得到的根挡在了遍历之外」那一种
            // （`singlefs-checker` 的 `walk.rs`）——机理对得上而 F 没落在空档里，所以照样是新发现。
            "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040；机理：根环槽数 24、最新根 txg 10、环里自证过的根槽 11 个、最老的自证过的根 txg 0、遍历的候选根槽 7 个、被实例表判抛弃的根槽 0 个、回退下界 F 3、低于 F 的根槽 4 个".to_string(),
        )],
        panic: None,
        newest_ring_root_txg: Some(10),
        root_ring_slot_count: Some(24),
        harness_judgement: None,
        model_disagreement: None,
        raised_floor_lands_only_on_abandoned_roots: lands_only_on_abandoned,
        record_check: RecordCheck::default(),
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

/// 大档与收缩用的跑法：checker（`SINGLEFS_RANDOM_HISTORY_CHECKER`：`run` / `continue-past-ring-turn` / `skip`，默认 `run`）与盘宽
/// （`SINGLEFS_RANDOM_HISTORY_DEVICES`：`4gib` / `small`，默认 `4gib`）从环境变量取——门禁里那几段的跑法都能在大档上换种子重跑。
fn execution_from_the_environment() -> HistoryExecution {
    let per_step_checker = match std::env::var("SINGLEFS_RANDOM_HISTORY_CHECKER").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("run") => PerStepChecker::Run,
        Ok("continue-past-ring-turn") => PerStepChecker::RunContinuingPastTheRingTurnForm,
        Ok("skip") => PerStepChecker::Skipped,
        Ok(other) => panic!(
            "SINGLEFS_RANDOM_HISTORY_CHECKER={other}：只认 run、continue-past-ring-turn 与 skip"
        ),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_RANDOM_HISTORY_CHECKER 不是 UTF-8：{raw:?}")
        }
    };
    let device_width = match std::env::var("SINGLEFS_RANDOM_HISTORY_DEVICES").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("4gib") => HistoryDeviceWidth::FourGibibytes,
        Ok("small") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok(other) => panic!("SINGLEFS_RANDOM_HISTORY_DEVICES={other}：只认 4gib 与 small"),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_RANDOM_HISTORY_DEVICES 不是 UTF-8：{raw:?}")
        }
    };
    HistoryExecution {
        per_step_checker,
        device_width,
    }
}

/// 大档：种子数、每段步数、第一个种子、线程数、比重（`broad` / `reuse` / `rollback` / `wall` / `unit-area-wall`）、跑法
/// （`execution_from_the_environment`）、收不收缩（`every` / `none`）从环境变量取；
/// 种子基没给就用这个测试周期写死的那一个，跟上面五段同基（给了什么就跑什么）。
#[test]
#[ignore = "大档：SINGLEFS_RANDOM_HISTORY_SEEDS 段、每段 SINGLEFS_RANDOM_HISTORY_OPERATIONS 步，从 SINGLEFS_RANDOM_HISTORY_FIRST_SEED 起（没给就用这个测试周期写死的种子基），SINGLEFS_RANDOM_HISTORY_THREADS 个线程，比重 SINGLEFS_RANDOM_HISTORY_WEIGHTS，checker SINGLEFS_RANDOM_HISTORY_CHECKER，盘宽 SINGLEFS_RANDOM_HISTORY_DEVICES，收缩 SINGLEFS_RANDOM_HISTORY_SHRINK；release 下后台跑"]
fn random_histories_large_tier_seeds_and_length_from_the_environment() {
    let first_seed = number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_FIRST_SEED",
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
    );
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
        Ok("rollback") => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        Ok("wall") => GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        Ok("unit-area-wall") => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        Ok(other) => panic!(
            "SINGLEFS_RANDOM_HISTORY_WEIGHTS={other}：只认 broad、reuse、rollback、wall 与 unit-area-wall"
        ),
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
        execution_from_the_environment(),
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
    let shrunk = shrink_to_reproduction(
        &history,
        &signature,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
    );
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
