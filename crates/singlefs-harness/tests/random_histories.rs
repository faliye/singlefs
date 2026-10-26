//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 1 件：随机历史。生成器、执行器、「已知红」清单与收缩在 `singlefs_harness::history`；
//! 这里是快档（普通 `cargo test`）、大档（`#[ignore]`，种子数与步数从环境变量取）与清单每一条的复现。
//!
//! 五段取样的种子基都是 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`：永久固定、不重抽
//! （为什么见它的文档注释），崩溃注入那个二进制用的是同一个；种子数照旧写死，里程碑的验收说的是「写死的种子数之内判红」。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/random_histories.rs"]
mod scenarios_of_random_histories;

use common::disk_snapshot;
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_random_histories::pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version;
use singlefs_checker::image::{chosen_system_configurations, valid_roots};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, roll_back_by_a_forward_publish,
    RollbackCandidateExclusion, RollbackError, RollbackTarget,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault, SharedFaultPlan,
};
use singlefs_harness::heavy_threads::{
    own_variable_or_default_heavy_threads, registered_heavy_threads,
    THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    allocation_records_on_the_image_under, classify_failure, environment_assignments_in_line,
    execute_history_handing_out_every_step_image, execute_history_with,
    execute_history_with_faults, generate_history, generate_history_with_weights,
    raised_floor_lands_only_on_abandoned_roots, random_history_to_reproduce_from_environment,
    run_history_campaign, shrink_to_reproduction, AppliedEffect, ColdStartReadBack, ContentChoice,
    ContentLength, FailureObservation, FailureSignature, FindingShrinking, FloorTargetChoice,
    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed,
    HistoryStartingPoint, HistoryTally, MountAllocationComparison, MountSpaceAdmissionOutcome,
    NewFindingReport, PerStepChecker, RecordReuse, RollbackTargetChoice, StepOutcome, StepPosition,
    RANDOM_HISTORY_VARIABLE_PREFIX,
};
use singlefs_harness::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
use singlefs_harness::model::{
    ModelCheckpointTxg, ModelInstanceGeneration, ModelRefusalReason, ModelRootKey,
    ObservedRefusalReason,
};
use singlefs_harness::model_comparison::refusal_reason_of_rollback_error;
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::SharedStream;
use std::io::Write as _;

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

const FAST_TIER_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::BROAD,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

const REUSE_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: REUSE_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

const ROLLBACK_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

const WALL_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: WALL_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

/// 单元区墙那一格空间准入关掉的那一段：两块单元区 256 槽的小盘。
const UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED: HistorySampling = HistorySampling {
    operations_per_history: UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
    execution: HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
        space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
    },
};

/// 单元区墙那一格空间准入判着的那一段（产品路径）：同样两块单元区 256 槽的小盘。
const UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED: HistorySampling = HistorySampling {
    operations_per_history: UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
    execution: HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    },
};

/// 大档没设的那几项：每段 60 步、各类操作都抽、每一步之后跑 checker、两块 4 GiB 的盘、判空间准入。
const LARGE_TIER_DEFAULT_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: 60,
    weights: GenerationWeights::BROAD,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

/// 五段的种子基都是这个测试周期的常量：两个二进制（随机历史与崩溃注入）同基，
/// `crates/mutations.tsv` 里名字以「增补 3 第 2 件」起头的那些「这条变异必须红」与里程碑的验收才是可判的命题——
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

fn count_of(counts: &std::collections::BTreeMap<String, u64>, key: &str) -> u64 {
    counts.get(key).copied().unwrap_or(0)
}

/// 各条路径都真的跑到了：崩溃恢复抛弃根之外的每类操作至少一次 Ok、崩溃恢复抛弃根至少被乙拒过一次，回退的目标候选集里外都试过，
/// 抬 F 撞过上限也回收过落点，复用改写过已释放的记录（含跨度变了的与被删的），六种内容长度都进过入口，冷启动读回过文件，
/// checker 真的判过 I-3.1 与 I-5.4，至少一段历史转过根环。
fn assert_every_path_was_exercised(tally: &HistoryTally) {
    for kind in HistoryOperationKind::ALL {
        let operation = tally
            .operations_by_kind
            .get(&kind)
            .copied()
            .unwrap_or_default();
        match kind {
            // 那一步藏的是这个会话发的、系统配置见证过的最新那条根：C554 乙之后读阶段重读一次仍读不出、取号之前拒可写
            // （用户 2026-09-27 定），这一步跑到了是「拒过」，不再有做成的。
            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {
                assert!(operation.refused >= 1, "{kind:?} 一次都没被拒过");
            }
            // 快档那组比重不抽它（种子钉着的取样点照旧）：它只在留孤儿那一组比重里抽，跑到与留下孤儿由 `singlefs_harness::history`
            // 的单元测试钉（`the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does` 与
            // `an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring`）。
            HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
                assert_eq!(
                    operation,
                    singlefs_harness::history::OperationTally::default(),
                    "{kind:?} 不在快档那组比重里，却跑到了"
                );
            }
            // 只进它自己那一组比重（`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`，C696），快档那一组不抽它；
            // 跑到了没有在那一组的取样点那条用例里核（`newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms`）。
            HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "快档那一组比重不抽 {kind:?}"
                );
            }
            // 建快照、读快照、删快照只进它们自己那一组比重（`GenerationWeights::SNAPSHOT_OPERATIONS`），快档那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`snapshot_operations_in_random_histories.rs`）。
            HistoryOperationKind::CreateSnapshot
            | HistoryOperationKind::ReadSnapshot
            | HistoryOperationKind::DestroySnapshot => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "快档那一组比重不抽 {kind:?}"
                );
            }
            // 挂着时整理只进它自己那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED`），快档那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`compaction_while_mounted_in_random_histories.rs`）。
            HistoryOperationKind::CompactWhileMounted => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "快档那一组比重不抽 {kind:?}"
                );
            }
            // 截断只进它自己那一组比重（`GenerationWeights::TRUNCATION`）与带整理的那一组，快档那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`truncation_random_histories_against_the_model.rs`）。
            HistoryOperationKind::Truncate => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "快档那一组比重不抽 {kind:?}"
                );
            }
            // 一次建几个 inode、按 inode 号写只进它们自己那一组比重（`GenerationWeights::WRITES_BY_INODE_NUMBER`），快档那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`writes_by_inode_number_in_random_histories_judged_by_the_model.rs`）。
            HistoryOperationKind::CreateInodes | HistoryOperationKind::WriteOfAnInode => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "快档那一组比重不抽 {kind:?}"
                );
            }
            // 按偏移写只进它自己那一组比重（`GenerationWeights::PARTIAL_WRITES`），快档那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`partial_write_against_the_model.rs`）。
            HistoryOperationKind::WriteAtOffset => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "快档那一组比重不抽 {kind:?}"
                );
            }
            HistoryOperationKind::PublishFirstFile
            | HistoryOperationKind::PublishOverwrite
            | HistoryOperationKind::PublishWithoutUnits
            | HistoryOperationKind::CloseAndMountWritable
            | HistoryOperationKind::RollBackWhileMounted
            | HistoryOperationKind::RaiseRollbackFloor
            | HistoryOperationKind::ColdStartRecover
            | HistoryOperationKind::Unlink
            | HistoryOperationKind::CreateInode => {
                assert!(operation.applied >= 1, "{kind:?} 一次 Ok 都没有");
            }
        }
    }
    let refusals = &tally.refusals_by_member;
    for member in [
        "MountError::RollbackFloorAboveCeiling",
        "RollbackError::TargetNotACandidate(NotInRing)",
        "PublishError::ContentExceedsDataUnit",
        "PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile",
        "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)",
    ] {
        assert!(count_of(refusals, member) >= 1, "没见过 {member}");
    }
    assert!(
        count_of(
            refusals,
            "RollbackError::TargetNotACandidate(BelowEffectiveFloor)"
        ) >= 1,
        "回退的目标没落到过 F 之下的根"
    );
    // 回退的目标落到被抛弃的根上、按被抛弃的时间线拒，这一路随机历史不再要求跑到：被抛弃的根只由崩溃恢复造出
    // （D23（journal 的角色与格式） 已定项 14），生成器那一步 C554 乙之后一律被拒、造不出它；乙罩不到的「系统配置没见证到最新那条根」
    // 那一形由写死的用例 `rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline` 钉。
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
    // 池级 checker 那一遍归 checker 档两段（用户 2026-10-13 定）：harness 一条不变量都不判，每一步的镜像经钩子交出去。
    // I-3.1、I-5.4、I-3.11、I-7.9、I-9.15 在同一批历史上真被判成立过，由 checker 档那一份用例核
    // （`crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`）。
    assert_the_harness_ran_no_pool_checker_and_handed_every_step_image_out(tally);
    assert!(
        tally.histories_that_turned_the_root_ring >= 1,
        "没有一段历史转过根环"
    );
    assert_the_model_judged_every_kind_of_answer(tally);
}

