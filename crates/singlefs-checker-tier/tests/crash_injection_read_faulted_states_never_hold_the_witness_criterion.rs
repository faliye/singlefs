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
//! 多线程（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」，C907）：工作线程数取崩溃注入战役同一个变量
//! `SINGLEFS_CRASH_INJECTION_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值；`crash_injection::CrashInjectionWorkerThreads`）。
//! 两层各按区间切片：找历史那一层一波试线程数那么多个种子，取够段数就停；判的那一层的单位是每段取到的历史上摆进来的每一段
//! （段上的崩溃态与它最长那个前缀上挂载的二次崩溃镜像、那一段的第三截表）与每段历史的第一次崩溃那张表，各片各自从空池建状态。
//! 计数按单位的次序相加，描述与各张表的结论按单位的次序拼，判定与线程数、调度无关（快档用例
//! `read_faulted_states_judged_on_several_worker_threads_and_on_one_report_the_same_states_and_counts` 对拍）。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use singlefs_checker_tier::crash_injection::{
    judge_the_given_states_on_the_cpu, seed_slices, CrashInjectionWorkerThreads,
    GivenStatesConclusionsOfATable, GivenStatesJudge, GivenStatesOfATable,
    HistoryAfterTheMakeFilesystem,
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

/// 崩溃点重放合表里这一项的名字（`.claude/gate.d/crash-point-replay-items.tsv`）。
#[cfg(feature = "verdict-store")]
const CRASH_POINT_REPLAY_ITEM: &str = "read-faulted-states-never-hold-the-witness-criterion";

/// 单项跑得超过它的，每过它一次每片至少打一行进度（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的进度间隔：5 分钟）。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// 每段取到的历史摆进来哪几段（那一步发的第一个写之后结束、最后一步结束之前结束的段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentsAfterTheStepTaken {
    /// 全量与轻型：每一段。
    Every,
    /// 多线程与单线程的对拍（同判法、小输入）：按段次序的头几段。
    AtMost(usize),
}

/// 一批摆多大：取几段那一步发过写的历史、每段历史摆进来哪几段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScaleOfTheBatch {
    histories: usize,
    segments_after_the_step_taken: SegmentsAfterTheStepTaken,
}

/// 多线程与单线程对拍的规模（取自全量那条用例：同一批种子的头两段历史，每段历史只摆头两段）：判那一层有至多 6 个单位，
/// 单线程切 4 片、多线程每个单位一片，两趟的切法不同。
const SCALE_OF_THE_THREAD_COUNT_COMPARISON: ScaleOfTheBatch = ScaleOfTheBatch {
    histories: 2,
    segments_after_the_step_taken: SegmentsAfterTheStepTaken::AtMost(2),
};

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

/// 一批读故障态判完的几个数：取了几段历史、摆了几个崩溃态与几个二次崩溃镜像、W 为真的态、给定状态表那几样判红的态，
/// 连同每张给定状态表的名字与结论（按判的次序：每段历史先各段的第三截表、再第一次崩溃那张表）。
#[derive(Debug, PartialEq)]
struct ReadFaultedStatesOfTheBatch {
    histories_taken: usize,
    read_faulted_crash_states: u64,
    second_crash_images: u64,
    images_holding_the_witness_criterion: Vec<String>,
    images_red_on_the_given_states: Vec<String>,
    conclusions_of_every_table: Vec<(String, GivenStatesConclusionsOfATable)>,
}

/// 第一次崩溃那一态的描述（W 为真的态与第一次崩溃那张表判红的态同一种写法）。
fn first_crash_description(seed: u64, first_write: usize, persisted_writes: usize) -> String {
    format!("种子 {seed} 第一次崩溃：段首写 {first_write}、落了 {persisted_writes} 个")
}

/// 第三截那个二次崩溃镜像的描述（W 为真的态与第三截那张表判红的态同一种写法）。
fn second_crash_description(
    seed: u64,
    first_write: usize,
    persisted_mount_writes: usize,
) -> String {
    format!("种子 {seed} 第三截：段首写 {first_write}、挂载写落了 {persisted_mount_writes} 个")
}

