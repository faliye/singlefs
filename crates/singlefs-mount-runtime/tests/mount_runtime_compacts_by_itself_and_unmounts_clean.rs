//! 运行库（D26（后台整理与放置回收） 已定项 13 第 5 条）的验收：挂载之后调用方不调任何后台入口，水位越线的池上 GC 自己跑、腾出空段，卸载干净。
//! 确定性的格走 `run_one_maintenance_pass` 与测试摆的钟（真 sleep 不进这条路）；起线程的只留几条与时间无关的：线程自己跑得起来、腾出能开的段、停得下、panic 之后前台拿到错误、
//! 冻结的发布不等整理到期就重发、长删除队列之下前台在步与步之间拿得到锁。根环几何扫一遍的在 `mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs`。
//! 水位越线靠挂载参数：全空聚簇段线取整段数的 100%，有一段不全空就越线；默认值下的判法在验证包 `compaction_watermarks_judge_every_branch.rs`。
mod common_pool_on_memory_devices;

use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount,
    pool_with_holes_in_the_data_segments, DeviceFaults, PoolOnMemoryDevices, RecordedMemoryDevice,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, ClusterSegmentEntryKey,
    GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes,
    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits,
    CompactionMountParameters, CompactionPacing, CompactionRunStop, CompactionStartJudgement,
    CompactionSwitches, CompactionTimerFallback, CompactionWatermarks,
    CompactionWriteBandwidthLimit, EmptyClusterSegmentsFloor, EmptyClusterSegmentsWatermark,
    FragmentationWatermark, UnitAreaOfOneDevice, UnreclaimableBytesWatermark,
    COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mounted_session::{
    AcceptRefused, BackgroundCompactionCall, BackgroundCompactionNotStarted,
    BackgroundCompactionTrigger, CompactionRunOutcome, ReasonOfThePublish, UserChange,
};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{
    publish_unlink_intent, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_mount_runtime::{
    BackgroundGpuNotOpened, ClockOfTheRuntime, GpuLeakedByTheBackgroundGpu,
    IgnoringTheMaintenanceEvents, MaintenanceEvent, MonotonicClockOfThisProcess, MountedRuntime,
    OpenedBackgroundGpu, OpensTheBackgroundGpu, ReadsTheGpusLeakedByTheBackgroundGpu,
    ReceiverOfTheMaintenanceEvents, RuntimeFailed, RuntimeMountRefusal, RuntimeMounted,
    RuntimeParameters, RuntimeRefused, SessionAndDevices, UnmountRefused,
};

/// 一拍之间钟走多久：默认周期。
const ONE_PERIOD: Duration = Duration::from_secs(30);
/// 确定性用例至多推几拍。
const MAINTENANCE_PASSES_AT_MOST: usize = 40;
/// 起线程的用例等线程做成一件事的上限：只防挂住，健康时远用不了。
const THREAD_DEADLINE: Duration = Duration::from_secs(300);
/// 第一块盘。
const FIRST_DEVICE: usize = 0;
/// 周期是一小时的用例等重发的上限：只差 `publish_poll_period` 那几毫秒，这个长度只决定红得多快。
const DEADLINE_OF_THE_RESEND: Duration = Duration::from_secs(60);
/// 长删除队列那条用例在几个发布点上试前台 fsync（第一个文件 40 个数据单元、一批一个，队列要走 40 步）。
const PUBLISH_POINTS_TRIED_BY_THE_FOREGROUND_FSYNC: usize = 3;
/// 压得任何一次发布都补不上：前台一直写不进。
const SEGMENTS_NO_PUBLISH_MAKES_UP: u64 = 1 << 40;
/// 预算记成扣完（远大过 `3S − 4`）。
const PUBLISHES_SPENDING_THE_BUDGET: u64 = 1_000;

/// 测试摆的钟。
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

/// 记下后台入口每次调用的结局。
#[derive(Debug, Default)]
struct RecordingTheCompactionCalls {
    /// 每次调用：起跑的话为什么、发了几版；没起跑是 `None`。
    calls: Mutex<Vec<Option<(BackgroundCompactionTrigger, usize)>>>,
    reasons_of_the_pending_publishes: Mutex<Vec<ReasonOfThePublish>>,
}

impl ReceiverOfTheMaintenanceEvents for RecordingTheCompactionCalls {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        match event {
            MaintenanceEvent::CompactionCallReturned(call) => {
                self.calls
                    .lock()
                    .expect("这把锁里不 panic")
                    .push(match call {
                        BackgroundCompactionCall::NotStarted { .. } => None,
                        BackgroundCompactionCall::Ran { trigger, run } => {
                            Some((*trigger, run.versions_published.len()))
                        }
                    });
            }
            MaintenanceEvent::PendingChangesPublished(published) => {
                self.reasons_of_the_pending_publishes
                    .lock()
                    .expect("这把锁里不 panic")
                    .push(published.reason);
            }
            MaintenanceEvent::PublishWhenDueRefused(_)
            | MaintenanceEvent::CompactionCallRefused(_)
            | MaintenanceEvent::FrozenPublishResent
            | MaintenanceEvent::FrozenPublishResendFailed(_)
            | MaintenanceEvent::FloorRaisesPushedByAnAccept(_)
            | MaintenanceEvent::MaintenanceThreadPanicked { .. } => {}
        }
    }
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关；批大小取默认。
fn parameters_over_the_watermark() -> RuntimeParameters {
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

fn mounted_without_a_thread(
    pool: PoolOnMemoryDevices,
    runtime_parameters: RuntimeParameters,
    clock: Arc<ClockMovedByTheTest>,
    receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
) -> RuntimeMounted<RecordedMemoryDevice> {
    MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool.devices,
        runtime_parameters,
        None,
        clock,
        receiver,
    )
    .expect("全合法的参数：挂载照常做成")
}

fn empty_segments_by_the_records(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| {
            locked.session.session().allocator.devices[FIRST_DEVICE].empty_segments()
        })
        .expect("运行库没坏")
}