/// harness 档不逐步跑池级 checker（那一遍归 checker 档两段），每一步的镜像都交出去了：checker 一次都没跑，交出去的镜像至少一张。
fn assert_the_harness_ran_no_pool_checker_and_handed_every_step_image_out(tally: &HistoryTally) {
    assert_eq!(
        tally.checker_runs, 0,
        "harness 档跑了池级 checker：每一步的池级 checker 归 checker 档两段，harness 只与理想模型比"
    );
    assert!(
        tally.step_images_handed_out >= tally.histories,
        "每段历史起点之后至少交出一张镜像：交了 {} 张、{} 段",
        tally.step_images_handed_out,
        tally.histories
    );
}

/// 每步镜像那个钩子（checker 档两段的录入段照它把每一步的镜像作为给定状态写进判定库）：快档那组比重、种子基那一段 12 步，
/// harness 一次池级 checker 都不跑；起点之后与每一步写过盘之后各交一张，交出去的张数与计数相同、录制流里的步数逐张变多，
/// 每一张都等于空盘叠上录制流头那么多步（checker 档按录制流重建给定状态，判的就是这一张）。
/// 判别力：harness 偷回逐步池级 checker（执行器里每张交出去的镜像又就地判一遍、记进计数），checker 跑过的次数不再是 0；
/// 钩子收到了却不交出去，交出去的张数与计数对不上。
// harness-test-granularity:one-scenario 一段历史跑一遍，循环里逐张重建的是同一条录制流的前缀、拿来比这一段交出去的那几张，不是几个独立场景
#[test]
fn every_step_image_is_handed_out_as_the_recorded_stream_prefix_and_the_harness_runs_no_pool_checker(
) {
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        12,
        &FAST_TIER_SAMPLING.weights,
    );
    let stream = SharedStream::retaining_contents();
    let mut handed_out: Vec<(usize, MemoryPool)> = Vec::new();
    let run = execute_history_handing_out_every_step_image(
        &history,
        execution,
        &stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |step_image| {
            handed_out.push((step_image.operations_recorded, step_image.image.clone()));
        },
    );
    assert_eq!(run.tally.checker_runs, 0, "harness 档不逐步跑池级 checker");
    assert_eq!(
        run.tally.step_images_handed_out,
        u64::try_from(handed_out.len()).expect("装得进 u64"),
        "计数与交出去的张数相同"
    );
    assert!(
        handed_out.len() >= 2,
        "起点之后与至少一步写过盘之后都交了：{}",
        handed_out.len()
    );
    assert!(
        handed_out.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "每一张都是写过盘之后交的，录制流里的步数逐张变多"
    );
    let operations = stream.retained_operations();
    for (index, (operations_recorded, image)) in handed_out.iter().enumerate() {
        let mut rebuilt = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        rebuilt.apply(&operations[..*operations_recorded]);
        assert!(
            rebuilt == *image,
            "交出去的第 {index} 张：空盘叠上录制流头 {operations_recorded} 步与交出去的镜像不同"
        );
    }
}

/// 一段历史在两块 4 GiB 的盘上跑，harness 不跑每一步的池级 checker（每一步的镜像经钩子交出去，那一遍归 checker 档两段）。
fn execute_history_without_the_pool_checker(history: &GeneratedHistory) -> HistoryRun {
    execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    )
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        FAST_TIER_FIRST_SEED,
        FAST_TIER_SEEDS,
        FAST_TIER_SAMPLING.operations_per_history,
        &FAST_TIER_SAMPLING.weights,
        FAST_TIER_SAMPLING.execution,
        worker_threads_from_environment(),
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
/// 这几个数不看罩住的那条删没删，变异下照样成立，门禁 checker-tier-crates-mutation-replay 看到的红只会是分类判出来的。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        REUSE_SAMPLING_FIRST_SEED,
        REUSE_SAMPLING_SEEDS,
        REUSE_SAMPLING.operations_per_history,
        &REUSE_SAMPLING.weights,
        REUSE_SAMPLING.execution,
        worker_threads_from_environment(),
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        ROLLBACK_SAMPLING_FIRST_SEED,
        ROLLBACK_SAMPLING_SEEDS,
        ROLLBACK_SAMPLING.operations_per_history,
        &ROLLBACK_SAMPLING.weights,
        ROLLBACK_SAMPLING.execution,
        worker_threads_from_environment(),
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

/// 崩溃注入读故障那一支的取样点（C696（崩溃注入的读故障一支在随机历史里从没触发），用户定设计调查的候选「丁」）：比重取
/// `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`（快档那一组、崩溃恢复抛弃根那几格换成「每块盘最新那一槽
/// 系统配置读回全 0 时的可写挂载」），种子从这个测试周期的种子基起 48 段、每段 24 步，两块 384 槽单元区的窄盘（设计调查量的那一档），
/// 每一步之后跑池级 checker、模型照判。先判清单外的失败（checker 判红、模型对不上都在其内），再核这一步真的跑到了、可写挂载做成过——
/// 做成就是取号、写行、暖机都在读故障下发了写（D22（单元原子性怎么合成） 已定项 8：单故障在承诺里，挂载照常往下走）。
/// 判别力：这一步藏成每块盘两槽都读回全 0（`crates/mutations.tsv` 一行：那块盘一份自证过的系统配置都不剩，双故障）时可写挂载被拒、
/// 一次都做不成，这一条红。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        48,
        24,
        &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：每块盘最新那一槽系统配置读回全 0 的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let step = report
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros)
        .copied()
        .unwrap_or_default();
    assert!(
        step.applied >= 1,
        "每块盘最新那一槽系统配置读回全 0 的可写挂载一次都没做成：{step:?}\n{rendered}"
    );
    assert_eq!(
        step.refused, 0,
        "单故障在承诺里，这一步不许被拒：{step:?}\n{rendered}"
    );
}

