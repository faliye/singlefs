//! 里程碑「第二个事务」增补 3 第 1 件：随机历史。生成器、执行器、「已知红」清单与收缩在 `singlefs_harness::history`；
//! 这里是快档（普通 `cargo test`）、大档（`#[ignore]`，种子数与步数从环境变量取）与清单每一条的复现。
//!
//! 五段取样的种子基都是 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`：这个测试周期开头抽一次、之后写死
//! （用户 2026-09-20 定案第 7 条），崩溃注入那个二进制用的是同一个；种子数照旧写死，里程碑的验收说的是「写死的种子数之内判红」。

use std::io::Write as _;

use singlefs_checker::image::{chosen_system_configurations, valid_roots};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PlacementRefusal;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::transaction::TransactionUnit;
use singlefs_harness::crash::{MemoryPool, RecordCheck, SparseBlockDevice};
use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    allocated_and_walked_bytes, allocation_records_on_the_image_under, classify_failure,
    execute_history, execute_history_observing, execute_history_with, generate_history,
    generate_history_with_weights, raised_floor_lands_only_on_abandoned_roots,
    run_history_campaign, shrink_to_reproduction, AppliedEffect, ContentChoice, ContentLength,
    FailureObservation, FailureSignature, FindingShrinking, FloorTargetChoice, GeneratedHistory,
    GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryOperationKind, HistoryRun, HistorySeed, HistoryStartingPoint, HistoryTally,
    MountAllocationComparison, NewFindingReport, PerStepChecker, RecordReuse, RollbackTargetChoice,
    StepOutcome, StepPosition, KNOWN_RED_FORMS,
};
use singlefs_harness::model::{ModelCheckpointTxg, ModelInstanceGeneration, ModelRootKey};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

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

/// 原分配记录墙那一格的取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
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
        ("越过原分配记录墙的取样点", WALL_SAMPLING_FIRST_SEED),
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
        "PublishError::ContentExceedsDataUnit",
        "PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile",
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
    // I-3.11（已分配减 defer 等于最新根走读）、I-7.9（回退下界 F 不高于抬 F 的上限） 与 I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）
    // 也要在随机历史上真被判成立过：零违例之外，还要分得开「判过、成立」与「全是不适用」。
    for invariant in ["I-3.1", "I-5.4", "I-3.11", "I-7.9", "I-9.15"] {
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

/// 原分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：比重取 `GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`，
/// 种子与步数见常量。分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1）没有「一个节点 812 条」那道墙，
/// 模型也没有那条拒绝理由：越过 812 条之后实现因为分配记录拒一次，模型就判对不上。每一步之后照跑池级 checker、判红就停（第二轮判决第三节
/// 第 3 条：攻方两条只有 checker 看得见的变异——根环转过之后回收门槛多一代、分配记录过 600 条之后「已分配」少记一槽——要 checker 才看得见）。
/// 先判没有新发现（checker 的判红、模型对不上、执行器判出的、panic 都算），再核这一路真的跑到了：checker 真的跑过、有一段历史的某一版
/// 分配记录多于 812 条（多叶的树真的写过、读过、判过）。
#[test]
fn allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        WALL_SAMPLING_FIRST_SEED,
        WALL_SAMPLING_SEEDS,
        WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：越过原分配记录墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    assert!(
        report.tally.checker_runs > 0,
        "这一段每一步之后跑池级 checker"
    );
    assert!(
        report.tally.most_allocation_records_in_one_version > 812,
        "没有一段历史越过 812 条分配记录（原先那道墙那一格没跑到）：最多 {} 条",
        report.tally.most_allocation_records_in_one_version
    );
}

