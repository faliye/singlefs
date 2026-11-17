//! 攻方腿探针（m4-gpugc-mount-runtime-code-r1，模型文件，不入产品）：拷进冻结副本 `crates/singlefs-mount-runtime/tests/` 跑。
//! 每条探针只打 `name=` 开头的读数，不断言结论：结果反过来也照样打出来，由判决读。
//! 崩溃只取发布边界（丢运行库、盘原样留着）；段内写的子集不枚举，崩溃状态枚举归崩溃点重放（报告「这条腿自己的限度」）。
mod common_pool_on_memory_devices;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount,
    pool_with_holes_in_the_data_segments, set_root_ring_slots_of_the_probe, PoolOnMemoryDevices,
    RecordedMemoryDevice, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionBatchUnits, CompactionMountParameters,
    CompactionRunStop, CompactionSwitches, CompactionWatermarks, EmptyClusterSegmentsFloor,
    EmptyClusterSegmentsWatermark, FragmentationWatermark, UnreclaimableBytesWatermark,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::transaction::{
    RelocatedUnitHolder, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, MaintenanceEvent, MonotonicClockOfThisProcess, MountedRuntime,
    ReceiverOfTheMaintenanceEvents, RuntimeMounted, RuntimeParameters, UnmountRefused,
};

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
const PASSES_AT_MOST: usize = 8;
const MARGIN_OF_THE_GATE: u64 = 8;
const USER_STATES_OF_THE_HISTORIES: usize = 6;
/// 起线程的探针每一臂量多久（挂钟）。
const MEASURING: Duration = Duration::from_secs(2);
/// 周期压到这么短：量的是前台与线程在锁上的交错，不是周期。
const SHORT_PERIOD: Duration = Duration::from_millis(1);
const DEADLINE_OF_A_TRIAL: Duration = Duration::from_secs(2);

type DevicesOfTheProbe = Vec<(DeviceIdentity, RecordedMemoryDevice)>;

/// 探针之间不交错：根环几何放在一个静态量里，起线程的探针量的是锁上的交错。
static ONE_PROBE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn only_this_probe() -> MutexGuard<'static, ()> {
    ONE_PROBE_AT_A_TIME
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

#[derive(Debug, Default)]
struct ClockMovedByTheProbe {
    reading: Mutex<Duration>,
}

impl ClockMovedByTheProbe {
    fn advance_by(&self, elapsed: Duration) {
        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
    }
}

impl ClockOfTheRuntime for ClockMovedByTheProbe {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
    }
}

fn relocates_a_user_visible_root(version: &TransactionOutput) -> bool {
    version
        .units_relocated_by_this_publish
        .iter()
        .any(|relocated| match relocated.holder {
            RelocatedUnitHolder::RoleOfThisVersion(role) => {
                matches!(
                    role,
                    TransactionUnit::InodeRoot | TransactionUnit::ExtentRoot
                )
            }
            RelocatedUnitHolder::KeptForTheSnapshots => false,
        })
}

/// 数维护线程送出来的事件。
#[derive(Debug, Default)]
struct CountingTheEvents {
    publishes: AtomicU64,
    root_relocating_publishes: AtomicU64,
    yields_to_a_fsync: AtomicU64,
    calls_not_started: AtomicU64,
    calls_refused: AtomicU64,
}

impl ReceiverOfTheMaintenanceEvents for CountingTheEvents {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        match event {
            MaintenanceEvent::CompactionCallReturned(call) => match call {
                BackgroundCompactionCall::NotStarted { .. } => {
                    self.calls_not_started.fetch_add(1, Ordering::SeqCst);
                }
                BackgroundCompactionCall::Ran { run, .. } => {
                    for version in &run.versions_published {
                        self.publishes.fetch_add(1, Ordering::SeqCst);
                        if relocates_a_user_visible_root(version) {
                            self.root_relocating_publishes
                                .fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    if matches!(
                        run.outcome,
                        CompactionRunOutcome::Stopped(CompactionRunStop::YieldedToAForegroundFsync)
                    ) {
                        self.yields_to_a_fsync.fetch_add(1, Ordering::SeqCst);
                    }
                }
            },
            MaintenanceEvent::CompactionCallRefused(_) => {
                self.calls_refused.fetch_add(1, Ordering::SeqCst);
            }
            MaintenanceEvent::PendingChangesPublished(_)
            | MaintenanceEvent::PublishWhenDueRefused(_)
            | MaintenanceEvent::FrozenPublishResent
            | MaintenanceEvent::FrozenPublishResendFailed(_)
            | MaintenanceEvent::MaintenanceThreadPanicked { .. } => {}
        }
    }
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关；批大小照给。
fn parameters_over_the_watermark(batch_units: usize) -> RuntimeParameters {
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction = CompactionMountParameters {
        batch_units: CompactionBatchUnits::new(batch_units).expect("至少 1"),
        watermarks: CompactionWatermarks {
            empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
                percent: 100,
                lowest_line_in_segments: 2,
            },
            fragmentation: FragmentationWatermark::Off,
            unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
        },
    };
    runtime_parameters
}

fn one_inode(write_time_seconds: u64) -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds,
    }
}