/// 原分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：比重取 `GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`，
/// 种子与步数见常量。分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1）没有「一个节点 812 条」那道墙，
/// 模型也没有那条拒绝理由：越过 812 条之后实现因为分配记录拒一次，模型就判对不上。每一步之后照跑池级 checker、判红就停（第二轮判决第三节
/// 第 3 条：攻方两条只有 checker 看得见的变异——根环转过之后回收门槛多一代、分配记录过 600 条之后「已分配」少记一槽——要 checker 才看得见）。
/// 先判没有新发现（checker 的判红、模型对不上、执行器判出的、panic 都算），再核这一路真的跑到了：checker 真的跑过、有一段历史的某一版
/// 分配记录多于 812 条（多叶的树真的写过、读过、判过）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        WALL_SAMPLING_FIRST_SEED,
        WALL_SAMPLING_SEEDS,
        WALL_SAMPLING.operations_per_history,
        &WALL_SAMPLING.weights,
        WALL_SAMPLING.execution,
        worker_threads_from_environment(),
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
        report.tally.step_images_handed_out > 0,
        "这一段每一步之后跑池级 checker"
    );
    assert!(
        report.tally.most_allocation_records_in_one_version > 812,
        "没有一段历史越过 812 条分配记录（原先那道墙那一格没跑到）：最多 {} 条",
        report.tally.most_allocation_records_in_one_version
    );
}

/// 单元区墙那一格空间准入关掉的取样点（两块单元区 256 槽的小盘，`HistoryDeviceWidth::UnitAreaOf256Slots`；比重
/// `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`，每一步之后照跑池级 checker、判红就停，种子与步数见常量；
/// 空间准入关掉，只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）跑到底、一次都没撞上落点墙：
/// 小盘为环窗口补足之后（用户 2026-10-02 定「甲」）这一段的单元区比环窗口那一截宽，覆盖写与第一个文件一次都没被「每块盘上都没有」拒过，
/// 模型也一次都没在单元区墙的区间里放行过。用户 2026-10-03 定案（Q-T「乙 照现值改钉并改名」）：种子与几何都不动，
/// 钉今天量到的那几个数——32 段全跑完、没有新发现，「每块盘上都没有」0 次，单元区墙区间里放行 0 次，池级 checker 跑 1141 次；
/// 落点墙那一格只靠写死的那 5 条用例罩着（`a_floor_raise_refused_for_space_counts_as_short_of_space.rs` 两条与
/// `admission_raises_the_floor_before_refusing.rs` 里删除类发布撞落得下那一判的三条）。
/// 判别力：这一段生成的历史或落点规则变了（比重里 unlink 那一格改成 0），这几个数跟着变。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED.operations_per_history,
        &UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED.weights,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED.execution,
        worker_threads_from_environment(),
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
    // 第一个文件直接调发布路径；覆盖写经挂着的会话（实七）：落点拒绝是「空间不够」的一种，会话先推抬 F、推满仍拒才交回，
    // 成员包在 `UserChangeRefused::NoSpaceAfterRaisingTheFloor` 里。
    let refused_on_every_device = count_of(
        &report.tally.refusals_by_member,
        "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)",
    ) + count_of(
        &report.tally.refusals_by_member,
        "UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))",
    );
    assert_eq!(
        (
            report.tally.histories_completed,
            refused_on_every_device,
            report
                .tally
                .model_counts
                .unit_area_wall_refusals_in_the_interval,
            report.tally.step_images_handed_out,
        ),
        (32, 0, 0, 1141),
        "32 段全跑完、「每块盘上都没有」0 次、单元区墙区间里放行 0 次、池级 checker 跑 1141 次（2026-10-03 量）：\n{rendered}"
    );
}

/// 同一段取样（两块单元区 256 槽的小盘、逼近单元区墙的比重、同一批种子与步数），空间准入判着（产品路径）：
/// D28（挂载期承诺量） 已定项 1 的式子接进发布与可写挂载之后（C363 (b) 判决第四节第 2 条），这几块小盘上墙由式子先拒。
/// 覆盖写经挂着的会话发布之后（实七），式子一拒会话先推一串抬 F 再重判（D16（发布语义） 已定项 1「准入」那一行，C283）：
/// 这一批种子上推过之后每一次都发成了（2026-09-26 量：推了 470 串、发成 470 次，最后被拒的 0 次），模型逐串比过推的每一串抬 F。
/// 推满仍不够时交回的 `UserChangeRefused::NoSpaceAfterRaisingTheFloor` 在这里一次都没出现，式子那一拒模型区间里放行的那一格
/// 在这批种子上看不到了（实七报告）。可写挂载准入不够时写行与暖机之后推抬 F 的空发布、再判，执行器把推的每一串照模型的抬 F 逐串比。
/// 先判没有新发现，再核这一路真的跑到了：覆盖写经会话推过抬 F 再发成、可写挂载写行之后推过抬 F、推的每一串模型都比过上限；
/// 落点那一道一次都没走到（式子先拒）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.operations_per_history,
        &UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.weights,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.execution,
        worker_threads_from_environment(),
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
        report
            .tally
            .user_changes_published_after_the_session_raised_the_floor
            >= 1,
        "覆盖写一次都没经会话推过抬 F 再发成（发布路径那一处的推没跑到）"
    );
    assert!(
        report.tally.model_counts.ceilings_compared
            >= report
                .tally
                .floor_raises_pushed_by_the_session_before_a_published_change
                + report.tally.floor_raises_pushed_by_mounts,
        "会话与挂载推的每一串抬 F 模型都要比过上限"
    );
    assert!(
        report
            .tally
            .mounts_by_space_admission
            .get(MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises { floor_raises: 0 }.name())
            .copied()
            .unwrap_or(0)
            >= 1
            && report.tally.floor_raises_pushed_by_mounts >= 1,
        "可写挂载一次都没在写行之后推过抬 F 再判够（可写挂载那一处没跑到）"
    );
    assert_eq!(
        count_of(
            &report.tally.refusals_by_member,
            "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)"
        ) + count_of(
            &report.tally.refusals_by_member,
            "UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))"
        ),
        0,
        "准入判着时式子先拒，落点那一道走不到"
    );
    assert!(
        report.tally.step_images_handed_out > 0,
        "这一段每一步之后跑池级 checker"
    );
}

