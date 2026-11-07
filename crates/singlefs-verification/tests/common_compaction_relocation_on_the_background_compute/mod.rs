//! 整理搬迁流照挂载参数 `background_compute` 录（D26（后台整理与放置回收） 已定项 13 第 1 条；里程碑四七·4，D13（验证路线） 已定项 4）：
//! 同一负载、同一池——`common_compaction_prefix` 的整理搬迁流那一串（取号 → 暖机 → A → B、C 两次顺序写 → 重挂 → 整理签发 → 一批 → 最后一批，
//! B、C 的内容同 `common_crash_points::content_needing` 那两份）——重挂时与整理每一次发布之前照计划
//! （[`BackendsOfTheCompactionPublishes`]）定后台计算走 `cpu` 还是 `gpu`。`gpu` 那一侧递进去的是验证包里背后照核心层 CPU 写法算的替身
//! （`common_background_gpu_compute`，不起真卡）；批间要换后端的，在那一次发布之前经会话请求一次通过校验的切换（下一批生效）。
//!
//! 录出来的整条录制流（mkfs 起块层的每一个请求连同内容）与 `common_compaction_prefix::prepare_the_compaction_crash_flow` 录的全程 `cpu`
//! 那一条逐字节比（[`assert_the_block_sequence_is_the_one_of_the_cpu_flow`]，一条断言把两份录制直接连起来比）；每一步发成的版本也逐项相同时，
//! 版本表照抄那一条（版本表只由 A 与每一步发成的版本、两次顺序写的内容定），写表、段与被判的根槽写从这一条自己的录制流算：
//! 崩溃点重放枚举的是这一条录下的写。必红世界在 `crates/mutations.tsv`：替身把一条记录的源槽算错（GPU 计划改掉一个源落点），块序列比对那一条断言红。
//!
//! 两份崩溃枚举测试文件（全程 `gpu` 与批间来回切）共用的判器核法与计数名也放在这里（与 `crash_enumeration_compaction_relocation_stream.rs`
//! 里同名的几样同一个写法：那几样是那份测试文件私有的，引不过来）。
#![allow(
    dead_code,
    reason = "两份崩溃枚举测试文件各自 mod 进来，每份只用到其中几样"
)]

use std::sync::Mutex;

use super::common::{
    build_pool, geometry, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
};
use super::common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, shared_cpu_backed_gpu_compute,
    CpuBackedBackgroundGpuComputeForTests,
};
use super::common_compaction_crash_point_observers::landed_counter_of;
use super::common_compaction_prefix::{
    compaction_switches_of_the_crash_flows, prepare_the_compaction_crash_flow, version_of_trees,
    CompactionCrashFlow, CompactionFlowStep, CompactionFlowStepKind, PreparedCompactionCrashFlow,
    RecordedCompactionFlow, COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST,
    PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE,
};
use super::common_crash_points::content_needing;
use singlefs_core::background_compute::{
    BackgroundComputeBackend, BackgroundComputeOfOneBatch, BackgroundComputePathTaken,
    BackgroundComputeSettings, BackgroundGpuComputeHandle, ReasonTheBatchRanOnTheCpu,
    BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use singlefs_core::mount::mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::transaction::TransactionOutput;
use singlefs_verification::crash::{CrashPointReplayTally, CrashPointReplayVersionTable};
use singlefs_verification::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_verification::segments::StepKind;

/// 整条流的段数（录流按真写表数出来的，同全程 `cpu` 的整理搬迁流：前二十段同多记录流，重挂、签发、一批、最后一批各自的单元、记录、根、轮换）。
pub const SEGMENTS_OF_THE_FLOW: usize = 42;

/// 每次写只取两态时整条流的闭式（`memory_pool::closed_form_state_count_of_write_index_segments`）：两条流逐字节相同，与全程 `cpu` 的整理搬迁流是同一个数。
pub const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 1_377_370_259;

/// 重挂与整理每一次发布各走哪个后端。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendsOfTheCompactionPublishes {
    /// 重挂就选 `gpu`，签发、一批、最后一批都在 GPU 路径上算。
    GpuOnEveryPublish,
    /// 重挂选 `cpu`，签发在 CPU 上；签发之后切到 `gpu`，那一批在 GPU 路径上；那一批之后切回 `cpu`，最后一批在 CPU 上
    /// （两个方向各切一次，两次都是通过校验的切换）。
    CpuThenGpuForTheBatchThenCpuAgain,
}

