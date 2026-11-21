//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification
//! 后台整理一批拆成三段（D26（后台整理与放置回收） 已定项 13 第 5、6 条）：会话锁里取输入（问让路、排空待发、开这一批、现行那一版只拷 `Arc` 指针，
//! 有活意图时把 R 里的单元读进票），锁外算（选段那一圈要读盘上的一段时回会话锁里读），回会话锁里发布（出错先于一切；冻结着就丢这一批；
//! 先排空待发；现行那一版变了重判回退门；重取隔离集；有活意图那一支比 R 里未释放记录的整条记录与集合、剔掉变了的单元；签发那一支在现行那一版上
//! 重判资格与签发前那一判、意图的量取现行那一版的）。会话上记有一批在锁外时，第二个后台调用者在取 GPU 实现那把锁与换设置之前就拒。
//! 让路：前台有 fsync 之外的入口在等会话锁也让，连着让路 N 次或累计 T 毫秒下一批就不让，一批发成、或取输入没让路，两个都清零。
//! 池照 `common::build_pool` 起、可写挂载、套上攒批的会话；整理照只供测试的开关强制起跑、强制选段与批大小。锁外期间的前台改动经攒批会话的入口发。
//! 池级 checker 那几判：录制流施加到内存镜像上，这张镜像作一张给定状态表交崩溃点重放的给定状态引擎两段判（表先写进这一判自己那一份一次性的库、
//! 从库里读回再判）。带这几判的场景各写两条用例，搭建与断言是同一个带判器参数的函数，两条只差判在哪：不带后缀的那一条照调用方的多机配置判
//! （配了 GPU 只由 GPU 判器判、没配只由 CPU 判器判），名字以 `_on_the_cpu_judge` 收尾的那一条在只开 CPU 的多机配置（`ENABLE_GPU=0`）下由 CPU 判器判；
//! 违例细节只有 CPU 判器给。照调用方配置的那一条走 GPU 而这个进程拿不到卡（没编进 gpu 特性，或编进了而服务没给卡：任务不拿卡）记「本次未跑」；
//! CPU 判器那一条不打「本次未跑」，照常判、照常核，`crates/mutations.tsv` 里点这几个场景的变异行点名的是它（证红脚本与变异门禁见到点名的测试打了
//! 「本次未跑」就整条用例记本次未跑）。
#[path = "../common/mod.rs"]
mod common;
#[path = "../common_background_gpu_compute/mod.rs"]
mod common_background_gpu_compute;