/// 单元区墙取样点（空间准入判着）那一批的第一个种子只跑 77 步：第 76 步那次可写挂载接在 unlink 删完的一版后面
/// （有树而文件集合为空，D8（核心索引结构） 已定项 16 第 5 条），写行与两次暖机写在这一形上、之后推一串三次空发布的抬 F。
/// 执行器逐次比挂载写出的分配记录时，写行与暖机按「有树」挑进比较链：六次发布逐次比过、改写或新增的记录的代都等于各自那次的 txg，
/// 历史跑完、没有新发现。
/// 判别力：写行与暖机按「带文件」挑（`PoolVersion::file_version`）时这一形的三次被跳过，比较的上一版停在挂载之前那棵树上，
/// 推的第一次空发布把那三次改写的记录全当成自己改的、代却不是自己的 txg，执行器判「改写或新增的分配记录的代不是这次发布的 txg」。
#[test]
fn a_mount_on_the_emptied_version_in_the_unit_area_wall_sampling_compares_its_row_and_warm_up_publishes(
) {
    const OPERATIONS: usize = 77;
    const STEP_OF_THE_MOUNT: usize = 76;
    let history = generate_history_with_weights(
        HistorySeed(UNIT_AREA_WALL_SAMPLING_FIRST_SEED),
        OPERATIONS,
        &UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.weights,
    );
    let run = execute_history_with(
        &history,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.execution,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "{OPERATIONS} 步跑完、没有新发现"
    );
    assert_eq!(
        history.operations[STEP_OF_THE_MOUNT],
        HistoryOperation::CloseAndMountWritable,
        "第 {STEP_OF_THE_MOUNT} 步是可写挂载"
    );
    let StepOutcome::Applied(AppliedEffect::Mounted {
        allocation_records_compared,
        ..
    }) = &run.outcomes[STEP_OF_THE_MOUNT]
    else {
        panic!(
            "第 {STEP_OF_THE_MOUNT} 步挂载做成：{:?}",
            run.outcomes[STEP_OF_THE_MOUNT]
        );
    };
    assert_eq!(
        *allocation_records_compared,
        MountAllocationComparison::Compared { publishes: 6 },
        "写行、两次暖机（都在删完的一版上）与推的三次空发布逐次比过"
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

/// 执行器每一步量模型写进根环几条根（`HistoryTally::most_roots_written_by_the_model_in_one_step`，断言不多于根环槽数；
/// `IdealModel::committed_versions` 那一段注释）：从 mkfs 起可写挂载那一步写行与暖机两条（txg 1、2），第一个文件那一步一条（txg 3），
/// 起点（只做过 mkfs）一条都不写，最多的那一步是两条。判别力：模型写根不计数时这里是 0；断言的界压到 1 时挂载那一步就停在断言上、历史跑不完。
#[test]
fn the_executor_measures_the_roots_the_model_writes_in_each_step_and_the_mount_writes_two() {
    let history = GeneratedHistory {
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
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_observation| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.most_roots_written_by_the_model_in_one_step, 2,
        "可写挂载那一步写行与暖机两条根，是三步里最多的"
    );
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
        let run = execute_history_with(
            history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |observation| {
                counted.push(allocation_records_on_the_image_under(
                    observation.image,
                    newest_root_on_the_image(observation.image),
                ));
            },
        );
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

/// 两块单元区 240 槽的小盘（准入关着，只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`，执行器与直接调挂载两处都关），
/// 从第一个文件起在 mkfs 那条会话里覆盖写 18 次空内容（txg 4–21），再可写挂载：挂载做成（实例 2），写行与一次暖机两次发布、
/// 执行器逐次比过分配记录、一条都没复用；同一份盘面（第 18 次覆盖写之后的镜像）另起两块内存盘直接调 `mount_writable`，
/// 同样做成、取号 2、暖机一次。这段历史原是增补 2 收口表第 39 行那一族（取号之前的准入不算落点）钉的那一形：240 槽的盘上挂载自己那一串
/// 在取号之前就取不到落点；小盘为环窗口补足之后（用户 2026-10-02 定「甲」）单元区宽了，这里走不到那道墙了。用户 2026-10-03 定案
/// （Q-T「乙 照现值改钉并改名」）：覆盖写次数与盘宽不动，钉今天的结局；落点墙那一格只靠写死的那 5 条用例罩着。
/// 判别力：暖机计划多推一次时挂载那一步报三次发布、直接挂载暖机两次。
#[test]
fn eighteen_empty_overwrites_on_240_slot_devices_with_the_admission_skipped_leave_room_for_a_writable_mount_with_one_warm_up(
) {
    const OVERWRITES: usize = 18;
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, OVERWRITES)
            .chain(std::iter::once(HistoryOperation::CloseAndMountWritable))
            .collect(),
    };
    let device_width = HistoryDeviceWidth::UnitAreaOf240Slots;
    let mut images_after_each_step: Vec<MemoryPool> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |observation| {
            images_after_each_step.push(observation.image.clone());
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
        StepOutcome::Applied(AppliedEffect::Mounted {
            instance: InstanceGeneration(2),
            publishes: 2,
            allocation_records_compared: MountAllocationComparison::Compared { publishes: 2 },
            reuse: RecordReuse::default(),
            space_admission: MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch,
        }),
        "挂载做成：取号 2，写行与一次暖机逐次比过分配记录、一条都没复用"
    );
    assert_eq!(
        images_after_each_step.len(),
        OVERWRITES + 2,
        "起点与每一步各一份镜像"
    );
    let image_before_the_mount = &images_after_each_step[OVERWRITES];
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image_before_the_mount
        .devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(
                image_before_the_mount.device_size_in_bytes,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = image.clone();
            (*identity, device)
        })
        .collect();
    let mounted = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .unwrap_or_else(|error| panic!("同一份盘面直接挂载做成：{error:?}"));
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.warm_up_publishes.len()
        ),
        (InstanceGeneration(2), 1),
        "直接挂载取号 2、暖机一次"
    );
}

/// 挂着的时候回退到环里最旧的那条根，在单元区 384 槽的小盘上做成、之后照常覆盖写（C558（回退目标是环里最旧的根时回退那次发布会写坏它）
/// 那一格在随机历史执行器上的对照：回退那次发布的根槽就是回退目标住的那一槽）：第一个文件之后可写挂载（实例 2），覆盖写 24 次，
/// 此时根环 24 槽里最旧的那条是挂载之后第二次暖机的根；回退到它（一次向前发布）、再覆盖写一次，每一步之后池级 checker 判绿、模型逐步比。
/// 空间准入关掉（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）：判着准入时 384 槽的盘上挂载之后第 11 次覆盖写就被式子拒，
/// 根环凑不满。旧形态（挂载时回退）下这一格护的是「取号之前的预演连写行换下的落点一起释放」；向前回退不取号、不预演，那一半不再有对象。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, 24))
            .chain([
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                    index_from_newest: 23,
                }),
                EMPTY_CONTENT_OVERWRITE,
            ])
            .collect(),
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[25],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到环里最旧的根做成：{:?}",
        run.outcomes[25]
    );
    assert!(
        matches!(run.outcomes[26], StepOutcome::Applied(_)),
        "回退之后覆盖写做成：{:?}",
        run.outcomes[26]
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
/// 同样的前缀（`wall_prefix`，挂载 9 次）之后覆盖写 46 次、抬到现行的 F、挂着的时候回退到最新那条根（一次向前发布；旧形态是
/// 挂载时回退，它的第二次暖机正好到 812 条）、再覆盖写 5 次，一步都不拒，
/// 镜像上按 checker 的解析数的分配记录条数越过 812；每一步之后跑池级 checker、模型逐步比（跑完即都对得上）。
/// 一段历史每一步接着上一步的盘面，次序本身就是被测对象，不切片并行。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: wall_prefix(9)
            .chain(std::iter::repeat_n(WALL_OVERWRITE, 46))
            .chain(std::iter::once(RAISE_TO_THE_CURRENT_FLOOR))
            .chain(std::iter::once(HistoryOperation::RollBackWhileMounted(
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
    assert!(
        run.tally.step_images_handed_out > 0,
        "每一步之后跑池级 checker"
    );
}

/// 一段写死的历史在 4 GiB 的盘上每一步之后跑池级 checker、跑完，交回每一步之后（含起点）镜像上最新那条根下的分配记录条数。
fn run_counting_allocation_records_after_each_step(
    history: &GeneratedHistory,
) -> (HistoryRun, Vec<Option<u64>>) {
    let mut counted_after_each_step: Vec<Option<u64>> = Vec::new();
    let run = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
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

/// 回退到 txg 低于 F_生效 的根要拒，理由是「低于 F」（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）；它是候选排除，不是
/// 「树表 0 条、第一版不支持」（增补 3 第 2 件代码三方第二轮判决第三节第 1 条：两者互报的变异里，「低于 F」报成「树表 0 条」那一条在门禁
/// 五段里各只红 1 段，这一条把它钉死）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6，
/// 推 txg 10、11）、挂着的时候回退到根环从新到旧第 6 条 (5, 2)：它带文件、没有被抛弃，只因 5 < 6 被挡。今天的代码上这段跑完、
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
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 6,
            }),
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
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
            member: "RollbackError::TargetNotACandidate(BelowEffectiveFloor)".to_string()
        }
    );
    assert_eq!(
        run.tally.model_counts.required_refusals_matched, 1,
        "模型要求拒、理由对上"
    );
}