fn current_version_of(
    locked_session: &singlefs_core::mounted_session::MountedSession,
) -> &TransactionOutput {
    locked_session
        .current
        .version_of_trees()
        .expect("有树的一版")
}

/// 验收第一条：产品开关（批大小默认、不强制选段）、水位真越线，调用方不调任何后台入口，几拍之后 GC 发布、整理完一条意图、记录口径的全空段涨了；
/// 那一段刚搬空时还开不成（放掉的落点在 defer 窗口里），根环转过一圈之后开得成。
#[test]
fn maintenance_passes_on_the_product_switches_empty_a_segment_that_can_be_opened_after_the_ring_turns(
) {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    let empty_before = empty_segments_by_the_records(&runtime);
    for _pass in 0..MAINTENANCE_PASSES_AT_MOST {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
    }
    let observations = runtime.observations();
    assert!(
        observations.compaction_publishes > 0,
        "GC 自己发布了：{observations:?}"
    );
    let (intent_completed, intents_completed, empty_after_by_the_records, lowest_openable) =
        runtime
            .with_the_session_and_devices(|locked| {
                let session = locked.session.session();
                (
                    session.compaction.last_completed_intent,
                    session.compaction.observations.intents_completed,
                    session.allocator.devices[FIRST_DEVICE].empty_segments(),
                    session.allocator.devices[FIRST_DEVICE].lowest_empty_segment(),
                )
            })
            .expect("运行库没坏");
    assert!(intents_completed >= 1, "至少做完一条意图：{observations:?}");
    assert!(
        empty_after_by_the_records > empty_before,
        "记录口径的全空段涨了：{empty_before} → {empty_after_by_the_records}"
    );
    let region: CompactionRegion = intent_completed.expect("记着上一条完成的意图").region;
    assert!(
        lowest_openable.is_none_or(|slot| slot.0 > region.start_slot().0),
        "刚搬空的 R 还在 defer 窗口里，开段开不到它：{lowest_openable:?}，R 从 {:?} 起",
        region.start_slot()
    );
    let mut opened_after_the_ring_turned = false;
    for _publish in 0..32 {
        runtime
            .publish_user_change(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 9,
            })
            .expect("建一个 inode");
        let lowest = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().allocator.devices[FIRST_DEVICE].lowest_empty_segment()
            })
            .expect("运行库没坏");
        if lowest.is_some_and(|slot| slot.0 <= region.start_slot().0) {
            opened_after_the_ring_turned = true;
            break;
        }
    }
    assert!(
        opened_after_the_ring_turned,
        "根环转过一圈之后，R 过了 defer 窗口，开段开得到它"
    );
    runtime.unmount().expect("卸载干净");
}

/// 水位没越线（默认线，大盘上用得很少）：一拍接一拍，一次整理都不发。
#[test]
fn pool_under_the_default_watermarks_publishes_no_compaction() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        RuntimeParameters::by_default(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    for _pass in 0..4 {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
    }
    let observations = runtime.observations();
    assert_eq!(observations.compaction_publishes, 0, "{observations:?}");
    assert_eq!(
        observations.last_not_started,
        Some(BackgroundCompactionNotStarted::NotDueByTheWatermarksAtTheTimerFallback),
        "每次都查了水位、没越线"
    );
}

/// 每次发布之间放锁：放锁的后台入口每次调用至多发一版。
#[test]
fn every_call_of_a_maintenance_pass_publishes_at_most_one_version() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let receiver = Arc::new(RecordingTheCompactionCalls::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
    );
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    let calls = receiver.calls.lock().expect("这把锁里不 panic").clone();
    assert!(
        pass.compaction_publishes >= 2,
        "这一趟不止一次发布：{pass:?}"
    );
    assert!(
        calls
            .iter()
            .flatten()
            .all(|(_trigger, versions_published)| *versions_published <= 1),
        "每次调用至多一版，调用之间放锁：{calls:?}"
    );
}

/// 定时兜底那一支在每拍之前都有一次用户改动的情形下照样轮得到：用户改动之后那一趟至多一批，同一拍接着调轮到定时兜底。
#[test]
fn with_a_user_change_before_every_pass_the_timer_fallback_still_runs() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let receiver = Arc::new(RecordingTheCompactionCalls::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
    );
    for pass_number in 0..4_u64 {
        clock.advance_by(ONE_PERIOD);
        runtime
            .publish_user_change(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10 + pass_number,
            })
            .expect("建一个 inode");
        runtime.run_one_maintenance_pass().expect("运行库没坏");
    }
    let calls = receiver.calls.lock().expect("这把锁里不 panic").clone();
    assert!(
        calls.iter().flatten().any(
            |(trigger, _versions)| *trigger == BackgroundCompactionTrigger::DueAfterAUserChange
        ),
        "用户改动之后那一趟跑了：{calls:?}"
    );
    assert!(
        calls
            .iter()
            .flatten()
            .any(|(trigger, _versions)| *trigger == BackgroundCompactionTrigger::TimerFallback),
        "定时兜底那一支也轮到了：{calls:?}"
    );
}

/// 待发在这一拍被整理排空：同一次调用先排空、再整理，排空的理由是整理之前那一种。
#[test]
fn pending_changes_are_published_before_the_compaction_of_the_same_call() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let receiver = Arc::new(RecordingTheCompactionCalls::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
    );
    let content = b"a change kept pending until the compaction starts".to_vec();
    runtime
        .accept(UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 4,
        }))
        .expect("收进待发");
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(
        receiver
            .reasons_of_the_pending_publishes
            .lock()
            .expect("这把锁里不 panic")
            .first()
            .copied(),
        Some(ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction),
        "整理起跑之前排空"
    );
    let pending_left = runtime
        .with_the_session_and_devices(|locked| locked.session.pending_changes_accepted())
        .expect("运行库没坏");
    assert_eq!(pending_left, 0);
}

