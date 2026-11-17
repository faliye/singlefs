//! 运行库与攒批会话在发布冻结之后的回答与记账（D16（发布语义） 已定项 7、17；D26（后台整理与放置回收） 已定项 13 第 6 条）：
//! 冻结着 fsync 不答成功；卸载重发又报错时那一次可能成了也可能没成；重发成了照那一次的种类记账。
//! 崩溃只取发布边界（丢运行库、盘原样留着）：重发途中每个写上的崩溃在验证包 `crash_enumeration_frozen_publish_resent_stream.rs`。
mod common_pool_on_memory_devices;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount,
    pool_with_holes_in_the_data_segments, DeviceFaults, PoolOnMemoryDevices,
    RecordedDeviceOperation, RecordedMemoryDevice, RecordedStream, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchUnits, CompactionMountParameters,
    CompactionPacing, CompactionWatermarks, EmptyClusterSegmentsWatermark, FragmentationWatermark,
    UnreclaimableBytesWatermark,
};
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::transaction::{
    publish_unlink_intent, KindOfTheFrozenPublish, PoolVersion, PoolWriter, PublishError,
    FIRST_INODE_NUMBER,
};
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MonotonicClockOfThisProcess, MountedRuntime,
    RuntimeMounted, RuntimeParameters, RuntimeRefused, SessionAndDevices, UnmountRefused,
};

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
/// 钟走过 `T_time`（默认 5 秒）。
const PAST_THE_TIME_THRESHOLD: Duration = Duration::from_secs(60);
/// 真线程那一格等冻结的上限：`T_time` 默认 5 秒。
const DEADLINE_OF_THE_FREEZE: Duration = Duration::from_secs(30);
/// 摆成那一份预算早已扣完的发布数。
const PUBLISHES_SPENDING_THE_BUDGET: u64 = 1_000;
/// 比较两臂发布数时走几拍。
const PASSES_AFTER_THE_USER_CHANGE: usize = 4;

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

fn one_inode(write_time_seconds: u64) -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds,
    }
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关。
fn parameters_over_the_watermark() -> RuntimeParameters {
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction = CompactionMountParameters {
        batch_units: CompactionBatchUnits::new(16).expect("至少 1"),
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
    devices: Vec<(DeviceIdentity, RecordedMemoryDevice)>,
    runtime_parameters: RuntimeParameters,
    clock: &Arc<ClockMovedByTheTest>,
) -> MountedRuntime<RecordedMemoryDevice> {
    let clock_of_the_runtime: Arc<ClockMovedByTheTest> = Arc::clone(clock);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        devices,
        runtime_parameters,
        None,
        clock_of_the_runtime,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("可写挂载");
    runtime
}

fn inode_watermark(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
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

fn is_frozen(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    kind_of_the_frozen_publish(runtime).is_some()
}

fn kind_of_the_frozen_publish(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
) -> Option<KindOfTheFrozenPublish> {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .allocator
                .frozen_publish()
                .map(|frozen| frozen.kind())
        })
        .expect("运行库没坏")
}

fn publishes_since_the_last_user_change(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .compaction
                .publishes_since_the_last_user_change
        })
        .expect("运行库没坏")
}

fn spend_the_budget(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session_for_tests_only()
                .compaction
                .publishes_since_the_last_user_change = PUBLISHES_SPENDING_THE_BUDGET;
        })
        .expect("运行库没坏");
}

fn resend_directly(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    let resent = runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session.resend_the_frozen_publish(devices)
        })
        .expect("运行库没坏")
        .expect("盘修好了：重发做得成");
    assert!(resent, "冻结着一次发布");
}

fn is_the_last_completed_intent_kept(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .compaction
                .last_completed_intent
                .is_some()
        })
        .expect("运行库没坏")
}

/// 崩溃：盘从运行库里拿出来，丢掉运行库（只停线程，不写盘）。只给不起线程的运行库用。
fn devices_after_a_crash(
    runtime: MountedRuntime<RecordedMemoryDevice>,
) -> Vec<(DeviceIdentity, RecordedMemoryDevice)> {
    let devices = runtime
        .with_the_session_and_devices(|locked| std::mem::take(&mut locked.devices))
        .expect("运行库没坏");
    drop(runtime);
    devices
}