/// 回退到 txg = F_生效 的根要做成（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效），模型按 B2 那一格判（攻方变异「回退到
/// txg = F_生效 的根也拒」）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6：上限是
/// 第 4 新的非空根 6 与盘 1 上最新的有效根 7 取小，推 txg 10、11 两次）、挂着的时候回退到候选集的下沿 (6, 2)（一次向前发布 txg 12）、
/// 冷启动读回。今天的代码上这段跑完：回退做成一次、落在 F 上，冷启动读回的是 txg 6 那次覆盖写的内容（模型比过）。取样点里这一格 48 个种子一窗只有
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
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
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
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
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
/// 现在这段历史跑完，起点、挂载、25 次覆盖写之后各交出一张镜像（这个二进制不调池级 checker；同一段历史带池级 checker 判、
/// 挂载内回收那一槽由 `root_ring_turn_in_one_mount.rs` 的 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it`
/// 与 checker 档同名用例判）。
#[test]
fn turning_the_root_ring_with_overwrites_in_one_mount_runs_to_the_end_handing_out_an_image_after_every_step(
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
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.step_images_handed_out, 27,
        "起点、挂载、25 次覆盖写之后各交出一张镜像"
    );
}

/// 增补 2 收口表第 ② 行在 mkfs 同一个进程那条会话里那一形，那条会话也装上根环表（`make_filesystem::allocator_after_make_filesystem`）
/// 之后跑完：同样 25 次覆盖写（txg 4–28），一次挂载都不做。txg 26 那次覆盖写（第 22 步）盖掉 txg 2 的暖机根，环里再没有一条有效根
/// 引用 mkfs 的第 0 版树表单元（1 槽，第一个文件版本在 txg 3 换下），它在这一次发布里按谓词回收、记账按回收之后的数写——
/// 不装那张表时它不回收、记账仍算已分配，checker 在这一步判 I-3.1 红（记账比遍历多 16384 字节）。
/// 装了表，这段历史跑完、根环转过一圈，起点与 25 次覆盖写之后各交出一张镜像（这个二进制不调池级 checker；mkfs 那条会话里覆盖写
/// 转过根环、按谓词回收、池级 checker 不红，由 `root_ring_turn_in_the_make_filesystem_session.rs` 的
/// `overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant` 与 checker 档同名用例判，
/// 覆盖写到 txg 30）。
#[test]
fn turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_runs_to_the_end_handing_out_an_image_after_every_step(
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
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.tally.step_images_handed_out, 26,
        "起点与 25 次覆盖写之后各交出一张镜像"
    );
    assert_eq!(
        run.tally.histories_that_turned_the_root_ring, 1,
        "最后一条根 txg 28 ≥ R × S = 24：根环转过了一圈"
    );
}

/// 「已知红」清单那一条（增补 2 收口表第 43 行）原先的复现（2026-09-18 快档种子 80 收缩出来的那一段）换成向前回退之后跑完：
/// 可写挂载（实例 2，txg 4、5）、覆盖写两次（6、7）、挂着的时候回退到最新那条根 (2, 7)（txg 8）、再回退到从新到旧第 4 条 (2, 5)（txg 9）、
/// 覆盖写（10）、可写挂载（实例 3，txg 11、12）、覆盖写三次（13–15）、抬 F（选择子 8）。旧形态下两次回退各开一个实例、抛弃前一段，
/// F = 8 落进回退留下的空档，I-3.1 记账多算 12 槽；向前回退不抛弃任何根，这一段没有空档，每一步之后 checker 判绿、跑完。
/// 清单那一条的形态今天只剩崩溃恢复抛弃的时间线造得出（实四乙第 7 件）；C554 乙之后只剩系统配置没见证到被抛弃那条根的那一形，
/// 那一形的时间线由 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`
/// 造（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：可写挂载时的预隔离护住被抛弃那条根的单元，抬 F 之前池级 checker 不红。
#[test]
fn the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback() {
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
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            (&run.outcomes[3], &run.outcomes[4]),
            (
                StepOutcome::Applied(AppliedEffect::RolledBack { .. }),
                StepOutcome::Applied(AppliedEffect::RolledBack { .. })
            )
        ),
        "两次回退都做成：{:?}",
        &run.outcomes[3..5]
    );
    assert!(
        matches!(
            run.outcomes[10],
            StepOutcome::Applied(AppliedEffect::RaisedFloor { .. })
        ),
        "抬 F 做成：{:?}",
        run.outcomes[10]
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_random_histories::crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
) {
    scenarios_of_random_histories::crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 崩溃恢复抛弃的那条根不在回退的候选集里：照 [`pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`] 造出被抛弃的 C
/// （实例 1、txg 5；系统配置没见证到它），挂在实例 2 上回退到 C：被拒成「在被抛弃的时间线上」，拒之前盘上逐字节不变
/// （两盘的系统配置槽、根环里读得出的根、录制流步数），胶水把这个成员映射成模型的「回退目标在被抛弃的时间线上」
/// （随机历史里模型对这一格要求拒的就是这一条）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline(
) {
    let (mut pool, abandoned) = pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version(
        "random-history-rollback-onto-the-abandoned",
    );
    assert_eq!(
        (abandoned.root.instance, abandoned.root.checkpoint_txg),
        (InstanceGeneration(1), CheckpointTxg(5))
    );
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = roll_back_by_a_forward_publish(
        &common::parameters(),
        pool.devices.as_mut().expect("镜像还开着"),
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: abandoned.root.instance,
            checkpoint_txg: abandoned.root.checkpoint_txg,
        },
    )
    .expect_err("C 在被抛弃的时间线上");
    assert!(
        matches!(
            refused,
            RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,
                ..
            }
        ),
        "被拒成「在被抛弃的时间线上」：{refused:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "拒之前一个字节都不写"
    );
    assert_eq!(
        refusal_reason_of_rollback_error(&refused),
        ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetOnAbandonedTimeline)
    );
}

