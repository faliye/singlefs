//! 回退门的余量随每区根槽数 S 走（D26（后台整理与放置回收） 已定项 13 第 5 条的加门，[`singlefs_core::compaction_plan::root_ring_slots_left_by_a_compaction_publish`]），第一版 mkfs 默认的 S 取 5 到 8，
//! 显式给 S 的建池入口下限是 4，这里每个 S 各走一遍：门关的那一刻再发 S 次发布恰好还剩 4 个用户状态、再多 1 次只剩 3 个；S 是 5 到 8 时产品批大小下一条意图走得完；
//! S 是 4 时走不完，门关着并报出来。S = 8 那一族崩溃循环与破例的格在验证包 `compaction_rollback_gate_keeps_four_user_states.rs`。
mod common_pool_on_memory_devices;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common_pool_on_memory_devices::{
    parameters_with_the_root_ring_slots, pool_after_a_sequential_write_in_an_earlier_mount_under,
    pool_with_holes_in_the_data_segments_under, PoolOnMemoryDevices, RecordedMemoryDevice,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionBatchSize, CompactionBatchUnits,
    CompactionMountParameters, CompactionRunStop, CompactionSwitches, CompactionWatermarks,
    EmptyClusterSegmentsWatermark, FragmentationWatermark, UnreclaimableBytesWatermark,
    COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MountedRuntime, RuntimeMounted,
    RuntimeParameters,
};

/// 一拍之间钟走多久：默认周期（定时兜底每一拍都到期）。
const ONE_PERIOD: Duration = Duration::from_secs(30);
/// 至多推几拍：每拍把这一趟推到停，门关之前不用这么多。
const MAINTENANCE_PASSES_AT_MOST: usize = 8;
/// 用户状态的个数：多过门要保住的 4 个。
const USER_STATES_OF_THE_HISTORIES: usize = 6;
/// 回退门保住的状态数。
const ROLLBACK_STATES_KEPT: usize = 4;

#[derive(Debug, Default)]
struct ClockMovedByTheTest {
    reading: Mutex<Duration>,
}

impl ClockMovedByTheTest {
    fn advance_by(&self, elapsed: Duration) {
        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
    }
}

impl ClockOfTheRuntime for ClockMovedByTheTest {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
    }
}

fn parameters_under(slots_per_region: u64) -> MakeFilesystemParameters {
    parameters_with_the_root_ring_slots(
        RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("用例给的 S 在格式区间里"),
    )
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关；批大小取产品默认。
fn runtime_parameters_over_the_watermark() -> RuntimeParameters {
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction = CompactionMountParameters {
        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
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

fn mounted(
    pool_parameters: &MakeFilesystemParameters,
    pool: PoolOnMemoryDevices,
    clock: &Arc<ClockMovedByTheTest>,
) -> MountedRuntime<RecordedMemoryDevice> {
    let clock_of_the_runtime: Arc<ClockMovedByTheTest> = Arc::clone(clock);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        pool_parameters,
        pool.devices,
        runtime_parameters_over_the_watermark(),
        None,
        clock_of_the_runtime,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("全合法的参数：挂载照常做成");
    runtime
}

fn one_inode() -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 20,
    }
}

fn version_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

fn current_txg(runtime: &MountedRuntime<RecordedMemoryDevice>) -> CheckpointTxg {
    runtime
        .with_the_session_and_devices(|locked| {
            version_of(locked.session.session()).root.checkpoint_txg
        })
        .expect("运行库没坏")
}

/// 强制选数据单元最多的那一段（只顺序写的池上段差不多都满，选段判据归别的用例），批大小照给。
fn force_the_fullest_data_segment(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    batch_size: Option<CompactionBatchSize>,
) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
            for record in version_of(session)
                .allocation_records
                .iter()
                .filter(|record| {
                    record.device == DeviceIdentity(0)
                        && !record.is_released
                        && record.span_slots == 2
                })
            {
                *data_units_by_segment
                    .entry(record.slot.0 / 64 * 64)
                    .or_default() += 1;
            }
            let (fullest_segment, _data_units) = data_units_by_segment
                .into_iter()
                .max_by_key(|(_segment, data_units)| *data_units)
                .expect("写了数据单元");
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.forced_region = Some(
                CompactionRegion::of_one_cluster_segment(
                    DeviceIdentity(0),
                    SlotNumber(fullest_segment),
                )
                .expect("按 64 取整的段起点"),
            );
            if let Some(batch_size) = batch_size {
                switches.batch_size = batch_size;
            }
            session.allocator.set_compaction_switches(switches);
        })
        .expect("运行库没坏");
}