impl BackendsOfTheCompactionPublishes {
    /// 重挂时给的后端：签发那一次发布用的就是它。
    #[must_use]
    pub const fn backend_of_the_remount(self) -> BackgroundComputeBackend {
        self.backend_of_the_compaction_publish(0)
    }

    /// 已经录了 `compaction_publishes_already_recorded` 次整理发布之后，下一次（签发是还一次都没录时那一次）走哪个后端。
    /// 超出这条流三次发布的照最后一批那一次（必红世界里一趟可能多发）。
    #[must_use]
    pub const fn backend_of_the_compaction_publish(
        self,
        compaction_publishes_already_recorded: usize,
    ) -> BackgroundComputeBackend {
        match self {
            Self::GpuOnEveryPublish => BackgroundComputeBackend::Gpu,
            Self::CpuThenGpuForTheBatchThenCpuAgain => {
                if compaction_publishes_already_recorded == 1 {
                    BackgroundComputeBackend::Gpu
                } else {
                    BackgroundComputeBackend::Cpu
                }
            }
        }
    }
}

/// B、C 两次顺序写的内容（同全程 `cpu` 的整理搬迁流：多记录流那两份）。
#[must_use]
pub fn contents_of_the_sequential_writes() -> Vec<Vec<u8>> {
    vec![content_needing(2, 1), content_needing(3, 2)]
}

/// 整理的一次发布那一趟：发成的是哪一种（签发、一批、最后一批）、计划给它的后端、那一趟每一批的后台计算记账、那一趟里替身被调了几次（三段合起来）。
#[derive(Clone, Debug)]
pub struct CompactionPublishOnTheBackgroundCompute {
    pub kind: CompactionFlowStepKind,
    pub planned_backend: BackgroundComputeBackend,
    pub background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch>,
    pub calls_handed_to_the_gpu_compute: usize,
}

/// 照计划的后端录好的整理搬迁流：流本身（形态同 `common_compaction_prefix::RecordedCompactionFlow`）与整理每一次发布那一趟的后台计算记账。
pub struct CompactionRelocationRecordedOnTheBackgroundCompute {
    pub backends: BackendsOfTheCompactionPublishes,
    pub recorded: RecordedCompactionFlow,
    pub compaction_publishes: Vec<CompactionPublishOnTheBackgroundCompute>,
}

/// 照计划的后端录好、按崩溃点重放要的样子摆好的流（[`prepare_the_compaction_relocation_on_the_background_compute`]）。
pub struct PreparedOnTheBackgroundCompute {
    pub backends: BackendsOfTheCompactionPublishes,
    pub prepared: PreparedCompactionCrashFlow,
    pub compaction_publishes: Vec<CompactionPublishOnTheBackgroundCompute>,
}

fn settings_of_the_backend(backend: BackgroundComputeBackend) -> BackgroundComputeSettings {
    match backend {
        BackgroundComputeBackend::Cpu => BackgroundComputeSettings::PRODUCT,
        BackgroundComputeBackend::Gpu => admissible_gpu_settings_naming_both_cards(),
    }
}

/// 挂载或切换时递进去的 GPU 实现：选 `gpu` 递替身，选 `cpu` 不递（切回 `cpu` 时沿用这次挂载已有的那一份）。
fn gpu_compute_handed_in_for(
    backend: BackgroundComputeBackend,
    gpu_compute_handle: &BackgroundGpuComputeHandle,
) -> Option<BackgroundGpuComputeHandle> {
    match backend {
        BackgroundComputeBackend::Cpu => None,
        BackgroundComputeBackend::Gpu => Some(gpu_compute_handle.clone()),
    }
}