/// 冷启动读回报的根取施加 journal 之后实际走的那条（实四丙交回 ④ offset 330，实七）：第一个文件之后覆盖写一次（txg 4），
/// 它的根槽 FUA 写被说谎的设备吞掉——journal 记录与单元都落了盘，根槽没落；接着冷启动。恢复择到 txg 3、施加 txg 4 的记录、
/// 读回 txg 4 那一版的内容，模型要的也是 txg 4。按施加之前所选的根报（`observed_read_back`），对拍报「冷启动读回」对不上。
#[test]
fn cold_start_after_a_swallowed_root_slot_write_reads_back_the_version_its_applied_record_rebuilt_and_the_model_agrees(
) {
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let parameters = execution.device_width.parameters();
    let root_slot_of_the_overwrite = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(4),
        parameters.geometry.root_ring_slots_per_region,
    );
    let plan = SharedFaultPlan::armed(
        execution.device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::OnlyDevice(
                parameters.region_devices
                    [usize::try_from(root_slot_of_the_overwrite.region).expect("区域号小于 3")],
            ),
            placement: FaultPlacement::OffsetExactly(singlefs_core::root_ring::slot_offset(
                root_slot_of_the_overwrite,
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(1),
        },
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 2999 },
                fill_seed: 7,
            }),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    let fired = plan.fired();
    assert_eq!(fired.len(), 1, "只吞掉覆盖写那一次根槽 FUA 写：{fired:?}");
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            &run.outcomes[1],
            StepOutcome::Applied(AppliedEffect::Recovered {
                read_back: ColdStartReadBack::FileRead,
                ..
            })
        ),
        "冷启动读回文件：{:?}",
        run.outcomes[1]
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
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            if observation.position == StepPosition::Operation(3) {
                image_before_raising = Some(observation.image.clone());
            }
        },
    );
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
    let run = execute_history_without_the_pool_checker(&history);
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice() {
    let history = generate_history(HistorySeed(7), 24);
    let run_keeping_every_image = || {
        let mut images: Vec<MemoryPool> = Vec::new();
        let run = execute_history_with(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |observation| {
                images.push(observation.image.clone());
            },
        );
        (run, images)
    };
    let (first, first_images) = run_keeping_every_image();
    let (second, second_images) = run_keeping_every_image();
    assert_eq!(first, second);
    assert!(
        first.tally.step_images_handed_out >= 2,
        "起点之后与至少一步写之后都跑了 checker：{}",
        first.tally.step_images_handed_out
    );
    assert!(
        first_images.len() >= 2,
        "观察者至少见到起点与一步之后的镜像：{}",
        first_images.len()
    );
    assert!(first_images == second_images, "每一步之后的镜像逐字节相同");
}

/// 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」那两段历史（攻方
/// `research/prompts/m2-final-code-r3-opus-model/rerun.sh` 里 `z13_one_seed` 那一格：比重 `REUSE_AFTER_RAISING_THE_FLOOR`、60 步、
/// 两块单元区 240 槽的小盘、每一步之后跑池级 checker），空间准入关掉：在 `step_of_the_raise` 那一步抬 F，结局是 `raise_outcome`，
/// 60 步跑完、每一步之后池级 checker 判绿、模型逐步对得上。那两段原先钉的是「抬 F 那一串的第二次在预演里取不到落点、一次都不发」
/// （C546 第二次起那一半）；小盘为环窗口补足之后（用户 2026-10-02 定「甲」）单元区宽了，那一步的抬 F 做成了。用户 2026-10-03 定案
/// （Q-T「乙 照现值改钉并改名」）：种子与几何都不动，钉今天的结局；预演里取不到落点那一形只靠写死的那 5 条用例罩着。
fn the_floor_raise_on_narrow_devices_with_the_admission_skipped_has_its_outcome_and_the_history_runs_to_the_end(
    seed: u64,
    step_of_the_raise: usize,
    raise_outcome: &StepOutcome,
) {
    let history = generate_history_with_weights(
        HistorySeed(seed),
        60,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    );
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
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
        Some(raise_outcome),
        "种子 {seed} 第 {step_of_the_raise} 步的抬 F（历史停在：{:?}）",
        run.ending
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "种子 {seed}：60 步跑完，每一步之后池级 checker 判绿"
    );
    assert_eq!(run.outcomes.len(), 60);
}

/// 种子 4000000045 第 27 步：F 抬到 19，三次空发布，回收 135 个落点。
/// 判别力：抬 F 生效之后不回收时回收数是 0；上限少算一档时新 F 不是 19。
#[test]
fn seed_4000000045_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_19_reclaiming_135_placements(
) {
    the_floor_raise_on_narrow_devices_with_the_admission_skipped_has_its_outcome_and_the_history_runs_to_the_end(
        4_000_000_045,
        27,
        &StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(19),
            publishes: 3,
            reclaimed_placements: 135,
            reuse: RecordReuse::default(),
        }),
    );
}

/// 种子 4000000001 第 30 步：F 抬到 7，两次空发布，一个落点都没回收。
/// 判别力：上限少算一档时新 F 不是 7。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn seed_4000000001_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_7_in_two_publishes(
) {
    the_floor_raise_on_narrow_devices_with_the_admission_skipped_has_its_outcome_and_the_history_runs_to_the_end(
        4_000_000_001,
        30,
        &StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(7),
            publishes: 2,
            reclaimed_placements: 0,
            reuse: RecordReuse::default(),
        }),
    );
}

/// 大档：种子数、第一个种子、线程数、收不收缩（`every` / `none`）从环境变量取；每段步数、比重、跑法（checker、盘宽、空间准入）
/// 经 `HistorySampling::from_environment` 读，变量名与判红时打的复现行同一张表（`SINGLEFS_RANDOM_HISTORY_OPERATIONS`、`_WEIGHTS`、
/// `_CHECKER`、`_DEVICES`、`_SPACE_ADMISSION`），没设的取 `LARGE_TIER_DEFAULT_SAMPLING`；
/// 种子基没给就用这个测试周期写死的那一个，跟上面五段同基（给了什么就跑什么）。
#[test]
#[ignore = "大档：SINGLEFS_RANDOM_HISTORY_SEEDS 段、每段 SINGLEFS_RANDOM_HISTORY_OPERATIONS 步，从 SINGLEFS_RANDOM_HISTORY_FIRST_SEED 起（没给就用这个测试周期写死的种子基），SINGLEFS_RANDOM_HISTORY_THREADS 个线程，比重 SINGLEFS_RANDOM_HISTORY_WEIGHTS（broad、reuse、rollback、wall、unit-area-wall），checker SINGLEFS_RANDOM_HISTORY_CHECKER（只认 skip），盘宽 SINGLEFS_RANDOM_HISTORY_DEVICES（4gib、unit-area-384、unit-area-256、unit-area-240），空间准入 SINGLEFS_RANDOM_HISTORY_SPACE_ADMISSION（judged、skipped），收缩 SINGLEFS_RANDOM_HISTORY_SHRINK；release 下后台跑"]
fn random_histories_large_tier_seeds_and_length_from_the_environment() {
    let first_seed = number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_FIRST_SEED",
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
    );
    let seed_count = count_from_environment("SINGLEFS_RANDOM_HISTORY_SEEDS", 1000);
    let worker_threads = worker_threads_from_environment();
    let sampling = HistorySampling::from_environment(
        RANDOM_HISTORY_VARIABLE_PREFIX,
        &process_environment_variable,
        &LARGE_TIER_DEFAULT_SAMPLING,
    )
    .unwrap_or_else(|error| panic!("{}", error.render()));
    let shrinking = match process_environment_variable("SINGLEFS_RANDOM_HISTORY_SHRINK").as_deref()
    {
        None | Some("every") => FindingShrinking::EveryFindingClass,
        Some("none") => FindingShrinking::ReportSeedsOnly,
        Some(other) => panic!("SINGLEFS_RANDOM_HISTORY_SHRINK={other}：只认 every 与 none"),
    };
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        first_seed,
        seed_count,
        sampling.operations_per_history,
        &sampling.weights,
        sampling.execution,
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

