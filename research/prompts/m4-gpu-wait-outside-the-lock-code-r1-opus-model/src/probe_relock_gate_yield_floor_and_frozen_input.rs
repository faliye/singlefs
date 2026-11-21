//! 攻方腿探针（代码轮第一轮，只在副本里跑）：三格历史，每格两臂，打出 RESULT 行，不断言结论，只在自检那几行断言两臂分得开。
//! 格一 relock_error_ignores_the_gate：有活意图、取输入时回退门开；锁外那一批 GPU 实现出错；锁外期间前台推空发布（与 accept 被空间拒时推的抬 F 同一条路）
//!   推 k 串；回锁。对照臂不注入出错。k 由用例扫（用户决定的动作放开扫）。
//! 格二 yield_floor_after_a_stop：让路一次之后，下一次调用没发布地停（回退门关）；再过 gap 毫秒、前台又在等，看这一批让不让。对照臂中间那次调用发成一次。
//! 格三 frozen_before_the_input：取输入之前就冻结着一次发布（同一 txg 一直冻到回锁），看回锁丢不丢这一批。对照臂冻结发生在锁外期间。
mod common;
mod common_background_gpu_compute;

use std::cell::Cell;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, device_lost_on_the_card_with_eight_gibibytes,
    CpuBackedBackgroundGpuComputeForTests,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::background_compute::{BackgroundGpuComputeHandle, GpuComputedSegment};
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
    BackgroundCompactionAfterThePendingChanges, BackgroundCompactionCall,
    BackgroundCompactionInputTaken, CompactionRunOutcome, MountedSession,
    MountedSessionWithPendingChanges, UserChange,
};
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_format::UNIT_AREA_START_SLOT;

type Devices = Vec<(DeviceIdentity, Recorded)>;

const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = UNIT_AREA_START_SLOT + 64;
const SEGMENT_OF_THE_FIRST_DATA_UNIT: u64 = UNIT_AREA_START_SLOT;
const UNITS_OF_A_BATCH_COVERING_A_SEGMENT: usize = 64;
const USER_CHANGES_FILLING_THE_ROOT_RING: usize = 30;
/// 格一里锁外期间至多推几串空发布：根环 3S 槽（S = 8）两圈。
const EMPTY_PUBLISH_SEQUENCES_SCANNED_AT_MOST: usize = 48;
/// 产品默认的让路下限（`crates/singlefs-mount-runtime/src/lib.rs` 的两个默认常量：8 次、5 秒）。
const PRODUCT_YIELDS_IN_A_ROW: u64 = 8;
const PRODUCT_YIELDING_TIME_IN_MILLISECONDS: u64 = 5000;
/// 格二扫的间隔（毫秒）：让路之后过多久前台又在等（用户决定的动作）。
const GAPS_SCANNED_IN_MILLISECONDS: [u64; 5] = [1000, 3000, 3999, 4000, 60_000];

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

fn force_the_empty_segments_watermark_crossed(pending: &mut MountedSessionWithPendingChanges) {
    pending
        .session_for_tests_only()
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
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

fn session_on_the_cpu_of(pool: &mut BuiltPool) -> (MountedSessionWithPendingChanges, Devices) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (MountedSessionWithPendingChanges::of_the_session(session), devices)
}

fn session_on_the_gpu_of(
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
    (MountedSessionWithPendingChanges::of_the_session(session), devices, shared)
}

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

/// 前台推一串空发布（同 `compaction_batch_computed_out_of_the_session_lock.rs` 的同名函数：先抬 F，F 到上限就推不抬 F 的）。
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

/// 此刻回退门：门关交回 (现在几个状态, 再发一次连同余量之后几个状态)，门开交回 `None`。
fn gate_now(pending: &MountedSessionWithPendingChanges, devices: &Devices) -> Option<(u64, u64)> {
    match pending
        .session()
        .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
    {
        None => None,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
            states_now,
            states_after_the_publish_and_the_margin,
        }) => Some((states_now, states_after_the_publish_and_the_margin)),
        Some(other) => panic!("回退门停在别的上：{other:?}"),
    }
}