/// 在 `base` 的拷贝上，一次挂载里发 `states` 次用户改动，丢掉会话（崩溃）。交回盘与每次改动的 txg。
fn pool_with_user_states(
    base: &PoolOnMemoryDevices,
    states: usize,
) -> (PoolOnMemoryDevices, Vec<CheckpointTxg>) {
    let mut pool = base.copied_with_an_empty_stream();
    let mounted = mount_writable(&parameters(), &mut pool.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let mut txgs = Vec::new();
    for _state in 0..states {
        session
            .publish_user_change(&mut pool.devices, one_inode(FIXED_WRITE_TIME_SECONDS))
            .expect("建一个 inode");
        txgs.push(
            session
                .current
                .version_of_trees()
                .expect("有树的一版")
                .root
                .checkpoint_txg,
        );
    }
    drop(session);
    (pool, txgs)
}

fn mounted_without_a_thread(
    devices: DevicesOfTheProbe,
    runtime_parameters: RuntimeParameters,
    clock: &Arc<ClockMovedByTheProbe>,
    events: &Arc<CountingTheEvents>,
) -> MountedRuntime<RecordedMemoryDevice> {
    let clock_of_the_runtime: Arc<ClockMovedByTheProbe> = Arc::clone(clock);
    let events_of_the_runtime: Arc<CountingTheEvents> = Arc::clone(events);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        devices,
        runtime_parameters,
        None,
        clock_of_the_runtime,
        events_of_the_runtime,
    )
    .expect("可写挂载");
    runtime
}

fn mounted_on_a_thread(
    devices: DevicesOfTheProbe,
    runtime_parameters: RuntimeParameters,
    events: &Arc<CountingTheEvents>,
) -> MountedRuntime<RecordedMemoryDevice> {
    let events_of_the_runtime: Arc<CountingTheEvents> = Arc::clone(events);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        devices,
        runtime_parameters,
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        events_of_the_runtime,
    )
    .expect("可写挂载");
    runtime
}

/// 崩溃：盘从运行库里拿出来，丢掉运行库（只停线程，不写盘）。
fn devices_after_a_crash(runtime: MountedRuntime<RecordedMemoryDevice>) -> DevicesOfTheProbe {
    let devices = runtime
        .with_the_session_and_devices(|locked| std::mem::take(&mut locked.devices))
        .expect("运行库没坏");
    drop(runtime);
    devices
}

fn current_txg(runtime: &MountedRuntime<RecordedMemoryDevice>) -> CheckpointTxg {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .current
                .version_of_trees()
                .expect("有树的一版")
                .root
                .checkpoint_txg
        })
        .expect("运行库没坏")
}

fn has_a_live_intent(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .current
                .version_of_trees()
                .and_then(|version| {
                    live_compaction_intent_of_the_entries(&version.logged_operations)
                })
                .is_some()
        })
        .expect("运行库没坏")
}

fn intents_completed(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .compaction
                .observations
                .intents_completed
        })
        .expect("运行库没坏")
}