/// 盘修好，再挂一次，读现行根的 inode 水位。
fn inode_watermark_after_a_remount(
    devices: Vec<(DeviceIdentity, RecordedMemoryDevice)>,
    faults: &DeviceFaults,
) -> u64 {
    faults.writes_left_to_fail.store(0, Ordering::SeqCst);
    faults.barriers_left_to_fail.store(0, Ordering::SeqCst);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(devices, RuntimeParameters::by_default(), &clock);
    inode_watermark(&runtime)
}

/// 待发是怎么冻结的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhereThePublishFroze {
    PublishWhenDueOfAMaintenancePass,
    AnEarlierFsync,
}

/// fsync 被拒之后发生什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AfterTheFsyncRefused {
    CrashWithTheDiskStillFailing,
    DiskHealsThenAPassThenCrash,
}

/// 收下一次改动、盘一直报错、照 `froze_in` 冻结，再调一次 fsync，再照 `after` 崩溃：fsync 不许答成功；交回崩溃前后的水位。
fn watermarks_around_a_fsync_refused_while_frozen(
    froze_in: WhereThePublishFroze,
    after: AfterTheFsyncRefused,
) -> (u64, u64) {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    let inode_watermark_before = inode_watermark(&runtime);
    runtime
        .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .expect("收下一次改动");
    faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
    match froze_in {
        WhereThePublishFroze::AnEarlierFsync => {
            assert!(
                runtime.publish_pending_now().is_err(),
                "写盘报错：那一次 fsync 没发成"
            );
        }
        WhereThePublishFroze::PublishWhenDueOfAMaintenancePass => {
            clock.advance_by(PAST_THE_TIME_THRESHOLD);
            runtime.run_one_maintenance_pass().expect("运行库没坏");
        }
    }
    assert!(is_frozen(&runtime), "写盘报错把发布路径冻结了");
    let resends_failed_before = runtime.observations().frozen_publish_resends_failed;
    let fsync = runtime.publish_pending_now();
    assert!(
        fsync.is_err(),
        "冻结着：fsync 不答成功（待发空着也不答 None）：{fsync:?}"
    );
    assert!(is_frozen(&runtime), "重发没成：照旧冻结着");
    assert_eq!(
        runtime.observations().frozen_publish_resends_failed,
        resends_failed_before + 1,
        "fsync 那一次重发记进观测"
    );
    match after {
        AfterTheFsyncRefused::CrashWithTheDiskStillFailing => {}
        AfterTheFsyncRefused::DiskHealsThenAPassThenCrash => {
            faults.writes_left_to_fail.store(0, Ordering::SeqCst);
            clock.advance_by(ONE_POLL);
            runtime.run_one_maintenance_pass().expect("运行库没坏");
            assert!(!is_frozen(&runtime), "盘修好：下一拍重发成了");
        }
    }
    let devices = devices_after_a_crash(runtime);
    (
        inode_watermark_before,
        inode_watermark_after_a_remount(devices, &faults),
    )
}

#[test]
fn fsync_after_a_freeze_in_a_maintenance_pass_is_refused_and_a_crash_with_the_disk_failing_keeps_the_watermark(
) {
    let (before, after) = watermarks_around_a_fsync_refused_while_frozen(
        WhereThePublishFroze::PublishWhenDueOfAMaintenancePass,
        AfterTheFsyncRefused::CrashWithTheDiskStillFailing,
    );
    assert_eq!(after, before);
}

#[test]
fn fsync_after_a_freeze_in_a_maintenance_pass_is_refused_and_a_later_resend_grows_the_watermark() {
    let (before, after) = watermarks_around_a_fsync_refused_while_frozen(
        WhereThePublishFroze::PublishWhenDueOfAMaintenancePass,
        AfterTheFsyncRefused::DiskHealsThenAPassThenCrash,
    );
    assert_eq!(after, before + 1);
}

#[test]
fn fsync_after_a_freeze_in_an_earlier_fsync_is_refused_and_a_crash_with_the_disk_failing_keeps_the_watermark(
) {
    let (before, after) = watermarks_around_a_fsync_refused_while_frozen(
        WhereThePublishFroze::AnEarlierFsync,
        AfterTheFsyncRefused::CrashWithTheDiskStillFailing,
    );
    assert_eq!(after, before);
}

