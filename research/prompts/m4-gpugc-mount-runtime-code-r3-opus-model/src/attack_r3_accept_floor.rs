//! 攻方腿探针（m4-gpugc-mount-runtime-code-r3，模型文件，不入产品）：`accept` 自己抬 F 之后，连续被拒时根环与状态数怎么走、一串抬 F 的净账、串里写盘报错之后 fsync 与待发、不同 S 上稳态覆盖写。
//! 只打 `name=` 读数，不断言结论；摆错的历史打 `arm_valid=false`，不 panic。用户动作放开扫（删几个、删多大、每拍之间推不推维护、崩几次、写多大），只固定前缀与故障。
//! 盘是内存稀疏盘，每个起点状态只填一次，之后每段历史拷盘。`--test-threads 1`：一个测试函数按次序跑各臂，到了时间预算就停、打 `name=stopped_by_the_time_budget`。
mod common_pool_on_memory_devices;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters_with_the_root_ring_slots, small_pool_under_for_the_probe, DeviceFaults,
    PoolOnMemoryDevices, RecordedDeviceOperation, RecordedMemoryDevice, RecordedStream,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionMountParameters, CompactionWatermarks,
    EmptyClusterSegmentsWatermark, FragmentationWatermark, UnreclaimableBytesWatermark,
    COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::RaisedFloor;
use singlefs_core::mounted_session::{
    AcceptRefused, BackgroundCompactionCall, ChangeAccepted, UserChange, UserChangeRefused,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::{KindOfTheFrozenPublish, PublishError};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, MaintenanceEvent, MountedRuntime, ReceiverOfTheMaintenanceEvents,
    RuntimeMounted, RuntimeParameters, RuntimeRefused, SessionAndDevices,
};

type DevicesOfTheProbe = Vec<(DeviceIdentity, RecordedMemoryDevice)>;
type RuntimeOfTheProbe = MountedRuntime<RecordedMemoryDevice>;

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
const SLOT_BYTES: i128 = 16384;
/// 前缀里一单元的文件数、32 单元的文件数（最后一个不删，拿来试 `accept`）。
const ONE_UNIT_FILES_OF_THE_PREFIX: usize = 8;
const THIRTY_TWO_UNIT_FILES_OF_THE_PREFIX: usize = 9;
const UNITS_OF_A_THIRTY_TWO_UNIT_FILE: usize = 32;
const FILES_AT_MOST: usize = 400;
const DESCENDING_UNITS: [usize; 6] = [1024, 256, 64, 16, 4, 1];
/// 放不下的那一次写：比两块盘单元区都大（320 MiB 的盘，一单元约 32 KiB）。
const UNITS_OF_A_WRITE_THAT_NEVER_FITS: usize = 8192;
const CONTENT_OF_THE_SMALL_WRITE: &[u8] = b"a foreground write of the attack on the accept";
/// 一段历史里连着试几次 `accept`。
const ACCEPTS_OF_THE_NET_ARM: usize = 4;
const CRASHES_OF_THE_LOOP: usize = 8;
const ACCEPTS_PER_MOUNT: usize = 3;
const RETRIES_IN_THE_LAST_MOUNT: usize = 16;
const CYCLES_OF_THE_STEADY_ARM: usize = 40;
const WRITE_FAULT_POSITIONS_AT_MOST: u64 = 64;
const BARRIER_FAULT_POSITIONS_AT_MOST: u64 = 24;
const WRITES_AFTER_THE_RESEND: usize = 8;
const TEXT_SHORT: usize = 300;
/// 全部臂合起来的时间预算（三方腿定义「造历史、写模型时这样取样」那一条的 40 分钟）。
const SECONDS_ALLOWED: u64 = 2400;

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

fn leading_characters(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

/// 维护拍里每一件事的结局，只记名字与停因：拒绝的调试文字全文能有上 GiB，不格式化。
#[derive(Default)]
struct EventsOfTheProbe {
    lines: Mutex<Vec<String>>,
}

impl EventsOfTheProbe {
    fn take_the_lines(&self) -> Vec<String> {
        std::mem::take(&mut *self.lines.lock().expect("事件的锁里不 panic"))
    }
}

impl ReceiverOfTheMaintenanceEvents for EventsOfTheProbe {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        let line = match event {
            MaintenanceEvent::PendingChangesPublished(_) => "pending_published".to_string(),
            MaintenanceEvent::PublishWhenDueRefused(_) => "publish_when_due_refused".to_string(),
            MaintenanceEvent::CompactionCallReturned(call) => match call {
                BackgroundCompactionCall::NotStarted { reason, .. } => {
                    format!("not_started:{reason:?}")
                }
                BackgroundCompactionCall::Ran { trigger, run } => format!(
                    "ran:{trigger:?}:versions={}:deletion_steps={}:batches={}:outcome={}",
                    run.versions_published.len(),
                    run.deletion_steps_published,
                    run.batches_published,
                    leading_characters(&format!("{:?}", run.outcome), TEXT_SHORT),
                ),
            },
            MaintenanceEvent::CompactionCallRefused(_) => "compaction_call_refused".to_string(),
            MaintenanceEvent::FrozenPublishResent => "frozen_publish_resent".to_string(),
            MaintenanceEvent::FrozenPublishResendFailed(_) => {
                "frozen_publish_resend_failed".to_string()
            }
            MaintenanceEvent::FloorRaisesPushedByAnAccept(raises) => {
                format!("floor_raises_pushed_by_an_accept:{}", raises.len())
            }
            MaintenanceEvent::MaintenanceThreadPanicked { description } => {
                format!("maintenance_thread_panicked:{description}")
            }
        };
        self.lines.lock().expect("事件的锁里不 panic").push(line);
    }
}

struct MountedPool {
    runtime: RuntimeOfTheProbe,
    stream: RecordedStream,
    faults: DeviceFaults,
    clock: Arc<ClockMovedByTheProbe>,
    events: Arc<EventsOfTheProbe>,
}