fn taken_under(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Devices,
    caller: &CallerForTests,
    yield_floor: CompactionYieldFloor,
) -> BackgroundCompactionInputTaken {
    pending
        .take_the_input_of_a_background_compaction_batch(
            devices,
            pacing_with_the_timer_fallback_off(),
            yield_floor,
            caller,
        )
        .unwrap_or_else(|refused| panic!("取输入被拒：{refused:?}"))
}

/// 一次调用发了几版、停在哪（Debug 去空格，截到 160 个字符）。
fn outcome_of(after: &BackgroundCompactionAfterThePendingChanges) -> (usize, String) {
    let (published, outcome) = match &after.call {
        BackgroundCompactionCall::NotStarted { reason, .. } => (0, format!("not_started:{reason:?}")),
        BackgroundCompactionCall::Ran { run, .. } => (
            run.versions_published.len(),
            match &run.outcome {
                CompactionRunOutcome::Stopped(stop) => format!("stopped:{stop:?}"),
                CompactionRunOutcome::Failed(failure) => format!("failed:{failure:?}"),
            },
        ),
    };
    let squeezed: String = outcome.chars().filter(|c| !c.is_whitespace()).take(160).collect();
    (published, squeezed)
}

fn gate_text(gate: Option<(u64, u64)>) -> String {
    match gate {
        None => "open".to_string(),
        Some((now, after)) => format!("closed:{now}->{after}"),
    }
}

/// 格一的一段历史：GPU 会话、照水位判（越线靠只供测试的开关）、活意图在单元区头一段、根环装满不同用户状态；`inject_error` 时锁外那一批 GPU 实现
/// 分类那一段出错；锁外期间前台推 `k` 串空发布；回锁。交回 RESULT 行与 (回锁前门关没有, 回锁发了几版)。
fn relock_error_ignores_the_gate_once(k: usize, inject_error: bool) -> (String, bool, usize) {
    let arm = if inject_error { "gpu_error_in_the_window" } else { "no_error" };
    let mut pool = build_pool(&format!("probe-relock-error-gate-{arm}-{k}"));
    let (mut pending, mut devices, shared) = session_on_the_gpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::FollowTheWatermarks,
        None,
        UNITS_OF_A_BATCH_COVERING_A_SEGMENT,
    );
    force_the_empty_segments_watermark_crossed(&mut pending);
    issue_an_intent_on(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    let caller = CallerForTests::answering(NOTHING_WAITING);
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    let gate_at_the_input = gate_now(&pending, &devices);
    if inject_error {
        shared.lock().expect("替身没在别处 panic 过").failing_segment = Some((
            GpuComputedSegment::RegionRecordClassification,
            device_lost_on_the_card_with_eight_gibibytes(),
        ));
    }
    let ticket = match taken_under(&mut pending, &mut devices, &caller, CompactionYieldFloor::NO_FLOOR) {
        BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => ticket,
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let (published, outcome) = outcome_of(&after);
            return (
                format!("RESULT name=relock_error_ignores_the_gate arm={arm} k={k} setup_failed=finished_in_the_lock published={published} outcome={outcome}"),
                false,
                published,
            );
        }
    };
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    for _ in 0..k {
        push_one_empty_publish_sequence(pending.session_for_tests_only(), &mut devices);
    }
    let gate_before_the_relock = gate_now(&pending, &devices);
    let after = pending
        .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份、待发排空发成");
    let (published, outcome) = outcome_of(&after);
    let live_intent_after = live_compaction_intent_of_the_entries(
        &current_of(pending.session()).logged_operations,
    )
    .is_some();
    let gate_after_the_relock = gate_now(&pending, &devices);
    let published_through_a_closed_gate = gate_before_the_relock.is_some() && published > 0;
    (
        format!(
            "RESULT name=relock_error_ignores_the_gate arm={arm} k={k} gate_at_the_input={} gate_before_the_relock={} published={published} published_through_a_closed_gate={} live_intent_after={live_intent_after} gate_after_the_relock={} outcome={outcome}",
            gate_text(gate_at_the_input),
            gate_text(gate_before_the_relock),
            u8::from(published_through_a_closed_gate),
            gate_text(gate_after_the_relock),
        ),
        gate_before_the_relock.is_some(),
        published,
    )
}