#[test]
fn fsync_after_a_freeze_in_an_earlier_fsync_is_refused_and_a_later_resend_grows_the_watermark() {
    let (before, after) = watermarks_around_a_fsync_refused_while_frozen(
        WhereThePublishFroze::AnEarlierFsync,
        AfterTheFsyncRefused::DiskHealsThenAPassThenCrash,
    );
    assert_eq!(after, before + 1);
}

/// 真线程、真钟，周期之外只把 P 压短：调用方只调 `accept` 与 `publish_pending_now`。崩溃走卸载（重发又报错，等价于崩溃）。
#[test]
fn fsync_while_the_maintenance_thread_froze_the_publish_is_refused() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.publish_poll_period = Duration::from_millis(5);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        runtime_parameters,
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("可写挂载");
    let inode_watermark_before = inode_watermark(&runtime);
    runtime
        .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .expect("收下一次改动");
    faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
    let given_up_at = Instant::now() + DEADLINE_OF_THE_FREEZE;
    while !is_frozen(&runtime) && Instant::now() < given_up_at {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(is_frozen(&runtime), "线程到了 T_time 发布、写盘报错、冻结");
    let fsync = runtime.publish_pending_now();
    assert!(fsync.is_err(), "冻结着：fsync 不答成功：{fsync:?}");
    let refused = runtime.unmount().expect_err("重发又报错：卸载没做成");
    let UnmountRefused::FrozenPublishResendFailed { devices, .. } = *refused else {
        panic!("卸载先重发、又报错");
    };
    assert_eq!(
        inode_watermark_after_a_remount(devices, &faults),
        inode_watermark_before,
        "写盘一直报错：那一次的根没落盘"
    );
}

/// 盘修好了：fsync 持锁先重发，发成了答成功，崩溃之后水位多 1。
#[test]
fn fsync_after_the_disk_healed_resends_the_frozen_publish_and_answers_success() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    let inode_watermark_before = inode_watermark(&runtime);
    runtime
        .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .expect("收下一次改动");
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime.publish_pending_now().is_err(),
        "写盘报错：那一次 fsync 没发成"
    );
    assert!(is_frozen(&runtime));
    let fsync = runtime.publish_pending_now();
    assert!(
        matches!(fsync, Ok(None)),
        "重发成了答成功，待发并在冻结的那一次里、没有别的要发：{fsync:?}"
    );
    assert!(!is_frozen(&runtime), "重发成了就不冻着");
    assert_eq!(runtime.observations().frozen_publishes_resent, 1);
    let devices = devices_after_a_crash(runtime);
    assert_eq!(
        inode_watermark_after_a_remount(devices, &faults),
        inode_watermark_before + 1,
        "答了成功：崩溃之后那一次在"
    );
}

/// 核心层的入口自己就拒：不重发、不走「待发空答 None」，一个写都不发。
#[test]
fn the_session_refuses_a_fsync_while_a_publish_is_frozen_with_nothing_pending() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let stream: RecordedStream = Arc::clone(&pool.stream);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .is_err(),
        "写盘报错：这次改动没发成"
    );
    let operations_before = stream.lock().expect("录制流的锁里不 panic").len();
    let refused = runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session.publish_pending_now(devices, CallerMonotonicClockReading(Duration::ZERO))
        })
        .expect("运行库没坏")
        .expect_err("冻结着：核心层拒");
    assert!(
        matches!(
            *refused.refusal,
            UserChangeRefused::Publish {
                cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. },
                ..
            }
        ),
        "{refused:?}"
    );
    assert_eq!(
        stream.lock().expect("录制流的锁里不 panic").len(),
        operations_before,
        "在任何读写之前拒"
    );
}

fn barriers_recorded(stream: &RecordedStream) -> usize {
    stream
        .lock()
        .expect("录制流的锁里不 panic")
        .iter()
        .filter(|operation| matches!(operation, RecordedDeviceOperation::Barrier { .. }))
        .count()
}