/// 前台有 fsync 在等：一批都不发（整理让路，这一趟留着，让路结束之后接着推）。
#[test]
fn waiting_fsync_holds_back_the_next_compaction_publish() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    clock.advance_by(ONE_PERIOD);
    {
        let _waiting = runtime.foreground_fsync_is_waiting();
        let held_back = runtime.run_one_maintenance_pass().expect("运行库没坏");
        assert_eq!(
            held_back.compaction_publishes, 0,
            "一批都不发：{held_back:?}"
        );
        assert!(held_back.is_a_trip_under_way, "这一趟留着：{held_back:?}");
        assert_eq!(
            runtime.observations().last_stop,
            Some(CompactionRunStop::YieldedToAForegroundFsync)
        );
    }
    let after_the_fsync = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        after_the_fsync.compaction_publishes > 0,
        "fsync 走了，不等一个周期接着推：{after_the_fsync:?}"
    );
}

/// 一批写盘报错：发布路径冻结，前台的写被拒、看得到；下一拍重发，前台接着写。
#[test]
fn frozen_publish_is_visible_to_the_foreground_and_resent_by_the_next_pass() {
    let pool = pool_with_holes_in_the_data_segments();
    let faults: DeviceFaults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    let is_frozen = runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .allocator
                .frozen_publish()
                .is_some()
        })
        .expect("运行库没坏");
    assert!(is_frozen, "写盘报错把发布路径冻结了");
    let content = b"a foreground write while the publish is frozen".to_vec();
    let change = UserChange::Overwrite(FirstFile {
        content: &content,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 5,
    });
    assert!(
        matches!(
            runtime.accept(change),
            Err(RuntimeRefused::Refused(
                AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
            ))
        ),
        "前台看得到冻结"
    );
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(
        runtime.observations().frozen_publishes_resent,
        1,
        "下一拍重发了"
    );
    runtime.accept(change).expect("重发之后前台接着写");
}

/// 上一趟不发布地停下之后现行那一版没变：下一拍不重判、一个盘都不读。
#[test]
fn stop_that_published_nothing_is_not_judged_again_until_the_next_publish() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.forced_region = Some(
                CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(1 << 30))
                    .expect("64 的倍数"),
            );
            session.allocator.set_compaction_switches(switches);
        })
        .expect("运行库没坏");
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(
        runtime.observations().last_stop,
        Some(CompactionRunStop::NoEligibleSegment),
        "强制选的那一段不在盘上：停、不发布"
    );
    let reads_before = faults.reads.load(Ordering::SeqCst);
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(
        faults.reads.load(Ordering::SeqCst),
        reads_before,
        "现行那一版没变：一个盘都不读"
    );
    assert_eq!(
        runtime.observations().last_not_started,
        Some(
            BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing {
                stop: CompactionRunStop::NoEligibleSegment
            }
        ),
        "报得出为什么没重判"
    );
    runtime
        .publish_user_change(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 12,
        })
        .expect("建一个 inode");
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session_for_tests_only()
                .compaction
                .start_judgement = CompactionStartJudgement::NotDue;
        })
        .expect("运行库没坏");
    let reads_before_the_timer_judges_again = faults.reads.load(Ordering::SeqCst);
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        faults.reads.load(Ordering::SeqCst) > reads_before_the_timer_judges_again,
        "现行那一版变了：备忘失效，定时兜底那一支照常重判"
    );
}

/// 后台入口问的两样：钟摆在 0、前台不等。
struct CallerNotWaiting;

impl CallerOfTheBackgroundCompaction for CallerNotWaiting {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 卸载带活意图：不用做完，意图留在盘上；再挂续做推到意图删了。
#[test]
fn unmounting_with_a_live_intent_leaves_it_for_the_next_mount_to_continue() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    let pacing = CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(NonZeroU64::MIN),
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    };
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let fullest_segment = {
                let mut data_units_by_segment: std::collections::BTreeMap<u64, u64> =
                    std::collections::BTreeMap::new();
                for record in current_version_of(session).allocation_records.iter().filter(|record| {
                    record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
                }) {
                    *data_units_by_segment
                        .entry(record.slot.0 / 64 * 64)
                        .or_default() += 1;
                }
                data_units_by_segment
                    .into_iter()
                    .max_by_key(|(_segment, data_units)| *data_units)
                    .expect("写了数据单元")
                    .0
            };
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.forced_region = Some(
                CompactionRegion::of_one_cluster_segment(
                    DeviceIdentity(0),
                    SlotNumber(fullest_segment),
                )
                .expect("按 64 取整的段起点"),
            );
            switches.batch_size =
                CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(1).expect("至少 1"));
            session.allocator.set_compaction_switches(switches);
            for _publish in 0..2 {
                let after_the_call = locked
                    .session
                    .run_compaction_in_the_background_for_at_most_one_publish(
                        &mut locked.devices,
                        pacing,
                        &CallerNotWaiting,
                    )
                    .expect("盘表是挂载的那一份");
                assert!(
                    matches!(
                        &after_the_call.call,
                        BackgroundCompactionCall::Ran { run, .. }
                            if matches!(run.outcome, CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone))
                    ),
                    "至多一次发布就返回：{:?}",
                    after_the_call.call
                );
            }
            assert!(
                locked.session.session().allocator.compaction().live_intent().is_some(),
                "签发与一批之后意图活着"
            );
        })
        .expect("运行库没坏");
    let unmounted = runtime.unmount().expect("带活意图照常卸载");
    let RuntimeMounted {
        output,
        runtime: runtime_of_the_next_mount,
    } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        unmounted.devices,
        RuntimeParameters::by_default(),
        None,
        Arc::clone(&clock) as Arc<dyn ClockOfTheRuntime>,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("再挂");
    let continuation = output
        .compaction_continuation
        .run
        .as_ref()
        .expect("盘上有活意图：挂载续做它");
    assert!(
        matches!(
            continuation.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoLiveIntentLeftToContinue)
        ),
        "默认批大小下续做推到意图删了：{:?}",
        continuation.outcome
    );
    let live_after_the_mount = runtime_of_the_next_mount
        .with_the_session_and_devices(|locked| {
            live_compaction_intent_of_the_entries(
                &current_version_of(locked.session.session()).logged_operations,
            )
        })
        .expect("运行库没坏");
    assert_eq!(live_after_the_mount, None, "挂完没有活意图");
    runtime_of_the_next_mount.unmount().expect("卸载干净");
}

