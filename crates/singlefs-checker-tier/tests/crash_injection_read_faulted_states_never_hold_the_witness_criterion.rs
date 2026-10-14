//! checker 档模块：crash_point_replay_progress、crash_injection、crash_library、crash_amplification、gpu_unit_checks
//! C696（崩溃注入的读故障一支在随机历史里从没触发）判别力自证里「第三截调用点写死成 NoneInjected」那一种接错，在随机历史上是等价变异
//! （第 26 批 C696 设计调查报告第三节）：写死之后只在三样都成立的态上改判定——第一次崩溃那一态注入过读故障、
//! 它上面那次可写挂载途中的二次崩溃镜像上再挂以判据 N-配置 拒、那个二次崩溃镜像上判据 W（`judge_writable_mount_refusal_of_a_crash_state`
//! 没注入读故障时报的 C670 那一条）为真。这里照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」
//! 第 2 点把等价性写成一条测试留档：在「每块盘最新那一槽系统配置读回全 0 的可写挂载」那一组比重的历史上，那一步发过写之后的每个崩溃态
//! （段内前缀）与它们上面可写挂载的每个写前缀（第三截的二次崩溃镜像），W 一个都不为真。真会改行为的那一种接错由
//! `crash_injection::tests::a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670`
//! 那条靶向用例钉（W 为真的阳性对照）。
//! 推翻「等价」的观测：这里任一个态上 W 为真。量的范围：一组比重、一档盘宽（单元区 384 槽）、种子基起前 4 段那一步发过写的历史、
//! 段内只摆前缀、第三截只摆挂载写的前缀；更宽的范围没量。
//! 同一批崩溃态与二次崩溃镜像另判池级 checker、两遍恢复、记录核对器（用户 2026-10-10 定「读故障那一项也接给定状态表，加判镜像那几样」）：
//! 交给定状态表（`crash_injection::GivenStatesOfATable`，基线是 mkfs 之后的池；第三截一张表在历史与挂载之间断开），编进 `verdict-store`
//! 特性时经崩溃点重放的引擎（照本机多机配置只走 GPU 或只走 CPU，表先写进判定库、从库里读回来判），没编进来时在 CPU 上就地判；一个都不许红。
//! W 照旧在 CPU 上判（豁免表 `witness-criterion-w` 那一行）。

use singlefs_checker_tier::crash_injection::{
    GivenStatesJudge, GivenStatesOfATable, HistoryAfterTheMakeFilesystem,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker, StepPosition,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RecordStreamSeam, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    RefusalConditionNotMet, WritableMountRefusalVerdict,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[cfg(feature = "verdict-store")]
use singlefs_checker_tier::crash_amplification::{
    FaultPutOnTheGpuSide, GivenStatesEngine, GivenStatesItem, GivenStatesRun,
};
#[cfg(feature = "verdict-store")]
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;

#[cfg(feature = "gpu")]
mod common_gpu_cards_of_the_caller;

const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
const HISTORIES: usize = 4;
const SEEDS_SEARCHED: u64 = 64;
const OPERATIONS_PER_HISTORY: usize = 24;
/// 小规模那几条（轻型里跑的，取自上面那一批的头一段历史）：只取一段。
#[cfg(feature = "verdict-store")]
const HISTORIES_OF_THE_SMALL_BATCH: usize = 1;

/// 这份测试文件的仓内路径：给定状态表那一路的设点文件（CPU 结论的版本按它出发的闭包取）。
#[cfg(feature = "verdict-store")]
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs";
/// 崩溃点重放合表里这一项的名字（`.claude/gate.d/crash-point-replay-items.tsv`）。
#[cfg(feature = "verdict-store")]
const CRASH_POINT_REPLAY_ITEM: &str = "read-faulted-states-never-hold-the-witness-criterion";

/// 判据 N-配置 为真、重读仍为真的一拒（判官自己在镜像上复算判据，不看这里填的读数）。
fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointTxg)) -> MountError {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: selected_version.0,
            checkpoint_txg: selected_version.1,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 1,
            comparison: WitnessedCounterComparison::Undecidable,
        },
    };
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: reading,
            reread: reading,
        },
    ))
}

