//! 攻方腿探针（代码轮，只在副本里跑）：三格历史，打出 RESULT 行，不断言结论；SELF 行只断言前提与对照臂。
//! 格 held_back_drop_after_the_gate_opens：GPU 会话、照水位判、有活意图、根环装满；锁外那一批 GPU 实现出错，锁外期间前台推空发布推到门关，
//!   回锁出错那一支的删意图被门压住。两臂：stays_stopped（整理停下，对照臂）、switch_requested_before_the_error（出错之前用户请求切到 `cpu`，
//!   停下当场解除）。之后建 inode 到门重开（再多建 extra 个，用户决定的动作放开扫），下一次调用走后台入口或显式入口（同样放开扫），
//!   看压住的那一次删意图补没补发、还是这条意图被接着搬。
//! 格 held_back_drop_while_the_exception_holds_at_the_relock：同样的出错与门关；全空聚簇段数低于底额（用只供测试的底额开关模拟前台写满）
//!   在取输入之前就成立（对照臂）或在锁外期间才成立；回锁那一刻两臂的门与底额都一样，看删意图发没发。
//! 格 stopped_drop_at_the_trip_start_and_the_yields_in_a_row：让路一次之后，切到一张卡都用不上的设置，下一次调用一趟开头判出停着、单发删意图；
//!   看连着让路的计数清没清。对照臂：同样让路一次之后由显式入口发成一次整理发布。之后解除停下、前台一直在等，数让几次之后不让（下限 2 次）。
mod common;
mod common_background_gpu_compute;

use std::cell::Cell;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, card_named, device_lost_on_the_card_with_eight_gibibytes,
    CpuBackedBackgroundGpuComputeForTests, EIGHT_GIBIBYTES,
};
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, BackgroundTask, ClusterSegmentEntryKey,
    GpuComputedSegment, GpuDeviceIdentifier, GpuEntryDecision, GpuInventory,
    GpuLeftOutUnderItsMemoryLimit, GpuMemoryInBytes, RegionRecordToClassifyOnTheGpu,
    UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::compaction_plan::{
    empty_cluster_segments_by_the_records, foreground_writes_are_below_the_empty_cluster_segments_floor,
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading, CallerOfTheBackgroundCompaction,
    CompactionBatchSize, CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionSwitches, CompactionTimerFallback, CompactionWatermarkCrossingForcing,
    CompactionWriteBandwidthLimit, CompactionYieldFloor, EmptyClusterSegmentsFloor, UnitAreaOfOneDevice,
    WatermarkQuantity,
};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use singlefs_core::mount::{
    mount_writable_with_background_compute_settings, push_one_floor_raise_within_the_admission_budget,
    raise_rollback_floor, FloorRaisePushedWithinTheAdmissionBudget, MountError,
};
use singlefs_core::mounted_session::{
    BackgroundCompactionAfterThePendingChanges, BackgroundCompactionCall, BackgroundCompactionInputTaken,
    CompactionRun, CompactionRunOutcome, MountedSession, MountedSessionWithPendingChanges, UserChange,
};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_format::UNIT_AREA_START_SLOT;

type Devices = Vec<(DeviceIdentity, Recorded)>;

const SEGMENT_OF_THE_FIRST_DATA_UNIT: u64 = UNIT_AREA_START_SLOT;
const UNITS_OF_A_BATCH_COVERING_A_SEGMENT: usize = 64;
const USER_CHANGES_FILLING_THE_ROOT_RING: usize = 30;
/// 锁外期间至多推几串空发布：根环 3S 槽（S = 8）两圈。
const EMPTY_PUBLISH_SEQUENCES_PUSHED_AT_MOST: usize = 48;
/// 回锁之后至多建几个 inode 等门重开。
const USER_CHANGES_REOPENING_THE_GATE_AT_MOST: usize = 60;
/// 格一里门重开之后再多建几个 inode（用户决定的动作）。
const EXTRA_USER_CHANGES_SCANNED: [usize; 2] = [0, 3];

struct CallerForTests {
    clock_in_milliseconds: Cell<u64>,
    answer_before_the_next_batch: Cell<AnswerOfTheCallerBeforeTheNextBatch>,
}