/// 根环里读得出、不低于 F、txg 不低于 `oldest_root_kept` 的根属于几个用户状态（某次用户改动的 txg 起到下一次用户改动之前算一个状态）。
/// 判法同入库用例 `compaction_rollback_gate_keeps_four_user_states.rs` 的 `user_states_among_the_roots_from`。
fn user_states_among_the_roots_from(
    devices: &[(DeviceIdentity, RecordedMemoryDevice)],
    txgs_of_the_user_changes: &[CheckpointTxg],
    oldest_root_kept: CheckpointTxg,
) -> usize {
    let pool_parameters = parameters();
    let roots = readable_roots(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let floor = effective_rollback_floor(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    txgs_of_the_user_changes
        .iter()
        .enumerate()
        .filter(|(index, txg)| {
            let next = txgs_of_the_user_changes.get(index + 1);
            roots.iter().any(|root| {
                root.checkpoint_txg >= floor
                    && root.checkpoint_txg >= oldest_root_kept
                    && root.checkpoint_txg >= **txg
                    && next.is_none_or(|next_txg| root.checkpoint_txg < *next_txg)
            })
        })
        .count()
}

fn user_states_now(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    txgs: &[CheckpointTxg],
) -> usize {
    runtime
        .with_the_session_and_devices(|locked| {
            user_states_among_the_roots_from(locked.devices.as_slice(), txgs, CheckpointTxg(0))
        })
        .expect("运行库没坏")
}

/// 从现行那一版起再发 `publishes_ahead` 次发布之后还留在根环里的用户状态（txg 为 x 的根由 x 加根环槽数那一次发布盖掉）。
fn user_states_left_after_the_next_publishes(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    txgs: &[CheckpointTxg],
    publishes_ahead: u64,
) -> usize {
    let root_ring_slots =
        ROOT_RING_REGIONS * parameters().geometry.root_ring_slots_per_region.count();
    let current = current_txg(runtime);
    let oldest_root_kept =
        CheckpointTxg((current.0 + publishes_ahead + 1).saturating_sub(root_ring_slots));
    runtime
        .with_the_session_and_devices(|locked| {
            user_states_among_the_roots_from(locked.devices.as_slice(), txgs, oldest_root_kept)
        })
        .expect("运行库没坏")
}

/// 探针强制选哪一段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegionOfTheProbe {
    /// 现行那一版 inode 根所在的整段（根在上一次挂载开的段里才选得上）。
    SegmentOfTheInodeRoot,
    /// 数据单元最多的那一段（入库回退门用例的取法）。
    FullestDataSegment,
    /// 产品的选段，不强制。
    ChosenByTheProduct,
}

fn region_of_the_probe(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    region: RegionOfTheProbe,
) -> Option<CompactionRegion> {
    runtime
        .with_the_session_and_devices(|locked| -> Option<CompactionRegion> {
            let version = locked.session.session().current.version_of_trees()?;
            match region {
                RegionOfTheProbe::ChosenByTheProduct => None,
                RegionOfTheProbe::SegmentOfTheInodeRoot => {
                    let inode_root = version
                        .units
                        .iter()
                        .find(|unit| matches!(unit.identity, TransactionUnit::InodeRoot))?;
                    CompactionRegion::of_one_cluster_segment(
                        DeviceIdentity(0),
                        SlotNumber(inode_root.slot.0 / 64 * 64),
                    )
                    .ok()
                }
                RegionOfTheProbe::FullestDataSegment => {
                    let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
                    for record in version.allocation_records.iter().filter(|record| {
                        record.device == DeviceIdentity(0)
                            && !record.is_released
                            && record.span_slots == 2
                    }) {
                        *data_units_by_segment
                            .entry(record.slot.0 / 64 * 64)
                            .or_default() += 1;
                    }
                    let (segment, _data_units) = data_units_by_segment
                        .into_iter()
                        .max_by_key(|(_segment, data_units)| *data_units)?;
                    CompactionRegion::of_one_cluster_segment(
                        DeviceIdentity(0),
                        SlotNumber(segment),
                    )
                    .ok()
                }
            }
        })
        .expect("运行库没坏")
}

fn force_the_region(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    region: Option<CompactionRegion>,
) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.forced_region = region;
            session.allocator.set_compaction_switches(switches);
        })
        .expect("运行库没坏");
}

fn is_stopped_at_the_gate(stop: Option<CompactionRunStop>) -> bool {
    matches!(
        stop,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
    )
}

/// 按产品线程体的节奏走几拍（一趟没完隔一个 P，否则隔一个周期），走到门关或走满。第一条意图做完之后不再强制选段，产品的选段接着挑。
fn passes_until_the_gate_closes(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    clock: &ClockMovedByTheProbe,
    passes_at_most: usize,
) {
    let completed_before = intents_completed(runtime);
    let mut is_the_trip_under_way = false;
    for _pass in 0..passes_at_most {
        clock.advance_by(if is_the_trip_under_way {
            ONE_POLL
        } else {
            ONE_PERIOD
        });
        let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
        is_the_trip_under_way = pass.is_a_trip_under_way;
        if intents_completed(runtime) > completed_before {
            force_the_region(runtime, None);
        }
        if is_stopped_at_the_gate(runtime.observations().last_stop) {
            break;
        }
    }
}

/// T2：门把搬根那一批数成一个状态。同一段历史（6 次用户改动、崩溃、再挂），两臂只差强制选的段：不带根的满段（对照，入库用例的取法）、带 inode 根的段。
/// 判别量：门关的那一刻再发 8 次发布之后还剩几个用户状态（入库用例 `the_gate_closes_exactly_where_the_next_eight_publishes_still_keep_four_user_states` 钉成 4）。
#[test]
fn gate_counts_a_moved_root_as_a_user_state() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for batch_units in [1_usize, 16] {
        for arm in [
            RegionOfTheProbe::FullestDataSegment,
            RegionOfTheProbe::SegmentOfTheInodeRoot,
        ] {
            let (pool, txgs) = pool_with_user_states(&base, USER_STATES_OF_THE_HISTORIES);
            let reads = pool.faults.reads.clone();
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let events = Arc::new(CountingTheEvents::default());
            let runtime = mounted_without_a_thread(
                pool.devices,
                parameters_over_the_watermark(batch_units),
                &clock,
                &events,
            );
            force_the_region(&runtime, region_of_the_probe(&runtime, arm));
            let reads_before = reads.load(Ordering::SeqCst);
            passes_until_the_gate_closes(&runtime, &clock, PASSES_AT_MOST);
            let observations = runtime.observations();
            println!(
                "name=gate_close batch_units={batch_units} arm={arm:?} compaction_publishes={} root_relocating_publishes={} intents_completed={} last_stop={:?} user_states_now={} user_states_left_after_the_margin={} reads_of_the_passes={}",
                observations.compaction_publishes,
                events.root_relocating_publishes.load(Ordering::SeqCst),
                intents_completed(&runtime),
                observations.last_stop,
                user_states_now(&runtime, &txgs),
                user_states_left_after_the_next_publishes(&runtime, &txgs, MARGIN_OF_THE_GATE),
                reads.load(Ordering::SeqCst) - reads_before,
            );
        }
    }
}