/// W：这个镜像当作没注入读故障的崩溃态，判官对一次判据 N-配置 的拒报不报 C670 那一条（判据在镜像上复算、撕裂的系统配置槽跳过仍为真）。
/// 只读恢复落不到一版时判据无从复算，交回假。
fn witness_criterion_holds_without_read_faults(image: &MemoryPool) -> bool {
    let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
    let Some(selected_version) = fields.report.effective_root else {
        return false;
    };
    match judge_writable_mount_refusal_of_a_crash_state(
        image,
        fields.report.effective_root,
        &witness_refusal_claimed_on(selected_version),
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ReadFaultsInTheCrashState::NoneInjected,
    ) {
        WritableMountRefusalVerdict::Illegal {
            conditions_not_met, ..
        } => conditions_not_met.iter().any(|condition| {
            matches!(
                condition,
                RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
            )
        }),
        WritableMountRefusalVerdict::Legal(_)
        | WritableMountRefusalVerdict::ReasonNotOnTheD23List => false,
    }
}

/// 崩溃镜像上起一次可写挂载（与崩溃注入第二截同一种做法），交回挂载写表与它的切段（做成与否都交，被拒时是空的）。
fn writes_of_a_writable_mount_on(image: &MemoryPool) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let parameters = WIDTH.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let _mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &WIDTH.fixed_geometry(),
    );
    (writes, segments)
}

/// 一批读故障态判完的几个数：取了几段历史、摆了几个崩溃态与几个二次崩溃镜像、W 为真的态、给定状态表那几样判红的态。
struct ReadFaultedStatesOfTheBatch {
    histories_taken: usize,
    read_faulted_crash_states: u64,
    second_crash_images: u64,
    images_holding_the_witness_criterion: Vec<String>,
    images_red_on_the_given_states: Vec<String>,
}

/// 一张给定状态表交 `judge` 判，判红的态（池级 checker、记录核对器两条里有一样红）按 `describe` 逐个记下。
fn judge_and_note_the_red_states(
    judge: &dyn GivenStatesJudge,
    table: &GivenStatesOfATable<'_>,
    describe: &dyn Fn(usize) -> String,
    red_states: &mut Vec<String>,
) {
    let judged = judge
        .judge_the_given_states(table)
        .unwrap_or_else(|reason| panic!("给定状态表 {} 判不了：{reason}", table.table_name));
    assert_eq!(
        judged.states.len(),
        table.persisted_sets.len(),
        "判器逐状态交回结论"
    );
    for (ordinal, state) in judged.states.iter().enumerate() {
        let items = state.red_items();
        if !items.is_empty() {
            red_states.push(format!("{}：{items:?}", describe(ordinal)));
        }
    }
}