/// 格一扫的 k（锁外期间前台推几串空发布）。
const K_SCANNED: [usize; 12] = [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, EMPTY_PUBLISH_SEQUENCES_SCANNED_AT_MOST];

#[test]
fn relock_error_ignores_the_gate() {
    let mut control_closed = 0_usize;
    let mut control_closed_but_published = 0_usize;
    let mut error_closed = 0_usize;
    let mut error_closed_and_published = 0_usize;
    for k in K_SCANNED {
        for inject_error in [false, true] {
            let (line, closed, published) = relock_error_ignores_the_gate_once(k, inject_error);
            println!("{line}");
            match (inject_error, closed) {
                (false, true) => {
                    control_closed += 1;
                    control_closed_but_published += usize::from(published > 0);
                }
                (true, true) => {
                    error_closed += 1;
                    error_closed_and_published += usize::from(published > 0);
                }
                (_, false) => {}
            }
        }
    }
    println!(
        "RESULT name=relock_error_ignores_the_gate_summary histories={} control_closed={control_closed} control_closed_but_published={control_closed_but_published} error_closed={error_closed} error_closed_and_published={error_closed_and_published}",
        K_SCANNED.len() * 2
    );
    let self_failed = usize::from(control_closed == 0) + usize::from(control_closed_but_published > 0);
    println!("SELF name=relock_error_ignores_the_gate checked=2 failed={self_failed}");
    assert_eq!(self_failed, 0, "自检：对照臂要有门关的历史，且门关时回锁一个都不发");
}

fn yielded(taken: &BackgroundCompactionInputTaken) -> bool {
    match taken {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            outcome_of(after).1 == "stopped:YieldedToAForegroundEntryWaitingForTheSessionLock"
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => false,
    }
}