/// B1 的崩溃循环加上搬根：6 次用户改动之后一次用户改动都没有，每次挂上走 3 拍、丢掉。三臂：对照（满段）、强制选带根的段、产品的选段。
/// 判别量：每次挂载之后环里还剩几个用户状态、这次挂载那一份预算读成多少（搬根算非空，「最新用户状态」就落到搬根那一批上）。
#[test]
fn a_crash_loop_with_moved_roots() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for arm in [
        RegionOfTheProbe::FullestDataSegment,
        RegionOfTheProbe::SegmentOfTheInodeRoot,
        RegionOfTheProbe::ChosenByTheProduct,
    ] {
        let (pool, txgs) = pool_with_user_states(&base, USER_STATES_OF_THE_HISTORIES);
        let mut devices = pool.devices;
        let mut txg_before = *txgs.last().expect("有用户改动");
        for mount in 1..=6_u32 {
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let events = Arc::new(CountingTheEvents::default());
            let runtime = mounted_without_a_thread(
                devices,
                parameters_over_the_watermark(16),
                &clock,
                &events,
            );
            let txg_after_the_mount = current_txg(&runtime);
            let states_after_the_mount = user_states_now(&runtime, &txgs);
            force_the_region(&runtime, region_of_the_probe(&runtime, arm));
            passes_until_the_gate_closes(&runtime, &clock, 3);
            let observations = runtime.observations();
            let publishes_counted = runtime
                .with_the_session_and_devices(|locked| {
                    locked
                        .session
                        .session()
                        .compaction
                        .publishes_since_the_last_user_change
                })
                .expect("运行库没坏");
            println!(
                "name=crash_loop arm={arm:?} mount={mount} txgs_advanced_by_the_mount={} states_after_the_mount={states_after_the_mount} compaction_publishes={} root_relocating_publishes={} states_after_the_passes={} publishes_counted_against_the_budget={publishes_counted} last_stop={:?} last_not_started={:?}",
                txg_after_the_mount.0 - txg_before.0,
                observations.compaction_publishes,
                events.root_relocating_publishes.load(Ordering::SeqCst),
                user_states_now(&runtime, &txgs),
                observations.last_stop,
                observations.last_not_started,
            );
            txg_before = current_txg(&runtime);
            devices = devices_after_a_crash(runtime);
        }
    }
}

/// T2、T4：余量 8 是照产品批大小 16 写死的常量（`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`），批大小却是挂载参数。放开批大小扫：
/// 门关时带着活意图崩溃，再挂（同一个批大小）续做那一趟发几次、之后环里还剩几个用户状态。
#[test]
fn the_margin_of_the_gate_under_every_batch_size() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for batch_units in [16_usize, 8, 4, 2, 1] {
        let (pool, txgs) = pool_with_user_states(&base, USER_STATES_OF_THE_HISTORIES);
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let events = Arc::new(CountingTheEvents::default());
        let runtime = mounted_without_a_thread(
            pool.devices,
            parameters_over_the_watermark(batch_units),
            &clock,
            &events,
        );
        force_the_region(
            &runtime,
            region_of_the_probe(&runtime, RegionOfTheProbe::FullestDataSegment),
        );
        passes_until_the_gate_closes(&runtime, &clock, PASSES_AT_MOST);
        let observations = runtime.observations();
        let live_before_the_crash = has_a_live_intent(&runtime);
        let states_before_the_crash = user_states_now(&runtime, &txgs);
        let txg_before = current_txg(&runtime);
        let devices = devices_after_a_crash(runtime);
        let clock_of_the_remount = Arc::new(ClockMovedByTheProbe::default());
        let events_of_the_remount: Arc<CountingTheEvents> = Arc::clone(&events);
        let RuntimeMounted { output, runtime } =
            MountedRuntime::mount_without_the_maintenance_thread(
                &parameters(),
                devices,
                parameters_over_the_watermark(batch_units),
                None,
                clock_of_the_remount,
                events_of_the_remount,
            )
            .expect("可写挂载");
        let publishes_continuing = output
            .compaction_continuation
            .run
            .as_ref()
            .map_or(0, |run| run.versions_published.len());
        println!(
            "name=margin_sweep batch_units={batch_units} publishes_before_the_crash={} last_stop={:?} live_before_the_crash={live_before_the_crash} states_before_the_crash={states_before_the_crash} warm_up_publishes={} publishes_continuing_at_the_mount={publishes_continuing} txgs_advanced_by_the_mount={} states_after_the_mount={}",
            observations.compaction_publishes,
            observations.last_stop,
            output.warm_up_publishes.len(),
            current_txg(&runtime).0 - txg_before.0,
            user_states_now(&runtime, &txgs),
        );
        drop(runtime);
    }
}