/// 这个池上一次用户改动发几道屏障（两块盘合计）。
fn barriers_of_one_publish(base: &PoolOnMemoryDevices) -> u64 {
    let pool = base.copied_with_an_empty_stream();
    let stream = Arc::clone(&pool.stream);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    let before = barriers_recorded(&stream);
    runtime
        .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .expect("建一个 inode");
    u64::try_from(barriers_recorded(&stream) - before).expect("屏障数")
}

/// 卸载重发又报错，报错在根落盘之后的最后一道屏障上：那一次成了（D16（发布语义） 已定项 17），再挂之后水位多 1。
#[test]
fn unmounting_when_the_resend_fails_after_the_root_landed_leaves_the_change_in() {
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    let barriers = barriers_of_one_publish(&base);
    let pool = base.copied_with_an_empty_stream();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    let inode_watermark_before = inode_watermark(&runtime);
    faults
        .barriers_let_through_before_the_faults
        .store(barriers - 1, Ordering::SeqCst);
    faults
        .barriers_left_to_fail
        .store(u64::MAX, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .is_err(),
        "最后一道屏障报错：这次改动报没发成"
    );
    assert!(is_frozen(&runtime));
    let refused = runtime.unmount().expect_err("重发又报错：卸载没做成");
    let UnmountRefused::FrozenPublishResendFailed { devices, .. } = *refused else {
        panic!("卸载先重发、又报错");
    };
    assert_eq!(
        inode_watermark_after_a_remount(devices, &faults),
        inode_watermark_before + 1,
        "根已落盘：读现行根判，那一次成了"
    );
}

/// 卸载重发又报错，报错在第一个写上：那一次没成，再挂之后水位不多。
#[test]
fn unmounting_when_the_resend_fails_on_the_first_write_leaves_the_change_out() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    let inode_watermark_before = inode_watermark(&runtime);
    faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .is_err(),
        "写盘报错：这次改动没发成"
    );
    let refused = runtime.unmount().expect_err("重发又报错：卸载没做成");
    let UnmountRefused::FrozenPublishResendFailed { devices, .. } = *refused else {
        panic!("卸载先重发、又报错");
    };
    assert_eq!(
        inode_watermark_after_a_remount(devices, &faults),
        inode_watermark_before,
        "根没落盘：读现行根判，那一次没成"
    );
}

/// 一次用户改动在预算已扣完的池上：照 `write_fails_once` 冻结、下一拍重发，或一次发成；走几拍，交回整理发了几次。
fn compaction_publishes_after_a_user_change_on_a_spent_budget(write_fails_once: bool) -> u64 {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    spend_the_budget(&runtime);
    if write_fails_once {
        faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    }
    let change = runtime.publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1));
    assert_eq!(change.is_err(), write_fails_once);
    for _pass in 0..PASSES_AFTER_THE_USER_CHANGE {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
    }
    assert_eq!(
        runtime.observations().frozen_publishes_resent,
        u64::from(write_fails_once)
    );
    runtime.observations().compaction_publishes
}

/// 重发成了的用户改动与一次发成的同一份账：重给的那一份预算一样多。
#[test]
fn resent_user_change_gives_the_same_budget_as_one_that_went_through_at_once() {
    let went_through_at_once = compaction_publishes_after_a_user_change_on_a_spent_budget(false);
    let resent = compaction_publishes_after_a_user_change_on_a_spent_budget(true);
    assert!(went_through_at_once > 0, "预算重给之后整理发了");
    assert_eq!(resent, went_through_at_once);
}

/// 重发成了的用户改动清上一条完成的意图（D26（后台整理与放置回收） 已定项 13 第 6 条「留到下一次用户改动发成为止」）。
#[test]
fn resent_user_change_clears_the_last_completed_intent() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session_for_tests_only()
                .compaction
                .last_completed_intent = Some(CompactionIntent {
                region: CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(0))
                    .expect("按 64 取整的段起点"),
                allocation_generation_at_issue: CheckpointTxg(1),
                empty_cluster_segments_at_issue: 0,
            });
        })
        .expect("运行库没坏");
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(runtime
        .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .is_err());
    assert!(
        is_the_last_completed_intent_kept(&runtime),
        "没发成：那一条照旧留着"
    );
    resend_directly(&runtime);
    assert!(
        !is_the_last_completed_intent_kept(&runtime),
        "重发成了：清掉"
    );
}