/// 落点拒绝那一格（增补 3 第 2 件代码三方第二轮判决第三节第 2 条）的取样点：两块单元区 256 槽的小盘（`HistoryDeviceWidth::UnitAreaOf256Slots`；
/// mkfs 同一个进程那条会话也装上根环表之后，384 槽那一档一次落点拒绝都走不到了），
/// 比重取 `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`，每一步之后照跑池级 checker、判红就停，种子与步数见常量。
/// 4 GiB 的盘上前四段一次落点拒绝都走不到，攻方把分配器「每块盘上都没有」报成「小盘写满」（只换原因）四段全绿；这里单元区写得满，
/// 胶水只把「每块盘上都没有」映射成单元区墙，别的原因映射成模型没有的理由、判对不上。先判没有新发现，再核这一路真的跑到了：
/// 发布里见过「每块盘上都没有」、模型在单元区墙的区间里放行过。
/// 空间准入关掉（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）：判着准入时这几块小盘上式子先拒，一次落点拒绝都走不到
/// （准入判着的那一档见 `unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval`）；
/// 这一档测的是准入放行之后落点仍取不到时那一条兜底拒绝（D3（空间分配） 已定项 5；C545（空间准入罩不住分裂与聚簇段层））。
#[test]
fn unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
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
        "新发现（checker 的判红、模型、执行器的判定与 panic）：\n{rendered}"
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

/// 同一段取样（两块单元区 256 槽的小盘、逼近单元区墙的比重、同一批种子与步数），空间准入判着（产品路径）：
/// D28（挂载期承诺量） 已定项 1 的式子接进发布与可写挂载之后（C363 (b) 判决第四节第 2 条），这几块小盘上墙由式子先拒——
/// 覆盖写被 `PublishError::SpaceAdmissionRefused` 拒、可写挂载在取号之前被 `MountError::SpaceAdmissionRefusedBeforeAcquisition` 拒，
/// 胶水把两者都映射成模型的单元区墙，模型全在允许拒绝的区间里放行（没有新发现：拒之前一个字节都没写、拒的都在区间里）。
/// 先判没有新发现，再核这一路真的跑到了：发布与挂载各被式子拒过、模型在区间里放行过；落点那一道一次都没走到（式子先拒）。
#[test]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        &GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    assert!(
        count_of(
            &report.tally.refusals_by_member,
            "PublishError::SpaceAdmissionRefused"
        ) >= 1,
        "覆盖写一次都没被空间准入拒过（发布路径那一处没跑到）"
    );
    assert!(
        count_of(
            &report.tally.refusals_by_member,
            "MountError::SpaceAdmissionRefusedBeforeAcquisition"
        ) >= 1,
        "可写挂载一次都没在取号之前被空间准入拒过（可写挂载那一处没跑到）"
    );
    assert_eq!(
        count_of(
            &report.tally.refusals_by_member,
            "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)"
        ),
        0,
        "准入判着时式子先拒，落点那一道走不到"
    );
    assert!(
        report
            .tally
            .model_counts
            .unit_area_wall_refusals_in_the_interval
            >= 1,
        "模型一次都没在单元区墙的区间里放行过（准入拒绝那一格没判过）"
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

/// 分配记录条数按 checker 的解析从镜像上数（`allocation_records_on_the_image_under`，不看分配器）：一条记录记一个单元、每盘一条、
/// 释放只改写不删（D3（空间分配） 已定项 7）。从 mkfs 起：第 0 代根与零单元写行、暖机那几版树表 0 条、没有分配记录树，数出 0 条；
/// 第一个文件那一版 28 条：七个单元（数据、extent 根、inode 叶、inode 根、记账、映射、树表）加分配记录树五个节点
/// （D8（核心索引结构） 已定项 14：4 GiB 两块盘上根在第 2 层，每块盘第 1 层一个、单元区起点那片叶一个），加 mkfs 的实例表与第 0 版树表，
/// 每盘各一条。从第一个文件起连着覆盖写（回收之前不复用；这几次的落点都还在单元区起点那片叶里，分配记录树每次重写那五个节点），
/// 每次每盘加 12 条：52、76、100。
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
        vec![Some(0), Some(0), Some(28)],
        "起点（txg 0）、挂载的零单元写行与暖机（txg 1、2）、第一个文件（txg 3）"
    );
    let from_the_first_file = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![overwrite; 3],
    };
    assert_eq!(
        count_after_each_step(&from_the_first_file),
        vec![Some(28), Some(52), Some(76), Some(100)]
    );
}