use std::cell::Cell;
use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, device_lost_on_the_card_with_eight_gibibytes,
    CpuBackedBackgroundGpuComputeForTests,
};
use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::allocator::AllocationRecordQuarantineMark;
use singlefs_core::background_compute::{
    BackgroundComputeSettings, BackgroundGcCpuCompensationThreshold, BackgroundGpuComputeHandle,
    BackgroundTask, GpuComputedSegment,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::compaction_plan::{
    empty_cluster_segments_by_the_records, AnswerOfTheCallerBeforeTheNextBatch,
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionSwitches, CompactionTimerFallback, CompactionWatermarkCrossingForcing,
    CompactionWriteBandwidthLimit, CompactionYieldFloor, UnitAreaOfOneDevice, WatermarkQuantity,
};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_background_compute_settings,
    push_one_floor_raise_within_the_admission_budget, raise_rollback_floor,
    FloorRaisePushedWithinTheAdmissionBudget, MountError,
};
use singlefs_core::mounted_session::{
    BackgroundCompactionAfterThePendingChanges, BackgroundCompactionBatchComputed,
    BackgroundCompactionBatchTicket, BackgroundCompactionCall, BackgroundCompactionInputRefused,
    BackgroundCompactionInputTaken, BackgroundCompactionNotStarted,
    BackgroundCompactionRelockRefused, CompactionRun, CompactionRunOutcome, MountedSession,
    MountedSessionWithPendingChanges, UserChange,
};
use singlefs_core::region_scan::{unit_reads_of_the_region_scan, RegionScanRefusal};
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::UNIT_AREA_START_SLOT;
use singlefs_verification::crash_amplification::{
    multi_host_configuration_of_this_process, one_test_changes_the_environment_at_a_time,
    CrashAmplificationEnvironment, GivenStatesEngine, GivenStatesItem, JudgedBy,
};
use singlefs_verification::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_verification::crash_library::{
    multi_host_configuration_on_the_cpu_keeping_the_prune_server_of, DisposableLibrary, RunStage,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::{
    cards_by_priority_from, multi_host_addresses_of_the_caller, CardLeaseChannelOfTheCaller,
    GpuQuiesce, CARDS_BY_PRIORITY_KEY, MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_memory_ledger::gpu_not_run_line;

mod compute_errors_and_the_rollback_gate_closed_in_the_window;
mod entries_while_a_batch_is_out_of_the_lock;
mod issues_at_the_relock;
mod publishes_frozen_and_drains_refused_at_the_relock;
mod relock_after_a_foreground_change_in_the_window;
mod yields_to_a_foreground_entry_waiting_for_the_session_lock;

type Devices = Vec<(DeviceIdentity, Recorded)>;

/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = UNIT_AREA_START_SLOT + 64;

/// 单元区头一段：实例表（钉住）与第一个文件的数据单元（50180–50181）都在这里。
const SEGMENT_OF_THE_FIRST_DATA_UNIT: u64 = UNIT_AREA_START_SLOT;

/// 一批的单元数罩得住一整段：计划把 R 里能搬的都放进同一批。
const UNITS_OF_A_BATCH_COVERING_A_SEGMENT: usize = 64;

/// 三段首尾相接与锁里一口气推完那一条比的调用次数上限。
const CALLS_COMPARED_AT_MOST: usize = 48;

/// 另一次挂载里顺序写第一个文件的数据单元数：重挂之后数据单元最多的那一段里有几十个两槽的数据单元。
const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;

/// 回锁那一刻门关的那一格：锁外期间至多推几串空发布（抬 F 的，F 到上限之后不抬 F 的）。根环 3S 槽（S = 8）两圈。
const EMPTY_PUBLISH_SEQUENCES_PUSHED_IN_THE_WINDOW_AT_MOST: usize = 48;

/// 改坏的那一份还被回退候选集里的一条根引用着：池级 checker 判红的就是这三条（校验和、近 K 代根自洽、近 K 代块未被复用）。
const INVARIANTS_RED_ON_THE_COPY_CORRUPTED_UNDER_A_CANDIDATE_ROOT: [&str; 3] =
    ["I-2.1", "I-4.8", "I-7.4"];

/// 门开着、根环里装满不同用户状态：取输入之前建几次 inode。
const USER_CHANGES_FILLING_THE_ROOT_RING: usize = 30;

/// 给定状态引擎里这一判的项名的头（再接 `::` 与表名；项名只进行里报的名字与这一判自己那一份一次性库的标签，块键只由表的内容定）。
const ITEM_NAME_OF_THE_POOL_CHECKER_JUDGEMENTS: &str =
    "compaction-batch-computed-out-of-the-session-lock";

/// 只开 CPU 的那一份多机配置在这一判自己那一份一次性库的临时目录里的文件名。
const FILE_NAME_OF_THE_CPU_ONLY_MULTI_HOST_CONFIGURATION: &str = "multi-host-on-the-cpu.env";

/// 池级 checker 的一判由哪一种判器判（两种都是两段：表先录进这一判自己那一份一次性的库、判段从库里读回再判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JudgeOfAPoolCheckerJudgement {
    /// 照调用方的多机配置：配了 GPU 只由 GPU 判器判（这份二进制没编进 gpu 特性、编进了而服务没给卡、这张表归别的机器判时打「本次未跑」，
    /// 判归 GPU 服务那一趟），没配只由 CPU 判器判。
    JudgeOfTheMultiHostConfigurationOfTheCaller,
    /// 只开 CPU 的多机配置（`ENABLE_GPU=0`，`crash_library::multi_host_configuration_on_the_cpu_keeping_the_prune_server_of`）下由 CPU 判器判：
    /// 不跨机、不上卡，录入段与判段同进程先后跑的那一趟在这台判出结局。
    CpuJudgeUnderACpuOnlyMultiHostConfiguration,
}

impl JudgeOfAPoolCheckerJudgement {
    /// 表名（与项名）接在调用方给的表名后面的那一截：同一个场景的两条用例各在一种判器上判同一张镜像，行里报的名字分得开。
    fn table_name_suffix(self) -> &'static str {
        match self {
            Self::JudgeOfTheMultiHostConfigurationOfTheCaller => "",
            Self::CpuJudgeUnderACpuOnlyMultiHostConfiguration => "-on-the-cpu-judge",
        }
    }
}