/// 替身到此刻被调了几次（三段合起来，每一段每次调用一项）。
fn calls_handed_to(gpu_compute: &Mutex<CpuBackedBackgroundGpuComputeForTests>) -> usize {
    gpu_compute
        .lock()
        .expect("替身的锁：拿过它的线程没有一个 panic 过")
        .entries_handed_in
        .len()
}

/// 实例 1 里顺序写一版，记成流里一步（同 `common_compaction_prefix` 录 B、C 的那一种写法）。
fn record_a_sequential_write(pool: &mut BuiltPool, content: &[u8]) -> CompactionFlowStep {
    let start = pool.stream.operation_count();
    let output = sequential_write_in_process_as_instance_one_and_advance_the_pool(pool, content);
    let end = pool.stream.operation_count();
    pool.output = output.clone();
    CompactionFlowStep {
        kind: CompactionFlowStepKind::SequentialWrite,
        operations: start..end,
        versions: vec![output],
        discards: Vec::new(),
    }
}

/// 重开镜像、照 `backend` 可写挂载（整理开关同录流，GC 停着时在 CPU 上补跑的阈值取默认），会话装上同一份整理开关。挂载发的写录进同一条流。
fn remount_on_the_backend(
    pool: &mut BuiltPool,
    backend: BackgroundComputeBackend,
    gpu_compute_handle: &BackgroundGpuComputeHandle,
) -> (CompactionFlowStep, MountedSession) {
    let start = pool.stream.operation_count();
    let mut devices = pool.reopen_recorded();
    let mounted =
        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
            &parameters(),
            &mut devices,
            settings_of_the_backend(backend),
            gpu_compute_handed_in_for(backend, gpu_compute_handle),
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            compaction_switches_of_the_crash_flows(),
        )
        .unwrap_or_else(|refused| {
            panic!("重挂选 {backend:?}：设置全合法、替身的两张卡都在，可写挂载做得成：{refused:?}")
        });
    pool.devices = Some(devices);
    let end = pool.stream.operation_count();
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_compaction_switches(compaction_switches_of_the_crash_flows());
    let mut versions = vec![version_of_trees(&output.row_publish).clone()];
    versions.extend(
        output
            .warm_up_publishes
            .iter()
            .map(|published| version_of_trees(published).clone()),
    );
    (
        CompactionFlowStep {
            kind: CompactionFlowStepKind::MountWritable,
            operations: start..end,
            versions,
            discards: Vec::new(),
        },
        session,
    )
}