/// 「收缩一个种子」那条入口读环境、生成要复现的那段历史：种子（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED`）必给，
/// 取样参数经 `random_history_to_reproduce_from_environment` 读（变量与判红时打的复现行同一张表），没设的取快档的。
/// 入口与「复现行喂回入口」那几条自证都经这一个函数：自证红了，就是照抄复现行跑入口跑不出原来那一段。
fn history_to_reproduce_from(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> (GeneratedHistory, HistorySampling) {
    let (seed, sampling) =
        random_history_to_reproduce_from_environment(lookup, &FAST_TIER_SAMPLING)
            .unwrap_or_else(|error| panic!("{}", error.render()));
    (sampling.generate(seed), sampling)
}

/// 收缩一个种子：快档与各取样点只报种子，判红时打的复现行（`random_history_reproduction_line`，带种子、步数、比重、checker、盘宽、空间准入）
/// 照抄过来跑这一条，生成的历史与跑法与判红那一段逐项相同，再把它收到最短复现。
#[test]
#[ignore = "收缩一个种子：照判红时打的「复现：」那一行设环境变量；SINGLEFS_RANDOM_HISTORY_SHRINK_SEED 必给，其余没设的取快档的（30 步、broad、skip、4gib、judged），SINGLEFS_RANDOM_HISTORY_THREADS 默认取每台默认重活线程数（SINGLEFS_THREAD_CAP，再没设取 .claude/heavy-thread-default 的登记值）"]
fn shrink_one_failing_seed_from_the_environment() {
    let (history, sampling) = history_to_reproduce_from(&process_environment_variable);
    let worker_threads = worker_threads_from_environment();
    let HistoryEnding::NewFinding {
        signature,
        observation,
    } = execute_history_with(
        &history,
        sampling.execution,
        &SharedStream::new(),
        &mut |_| {},
    )
    .ending
    else {
        panic!(
            "种子 {} 的 {} 步（比重 {}；{}）没有撞到新发现",
            history.seed.0,
            sampling.operations_per_history,
            sampling.weights.environment_value(),
            sampling.execution.name()
        );
    };
    let shrunk = shrink_to_reproduction(&history, &signature, sampling.execution, worker_threads);
    print_uncaptured(
        &NewFindingReport {
            signature,
            first_seed: history.seed,
            observation,
            seeds: vec![history.seed],
            shrunk: Some(shrunk),
            sampling,
        }
        .render(),
    );
}

/// 判红时对一段取样会打的那一行复现命令（`NewFindingReport::render` 里「复现：」那一行；造一个假发现，不必真红），
/// 原样喂回复现入口（`history_to_reproduce_from`，「收缩一个种子」那条用的同一个函数）：生成的历史逐步与原趟相同，跑法也相同。
/// 种子取种子基之后第 5 个（取样区间里任一个都行，只要不是 0）。
fn the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
    sampling: HistorySampling,
) {
    let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 5);
    let original = sampling.generate(seed);
    let rendered = NewFindingReport {
        signature: FailureSignature::HarnessJudgement {
            judgement: "自证用的假发现",
        },
        first_seed: seed,
        observation: FailureObservation {
            position: StepPosition::Operation(3),
            operation_kind: Some(original.operations[3].kind()),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        },
        seeds: vec![seed],
        shrunk: None,
        sampling,
    }
    .render();
    let reproduction_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED="))
        .collect();
    assert_eq!(
        reproduction_lines.len(),
        1,
        "判红的报告里恰好一行复现命令：\n{rendered}"
    );
    let assignments = environment_assignments_in_line(reproduction_lines[0]);
    let (reproduced, reproduced_sampling) =
        history_to_reproduce_from(&|variable: &str| assignments.get(variable).cloned());
    assert_eq!(
        reproduced.starting_point, original.starting_point,
        "复现出来的起点不同；复现行：{}",
        reproduction_lines[0]
    );
    let first_differing_step = original
        .operations
        .iter()
        .zip(&reproduced.operations)
        .position(|(original_step, reproduced_step)| original_step != reproduced_step);
    assert_eq!(
        (first_differing_step, reproduced.operations.len()),
        (None, original.operations.len()),
        "复现出来的历史与原趟逐步比：第一个不同的步、步数；复现行：{}",
        reproduction_lines[0]
    );
    assert_eq!(reproduced, original);
    assert_eq!(
        reproduced_sampling.execution, sampling.execution,
        "复现出来的跑法（checker、盘宽、空间准入）不同；复现行：{}",
        reproduction_lines[0]
    );
    assert_eq!(reproduced_sampling, sampling);
}

#[test]
fn reproduction_line_of_the_fast_tier_regenerates_the_same_history_and_execution() {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        FAST_TIER_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_reuse_sampling_regenerates_the_same_history_and_execution() {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        REUSE_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_rollback_sampling_regenerates_the_same_history_and_execution() {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        ROLLBACK_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_allocation_record_wall_sampling_regenerates_the_same_history_and_execution(
) {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        WALL_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_unit_area_wall_sampling_with_the_space_admission_skipped_regenerates_the_same_history_and_execution(
) {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED,
    );
}

#[test]
fn reproduction_line_of_the_unit_area_wall_sampling_with_the_space_admission_judged_regenerates_the_same_history_and_execution(
) {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED,
    );
}

/// 数值变量的值（`None` 是没设）：没设取 `default`；设了而不是非负整数就 panic、消息带变量名与原值
/// （经数值环境变量共用的那一份解析，写错的规模不悄悄退回默认）。
///
/// # Panics
/// 设了而不是非负整数。
fn number_from_the_value_or_default(name: &str, value: Option<&str>, default: u64) -> u64 {
    number_from_the_variable_value_panicking_when_unreadable::<u64>(name, value).unwrap_or(default)
}

/// 进程环境里的数值变量，判法同 [`number_from_the_value_or_default`]。
///
/// # Panics
/// 设了而不是非负整数，或值不是 UTF-8。
fn number_from_environment(name: &str, default: u64) -> u64 {
    number_from_the_value_or_default(name, process_environment_variable(name).as_deref(), default)
}

/// 「跑几个」的计数变量的值（`None` 是没设，种子数这一类）：没设取 `default`；设了而不是正整数（0 也算）就 panic、消息带变量名与原值。
/// 0 段的一批没有「已知红」清单外的失败，只是因为一段都没跑，照样判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」）。
///
/// # Panics
/// 设了而不是正整数。
fn count_from_the_value_or_default(name: &str, value: Option<&str>, default: u64) -> u64 {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)
        .map_or(default, std::num::NonZeroU64::get)
}

/// 进程环境里的计数变量，判法同 [`count_from_the_value_or_default`]。
///
/// # Panics
/// 设了而不是正整数，或值不是 UTF-8。
fn count_from_environment(name: &str, default: u64) -> u64 {
    count_from_the_value_or_default(name, process_environment_variable(name).as_deref(), default)
}

const WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_RANDOM_HISTORY_THREADS";

/// 线程数变量的值（`None` 是没设）：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，
/// 设了照它，再没设取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_harness::heavy_threads`）；快档、取样活动与大档同一个取法。
/// 设了而不是正整数（0 也算）就 panic、消息带变量名与原值，与仓里别的线程数变量同一口径（经数值环境变量共用的那一份解析，取 `NonZeroUsize`）。
///
/// # Panics
/// 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn worker_threads_from_the_value(value: Option<&str>, thread_cap_value: Option<&str>) -> usize {
    own_variable_or_default_heavy_threads(
        WORKER_THREADS_ENVIRONMENT_VARIABLE,
        value,
        thread_cap_value,
    )
    .get()
}