/// 挂上（不起维护线程，探针一拍一拍推）。被拒打读数交回 `None`。
fn mounted(
    parameters: &MakeFilesystemParameters,
    runtime_parameters: RuntimeParameters,
    pool: PoolOnMemoryDevices,
    label: &str,
) -> Option<MountedPool> {
    let PoolOnMemoryDevices {
        devices,
        stream,
        faults,
    } = pool;
    let clock = Arc::new(ClockMovedByTheProbe::default());
    let events = Arc::new(EventsOfTheProbe::default());
    let clock_of_the_runtime: Arc<ClockMovedByTheProbe> = Arc::clone(&clock);
    let receiver: Arc<EventsOfTheProbe> = Arc::clone(&events);
    match MountedRuntime::mount_without_the_maintenance_thread(
        parameters,
        devices,
        runtime_parameters,
        None,
        clock_of_the_runtime,
        receiver,
    ) {
        Ok(RuntimeMounted { runtime, .. }) => {
            clear_the_stream(&stream);
            Some(MountedPool {
                runtime,
                stream,
                faults,
                clock,
                events,
            })
        }
        Err(refused) => {
            println!(
                "name=mount_refused {label} reason={}",
                leading_characters(&format!("{:?}", refused.reason), TEXT_SHORT)
            );
            None
        }
    }
}

/// 崩溃：盘从运行库里拿出来，丢掉运行库（不写盘）。交回同一条流与故障的池。
fn pool_after_a_crash(mounted_pool: MountedPool) -> PoolOnMemoryDevices {
    let MountedPool {
        runtime,
        stream,
        faults,
        ..
    } = mounted_pool;
    let devices: DevicesOfTheProbe = runtime
        .with_the_session_and_devices(|locked| std::mem::take(&mut locked.devices))
        .expect("运行库没坏");
    drop(runtime);
    PoolOnMemoryDevices {
        devices,
        stream,
        faults,
    }
}

fn clear_the_stream(stream: &RecordedStream) {
    stream.lock().expect("录制流的锁里不 panic").clear();
}

/// 录制流此刻的写次数、屏障次数、写的字节，读完清掉。
fn operations_taken_from_the_stream(stream: &RecordedStream) -> (u64, u64, u64) {
    let mut operations = stream.lock().expect("录制流的锁里不 panic");
    let mut writes = 0_u64;
    let mut barriers = 0_u64;
    let mut written_bytes = 0_u64;
    for operation in operations.iter() {
        match operation {
            RecordedDeviceOperation::Write { bytes, .. } => {
                writes += 1;
                written_bytes += u64::try_from(bytes.len()).expect("写长装得进 u64");
            }
            RecordedDeviceOperation::Barrier { .. } => barriers += 1,
            RecordedDeviceOperation::WriteZeroes { .. }
            | RecordedDeviceOperation::Discard { .. } => {}
        }
    }
    operations.clear();
    (writes, barriers, written_bytes)
}

/// 池此刻的读数（盘 0 那一份；第一版两块盘同样多）。`states` 是回退门读的那一份：(此刻的不同状态数, 走掉下一次与余量之后还剩几个, 最新状态之后的根数)。
#[derive(Clone, Copy, Debug)]
struct ReadingOfThePool {
    txg: u64,
    floor: u64,
    states: Option<(u64, u64, u64)>,
    free: u64,
    unblocked: u64,
    deferred: u64,
    allocated: u64,
    isolated: u64,
    empty_segments: u64,
    pending: u64,
    frozen_kind: Option<KindOfTheFrozenPublish>,
}

fn reading_of(runtime: &RuntimeOfTheProbe) -> ReadingOfThePool {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session();
            let root = session.current.root();
            let device_map = &session.allocator.devices[0];
            ReadingOfThePool {
                txg: root.checkpoint_txg.0,
                floor: root.rollback_floor.0,
                states: session.probe_reading_of_the_rollback_states(locked.devices.as_slice()),
                free: device_map.free_slots(),
                unblocked: device_map.unblocked_slots(),
                deferred: device_map.deferred_slots(),
                allocated: device_map.allocated_slots(),
                isolated: device_map.isolated_slots(),
                empty_segments: device_map.empty_segments(),
                pending: locked.session.pending_changes_accepted(),
                frozen_kind: session
                    .allocator
                    .frozen_publish()
                    .map(|frozen| frozen.kind()),
            }
        })
        .expect("运行库没坏")
}

fn text_of_a_reading(reading: &ReadingOfThePool) -> String {
    format!(
        "txg={} floor={} states={:?} free={} unblocked={} deferred={} allocated={} isolated={} empty_segments={} pending={} frozen={:?}",
        reading.txg,
        reading.floor,
        reading.states,
        reading.free,
        reading.unblocked,
        reading.deferred,
        reading.allocated,
        reading.isolated,
        reading.empty_segments,
        reading.pending,
        reading.frozen_kind,
    )
}

/// 一次 `accept` 的结局，只取要读的量（拒绝全文不格式化）。
#[derive(Debug, Default)]
struct SummaryOfTheAccept {
    outcome: String,
    went_through: bool,
    series: usize,
    empties: usize,
    reclaimed: usize,
    available_before_the_floor_raises: Option<i128>,
    available_after_the_floor_raises: Option<i128>,
    demand: Option<u64>,
    short_by: Option<String>,
}

fn availability_of(cause: &PublishError) -> Option<(i128, u64, String)> {
    match cause {
        PublishError::SpaceAdmissionRefused(refused) => {
            refused.short_devices.first().map(|short| {
                (
                    short.available.0,
                    short.demand.0,
                    format!("{:?}", short.judged_short_by),
                )
            })
        }
        _ => None,
    }
}

fn name_of_a_publish_error(cause: &PublishError) -> String {
    leading_characters(&format!("{cause:?}"), TEXT_SHORT)
}

/// 推的那几串：串数、空发布次数、回收的落点数。
fn figures_of_the_floor_raises(floor_raises: &[RaisedFloor]) -> (usize, usize, usize) {
    (
        floor_raises.len(),
        floor_raises
            .iter()
            .map(|raised| raised.publishes.len())
            .sum(),
        floor_raises
            .iter()
            .map(|raised| raised.reclaimed.len())
            .sum(),
    )
}

fn summary_with_the_floor_raises(
    outcome: String,
    went_through: bool,
    floor_raises: &[RaisedFloor],
    refusals_that_pushed_them: &[PublishError],
) -> SummaryOfTheAccept {
    let (series, empties, reclaimed) = figures_of_the_floor_raises(floor_raises);
    SummaryOfTheAccept {
        outcome,
        went_through,
        series,
        empties,
        reclaimed,
        available_before_the_floor_raises: refusals_that_pushed_them
            .first()
            .and_then(availability_of)
            .map(|(available, _, _)| available),
        ..SummaryOfTheAccept::default()
    }
}