/// 后台入口的调用方：钟读数与这一批之前的回答都由用例摆。
struct CallerForTests {
    clock_in_milliseconds: Cell<u64>,
    answer_before_the_next_batch: Cell<AnswerOfTheCallerBeforeTheNextBatch>,
}

impl CallerForTests {
    fn answering(answer_before_the_next_batch: AnswerOfTheCallerBeforeTheNextBatch) -> Self {
        CallerForTests {
            clock_in_milliseconds: Cell::new(0),
            answer_before_the_next_batch: Cell::new(answer_before_the_next_batch),
        }
    }

    fn with_nothing_waiting() -> Self {
        Self::answering(AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting)
    }
}

impl CallerOfTheBackgroundCompaction for CallerForTests {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(self.clock_in_milliseconds.get()))
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        self.answer_before_the_next_batch.get()
    }
}

fn pacing_with_the_timer_fallback_off() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::Off,
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

fn pacing_with_a_timer_fallback_every_second() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(
            NonZeroU64::new(1000).expect("不是 0"),
        ),
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

fn region_starting_at(segment_start: u64) -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(segment_start))
        .expect("段起点是 64 的倍数")
}

/// 重开、可写挂载（产品默认 `cpu`）、套上攒批的会话。
fn session_with_pending_changes_of(
    pool: &mut BuiltPool,
) -> (MountedSessionWithPendingChanges, Devices) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (
        MountedSessionWithPendingChanges::of_the_session(session),
        devices,
    )
}

/// 重开、可写挂载（选 `gpu`，GPU 实现是背后照 CPU 写法算的假实现）、套上攒批的会话。交回会话、盘与用例留着的那一份实现。
fn session_with_pending_changes_on_the_gpu_of(
    pool: &mut BuiltPool,
) -> (
    MountedSessionWithPendingChanges,
    Devices,
    Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
) {
    let shared = Arc::new(Mutex::new(
        CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
    ));
    let handle = BackgroundGpuComputeHandle::of_a_shared_implementation(shared.clone());
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(handle),
    )
    .expect("选 gpu 的可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (
        MountedSessionWithPendingChanges::of_the_session(session),
        devices,
        shared,
    )
}

/// 会话装上整理只供测试的开关：起跑照 `run_forcing`、强制选 `forced_region`（`None` 是 greedy）、一批 `batch_units` 个，其余照产品值。
fn force_the_compaction_switches(
    pending: &mut MountedSessionWithPendingChanges,
    run_forcing: CompactionRunForcing,
    forced_region: Option<CompactionRegion>,
    batch_units: usize,
) {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    pending
        .session_for_tests_only()
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(batch_units).expect("至少 1"),
            ),
            forced_region,
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
}