/// 一张给定状态表交 `judge` 判，判红的态（池级 checker、记录核对器两条里有一样红）按 `describe` 逐个记下；交回判红的态与这张表的结论。
fn judge_and_note_the_red_states(
    judge: &dyn GivenStatesJudge,
    table: &GivenStatesOfATable<'_>,
    describe: &dyn Fn(usize) -> String,
) -> (Vec<String>, GivenStatesConclusionsOfATable) {
    let judged = judge
        .judge_the_given_states(table)
        .unwrap_or_else(|reason| panic!("给定状态表 {} 判不了：{reason}", table.table_name));
    assert_eq!(
        judged.states.len(),
        table.persisted_sets.len(),
        "判器逐状态交回结论"
    );
    let mut red_states = Vec::new();
    for (ordinal, state) in judged.states.iter().enumerate() {
        let items = state.red_items();
        if !items.is_empty() {
            red_states.push(format!("{}：{items:?}", describe(ordinal)));
        }
    }
    (red_states, judged)
}

/// 一段那一步发过写的历史：整条写表、切段、每个写在录制流里的下标、mkfs 写完时录制流的下标，连同摆进这一批的段（段号，升序）。
struct HistoryWhoseStepSentWrites {
    seed: u64,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    stream_indexes: Vec<usize>,
    operations_written_by_make_filesystem: usize,
    segments_taken: Vec<usize>,
}

impl HistoryWhoseStepSentWrites {
    /// 给定状态表的基线是 mkfs 之后的池，写表从 mkfs 第一个不是清零的写起、mkfs 的数据写每个状态里都落了（用户 2026-10-10 定，
    /// 创世那几次发布照样进记录核对器；与崩溃注入第一截、第三截同一个构造）。判的那一层每个单位各建一份（各片各自建状态）。
    fn after_the_make_filesystem(
        &self,
        execution: HistoryExecution,
    ) -> HistoryAfterTheMakeFilesystem<'_> {
        HistoryAfterTheMakeFilesystem::of(
            &self.writes,
            &self.segments,
            &self.stream_indexes,
            self.operations_written_by_make_filesystem,
            execution,
        )
    }
}

/// 种子 `seed` 那段历史（那一组比重、24 步）：生成、执行、切写表，摆进来的段照 `segments_after_the_step_taken` 挑；
/// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」那一步一个写都没发就交 `None`。
fn history_whose_step_sent_writes(
    seed: u64,
    execution: HistoryExecution,
    segments_after_the_step_taken: SegmentsAfterTheStepTaken,
) -> Option<HistoryWhoseStepSentWrites> {
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
        run.earliest_stream_index_after_each_power_loss.is_empty(),
        "这一组比重里没有掉电那一步：给定状态表不带掉电接缝"
    );
    let mut operations_before_the_step = 0usize;
    let mut earliest_stream_index_of_the_step = None;
    for (position, operations_after_the_step) in &operations_after_each_step {
        if let StepPosition::Operation(step_index) = position {
            if earliest_stream_index_of_the_step.is_none()
                && history.operations[*step_index].kind()
                    == HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
                && operations_before_the_step < *operations_after_the_step
            {
                earliest_stream_index_of_the_step = Some(operations_before_the_step);
            }
        }
        operations_before_the_step = *operations_after_the_step;
    }
    let earliest_stream_index_of_the_step = earliest_stream_index_of_the_step?;
    let operations_after_the_last_step = operations_before_the_step;
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &WIDTH.fixed_geometry(),
    );
    // 迭代上界是段数；段按写表次序排、段尾的录制流下标递增：到了最后一步结束之后才结束的那一段就停，那一步之前结束的段不摆
    let mut segments_taken = Vec::new();
    for (segment_index, segment) in segments.iter().enumerate() {
        let last_stream_index = stream_indexes[*segment.last().expect("段非空")];
        if last_stream_index >= operations_after_the_last_step {
            break;
        }
        if last_stream_index < earliest_stream_index_of_the_step {
            continue;
        }
        let enough_segments_are_taken = match segments_after_the_step_taken {
            SegmentsAfterTheStepTaken::Every => false,
            SegmentsAfterTheStepTaken::AtMost(segment_limit) => {
                segments_taken.len() == segment_limit
            }
        };
        if enough_segments_are_taken {
            break;
        }
        segments_taken.push(segment_index);
    }
    Some(HistoryWhoseStepSentWrites {
        seed,
        writes,
        segments,
        stream_indexes,
        operations_written_by_make_filesystem: run.operations_written_by_make_filesystem,
        segments_taken,
    })
}