fn summary_of_a_user_change_refusal(refusal: &UserChangeRefused) -> SummaryOfTheAccept {
    match refusal {
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) => {
            let mut summary = summary_with_the_floor_raises(
                format!(
                    "no_space:{}",
                    leading_characters(&format!("{:?}", no_space.stop), TEXT_SHORT)
                ),
                false,
                &no_space.floor_raises,
                &no_space.refusals_that_pushed_the_floor_raises,
            );
            if let Some((available, demand, short_by)) = availability_of(&no_space.last_refusal) {
                summary.available_after_the_floor_raises = Some(available);
                summary.demand = Some(demand);
                summary.short_by = Some(short_by);
            }
            summary
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
            summary_with_the_floor_raises(
                format!(
                    "floor_raise_failed:{}",
                    leading_characters(&format!("{:?}", failed.cause), TEXT_SHORT)
                ),
                false,
                &failed.floor_raises,
                &failed.refusals_that_pushed_the_floor_raises,
            )
        }
        UserChangeRefused::Publish {
            cause,
            floor_raises,
            ..
        } => summary_with_the_floor_raises(
            format!("publish:{}", name_of_a_publish_error(cause)),
            false,
            floor_raises,
            &[],
        ),
        _ => SummaryOfTheAccept {
            outcome: "other_user_change_refusal".to_string(),
            ..SummaryOfTheAccept::default()
        },
    }
}

fn summary_of_the_accept(
    accepted: &Result<ChangeAccepted, RuntimeRefused<AcceptRefused>>,
) -> SummaryOfTheAccept {
    let plain = |outcome: String| SummaryOfTheAccept {
        outcome,
        ..SummaryOfTheAccept::default()
    };
    match accepted {
        Ok(accepted) => summary_with_the_floor_raises(
            format!("accepted:publishes={}", accepted.publishes.len()),
            true,
            &accepted.floor_raises_pushed_for_space.floor_raises,
            &accepted
                .floor_raises_pushed_for_space
                .refusals_that_pushed_the_floor_raises,
        ),
        Err(RuntimeRefused::Failed(failure)) => {
            plain(format!("runtime_failed:{}", failure.description))
        }
        Err(RuntimeRefused::Refused(refused)) => match refused {
            AcceptRefused::RefusedAfterRaisingTheFloorForSpace { refusal, .. } => {
                summary_of_a_user_change_refusal(refusal)
            }
            AcceptRefused::ChangeRefusedByThePublishJudgement(cause) => {
                plain(format!("change_refused:{}", name_of_a_publish_error(cause)))
            }
            AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
                cause,
            ) => plain(format!(
                "together_refused:{}",
                name_of_a_publish_error(cause)
            )),
            AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. } => {
                plain("frozen_not_resent".to_string())
            }
            AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion => {
                plain("queue_not_walked".to_string())
            }
            AcceptRefused::AcceptedButTheTriggeredPublishRefused {
                floor_raises_pushed_for_space,
                ..
            } => summary_with_the_floor_raises(
                "accepted_but_the_triggered_publish_refused".to_string(),
                false,
                &floor_raises_pushed_for_space.floor_raises,
                &floor_raises_pushed_for_space.refusals_that_pushed_the_floor_raises,
            ),
            _ => plain("other_accept_refusal".to_string()),
        },
    }
}

/// `accept` 一次，打一行读数，交回结局与那之后的读数。`gap_is_in_the_deferred`：差的字节不超过此刻 defer 里的字节（回收门槛放开就够）；`gap_is_in_the_deferred=false` 说明差的不全是回退窗口占的。
fn accept_and_print(
    mounted_pool: &MountedPool,
    change: UserChange<'_>,
    line: &str,
    label: &str,
    step: usize,
) -> (SummaryOfTheAccept, ReadingOfThePool) {
    let before = reading_of(&mounted_pool.runtime);
    clear_the_stream(&mounted_pool.stream);
    let accepted = mounted_pool.runtime.accept(change);
    let (writes, barriers, written_bytes) = operations_taken_from_the_stream(&mounted_pool.stream);
    let after = reading_of(&mounted_pool.runtime);
    let summary = summary_of_the_accept(&accepted);
    drop(accepted);
    let gap_in_bytes = match (summary.available_after_the_floor_raises, summary.demand) {
        (Some(available), Some(demand)) => Some(i128::from(demand) - available),
        _ => None,
    };
    let gap_is_in_the_deferred =
        gap_in_bytes.map(|gap| gap <= i128::from(after.deferred) * SLOT_BYTES);
    // 这一串自己放掉、还压在 defer 里的槽（推的量：defer 的差加回收的落点数）。
    let deferred_released_by_the_series = i128::from(after.deferred) - i128::from(before.deferred)
        + i128::try_from(summary.reclaimed).expect("回收的落点数");
    println!(
        "name={line} {label} step={step} outcome={} went_through={} series={} empties={} reclaimed={} available_before={:?} available_after={:?} demand={:?} short_by={:?} gap_bytes={gap_in_bytes:?} gap_is_in_the_deferred={gap_is_in_the_deferred:?} deferred_released_by_the_series={deferred_released_by_the_series} writes={writes} barriers={barriers} written_bytes={written_bytes} before=[{}] after=[{}]",
        summary.outcome,
        summary.went_through,
        summary.series,
        summary.empties,
        summary.reclaimed,
        summary.available_before_the_floor_raises,
        summary.available_after_the_floor_raises,
        summary.demand,
        summary.short_by,
        text_of_a_reading(&before),
        text_of_a_reading(&after),
    );
    (summary, after)
}

/// fsync 一次，打一行读数，交回它答的是不是成功。
fn fsync_and_print(mounted_pool: &MountedPool, line: &str, label: &str, step: usize) -> bool {
    clear_the_stream(&mounted_pool.stream);
    let published = mounted_pool.runtime.publish_pending_now();
    let (writes, barriers, written_bytes) = operations_taken_from_the_stream(&mounted_pool.stream);
    let (answered_success, outcome) = match &published {
        Ok(Some(_)) => (true, "published".to_string()),
        Ok(None) => (true, "nothing_pending".to_string()),
        Err(RuntimeRefused::Failed(failure)) => {
            (false, format!("runtime_failed:{}", failure.description))
        }
        Err(RuntimeRefused::Refused(refused)) => (
            false,
            format!(
                "refused:{}",
                summary_of_a_user_change_refusal(&refused.refusal).outcome
            ),
        ),
    };
    drop(published);
    println!(
        "name={line} {label} step={step} fsync_outcome={outcome} writes={writes} barriers={barriers} written_bytes={written_bytes} after=[{}]",
        text_of_a_reading(&reading_of(&mounted_pool.runtime)),
    );
    answered_success
}