/// 场景：那一组比重、种子基起「每块盘最新那一槽系统配置读回全 0 的可写挂载」发过写的前 `histories` 段历史，每段 24 步。崩溃态取那一步发的第一个写
/// 之后结束的每一段的每个前缀（崩溃窗口末尾在那一步之后，崩溃注入认成注入过读故障的那一类）；每段最长那个前缀上再起一次可写挂载，
/// 挂载写的每个前缀是一个第三截的二次崩溃镜像。每个镜像判 W；崩溃态一段历史一张给定状态表、二次崩溃一个最长前缀一张表，交 `judge` 判。
fn judge_the_read_faulted_states(
    judge: &dyn GivenStatesJudge,
    histories: usize,
) -> ReadFaultedStatesOfTheBatch {
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: WIDTH,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mut batch = ReadFaultedStatesOfTheBatch {
        histories_taken: 0,
        read_faulted_crash_states: 0,
        second_crash_images: 0,
        images_holding_the_witness_criterion: Vec::new(),
        images_red_on_the_given_states: Vec::new(),
    };
    // 迭代上界是 SEEDS_SEARCHED 个种子；跨轮携带几样计数与判红的态的描述；取够 `histories` 段就停，判器叫停了也停（手里那一段判完）。
    for seed_offset in 0..SEEDS_SEARCHED {
        if batch.histories_taken == histories || judge.stop_is_requested() {
            break;
        }
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset;
        let history = generate_history_with_weights(
            HistorySeed(seed),
            OPERATIONS_PER_HISTORY,
            &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
        );
        let stream = SharedStream::retaining_contents();
        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            operations_after_each_step.push((observation.position, stream.operation_count()));
        });
        assert!(
            run.first_stream_index_after_each_power_loss.is_empty(),
            "这一组比重里没有掉电那一步：给定状态表不带掉电接缝"
        );
        let mut operations_before_the_step = 0usize;
        let mut first_stream_index_of_the_step = None;
        for (position, operations_after_the_step) in &operations_after_each_step {
            if let StepPosition::Operation(step_index) = position {
                if first_stream_index_of_the_step.is_none()
                    && history.operations[*step_index].kind()
                        == HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
                    && operations_before_the_step < *operations_after_the_step
                {
                    first_stream_index_of_the_step = Some(operations_before_the_step);
                }
            }
            operations_before_the_step = *operations_after_the_step;
        }
        let Some(first_stream_index_of_the_step) = first_stream_index_of_the_step else {
            continue;
        };
        let operations_after_the_last_step = operations_before_the_step;
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &WIDTH.fixed_geometry(),
        );
        // 给定状态表的基线是 mkfs 之后的池，写表从 mkfs 第一个不是清零的写起、mkfs 的数据写每个状态里都落了（用户 2026-10-10 定，
        // 创世那几次发布照样进记录核对器；与崩溃注入第一截、第三截同一个构造）
        let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
            &writes,
            &segments,
            &stream_indexes,
            run.operations_written_by_make_filesystem,
            execution,
        );
        let base_after_the_make_filesystem = history_after_the_make_filesystem.base();
        let writes_after_the_make_filesystem = history_after_the_make_filesystem.writes();
        let segments_after_the_make_filesystem = history_after_the_make_filesystem.segments();
        let mut first_crash_persisted_sets: Vec<Vec<bool>> = Vec::new();
        let mut first_crash_descriptions: Vec<String> = Vec::new();
        let mut base = MemoryPool::with_devices(&DEVICES, WIDTH.device_bytes());
        let mut writes_applied_to_the_base = 0usize;
        for segment in &segments {
            let first_write = segment[0];
            let last_stream_index = stream_indexes[*segment.last().expect("段非空")];
            if last_stream_index >= operations_after_the_last_step {
                break;
            }
            base.apply_writes(&writes[writes_applied_to_the_base..first_write]);
            writes_applied_to_the_base = first_write;
            if last_stream_index < first_stream_index_of_the_step {
                continue;
            }
            let first_write_after_the_make_filesystem =
                history_after_the_make_filesystem.index_in_the_table(first_write);
            let mut longest_prefix = base.clone();
            for persisted_writes in 0..segment.len() {
                let mut image = base.clone();
                image.apply_writes(&writes[first_write..first_write + persisted_writes]);
                batch.read_faulted_crash_states += 1;
                if witness_criterion_holds_without_read_faults(&image) {
                    batch.images_holding_the_witness_criterion.push(format!(
                        "种子 {seed} 第一次崩溃：段首写 {first_write}、落了 {persisted_writes} 个"
                    ));
                }
                let mut persisted = vec![false; writes_after_the_make_filesystem.len()];
                persisted[..first_write_after_the_make_filesystem + persisted_writes].fill(true);
                first_crash_persisted_sets.push(persisted);
                first_crash_descriptions.push(format!(
                    "种子 {seed} 第一次崩溃：段首写 {first_write}、落了 {persisted_writes} 个"
                ));
                longest_prefix = image;
            }
            let (mount_writes, mount_segments) = writes_of_a_writable_mount_on(&longest_prefix);
            for persisted_mount_writes in 0..mount_writes.len() {
                let mut second_crash_image = longest_prefix.clone();
                second_crash_image.apply_writes(&mount_writes[..persisted_mount_writes]);
                batch.second_crash_images += 1;
                if witness_criterion_holds_without_read_faults(&second_crash_image) {
                    batch.images_holding_the_witness_criterion.push(format!(
                        "种子 {seed} 第三截：段首写 {first_write}、挂载写落了 {persisted_mount_writes} 个"
                    ));
                }
            }
            // 第三截：mkfs 之后的历史接挂载写，在历史与挂载之间断开；历史那一截照最长那个前缀落、挂载写取每个前缀
            let longest_prefix_writes = first_write_after_the_make_filesystem + segment.len() - 1;
            let history_write_count = writes_after_the_make_filesystem.len();
            let second_crash_writes: Vec<RetainedWrite> = writes_after_the_make_filesystem
                .iter()
                .chain(&mount_writes)
                .cloned()
                .collect();
            let second_crash_segments: Vec<Vec<usize>> = segments_after_the_make_filesystem
                .iter()
                .cloned()
                .chain(mount_segments.iter().map(|mount_segment| {
                    mount_segment
                        .iter()
                        .map(|write| write + history_write_count)
                        .collect()
                }))
                .collect();
            let seams = [RecordStreamSeam::AfterACrash {
                first_write_after_the_crash: history_write_count,
                version_landed_on_after_the_crash: recover(&longest_prefix, JournalPolicy::Consult)
                    .effective_root,
            }];
            let second_crash_persisted_sets: Vec<Vec<bool>> = (0..mount_writes.len())
                .map(|persisted_mount_writes| {
                    let mut persisted = vec![false; second_crash_writes.len()];
                    persisted[..longest_prefix_writes].fill(true);
                    persisted[history_write_count..history_write_count + persisted_mount_writes]
                        .fill(true);
                    persisted
                })
                .collect();
            judge_and_note_the_red_states(
                judge,
                &GivenStatesOfATable {
                    table_name: format!(
                        "seed={seed}/second-crash/segment-first-write={first_write}"
                    ),
                    base: base_after_the_make_filesystem,
                    writes: &second_crash_writes,
                    segments: &second_crash_segments,
                    seams: &seams,
                    persisted_sets: second_crash_persisted_sets,
                },
                &|ordinal| {
                    format!("种子 {seed} 第三截：段首写 {first_write}、挂载写落了 {ordinal} 个")
                },
                &mut batch.images_red_on_the_given_states,
            );
        }
        judge_and_note_the_red_states(
            judge,
            &GivenStatesOfATable {
                table_name: format!("seed={seed}/first-crash"),
                base: base_after_the_make_filesystem,
                writes: writes_after_the_make_filesystem,
                segments: segments_after_the_make_filesystem,
                seams: &[],
                persisted_sets: first_crash_persisted_sets,
            },
            &|ordinal| first_crash_descriptions[ordinal].clone(),
            &mut batch.images_red_on_the_given_states,
        );
        batch.histories_taken += 1;
    }
    batch
}