/// 判的那一层的一个单位落在一段历史的哪一部分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartOfAHistory {
    /// 摆进来的一段：段内每个前缀是一个崩溃态（判 W），最长那个前缀上起一次可写挂载、挂载写的每个前缀是一个第三截的二次崩溃镜像（判 W），
    /// 这些二次崩溃镜像合成这一段的第三截表（mkfs 之后的历史接挂载写，在历史与挂载之间断开）。
    SegmentAfterTheStep { segment_index: usize },
    /// 这段历史摆进来的每一段的每个前缀合成的第一次崩溃那张表（持久集合只按下标算，不摆镜像）。
    FirstCrashTable,
}

/// 判的那一层的一个单位：取到的第几段历史（在取到的那几段里的下标）、它的哪一部分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitOfTheBatch {
    history_index_among_those_taken: usize,
    part: PartOfAHistory,
}

/// 一个单位判完的几样：摆了几个崩溃态与几个二次崩溃镜像、W 为真的态、它那张给定状态表判红的态，与那张表的名字与结论。
struct OutcomeOfAUnit {
    read_faulted_crash_states: u64,
    second_crash_images: u64,
    images_holding_the_witness_criterion: Vec<String>,
    images_red_on_the_given_states: Vec<String>,
    table_name_and_conclusions: (String, GivenStatesConclusionsOfATable),
}

/// 判摆进来的一段（[`PartOfAHistory::SegmentAfterTheStep`]）：每判过一个镜像的 W 往 `images_judged` 加一（进度行读它）。
fn judge_a_segment_after_the_step(
    judge: &dyn GivenStatesJudge,
    history: &HistoryWhoseStepSentWrites,
    segment_index: usize,
    execution: HistoryExecution,
    images_judged: &AtomicU64,
) -> OutcomeOfAUnit {
    let seed = history.seed;
    let writes = &history.writes;
    let segment = &history.segments[segment_index];
    let first_write = segment[0];
    let history_after_the_make_filesystem = history.after_the_make_filesystem(execution);
    let writes_after_the_make_filesystem = history_after_the_make_filesystem.writes();
    let first_write_after_the_make_filesystem =
        history_after_the_make_filesystem.index_in_the_table(first_write);
    // 各片各自建状态：段的起步镜像从空池按写表次序叠到段首，与一段接一段累加出来的是同一份
    let mut base = MemoryPool::with_devices(&DEVICES, WIDTH.device_bytes());
    base.apply_writes(&writes[..first_write]);
    let mut read_faulted_crash_states = 0u64;
    let mut second_crash_images = 0u64;
    let mut images_holding_the_witness_criterion = Vec::new();
    let mut longest_prefix = base.clone();
    for persisted_writes in 0..segment.len() {
        let mut image = base.clone();
        image.apply_writes(&writes[first_write..first_write + persisted_writes]);
        read_faulted_crash_states += 1;
        if witness_criterion_holds_without_read_faults(&image) {
            images_holding_the_witness_criterion.push(first_crash_description(
                seed,
                first_write,
                persisted_writes,
            ));
        }
        images_judged.fetch_add(1, Ordering::Relaxed);
        longest_prefix = image;
    }
    let (mount_writes, mount_segments) = writes_of_a_writable_mount_on(&longest_prefix);
    for persisted_mount_writes in 0..mount_writes.len() {
        let mut second_crash_image = longest_prefix.clone();
        second_crash_image.apply_writes(&mount_writes[..persisted_mount_writes]);
        second_crash_images += 1;
        if witness_criterion_holds_without_read_faults(&second_crash_image) {
            images_holding_the_witness_criterion.push(second_crash_description(
                seed,
                first_write,
                persisted_mount_writes,
            ));
        }
        images_judged.fetch_add(1, Ordering::Relaxed);
    }
    // 第三截：mkfs 之后的历史接挂载写，在历史与挂载之间断开；历史那一截照最长那个前缀落、挂载写取每个前缀
    let longest_prefix_writes = first_write_after_the_make_filesystem + segment.len() - 1;
    let history_write_count = writes_after_the_make_filesystem.len();
    let second_crash_writes: Vec<RetainedWrite> = writes_after_the_make_filesystem
        .iter()
        .chain(&mount_writes)
        .cloned()
        .collect();
    let second_crash_segments: Vec<Vec<usize>> = history_after_the_make_filesystem
        .segments()
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
            persisted[history_write_count..history_write_count + persisted_mount_writes].fill(true);
            persisted
        })
        .collect();
    let table_name = format!("seed={seed}/second-crash/segment-first-write={first_write}");
    let (images_red_on_the_given_states, conclusions) = judge_and_note_the_red_states(
        judge,
        &GivenStatesOfATable {
            table_name: table_name.clone(),
            base: history_after_the_make_filesystem.base_pool(),
            writes: &second_crash_writes,
            segments: &second_crash_segments,
            seams: &seams,
            persisted_sets: second_crash_persisted_sets,
        },
        &|ordinal| second_crash_description(seed, first_write, ordinal),
    );
    OutcomeOfAUnit {
        read_faulted_crash_states,
        second_crash_images,
        images_holding_the_witness_criterion,
        images_red_on_the_given_states,
        table_name_and_conclusions: (table_name, conclusions),
    }
}