fn unit_area_of_the_first_device(session: &MountedSession) -> UnitAreaOfOneDevice {
    let device_map = session.allocator.devices.first().expect("池里有盘");
    UnitAreaOfOneDevice {
        device: device_map.device,
        start: device_map.unit_area_start(),
        slots: device_map.unit_area_slots(),
    }
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// 在 `region` 上签一条意图（同会话的取法：`c0` 是现行那一版的 txg，`segs0` 是那一版按记录数出的全空段数；分配器先排他 R），现行那一版换成签发那一版。
fn issue_an_intent_on(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    region: CompactionRegion,
) {
    let session = pending.session_for_tests_only();
    let previous = current_of(session).clone();
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &previous.allocation_records,
            unit_area_of_the_first_device(session),
        ),
    };
    session.allocator.compaction_mut().issue(intent);
    let publish_parameters = parameters();
    let issued = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(
            &mut writer,
            &mut session.allocator,
            &previous,
            &[],
            CompactionIntentStep::Issue(intent),
            session.instance,
        )
        .expect("签发")
    };
    session.current = PoolVersion::of_a_version_of_trees(issued);
}

/// 建一个 inode（一次用户改动，强制起跑时记下「该跑了」）。
fn create_one_inode(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    caller: &CallerForTests,
) {
    pending
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            caller.clock_reading(),
        )
        .expect("建一个 inode");
}

/// 取输入：这一批交到锁外，交回票（在锁里就走完了、被拒都 panic）。
fn ticket_taken(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    caller: &CallerForTests,
) -> Box<BackgroundCompactionBatchTicket> {
    match pending.take_the_input_of_a_background_compaction_batch(
        devices,
        pacing_with_the_timer_fallback_off(),
        CompactionYieldFloor::NO_FLOOR,
        caller,
    ) {
        Ok(BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. }) => ticket,
        Ok(BackgroundCompactionInputTaken::FinishedInTheLock(after)) => {
            panic!("这一批应当交到锁外：在锁里就走完了，{:?}", after.call)
        }
        Err(refused) => panic!("这一批应当交到锁外：取输入被拒，{refused:?}"),
    }
}

/// 锁外算：要读盘上一段时回会话里读（用例里没有真放锁，会话与盘这一刻没人动）。
fn computed_reading_in_the_session(
    ticket: BackgroundCompactionBatchTicket,
    pending: &MountedSessionWithPendingChanges,
    devices: &Devices,
) -> BackgroundCompactionBatchComputed {
    ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    })
}