/// 种子与规模（进给定状态表的块键，第三轮正文 H8）。
#[cfg(feature = "verdict-store")]
fn seeds_and_scale_of(histories: usize) -> String {
    format!(
        "seed_base={SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE} seeds_searched={SEEDS_SEARCHED} histories={histories} operations_per_history={OPERATIONS_PER_HISTORY} weights=NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS width={WIDTH:?}"
    )
}

#[cfg(feature = "verdict-store")]
fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../Cargo.lock")
        .canonicalize()
        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
        .parent()
        .expect("Cargo.lock 所在的目录就是仓根")
        .to_path_buf()
}

/// 起引擎时拿不拿改环境的锁（同 `crash_injection_campaign.rs` 的 `EngineStart`）：拿着 `CrashAmplificationEnvironment` 的调用方不许再拿
/// （同一把锁，不可重入），别的调用方要拿。引擎起步不是一下读完环境（先读多机配置与分片，照树算完结论版本才读库目录），读到一半同一个进程里
/// 别的用例拿着守卫换了多机配置、分片与库目录，这一趟就读成一半调用方的、一半那条用例的：分片照调用方的、库是那条用例起跑清空的那一份，
/// 两条用例往同一份库里判同一批键相同的表（项名、种子与规模、表名都相同），那条用例「从空库起、一张都不复用」的账就不成立。
#[cfg(feature = "verdict-store")]
#[derive(Clone, Copy)]
enum EngineStart {
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            dead_code,
            reason = "拿着环境守卫起引擎的只有带 gpu 特性的 GPU 与 CPU 两趟对拍"
        )
    )]
    UnderTheCallersEnvironmentGuard,
    HoldingTheEnvironmentLock,
}