/// 判一段历史第一次崩溃那张表（[`PartOfAHistory::FirstCrashTable`]）：崩溃态取摆进来的每一段的每个前缀（崩溃窗口末尾在那一步之后，
/// 崩溃注入认成注入过读故障的那一类）。镜像上的 W 归各段那几个单位判，这里只判表。
fn judge_the_first_crash_table(
    judge: &dyn GivenStatesJudge,
    history: &HistoryWhoseStepSentWrites,
    execution: HistoryExecution,
) -> OutcomeOfAUnit {
    let seed = history.seed;
    let history_after_the_make_filesystem = history.after_the_make_filesystem(execution);
    let writes_after_the_make_filesystem = history_after_the_make_filesystem.writes();
    let mut first_crash_persisted_sets: Vec<Vec<bool>> = Vec::new();
    let mut first_crash_descriptions: Vec<String> = Vec::new();
    for &segment_index in &history.segments_taken {
        let segment = &history.segments[segment_index];
        let first_write = segment[0];
        let first_write_after_the_make_filesystem =
            history_after_the_make_filesystem.index_in_the_table(first_write);
        for persisted_writes in 0..segment.len() {
            let mut persisted = vec![false; writes_after_the_make_filesystem.len()];
            persisted[..first_write_after_the_make_filesystem + persisted_writes].fill(true);
            first_crash_persisted_sets.push(persisted);
            first_crash_descriptions.push(first_crash_description(
                seed,
                first_write,
                persisted_writes,
            ));
        }
    }
    let table_name = format!("seed={seed}/first-crash");
    let (images_red_on_the_given_states, conclusions) = judge_and_note_the_red_states(
        judge,
        &GivenStatesOfATable {
            table_name: table_name.clone(),
            base: history_after_the_make_filesystem.base_pool(),
            writes: writes_after_the_make_filesystem,
            segments: history_after_the_make_filesystem.segments(),
            seams: &[],
            persisted_sets: first_crash_persisted_sets,
        },
        &|ordinal| first_crash_descriptions[ordinal].clone(),
    );
    OutcomeOfAUnit {
        read_faulted_crash_states: 0,
        second_crash_images: 0,
        images_holding_the_witness_criterion: Vec::new(),
        images_red_on_the_given_states,
        table_name_and_conclusions: (table_name, conclusions),
    }
}

/// 进度行与跑完那一行是哪一层打的（行首的前缀）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayerOfTheBatch {
    /// 找历史那一层的一波种子。
    SeedWave,
    /// 判的那一层。
    Units,
}

impl LayerOfTheBatch {
    fn line_prefix(self) -> &'static str {
        match self {
            Self::SeedWave => "READ_FAULTED_STATES_SEED_WAVE",
            Self::Units => "READ_FAULTED_STATES_UNITS",
        }
    }
}

/// 一片走到哪了：工作线程边判边记，调用线程每过 [`PROGRESS_INTERVAL`] 照它给每片打一行。
struct ProgressOfASlice {
    is_claimed: AtomicBool,
    is_finished: AtomicBool,
    units_finished: AtomicU64,
    images_judged: AtomicU64,
}

impl ProgressOfASlice {
    fn not_claimed_yet() -> Self {
        Self {
            is_claimed: AtomicBool::new(false),
            is_finished: AtomicBool::new(false),
            units_finished: AtomicU64::new(0),
            images_judged: AtomicU64::new(0),
        }
    }

    fn state_name(&self) -> &'static str {
        if self.is_finished.load(Ordering::Relaxed) {
            "finished"
        } else if self.is_claimed.load(Ordering::Relaxed) {
            "running"
        } else {
            "waiting"
        }
    }
}