/// T2、T4：S 是 5–8 时门放几次、一条意图做不做得完、之后闲着推不推得动。K 取产品 16。
#[test]
fn what_the_gate_lets_through_under_every_geometry() {
    let _only_this = only_this_probe();
    for slots_per_region in [5_u64, 6, 7, 8] {
        set_root_ring_slots_of_the_probe(slots_per_region);
        let base = pool_with_holes_in_the_data_segments();
        for arm in [
            RegionOfTheProbe::FullestDataSegment,
            RegionOfTheProbe::SegmentOfTheInodeRoot,
        ] {
            let (pool, txgs) = pool_with_user_states(&base, USER_STATES_OF_THE_HISTORIES);
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let events = Arc::new(CountingTheEvents::default());
            let runtime = mounted_without_a_thread(
                pool.devices,
                parameters_over_the_watermark(16),
                &clock,
                &events,
            );
            force_the_region(&runtime, region_of_the_probe(&runtime, arm));
            passes_until_the_gate_closes(&runtime, &clock, PASSES_AT_MOST);
            let at_the_stop = runtime.observations();
            let mut idle_passes_publishing = 0_u64;
            for _idle in 0..4 {
                clock.advance_by(ONE_PERIOD);
                idle_passes_publishing +=
                    runtime.run_one_maintenance_pass().expect("运行库没坏").compaction_publishes;
            }
            println!(
                "name=geometry_sweep s={slots_per_region} arm={arm:?} publishes_until_the_stop={} intents_completed={} live_intent={} last_stop={:?} last_not_started={:?} publishes_of_four_idle_passes={idle_passes_publishing} user_states_now={} user_states_left_after_the_margin={}",
                at_the_stop.compaction_publishes,
                intents_completed(&runtime),
                has_a_live_intent(&runtime),
                at_the_stop.last_stop,
                runtime.observations().last_not_started,
                user_states_now(&runtime, &txgs),
                user_states_left_after_the_next_publishes(&runtime, &txgs, MARGIN_OF_THE_GATE),
            );
        }
    }
    set_root_ring_slots_of_the_probe(8);
}

/// T3：`accept` 不算「前台 fsync 在等」，std 的锁不保证交接。真线程，周期压短，每次试先发 4 次用户改动给门与预算，等到后台真在发，再起一次前台入口：
/// 从起调到拿到锁之间后台又发成了几次（整理每次发布之间放锁，判决要的是前台在这之间排得上）。两臂：`accept`、`publish_pending_now`。
#[test]
fn a_foreground_entry_overtaken_by_the_maintenance_thread() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for entry in ["accept", "publish_pending_now"] {
        let pool = base.copied_with_an_empty_stream();
        let events = Arc::new(CountingTheEvents::default());
        let mut runtime_parameters = parameters_over_the_watermark(1);
        runtime_parameters.compaction_period = SHORT_PERIOD;
        runtime_parameters.publish_poll_period = SHORT_PERIOD;
        let runtime = mounted_on_a_thread(pool.devices, runtime_parameters, &events);
        let mut overtaking_of_the_trials = Vec::new();
        for trial in 0..12_u32 {
            for _change in 0..4 {
                runtime
                    .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
                    .expect("建一个 inode");
            }
            let before_the_wait = runtime.observations().compaction_publishes;
            let started_waiting = Instant::now();
            while runtime.observations().compaction_publishes == before_the_wait
                && started_waiting.elapsed() < DEADLINE_OF_A_TRIAL
            {
                std::thread::yield_now();
            }
            if runtime.observations().compaction_publishes == before_the_wait {
                println!("name=entry_overtaken entry={entry} trial={trial} no_compaction_in_this_trial=true last_stop={:?}", runtime.observations().last_stop);
                continue;
            }
            let before_the_entry = runtime.observations().compaction_publishes;
            let called_at = Instant::now();
            let was_refused = match entry {
                "accept" => runtime
                    .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 2))
                    .is_err(),
                _ => runtime.publish_pending_now().is_err(),
            };
            let waited = called_at.elapsed();
            let overtaking = runtime.observations().compaction_publishes - before_the_entry;
            overtaking_of_the_trials.push(overtaking);
            println!(
                "name=entry_overtaken entry={entry} trial={trial} publishes_overtaking={overtaking} waited_microseconds={} was_refused={was_refused}",
                waited.as_micros()
            );
            if runtime.publish_pending_now().is_err() {
                println!("name=entry_overtaken entry={entry} trial={trial} drain_refused=true");
            }
        }
        overtaking_of_the_trials.sort_unstable();
        println!(
            "name=entry_overtaken_summary entry={entry} trials={} sorted_publishes_overtaking={overtaking_of_the_trials:?}",
            overtaking_of_the_trials.len()
        );
        drop(runtime);
    }
}