fn is_stopped_at_the_gate(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    matches!(
        runtime.observations().last_stop,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
    )
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

/// 从现行那一版起再发 `publishes_ahead` 次发布之后还留在根环里的根属于几个用户状态：txg 为 x 的根由 x 加根环槽数那一次发布盖掉，
/// 某次用户改动的 txg 起到下一次用户改动之前算一个状态（判法同验证包 `compaction_rollback_gate_keeps_four_user_states.rs` 的 `user_states_among_the_roots_from`）。
fn user_states_left_after_the_next_publishes(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    pool_parameters: &MakeFilesystemParameters,
    txgs_of_the_user_changes: &[CheckpointTxg],
    publishes_ahead: u64,
) -> usize {
    let root_ring_slots =
        ROOT_RING_REGIONS * pool_parameters.geometry.root_ring_slots_per_region.count();
    let oldest_root_kept = CheckpointTxg(
        (current_txg(runtime).0 + publishes_ahead + 1).saturating_sub(root_ring_slots),
    );
    runtime
        .with_the_session_and_devices(|locked| {
            let roots = readable_roots(
                locked.devices.as_slice(),
                &pool_parameters.region_devices,
                &pool_parameters.geometry,
                &pool_parameters.filesystem_identifier,
            );
            let floor = effective_rollback_floor(
                locked.devices.as_slice(),
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
        })
        .expect("运行库没坏")
}

/// 门关的那一刻钉到槽：这次挂载里 [`USER_STATES_OF_THE_HISTORIES`] 次用户改动之后推到门关（一批一个，一条意图做不完），再发 S 次发布恰好还剩 4 个用户状态，再多 1 次只剩 3 个。
fn the_gate_closes_exactly_where_the_publishes_of_the_margin_still_keep_four_user_states(
    slots_per_region: u64,
) {
    let pool_parameters = parameters_under(slots_per_region);
    let pool = pool_after_a_sequential_write_in_an_earlier_mount_under(&pool_parameters);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted(&pool_parameters, pool, &clock);
    force_the_fullest_data_segment(
        &runtime,
        Some(CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(1).expect("至少 1"),
        )),
    );
    let mut txgs = Vec::new();
    for _change in 0..USER_STATES_OF_THE_HISTORIES {
        runtime
            .publish_user_change(one_inode())
            .expect("建一个 inode");
        txgs.push(current_txg(&runtime));
    }
    for _pass in 0..MAINTENANCE_PASSES_AT_MOST {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
        if is_stopped_at_the_gate(&runtime) {
            break;
        }
    }
    let observations = runtime.observations();
    assert!(
        matches!(
            observations.last_stop,
            Some(
                CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                    states_after_the_publish_and_the_margin: 3,
                    ..
                }
            )
        ),
        "S = {slots_per_region}：停在回退门上：{observations:?}"
    );
    assert_eq!(
        user_states_left_after_the_next_publishes(
            &runtime,
            &pool_parameters,
            &txgs,
            slots_per_region
        ),
        ROLLBACK_STATES_KEPT,
        "S = {slots_per_region}：门关的那一刻再发 S 次还剩 4 个用户状态：{observations:?}"
    );
    assert_eq!(
        user_states_left_after_the_next_publishes(
            &runtime,
            &pool_parameters,
            &txgs,
            slots_per_region + 1
        ),
        ROLLBACK_STATES_KEPT - 1,
        "S = {slots_per_region}：门放的这一次发布再加余量就挤掉一个，所以关了：{observations:?}"
    );
}

#[test]
fn at_four_slots_per_region_the_gate_closes_exactly_where_the_next_four_publishes_keep_four_states()
{
    the_gate_closes_exactly_where_the_publishes_of_the_margin_still_keep_four_user_states(4);
}