/// 把 `units` 切成首尾相接的片（`crash_injection::seed_slices`：片数取 min(单位数, 4 × 线程数)，各片长度相差至多 1），分给
/// `worker_threads.count()` 个工作线程：线程按片号从小到大领片，领片之前问判器叫没叫停（叫停了就不再领，领了的片照样判完交回），
/// 每片各自判自己那几个单位。调用线程收到一片打一行，每过 [`PROGRESS_INTERVAL`] 给每片各打一行（片号、单位区间、等着 / 在判 / 判完、
/// 判完几个单位、判过几个镜像），全部收回之后按片号从小到大放回单位的次序，与线程数、调度次序无关。
/// 交回与 `units` 逐个对应：领了的片里的单位是 `Some`，叫停之后没人领的片里的是 `None`（片号一个接一个发，`Some` 总在前头）。
///
/// # Panics
/// 某个工作线程 panic（判器判不了一张表、用例里的断言不成立）；有一片领了却没交回。
fn outcomes_of_units_judged_in_slices_on_worker_threads<Unit: Sync, Outcome: Send>(
    layer: LayerOfTheBatch,
    units: &[Unit],
    worker_threads: CrashInjectionWorkerThreads,
    judge: &dyn GivenStatesJudge,
    judge_a_unit: &(dyn Fn(&Unit, &AtomicU64) -> Outcome + Sync),
) -> Vec<Option<Outcome>> {
    let unit_count = u64::try_from(units.len()).expect("单位数装得进 u64");
    let slices: Vec<std::ops::Range<usize>> = seed_slices(unit_count, worker_threads.count())
        .into_iter()
        .map(|slice| {
            usize::try_from(slice.start).expect("片首不超过单位数，装得进 usize")
                ..usize::try_from(slice.end).expect("片尾不超过单位数，装得进 usize")
        })
        .collect();
    let spawned_worker_threads = worker_threads.count().min(slices.len());
    let progress: Vec<ProgressOfASlice> = slices
        .iter()
        .map(|_| ProgressOfASlice::not_claimed_yet())
        .collect();
    let next_slice_index = AtomicUsize::new(0);
    let line_prefix = layer.line_prefix();
    let started = Instant::now();
    let finished_slices: BTreeMap<usize, Vec<Outcome>> = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<Outcome>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let progress = &progress;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                if judge.stop_is_requested() {
                    break;
                }
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let progress_of_the_slice = &progress[slice_index];
                progress_of_the_slice
                    .is_claimed
                    .store(true, Ordering::Relaxed);
                let outcomes: Vec<Outcome> = units[slice.clone()]
                    .iter()
                    .map(|unit| {
                        let outcome = judge_a_unit(unit, &progress_of_the_slice.images_judged);
                        progress_of_the_slice
                            .units_finished
                            .fetch_add(1, Ordering::Relaxed);
                        outcome
                    })
                    .collect();
                if sender.send((slice_index, outcomes)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut finished_slices: BTreeMap<usize, Vec<Outcome>> = BTreeMap::new();
        let mut next_progress_report = started + PROGRESS_INTERVAL;
        // 迭代上界：每片收一次，加每个进度间隔醒一次；工作线程都退了（发送端全丢了）就停
        loop {
            match receiver
                .recv_timeout(next_progress_report.saturating_duration_since(Instant::now()))
            {
                Ok((slice_index, outcomes)) => {
                    progress[slice_index]
                        .is_finished
                        .store(true, Ordering::Relaxed);
                    let slice = &slices[slice_index];
                    println!(
                        "{line_prefix}_SLICE_FINISHED slice={}/{} units=[{},{}) finished_slices={}/{} elapsed_seconds={:.1}",
                        slice_index + 1,
                        slices.len(),
                        slice.start,
                        slice.end,
                        finished_slices.len() + 1,
                        slices.len(),
                        started.elapsed().as_secs_f64()
                    );
                    finished_slices.insert(slice_index, outcomes);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if Instant::now() >= next_progress_report {
                for (slice_index, (slice, progress_of_the_slice)) in
                    slices.iter().zip(&progress).enumerate()
                {
                    println!(
                        "{line_prefix}_PROGRESS slice={}/{} units=[{},{}) state={} units_finished={}/{} images_judged={} elapsed_seconds={:.1}",
                        slice_index + 1,
                        slices.len(),
                        slice.start,
                        slice.end,
                        progress_of_the_slice.state_name(),
                        progress_of_the_slice.units_finished.load(Ordering::Relaxed),
                        slice.len(),
                        progress_of_the_slice.images_judged.load(Ordering::Relaxed),
                        started.elapsed().as_secs_f64()
                    );
                }
                next_progress_report = Instant::now() + PROGRESS_INTERVAL;
            }
        }
        finished_slices
    });
    // 领片的下标从 0 起一个接一个发，叫停只让后面的片没人领：领了的片都该交回了
    let slices_claimed = next_slice_index.load(Ordering::Relaxed).min(slices.len());
    assert_eq!(
        finished_slices.len(),
        slices_claimed,
        "领了的每一片都交回了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "{line_prefix}_FINISHED units={} slices={} slices_claimed={slices_claimed} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} elapsed_seconds={:.1}",
        units.len(),
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name(),
        started.elapsed().as_secs_f64()
    );
    let mut outcomes_in_unit_order: Vec<Option<Outcome>> = units.iter().map(|_| None).collect();
    for (slice_index, outcomes_of_the_slice) in finished_slices {
        for (unit_index, outcome) in slices[slice_index].clone().zip(outcomes_of_the_slice) {
            outcomes_in_unit_order[unit_index] = Some(outcome);
        }
    }
    outcomes_in_unit_order
}