fn relocked(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    computed: BackgroundCompactionBatchComputed,
    caller: &CallerForTests,
) -> BackgroundCompactionAfterThePendingChanges {
    pending
        .publish_the_batch_computed_out_of_the_lock(devices, computed, caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份、待发排空发成")
}

/// 这一次调用起跑了的那一趟。
fn run_of(after: BackgroundCompactionAfterThePendingChanges) -> CompactionRun {
    match after.call {
        BackgroundCompactionCall::Ran { run, .. } => *run,
        BackgroundCompactionCall::NotStarted { reason, .. } => {
            panic!("这一次调用应当起跑：没起跑，{reason:?}")
        }
    }
}

fn stop_of(run: &CompactionRun) -> CompactionRunStop {
    match &run.outcome {
        CompactionRunOutcome::Stopped(stop) => *stop,
        CompactionRunOutcome::Failed(failure) => panic!("这一趟应当照判据停：没发成，{failure:?}"),
    }
}

/// 这一趟各次发布搬走的单元的映射 key，按先后。
fn mapping_keys_relocated_by(run: &CompactionRun) -> Vec<Vec<u8>> {
    run.versions_published
        .iter()
        .flat_map(|version| version.units_relocated_by_this_publish.iter())
        .map(|relocated| relocated.mapping_key.clone())
        .collect()
}

/// 照调用方的多机配置那一判上卡会不会被拒：这份二进制编进了 gpu 特性、调用方的多机配置开了 GPU（引擎照它只走 GPU 判器），
/// 而服务既没交卡串（`GPU_CARDS_BY_PRIORITY`）也没交领卡通道（任务不拿卡）时，引擎第一次在卡上判表那一刻取卡照
/// `gpu_card_configuration::cards_by_priority_from` 报错；这里拿同样的输入先问它一遍，交回它报的那一句（调用方打「本次未跑」、不起引擎）。
/// 别的情形交 `None`、照常起引擎：没编进 gpu 特性的引擎自己打「本次未跑」、交占位结论；交了卡串或领卡通道的照它取卡，取不到照旧判红。
/// 在改环境的那把锁里读（引擎起步也在这把锁里读这几样），读到的是调用方的环境、不是别的用例环境守卫设的。
///
/// # Panics
/// 调用方的多机配置读不了。
fn reason_the_gpu_judge_of_the_caller_gets_no_card(item_name: &str) -> Option<String> {
    let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
    let configuration_of_the_caller = multi_host_configuration_of_this_process()
        .unwrap_or_else(|refusal| panic!("{item_name}：{refusal}"));
    let the_engine_judges_on_a_card =
        cfg!(feature = "gpu") && configuration_of_the_caller.mode.uses_gpu();
    if !the_engine_judges_on_a_card {
        return None;
    }
    let cards_handed_out_by_the_service = std::env::var(CARDS_BY_PRIORITY_KEY).ok();
    if cards_handed_out_by_the_service.is_some()
        || CardLeaseChannelOfTheCaller::read_from_the_environment_now().is_some()
    {
        return None;
    }
    cards_by_priority_from(
        cards_handed_out_by_the_service,
        Some(configuration_of_the_caller.configuration_file.as_path()),
        GpuQuiesce::of_this_process(),
        &multi_host_addresses_of_the_caller,
    )
    .err()
}

/// 录制流施加到内存镜像上，这张镜像建成一张给定状态表（基线是它、写表为空、一个空的持久集合：池级 checker 判的就是这张镜像本身），
/// 交崩溃点重放的给定状态引擎两段判：表先写进这一判自己那一份一次性的库（`DisposableLibrary`，结论都是这一趟现判的）、判段从库里读回再判；
/// 判器照 `judge`：照调用方的多机配置（配了 GPU 只由 GPU 判器判、没配只由 CPU 判器判），或在只开 CPU 的多机配置下由 CPU 判器判
/// （那一份写进一次性库的临时目录，起引擎时用环境守卫把 `SINGLEFS_MULTI_HOST_CONFIG` 指过去，一趟读的别的变量清掉）。
/// 交回这一个状态的结论与判它的那一个判器。
/// 这一判不作数打「本次未跑」那一行、交 `None`：录入段与判段分进程跑（两种判器都是）；照调用方配置的那一种另有两台分片而这张表归别的机器判、
/// 配了 GPU 而这份二进制没编进 gpu 特性、编进了而服务没给卡（[`reason_the_gpu_judge_of_the_caller_gets_no_card`]，不起引擎）。
///
/// # Panics
/// 判不了（判定库读写失败、服务给了卡串或领卡通道而卡用不上），不退回别处判；只开 CPU 的那一种没在这台由 CPU 判器判出结局。
fn pool_checker_conclusion_of(
    pool: &BuiltPool,
    table_name_of_the_caller: &str,
    judge: JudgeOfAPoolCheckerJudgement,
) -> Option<(GivenStateConclusion, JudgedBy)> {
    let table_name = format!("{table_name_of_the_caller}{}", judge.table_name_suffix());
    let item_name = format!("{ITEM_NAME_OF_THE_POOL_CHECKER_JUDGEMENTS}::{table_name}");
    // 选段在改环境的那把锁里读：同一个二进制里别的用例拿着环境守卫时清掉了选段那个变量，锁外读到的不是调用方的
    let stage_of_the_caller = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        RunStage::of_this_process()
    };
    match stage_of_the_caller {
        RunStage::RecordThenJudge => {}
        stage @ (RunStage::RecordOnly | RunStage::JudgeOnly) => {
            println!(
                "{}",
                gpu_not_run_line(
                    &item_name,
                    &format!(
                        "这一判在自己那一份一次性的库里录、判，录入段与判段分进程跑（这一趟只跑 {}）时两段拿不到同一份库",
                        stage.name()
                    )
                )
            );
            return None;
        }
    }
    if judge == JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller {
        if let Some(refusal) = reason_the_gpu_judge_of_the_caller_gets_no_card(&item_name) {
            println!(
                "{}",
                gpu_not_run_line(
                    &item_name,
                    &format!(
                        "照调用方的多机配置这一判由 GPU 判器判，服务没给这个进程卡（任务不拿卡，判归 GPU 服务那一趟）：{refusal}"
                    )
                )
            );
            return None;
        }
    }
    let library = DisposableLibrary::new(&item_name)
        .unwrap_or_else(|reason| panic!("{item_name} 自己那一份一次性的库建不了：{reason}"));
    let item = GivenStatesItem {
        item_name: item_name.clone(),
    };
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    let engine = {
        // 先设一次性库、再拿改环境的那把锁（全仓用例都是这个次序，两把锁不会互相等）；引擎起步时把要读的都读完，起完就放
        let _the_library_of_this_process = library.make_it_the_library_of_this_process();
        let started = match judge {
            JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller => {
                let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
                GivenStatesEngine::start_from_environment(item, &toolchain)
            }
            JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration => {
                // 调用方的多机配置在锁里、守卫清掉 SINGLEFS_MULTI_HOST_CONFIG 之前取（仓副本里只经这个变量交进来），只为照抄它那一处 prune-server
                let configuration_of_the_caller = {
                    let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
                    multi_host_configuration_of_this_process()
                        .unwrap_or_else(|refusal| panic!("{item_name}：{refusal}"))
                        .configuration_file
                };
                let configuration_on_the_cpu =
                    library.join(FILE_NAME_OF_THE_CPU_ONLY_MULTI_HOST_CONFIGURATION);
                std::fs::write(
                    &configuration_on_the_cpu,
                    multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
                        &configuration_of_the_caller,
                    )
                    .unwrap_or_else(|refusal| panic!("{item_name}：{refusal}")),
                )
                .expect("这一判自己那一份一次性库的临时目录里写得了只开 CPU 的多机配置");
                let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
                    MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
                    configuration_on_the_cpu.as_os_str(),
                )]);
                GivenStatesEngine::start_from_environment(item, &toolchain)
            }
        };
        started.unwrap_or_else(|refusal| panic!("{item_name}：起不了给定状态表的引擎：{refusal}"))
    };
    let image = pool.memory_pool();
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name,
            base: &image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| panic!("{item_name}：给定状态表判不了（不退回别处判）：{reason}"));
    let run = engine.finish();
    assert_eq!(
        run.red_items(),
        Vec::<String>::new(),
        "{item_name}：GPU 判不了的状态判红"
    );
    match (judged.source, judge) {
        (
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
            JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
        ) => {}
        (
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
            JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
        ) => assert_eq!(
            run.judged_by,
            JudgedBy::CpuJudge,
            "{item_name}：只开 CPU 的多机配置下由 CPU 判器判"
        ),
        (
            source @ (GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly),
            JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
        ) => panic!(
            "{item_name}：只开 CPU 的多机配置下不跨机、不上卡，这张表该在这台由 CPU 判器判出结局，交回的却是占位结论（{source:?}）"
        ),
        (
            source @ (GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly),
            JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
        ) => {
            println!(
                "{}",
                gpu_not_run_line(
                    &item_name,
                    &format!("这张表这一趟没在这台判（{source:?}），交的是占位结论")
                )
            );
            return None;
        }
    }
    assert_eq!(
        judged.states.len(),
        1,
        "{item_name}：一张表一个状态，判器逐状态交回结论"
    );
    let conclusion = judged
        .states
        .into_iter()
        .next()
        .expect("上一句核过恰好一个状态");
    Some((conclusion, run.judged_by))
}