/// 内容为空的覆盖写：每次重写一个数据单元与提交内生块（extent 根、inode 叶与根、记账树、映射树、树表，加分配记录树按位置寻址
/// 重写的根与装着改了的记录的那几片叶，D8（核心索引结构） 已定项 14），小盘上每盘占 12 至 14 槽。
const EMPTY_CONTENT_OVERWRITE: HistoryOperation =
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    });

/// 增补 2 收口表第 39 行那一族（取号之前的准入不算落点）：两块单元区 240 槽的小盘，从第一个文件起在 mkfs 那条会话里
/// 覆盖写 16 次（txg 4–19；分配记录树按位置寻址之后每次多写几个节点，原先是 22 次，D8（核心索引结构） 已定项 14），
/// 再可写挂载——挂载自己那一串（写行与暖机）拿不到落点。这段历史是单元区墙取样点的比重
/// （`GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`）在 240 槽上跑出来、收缩到最短的（种子 7463871032432355113 在第 28 步
/// 撞上「拒绝之前写了盘」）。改之前取号写完、写行那次才被落点拒绝（`MountError::Publish`）：实例代号一去不回、录制流里多了写与屏障，
/// 模型判「拒绝之前写了盘」。今天取号之前在分配器的拷贝上就取不到，返回 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`：
/// 那一步前后镜像逐字节相同、录制流一步没多，跑完、每一步之后池级 checker 判绿。
/// 同一份盘面（第 16 次覆盖写之后的镜像）另起两块内存盘直接调 `mount_writable`，钉住成员的每个字段，录制流一步都不许有。
/// 覆盖写的次数在草稿副本上按 5–39 次扫过：16、17 两档「每一次都做成、挂载那一步被落点拒」，18 次起覆盖写自己先被拒；
/// 17 次那一档写行那次自己就取不到，16 次这一档写行与第一次暖机都取得到、第二次暖机才取不到——取这一档，钉住预演罩到暖机那几次。
/// 空间准入关掉（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`，执行器与直接调挂载两处都关）：判着准入时 240 槽的盘上
/// 覆盖写没到 16 次就被式子拒、挂载在取号之前被式子拒，走不到预演取不到落点那一道——这里测的是准入放行之后那一条兜底拒绝。
#[test]
fn a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged(
) {
    const OVERWRITES: usize = 16;
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, OVERWRITES)
            .chain(std::iter::once(HistoryOperation::CloseAndMountWritable))
            .collect(),
    };
    let device_width = HistoryDeviceWidth::UnitAreaOf240Slots;
    let stream = SharedStream::new();
    let mut images_after_each_step: Vec<MemoryPool> = Vec::new();
    let mut recorded_steps_after_each_step: Vec<usize> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &stream,
        &mut |observation| {
            images_after_each_step.push(observation.image.clone());
            recorded_steps_after_each_step.push(stream.operations().len());
        },
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes[..OVERWRITES].iter().all(|outcome| matches!(
            outcome,
            StepOutcome::Applied(AppliedEffect::Published { .. })
        )),
        "{OVERWRITES} 次覆盖写都做成：{:?}",
        &run.outcomes[..OVERWRITES]
    );
    assert_eq!(
        run.outcomes[OVERWRITES],
        StepOutcome::Refused {
            member:
                "MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice)"
                    .to_string()
        }
    );
    assert_eq!(
        images_after_each_step.len(),
        OVERWRITES + 2,
        "起点与每一步各一份镜像"
    );
    assert!(
        images_after_each_step[OVERWRITES] == images_after_each_step[OVERWRITES + 1],
        "挂载那一步前后镜像逐字节相同"
    );
    assert_eq!(
        recorded_steps_after_each_step[OVERWRITES],
        recorded_steps_after_each_step[OVERWRITES + 1],
        "挂载那一步录制流一步没多：一个写、一道屏障都没发"
    );

    let direct_stream = SharedStream::new();
    let image_before_the_mount = &images_after_each_step[OVERWRITES];
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        image_before_the_mount
            .devices
            .iter()
            .map(|(identity, image)| {
                let mut device = SparseBlockDevice::new(
                    image_before_the_mount.device_size_in_bytes,
                    PhysicalBlockSizeInBytes(512),
                );
                device.image = image.clone();
                (
                    *identity,
                    RecordingBlockDevice::with_shared_stream(
                        *identity,
                        device,
                        direct_stream.clone(),
                    ),
                )
            })
            .collect();
    let refused = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    );
    assert!(
        matches!(
            refused,
            Err(
                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
                    instance_to_acquire: InstanceGeneration(2),
                    publish_index: 2,
                    warm_up_publishes_planned: 2,
                    unit: TransactionUnit::TreeTable,
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                }
            )
        ),
        "写行那次与第一次暖机在拷贝上都取得到，第二次暖机的最后一个固定点（树表）取不到：{:?}",
        refused.as_ref().err()
    );
    assert!(
        direct_stream.operations().is_empty(),
        "在任何写之前返回：{:?}",
        direct_stream.operations()
    );
    for (identity, device) in &devices {
        assert!(
            device.inner().image == image_before_the_mount.devices[identity],
            "盘 {identity:?} 逐字节不变"
        );
    }
}