/// 开卡器拒：在读任何一块盘之前拒绝挂载，盘原样交回。
struct OpenerRefusing;

impl OpensTheBackgroundGpu for OpenerRefusing {
    fn open_the_background_gpu(
        &self,
        _settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
        Err(BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung)
    }
}

#[test]
fn refusal_of_the_gpu_opener_hands_the_devices_back_before_any_read() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let stream = Arc::clone(&pool.stream);
    let operations_before = stream.lock().expect("这条线程上").len();
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.background_compute.background_compute = BackgroundComputeBackend::Gpu;
    let reads_before = faults.reads.load(Ordering::SeqCst);
    let refused = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool.devices,
        runtime_parameters,
        Some(&OpenerRefusing),
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect_err("开卡器拒：挂载拒");
    assert!(
        matches!(
            refused.reason,
            RuntimeMountRefusal::BackgroundGpuNotOpened(
                BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung
            )
        ),
        "{refused:?}"
    );
    assert_eq!(refused.devices.len(), 2, "盘原样交回");
    assert_eq!(
        faults.reads.load(Ordering::SeqCst),
        reads_before,
        "一个盘都没读"
    );
    assert_eq!(
        stream.lock().expect("这条线程上").len(),
        operations_before,
        "一个写都没发"
    );
}

/// 周期是 0：在读任何一块盘之前拒。
#[test]
fn period_of_zero_is_refused_before_any_read() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let reads_before = faults.reads.load(Ordering::SeqCst);
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction_period = Duration::ZERO;
    let refused = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool.devices,
        runtime_parameters,
        None,
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect_err("周期是 0：挂载拒");
    assert!(
        matches!(refused.reason, RuntimeMountRefusal::PeriodOfZero),
        "{refused:?}"
    );
    assert_eq!(
        faults.reads.load(Ordering::SeqCst),
        reads_before,
        "一个盘都没读"
    );
}

fn parameters_with_a_short_period() -> RuntimeParameters {
    let mut runtime_parameters = parameters_over_the_watermark();
    runtime_parameters.compaction_period = Duration::from_millis(20);
    runtime_parameters.publish_poll_period = Duration::from_millis(5);
    runtime_parameters
}

/// 真线程真钟：调用方不调任何后台入口，线程自己把全空段腾出来，根环转过一圈之后开得到；待发在卸载时排空；卸载之后盘重挂得上。
// 不切片并行：一个池、一条维护线程，被测的就是线程与前台在一把锁上的交错，没有可切的区间。
#[test]
fn the_maintenance_thread_compacts_on_its_own_and_unmount_stops_it_and_publishes_the_pending_changes(
) {
    let pool = pool_with_holes_in_the_data_segments();
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        parameters_with_a_short_period(),
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("全合法的参数：挂载照常做成");
    let given_up_at = Instant::now() + THREAD_DEADLINE;
    while runtime.observations().compaction_publishes == 0 {
        assert!(Instant::now() < given_up_at, "线程起了、调了后台入口");
        std::thread::yield_now();
    }
    let region_emptied_by_the_thread = loop {
        let completed = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().compaction.last_completed_intent
            })
            .expect("运行库没坏");
        if let Some(completed) = completed {
            break completed.region;
        }
        assert!(Instant::now() < given_up_at, "线程自己做完一条意图");
        std::thread::yield_now();
    };
    let mut opened_after_the_ring_turned = false;
    for _publish in 0..32 {
        runtime
            .publish_user_change(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 9,
            })
            .expect("建一个 inode");
        let lowest = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().allocator.devices[FIRST_DEVICE].lowest_empty_segment()
            })
            .expect("运行库没坏");
        if lowest.is_some_and(|slot| slot.0 <= region_emptied_by_the_thread.start_slot().0) {
            opened_after_the_ring_turned = true;
            break;
        }
    }
    assert!(
        opened_after_the_ring_turned,
        "线程腾出的那一段：根环转过一圈之后开段开得到它"
    );
    let content = b"pending at the unmount".to_vec();
    runtime
        .accept(UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 6,
        }))
        .expect("收进待发");
    let unmounted = runtime.unmount().expect("卸载干净");
    let RuntimeMounted {
        runtime: runtime_of_the_next_mount,
        ..
    } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        unmounted.devices,
        RuntimeParameters::by_default(),
        None,
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("卸载之后重挂得上");
    let size_of_the_first_file = runtime_of_the_next_mount
        .with_the_session_and_devices(|locked| {
            current_version_of(locked.session.session())
                .inode_record_of(InodeNumber(FIRST_INODE_NUMBER))
                .expect("第一个文件还在")
                .size
        })
        .expect("运行库没坏");
    assert_eq!(
        size_of_the_first_file,
        u64::try_from(content.len()).expect("内容长度"),
        "待发在卸载之前排空了（或线程先发了）：重挂读到的是收下的那一份"
    );
}

/// 收到后台入口的头一次结局就 panic：维护线程上的 panic。
#[derive(Debug, Default)]
struct PanickingOnTheFirstCompactionCall;

impl ReceiverOfTheMaintenanceEvents for PanickingOnTheFirstCompactionCall {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        assert!(
            !matches!(event, MaintenanceEvent::CompactionCallReturned(_)),
            "用例摆的：维护线程上 panic"
        );
    }
}

/// 线程 panic 等价于崩溃：前台入口拿到错误，不接着用；卸载交回盘。
// 不切片并行：一个池、一条维护线程，被测的是这条线程 panic 之后前台看到什么，没有可切的区间。
#[test]
fn panic_on_the_maintenance_thread_makes_the_foreground_report_the_failure() {
    let pool = pool_with_holes_in_the_data_segments();
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        parameters_with_a_short_period(),
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(PanickingOnTheFirstCompactionCall),
    )
    .expect("全合法的参数：挂载照常做成");
    let given_up_at = Instant::now() + THREAD_DEADLINE;
    while runtime.with_the_session_and_devices(|_locked| ()).is_ok() {
        assert!(
            Instant::now() < given_up_at,
            "线程 panic 之后前台拿得到错误"
        );
        std::thread::yield_now();
    }
    let refused = runtime.unmount().expect_err("坏了不走卸载");
    assert!(
        matches!(
            *refused,
            singlefs_mount_runtime::UnmountRefused::Failed { ref devices, .. } if devices.len() == 2
        ),
        "盘交回：{refused:?}"
    );
}