/// 一拍维护，交回这一拍做了什么的文字（只取事件里的名字与停因）。
fn one_pass(mounted_pool: &MountedPool, elapsed: Duration) -> String {
    mounted_pool.clock.advance_by(elapsed);
    clear_the_stream(&mounted_pool.stream);
    let pass = mounted_pool
        .runtime
        .run_one_maintenance_pass()
        .expect("运行库没坏");
    let (writes, barriers, _written_bytes) = operations_taken_from_the_stream(&mounted_pool.stream);
    let events = mounted_pool.events.take_the_lines();
    format!(
        "calls={} compaction_publishes={} deletion_steps={} trip_under_way={} writes={writes} barriers={barriers} events={:?}",
        pass.compaction_calls,
        pass.compaction_publishes,
        pass.deletion_continuation_publishes,
        pass.is_a_trip_under_way,
        events.iter().take(6).collect::<Vec<&String>>(),
    )
}

fn parameters_under(slots_per_region: u64) -> MakeFilesystemParameters {
    let mut parameters = parameters_with_the_root_ring_slots(
        RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("探针给的 S 在格式区间里"),
    );
    parameters.geometry.journal_ring_bytes = 64 << 20;
    parameters
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关：整理一直到期，门才碰得到。
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShapeOfTheFill {
    ThirtyTwoUnitFiles,
    DescendingSizes,
    DescendingSizesWithCompaction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SizeOfTheUnlinkedFile {
    Nothing,
    OneUnit,
    ThirtyTwoUnits,
    Largest,
}

fn inode_watermark(runtime: &RuntimeOfTheProbe) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .current
                .version_of_trees()
                .expect("有树的一版")
                .inode_number_watermark()
        })
        .expect("运行库没坏")
}

fn new_inode(runtime: &RuntimeOfTheProbe) -> Option<InodeNumber> {
    let inode = InodeNumber(inode_watermark(runtime));
    runtime
        .publish_user_change(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        })
        .ok()
        .map(|_published| inode)
}

fn whole_write_goes_through(runtime: &RuntimeOfTheProbe, inode: InodeNumber, units: usize) -> bool {
    let content = vec![0x5A_u8; units * data_unit_payload_capacity()];
    runtime
        .publish_user_change(UserChange::SequentialWriteOfAnInode {
            inode,
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        })
        .is_ok()
}

struct FilesOfTheFill {
    one_unit: Vec<InodeNumber>,
    thirty_two_units: Vec<InodeNumber>,
    largest: Option<InodeNumber>,
    surviving: InodeNumber,
}

struct FilledPool {
    parameters: MakeFilesystemParameters,
    pool: PoolOnMemoryDevices,
    files: FilesOfTheFill,
    shape: ShapeOfTheFill,
}

/// 前台真写到被拒（照用例 `mount_runtime_accept_raises_the_floor_for_space.rs` 的填法，前缀文件多几个，好扫删几个）。填法没走到被拒不 panic，打 `was_refused_a_new_inode=false`。
fn filled_pool(slots_per_region: u64, shape: ShapeOfTheFill) -> FilledPool {
    let started = Instant::now();
    let parameters = parameters_under(slots_per_region);
    let pool = small_pool_under_for_the_probe(&parameters);
    let mounted_pool = mounted(&parameters, RuntimeParameters::by_default(), pool, "fill")
        .expect("填盘之前的池挂得上");
    let runtime = &mounted_pool.runtime;
    let mut one_unit = Vec::new();
    for _file in 0..ONE_UNIT_FILES_OF_THE_PREFIX {
        let inode = new_inode(runtime).expect("空池上建 inode");
        assert!(
            whole_write_goes_through(runtime, inode, 1),
            "空池上写一单元"
        );
        one_unit.push(inode);
    }
    let mut thirty_two_units = Vec::new();
    for _file in 0..THIRTY_TWO_UNIT_FILES_OF_THE_PREFIX {
        let inode = new_inode(runtime).expect("空池上建 inode");
        assert!(
            whole_write_goes_through(runtime, inode, UNITS_OF_A_THIRTY_TWO_UNIT_FILE),
            "空池上写 32 单元"
        );
        thirty_two_units.push(inode);
    }
    let surviving = *thirty_two_units.last().expect("前缀写了 32 单元的文件");
    clear_the_stream(&mounted_pool.stream);
    let runs_compaction = shape == ShapeOfTheFill::DescendingSizesWithCompaction;
    let ladder: &[usize] = match shape {
        ShapeOfTheFill::ThirtyTwoUnitFiles => &[UNITS_OF_A_THIRTY_TWO_UNIT_FILE],
        ShapeOfTheFill::DescendingSizes | ShapeOfTheFill::DescendingSizesWithCompaction => {
            &DESCENDING_UNITS
        }
    };
    let mut largest = None;
    let mut inode_not_written_yet = None;
    let mut files_written = 0_usize;
    let mut was_refused_a_new_inode = false;
    'ladder: for &units in ladder {
        let mut retried_after_the_maintenance = false;
        loop {
            if files_written >= FILES_AT_MOST {
                break 'ladder;
            }
            let inode = match inode_not_written_yet.take().or_else(|| new_inode(runtime)) {
                Some(inode) => inode,
                None => {
                    if runs_compaction && !retried_after_the_maintenance {
                        retried_after_the_maintenance = true;
                        let _pass = one_pass(&mounted_pool, ONE_PERIOD);
                        continue;
                    }
                    was_refused_a_new_inode = true;
                    break 'ladder;
                }
            };
            if whole_write_goes_through(runtime, inode, units) {
                retried_after_the_maintenance = false;
                files_written += 1;
                if largest.is_none() && units >= 64 {
                    largest = Some(inode);
                }
                if runs_compaction {
                    let _pass = one_pass(&mounted_pool, ONE_PERIOD);
                }
                clear_the_stream(&mounted_pool.stream);
            } else {
                inode_not_written_yet = Some(inode);
                if runs_compaction && !retried_after_the_maintenance {
                    retried_after_the_maintenance = true;
                    let _pass = one_pass(&mounted_pool, ONE_PERIOD);
                    continue;
                }
                break;
            }
        }
    }
    clear_the_stream(&mounted_pool.stream);
    println!(
        "name=fill_finished s={slots_per_region} shape={shape:?} files_written={files_written} was_refused_a_new_inode={was_refused_a_new_inode} elapsed_seconds={} reading=[{}]",
        started.elapsed().as_secs(),
        text_of_a_reading(&reading_of(runtime)),
    );
    let pool = pool_after_a_crash(mounted_pool);
    FilledPool {
        parameters,
        pool,
        files: FilesOfTheFill {
            one_unit,
            thirty_two_units,
            largest,
            surviving,
        },
        shape,
    }
}