impl CallerForTests {
    fn answering(answer: AnswerOfTheCallerBeforeTheNextBatch) -> Self {
        CallerForTests {
            clock_in_milliseconds: Cell::new(0),
            answer_before_the_next_batch: Cell::new(answer),
        }
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

const NOTHING_WAITING: AnswerOfTheCallerBeforeTheNextBatch =
    AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting;
const ENTRY_WAITING: AnswerOfTheCallerBeforeTheNextBatch =
    AnswerOfTheCallerBeforeTheNextBatch::ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock;

fn pacing_with_the_timer_fallback_off() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::Off,
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

fn region_starting_at(segment_start: u64) -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(segment_start))
        .expect("段起点是 64 的倍数")
}

/// 整理只供测试的开关：照水位判、greedy 选段、一批 `batch_units` 个，底额照 `floor`，其余照产品值；越线靠只供测试的开关。
fn force_the_switches_following_the_watermarks(
    pending: &mut MountedSessionWithPendingChanges,
    batch_units: usize,
    floor: EmptyClusterSegmentsFloor,
) {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: _,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    let session = pending.session_for_tests_only();
    session.allocator.set_compaction_switches(CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(batch_units).expect("至少 1"),
        ),
        forced_region: None,
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: floor,
        relocation_publish_order,
    });
    session.allocator.compaction_mut().watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
}

/// 只把底额开关换成 `floor`，别的开关照旧（模拟前台把全空聚簇段用到底额之下）。
fn set_the_empty_cluster_segments_floor(pending: &mut MountedSessionWithPendingChanges, floor: EmptyClusterSegmentsFloor) {
    let session = pending.session_for_tests_only();
    let mut switches = session.allocator.compaction().switches;
    switches.empty_cluster_segments_floor = floor;
    session.allocator.set_compaction_switches(switches);
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

fn unit_area_of_the_first_device(session: &MountedSession) -> UnitAreaOfOneDevice {
    let device_map = session.allocator.devices.first().expect("池里有盘");
    UnitAreaOfOneDevice {
        device: device_map.device,
        start: device_map.unit_area_start(),
        slots: device_map.unit_area_slots(),
    }
}

/// 重开、可写挂载（选 `gpu`、两张卡都点名），GPU 实现是 `implementation`，套上攒批的会话。
fn session_on_the_gpu_with(
    pool: &mut BuiltPool,
    implementation: Arc<Mutex<dyn BackgroundGpuCompute + Send>>,
) -> (MountedSessionWithPendingChanges, Devices) {
    let handle = BackgroundGpuComputeHandle::of_a_shared_implementation(implementation);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(handle),
    )
    .expect("选 gpu 的可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (MountedSessionWithPendingChanges::of_the_session(session), devices)
}

fn issue_an_intent_on(pending: &mut MountedSessionWithPendingChanges, devices: &mut Devices, region: CompactionRegion) -> CompactionIntent {
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
    intent
}

fn create_one_inode(pending: &mut MountedSessionWithPendingChanges, devices: &mut Devices, caller: &CallerForTests) {
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

/// 前台推一串空发布（同 `compaction_batch_computed_out_of_the_session_lock/main.rs` 的同名函数：先抬 F，F 到上限就推不抬 F 的）。
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
        let kept = match raise_rollback_floor(&publish_parameters, devices, &mut session.allocator, &mut version, floor, session.shadow_ledger) {
            Err(MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { effective, .. }) => raise_rollback_floor(
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

fn is_the_gate_closed(pending: &MountedSessionWithPendingChanges, devices: &Devices) -> bool {
    pending.session().stop_of_the_rollback_gate_now_for_tests(devices.as_slice()).is_some()
}

/// 锁外期间推空发布推到门关，交回推了几串。
fn push_until_the_gate_closes(pending: &mut MountedSessionWithPendingChanges, devices: &mut Devices) -> usize {
    let mut pushed = 0_usize;
    while !is_the_gate_closed(pending, devices) && pushed < EMPTY_PUBLISH_SEQUENCES_PUSHED_AT_MOST {
        push_one_empty_publish_sequence(pending.session_for_tests_only(), devices);
        pushed += 1;
    }
    pushed
}

fn live_intent_of(pending: &MountedSessionWithPendingChanges) -> Option<CompactionIntent> {
    live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations).map(|(_, intent)| intent)
}

fn taken_under(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    caller: &CallerForTests,
    yield_floor: CompactionYieldFloor,
) -> BackgroundCompactionInputTaken {
    pending
        .take_the_input_of_a_background_compaction_batch(devices, pacing_with_the_timer_fallback_off(), yield_floor, caller)
        .unwrap_or_else(|refused| panic!("取输入被拒：{refused:?}"))
}

/// 一次调用的那一趟（没起跑交回 `None`）与停在哪（Debug 去空格，截到 120 个字符）。
fn run_of_the_call(after: BackgroundCompactionAfterThePendingChanges) -> (Option<CompactionRun>, String) {
    match after.call {
        BackgroundCompactionCall::NotStarted { reason, .. } => (None, format!("not_started:{reason:?}")),
        BackgroundCompactionCall::Ran { run, .. } => {
            let text = outcome_text(&run);
            (Some(*run), text)
        }
    }
}

fn outcome_text(run: &CompactionRun) -> String {
    let outcome = match &run.outcome {
        CompactionRunOutcome::Stopped(stop) => format!("stopped:{stop:?}"),
        CompactionRunOutcome::Failed(failure) => format!("failed:{failure:?}"),
    };
    outcome.chars().filter(|c| !c.is_whitespace()).take(120).collect()
}

/// 只点名「card-eight」的那一组交进来之后答「参与的卡因显存上限一张都不剩」（同 `compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs` 的包法）。
struct GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
    computed_on_the_cpu: CpuBackedBackgroundGpuComputeForTests,
    settings_handed_in_last: Option<AdmittedGpuSettings>,
}

impl GpuInventory for GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.computed_on_the_cpu.devices_with_total_memory()
    }
}