/// 取号之前在分配器的拷贝上预演那一串（`mount::dry_run_of_the_publishes_after_acquisition`，走的是发布路径落盘之前那一段）要连写行换下的
/// 落点一起释放：两块单元区 384 槽的小盘，第一个文件之后可写挂载（实例 2），再覆盖写 24 次，此时根环 24 槽正好是回退目标之后连着的
/// 23 条根加它自己；回退到环里最旧的那条根（实例 3），其余 23 条全被抛弃。写行那次的根正好盖掉回退目标那一槽，环里再没有比它旧的有效根，
/// 按可再分配谓词当场回收写行换下的那几片，暖机复用它们（真发起来改写了 8 条已回收的记录）。
/// 拷贝上不释放那几片的话，预演与真发取到的落点分叉——挂载自己的断言判出，或这一串在取号之前就被判「取不到落点」。
/// 今天回退做成、每一步之后池级 checker 判绿；拷贝上取的与真发的逐次相同由挂载自己断言。
/// 分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）每次发布多写几个节点，原先那一档（240 槽、覆盖写 21 次）在回退那一步被落点拒；
/// 这一档是在草稿副本上重扫（盘宽 240 / 256 / 384、覆盖写 18–35 次、回退目标取环里第 22 / 23 新）挑出来的：240 槽上回退一概被拒，
/// 256 槽上回退到最旧的根一概被拒；384 槽上回退到最旧的根都做成，覆盖写 24 次那一档暖机改写了 8 条已回收的记录（与原先那一档同一个数）。
/// 空间准入关掉（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）：判着准入时 384 槽的盘上挂载之后第 11 次覆盖写就被式子拒，
/// 根环凑不满这 24 条根，这一格（预演与真发在根环转满时逐次相同）就造不出来。
#[test]
fn rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, 24))
            .chain([
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                    index_from_newest: 23,
                }),
                EMPTY_CONTENT_OVERWRITE,
            ])
            .collect(),
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.outcomes[25],
        StepOutcome::Applied(AppliedEffect::Mounted {
            instance: InstanceGeneration(3),
            publishes: 2,
            allocation_records_compared: MountAllocationComparison::Compared { publishes: 2 },
            reuse: RecordReuse {
                rewritten_from_released: 8,
                rewritten_with_changed_span: 2,
                released_records_removed: 0,
                released_records_covered: 0,
            },
        }),
        "回退做成：写行与一次暖机，暖机复用写行当场回收的那几片"
    );
}

/// 抬到现行的 F（0）：只推空发布、一个落点都不回收（释放代 ≤ 0 的一条都没有）。
const RAISE_TO_THE_CURRENT_FLOOR: HistoryOperation =
    HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
        steps_above_current_floor: 0,
    });

/// 越过原分配记录墙那一条写死用例的覆盖写：内容长度固定在一个数据单元之内。
const WALL_OVERWRITE: HistoryOperation = HistoryOperation::PublishOverwrite(ContentChoice {
    length: ContentLength::InsideOneDataUnit { selector: 2999 },
    fill_seed: 1,
});