/// 找历史那一层：种子基起一波试 `worker_threads.count()` 个种子（各片各自生成、执行、切写表），按种子从小到大取那一步发过写的历史，
/// 取够 `scale.histories` 段就停（最后一波里多试的种子白算，换来的是取到的段与单线程逐个相同）；判器叫停了也停。
fn histories_whose_step_sent_writes(
    judge: &dyn GivenStatesJudge,
    scale: ScaleOfTheBatch,
    execution: HistoryExecution,
    worker_threads: CrashInjectionWorkerThreads,
) -> Vec<HistoryWhoseStepSentWrites> {
    let seeds_per_wave = u64::try_from(worker_threads.count()).expect("线程数装得进 u64");
    let mut histories_taken: Vec<HistoryWhoseStepSentWrites> = Vec::new();
    let mut next_seed_offset = 0u64;
    // 迭代上界是 SEEDS_SEARCHED 个种子按每波 seeds_per_wave 个分的波数；跨轮携带取到的历史与下一波起头的种子
    while histories_taken.len() < scale.histories
        && next_seed_offset < SEEDS_SEARCHED
        && !judge.stop_is_requested()
    {
        let seed_offsets: Vec<u64> =
            (next_seed_offset..(next_seed_offset + seeds_per_wave).min(SEEDS_SEARCHED)).collect();
        next_seed_offset += u64::try_from(seed_offsets.len()).expect("一波的种子数装得进 u64");
        let wave = outcomes_of_units_judged_in_slices_on_worker_threads(
            LayerOfTheBatch::SeedWave,
            &seed_offsets,
            worker_threads,
            judge,
            &|seed_offset: &u64, _images_judged: &AtomicU64| {
                history_whose_step_sent_writes(
                    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset,
                    execution,
                    scale.segments_after_the_step_taken,
                )
            },
        );
        for tried_seed in wave {
            // 叫停之后没人领的种子：它与它后面的种子都不取（取到的段照旧是按种子次序的头几段）
            let Some(history_of_the_seed) = tried_seed else {
                break;
            };
            if let Some(history) = history_of_the_seed {
                if histories_taken.len() < scale.histories {
                    histories_taken.push(history);
                }
            }
        }
    }
    histories_taken
}