/// 池级 checker 在 `pool` 这张镜像上判红的不变量（[`pool_checker_conclusion_of`] 在 `judge` 上两段判）；这一判不作数交 `None`。
fn invariants_violated_on(
    pool: &BuiltPool,
    table_name: &str,
    judge: JudgeOfAPoolCheckerJudgement,
) -> Option<Vec<&'static str>> {
    let (conclusion, _judged_by) = pool_checker_conclusion_of(pool, table_name, judge)?;
    Some(
        conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(name, _detail)| name)
            .collect(),
    )
}

/// 池级 checker 在 `pool` 这张镜像上判红的不变量与它点名的头一处违例（[`pool_checker_conclusion_of`] 在 `judge` 上两段判）：违例细节只有 CPU 判器给，
/// GPU 判器判的那一份细节是 `None`（GPU 判器只交违例位）。这一判不作数交 `None`。
fn violations_on(
    pool: &BuiltPool,
    table_name: &str,
    judge: JudgeOfAPoolCheckerJudgement,
) -> Option<Vec<(&'static str, Option<String>)>> {
    let (conclusion, judged_by) = pool_checker_conclusion_of(pool, table_name, judge)?;
    Some(
        conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(name, detail)| match judged_by {
                JudgedBy::CpuJudge => (name, Some(detail)),
                JudgedBy::GpuJudge => (name, None),
            })
            .collect(),
    )
}