/// 越过原分配记录墙那一条写死用例的前缀（原先逼近墙那三条共用的）：从第一个文件（txg 3）起在 mkfs 同一个进程那条会话里覆盖写 20 次
/// （txg 4–23，这一段根环还没转过），再连着可写挂载 `mounts` 次。根环转过之后，按可再分配谓词回收的槽被之后的发布复用改写
/// （D3（空间分配） 已定项 7：释放只改写不删），每一步加几条随复用走。
fn wall_prefix(mounts: usize) -> impl Iterator<Item = HistoryOperation> {
    std::iter::repeat_n(WALL_OVERWRITE, 20).chain(std::iter::repeat_n(
        HistoryOperation::CloseAndMountWritable,
        mounts,
    ))
}

/// 原先分配记录墙那三格（覆盖写正好到 812 条、抬 F 的第二次空发布正好到 812 条、回退的第二次暖机正好到 812 条；增补 3 第 2 件
/// 代码三方第一轮判决第三节第 1 条、第二轮判决第三节第 4 条）拆墙之后（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1）并成这一条：
/// 同样的前缀（`wall_prefix`，挂载 9 次）之后覆盖写 46 次、抬到现行的 F、回退到最新那条根、再覆盖写 5 次，一步都不拒，
/// 镜像上按 checker 的解析数的分配记录条数越过 812；每一步之后跑池级 checker、模型逐步比（跑完即都对得上）。
/// 一段历史每一步接着上一步的盘面，次序本身就是被测对象，不切片并行。
#[test]
fn overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: wall_prefix(9)
            .chain(std::iter::repeat_n(WALL_OVERWRITE, 46))
            .chain(std::iter::once(RAISE_TO_THE_CURRENT_FLOOR))
            .chain(std::iter::once(HistoryOperation::CloseAndMountRollback(
                RollbackTargetChoice::RingRoot {
                    index_from_newest: 0,
                },
            )))
            .chain(std::iter::repeat_n(WALL_OVERWRITE, 5))
            .collect(),
    };
    let (run, counted_after_each_step) = run_counting_allocation_records_after_each_step(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    let refused: Vec<&StepOutcome> = run
        .outcomes
        .iter()
        .filter(|outcome| !matches!(outcome, StepOutcome::Applied(_)))
        .collect();
    assert_eq!(refused, Vec::<&StepOutcome>::new(), "一步都不拒");
    let most_counted = counted_after_each_step
        .iter()
        .map(|counted| counted.expect("每一步之后镜像上都数得出最新那条根下的分配记录"))
        .max()
        .expect("至少起点那一次");
    assert!(
        most_counted > 812,
        "镜像上数的分配记录条数要越过 812，最多 {most_counted} 条"
    );
    assert!(run.tally.checker_runs > 0, "每一步之后跑池级 checker");
}

/// 一段写死的历史在 4 GiB 的盘上每一步之后跑池级 checker、跑完，交回每一步之后（含起点）镜像上最新那条根下的分配记录条数。
fn run_counting_allocation_records_after_each_step(
    history: &GeneratedHistory,
) -> (HistoryRun, Vec<Option<u64>>) {
    let mut counted_after_each_step: Vec<Option<u64>> = Vec::new();
    let run = execute_history_with(
        history,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
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

/// 增补 2 收口表第 ② 行（一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红）在一次挂载里那一形，C518（一次挂载之内环转过一圈之后不回收） 修好之后跑完：
/// 第一个文件之后可写挂载一次（写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写 25 次（txg 6–30）。根环 R × S = 24 槽：txg 24、25、26
/// 依次盖掉第 0 代根与两条暖机根，txg 26 盖掉之后再没有一条有效根引用 mkfs 的第 0 版树表单元（1 槽，txg 3 释放），它在这一次发布里
/// 按谓词回收、记账按回收之后的数写——修之前它没回收、记账仍算已分配，checker 在这一步判 I-3.1 红（记账比遍历多 16384 字节）。
/// 现在每一步之后 checker 都跑、都不红：起点、挂载、25 次覆盖写之后各一次。
#[test]
fn turning_the_root_ring_with_overwrites_in_one_mount_runs_to_the_end_with_the_checker_green_after_every_step(
) {
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
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.checker_runs, 27,
        "起点、挂载、25 次覆盖写之后各跑一次 checker，一次都没红"
    );
}

/// 增补 2 收口表第 ② 行在 mkfs 同一个进程那条会话里那一形，那条会话也装上根环表（`make_filesystem::allocator_after_make_filesystem`）
/// 之后跑完：同样 25 次覆盖写（txg 4–28），一次挂载都不做。txg 26 那次覆盖写（第 22 步）盖掉 txg 2 的暖机根，环里再没有一条有效根
/// 引用 mkfs 的第 0 版树表单元（1 槽，第一个文件版本在 txg 3 换下），它在这一次发布里按谓词回收、记账按回收之后的数写——
/// 不装那张表时它不回收、记账仍算已分配，checker 在这一步判 I-3.1 红（记账比遍历多 16384 字节）。
/// 装了表，每一步之后 checker 都跑、都不红：起点与 25 次覆盖写之后各一次。
#[test]
fn turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_runs_to_the_end_with_the_checker_green_after_every_step(
) {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(overwrite, 25).collect(),
    };
    let run = execute_history(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.checker_runs, 26,
        "起点与 25 次覆盖写之后各跑一次 checker，一次都没红"
    );
    assert_eq!(
        run.tally.histories_that_turned_the_root_ring, 1,
        "最后一条根 txg 28 ≥ R × S = 24：根环转过了一圈"
    );
}