/// 丢弃只停线程：没走卸载，待发不排空、不写盘。
// 不切片并行：一个池、一条维护线程，被测的是停这条线程，没有可切的区间。
#[test]
fn dropping_the_runtime_stops_the_thread_without_publishing_the_pending_changes() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let stream = Arc::clone(&pool.stream);
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction_period = Duration::from_secs(3600);
    runtime_parameters.publish_poll_period = Duration::from_secs(3600);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        runtime_parameters,
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("全合法的参数：挂载照常做成");
    let content = b"never published".to_vec();
    runtime
        .accept(UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 7,
        }))
        .expect("收进待发");
    let operations_before = stream.lock().expect("线程没 panic").len();
    drop(runtime);
    assert_eq!(
        stream.lock().expect("线程没 panic").len(),
        operations_before,
        "丢弃不写盘（线程睡着，周期一小时）"
    );
}

/// 前台在锁里 panic：之后每个入口报 `RuntimeFailed`，维护的一拍也报，不接着用半个更新里的会话；卸载交回盘。
// 不切片并行：同一把锁上按次序走的一个场景，没有可切的区间。
#[test]
fn panic_under_the_session_lock_fails_every_entry_and_unmount_hands_the_devices_back() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        RuntimeParameters::by_default(),
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.with_the_session_and_devices(|_locked| panic!("用例摆的：前台在锁里 panic"))
    }));
    assert!(panicked.is_err(), "闭包 panic 了");
    assert!(
        matches!(
            runtime.accept(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 8,
            }),
            Err(RuntimeRefused::Failed(_))
        ),
        "前台入口报坏了"
    );
    let failure: Result<_, RuntimeFailed> = runtime.run_one_maintenance_pass();
    assert!(failure.is_err(), "维护的一拍也报坏了");
    let refused = runtime.unmount().expect_err("坏了不走卸载");
    assert!(
        matches!(
            *refused,
            singlefs_mount_runtime::UnmountRefused::Failed { ref devices, .. } if devices.len() == 2
        ),
        "盘交回：{refused:?}"
    );
}

/// 把 `N₀` 压到前台一直写不进；那一份预算照给。
fn below_the_floor(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    budget: BudgetOfTheLastUserChange,
) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.empty_cluster_segments_floor =
                EmptyClusterSegmentsFloor::ForcedForTests(SEGMENTS_NO_PUBLISH_MAKES_UP);
            session.allocator.set_compaction_switches(switches);
            match budget {
                BudgetOfTheLastUserChange::Spent => {
                    session.compaction.publishes_since_the_last_user_change =
                        PUBLISHES_SPENDING_THE_BUDGET;
                }
                BudgetOfTheLastUserChange::Left => {}
            }
        })
        .expect("运行库没坏");
}

/// 那一份预算扣没扣完。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BudgetOfTheLastUserChange {
    Spent,
    Left,
}

/// 前台一直有 fsync 在等、全空段低于底额：前台已写不进，让路换不来什么，破例排在让路之前，每趟仍发 1 次。
fn every_trip_below_the_floor_publishes_once_past_a_waiting_fsync(
    budget: BudgetOfTheLastUserChange,
) {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    below_the_floor(&runtime, budget);
    let _waiting = runtime.foreground_fsync_is_waiting();
    for trip in 1..=3 {
        clock.advance_by(ONE_PERIOD);
        let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
        assert_eq!(
            pass.compaction_publishes,
            1,
            "第 {trip} 趟：破例不让路，只这一次：{pass:?}，停因 {:?}",
            runtime.observations().last_stop
        );
        assert!(
            !pass.is_a_trip_under_way,
            "第 {trip} 趟：前台写不进时不留着没完的一趟，下一趟照样破例：{pass:?}"
        );
    }
}

#[test]
fn below_the_floor_with_the_budget_spent_a_waiting_fsync_does_not_hold_back_the_publish_of_a_trip()
{
    every_trip_below_the_floor_publishes_once_past_a_waiting_fsync(
        BudgetOfTheLastUserChange::Spent,
    );
}

#[test]
fn below_the_floor_with_budget_left_a_waiting_fsync_does_not_hold_back_the_publish_of_a_trip() {
    every_trip_below_the_floor_publishes_once_past_a_waiting_fsync(BudgetOfTheLastUserChange::Left);
}

/// 周期一小时：整理这一小时都不到期，冻结的发布照样在 `publish_poll_period` 的那一拍重发，前台不等整理。
// 不切片并行：一个池、一条维护线程，被测的是线程体里没到期那一支，没有可切的区间。
#[test]
fn frozen_publish_is_resent_by_the_next_poll_long_before_the_compaction_is_due() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults: DeviceFaults = pool.faults.clone();
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction_period = Duration::from_secs(3600);
    runtime_parameters.publish_poll_period = Duration::from_millis(5);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        runtime_parameters,
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("全合法的参数：挂载照常做成");
    // 挂载之后第一拍到期：等它走完后台入口那一次调用，之后这一小时整理都不到期。
    let given_up_at = Instant::now() + THREAD_DEADLINE;
    while runtime.observations().last_not_started.is_none() {
        assert!(Instant::now() < given_up_at, "第一拍调了后台入口");
        std::thread::yield_now();
    }
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 13,
            })
            .is_err(),
        "写盘报错：这次改动没发成"
    );
    let the_resend_given_up_at = Instant::now() + DEADLINE_OF_THE_RESEND;
    while runtime.observations().frozen_publishes_resent == 0 {
        assert!(
            Instant::now() < the_resend_given_up_at,
            "整理不到期，冻结的发布也在下一个 P 重发：{:?}",
            runtime.observations()
        );
        std::thread::yield_now();
    }
    runtime
        .accept(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 14,
        })
        .expect("重发之后前台接着写");
}