/// 格二的一段历史：CPU 会话、照水位判、强制选第一次发布那一段、根环装满不同用户状态；钟 0 毫秒前台在等 → 取输入（产品默认下限 8 次、5 秒）；
/// 钟 1000 毫秒前台不等：`middle_publishes` 时这一批交到锁外、回锁签发（对照臂），否则先推空发布到回退门关、这一批停在门上不发；
/// 再建 30 个 inode（门重开、又「该跑了」）；钟 1000 + `gap` 毫秒前台又在等 → 取输入，看让不让。
fn yield_floor_after_a_stop_once(gap: u64, middle_publishes: bool) -> (String, bool) {
    let arm = if middle_publishes { "middle_call_publishes" } else { "middle_call_stops_at_the_gate" };
    let mut pool = build_pool(&format!("probe-yield-floor-{arm}-{gap}"));
    let (mut pending, mut devices) = session_on_the_cpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::FollowTheWatermarks,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    force_the_empty_segments_watermark_crossed(&mut pending);
    let caller = CallerForTests::answering(NOTHING_WAITING);
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    let floor = CompactionYieldFloor {
        yields_in_a_row_at_most: NonZeroU64::new(PRODUCT_YIELDS_IN_A_ROW),
        yielding_time_at_most: Some(Duration::from_millis(PRODUCT_YIELDING_TIME_IN_MILLISECONDS)),
    };
    caller.answer_before_the_next_batch.set(ENTRY_WAITING);
    let first = taken_under(&mut pending, &mut devices, &caller, floor);
    let yielded_first = yielded(&first);
    let yields_after_the_first = pending.session().compaction.yields_in_a_row();
    caller.clock_in_milliseconds.set(1000);
    caller.answer_before_the_next_batch.set(NOTHING_WAITING);
    let mut sequences_pushed = 0_usize;
    if !middle_publishes {
        while gate_now(&pending, &devices).is_none()
            && sequences_pushed < EMPTY_PUBLISH_SEQUENCES_SCANNED_AT_MOST
        {
            push_one_empty_publish_sequence(pending.session_for_tests_only(), &mut devices);
            sequences_pushed += 1;
        }
    }
    let (middle_published, middle_outcome) =
        match taken_under(&mut pending, &mut devices, &caller, floor) {
            BackgroundCompactionInputTaken::FinishedInTheLock(after) => outcome_of(&after),
            BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => {
                let computed = ticket.compute_out_of_the_lock(&mut |reads| {
                    pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
                });
                let after = pending
                    .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
                    .expect("回锁");
                outcome_of(&after)
            }
        };
    let yields_before_the_later_wait = pending.session().compaction.yields_in_a_row();
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    let gate_at_the_later_wait = gate_now(&pending, &devices);
    caller.clock_in_milliseconds.set(1000 + gap);
    caller.answer_before_the_next_batch.set(ENTRY_WAITING);
    let later = taken_under(&mut pending, &mut devices, &caller, floor);
    let yielded_later = yielded(&later);
    let later_text = match &later {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => outcome_of(after).1,
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => "batch_out_of_the_lock".to_string(),
    };
    (
        format!(
            "RESULT name=yield_floor_after_a_stop arm={arm} gap_ms={gap} yielded_first={} yields_after_the_first={yields_after_the_first} sequences_pushed={sequences_pushed} middle_published={middle_published} middle_outcome={middle_outcome} yields_before_the_later_wait={yields_before_the_later_wait} gate_at_the_later_wait={} yielded_at_the_later_wait={} later={later_text}",
            u8::from(yielded_first),
            gate_text(gate_at_the_later_wait),
            u8::from(yielded_later),
        ),
        yielded_later,
    )
}

#[test]
fn yield_floor_after_a_stop() {
    let mut control_not_yielded = 0_usize;
    let mut stopped_arm_not_yielded = 0_usize;
    for gap in GAPS_SCANNED_IN_MILLISECONDS {
        for middle_publishes in [true, false] {
            let (line, yielded_later) = yield_floor_after_a_stop_once(gap, middle_publishes);
            println!("{line}");
            if !yielded_later {
                if middle_publishes {
                    control_not_yielded += 1;
                } else {
                    stopped_arm_not_yielded += 1;
                }
            }
        }
    }
    println!(
        "RESULT name=yield_floor_after_a_stop_summary histories={} control_not_yielded={control_not_yielded} stopped_arm_not_yielded={stopped_arm_not_yielded}",
        GAPS_SCANNED_IN_MILLISECONDS.len() * 2
    );
    let self_failed = usize::from(control_not_yielded > 0);
    println!("SELF name=yield_floor_after_a_stop checked=1 failed={self_failed}");
    assert_eq!(self_failed, 0, "自检：对照臂（中间那次发成）每个间隔都要让");
}

/// 写到这一刻摆了故障就报错（报错时什么都不写）的盘：锁外期间前台那一次发布写盘报错、冻结。
struct DeviceFailingWritesWhileTheFaultIsSet {
    device: Recorded,
    fault: Arc<std::sync::atomic::AtomicBool>,
}

impl BlockDevice for DeviceFailingWritesWhileTheFaultIsSet {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.device.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        if self.fault.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("用例摆的写错"),
            ));
        }
        self.device.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.device.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<
        singlefs_core::block_device::DiscardOutcome,
        singlefs_core::block_device::BlockDeviceError,
    > {
        self.device.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.device.barrier()
    }

    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.device.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.device.size_in_bytes()
    }
}