/// 「已知红」清单那一条（增补 2 收口表第 43 行）的复现，2026-09-18 在快档种子 80 上撞到、收缩出来的那一段（种子号随生成器的比重变，
/// 这一段不随）：可写挂载（实例 2，txg 4、5）、覆盖写两次（6、7）、回退到 (2, 7)（实例 3，txg 8–10）、再回退到 (2, 7)（实例 4，
/// txg 11–13：实例 3 的三条根被抛弃）、覆盖写（14）、可写挂载（实例 5，txg 15、16）、覆盖写三次（17–19）、抬 F 到 8（txg 20–22）。
/// F = 8 那个 txg 上的根被抛弃了，(2, 7) 落到 F 之下出了候选集，而它写行那次换下的实例表与固定点单元（实例表 2 槽，记账树、映射树、
/// 树表各 1 槽，加分配记录树按位置寻址那一次重写的几个节点，D8（核心索引结构） 已定项 14；合 12 槽）在 txg 11 才释放、释放代 11 > 8
/// 不回收 ⇒ 记账比遍历多 12 × 16384 字节，I-3.1 红。这一条修好之后本用例要红——那时把清单里这一条删掉、这里改成「跑完」。
#[test]
fn raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_known_red_form_of_closeout_row_43()
{
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
    assert_eq!(*form, 0, "{}", KNOWN_RED_FORMS[*form].shape);
    assert_eq!(observation.position, StepPosition::Operation(10));
    assert_eq!(
        run.outcomes[10],
        StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(8),
            publishes: 3,
            reclaimed_placements: 42,
            reuse: RecordReuse::default(),
        })
    );
    let (allocated, walked) = allocated_and_walked_bytes(&observation.violations[0].1)
        .expect("I-3.1 的违例文字带记账与遍历两个数");
    assert_eq!(
        allocated - walked,
        12 * 16384,
        "(2, 7) 的实例表 2 槽、记账树·映射树·树表各 1 槽、分配记录树那一次重写的节点，合 12 槽"
    );
}

/// 不经回退的抬 F 之后 I-3.1 记账多算是新发现，不是清单那一条（收口表第 43 行；代码三方第一轮判决第二节第 1 条）。历史照攻方的 `opus_z2_history.rs`：
/// 第一个文件之后可写挂载（实例 2，txg 4、5）、覆盖写三次（6、7、8）、抬 F（选择子 3 ⇒ F = 3），一次回退都没有。今天的代码上这段跑完，
/// 抬 F 之前的镜像上 txg 3 那条根是实例 1 的有效根 ⇒ 「F 那个 txg 上的根全属于被抛弃的实例」为假；拿它配上攻方在回收门槛差一
/// （A1）下量到的那一步违例（`盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040`），分类是新发现。同一个判定在那一条的复现
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