/// 把 `N₀` 压到前台一直写不进（发布补不上这个差），预算记成扣完。
fn below_the_floor_with_the_budget_spent(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    floor_in_segments: Option<u64>,
) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let empty_now = session.allocator.devices[0].empty_segments();
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.empty_cluster_segments_floor = EmptyClusterSegmentsFloor::ForcedForTests(
                floor_in_segments.unwrap_or(empty_now + 1),
            );
            session.allocator.set_compaction_switches(switches);
            session.compaction.publishes_since_the_last_user_change = 1_000;
        })
        .expect("运行库没坏");
}

/// T3：`fsync_waiters > 0` 时整理让路，没有上界；待发是空的 fsync 也算。低于底额（前台等整理腾段）时几个线程反复 `publish_pending_now`：
/// 这几秒整理发了几次、让路几次。对照是 0 个线程。
#[test]
fn fsyncs_of_nothing_hold_back_the_compaction_below_the_floor() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for threads_calling_fsync in [0_u32, 1, 2, 4] {
        let pool = base.copied_with_an_empty_stream();
        let events = Arc::new(CountingTheEvents::default());
        let mut runtime_parameters = parameters_over_the_watermark(16);
        runtime_parameters.compaction_period = SHORT_PERIOD;
        runtime_parameters.publish_poll_period = SHORT_PERIOD;
        let runtime = mounted_on_a_thread(pool.devices, runtime_parameters, &events);
        below_the_floor_with_the_budget_spent(&runtime, Some(1 << 40));
        let before = runtime.observations();
        let stop = AtomicBool::new(false);
        std::thread::scope(|scope| {
            for _thread in 0..threads_calling_fsync {
                scope.spawn(|| {
                    while !stop.load(Ordering::SeqCst) {
                        let _ignored = runtime.publish_pending_now().is_err();
                    }
                });
            }
            std::thread::sleep(MEASURING);
            stop.store(true, Ordering::SeqCst);
        });
        let after = runtime.observations();
        println!(
            "name=fsync_of_nothing threads={threads_calling_fsync} measured_seconds={} compaction_publishes={} maintenance_passes={} yields={} calls_not_started={} calls_refused={} last_stop={:?}",
            MEASURING.as_secs(),
            after.compaction_publishes - before.compaction_publishes,
            after.maintenance_passes - before.maintenance_passes,
            events.yields_to_a_fsync.load(Ordering::SeqCst),
            events.calls_not_started.load(Ordering::SeqCst),
            events.calls_refused.load(Ordering::SeqCst),
            after.last_stop,
        );
        drop(runtime);
    }
}