/// 第一个文件此刻读回的整份内容。
fn first_file_read_back(pending: &MountedSessionWithPendingChanges, devices: &Devices) -> Vec<u8> {
    let size = current_of(pending.session())
        .inode_record
        .expect("第一个文件那条记录")
        .size;
    pending
        .session()
        .read_the_file(
            devices.as_slice(),
            InodeNumber(FIRST_INODE_NUMBER),
            FileOffsetInBytes(0),
            size,
        )
        .expect("读得出第一个文件")
        .bytes
}

/// 改坏一块盘上某个槽起的那一份单元：翻它第 4096 字节起那个扇区的第一个字节（数据单元与索引节点都罩得住这一处），位置项里的校验和从此对不上
/// （同 `common_images_for_the_given_states_engine/release_checksum_quarantine.rs` 的造法）。写经录制器落盘。
fn corrupt_the_copy_on(devices: &mut Devices, device: DeviceIdentity, slot: SlotNumber) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let sector_offset = DeviceOffsetInBytes(slot.to_device_offset().0 + 4096);
    let mut sector = vec![0u8; 512];
    block_device
        .read_at(sector_offset, &mut sector)
        .expect("读那一个扇区");
    sector[0] ^= 0xff;
    block_device
        .write_at(sector_offset, &sector, WriteDurability::Plain)
        .expect("写回改坏的扇区");
}

/// 一次调用的结局（比两条路用）：没起跑的原因，或起跑那一趟为什么起跑、停在哪、发了几版、搬了几批。
fn outcome_of_the_call(call: &BackgroundCompactionCall) -> String {
    match call {
        BackgroundCompactionCall::NotStarted { reason, .. } => format!("not started {reason:?}"),
        BackgroundCompactionCall::Ran { trigger, run } => format!(
            "{trigger:?} {:?} versions {} batches {}",
            run.outcome,
            run.versions_published.len(),
            run.batches_published
        ),
    }
}

/// 起池、可写挂载、强制起跑、greedy 选段、一批一个、建一个 inode（「该跑了」）。
fn pool_due_after_a_user_change(
    tag: &str,
) -> (BuiltPool, MountedSessionWithPendingChanges, Devices) {
    let mut pool = build_pool(tag);
    let (mut pending, mut devices) = session_with_pending_changes_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        None,
        1,
    );
    create_one_inode(
        &mut pending,
        &mut devices,
        &CallerForTests::with_nothing_waiting(),
    );
    (pool, pending, devices)
}