/// 一次用户改动写盘报错，发布路径冻结着。交回运行库与用例摆的故障。
fn runtime_with_a_frozen_publish() -> (MountedRuntime<RecordedMemoryDevice>, DeviceFaults) {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults: DeviceFaults = pool.faults.clone();
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        RuntimeParameters::by_default(),
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 15,
            })
            .is_err(),
        "写盘报错：这次改动没发成"
    );
    let is_frozen = runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .allocator
                .frozen_publish()
                .is_some()
        })
        .expect("运行库没坏");
    assert!(is_frozen, "写盘报错把发布路径冻结了");
    (runtime, faults)
}

/// 冻结着卸载：先试一次重发，重发成了照常卸载，冻结的那一次落了盘。
#[test]
fn unmounting_with_a_frozen_publish_resends_it_and_unmounts_clean() {
    let (runtime, _faults) = runtime_with_a_frozen_publish();
    let unmounted = runtime.unmount().expect("重发成了：卸载照常做成");
    let RuntimeMounted {
        runtime: runtime_of_the_next_mount,
        ..
    } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        unmounted.devices,
        RuntimeParameters::by_default(),
        None,
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("卸载之后重挂得上");
    runtime_of_the_next_mount.unmount().expect("卸载干净");
}

/// 冻结着卸载、重发又报错：交回错误与盘，等价于崩溃，盘修好之后走恢复挂得上。
#[test]
fn unmounting_with_a_frozen_publish_whose_resend_fails_hands_the_devices_back_as_a_crash() {
    let (runtime, faults) = runtime_with_a_frozen_publish();
    faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
    let refused = runtime.unmount().expect_err("重发又报错：卸载没做成");
    let UnmountRefused::FrozenPublishResendFailed { devices, .. } = *refused else {
        panic!("报的是重发没成：{refused:?}");
    };
    assert_eq!(devices.len(), 2, "盘交回");
    faults.writes_left_to_fail.store(0, Ordering::SeqCst);
    let RuntimeMounted {
        runtime: runtime_after_the_recovery,
        ..
    } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        devices,
        RuntimeParameters::by_default(),
        None,
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("等价于崩溃：盘修好之后走恢复挂得上");
    runtime_after_the_recovery.unmount().expect("卸载干净");
}

/// 把一个 inode 放进删除队列（意图那一次发布，不走空），一批只放一个单元：有数据单元的文件要走很多步。交回之前把预算归零。
fn put_an_inode_into_the_deletion_queue(
    locked: &mut SessionAndDevices<RecordedMemoryDevice>,
    inode: InodeNumber,
) {
    let SessionAndDevices { session, devices } = locked;
    let session = session.session_for_tests_only();
    let mut switches = session.allocator.deletion_switches();
    switches.batch_size = UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 是合法的批大小"),
    );
    session.allocator.set_deletion_switches(switches);
    let pool_parameters = parameters();
    let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
    let version = session.current.version_of_trees().expect("有树的一版");
    let after_the_intent = publish_unlink_intent(
        &mut writer,
        &mut session.allocator,
        version,
        inode,
        FIXED_WRITE_TIME_SECONDS + 16,
        session.instance,
    )
    .expect("点名的 inode 是一条活记录");
    session.current = PoolVersion::of_a_version_of_trees(after_the_intent);
    session.compaction.publishes_since_the_last_user_change = 0;
}

fn deletion_queue_is_empty(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    runtime
        .with_the_session_and_devices(|locked| {
            current_version_of(locked.session.session()).queue_of_committed_intents_is_empty()
        })
        .expect("运行库没坏")
}

/// 删除队列放进去之后，后台入口每发成一次就在锁外等用例接着放行（送事件在放锁之后）。
struct WaitingForTheTestAfterEveryPublish {
    is_the_queue_put_in: AtomicBool,
    publishes_of_the_call: Mutex<Sender<usize>>,
    the_test_lets_it_go_on: Mutex<Receiver<()>>,
}

impl ReceiverOfTheMaintenanceEvents for WaitingForTheTestAfterEveryPublish {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        let MaintenanceEvent::CompactionCallReturned(BackgroundCompactionCall::Ran { run, .. }) =
            event
        else {
            return;
        };
        if !self.is_the_queue_put_in.load(Ordering::SeqCst) || run.versions_published.is_empty() {
            return;
        }
        // 用例走了（那一头丢了）就不等：丢运行库要 join 这条线程。
        let _test_gone = self
            .publishes_of_the_call
            .lock()
            .expect("这把锁里不 panic")
            .send(run.versions_published.len());
        let _test_gone_or_let_go = self
            .the_test_lets_it_go_on
            .lock()
            .expect("这把锁里不 panic")
            .recv();
    }
}

/// 长删除队列：线程每发一步就放锁，前台 fsync 在步与步之间拿得到锁，那一刻队列还没走空。整段持锁时头一次放锁队列已经空了。
// 不切片并行：一个池、一条维护线程，被测的就是线程与前台在一把锁上的交错，没有可切的区间。
#[test]
fn foreground_fsync_gets_the_lock_between_the_publishes_of_a_long_deletion_queue() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let (sender_of_the_publishes, publishes_of_the_calls) = channel::<usize>();
    let (let_the_thread_go_on, the_test_lets_it_go_on) = channel::<()>();
    let receiver = Arc::new(WaitingForTheTestAfterEveryPublish {
        is_the_queue_put_in: AtomicBool::new(false),
        publishes_of_the_call: Mutex::new(sender_of_the_publishes),
        the_test_lets_it_go_on: Mutex::new(the_test_lets_it_go_on),
    });
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        parameters_with_a_short_period(),
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
    )
    .expect("全合法的参数：挂载照常做成");
    // 换到运行库之后声明的名字上：断言红了展开时这一头先丢，线程不再等，运行库 join 得上。
    let let_the_thread_go_on_dropped_before_the_runtime = let_the_thread_go_on;
    runtime
        .with_the_session_and_devices(|locked| {
            put_an_inode_into_the_deletion_queue(locked, InodeNumber(FIRST_INODE_NUMBER));
            receiver.is_the_queue_put_in.store(true, Ordering::SeqCst);
        })
        .expect("运行库没坏");
    for publish_point in 0..PUBLISH_POINTS_TRIED_BY_THE_FOREGROUND_FSYNC {
        let publishes = publishes_of_the_calls
            .recv_timeout(THREAD_DEADLINE)
            .expect("线程自己走删除队列");
        assert_eq!(
            publishes, 1,
            "第 {publish_point} 个发布点：一次调用至多一版"
        );
        assert!(
            !deletion_queue_is_empty(&runtime),
            "第 {publish_point} 个发布点：前台拿到锁的时候队列还没走空"
        );
        assert!(
            matches!(runtime.publish_pending_now(), Ok(None)),
            "第 {publish_point} 个发布点：前台 fsync 拿得到锁，待发是空的"
        );
        let_the_thread_go_on_dropped_before_the_runtime
            .send(())
            .expect("线程在等");
    }
}