/// 录一条整理搬迁流：后端照 `backends`（`tag` 是镜像文件名里那一段）。整理一次一次发（预算压到 1），每一次发布记成一步，最后一批同一次发布删了意图就停；
/// 一趟没恰好发成一次、或发的那一次既没写意图也没搬单元（必红世界里会这样），就停在那里、不再往下录，块序列比对那一条断言判它。
///
/// # Panics
/// 顺序写、重挂、请求切换被拒：这几样按条款都做得成。
#[must_use]
pub fn record_the_compaction_relocation_on_the_background_compute(
    tag: &str,
    backends: BackendsOfTheCompactionPublishes,
) -> CompactionRelocationRecordedOnTheBackgroundCompute {
    let (gpu_compute, gpu_compute_handle) = shared_cpu_backed_gpu_compute();
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = Vec::new();
    for content in contents_of_the_sequential_writes() {
        steps.push(record_a_sequential_write(&mut pool, &content));
    }
    let backend_of_the_remount = backends.backend_of_the_remount();
    let (mount_step, mut session) =
        remount_on_the_backend(&mut pool, backend_of_the_remount, &gpu_compute_handle);
    steps.push(mount_step);
    let mut backend_in_effect = backend_of_the_remount;
    let mut compaction_publishes: Vec<CompactionPublishOnTheBackgroundCompute> = Vec::new();
    while compaction_publishes.len() < COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST {
        let planned_backend =
            backends.backend_of_the_compaction_publish(compaction_publishes.len());
        if planned_backend != backend_in_effect {
            session
                .switch_background_compute_from_the_next_batch(
                    settings_of_the_backend(planned_backend),
                    gpu_compute_handed_in_for(planned_backend, &gpu_compute_handle),
                )
                .unwrap_or_else(|refused| {
                    panic!("切到 {planned_backend:?}：设置全合法、替身的两张卡都在，切换通过校验：{refused:?}")
                });
            backend_in_effect = planned_backend;
        }
        session.compaction.publishes_since_the_last_user_change =
            PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
        let calls_before_the_run = calls_handed_to(&gpu_compute);
        let start = pool.stream.operation_count();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let run = session.run_compaction(devices).unwrap_or_else(|refused| {
            panic!("整理一趟：盘表是挂载核过的那一份、现行那一版有树：{refused:?}")
        });
        let end = pool.stream.operation_count();
        let calls_handed_to_the_gpu_compute = calls_handed_to(&gpu_compute) - calls_before_the_run;
        let discards = run.discards.applied.clone();
        let background_compute_of_each_batch = run.background_compute_of_each_batch.clone();
        let Ok([published]) = <[TransactionOutput; 1]>::try_from(run.versions_published) else {
            break;
        };
        let kind = match (
            published.logged_operations.is_empty(),
            published.units_relocated_by_this_publish.is_empty(),
        ) {
            (false, true) => CompactionFlowStepKind::CompactionIssue,
            (false, false) => CompactionFlowStepKind::CompactionBatch,
            (true, false) => CompactionFlowStepKind::CompactionFinalBatch,
            (true, true) => break,
        };
        steps.push(CompactionFlowStep {
            kind,
            operations: start..end,
            versions: vec![published],
            discards,
        });
        compaction_publishes.push(CompactionPublishOnTheBackgroundCompute {
            kind,
            planned_backend,
            background_compute_of_each_batch,
            calls_handed_to_the_gpu_compute,
        });
        if kind == CompactionFlowStepKind::CompactionFinalBatch {
            break;
        }
    }
    pool.output = version_of_trees(&session.current).clone();
    CompactionRelocationRecordedOnTheBackgroundCompute {
        backends,
        recorded: RecordedCompactionFlow {
            flow: CompactionCrashFlow::CompactionRelocation,
            pool,
            first_file,
            steps,
            snapshot: None,
        },
        compaction_publishes,
    }
}

/// 两条录制流逐字节比：mkfs 起块层的每一个请求（种类、盘、偏移、长度、内容摘要，连同写的内容）。一条断言把两份录制直接连起来比。
///
/// # Panics
/// 两条录制流有一处不同（报两条各多少个请求、最早不同的那个请求的下标）。
pub fn assert_the_block_sequence_is_the_one_of_the_cpu_flow(
    on_the_background_compute: &RecordedCompactionFlow,
    of_the_cpu: &RecordedCompactionFlow,
) {
    let requests_on_the_background_compute = on_the_background_compute.pool.retained_operations();
    let requests_of_the_cpu = of_the_cpu.pool.retained_operations();
    let earliest_differing_request = requests_on_the_background_compute
        .iter()
        .zip(&requests_of_the_cpu)
        .position(|(request_on_the_background_compute, request_of_the_cpu)| {
            request_on_the_background_compute != request_of_the_cpu
        });
    assert!(
        requests_on_the_background_compute == requests_of_the_cpu,
        "照计划的后端录的整理搬迁流与全程 cpu 的那一条，录下的块层请求逐字节相同：两条各 {} 与 {} 个请求，最早不同的那个请求的下标 {earliest_differing_request:?}（mkfs 起数）",
        requests_on_the_background_compute.len(),
        requests_of_the_cpu.len()
    );
}