impl BackgroundGpuCompute for GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        self.computed_on_the_cpu.verify_unit_checksums(units)
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        self.computed_on_the_cpu.classify_region_records(region, records, mapping_entries)
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        self.computed_on_the_cpu.count_live_slots_per_cluster_segment(allocation_records, unit_area, segments)
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.settings_handed_in_last = Some(settings.clone());
        self.computed_on_the_cpu.use_the_settings_from_the_next_batch(settings);
    }

    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        let settings = self.settings_handed_in_last.as_ref()?;
        let names_only_the_card_with_eight_gibibytes = settings
            .participating_devices()
            .iter()
            .map(|participating| participating.device.clone())
            .eq(std::iter::once(card_named("card-eight")));
        names_only_the_card_with_eight_gibibytes.then(|| BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                device: card_named("card-eight"),
                memory_limit: GpuMemoryInBytes(64),
                smallest_piece_bytes: GpuMemoryInBytes(972),
                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
            }],
        })
    }
}

fn settings_naming_only_the_card_with_eight_gibibytes() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

/// 格一、格二共用的前缀：GPU 会话（背后照 CPU 写法算的假实现）、照水位判、一批罩一整段、底额 `floor`，在单元区头一段签一条意图、根环装满不同用户状态，
/// GPU 实现分类那一段出错。交回会话、盘、那条意图、取输入那一刻门关没有。
fn prefix_with_a_live_intent_and_an_error_injected(
    pool_name: &str,
    floor: EmptyClusterSegmentsFloor,
) -> (BuiltPool, MountedSessionWithPendingChanges, Devices, CompactionIntent, CallerForTests) {
    let mut pool = build_pool(pool_name);
    let shared = Arc::new(Mutex::new(CpuBackedBackgroundGpuComputeForTests::with_nothing_injected()));
    let (mut pending, mut devices) = session_on_the_gpu_with(&mut pool, shared.clone());
    force_the_switches_following_the_watermarks(&mut pending, UNITS_OF_A_BATCH_COVERING_A_SEGMENT, EmptyClusterSegmentsFloor::OfTheFirstVersion);
    let intent = issue_an_intent_on(&mut pending, &mut devices, region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT));
    let caller = CallerForTests::answering(NOTHING_WAITING);
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    set_the_empty_cluster_segments_floor(&mut pending, floor);
    shared.lock().expect("替身没在别处 panic 过").failing_segment =
        Some((GpuComputedSegment::RegionRecordClassification, device_lost_on_the_card_with_eight_gibibytes()));
    (pool, pending, devices, intent, caller)
}