/// 前 `histories` 段经崩溃点重放的引擎判给定状态表（`fault` 只给证明 GPU 与 CPU 两趟的结论会不同的那条用例）。
#[cfg(feature = "verdict-store")]
fn read_faulted_states_through_the_given_states_engine(
    histories: usize,
    fault: FaultPutOnTheGpuSide,
    start: EngineStart,
) -> (ReadFaultedStatesOfTheBatch, GivenStatesRun) {
    let start_the_engine = || {
        GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side(
            GivenStatesItem {
                item_name: CRASH_POINT_REPLAY_ITEM.to_string(),
                setting_file: THIS_FILE.to_string(),
                seeds_and_scale: seeds_and_scale_of(histories),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
            &repository_root(),
            fault,
        )
        .expect("崩溃点重放的引擎照本机配置起得来（配了 GPU 卡用不上、结论版本与编译时对不上都在这里红，不退回 CPU）")
    };
    let engine = match start {
        EngineStart::UnderTheCallersEnvironmentGuard => start_the_engine(),
        EngineStart::HoldingTheEnvironmentLock => {
            let _one_test_at_a_time =
                singlefs_checker_tier::crash_amplification::one_test_changes_the_environment_at_a_time();
            start_the_engine()
        }
    };
    let batch = judge_the_read_faulted_states(&engine, histories);
    (batch, engine.finish())
}

/// 前 `histories` 段：编进 `verdict-store` 特性时经崩溃点重放的引擎（GPU 判不了都在这里红），没编进来时在 CPU 上就地判。
/// 引擎那一趟被叫停、或两台分片而别的机器的表还没导回来时这一批不作数：打「本次未跑」那一行（`test_name` 是用例名）、交 `None`。
fn read_faulted_states_through_the_crash_point_replay_entry(
    histories: usize,
    test_name: &str,
) -> Option<ReadFaultedStatesOfTheBatch> {
    #[cfg(feature = "verdict-store")]
    {
        use singlefs_harness::gpu_memory_ledger::gpu_not_run_line;
        let (batch, run) = read_faulted_states_through_the_given_states_engine(
            histories,
            FaultPutOnTheGpuSide::NoFault,
            EngineStart::HoldingTheEnvironmentLock,
        );
        assert_eq!(
            run.red_items(),
            Vec::<String>::new(),
            "给定状态表那一路判红（GPU 判不了）"
        );
        if run.stopped_early {
            println!(
                "{}",
                gpu_not_run_line(
                    test_name,
                    &format!(
                        "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE 指的文件在）：取了 {} 段就停，判过的表在库里",
                        batch.histories_taken
                    )
                )
            );
            return None;
        }
        if run.recorded_only() {
            println!(
                "{}",
                gpu_not_run_line(
                    test_name,
                    &format!(
                        "只跑了录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）：{} 张给定状态表写进库、没判，判段那一趟从库里读回判",
                        run.tally.tables_recorded_only
                    )
                )
            );
            return None;
        }
        if run.left_tables_to_the_other_hosts() {
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
            return None;
        }
        Some(batch)
    }
}

/// 一批的预期：取够了段、两类态各至少一个（不是空判）、W 一个都不为真、给定状态表那几样一个都不红。
fn assert_the_read_faulted_states_hold_neither_the_witness_criterion_nor_a_red_image(
    batch: &ReadFaultedStatesOfTheBatch,
    histories: usize,
) {
    assert_eq!(
        batch.histories_taken, histories,
        "前提：种子基起 {SEEDS_SEARCHED} 段里有 {histories} 段那一步发过写"
    );
    assert!(
        batch.read_faulted_crash_states >= 1 && batch.second_crash_images >= 1,
        "前提：摆到了注入过读故障的崩溃态 {} 个、第三截二次崩溃镜像 {} 个",
        batch.read_faulted_crash_states,
        batch.second_crash_images
    );
    assert_eq!(
        batch.images_holding_the_witness_criterion,
        Vec::<String>::new(),
        "注入过读故障的崩溃态 {} 个、二次崩溃镜像 {} 个上 W 都不为真（写死 NoneInjected 是等价变异）",
        batch.read_faulted_crash_states,
        batch.second_crash_images
    );
    assert_eq!(
        batch.images_red_on_the_given_states,
        Vec::<String>::new(),
        "这批崩溃态与二次崩溃镜像上池级 checker、记录核对器一个都不红"
    );
}

/// 场景：那一组比重、种子基起前 4 段那一步发过写的历史（摆法见 [`judge_the_read_faulted_states`]）。
/// 预期：这些镜像上 W 一个都不为真（写死 NoneInjected 与正确接线在这批态上判出一样的东西）；崩溃态与二次崩溃镜像各至少一个（不是空判）；
/// 池级 checker、两遍恢复、记录核对器交给定状态表判，一个都不红。
#[test]
#[ignore = "checker 档：自己逐个摆崩溃态复算判据，release 下跑"]
fn read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion() {
    let Some(batch) = read_faulted_states_through_the_crash_point_replay_entry(
        HISTORIES,
        "read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion",
    ) else {
        return;
    };
    assert_the_read_faulted_states_hold_neither_the_witness_criterion_nor_a_red_image(
        &batch, HISTORIES,
    );
}

/// 轻型那一步（取自上面那条重型用例：同一批种子的头一段历史）：经崩溃点重放的引擎判，照本机配置走 GPU 或 CPU，W 不为真、给定状态表那几样不红。
// crash-case-check:not-a-crash-case 一段历史的小规模那一步，全量是同文件 read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion
#[cfg(feature = "verdict-store")]
#[test]
fn the_first_read_faulted_history_judged_through_the_crash_point_replay_entry_holds_no_red_image() {
    let Some(batch) = read_faulted_states_through_the_crash_point_replay_entry(
        HISTORIES_OF_THE_SMALL_BATCH,
        "the_first_read_faulted_history_judged_through_the_crash_point_replay_entry_holds_no_red_image",
    ) else {
        return;
    };
    assert_the_read_faulted_states_hold_neither_the_witness_criterion_nor_a_red_image(
        &batch,
        HISTORIES_OF_THE_SMALL_BATCH,
    );
}

/// 开 GPU 的头一段历史：这一趟自己写一份开 GPU 的多机配置、在环境守卫里跑；`fault` 塞在 GPU 那一侧。库是家目录下流水线单元测试那一类里
/// 以用例标签命名的那一份、起跑先清空、跑完留着：不用给定状态表那一类的长期库——塞了故障的那一趟会把错的 GPU 结论存进去、下一趟复用，
/// 一致那一趟又会把上一趟判过的全复用、一个状态都不在卡上判。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
fn the_first_read_faulted_history_judged_on_the_gpu(
    tag: &str,
    fault: FaultPutOnTheGpuSide,
) -> (ReadFaultedStatesOfTheBatch, GivenStatesRun) {
    use singlefs_checker_tier::crash_amplification::CrashAmplificationEnvironment;
    let directory =
        singlefs_checker_tier::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
            &format!("read-faulted-given-states-on-the-gpu-{tag}"),
        )
        .expect("流水线单元测试那一类里这条用例的目录");
    let library = directory.join("library");
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    // 清场之下要清场之后那一份显卡先后：调用方给这个进程的那一份经环境变量交给引擎（不读此刻的环境；
    // 抄进配置在调用方的显卡先后只写在仓根 multi-host.env、副本里又没有那一份时抄成空串，见 common_gpu_cards_of_the_caller）
    let cards =
        common_gpu_cards_of_the_caller::card_priority_value_of_the_caller(&repository_root());
    // 机器角色同样照抄调用方给这个进程的那一份：守卫连它一起清掉，引擎取卡按此刻的环境认这台机器、卡池按调用方那一份认，
    // 不设回去时第二台上的引擎认成本机、拿本机那几张卡去对第二台的卡池（同 crash_injection_campaign.rs 的 settings_of_an_engine_on_the_gpu）
    use singlefs_checker_tier::gpu_card_configuration::{
        GpuConfigurationOfTheCaller, CARDS_BY_PRIORITY_KEY, HOST_ROLE_ENVIRONMENT_VARIABLE,
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    };
    let host_of_the_caller = GpuConfigurationOfTheCaller::of_this_process()
        .host()
        .expect("调用方给的机器角色是 local 或 peer（写错时卡池起不来，用例开头起池那一步先红）");
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            HOST_ROLE_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new(host_of_the_caller.name()),
        ),
        (CARDS_BY_PRIORITY_KEY, std::ffi::OsStr::new(&cards)),
        ("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", library.as_os_str()),
    ]);
    let judged = read_faulted_states_through_the_given_states_engine(
        HISTORIES_OF_THE_SMALL_BATCH,
        fault,
        EngineStart::UnderTheCallersEnvironmentGuard,
    );
    drop(environment);
    judged
}