/// T3：前台几个线程反复短持锁（读路径都走这把锁）：维护线程这几秒走了几拍。对照是 0 个线程。
#[test]
fn foreground_threads_spinning_on_the_lock_hold_back_the_maintenance_thread() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for threads_spinning in [0_u32, 1, 4, 8] {
        let pool = base.copied_with_an_empty_stream();
        let events = Arc::new(CountingTheEvents::default());
        let mut runtime_parameters = RuntimeParameters::by_default();
        runtime_parameters.compaction_period = SHORT_PERIOD;
        runtime_parameters.publish_poll_period = SHORT_PERIOD;
        let runtime = mounted_on_a_thread(pool.devices, runtime_parameters, &events);
        let before = runtime.observations();
        let stop = AtomicBool::new(false);
        let holds = AtomicU64::new(0);
        std::thread::scope(|scope| {
            for _thread in 0..threads_spinning {
                scope.spawn(|| {
                    while !stop.load(Ordering::SeqCst) {
                        runtime
                            .with_the_session_and_devices(|_locked| ())
                            .expect("运行库没坏");
                        holds.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
            std::thread::sleep(MEASURING);
            stop.store(true, Ordering::SeqCst);
        });
        let after = runtime.observations();
        println!(
            "name=foreground_spinning threads={threads_spinning} measured_seconds={} maintenance_passes={} foreground_holds={}",
            MEASURING.as_secs(),
            after.maintenance_passes - before.maintenance_passes,
            holds.load(Ordering::Relaxed),
        );
        drop(runtime);
    }
}

/// T3、T2：低于底额、预算花完时，按产品线程体的节奏（一趟没完隔一个 P，否则隔一个周期）前台要等多久才写得进；破例这几次发布挤掉几个用户状态。
#[test]
fn below_the_floor_the_foreground_waits_one_period_per_publish() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    let (pool, txgs) = pool_with_user_states(&base, USER_STATES_OF_THE_HISTORIES);
    let clock = Arc::new(ClockMovedByTheProbe::default());
    let events = Arc::new(CountingTheEvents::default());
    let runtime = mounted_without_a_thread(
        pool.devices,
        parameters_over_the_watermark(16),
        &clock,
        &events,
    );
    below_the_floor_with_the_budget_spent(&runtime, None);
    let content = b"foreground write while the empty segments are below the floor".to_vec();
    let mut simulated = Duration::ZERO;
    let mut is_the_trip_under_way = false;
    for pass_number in 1..=40_u32 {
        let elapsed = if is_the_trip_under_way {
            ONE_POLL
        } else {
            ONE_PERIOD
        };
        clock.advance_by(elapsed);
        simulated += elapsed;
        let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
        is_the_trip_under_way = pass.is_a_trip_under_way;
        let write = runtime.publish_user_change(UserChange::OverwriteOfAnInode {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
        });
        let observations = runtime.observations();
        println!(
            "name=below_the_floor pass={pass_number} simulated_seconds={} publishes_of_the_pass={} trip_under_way={is_the_trip_under_way} total_publishes={} last_stop={:?} last_not_started={:?} foreground_write_went_through={} user_states_now={}",
            simulated.as_secs(),
            pass.compaction_publishes,
            observations.compaction_publishes,
            observations.last_stop,
            observations.last_not_started,
            write.is_ok(),
            user_states_now(&runtime, &txgs),
        );
        if write.is_ok() {
            break;
        }
    }
    drop(runtime);
}

/// T3「没定的一处」：低于底额、上一条无产出（`N₀` 压在全空段数加一，强制满段）。用户动作放开扫：这时候哪一种用户动作解得开，解开之后几拍里整理发不发。
#[test]
fn no_yield_below_the_floor_under_every_user_action() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    for action in [
        "nothing",
        "fsync",
        "accept_overwrite",
        "publish_overwrite",
        "publish_new_inode",
        "publish_truncate_to_zero",
        "publish_unlink",
        "publish_create_snapshot",
        "crash_and_mount_again",
    ] {
        let pool = base.copied_with_an_empty_stream();
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let events = Arc::new(CountingTheEvents::default());
        let mut runtime = mounted_without_a_thread(
            pool.devices,
            parameters_over_the_watermark(16),
            &clock,
            &events,
        );
        force_the_region(
            &runtime,
            region_of_the_probe(&runtime, RegionOfTheProbe::FullestDataSegment),
        );
        below_the_floor_with_the_budget_spent(&runtime, None);
        let floor = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().allocator.devices[0].empty_segments() + 1
            })
            .expect("运行库没坏");
        for _pass in 0..16 {
            clock.advance_by(ONE_PERIOD);
            runtime.run_one_maintenance_pass().expect("运行库没坏");
        }
        let at_the_action = runtime.observations();
        let content = b"a write after the intent yielded nothing".to_vec();
        let write_time_seconds = FIXED_WRITE_TIME_SECONDS + 4;
        let overwrite = UserChange::OverwriteOfAnInode {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            content: &content,
            write_time_seconds,
        };
        let outcome = match action {
            "nothing" => "not applicable".to_string(),
            "fsync" => format!("{:?}", runtime.publish_pending_now().map(|_published| ())),
            "accept_overwrite" => format!("{:?}", runtime.accept(overwrite).map(|_taken| ())),
            "publish_overwrite" => {
                format!("{:?}", runtime.publish_user_change(overwrite).map(|_published| ()))
            }
            "publish_new_inode" => format!(
                "{:?}",
                runtime
                    .publish_user_change(one_inode(write_time_seconds))
                    .map(|_published| ())
            ),
            "publish_truncate_to_zero" => format!(
                "{:?}",
                runtime
                    .publish_user_change(UserChange::Truncate {
                        inode: InodeNumber(FIRST_INODE_NUMBER),
                        new_size: 0,
                        write_time_seconds,
                    })
                    .map(|_published| ())
            ),
            "publish_unlink" => format!(
                "{:?}",
                runtime
                    .publish_user_change(UserChange::Unlink {
                        inode: InodeNumber(FIRST_INODE_NUMBER),
                        write_time_seconds,
                    })
                    .map(|_published| ())
            ),
            "publish_create_snapshot" => format!(
                "{:?}",
                runtime
                    .publish_user_change(UserChange::CreateSnapshot)
                    .map(|_published| ())
            ),
            _ => {
                let devices = devices_after_a_crash(runtime);
                runtime = mounted_without_a_thread(
                    devices,
                    parameters_over_the_watermark(16),
                    &clock,
                    &events,
                );
                below_the_floor_with_the_budget_spent(&runtime, Some(floor));
                "mounted again".to_string()
            }
        };
        let publishes_before = events.publishes.load(Ordering::SeqCst);
        for _pass in 0..4 {
            clock.advance_by(ONE_PERIOD);
            runtime.run_one_maintenance_pass().expect("运行库没坏");
        }
        let outcome_short: String = outcome.chars().take(160).collect();
        println!(
            "name=no_yield_sweep action={action} stop_before_the_action={:?} outcome={outcome_short} publishes_of_the_next_four_passes={} last_stop={:?} last_not_started={:?}",
            at_the_action.last_stop,
            events.publishes.load(Ordering::SeqCst) - publishes_before,
            runtime.observations().last_stop,
            runtime.observations().last_not_started,
        );
        drop(runtime);
    }
}