/// 取输入、锁外算（出错），交回算完的那一批；在锁里就推完了交回那一次的停因。
fn computed_out_of_the_lock(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    caller: &CallerForTests,
) -> Result<singlefs_core::mounted_session::BackgroundCompactionBatchComputed, String> {
    match taken_under(pending, devices, caller, CompactionYieldFloor::NO_FLOOR) {
        BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => Ok(ticket.compute_out_of_the_lock(&mut |reads| {
            pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
        })),
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => Err(run_of_the_call(after).1),
    }
}

/// 格一的一段历史。`switch_before_the_error`：锁外期间（出错之前已请求、还没生效）请求切到 `cpu`；`explicit_entry`：门重开之后那一次走显式入口；
/// `extra`：门重开之后再多建几个 inode。交回 RESULT 行与（回锁时压住了没有，下一次调用把那条意图接着搬了没有，那条意图补发删掉了没有）。
fn held_back_drop_after_the_gate_opens_once(switch_before_the_error: bool, explicit_entry: bool, extra: usize) -> (String, bool, bool, bool) {
    let arm = if switch_before_the_error { "switch_requested_before_the_error" } else { "stays_stopped" };
    let entry = if explicit_entry { "explicit" } else { "background" };
    let (_pool, mut pending, mut devices, intent, caller) = prefix_with_a_live_intent_and_an_error_injected(
        &format!("probe-held-back-drop-{arm}-{entry}-{extra}"),
        EmptyClusterSegmentsFloor::OfTheFirstVersion,
    );
    let gate_closed_at_the_input = is_the_gate_closed(&pending, &devices);
    let computed = match computed_out_of_the_lock(&mut pending, &mut devices, &caller) {
        Ok(computed) => computed,
        Err(outcome) => {
            return (
                format!("RESULT name=held_back_drop_after_the_gate_opens arm={arm} entry={entry} extra={extra} setup_failed=finished_in_the_lock outcome={outcome}"),
                false,
                false,
                false,
            )
        }
    };
    if switch_before_the_error {
        pending
            .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("切到 cpu 照收");
    }
    let sequences_pushed = push_until_the_gate_closes(&mut pending, &mut devices);
    let gate_closed_before_the_relock = is_the_gate_closed(&pending, &devices);
    let after_the_relock = pending
        .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份");
    let (run_at_the_relock, outcome_at_the_relock) = run_of_the_call(after_the_relock);
    let published_at_the_relock = run_at_the_relock.as_ref().map_or(0, |run| run.versions_published.len());
    let stopped_after_the_relock =
        pending.session().background_compute().is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
    let held_back = published_at_the_relock == 0 && live_intent_of(&pending) == Some(intent);
    let mut changes_until_the_gate_opens = 0_usize;
    while is_the_gate_closed(&pending, &devices) && changes_until_the_gate_opens < USER_CHANGES_REOPENING_THE_GATE_AT_MOST {
        create_one_inode(&mut pending, &mut devices, &caller);
        changes_until_the_gate_opens += 1;
    }
    for _ in 0..extra {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    let gate_closed_at_the_next_call = is_the_gate_closed(&pending, &devices);
    let live_before_the_next_call = live_intent_of(&pending) == Some(intent);
    let (next_run, next_outcome) = if explicit_entry {
        let run = pending
            .session_for_tests_only()
            .run_compaction(&mut devices)
            .expect("盘表是挂载的那一份、现行那一版有树");
        let text = outcome_text(&run);
        (Some(run), text)
    } else {
        match taken_under(&mut pending, &mut devices, &caller, CompactionYieldFloor::NO_FLOOR) {
            BackgroundCompactionInputTaken::FinishedInTheLock(after) => run_of_the_call(after),
            BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => {
                let computed = ticket.compute_out_of_the_lock(&mut |reads| {
                    pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
                });
                let after = pending
                    .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
                    .expect("回锁");
                let (run, text) = run_of_the_call(after);
                (run, format!("out_of_the_lock:{text}"))
            }
        }
    };
    let next_published = next_run.as_ref().map_or(0, |run| run.versions_published.len());
    let next_units_relocated: usize = next_run.as_ref().map_or(0, |run| {
        run.versions_published.iter().map(|version| version.units_relocated_by_this_publish.len()).sum()
    });
    let original_live_after = live_intent_of(&pending) == Some(intent);
    let carried_on = live_before_the_next_call && next_units_relocated > 0;
    let dropped = live_before_the_next_call && next_units_relocated == 0 && next_published > 0 && !original_live_after;
    (
        format!(
            "RESULT name=held_back_drop_after_the_gate_opens arm={arm} entry={entry} extra={extra} gate_closed_at_the_input={} sequences_pushed={sequences_pushed} gate_closed_before_the_relock={} published_at_the_relock={published_at_the_relock} stopped_after_the_relock={} held_back={} outcome_at_the_relock={outcome_at_the_relock} changes_until_the_gate_opens={changes_until_the_gate_opens} gate_closed_at_the_next_call={} next_published={next_published} next_units_relocated={next_units_relocated} original_intent_live_after={} carried_on={} dropped={} next_outcome={next_outcome}",
            u8::from(gate_closed_at_the_input),
            u8::from(gate_closed_before_the_relock),
            u8::from(stopped_after_the_relock),
            u8::from(held_back),
            u8::from(gate_closed_at_the_next_call),
            u8::from(original_live_after),
            u8::from(carried_on),
            u8::from(dropped),
        ),
        held_back,
        carried_on,
        dropped,
    )
}

#[test]
fn held_back_drop_after_the_gate_opens() {
    let mut histories = 0_usize;
    let mut control_held_back = 0_usize;
    let mut control_dropped = 0_usize;
    let mut switched_held_back = 0_usize;
    let mut switched_carried_on = 0_usize;
    let mut switched_dropped = 0_usize;
    for switch_before_the_error in [false, true] {
        for explicit_entry in [false, true] {
            for extra in EXTRA_USER_CHANGES_SCANNED {
                let (line, held_back, carried_on, dropped) =
                    held_back_drop_after_the_gate_opens_once(switch_before_the_error, explicit_entry, extra);
                println!("{line}");
                histories += 1;
                if switch_before_the_error {
                    switched_held_back += usize::from(held_back);
                    switched_carried_on += usize::from(held_back && carried_on);
                    switched_dropped += usize::from(held_back && dropped);
                } else {
                    control_held_back += usize::from(held_back);
                    control_dropped += usize::from(held_back && dropped);
                }
            }
        }
    }
    println!(
        "RESULT name=held_back_drop_after_the_gate_opens_summary histories={histories} control_held_back={control_held_back} control_dropped={control_dropped} switched_held_back={switched_held_back} switched_carried_on={switched_carried_on} switched_dropped={switched_dropped}"
    );
    let per_arm = histories / 2;
    let self_failed = usize::from(control_held_back != per_arm) + usize::from(control_dropped != per_arm) + usize::from(switched_held_back != per_arm);
    println!("SELF name=held_back_drop_after_the_gate_opens checked=3 failed={self_failed}");
    assert_eq!(self_failed, 0, "自检：两臂回锁时都被门压住；对照臂（停着）门开之后补发删意图");
}

/// 格二的一段历史。`floor_crossed_before_the_input`：取输入之前全空聚簇段数就低于底额（这一趟是破例那一档，对照臂）；否则锁外期间推到门关之后才低于底额。
/// 交回 RESULT 行与（回锁那一刻门关没有、底额之下没有，回锁发了几版）。
fn held_back_drop_while_the_exception_holds_at_the_relock_once(floor_crossed_before_the_input: bool) -> (String, bool, usize) {
    let arm = if floor_crossed_before_the_input { "floor_crossed_before_the_input" } else { "floor_crossed_in_the_window" };
    let below_the_floor = EmptyClusterSegmentsFloor::ForcedForTests(u64::MAX);
    let (_pool, mut pending, mut devices, intent, caller) = prefix_with_a_live_intent_and_an_error_injected(
        &format!("probe-exception-at-the-relock-{arm}"),
        if floor_crossed_before_the_input { below_the_floor } else { EmptyClusterSegmentsFloor::OfTheFirstVersion },
    );
    let gate_closed_at_the_input = is_the_gate_closed(&pending, &devices);
    let computed = match computed_out_of_the_lock(&mut pending, &mut devices, &caller) {
        Ok(computed) => computed,
        Err(outcome) => {
            return (
                format!("RESULT name=held_back_drop_while_the_exception_holds_at_the_relock arm={arm} setup_failed=finished_in_the_lock outcome={outcome}"),
                false,
                usize::MAX,
            )
        }
    };
    let sequences_pushed = push_until_the_gate_closes(&mut pending, &mut devices);
    if !floor_crossed_before_the_input {
        set_the_empty_cluster_segments_floor(&mut pending, below_the_floor);
    }
    let gate_closed_at_the_relock = is_the_gate_closed(&pending, &devices);
    let below_the_floor_at_the_relock = foreground_writes_are_below_the_empty_cluster_segments_floor(&pending.session().allocator);
    let after = pending
        .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份");
    let (run, outcome) = run_of_the_call(after);
    let published = run.as_ref().map_or(0, |run| run.versions_published.len());
    let original_live_after = live_intent_of(&pending) == Some(intent);
    (
        format!(
            "RESULT name=held_back_drop_while_the_exception_holds_at_the_relock arm={arm} gate_closed_at_the_input={} sequences_pushed={sequences_pushed} gate_closed_at_the_relock={} below_the_floor_at_the_relock={} published_at_the_relock={published} original_intent_live_after={} outcome={outcome}",
            u8::from(gate_closed_at_the_input),
            u8::from(gate_closed_at_the_relock),
            u8::from(below_the_floor_at_the_relock),
            u8::from(original_live_after),
        ),
        gate_closed_at_the_relock && below_the_floor_at_the_relock,
        published,
    )
}

#[test]
fn held_back_drop_while_the_exception_holds_at_the_relock() {
    let mut same_state_at_the_relock = 0_usize;
    let mut control_published = usize::MAX;
    let mut window_published = usize::MAX;
    for floor_crossed_before_the_input in [true, false] {
        let (line, same_state, published) = held_back_drop_while_the_exception_holds_at_the_relock_once(floor_crossed_before_the_input);
        println!("{line}");
        same_state_at_the_relock += usize::from(same_state);
        if floor_crossed_before_the_input {
            control_published = published;
        } else {
            window_published = published;
        }
    }
    println!(
        "RESULT name=held_back_drop_while_the_exception_holds_at_the_relock_summary histories=2 same_state_at_the_relock={same_state_at_the_relock} control_published={control_published} window_published={window_published}"
    );
    let self_failed = usize::from(same_state_at_the_relock != 2) + usize::from(control_published != 1);
    println!("SELF name=held_back_drop_while_the_exception_holds_at_the_relock checked=2 failed={self_failed}");
    assert_eq!(self_failed, 0, "自检：回锁那一刻两臂都门关、都在底额之下；对照臂（破例那一档）照发删意图");
}

/// 这一次取输入为前台在等会话锁的入口让了路。
fn yielded(taken: BackgroundCompactionInputTaken) -> bool {
    match taken {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            run_of_the_call(after).1 == "stopped:YieldedToAForegroundEntryWaitingForTheSessionLock"
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => false,
    }
}

/// 格三的一段历史。`drop_at_a_stop`：让路一次之后切到一张卡都用不上的设置，下一次调用（前台照 `answer_at_the_drop`）一趟开头判出停着、单发删意图；
/// 否则（对照臂）让路一次之后由显式入口发成一次整理发布。之后解除停下（对照臂不用解除），建一个 inode，前台一直在等，下限 2 次，数让几次之后不让。
/// 交回 RESULT 行与（那一次发成了没有，发成之后连着让路的次数，之后让了几次）。
fn stopped_drop_at_the_trip_start_and_the_yields_in_a_row_once(
    drop_at_a_stop: bool,
    answer_at_the_drop: AnswerOfTheCallerBeforeTheNextBatch,
) -> (String, bool, u64, usize) {
    let arm = if drop_at_a_stop { "drop_at_the_trip_start_while_stopped" } else { "publish_of_the_explicit_entry" };
    let answer = if answer_at_the_drop == NOTHING_WAITING { "nothing_waiting" } else { "entry_waiting" };
    let mut pool = build_pool(&format!("probe-stopped-drop-yields-{arm}-{answer}"));
    let implementation = Arc::new(Mutex::new(GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
        computed_on_the_cpu: CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
        settings_handed_in_last: None,
    }));
    let (mut pending, mut devices) = session_on_the_gpu_with(&mut pool, implementation);
    force_the_switches_following_the_watermarks(&mut pending, UNITS_OF_A_BATCH_COVERING_A_SEGMENT, EmptyClusterSegmentsFloor::OfTheFirstVersion);
    let intent = issue_an_intent_on(&mut pending, &mut devices, region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT));
    let caller = CallerForTests::answering(NOTHING_WAITING);
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    let floor = CompactionYieldFloor {
        yields_in_a_row_at_most: NonZeroU64::new(2),
        yielding_time_at_most: None,
    };
    caller.answer_before_the_next_batch.set(ENTRY_WAITING);
    let yielded_before = yielded(taken_under(&mut pending, &mut devices, &caller, floor));
    let yields_before = pending.session().compaction.yields_in_a_row();
    let (published, outcome) = if drop_at_a_stop {
        pending
            .switch_background_compute_from_the_next_batch(settings_naming_only_the_card_with_eight_gibibytes(), None)
            .expect("只点名 card-eight 的那一组校验照收");
        caller.answer_before_the_next_batch.set(answer_at_the_drop);
        let (run, text) = match taken_under(&mut pending, &mut devices, &caller, floor) {
            BackgroundCompactionInputTaken::FinishedInTheLock(after) => run_of_the_call(after),
            BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => (None, "batch_out_of_the_lock".to_string()),
        };
        (run.map_or(0, |run| run.versions_published.len()), text)
    } else {
        let run = pending
            .session_for_tests_only()
            .run_compaction(&mut devices)
            .expect("盘表是挂载的那一份、现行那一版有树");
        let text = outcome_text(&run);
        (run.versions_published.len(), text)
    };
    let stopped =
        pending.session().background_compute().is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
    let original_live_after = live_intent_of(&pending) == Some(intent);
    let yields_after_the_publish = pending.session().compaction.yields_in_a_row();
    if drop_at_a_stop {
        pending
            .switch_background_compute_from_the_next_batch(admissible_gpu_settings_naming_both_cards(), None)
            .expect("两张卡都点名的那一组校验照收");
    }
    create_one_inode(&mut pending, &mut devices, &caller);
    caller.answer_before_the_next_batch.set(ENTRY_WAITING);
    let mut yields_afterwards = 0_usize;
    while yields_afterwards < 4 && yielded(taken_under(&mut pending, &mut devices, &caller, floor)) {
        yields_afterwards += 1;
    }
    let stopped_at_the_end =
        pending.session().background_compute().is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
    (
        format!(
            "RESULT name=stopped_drop_at_the_trip_start_and_the_yields_in_a_row arm={arm} answer_at_the_drop={answer} yielded_before={} yields_before={yields_before} published={published} stopped_at_the_publish={} original_intent_live_after={} yields_in_a_row_after_the_publish={yields_after_the_publish} stopped_at_the_end={} yields_afterwards={yields_afterwards} outcome={outcome}",
            u8::from(yielded_before),
            u8::from(stopped),
            u8::from(original_live_after),
            u8::from(stopped_at_the_end),
        ),
        yielded_before && published >= 1 && (!drop_at_a_stop || (published == 1 && !original_live_after)),
        yields_after_the_publish,
        yields_afterwards,
    )
}

