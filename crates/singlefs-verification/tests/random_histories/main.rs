//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 1 件：随机历史。生成器、执行器、「已知红」清单与收缩在 `singlefs_verification::history`；
//! 这里是随机历史这一批（不设变量跑默认批，设了 `SINGLEFS_RANDOM_HISTORY_*` 照变量跑）、几段专门的取样点与清单每一条的复现。
//! 「收缩一个种子」是工具、不是判定，不在这个测试目标里：判红时打的复现行起的是 harness 的 example `shrink_one_failing_random_history_seed`
//! （`cargo run --example`），这里只留复现行喂回同一个读环境的函数那几条自证。
//!
//! 五段取样的种子基都是 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`：永久固定、不重抽
//! （为什么见它的文档注释），崩溃注入那个二进制用的是同一个；种子数照旧写死，里程碑的验收说的是「写死的种子数之内判红」。
#[path = "../common/mod.rs"]
mod common;
#[path = "../common_images_for_the_given_states_engine/mod.rs"]
mod common_images_for_the_given_states_engine;
#[path = "../common_shared_by_harness_scenarios/mod.rs"]
mod common_shared_by_harness_scenarios;
#[path = "../common_images_for_the_given_states_engine/random_histories.rs"]
mod scenarios_of_random_histories;

mod batch_and_sampling_settings;
mod file_deletion_histories;
mod fixed_histories_of_rollbacks_and_floor_raises;
mod history_execution_and_uncaptured_output;
mod reproduction_lines_regenerating_a_history;
mod samplings_ending_only_in_known_red_forms;

use common::disk_snapshot;
use common_images_for_the_given_states_engine::PoolCheckerImages;
use scenarios_of_random_histories::pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version;
use singlefs_checker::image::{chosen_system_configurations, valid_roots};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, roll_back_by_a_forward_publish,
    RollbackCandidateExclusion, RollbackError, RollbackTarget,
};
use singlefs_verification::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault, SharedFaultPlan,
};
use singlefs_verification::heavy_threads::{
    own_variable_or_default_heavy_threads, registered_heavy_threads,
    THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_verification::history::{
    allocation_records_on_the_image_under, classify_failure, environment_assignments_in_line,
    execute_history_handing_out_every_step_image, execute_history_with,
    execute_history_with_faults, generate_history, generate_history_with_weights,
    raised_floor_lands_only_on_abandoned_roots, random_history_to_reproduce_from_environment,
    run_history_campaign, AppliedEffect, ColdStartReadBack, ContentChoice, ContentLength,
    FailureObservation, FailureSignature, FindingShrinking, FloorTargetChoice, GeneratedHistory,
    GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed, HistoryStartingPoint,
    HistoryTally, MountAllocationComparison, MountSpaceAdmissionOutcome, NewFindingReport,
    PerStepChecker, RecordReuse, RollbackTargetChoice, StepOutcome, StepPosition,
    RANDOM_HISTORY_DEFAULT_SAMPLING, RANDOM_HISTORY_VARIABLE_PREFIX,
    RANDOM_HISTORY_WORKER_THREADS_VARIABLE,
};
use singlefs_verification::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
use singlefs_verification::model::{
    ModelCheckpointTxg, ModelInstanceGeneration, ModelRefusalReason, ModelRootKey,
    ObservedRefusalReason,
};
use singlefs_verification::model_comparison::refusal_reason_of_rollback_error;
use singlefs_verification::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_verification::SharedStream;
use std::io::Write as _;

use batch_and_sampling_settings::*;
use history_execution_and_uncaptured_output::*;