/// 进程环境里的线程数变量与 `SINGLEFS_THREAD_CAP`，判法同 [`worker_threads_from_the_value`]。
///
/// # Panics
/// 设了而不是正整数，或值不是 UTF-8。
fn worker_threads_from_environment() -> usize {
    worker_threads_from_the_value(
        process_environment_variable(WORKER_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 线程数变量没设：取登记的每台默认重活线程数（不取本机核数，也不再夹到 16）；设了照它。
#[test]
fn random_history_worker_threads_take_the_registered_heavy_thread_default_when_unset() {
    assert_eq!(
        worker_threads_from_the_value(None, None),
        registered_heavy_threads().get(),
        "SINGLEFS_RANDOM_HISTORY_THREADS 与 SINGLEFS_THREAD_CAP 都没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(
        worker_threads_from_the_value(None, Some("5")),
        5,
        "没设自己的变量读 SINGLEFS_THREAD_CAP"
    );
    assert_eq!(
        worker_threads_from_the_value(Some("3"), Some("5")),
        3,
        "自己的变量先读"
    );
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_SEEDS=\"ten\" 认不出")]
fn an_unreadable_random_history_seed_count_panics_naming_the_variable_and_the_value() {
    let _seed_count =
        count_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("ten"), 1000);
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_FIRST_SEED=\"ten\" 认不出")]
fn an_unreadable_random_history_first_seed_panics_naming_the_variable_and_the_value() {
    let _first_seed =
        number_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_FIRST_SEED", Some("ten"), 7);
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_SEEDS=\"0\" 认不出")]
fn a_random_history_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _seed_count =
        count_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("0"), 1000);
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_THREADS=\"0\" 认不出")]
fn a_random_history_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _threads = worker_threads_from_the_value(Some("0"), None);
}

/// 删除多于一片下段叶的文件那一格（代码轮 U13）的取样点：种子区间与每段步数，写死。
const DELETION_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

const DELETION_SAMPLING_SEEDS: u64 = 48;

const DELETION_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: 30,
    weights: GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

/// 删除多于一片下段叶的文件那一格（代码轮第一轮判决 U13：模型罩多个数据单元的文件、随机历史里出现 ≥ 2 片下段叶的删除批；
/// 第二轮判决第二节 N3「走空多步」）：比重取 `GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`，种子与步数见常量。
/// 覆盖写的内容另抽跨几个单元的、经会话的顺序写发（多半两片下段叶），unlink 之后建 inode、多回退到第四新的根（意图那一版或第一个删除批那一版）、
/// 再建 inode、覆盖写或 unlink：会话先把队列走空。先判没有新发现，再核这一格真的跑到了：两片下段叶的文件发成过、模型比过不是最后一批的删除批的段数
/// 与会话改动之前先走空走了至少两步的段数都不为 0（「走空被空间拒」在 4 GiB 的盘上走不到，只打数不判）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn random_histories_deleting_files_of_several_lower_leaves_compare_every_batch_and_the_walks_before_a_change(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        DELETION_SAMPLING_FIRST_SEED,
        DELETION_SAMPLING_SEEDS,
        DELETION_SAMPLING.operations_per_history,
        &DELETION_SAMPLING.weights,
        DELETION_SAMPLING.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向删除多于一片下段叶的文件的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let two_lower_leaves = ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
        data_units: 147,
        selector: 0,
    };
    assert!(
        report
            .tally
            .content_lengths_published
            .get(two_lower_leaves.name())
            .copied()
            .unwrap_or(0)
            >= 1,
        "两片下段叶的文件一次都没经顺序写发成：\n{rendered}"
    );
    assert!(
        report
            .tally
            .histories_with_a_deletion_batch_keeping_data_units
            >= 1,
        "没有一段历史里模型比过不是最后一批的删除批（≥ 2 片下段叶）：\n{rendered}"
    );
    assert!(
        report
            .tally
            .histories_with_a_walk_of_several_steps_before_a_user_change
            >= 1,
        "没有一段历史里会话改动之前先走空走了至少两步：\n{rendered}"
    );
}

/// 两片下段叶的文件（150 个数据单元，`UserChange::SequentialWrite` 写的）经会话 unlink：意图那次、一个不是最后一批的删除批
/// （释放尾部 144 个，留 6 个）、最后一批，三版逐版交模型比（代码轮 U13）。判别力：模型把文件当一个数据单元（意图之后的下一步就是最后一批）时，
/// 第一个删除批交回的这一版还带着数据单元、inode 叶与 `deleted_inodes` 根，模型那一版已经没有这几个角色，判「单元的分配代」对不上。
#[test]
fn deleting_a_file_of_two_lower_leaves_publishes_a_batch_keeping_data_units_that_the_model_compares(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 150,
                    selector: 7,
                },
                fill_seed: 11,
            }),
            HistoryOperation::Unlink,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
        "顺序写与 unlink 都做成：{:?}",
        run.outcomes
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.deletion_batches_keeping_data_units_matched,
            run.tally.histories_with_a_deletion_batch_keeping_data_units,
            counts.file_contents_compared,
        ),
        (1, 1, 2),
        "一个不是最后一批的删除批比过；带文件的两版（起点的第一个文件、150 个单元的那一版）各比过一次整份内容"
    );
}

/// 两片下段叶的文件 unlink 之后建一个 inode（删完的那一版没有文件、回退不接，建过 inode 的才接），再回退到 unlink 意图那一版
/// （第四新的根：建 inode、最后一批、删除批、意图），号回到队列；随后再建一个 inode，会话先走空——一个删除批、最后一批两步——再建 inode，
/// 三版逐版交模型比（第二轮判决第二节 N3「走空多步」）。判别力：模型把删除中的那一版的下一步一律当最后一批时，走空的第一步交回的这一版
/// 还带着数据单元，判「单元的分配代」对不上。
#[test]
fn deleting_a_file_of_two_lower_leaves_then_rolling_back_to_its_intent_walks_two_steps_before_the_next_change(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 150,
                    selector: 7,
                },
                fill_seed: 11,
            }),
            HistoryOperation::Unlink,
            HistoryOperation::CreateInode,
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::CreateInode,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[3],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到意图那一版做成：{:?}",
        run.outcomes[3]
    );
    assert!(
        matches!(run.outcomes[4], StepOutcome::Applied(_)),
        "走空之后建 inode 做成：{:?}",
        run.outcomes[4]
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.walks_of_several_steps_before_a_user_change_matched,
            run.tally
                .histories_with_a_walk_of_several_steps_before_a_user_change,
            counts.deletion_batches_keeping_data_units_matched,
        ),
        (1, 1, 2),
        "建 inode 之前走空两步比过一次；不是最后一批的删除批 unlink 里一个、走空里一个"
    );
}

/// 跨三片下段叶的文件（292 个单元，`UserChange::SequentialWrite` 写的）：映射条目多于一片叶装的 294 条，实现的中央映射树从中间切开、
/// 长出根之下的节点（D8（核心索引结构） 已定项 11）；再经会话 unlink：意图那次、两个不是最后一批的删除批（292 → 148 → 4）、最后一批，
/// 逐版交模型比（U13 慢档取样段在种子 …115 等六段上撞到的那一形）。判别力：胶水把映射树根之下的节点当成模型罩不到的池而 panic 时，
/// 顺序写那一步就以 panic 收尾。
#[test]
fn a_file_of_three_lower_leaves_grows_the_central_mapping_tree_below_its_root_and_the_model_compares_every_version(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 292,
                    selector: 7,
                },
                fill_seed: 11,
            }),
            HistoryOperation::Unlink,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
        "顺序写与 unlink 都做成：{:?}",
        run.outcomes
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.deletion_batches_keeping_data_units_matched,
            counts.file_contents_compared,
        ),
        (2, 2),
        "两个不是最后一批的删除批比过；带文件的两版（起点的第一个文件、292 个单元的那一版）各比过一次整份内容"
    );
}