/// 没开 GPU 的头一段历史：这一趟自己写一份 `ENABLE_GPU=0` 的多机配置、在环境守卫里跑，库同 [`the_first_read_faulted_history_judged_on_the_gpu`]
/// （用例标签命名、起跑清空、跑完留着）：GPU 那一趟的对拍对象，两趟各判各的、各写各的库。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
fn the_first_read_faulted_history_judged_on_the_cpu(
    tag: &str,
) -> (ReadFaultedStatesOfTheBatch, GivenStatesRun) {
    use singlefs_checker_tier::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_checker_tier::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
    let directory =
        singlefs_checker_tier::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
            &format!("read-faulted-given-states-on-the-cpu-{tag}"),
        )
        .expect("流水线单元测试那一类里这条用例的目录");
    let library = directory.join("library");
    let configuration = directory.join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了没开 GPU 的配置");
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        ("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", library.as_os_str()),
    ]);
    let judged = read_faulted_states_through_the_given_states_engine(
        HISTORIES_OF_THE_SMALL_BATCH,
        FaultPutOnTheGpuSide::NoFault,
        EngineStart::UnderTheCallersEnvironmentGuard,
    );
    drop(environment);
    judged
}

/// GPU 判器与 CPU 判器的对拍是独立的两趟（用户 2026-10-11 定：一段里 CPU 不既跑数据又对拍）：同一段历史一趟配了 GPU、由 GPU 判器判，
/// 一趟没开 GPU、由 CPU 判器判，各写各的库；两趟摆出的崩溃态与二次崩溃镜像一样多，W 为真的态与给定状态表判红的态逐个相同，
/// 两趟都没有判不了的。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
// crash-case-check:not-a-crash-case 一段历史的小规模对拍，只核 GPU 与 CPU 两趟的结论相同；全量是同文件 read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
#[test]
fn the_first_read_faulted_history_judged_in_a_gpu_run_and_in_a_cpu_run_reports_the_same_states() {
    use singlefs_checker_tier::crash_amplification::JudgedBy;
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    if gpu_card_pool_or_report_not_run(
        "the_first_read_faulted_history_judged_in_a_gpu_run_and_in_a_cpu_run_reports_the_same_states",
    )
    .is_none()
    {
        return;
    }
    let (gpu_batch, gpu_run) =
        the_first_read_faulted_history_judged_on_the_gpu("agrees", FaultPutOnTheGpuSide::NoFault);
    let (cpu_batch, cpu_run) = the_first_read_faulted_history_judged_on_the_cpu("agrees");
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
    assert_eq!(
        gpu_run.red_items(),
        Vec::<String>::new(),
        "GPU 那一趟都判得了"
    );
    assert_eq!(cpu_run.red_items(), Vec::<String>::new());
    assert!(
        gpu_batch.read_faulted_crash_states > 0 && gpu_batch.second_crash_images > 0,
        "两类态都摆出来了"
    );
    assert!(
        gpu_run.tally.states_judged_by_the_gpu_here > 0 && gpu_run.tally.states_reused == 0,
        "GPU 那一趟从空库起、每个状态都是这一趟在卡上判的，不是复用的：{:?}",
        gpu_run.tally
    );
    assert!(
        cpu_run.tally.states_judged_by_the_cpu_here > 0 && cpu_run.tally.states_reused == 0,
        "CPU 那一趟从空库起、每个状态都是这一趟判的，不是复用的：{:?}",
        cpu_run.tally
    );
    assert_eq!(
        (
            gpu_batch.histories_taken,
            gpu_batch.read_faulted_crash_states,
            gpu_batch.second_crash_images,
            &gpu_batch.images_holding_the_witness_criterion,
            &gpu_batch.images_red_on_the_given_states,
        ),
        (
            cpu_batch.histories_taken,
            cpu_batch.read_faulted_crash_states,
            cpu_batch.second_crash_images,
            &cpu_batch.images_holding_the_witness_criterion,
            &cpu_batch.images_red_on_the_given_states,
        ),
        "GPU 那一趟与 CPU 那一趟逐态相同"
    );
}