/// 起池、可写挂载（`cpu`）、强制起跑、强制选第一个文件那次发布的段、一批一个、建一个 inode：取输入交到锁外的是签发那一批。
fn pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
    tag: &str,
) -> (BuiltPool, MountedSessionWithPendingChanges, Devices) {
    let mut pool = build_pool(tag);
    let (mut pending, mut devices) = session_with_pending_changes_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    create_one_inode(
        &mut pending,
        &mut devices,
        &CallerForTests::with_nothing_waiting(),
    );
    (pool, pending, devices)
}

/// 锁外期间前台做的那一种改动。
#[derive(Clone, Copy, Debug)]
enum ForegroundChangeInTheWindow {
    /// 覆盖写第一个文件：它的数据单元在 R 里，换下就释放。
    OverwriteOfTheFirstFile,
    /// 顺序写另一个 inode（取输入之前建的那一个）：新单元不落进 R（R 排他），R 里的记录不变。
    WriteOfAnotherInode,
    /// 建一个 inode：R 里的记录不变。
    NewInode,
    /// 删第一个文件：它在 R 里的数据单元释放。
    UnlinkOfTheFirstFile,
}

fn make_the_foreground_change(
    change: ForegroundChangeInTheWindow,
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    caller: &CallerForTests,
) {
    let content_of_the_overwrite = vec![0x5Au8; 2500];
    let content_of_the_other_inode = vec![0xA5u8; data_unit_payload_capacity()];
    let user_change = match change {
        ForegroundChangeInTheWindow::OverwriteOfTheFirstFile => UserChange::Overwrite(FirstFile {
            content: &content_of_the_overwrite,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        }),
        ForegroundChangeInTheWindow::WriteOfAnotherInode => UserChange::SequentialWriteOfAnInode {
            inode: InodeNumber(FIRST_INODE_NUMBER + 1),
            content: &content_of_the_other_inode,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        ForegroundChangeInTheWindow::NewInode => UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        ForegroundChangeInTheWindow::UnlinkOfTheFirstFile => UserChange::Unlink {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
    };
    pending
        .publish_user_change(devices, user_change, caller.clock_reading())
        .unwrap_or_else(|refused| panic!("前台改动 {change:?} 发成：{refused:?}"));
}

/// 前台推一串空发布、现行那一版换成这一串最后那一版：先照准入那一条抬 F 到上限（这次准入已推 0 次）；抬不动（F 已在上限）就推一串
/// 不抬 F 的空发布（给现行的 F，低于盘上现算的生效值时给生效值）。同一个用户状态的空发布挤掉根环里旧的用户状态，推几串门就关。
fn push_one_empty_publish_sequence(session: &mut MountedSession, devices: &mut Devices) {
    let publish_parameters = parameters();
    let mut version = current_of(session).clone();
    let raised = push_one_floor_raise_within_the_admission_budget(
        &publish_parameters,
        devices,
        &mut session.allocator,
        &mut version,
        session.shadow_ledger,
        0,
    )
    .expect("推抬 F 不报错");
    if let FloorRaisePushedWithinTheAdmissionBudget::Stopped(_floor_at_the_ceiling) = raised {
        let floor = version.root.rollback_floor;
        let kept = match raise_rollback_floor(
            &publish_parameters,
            devices,
            &mut session.allocator,
            &mut version,
            floor,
            session.shadow_ledger,
        ) {
            Err(MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                effective,
                ..
            }) => raise_rollback_floor(
                &publish_parameters,
                devices,
                &mut session.allocator,
                &mut version,
                effective,
                session.shadow_ledger,
            ),
            other => other,
        };
        kept.unwrap_or_else(|error| panic!("F 在上限时推不抬 F 的空发布不报错：{error:?}"));
    }
    session.current = PoolVersion::of_a_version_of_trees(version);
}