/// 场景：那一组比重、种子基起「每块盘最新那一槽系统配置读回全 0 的可写挂载」发过写的前 `scale.histories` 段历史，每段 24 步。
/// 崩溃态取那一步发的第一个写之后结束的段（`scale.segments_after_the_step_taken` 挑哪几段）的每个前缀（崩溃窗口末尾在那一步之后，
/// 崩溃注入认成注入过读故障的那一类）；每段最长那个前缀上再起一次可写挂载，挂载写的每个前缀是一个第三截的二次崩溃镜像。
/// 每个镜像判 W；崩溃态一段历史一张给定状态表、二次崩溃一个最长前缀一张表，交 `judge` 判。
/// 两层都分到 `worker_threads` 上（[`histories_whose_step_sent_writes`]、[`outcomes_of_units_judged_in_slices_on_worker_threads`]）；
/// 合并时计数按单位的次序相加、描述与表的结论按单位的次序拼，一段历史的单位都判完了才算取到这一段（叫停时后面的单位没人判）。
fn judge_the_read_faulted_states(
    judge: &dyn GivenStatesJudge,
    scale: ScaleOfTheBatch,
    worker_threads: CrashInjectionWorkerThreads,
) -> ReadFaultedStatesOfTheBatch {
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: WIDTH,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    println!(
        "READ_FAULTED_STATES_START histories={} segments_after_the_step_taken={:?} seeds_searched={SEEDS_SEARCHED} configured_worker_threads={} worker_threads_source={}",
        scale.histories,
        scale.segments_after_the_step_taken,
        worker_threads.count(),
        worker_threads.source_name()
    );
    let histories = histories_whose_step_sent_writes(judge, scale, execution, worker_threads);
    let units: Vec<UnitOfTheBatch> = histories
        .iter()
        .enumerate()
        .flat_map(|(history_index_among_those_taken, history)| {
            history
                .segments_taken
                .iter()
                .map(move |&segment_index| UnitOfTheBatch {
                    history_index_among_those_taken,
                    part: PartOfAHistory::SegmentAfterTheStep { segment_index },
                })
                .chain(std::iter::once(UnitOfTheBatch {
                    history_index_among_those_taken,
                    part: PartOfAHistory::FirstCrashTable,
                }))
        })
        .collect();
    let outcomes = outcomes_of_units_judged_in_slices_on_worker_threads(
        LayerOfTheBatch::Units,
        &units,
        worker_threads,
        judge,
        &|unit: &UnitOfTheBatch, images_judged: &AtomicU64| {
            let history = &histories[unit.history_index_among_those_taken];
            match unit.part {
                PartOfAHistory::SegmentAfterTheStep { segment_index } => {
                    judge_a_segment_after_the_step(
                        judge,
                        history,
                        segment_index,
                        execution,
                        images_judged,
                    )
                }
                PartOfAHistory::FirstCrashTable => {
                    judge_the_first_crash_table(judge, history, execution)
                }
            }
        },
    );
    let mut batch = ReadFaultedStatesOfTheBatch {
        histories_taken: 0,
        read_faulted_crash_states: 0,
        second_crash_images: 0,
        images_holding_the_witness_criterion: Vec::new(),
        images_red_on_the_given_states: Vec::new(),
        conclusions_of_every_table: Vec::new(),
    };
    let mut units_not_judged_of_each_history: Vec<usize> = histories
        .iter()
        .map(|history| history.segments_taken.len() + 1)
        .collect();
    for (unit, outcome) in units.iter().zip(outcomes) {
        let Some(outcome) = outcome else {
            continue;
        };
        batch.read_faulted_crash_states += outcome.read_faulted_crash_states;
        batch.second_crash_images += outcome.second_crash_images;
        batch
            .images_holding_the_witness_criterion
            .extend(outcome.images_holding_the_witness_criterion);
        batch
            .images_red_on_the_given_states
            .extend(outcome.images_red_on_the_given_states);
        batch
            .conclusions_of_every_table
            .push(outcome.table_name_and_conclusions);
        units_not_judged_of_each_history[unit.history_index_among_those_taken] -= 1;
    }
    batch.histories_taken = units_not_judged_of_each_history
        .iter()
        .filter(|units_not_judged| **units_not_judged == 0)
        .count();
    batch
}