/// 上一条的判别力：往 GPU 那一侧塞一个已知故障（每张表第一个状态的第一条不变量违例位翻过来），GPU 那一趟判红的态与 CPU 那一趟不同。
// crash-case-check:not-a-crash-case 一段历史的小规模对拍，只核塞在 GPU 那一侧的故障让两趟的结论分开；全量是同文件 read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
#[test]
fn a_fault_put_on_the_gpu_side_of_the_first_read_faulted_history_makes_the_gpu_run_differ_from_the_cpu_run(
) {
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    if gpu_card_pool_or_report_not_run(
        "a_fault_put_on_the_gpu_side_of_the_first_read_faulted_history_makes_the_gpu_run_differ_from_the_cpu_run",
    )
    .is_none()
    {
        return;
    }
    let (gpu_batch, _gpu_run) = the_first_read_faulted_history_judged_on_the_gpu(
        "fault",
        FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
    );
    let (cpu_batch, _cpu_run) = the_first_read_faulted_history_judged_on_the_cpu("fault");
    assert!(
        !gpu_batch.images_red_on_the_given_states.is_empty(),
        "GPU 那一侧塞了故障，GPU 那一趟一个红态都没判出来"
    );
    assert_ne!(
        gpu_batch.images_red_on_the_given_states, cpu_batch.images_red_on_the_given_states,
        "GPU 那一侧塞了故障，两趟判红的态却相同"
    );
}