/// A 那一版与每一步（种类、发出的那段录制流、发成的每一版、之后的 discard）逐项相同：版本表照抄全程 `cpu` 那一条的前提。
///
/// # Panics
/// 有一样不同。
pub fn assert_the_steps_publish_the_versions_of_the_cpu_flow(
    on_the_background_compute: &RecordedCompactionFlow,
    of_the_cpu: &RecordedCompactionFlow,
) {
    assert!(
        on_the_background_compute.first_file == of_the_cpu.first_file,
        "A 那一版相同"
    );
    let steps_of = |recorded: &RecordedCompactionFlow| {
        recorded
            .steps
            .iter()
            .map(|step| {
                (
                    step.kind,
                    step.operations.clone(),
                    step.versions.clone(),
                    step.discards.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    let steps_on_the_background_compute = steps_of(on_the_background_compute);
    let steps_of_the_cpu = steps_of(of_the_cpu);
    assert!(
        steps_on_the_background_compute == steps_of_the_cpu,
        "每一步的种类、发出的那段录制流、发成的每一版、之后的 discard 逐项相同：两条的步 {:?} 与 {:?}",
        steps_on_the_background_compute
            .iter()
            .map(|(kind, operations, _versions, _discards)| (*kind, operations.clone()))
            .collect::<Vec<_>>(),
        steps_of_the_cpu
            .iter()
            .map(|(kind, operations, _versions, _discards)| (*kind, operations.clone()))
            .collect::<Vec<_>>()
    );
}

/// 录一条整理搬迁流（后端照 `backends`），再录全程 `cpu` 的那一条（`common_compaction_prefix::prepare_the_compaction_crash_flow`），
/// 两条的块序列与每一步逐项比过之后摆好：起点镜像、写表、段、被判的根槽写从这一条自己的录制流算，版本表照抄全程 `cpu` 那一条。
///
/// # Panics
/// 两条的块序列或每一步有一处不同；录流本身的那几样（[`record_the_compaction_relocation_on_the_background_compute`]）。
#[must_use]
pub fn prepare_the_compaction_relocation_on_the_background_compute(
    tag: &str,
    backends: BackendsOfTheCompactionPublishes,
) -> PreparedOnTheBackgroundCompute {
    let of_the_cpu = prepare_the_compaction_crash_flow(
        &format!("{tag}-cpu-throughout"),
        CompactionCrashFlow::CompactionRelocation,
        &contents_of_the_sequential_writes(),
    );
    let CompactionRelocationRecordedOnTheBackgroundCompute {
        backends: backends_recorded,
        recorded,
        compaction_publishes,
    } = record_the_compaction_relocation_on_the_background_compute(tag, backends);
    assert_the_block_sequence_is_the_one_of_the_cpu_flow(&recorded, &of_the_cpu.recorded);
    assert_the_steps_publish_the_versions_of_the_cpu_flow(&recorded, &of_the_cpu.recorded);
    let base = recorded.pool.memory_pool_after_mkfs();
    let operations = recorded.pool.retained_operations();
    let operations_after_mkfs = operations[recorded.pool.mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_after_mkfs, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let PreparedCompactionCrashFlow {
        versions_with_a_file,
        versions_without_a_file,
        versions_of_the_flow,
        ..
    } = of_the_cpu;
    PreparedOnTheBackgroundCompute {
        backends: backends_recorded,
        prepared: PreparedCompactionCrashFlow {
            recorded,
            base,
            operations_after_mkfs,
            writes,
            segments,
            stream_indexes,
            judged_root_index,
            versions_with_a_file,
            versions_without_a_file,
            versions_of_the_flow,
        },
        compaction_publishes,
    }
}

/// 整理的三次发布（签发、一批、最后一批）各照计划的后端算：这一趟每一批开头读到的设置就是计划的后端（切换从下一批起生效）、没有一批算着出错；
/// 走 GPU 的那几趟每一批都走 GPU 路径、替身被调过、乙段（区域扫描里逐条记录的单元头解析与映射反查）确认过记录，三段一条不确定、一条不交 CPU 判
/// （替身不注入故障）；走 CPU 的那几趟每一批都因设置选 `cpu` 在 CPU 上算、替身一次都没被调。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_every_compaction_publish_ran_on_the_planned_backend(
    backends: BackendsOfTheCompactionPublishes,
    compaction_publishes: &[CompactionPublishOnTheBackgroundCompute],
) {
    assert_eq!(
        compaction_publishes
            .iter()
            .map(|publish| publish.kind)
            .collect::<Vec<_>>(),
        vec![
            CompactionFlowStepKind::CompactionIssue,
            CompactionFlowStepKind::CompactionBatch,
            CompactionFlowStepKind::CompactionFinalBatch,
        ],
        "签发 → 一批 → 最后一批"
    );
    for (publishes_recorded_before, publish) in compaction_publishes.iter().enumerate() {
        assert_eq!(
            publish.planned_backend,
            backends.backend_of_the_compaction_publish(publishes_recorded_before),
            "{:?}：录的时候照的是这一次的计划",
            publish.kind
        );
        assert!(
            !publish.background_compute_of_each_batch.is_empty(),
            "{:?}：这一趟至少一批记了后台计算的账",
            publish.kind
        );
        let mut region_records_confirmed_on_the_gpu = 0_u64;
        for batch in &publish.background_compute_of_each_batch {
            assert_eq!(
                batch.requested_backend, publish.planned_backend,
                "{:?}：这一批开头读到的设置是计划的后端（切换从下一批起生效）：{batch:?}",
                publish.kind
            );
            assert!(
                batch.compute_error_that_abandoned_the_batch.is_none(),
                "{:?}：替身不注入故障，没有一批算着出错：{batch:?}",
                publish.kind
            );
            match (&batch.path_taken, publish.planned_backend) {
                (
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
                    ),
                    BackgroundComputeBackend::Cpu,
                ) => {}
                (BackgroundComputePathTaken::Gpu(gpu_work), BackgroundComputeBackend::Gpu) => {
                    let entries = gpu_work.entries_of_the_three_segments();
                    assert_eq!(
                        (
                            entries.uncertain_this_round,
                            entries.judged_by_the_cpu_reference
                        ),
                        (0, 0),
                        "{:?}：替身不注入故障，三段一条不确定、一条不交 CPU 判：{gpu_work:?}",
                        publish.kind
                    );
                    region_records_confirmed_on_the_gpu += gpu_work
                        .region_record_classification
                        .confirmed_by_both_kernels;
                }
                (
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_),
                    ),
                    BackgroundComputeBackend::Cpu | BackgroundComputeBackend::Gpu,
                )
                | (
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
                    ),
                    BackgroundComputeBackend::Gpu,
                )
                | (BackgroundComputePathTaken::Gpu(_), BackgroundComputeBackend::Cpu) => panic!(
                    "{:?}：计划 {:?}，这一批走的路对不上：{batch:?}",
                    publish.kind, publish.planned_backend
                ),
            }
        }
        match publish.planned_backend {
            BackgroundComputeBackend::Gpu => assert!(
                publish.calls_handed_to_the_gpu_compute > 0 && region_records_confirmed_on_the_gpu > 0,
                "{:?}：走 GPU 的这一趟替身被调过（{} 次）、乙段确认过记录（{region_records_confirmed_on_the_gpu} 条）",
                publish.kind,
                publish.calls_handed_to_the_gpu_compute
            ),
            BackgroundComputeBackend::Cpu => assert_eq!(
                publish.calls_handed_to_the_gpu_compute, 0,
                "{:?}：走 CPU 的这一趟替身一次都没被调",
                publish.kind
            ),
        }
    }
}

/// 版本表：有文件的与文件从那一版起没有了的（同全程 `cpu` 的整理搬迁流）。
#[must_use]
pub fn version_table(prepared: &PreparedCompactionCrashFlow) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

/// 判器一条都没判违例：oracle 两遍恢复、走读、记录核对器两条判据、池级 checker 每条实现了的不变量。
///
/// # Panics
/// 有一样判了违例。
pub fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 流里每一种那一版的计数名（A、B 与 C、重挂、签发、一批、最后一批），每一种都要有状态。
#[must_use]
pub fn landed_counters(prepared: &PreparedCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in &prepared.versions_of_the_flow {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}