/// 种子与规模（进给定状态表的块键，第三轮正文 H8）。
#[cfg(feature = "verdict-store")]
fn seeds_and_scale_of(histories: usize) -> String {
    format!(
        "seed_base={SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE} seeds_searched={SEEDS_SEARCHED} histories={histories} operations_per_history={OPERATIONS_PER_HISTORY} weights=NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS width={WIDTH:?}"
    )
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

/// 前 `histories` 段经崩溃点重放的引擎判给定状态表（`fault` 只给证明 GPU 与 CPU 两趟的结论会不同的那条用例）；工作线程照环境取
/// （[`CrashInjectionWorkerThreads::from_the_environment`]），各工作线程共用这一个引擎（同崩溃注入战役）。
#[cfg(feature = "verdict-store")]
fn read_faulted_states_through_the_given_states_engine(
    histories: usize,
    fault: FaultPutOnTheGpuSide,
    start: EngineStart,
) -> (ReadFaultedStatesOfTheBatch, GivenStatesRun) {
    let start_the_engine = || {
        GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side(
            GivenStatesItem {
                item_name: format!("{}::{}", CRASH_POINT_REPLAY_ITEM, seeds_and_scale_of(histories)),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
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
    let batch = judge_the_read_faulted_states(
        &engine,
        ScaleOfTheBatch {
            histories,
            segments_after_the_step_taken: SegmentsAfterTheStepTaken::Every,
        },
        CrashInjectionWorkerThreads::from_the_environment(),
    );
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

/// 在 CPU 上就地判给定状态表、从不叫停的判器（不经判定库：对拍的两趟各判各的，不复用另一趟的结论）。
struct GivenStatesJudgedOnTheCpuInPlace;

impl GivenStatesJudge for GivenStatesJudgedOnTheCpuInPlace {
    fn stop_is_requested(&self) -> bool {
        false
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        Ok(judge_the_given_states_on_the_cpu(table))
    }
}

/// 多线程与单线程的对拍（`.claude/rules/test-script-discipline.md`「编译与算法都要快」：合并要确定，对拍进快档）：同一组输入
/// （[`SCALE_OF_THE_THREAD_COUNT_COMPARISON`]，取自全量那条用例的头两段历史、每段历史头两段），一趟照环境取的线程数（不到 2 取 2）、
/// 一趟 1 个线程，两趟都在 CPU 上就地判给定状态表；取到的段数、两类态的个数、W 为真的态、判红的态、每张表的名字与逐状态结论逐字相同。
// crash-case-check:not-a-crash-case 两段历史各摆头两段的小输入，只核多线程与单线程两趟逐字相同；全量是同文件 read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion
#[test]
fn read_faulted_states_judged_on_several_worker_threads_and_on_one_report_the_same_states_and_counts(
) {
    let several_worker_threads = CrashInjectionWorkerThreads::GivenByCaller(
        CrashInjectionWorkerThreads::from_the_environment()
            .count()
            .max(2),
    );
    let on_several_worker_threads = judge_the_read_faulted_states(
        &GivenStatesJudgedOnTheCpuInPlace,
        SCALE_OF_THE_THREAD_COUNT_COMPARISON,
        several_worker_threads,
    );
    let on_one_worker_thread = judge_the_read_faulted_states(
        &GivenStatesJudgedOnTheCpuInPlace,
        SCALE_OF_THE_THREAD_COUNT_COMPARISON,
        CrashInjectionWorkerThreads::GivenByCaller(1),
    );
    assert_eq!(
        on_several_worker_threads,
        on_one_worker_thread,
        "{} 个工作线程与 1 个工作线程在同一组输入上判出的段数、计数、W 为真的态、判红的态与每张表的结论逐字相同",
        several_worker_threads.count()
    );
    // 不是空对拍：单线程那一趟（两趟相同，也就是两趟）取够了段、两类态各摆到了
    assert!(
        on_one_worker_thread.histories_taken == SCALE_OF_THE_THREAD_COUNT_COMPARISON.histories
            && on_one_worker_thread.read_faulted_crash_states >= 1
            && on_one_worker_thread.second_crash_images >= 1,
        "前提：单线程那一趟取够了 {} 段历史、两类态各摆到了（取了 {} 段、崩溃态 {} 个、二次崩溃镜像 {} 个）",
        SCALE_OF_THE_THREAD_COUNT_COMPARISON.histories,
        on_one_worker_thread.histories_taken,
        on_one_worker_thread.read_faulted_crash_states,
        on_one_worker_thread.second_crash_images
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
    let directory = singlefs_checker_tier::crash_library::DisposableLibraryDirectory::new(
        &format!("read-faulted-given-states-on-the-gpu-{tag}"),
    )
    .expect("流水线单元测试那一类里这条用例的目录");
    let library = directory.join("library");
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    // 清场之下要清场之后那一份显卡先后：调用方给这个进程的那一份经环境变量交给引擎（不读此刻的环境；
    // 抄进配置在调用方的显卡先后只写在仓根 multi-host.env、副本里又没有那一份时抄成空串，见 common_gpu_cards_of_the_caller）
    let cards = common_gpu_cards_of_the_caller::card_priority_value_of_the_caller();
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
    let directory = singlefs_checker_tier::crash_library::DisposableLibraryDirectory::new(
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