#[test]
fn at_five_slots_per_region_the_gate_closes_exactly_where_the_next_five_publishes_keep_four_states()
{
    the_gate_closes_exactly_where_the_publishes_of_the_margin_still_keep_four_user_states(5);
}

#[test]
fn at_six_slots_per_region_the_gate_closes_exactly_where_the_next_six_publishes_keep_four_states() {
    the_gate_closes_exactly_where_the_publishes_of_the_margin_still_keep_four_user_states(6);
}

#[test]
fn at_seven_slots_per_region_the_gate_closes_exactly_where_the_next_seven_publishes_keep_four_states(
) {
    the_gate_closes_exactly_where_the_publishes_of_the_margin_still_keep_four_user_states(7);
}

#[test]
fn at_eight_slots_per_region_the_gate_closes_exactly_where_the_next_eight_publishes_keep_four_states(
) {
    the_gate_closes_exactly_where_the_publishes_of_the_margin_still_keep_four_user_states(8);
}

/// 上一次挂载里 [`USER_STATES_OF_THE_HISTORIES`] 次用户改动、会话丢掉，再挂（写行与暖机占掉几槽）：产品批大小、强制选数据单元最多的那一段，推到门关或做完一条意图。
fn runtime_pushed_after_user_states_of_an_earlier_mount(
    slots_per_region: u64,
) -> MountedRuntime<RecordedMemoryDevice> {
    let pool_parameters = parameters_under(slots_per_region);
    let mut pool = pool_with_holes_in_the_data_segments_under(&pool_parameters);
    let mounted_for_the_user_states =
        mount_writable(&pool_parameters, &mut pool.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted_for_the_user_states);
    for _change in 0..USER_STATES_OF_THE_HISTORIES {
        session
            .publish_user_change(&mut pool.devices, one_inode())
            .expect("建一个 inode");
    }
    drop(session);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted(&pool_parameters, pool, &clock);
    force_the_fullest_data_segment(&runtime, None);
    for _pass in 0..MAINTENANCE_PASSES_AT_MOST {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
        if is_stopped_at_the_gate(&runtime) || intents_completed(&runtime) >= 1 {
            break;
        }
    }
    runtime
}

/// 门不先关：产品批大小下一条意图做得完。
fn one_intent_finishes_under_the_default_batch_size(slots_per_region: u64) {
    let runtime = runtime_pushed_after_user_states_of_an_earlier_mount(slots_per_region);
    let observations = runtime.observations();
    assert!(
        observations.compaction_publishes > 0,
        "S = {slots_per_region}：门没有先关，整理发过：{observations:?}"
    );
    assert!(
        intents_completed(&runtime) >= 1,
        "S = {slots_per_region}：一条意图做完了：{observations:?}"
    );
}

#[test]
fn at_five_slots_per_region_one_intent_finishes_under_the_default_batch_size() {
    one_intent_finishes_under_the_default_batch_size(5);
}

#[test]
fn at_six_slots_per_region_one_intent_finishes_under_the_default_batch_size() {
    one_intent_finishes_under_the_default_batch_size(6);
}

#[test]
fn at_seven_slots_per_region_one_intent_finishes_under_the_default_batch_size() {
    one_intent_finishes_under_the_default_batch_size(7);
}

#[test]
fn at_eight_slots_per_region_one_intent_finishes_under_the_default_batch_size() {
    one_intent_finishes_under_the_default_batch_size(8);
}

/// S = 4：再挂之后门只放得下写行与暖机之外那一两次，一条意图做不完，停在门上报出来。
#[test]
fn at_four_slots_per_region_the_gate_closes_before_one_intent_finishes_and_reports_the_stop() {
    let runtime = runtime_pushed_after_user_states_of_an_earlier_mount(4);
    let observations = runtime.observations();
    assert!(
        is_stopped_at_the_gate(&runtime),
        "门关着并报出来：{observations:?}"
    );
    assert_eq!(
        intents_completed(&runtime),
        0,
        "门先关，一条意图做不完：{observations:?}"
    );
}