/// 不起线程：一趟已经起跑、走了删除队列的一步之后那一份预算扣完（删除续做不占预算），接着推的那几次调用照样把队列走空。
#[test]
fn spent_budget_does_not_stop_the_deletion_queue_of_a_trip_under_way() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    let did_the_first_call_walk_one_step = runtime
        .with_the_session_and_devices(|locked| {
            put_an_inode_into_the_deletion_queue(locked, InodeNumber(FIRST_INODE_NUMBER));
            let after_the_call = locked
                .session
                .run_compaction_in_the_background_for_at_most_one_publish(
                    &mut locked.devices,
                    CompactionPacing {
                        timer_fallback: CompactionTimerFallback::EveryMilliseconds(NonZeroU64::MIN),
                        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
                    },
                    &CallerNotWaiting,
                )
                .expect("盘表是挂载的那一份");
            locked
                .session
                .session_for_tests_only()
                .compaction
                .publishes_since_the_last_user_change = PUBLISHES_SPENDING_THE_BUDGET;
            matches!(
                &after_the_call.call,
                BackgroundCompactionCall::Ran { run, .. } if run.versions_published.len() == 1
            )
        })
        .expect("运行库没坏");
    assert!(did_the_first_call_walk_one_step, "头一次调用走了队列的一步");
    assert!(!deletion_queue_is_empty(&runtime), "一步走不空");
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        deletion_queue_is_empty(&runtime),
        "预算扣完了，这一趟接着把队列走空：{pass:?}"
    );
}

/// 不起线程：全空段低于底额、预算扣完、删除队列不空：队列不空那一趟只走队列（不占破例），走空之后下一趟照破例，整理照发 1 次。
#[test]
fn below_the_floor_the_deletion_queue_does_not_use_up_the_exception_of_the_trip() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    // 带洞的池上建过的空 inode：意图之后一步走空，第一个文件还在，整理有东西可搬。
    runtime
        .with_the_session_and_devices(|locked| {
            put_an_inode_into_the_deletion_queue(locked, InodeNumber(FIRST_INODE_NUMBER + 1));
        })
        .expect("运行库没坏");
    below_the_floor(&runtime, BudgetOfTheLastUserChange::Spent);
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        deletion_queue_is_empty(&runtime),
        "这一拍把队列走空：{pass:?}"
    );
    let compaction_publishes = runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .compaction
                .publishes_since_the_last_user_change
        })
        .expect("运行库没坏");
    assert_eq!(
        compaction_publishes,
        PUBLISHES_SPENDING_THE_BUDGET + 1,
        "队列走完，这一趟的破例照发 1 次整理：{pass:?}，停因 {:?}",
        runtime.observations().last_stop
    );
}

/// 后台入口问的两样：钟由用例给、前台不等。
struct CallerAtAReading {
    reading: Duration,
}

impl CallerOfTheBackgroundCompaction for CallerAtAReading {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(self.reading)
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 全空段低于底额、预算扣完：一趟的破例还没发就被写带宽截住，这一趟留着，破例也留着：窗口过了接着推，整理照发 1 次。
#[test]
fn below_the_floor_a_trip_held_back_by_the_write_bandwidth_keeps_the_exception_of_the_trip() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
        pool,
        parameters_over_the_watermark(),
        Arc::clone(&clock),
        Arc::new(IgnoringTheMaintenanceEvents),
    );
    below_the_floor(&runtime, BudgetOfTheLastUserChange::Spent);
    let one_call_at = |reading: Duration| {
        runtime
            .with_the_session_and_devices(|locked| {
                let SessionAndDevices { session, devices } = locked;
                drop(
                    session
                        .run_compaction_in_the_background_for_at_most_one_publish(
                            devices,
                            CompactionPacing {
                                timer_fallback: CompactionTimerFallback::EveryMilliseconds(
                                    NonZeroU64::MIN,
                                ),
                                write_bandwidth_limit:
                                    CompactionWriteBandwidthLimit::BytesPerSecond(NonZeroU64::MIN),
                            },
                            &CallerAtAReading { reading },
                        )
                        .expect("盘表是挂载的那一份"),
                );
                session
                    .session()
                    .compaction
                    .publishes_since_the_last_user_change
            })
            .expect("运行库没坏")
    };
    let after_the_first_trip = one_call_at(Duration::ZERO);
    assert_eq!(
        after_the_first_trip,
        PUBLISHES_SPENDING_THE_BUDGET + 1,
        "头一趟破例发 1 次，这一个窗口的写带宽用完了"
    );
    assert_eq!(
        one_call_at(Duration::ZERO),
        after_the_first_trip,
        "这一趟的破例用掉了：预算扣完，不再推"
    );
    assert_eq!(
        one_call_at(Duration::from_millis(10)),
        after_the_first_trip,
        "下一趟起跑，破例还没发就被写带宽截住"
    );
    assert_eq!(
        one_call_at(Duration::from_secs(10)),
        after_the_first_trip + 1,
        "窗口过了：这一趟接着推，破例还在"
    );
}

/// 用例的假卡：只答卡清单。这条用例不推整理，三段不该被调。
struct GpuComputeThatOnlyAnswersTheInventory;