fn mounted_copy_of(
    filled: &FilledPool,
    runtime_parameters: RuntimeParameters,
    label: &str,
) -> Option<MountedPool> {
    mounted(
        &filled.parameters,
        runtime_parameters,
        filled.pool.copied_with_an_empty_stream(),
        label,
    )
}

fn unlink(inode: InodeNumber) -> UserChange<'static> {
    UserChange::Unlink {
        inode,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10,
    }
}

fn small_overwrite(inode: InodeNumber) -> UserChange<'static> {
    UserChange::OverwriteOfAnInode {
        inode,
        content: CONTENT_OF_THE_SMALL_WRITE,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 20,
    }
}

fn whole_write(inode: InodeNumber, content: &[u8]) -> UserChange<'_> {
    UserChange::SequentialWriteOfAnInode {
        inode,
        content,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 30,
    }
}

/// 干净地 unlink，打一行读数，交回成没成。
fn unlink_and_print(mounted_pool: &MountedPool, inode: InodeNumber, label: &str) -> bool {
    clear_the_stream(&mounted_pool.stream);
    let published = mounted_pool.runtime.publish_user_change(unlink(inode));
    let went_through = published.is_ok();
    drop(published);
    let (writes, barriers, _written_bytes) = operations_taken_from_the_stream(&mounted_pool.stream);
    println!(
        "name=unlink {label} inode={} went_through={went_through} writes={writes} barriers={barriers} after=[{}]",
        inode.0,
        text_of_a_reading(&reading_of(&mounted_pool.runtime)),
    );
    went_through
}

/// 这一刻读 `inode` 的头几个字节，比不比得上 `expected`。读不出打文字。
fn content_matches(runtime: &RuntimeOfTheProbe, inode: InodeNumber, expected: &[u8]) -> String {
    let length_in_bytes = u64::try_from(expected.len()).expect("内容长度装得进 u64");
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            match session.session().read_the_file(
                devices.as_slice(),
                inode,
                FileOffsetInBytes(0),
                length_in_bytes,
            ) {
                Ok(read) => format!("{}", read.bytes == expected),
                Err(refused) => {
                    format!(
                        "unreadable:{}",
                        leading_characters(&format!("{refused:?}"), TEXT_SHORT)
                    )
                }
            }
        })
        .expect("运行库没坏")
}