/// 格三的一段历史：盘套上可摆写错的那一层，强制起跑、强制选第一次发布那一段、建一个 inode（「该跑了」）；`frozen_before_the_input` 时取输入之前
/// 前台那一次发布写错冻结（运行库里同一拍 `publish_when_due` 写错冻结之后接着取输入就是这一格），否则冻结发生在锁外期间（对照臂，同现有用例）；回锁。
fn frozen_before_the_input_once(frozen_before_the_input: bool) -> (String, String) {
    let arm = if frozen_before_the_input { "frozen_before_the_input" } else { "frozen_in_the_window" };
    let mut pool = build_pool(&format!("probe-frozen-{arm}"));
    let fault = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut devices: Vec<(DeviceIdentity, DeviceFailingWritesWhileTheFaultIsSet)> = pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceFailingWritesWhileTheFaultIsSet {
                    device,
                    fault: Arc::clone(&fault),
                },
            )
        })
        .collect();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pending = MountedSessionWithPendingChanges::of_the_session(session);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    let caller = CallerForTests::answering(NOTHING_WAITING);
    let mut freeze_one_publish = |pending: &mut MountedSessionWithPendingChanges,
                                  devices: &mut Vec<(DeviceIdentity, DeviceFailingWritesWhileTheFaultIsSet)>,
                                  write_time_seconds: u64| {
        fault.store(true, std::sync::atomic::Ordering::SeqCst);
        let refused = pending.publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds,
            },
            caller.clock_reading(),
        );
        fault.store(false, std::sync::atomic::Ordering::SeqCst);
        refused.is_err() && pending.session().allocator.frozen_publish().is_some()
    };
    pending
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            caller.clock_reading(),
        )
        .expect("建一个 inode");
    let mut frozen_as_planned = true;
    if frozen_before_the_input {
        frozen_as_planned &= freeze_one_publish(&mut pending, &mut devices, FIXED_WRITE_TIME_SECONDS + 1);
    }
    let taken = pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        )
        .unwrap_or_else(|refused| panic!("取输入被拒：{refused:?}"));
    let ticket = match taken {
        BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => ticket,
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let (published, outcome) = outcome_of(&after);
            return (
                format!("RESULT name=frozen_before_the_input arm={arm} batch_out_of_the_lock=0 published={published} outcome={outcome}"),
                outcome,
            );
        }
    };
    if !frozen_before_the_input {
        frozen_as_planned &= freeze_one_publish(&mut pending, &mut devices, FIXED_WRITE_TIME_SECONDS + 1);
    }
    let frozen_at_the_relock = pending.session().allocator.frozen_publish().is_some();
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    let after = pending
        .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份");
    let (published, outcome) = outcome_of(&after);
    let dropped = pending
        .session()
        .compaction
        .observations
        .batches_dropped_while_a_publish_is_frozen_at_the_relock;
    (
        format!(
            "RESULT name=frozen_before_the_input arm={arm} frozen_as_planned={} batch_out_of_the_lock=1 frozen_at_the_relock={} published={published} batches_dropped_while_frozen={dropped} trip_under_way={} outcome={outcome}",
            u8::from(frozen_as_planned),
            u8::from(frozen_at_the_relock),
            u8::from(pending.session().compaction.has_a_background_trip_under_way()),
        ),
        outcome,
    )
}

#[test]
fn frozen_before_the_input() {
    let mut control_dropped = false;
    for frozen_before_the_input in [false, true] {
        let (line, outcome) = frozen_before_the_input_once(frozen_before_the_input);
        println!("{line}");
        if !frozen_before_the_input {
            control_dropped = outcome == "stopped:BatchDroppedWhileAPublishIsFrozenAtTheRelock";
        }
    }
    let self_failed = usize::from(!control_dropped);
    println!("SELF name=frozen_before_the_input checked=1 failed={self_failed}");
    assert_eq!(self_failed, 0, "自检：对照臂（锁外期间冻结）回锁丢这一批");
}