fn card_of_the_double() -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(b"card-of-the-double".to_vec())
}

const MEMORY_OF_THE_CARD_OF_THE_DOUBLE: GpuMemoryInBytes = GpuMemoryInBytes(8 << 30);

impl GpuInventory for GpuComputeThatOnlyAnswersTheInventory {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        vec![(card_of_the_double(), MEMORY_OF_THE_CARD_OF_THE_DOUBLE)]
    }
}

impl BackgroundGpuCompute for GpuComputeThatOnlyAnswersTheInventory {
    fn verify_unit_checksums(
        &mut self,
        _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        panic!("这条用例不推整理");
    }

    fn classify_region_records(
        &mut self,
        _region: CompactionRegion,
        _records: &[RegionRecordToClassifyOnTheGpu<'_>],
        _mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        panic!("这条用例不推整理");
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        _allocation_records: &[AllocationRecord],
        _unit_area: UnitAreaOfOneDevice,
        _segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        panic!("这条用例不推整理");
    }

    fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
}

/// 开成：交回用例拿着的那一份把手。
struct OpenerHandingOverTheHandleOfTheTest {
    gpu_compute: BackgroundGpuComputeHandle,
}

impl OpensTheBackgroundGpu for OpenerHandingOverTheHandleOfTheTest {
    fn open_the_background_gpu(
        &self,
        _settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
        Ok(OpenedBackgroundGpu {
            gpu_compute: self.gpu_compute.clone(),
            report_of_the_opening: String::new(),
            gpus_leaked: None,
        })
    }
}

/// 选 `gpu`、开卡器开成：开卡器交回的把手递给了挂载，这次挂载生效的那一组带的就是它（不要卡：把手是用例的假实现）。
#[test]
fn the_handle_of_the_gpu_opener_reaches_the_mount() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let gpu_compute =
        BackgroundGpuComputeHandle::of_an_implementation(GpuComputeThatOnlyAnswersTheInventory);
    let opener = OpenerHandingOverTheHandleOfTheTest {
        gpu_compute: gpu_compute.clone(),
    };
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.background_compute = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_of_the_double()]),
        background_gpu_memory_limit: Some(vec![MEMORY_OF_THE_CARD_OF_THE_DOUBLE]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool.devices,
        runtime_parameters,
        Some(&opener),
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("开卡器开成、全合法的设置：挂载照常做成");
    let is_the_handle_of_the_mount_the_one_handed_over = runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .parameters_and_device_table
                .background_compute_of_this_mount()
                .in_effect()
                .gpu_compute()
                == Some(&gpu_compute)
        })
        .expect("运行库没坏");
    assert!(
        is_the_handle_of_the_mount_the_one_handed_over,
        "挂载用的是开卡器交回的那一份把手"
    );
}

/// 读泄漏的替身：切换关卡时泄漏了「card-of-the-double」600 字节，还没丢。
#[derive(Debug)]
struct GpusLeakedReadByTheDouble;

impl ReadsTheGpusLeakedByTheBackgroundGpu for GpusLeakedReadByTheDouble {
    fn gpus_leaked_when_closed_at_a_switch(&self) -> Vec<GpuLeakedByTheBackgroundGpu> {
        vec![GpuLeakedByTheBackgroundGpu {
            device: card_of_the_double(),
            memory_held_on_the_device: GpuMemoryInBytes(600),
        }]
    }

    fn gpus_leaked_when_dropped(&self) -> Option<Vec<GpuLeakedByTheBackgroundGpu>> {
        None
    }
}

/// 开成：交回用例的把手、一句开卡的报告与读泄漏的替身。
struct OpenerHandingOverTheLeaksOfTheTest {
    gpu_compute: BackgroundGpuComputeHandle,
    gpus_leaked: Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>,
}

impl OpensTheBackgroundGpu for OpenerHandingOverTheLeaksOfTheTest {
    fn open_the_background_gpu(
        &self,
        _settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
        Ok(OpenedBackgroundGpu {
            gpu_compute: self.gpu_compute.clone(),
            report_of_the_opening: "替身开卡的报告".to_string(),
            gpus_leaked: Some(Arc::clone(&self.gpus_leaked)),
        })
    }
}

/// 开卡器交来的读泄漏的那一份运行库留着：挂着时读得到切换关卡泄漏的卡与开卡的报告，卸载之后在卸载的结局里照样读得到同一份。
#[test]
fn the_leaks_and_the_report_of_the_gpu_opener_are_read_through_the_runtime_and_its_unmount() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let gpus_leaked: Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu> =
        Arc::new(GpusLeakedReadByTheDouble);
    let opener = OpenerHandingOverTheLeaksOfTheTest {
        gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
            GpuComputeThatOnlyAnswersTheInventory,
        ),
        gpus_leaked: Arc::clone(&gpus_leaked),
    };
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.background_compute = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_of_the_double()]),
        background_gpu_memory_limit: Some(vec![MEMORY_OF_THE_CARD_OF_THE_DOUBLE]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool.devices,
        runtime_parameters,
        Some(&opener),
        Arc::new(ClockMovedByTheTest::default()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("开卡器开成、全合法的设置：挂载照常做成");
    let leaked_at_a_switch = vec![GpuLeakedByTheBackgroundGpu {
        device: card_of_the_double(),
        memory_held_on_the_device: GpuMemoryInBytes(600),
    }];
    assert_eq!(
        runtime.report_of_the_opening(),
        Some("替身开卡的报告".to_string())
    );
    assert_eq!(
        runtime
            .gpus_leaked_by_the_background_gpu()
            .map(|read| read.gpus_leaked_when_closed_at_a_switch()),
        Some(leaked_at_a_switch.clone()),
        "挂着时读得到切换关卡泄漏的卡"
    );
    let unmounted = runtime.unmount().expect("卸载干净");
    assert_eq!(
        unmounted
            .gpus_leaked_by_the_background_gpu
            .map(|read| read.gpus_leaked_when_closed_at_a_switch()),
        Some(leaked_at_a_switch),
        "卸载之后照样读得到"
    );
}