/// 到了时间预算就打一行、交回 `true`：调用方不接着跑。
fn is_past_the_time_budget(started: Instant, arm: &str) -> bool {
    let seconds = started.elapsed().as_secs();
    if seconds > SECONDS_ALLOWED {
        println!("name=stopped_by_the_time_budget arm={arm} elapsed_seconds={seconds}");
        return true;
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteOfTheCrashLoop {
    /// 一个数据单元以内的覆盖写：回收够了就写得进。
    Small,
    /// 比单元区还大的整份写：永远放不下，调用方 ENOSPC 重试的形状。
    NeverFits,
    /// 不写：只崩了再挂载，对照臂。
    Nothing,
}

/// 崩溃循环（T1「连续 accept 被拒时根环与状态数怎么走」，T2「accept 抬 F 在崩溃循环里的边界」）：写满的池上崩了再挂载，每次挂载试几次 `accept`，
/// 最后一次挂载里接着重试。对照臂不写。翻面量：对照臂与写的臂同一次挂载上的 `states`、每次被拒的 `empties`。
fn arm_crash_loop(
    filled: &FilledPool,
    write: WriteOfTheCrashLoop,
    content_that_never_fits: &[u8],
    started: Instant,
) {
    let label = format!("shape={:?} s_pool write={write:?}", filled.shape,);
    let surviving = filled.files.surviving;
    let change_of_the_arm = || match write {
        WriteOfTheCrashLoop::Small | WriteOfTheCrashLoop::Nothing => small_overwrite(surviving),
        WriteOfTheCrashLoop::NeverFits => whole_write(surviving, content_that_never_fits),
    };
    let accepts_per_mount = match write {
        WriteOfTheCrashLoop::Nothing => 0,
        WriteOfTheCrashLoop::Small | WriteOfTheCrashLoop::NeverFits => ACCEPTS_PER_MOUNT,
    };
    let mut pool = filled.pool.copied_with_an_empty_stream();
    let mut empties_in_total = 0_usize;
    let mut refused_with_empties = 0_usize;
    let mut refused_without_empties = 0_usize;
    for crash in 0..=CRASHES_OF_THE_LOOP {
        if is_past_the_time_budget(started, "crash_loop") {
            return;
        }
        let Some(mounted_pool) = mounted(
            &filled.parameters,
            RuntimeParameters::by_default(),
            pool,
            &label,
        ) else {
            println!("name=crash_loop_stopped {label} crash={crash}");
            return;
        };
        println!(
            "name=crash_loop_mount {label} crash={crash} reading=[{}]",
            text_of_a_reading(&reading_of(&mounted_pool.runtime)),
        );
        let retries = if crash == CRASHES_OF_THE_LOOP && write != WriteOfTheCrashLoop::Nothing {
            RETRIES_IN_THE_LAST_MOUNT
        } else {
            0
        };
        for attempt in 0..accepts_per_mount + retries {
            let (summary, _after) = accept_and_print(
                &mounted_pool,
                change_of_the_arm(),
                "crash_loop_accept",
                &format!("{label} crash={crash}"),
                attempt,
            );
            empties_in_total += summary.empties;
            if summary.went_through {
                let _answered = fsync_and_print(
                    &mounted_pool,
                    "crash_loop_fsync",
                    &format!("{label} crash={crash}"),
                    attempt,
                );
            } else if summary.empties > 0 {
                refused_with_empties += 1;
            } else {
                refused_without_empties += 1;
            }
        }
        pool = pool_after_a_crash(mounted_pool);
    }
    println!(
        "name=crash_loop_summary {label} empties_in_total={empties_in_total} refused_with_empties={refused_with_empties} refused_without_empties={refused_without_empties}"
    );
}

/// 一段历史：删几个、各多大，推一拍，再连着试 `accept`（每次之间推一拍），收下就 fsync、崩了再挂载读。T1「只删 1 个文件写不进」、T3「删够 4 个」放开扫的那一张。
fn arm_net_of_the_floor_raises(
    filled: &FilledPool,
    size: SizeOfTheUnlinkedFile,
    count: usize,
    write_units: Option<usize>,
) {
    let label = format!(
        "shape={:?} size={size:?} count={count} write_units={write_units:?}",
        filled.shape
    );
    let Some(mounted_pool) = mounted_copy_of(filled, RuntimeParameters::by_default(), &label)
    else {
        return;
    };
    let inodes: Vec<InodeNumber> = match size {
        SizeOfTheUnlinkedFile::Nothing => Vec::new(),
        SizeOfTheUnlinkedFile::OneUnit => filled.files.one_unit[..count].to_vec(),
        SizeOfTheUnlinkedFile::ThirtyTwoUnits => filled.files.thirty_two_units[..count].to_vec(),
        SizeOfTheUnlinkedFile::Largest => filled.files.largest.into_iter().collect(),
    };
    for inode in inodes {
        unlink_and_print(&mounted_pool, inode, &label);
    }
    let pass = one_pass(&mounted_pool, ONE_POLL);
    println!(
        "name=net_pass {label} pass=[{pass}] reading=[{}]",
        text_of_a_reading(&reading_of(&mounted_pool.runtime)),
    );
    let content: Vec<u8> = match write_units {
        Some(units) => vec![0x6B_u8; units * data_unit_payload_capacity()],
        None => CONTENT_OF_THE_SMALL_WRITE.to_vec(),
    };
    let change = match write_units {
        Some(_) => whole_write(filled.files.surviving, &content),
        None => small_overwrite(filled.files.surviving),
    };
    let mut went_through_at = None;
    for attempt in 0..ACCEPTS_OF_THE_NET_ARM {
        let (summary, _after) =
            accept_and_print(&mounted_pool, change, "net_accept", &label, attempt);
        if summary.went_through {
            went_through_at = Some(attempt);
            break;
        }
        let pass = one_pass(&mounted_pool, ONE_PERIOD);
        println!("name=net_pass {label} attempt={attempt} pass=[{pass}]");
    }
    let mut answered_success = false;
    if went_through_at.is_some() {
        answered_success = fsync_and_print(&mounted_pool, "net_fsync", &label, 0);
    }
    let pool = pool_after_a_crash(mounted_pool);
    let read_back = mounted(
        &filled.parameters,
        RuntimeParameters::by_default(),
        pool,
        &label,
    )
    .map(|remounted| content_matches(&remounted.runtime, filled.files.surviving, &content));
    println!(
        "name=net_summary {label} went_through_at={went_through_at:?} fsync_answered_success={answered_success} content_after_a_crash_and_a_remount={read_back:?}"
    );
}

/// S 与水位的稳态：写满的池上删 4 个小文件，之后反复整份写同一个 32 单元的文件、fsync、推一拍；被拒就再删一个前缀文件。T1「D16 已定项 1 保留 4 个状态」「回退门的计数绕没绕过去」。
/// 末了崩一次，接着用放不下的写重试几次（门碰过之后一次崩溃走不走到状态数往下掉）。
fn arm_steady_writes(
    filled: &FilledPool,
    slots_per_region: u64,
    watermarks: &str,
    runtime_parameters: fn() -> RuntimeParameters,
    content_that_never_fits: &[u8],
    started: Instant,
) {
    let label = format!(
        "s={slots_per_region} watermarks={watermarks} shape={:?}",
        filled.shape
    );
    let Some(mounted_pool) = mounted_copy_of(filled, runtime_parameters(), &label) else {
        return;
    };
    let mut files_left_to_unlink: Vec<InodeNumber> = filled.files.one_unit.clone();
    files_left_to_unlink
        .extend(filled.files.thirty_two_units[..filled.files.thirty_two_units.len() - 1].iter());
    files_left_to_unlink.reverse();
    for _file in 0..4 {
        if let Some(inode) = files_left_to_unlink.pop() {
            unlink_and_print(&mounted_pool, inode, &label);
        }
    }
    let surviving = filled.files.surviving;
    let mut went_through = 0_usize;
    let mut refused = 0_usize;
    let mut cycles_with_series = 0_usize;
    let mut empties_in_total = 0_usize;
    let mut least_states = u64::MAX;
    let mut cycles_below_four_states = 0_usize;
    for cycle in 0..CYCLES_OF_THE_STEADY_ARM {
        if is_past_the_time_budget(started, "steady_writes") {
            return;
        }
        let fill_byte = u8::try_from(cycle % 251).expect("小于 256");
        let content =
            vec![fill_byte; UNITS_OF_A_THIRTY_TWO_UNIT_FILE * data_unit_payload_capacity()];
        let (summary, _after) = accept_and_print(
            &mounted_pool,
            whole_write(surviving, &content),
            "steady_accept",
            &label,
            cycle,
        );
        empties_in_total += summary.empties;
        if summary.series > 0 {
            cycles_with_series += 1;
        }
        if summary.went_through {
            went_through += 1;
            let _answered = fsync_and_print(&mounted_pool, "steady_fsync", &label, cycle);
        } else {
            refused += 1;
            if let Some(inode) = files_left_to_unlink.pop() {
                unlink_and_print(&mounted_pool, inode, &label);
            }
        }
        let pass = one_pass(&mounted_pool, ONE_PERIOD);
        let reading = reading_of(&mounted_pool.runtime);
        if let Some((states_now, _, _)) = reading.states {
            least_states = least_states.min(states_now);
            if states_now < 4 {
                cycles_below_four_states += 1;
            }
        }
        println!(
            "name=steady_pass {label} cycle={cycle} pass=[{pass}] reading=[{}]",
            text_of_a_reading(&reading),
        );
    }
    println!(
        "name=steady_summary {label} went_through={went_through} refused={refused} cycles_with_series={cycles_with_series} empties_in_total={empties_in_total} least_states={least_states} cycles_below_four_states={cycles_below_four_states} reading=[{}]",
        text_of_a_reading(&reading_of(&mounted_pool.runtime)),
    );
    let pool = pool_after_a_crash(mounted_pool);
    let Some(remounted) = mounted(&filled.parameters, runtime_parameters(), pool, &label) else {
        return;
    };
    println!(
        "name=steady_after_a_crash {label} reading=[{}]",
        text_of_a_reading(&reading_of(&remounted.runtime)),
    );
    for attempt in 0..RETRIES_IN_THE_LAST_MOUNT {
        accept_and_print(
            &remounted,
            whole_write(surviving, content_that_never_fits),
            "steady_after_a_crash_accept",
            &label,
            attempt,
        );
    }
}

/// 摆好「F 低于上限、待发非空、再来一次放不下的写」的起点：删最大的文件、写一次、fsync（F 到上限、空间大），再删 3 个小文件（上限走到 F 之上），崩。
/// 从这个起点挂载之后 `accept` 小写收进待发（不推），放不下的写推抬 F：故障打在这一串上。
fn pool_ready_for_a_fault_in_the_series(filled: &FilledPool) -> Option<PoolOnMemoryDevices> {
    let label = "fault_base";
    let mounted_pool = mounted_copy_of(filled, RuntimeParameters::by_default(), label)?;
    let largest = filled.files.largest?;
    unlink_and_print(&mounted_pool, largest, label);
    let (summary, _after) = accept_and_print(
        &mounted_pool,
        small_overwrite(filled.files.surviving),
        "fault_base_accept",
        label,
        0,
    );
    println!(
        "name=fault_base_went_through went_through={}",
        summary.went_through
    );
    if summary.went_through {
        fsync_and_print(&mounted_pool, "fault_base_fsync", label, 0);
    }
    for inode in &filled.files.one_unit[..3] {
        unlink_and_print(&mounted_pool, *inode, label);
    }
    Some(pool_after_a_crash(mounted_pool))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KindOfTheFault {
    NoFault,
    WriteFailsAfter(u64),
    BarrierFailsAfter(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhatHappensAfterTheFault {
    /// fsync 重发成了：收下的改动要在崩了再挂载之后读得出来。
    FsyncWithTheResendGoingThrough,
    /// fsync 重发又报错：答不答成功、崩了之后读到什么。
    FsyncWithTheResendFailing,
    /// 一拍维护重发，之后接着写：与没故障的对照臂比写得进几次、扣住的槽（free − unblocked − 隔离）留没留着；末了正常卸载。
    PassThenWrites,
}

/// 串里写盘报错（T1「accept 自己写盘之后，写失败、冻结、待发放回那几条路上 fsync 还是不是只在持久之后答成功」）。翻面量：`violation=true`（fsync 答成功而崩了再挂载读不出收下的内容）。
/// 交回故障打没打着。
fn arm_fault_in_the_series(
    filled: &FilledPool,
    base: &PoolOnMemoryDevices,
    fault: KindOfTheFault,
    what_happens: WhatHappensAfterTheFault,
    content_that_never_fits: &[u8],
) -> bool {
    let label = format!("fault={fault:?} then={what_happens:?}");
    let Some(mounted_pool) = mounted(
        &filled.parameters,
        RuntimeParameters::by_default(),
        base.copied_with_an_empty_stream(),
        &label,
    ) else {
        return false;
    };
    let surviving = filled.files.surviving;
    let (pending_write, _after) = accept_and_print(
        &mounted_pool,
        small_overwrite(surviving),
        "fault_pending_accept",
        &label,
        0,
    );
    if !pending_write.went_through || pending_write.series > 0 {
        println!(
            "name=fault_arm_invalid {label} arm_valid=false pending_went_through={} pending_series={}",
            pending_write.went_through, pending_write.series
        );
        return false;
    }
    let faults = &mounted_pool.faults;
    match fault {
        KindOfTheFault::NoFault => {}
        KindOfTheFault::WriteFailsAfter(writes_let_through) => {
            faults
                .writes_let_through_before_the_faults
                .store(writes_let_through, Ordering::SeqCst);
            faults.writes_left_to_fail.store(1, Ordering::SeqCst);
        }
        KindOfTheFault::BarrierFailsAfter(barriers_let_through) => {
            faults
                .barriers_let_through_before_the_faults
                .store(barriers_let_through, Ordering::SeqCst);
            faults.barriers_left_to_fail.store(1, Ordering::SeqCst);
        }
    }
    let (series_arm, after_the_series) = accept_and_print(
        &mounted_pool,
        whole_write(surviving, content_that_never_fits),
        "fault_series_accept",
        &label,
        0,
    );
    let fired = match fault {
        KindOfTheFault::NoFault => false,
        KindOfTheFault::WriteFailsAfter(_) => {
            faults.writes_left_to_fail.load(Ordering::SeqCst) == 0
        }
        KindOfTheFault::BarrierFailsAfter(_) => {
            faults.barriers_left_to_fail.load(Ordering::SeqCst) == 0
        }
    };
    // 故障没打着（串比这个位置短）：撤掉，不算这一臂。
    faults
        .writes_let_through_before_the_faults
        .store(0, Ordering::SeqCst);
    faults.writes_left_to_fail.store(0, Ordering::SeqCst);
    faults
        .barriers_let_through_before_the_faults
        .store(0, Ordering::SeqCst);
    faults.barriers_left_to_fail.store(0, Ordering::SeqCst);
    println!(
        "name=fault_series {label} fired={fired} series_outcome={} frozen={:?} pending={} empties_reported={} txg={} floor={}",
        series_arm.outcome,
        after_the_series.frozen_kind,
        after_the_series.pending,
        series_arm.empties,
        after_the_series.txg,
        after_the_series.floor,
    );
    if !fired && fault != KindOfTheFault::NoFault {
        return false;
    }
    match what_happens {
        WhatHappensAfterTheFault::FsyncWithTheResendGoingThrough
        | WhatHappensAfterTheFault::FsyncWithTheResendFailing => {
            if what_happens == WhatHappensAfterTheFault::FsyncWithTheResendFailing {
                faults.writes_left_to_fail.store(1024, Ordering::SeqCst);
            }
            let answered_success = fsync_and_print(&mounted_pool, "fault_fsync", &label, 0);
            faults.writes_left_to_fail.store(0, Ordering::SeqCst);
            let pool = pool_after_a_crash(mounted_pool);
            let read_back = mounted(
                &filled.parameters,
                RuntimeParameters::by_default(),
                pool,
                &label,
            )
            .map(|remounted| {
                content_matches(&remounted.runtime, surviving, CONTENT_OF_THE_SMALL_WRITE)
            });
            let violation = answered_success && read_back.as_deref() != Some("true");
            println!(
                "name=fault_durability {label} fsync_answered_success={answered_success} content_after_a_crash_and_a_remount={read_back:?} violation={violation}"
            );
        }
        WhatHappensAfterTheFault::PassThenWrites => {
            let pass = one_pass(&mounted_pool, ONE_POLL);
            let after_the_pass = reading_of(&mounted_pool.runtime);
            println!(
                "name=fault_after_the_pass {label} pass=[{pass}] held_slots={} reading=[{}]",
                i128::from(after_the_pass.free)
                    - i128::from(after_the_pass.unblocked)
                    - i128::from(after_the_pass.isolated),
                text_of_a_reading(&after_the_pass),
            );
            let mut went_through = 0_usize;
            for attempt in 0..WRITES_AFTER_THE_RESEND {
                let (summary, after) = accept_and_print(
                    &mounted_pool,
                    small_overwrite(surviving),
                    "fault_later_accept",
                    &label,
                    attempt,
                );
                if summary.went_through {
                    went_through += 1;
                    fsync_and_print(&mounted_pool, "fault_later_fsync", &label, attempt);
                }
                println!(
                    "name=fault_later {label} attempt={attempt} held_slots={}",
                    i128::from(after.free)
                        - i128::from(after.unblocked)
                        - i128::from(after.isolated),
                );
            }
            let observations = mounted_pool.runtime.observations();
            let reading = reading_of(&mounted_pool.runtime);
            let unmounted = mounted_pool.runtime.unmount().is_ok();
            println!(
                "name=fault_later_summary {label} went_through={went_through} of={WRITES_AFTER_THE_RESEND} floor_raises_pushed_by_accepts={} empty_publishes_pushed_by_accepts={} frozen_publishes_resent={} unmounted={unmounted} reading=[{}]",
                observations.floor_raises_pushed_by_accepts,
                observations.empty_publishes_pushed_by_accepts,
                observations.frozen_publishes_resent,
                text_of_a_reading(&reading),
            );
        }
    }
    true
}

#[test]
fn every_arm_of_the_attack_in_order() {
    let started = Instant::now();
    let content_that_never_fits =
        vec![0x7C_u8; UNITS_OF_A_WRITE_THAT_NEVER_FITS * data_unit_payload_capacity()];
    let descending = filled_pool(8, ShapeOfTheFill::DescendingSizes);
    println!(
        "name=estimate after=fill elapsed_seconds={} planned_fills=6 budget_seconds={SECONDS_ALLOWED} note=每臂之前查预算，到了就停",
        started.elapsed().as_secs()
    );
    // 崩溃循环：放不下的写、小写、只崩不写的对照。
    for write in [
        WriteOfTheCrashLoop::Nothing,
        WriteOfTheCrashLoop::NeverFits,
        WriteOfTheCrashLoop::Small,
    ] {
        arm_crash_loop(&descending, write, &content_that_never_fits, started);
    }
    // 串里写盘报错：对照一条，写的位置逐个扫，屏障逐个扫。
    if let Some(base) = pool_ready_for_a_fault_in_the_series(&descending) {
        let every_following = [
            WhatHappensAfterTheFault::FsyncWithTheResendGoingThrough,
            WhatHappensAfterTheFault::FsyncWithTheResendFailing,
            WhatHappensAfterTheFault::PassThenWrites,
        ];
        for what_happens in every_following {
            arm_fault_in_the_series(
                &descending,
                &base,
                KindOfTheFault::NoFault,
                what_happens,
                &content_that_never_fits,
            );
        }
        for position in 0..WRITE_FAULT_POSITIONS_AT_MOST {
            if is_past_the_time_budget(started, "fault_writes") {
                break;
            }
            let mut any_fired = false;
            for what_happens in every_following {
                any_fired |= arm_fault_in_the_series(
                    &descending,
                    &base,
                    KindOfTheFault::WriteFailsAfter(position),
                    what_happens,
                    &content_that_never_fits,
                );
            }
            if !any_fired {
                break;
            }
        }
        for position in 0..BARRIER_FAULT_POSITIONS_AT_MOST {
            if is_past_the_time_budget(started, "fault_barriers") {
                break;
            }
            let mut any_fired = false;
            for what_happens in every_following {
                any_fired |= arm_fault_in_the_series(
                    &descending,
                    &base,
                    KindOfTheFault::BarrierFailsAfter(position),
                    what_happens,
                    &content_that_never_fits,
                );
            }
            if !any_fired {
                break;
            }
        }
    } else {
        println!("name=fault_base_invalid arm_valid=false");
    }
    // 净账：三种填法，删几个、多大、写多大放开扫。
    let thirty_two = filled_pool(8, ShapeOfTheFill::ThirtyTwoUnitFiles);
    let with_compaction = filled_pool(8, ShapeOfTheFill::DescendingSizesWithCompaction);
    arm_crash_loop(
        &thirty_two,
        WriteOfTheCrashLoop::NeverFits,
        &content_that_never_fits,
        started,
    );
    for filled in [&descending, &thirty_two, &with_compaction] {
        let mut cases: Vec<(SizeOfTheUnlinkedFile, usize)> =
            vec![(SizeOfTheUnlinkedFile::Nothing, 0)];
        for count in 1..=ONE_UNIT_FILES_OF_THE_PREFIX - 1 {
            cases.push((SizeOfTheUnlinkedFile::OneUnit, count));
            cases.push((SizeOfTheUnlinkedFile::ThirtyTwoUnits, count));
        }
        if filled.files.largest.is_some() {
            cases.push((SizeOfTheUnlinkedFile::Largest, 1));
        }
        for (size, count) in cases {
            if is_past_the_time_budget(started, "net") {
                break;
            }
            arm_net_of_the_floor_raises(filled, size, count, None);
            if matches!(count, 1 | 4 | 7) || size == SizeOfTheUnlinkedFile::Largest {
                arm_net_of_the_floor_raises(
                    filled,
                    size,
                    count,
                    Some(UNITS_OF_A_THIRTY_TWO_UNIT_FILE),
                );
            }
        }
    }
    drop(thirty_two);
    drop(with_compaction);
    // 稳态：S 从 5 到 8，产品水位与越线水位。S = 8 用已填好的那一份，别的 S 现填一次、用完丢。
    arm_steady_writes(
        &descending,
        8,
        "product",
        RuntimeParameters::by_default,
        &content_that_never_fits,
        started,
    );
    arm_steady_writes(
        &descending,
        8,
        "over_the_line",
        runtime_parameters_over_the_watermark,
        &content_that_never_fits,
        started,
    );
    for slots_per_region in [7_u64, 6, 5] {
        if is_past_the_time_budget(started, "steady_fill") {
            break;
        }
        let filled_under_this_s = filled_pool(slots_per_region, ShapeOfTheFill::DescendingSizes);
        arm_steady_writes(
            &filled_under_this_s,
            slots_per_region,
            "product",
            RuntimeParameters::by_default,
            &content_that_never_fits,
            started,
        );
        arm_steady_writes(
            &filled_under_this_s,
            slots_per_region,
            "over_the_line",
            runtime_parameters_over_the_watermark,
            &content_that_never_fits,
            started,
        );
    }
    println!(
        "name=probe_finished elapsed_seconds={}",
        started.elapsed().as_secs()
    );
}