/// T3：写盘报错冻结之后，重发只在「整理到期」那一支（线程体 `now >= next_compaction_at`）。周期长的那一臂量 P 走了几百次之后重发了没有、前台见到什么。
#[test]
fn a_frozen_publish_waits_for_the_compaction_period() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for compaction_period in [Duration::from_secs(3600), SHORT_PERIOD] {
        let pool = base.copied_with_an_empty_stream();
        let faults = pool.faults.clone();
        let events = Arc::new(CountingTheEvents::default());
        let mut runtime_parameters = RuntimeParameters::by_default();
        runtime_parameters.compaction_period = compaction_period;
        runtime_parameters.publish_poll_period = SHORT_PERIOD;
        let runtime = mounted_on_a_thread(pool.devices, runtime_parameters, &events);
        std::thread::sleep(Duration::from_millis(100));
        faults.writes_left_to_fail.store(1, Ordering::SeqCst);
        let refused = runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 5))
            .is_err();
        let is_frozen = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().allocator.frozen_publish().is_some()
            })
            .expect("运行库没坏");
        std::thread::sleep(Duration::from_millis(500));
        let observations = runtime.observations();
        let accept_after_half_a_second = runtime
            .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 6))
            .is_ok();
        println!(
            "name=frozen_resend compaction_period_seconds={} write_refused={refused} frozen={is_frozen} resent_after_half_a_second={} resends_failed={} accept_after_half_a_second_went_through={accept_after_half_a_second}",
            compaction_period.as_secs(),
            observations.frozen_publishes_resent,
            observations.frozen_publish_resends_failed,
        );
        drop(runtime);
    }
}

/// T3：冻结着卸载（线程已停、没人重发）：报哪一种。两个来源：整理一批写盘报错、用户改动写盘报错。卸载文档说 `PendingChangesNotPublished` 交回运行库可以重发；待发空时走哪一支。
#[test]
fn unmounting_while_a_publish_is_frozen() {
    let _only_this = only_this_probe();
    set_root_ring_slots_of_the_probe(8);
    let base = pool_with_holes_in_the_data_segments();
    for source in ["compaction_batch", "user_change"] {
        let (pool, txgs) = pool_with_user_states(&base, USER_STATES_OF_THE_HISTORIES);
        let faults = pool.faults.clone();
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let events = Arc::new(CountingTheEvents::default());
        let runtime = mounted_without_a_thread(
            pool.devices,
            parameters_over_the_watermark(16),
            &clock,
            &events,
        );
        faults.writes_left_to_fail.store(1, Ordering::SeqCst);
        if source == "compaction_batch" {
            clock.advance_by(ONE_PERIOD);
            runtime.run_one_maintenance_pass().expect("运行库没坏");
        } else {
            let refused = runtime
                .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 7))
                .is_err();
            println!("name=unmount_frozen source={source} write_refused={refused}");
        }
        let is_frozen = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().allocator.frozen_publish().is_some()
            })
            .expect("运行库没坏");
        faults.writes_left_to_fail.store(0, Ordering::SeqCst);
        let verdict = match runtime.unmount() {
            Ok(_unmounted) => "unmounted".to_string(),
            Err(refused) => match *refused {
                UnmountRefused::Failed { .. } => "failed".to_string(),
                UnmountRefused::PendingChangesNotPublished { .. } => {
                    "pending_changes_not_published".to_string()
                }
                UnmountRefused::UnmountFailed { refusal, .. } => {
                    let refusal_text = format!("{refusal:?}");
                    let refusal_short: String = refusal_text.chars().take(120).collect();
                    format!("unmount_failed {refusal_short}")
                }
            },
        };
        println!(
            "name=unmount_frozen source={source} frozen={is_frozen} verdict={verdict} user_states_recorded={}",
            txgs.len()
        );
    }
}