#[test]
fn stopped_drop_at_the_trip_start_and_the_yields_in_a_row() {
    let mut histories = 0_usize;
    let mut premises_held = 0_usize;
    let mut control_not_cleared = 0_usize;
    let mut drop_not_cleared = 0_usize;
    for drop_at_a_stop in [false, true] {
        for answer_at_the_drop in [NOTHING_WAITING, ENTRY_WAITING] {
            let (line, premise, yields_after_the_publish, _yields_afterwards) =
                stopped_drop_at_the_trip_start_and_the_yields_in_a_row_once(drop_at_a_stop, answer_at_the_drop);
            println!("{line}");
            histories += 1;
            premises_held += usize::from(premise);
            if drop_at_a_stop {
                drop_not_cleared += usize::from(premise && yields_after_the_publish > 0);
            } else {
                control_not_cleared += usize::from(premise && yields_after_the_publish > 0);
            }
        }
    }
    println!(
        "RESULT name=stopped_drop_at_the_trip_start_and_the_yields_in_a_row_summary histories={histories} premises_held={premises_held} control_not_cleared={control_not_cleared} drop_not_cleared={drop_not_cleared}"
    );
    let self_failed = usize::from(premises_held != histories) + usize::from(control_not_cleared > 0);
    println!("SELF name=stopped_drop_at_the_trip_start_and_the_yields_in_a_row checked=2 failed={self_failed}");
    assert_eq!(self_failed, 0, "自检：每段历史都先让一次、那一次发成（停着那一臂只发一版且活意图没了）；对照臂（显式入口发成）清零");
}