/// 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」的两个种子（攻方
/// `research/prompts/m2-final-code-r3-opus-model/rerun.sh` 里 `z13_one_seed` 那一格：比重 `REUSE_AFTER_RAISING_THE_FLOOR`、60 步、
/// 两块单元区 240 槽的小盘、每一步之后跑池级 checker），空间准入关掉（判着准入时这几块小盘上式子先拒，走不到抬 F 那一串撞墙）。
/// 两段历史都在一次回退之后把 F 抬进回退留下的空档：种子 4000000045 第 27 步抬到 7（回退目标 (1, 5) 在 F 之下，它那一版被
/// txg 8 换下的单元释放代 8 > 7 不回收），种子 4000000204 第 31 步抬到 20（回退目标 (3, 18) 在 F 之下，释放代 22 > 20）。
/// 这一串要推 `publishes_in_the_sequence` 次空发布（种子 4000000045 那一步两次、4000000204 那一步三次），第二次取不到落点。
/// 改之前逐次发：第一次（带新 F 的根）已经落盘、第二次才被拒，那条落了盘的新 F 让 checker 在那一步判 I-3.1 红、记账比遍历多
/// 8 × 16384（收口表第 43 行那一形，但这一步不是做成的抬 F，已知红清单不接，判成新发现，历史停在那一步）；
/// 扣住的槽也留在进程里（C546（抬 F 被拒时扣住的槽不退回） 第二次起那一半）。
/// 今天这一串在任何写之前整串预演，第二次取不到落点就一次都不发：那一步报 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`、
/// 盘上一个字节都没动（录制流不多一步，那一步不跑 checker），60 步跑完、每一步之后池级 checker 判绿、模型逐步对得上。
fn the_raise_refused_part_way_by_the_rehearsal_writes_nothing_and_the_history_runs_to_the_end(
    seed: u64,
    step_of_the_raise: usize,
    publishes_in_the_sequence: usize,
) {
    let history = generate_history_with_weights(
        HistorySeed(seed),
        60,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    );
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: HistoryDeviceWidth::UnitAreaOf240Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    assert!(
        matches!(
            history.operations[step_of_the_raise],
            HistoryOperation::RaiseRollbackFloor(_)
        ),
        "种子 {seed} 第 {step_of_the_raise} 步是抬 F"
    );
    assert_eq!(
        run.outcomes.get(step_of_the_raise),
        Some(&StepOutcome::Refused {
            member: format!(
                "MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite(publish 2 of {publishes_in_the_sequence}, PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))"
            ),
        }),
        "种子 {seed} 第 {step_of_the_raise} 步：这一串的第二次在预演里取不到落点，一次都不发（历史停在：{:?}）",
        run.ending
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "种子 {seed}：60 步跑完，每一步之后池级 checker 判绿"
    );
    assert_eq!(run.outcomes.len(), 60);
}

#[test]
fn seed_4000000045_raising_the_floor_into_a_rollback_gap_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device(
) {
    the_raise_refused_part_way_by_the_rehearsal_writes_nothing_and_the_history_runs_to_the_end(
        4_000_000_045,
        27,
        2,
    );
}

#[test]
fn seed_4000000204_raising_the_floor_into_a_rollback_gap_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device(
) {
    the_raise_refused_part_way_by_the_rehearsal_writes_nothing_and_the_history_runs_to_the_end(
        4_000_000_204,
        31,
        3,
    );
}

/// 大档与收缩用的跑法：checker（`SINGLEFS_RANDOM_HISTORY_CHECKER`：`run` / `skip`，默认 `run`）与盘宽
/// （`SINGLEFS_RANDOM_HISTORY_DEVICES`：`4gib` / `small`，默认 `4gib`）从环境变量取——门禁里那几段的跑法都能在大档上换种子重跑。
fn execution_from_the_environment() -> HistoryExecution {
    let per_step_checker = match std::env::var("SINGLEFS_RANDOM_HISTORY_CHECKER").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("run") => PerStepChecker::Run,
        Ok("skip") => PerStepChecker::Skipped,
        Ok(other) => panic!("SINGLEFS_RANDOM_HISTORY_CHECKER={other}：只认 run 与 skip"),
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
        space_admission: SpaceAdmission::JudgedByTheFormula,
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