/// 整理的一次发布冻结：种类记成整理，重发成了扣预算、记根槽。
#[test]
fn resent_compaction_publish_is_counted_against_the_budget() {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(
        kind_of_the_frozen_publish(&runtime),
        Some(KindOfTheFrozenPublish::CompactionPublish)
    );
    let root_ring_slots_used = || {
        runtime
            .with_the_session_and_devices(|locked| {
                locked
                    .session
                    .session()
                    .compaction
                    .observations
                    .root_ring_slots_used
            })
            .expect("运行库没坏")
    };
    let root_ring_slots_used_before = root_ring_slots_used();
    let publishes_before = publishes_since_the_last_user_change(&runtime);
    resend_directly(&runtime);
    assert_eq!(
        publishes_since_the_last_user_change(&runtime),
        publishes_before + 1
    );
    assert_eq!(root_ring_slots_used(), root_ring_slots_used_before + 1);
}

/// 把第一个文件放进删除队列（只发意图那一次，不走）。
fn put_the_first_file_into_the_deletion_queue(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            let session = session.session_for_tests_only();
            let pool_parameters = parameters();
            let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
            let version = session.current.version_of_trees().expect("有树的一版");
            let after_the_intent = publish_unlink_intent(
                &mut writer,
                &mut session.allocator,
                version,
                InodeNumber(FIRST_INODE_NUMBER),
                FIXED_WRITE_TIME_SECONDS + 16,
                session.instance,
            )
            .expect("第一个文件是一条活记录");
            session.current = PoolVersion::of_a_version_of_trees(after_the_intent);
        })
        .expect("运行库没坏");
}

/// 后台走队列的一步冻结：这一步不算用户改动，也不算整理的发布：重发成了不记账。
#[test]
fn resent_deletion_step_of_the_background_is_not_counted() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    put_the_first_file_into_the_deletion_queue(&runtime);
    spend_the_budget(&runtime);
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            let _call_with_the_step_failed = session
                .run_compaction_in_the_background_for_at_most_one_publish(
                    devices,
                    CompactionPacing::BY_DEFAULT,
                    &CallerNotWaiting,
                )
                .expect("盘表是挂载的那一份");
        })
        .expect("运行库没坏");
    assert_eq!(
        kind_of_the_frozen_publish(&runtime),
        Some(KindOfTheFrozenPublish::NeitherOfTheCountedKinds)
    );
    resend_directly(&runtime);
    assert_eq!(
        publishes_since_the_last_user_change(&runtime),
        PUBLISHES_SPENDING_THE_BUDGET
    );
}

/// unlink 的意图发成、之后第一个删除步冻结：这次改动已经算发成，重发成了那一份预算照给。
#[test]
fn deletion_step_frozen_after_the_intent_of_an_unlink_counts_as_the_user_change() {
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    let barriers = barriers_of_one_publish(&base);
    let pool = base.copied_with_an_empty_stream();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    spend_the_budget(&runtime);
    faults
        .barriers_let_through_before_the_faults
        .store(barriers, Ordering::SeqCst);
    faults.barriers_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            })
            .is_err(),
        "意图之后那一步的第一道屏障报错"
    );
    assert_eq!(
        kind_of_the_frozen_publish(&runtime),
        Some(KindOfTheFrozenPublish::UserChange)
    );
    resend_directly(&runtime);
    assert_eq!(publishes_since_the_last_user_change(&runtime), 0);
}

/// 改动被拒是因为已经冻结着：冻结的那一次种类不变，不被后到的用户改动改写。
#[test]
fn user_change_refused_while_frozen_does_not_change_the_kind_of_the_frozen_publish() {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(is_frozen(&runtime));
    let refused = runtime.publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1));
    assert!(
        matches!(refused, Err(RuntimeRefused::Refused(_))),
        "冻结着：用户改动被拒"
    );
    assert_eq!(
        kind_of_the_frozen_publish(&runtime),
        Some(KindOfTheFrozenPublish::CompactionPublish)
    );
}
