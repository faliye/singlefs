//! 攻方腿探针（m4-gpugc-mount-runtime-code-r2，模型文件，不入产品）：拷进冻结副本 `crates/singlefs-mount-runtime/tests/` 跑（共用搭建由 `tools/patch_the_fixture.py` 加三样故障）。
//! 每条探针只打 `name=` 开头的读数，不断言结论：结果反过来也照样打出来，由判决读。
//! 崩溃只取发布边界（丢运行库、盘原样留着）；段内写的子集不枚举，崩溃状态枚举归崩溃点重放（报告「这条腿自己的限度」）。
mod common_pool_on_memory_devices;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, parameters_with_the_root_ring_slots,
    pool_after_a_sequential_write_in_an_earlier_mount, pool_with_holes_in_the_data_segments,
    pool_with_holes_in_the_data_segments_under, PoolOnMemoryDevices, RecordedDeviceOperation,
    RecordedMemoryDevice, RecordedStream, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionBatchUnits, CompactionMountParameters,
    CompactionRunStop, CompactionSwitches, CompactionWatermarks, EmptyClusterSegmentsWatermark,
    FragmentationWatermark, UnreclaimableBytesWatermark,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::{
    publish_unlink_intent, PoolVersion, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MonotonicClockOfThisProcess, MountedRuntime,
    RuntimeMounted, RuntimeParameters, SessionAndDevices, UnmountRefused,
};

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
/// 钟走过 `T_time`（默认 5 秒）。
const PAST_THE_TIME_THRESHOLD: Duration = Duration::from_secs(60);
/// 判一件事「一直不发生」走多少拍。
const PASSES_OF_A_SWEEP: u32 = 20;
const USER_STATES_OF_THE_HISTORIES: usize = 6;
/// 带线程的探针等冻结的上限：`T_time` 默认 5 秒。
const DEADLINE_OF_THE_FREEZE: Duration = Duration::from_secs(30);

type DevicesOfTheProbe = Vec<(DeviceIdentity, RecordedMemoryDevice)>;

/// 探针之间不交错：起线程的探针量的是锁上的交错。
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

fn first_characters(text: &str) -> String {
    text.chars().take(160).collect()
}

fn one_inode(write_time_seconds: u64) -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds,
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

fn parameters_under(slots_per_region: u64) -> MakeFilesystemParameters {
    parameters_with_the_root_ring_slots(
        RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("探针给的 S 在格式区间里"),
    )
}

fn mounted_or_why_not(
    pool_parameters: &MakeFilesystemParameters,
    devices: DevicesOfTheProbe,
    runtime_parameters: RuntimeParameters,
    clock: &Arc<ClockMovedByTheProbe>,
) -> Result<MountedRuntime<RecordedMemoryDevice>, String> {
    let clock_of_the_runtime: Arc<ClockMovedByTheProbe> = Arc::clone(clock);
    match MountedRuntime::mount_without_the_maintenance_thread(
        pool_parameters,
        devices,
        runtime_parameters,
        None,
        clock_of_the_runtime,
        Arc::new(IgnoringTheMaintenanceEvents),
    ) {
        Ok(RuntimeMounted { runtime, .. }) => Ok(runtime),
        Err(refused) => Err(first_characters(&format!("{refused:?}"))),
    }
}

fn mounted_without_a_thread(
    pool_parameters: &MakeFilesystemParameters,
    devices: DevicesOfTheProbe,
    runtime_parameters: RuntimeParameters,
    clock: &Arc<ClockMovedByTheProbe>,
) -> MountedRuntime<RecordedMemoryDevice> {
    mounted_or_why_not(pool_parameters, devices, runtime_parameters, clock).expect("可写挂载")
}

/// 崩溃：盘从运行库里拿出来，丢掉运行库（只停线程，不写盘）。只给不起线程的运行库用。
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
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .allocator
                .frozen_publish()
                .is_some()
        })
        .expect("运行库没坏")
}

fn deletion_queue_is_empty(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .current
                .version_of_trees()
                .expect("有树的一版")
                .queue_of_committed_intents_is_empty()
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

fn barriers_recorded(stream: &RecordedStream) -> usize {
    stream
        .lock()
        .expect("录制流的锁里不 panic")
        .iter()
        .filter(|operation| matches!(operation, RecordedDeviceOperation::Barrier { .. }))
        .count()
}

/// 录制流里写的次数与字节数（写零、discard 不算）。
fn writes_recorded(stream: &RecordedStream) -> (u64, u64) {
    let mut writes = 0_u64;
    let mut bytes_written = 0_u64;
    for operation in stream.lock().expect("录制流的锁里不 panic").iter() {
        if let RecordedDeviceOperation::Write { bytes, .. } = operation {
            writes += 1;
            bytes_written += u64::try_from(bytes.len()).expect("写长");
        }
    }
    (writes, bytes_written)
}

/// 待发是怎么冻结的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhereThePublishFroze {
    PublishWhenDueOfAMaintenancePass,
    AnEarlierFsync,
}

/// fsync 返回之后发生什么（用户与盘决定的，放开扫）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AfterTheFsyncReturned {
    CrashWithTheDiskStillFailing,
    DiskHealsThenAPassThenCrash,
}

/// T3：冻结着、待发已并进冻结的那一次时 fsync 答什么；答了之后崩溃，那次改动还在不在。
/// 判别量：`fsync_outcome`、`frozen_when_the_fsync_returned`、`inode_watermark_after_the_remount` 比 `inode_watermark_before` 多没多 1。
#[test]
fn a_fsync_while_a_publish_is_frozen() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    for froze_in in [
        WhereThePublishFroze::PublishWhenDueOfAMaintenancePass,
        WhereThePublishFroze::AnEarlierFsync,
    ] {
        for after in [
            AfterTheFsyncReturned::CrashWithTheDiskStillFailing,
            AfterTheFsyncReturned::DiskHealsThenAPassThenCrash,
        ] {
            let pool = base.copied_with_an_empty_stream();
            let faults = pool.faults.clone();
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let runtime = mounted_without_a_thread(
                &pool_parameters,
                pool.devices,
                RuntimeParameters::by_default(),
                &clock,
            );
            let inode_watermark_before = inode_watermark(&runtime);
            let accepted = runtime.accept(one_inode(FIXED_WRITE_TIME_SECONDS + 1));
            let accepted_short = first_characters(&format!("{accepted:?}"));
            faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
            let earlier_fsync = match froze_in {
                WhereThePublishFroze::AnEarlierFsync => {
                    first_characters(&format!("{:?}", runtime.publish_pending_now()))
                }
                WhereThePublishFroze::PublishWhenDueOfAMaintenancePass => {
                    clock.advance_by(PAST_THE_TIME_THRESHOLD);
                    let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
                    "no_earlier_fsync".to_string()
                }
            };
            let frozen_before_the_fsync = is_frozen(&runtime);
            let fsync = runtime.publish_pending_now();
            let fsync_reported_nothing_to_do = matches!(fsync, Ok(None));
            let fsync_outcome = first_characters(&format!("{fsync:?}"));
            let frozen_when_the_fsync_returned = is_frozen(&runtime);
            match after {
                AfterTheFsyncReturned::CrashWithTheDiskStillFailing => {}
                AfterTheFsyncReturned::DiskHealsThenAPassThenCrash => {
                    faults.writes_left_to_fail.store(0, Ordering::SeqCst);
                    clock.advance_by(ONE_POLL);
                    let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
                }
            }
            let frozen_at_the_crash = is_frozen(&runtime);
            let devices = devices_after_a_crash(runtime);
            faults.writes_left_to_fail.store(0, Ordering::SeqCst);
            let clock_of_the_remount = Arc::new(ClockMovedByTheProbe::default());
            let remounted = mounted_or_why_not(
                &pool_parameters,
                devices,
                RuntimeParameters::by_default(),
                &clock_of_the_remount,
            );
            let inode_watermark_after_the_remount = match remounted {
                Ok(runtime_of_the_remount) => {
                    let watermark = inode_watermark(&runtime_of_the_remount);
                    drop(runtime_of_the_remount);
                    format!("{watermark}")
                }
                Err(refused) => format!("remount_refused:{refused}"),
            };
            println!(
                "name=fsync_while_frozen froze_in={froze_in:?} after={after:?} accepted={accepted_short} earlier_fsync={earlier_fsync} frozen_before_the_fsync={frozen_before_the_fsync} fsync_reported_nothing_to_do={fsync_reported_nothing_to_do} fsync_outcome={fsync_outcome} frozen_when_the_fsync_returned={frozen_when_the_fsync_returned} frozen_at_the_crash={frozen_at_the_crash} inode_watermark_before={inode_watermark_before} inode_watermark_after_the_remount={inode_watermark_after_the_remount}"
            );
        }
    }
}

/// 同上，真线程、真钟、产品的周期之外只把 P 压短（`T_time` 照默认 5 秒）：调用方只调 `accept` 与 `publish_pending_now`。崩溃走卸载（重发又报错，等价于崩溃）。
#[test]
fn a_fsync_while_a_publish_is_frozen_by_the_maintenance_thread() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.publish_poll_period = Duration::from_millis(5);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &pool_parameters,
        pool.devices,
        runtime_parameters,
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("可写挂载");
    let inode_watermark_before = inode_watermark(&runtime);
    let accepted = runtime
        .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .is_ok();
    faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
    let given_up_at = Instant::now() + DEADLINE_OF_THE_FREEZE;
    while !is_frozen(&runtime) && Instant::now() < given_up_at {
        std::thread::sleep(Duration::from_millis(5));
    }
    let frozen_before_the_fsync = is_frozen(&runtime);
    let fsync = runtime.publish_pending_now();
    let fsync_reported_nothing_to_do = matches!(fsync, Ok(None));
    let fsync_outcome = first_characters(&format!("{fsync:?}"));
    let observations = runtime.observations();
    let inode_watermark_after_the_remount = match runtime.unmount() {
        Ok(_unmounted) => "unmounted_clean".to_string(),
        Err(refused) => match *refused {
            UnmountRefused::FrozenPublishResendFailed { devices, .. } => {
                faults.writes_left_to_fail.store(0, Ordering::SeqCst);
                let clock = Arc::new(ClockMovedByTheProbe::default());
                match mounted_or_why_not(
                    &pool_parameters,
                    devices,
                    RuntimeParameters::by_default(),
                    &clock,
                ) {
                    Ok(runtime_of_the_remount) => {
                        let watermark = inode_watermark(&runtime_of_the_remount);
                        drop(runtime_of_the_remount);
                        format!("{watermark}")
                    }
                    Err(refused_remount) => format!("remount_refused:{refused_remount}"),
                }
            }
            other => first_characters(&format!("unmount_other:{other:?}")),
        },
    };
    println!(
        "name=fsync_while_frozen_on_the_thread accepted={accepted} frozen_before_the_fsync={frozen_before_the_fsync} fsync_reported_nothing_to_do={fsync_reported_nothing_to_do} fsync_outcome={fsync_outcome} frozen_publish_resends_failed={} inode_watermark_before={inode_watermark_before} inode_watermark_after_the_remount={inode_watermark_after_the_remount}",
        observations.frozen_publish_resends_failed,
    );
}

/// 重发成了之后调用方做什么（放开扫）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UserActionAfterTheResend {
    NothingButAccept,
    PublishUserChange,
    BarrierThroughAccept,
    RemountThenAccept,
}

/// 水位取哪一档。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WatermarksOfTheProbe {
    ProductDefaults,
    OverTheLine,
}

/// T3：unlink 的意图那次写盘报错、冻结，下一拍重发成了：意图落了盘，文件进了删除队列，走空没走。
/// 维护线程走不走它（`MaintenanceEvent::FrozenPublishResent` 的文档写「前台的写从此放行」）；`accept` 什么时候放行。
/// 判别量：`accept_went_through_at_pass`、`queue_empty_at_the_end`、`last_not_started`。不用任何只供测试的开关（水位那一臂除外）。
#[test]
fn the_deletion_queue_after_a_resent_unlink_intent() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    for watermarks in [
        WatermarksOfTheProbe::ProductDefaults,
        WatermarksOfTheProbe::OverTheLine,
    ] {
        for action in [
            UserActionAfterTheResend::NothingButAccept,
            UserActionAfterTheResend::PublishUserChange,
            UserActionAfterTheResend::BarrierThroughAccept,
            UserActionAfterTheResend::RemountThenAccept,
        ] {
            let pool = base.copied_with_an_empty_stream();
            let faults = pool.faults.clone();
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let runtime_parameters = match watermarks {
                WatermarksOfTheProbe::ProductDefaults => RuntimeParameters::by_default(),
                WatermarksOfTheProbe::OverTheLine => parameters_over_the_watermark(16),
            };
            let runtime = mounted_without_a_thread(
                &pool_parameters,
                pool.devices,
                runtime_parameters.clone(),
                &clock,
            );
            let spare_inode = InodeNumber(inode_watermark(&runtime));
            runtime
                .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
                .expect("建一个 inode");
            faults.writes_left_to_fail.store(1, Ordering::SeqCst);
            let unlink_refused = runtime
                .publish_user_change(UserChange::Unlink {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                })
                .is_err();
            let frozen_after_the_unlink = is_frozen(&runtime);
            clock.advance_by(ONE_POLL);
            let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
            let resent = runtime.observations().frozen_publishes_resent;
            let queue_empty_after_the_resend = deletion_queue_is_empty(&runtime);
            let mut accept_went_through_at_pass = None;
            let mut last_accept = String::new();
            let mut passes = 0_u32;
            for pass_number in 1..=PASSES_OF_A_SWEEP {
                clock.advance_by(ONE_PERIOD);
                let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
                passes = pass_number;
                let accepted = runtime.accept(one_inode(FIXED_WRITE_TIME_SECONDS + 3));
                last_accept = first_characters(&format!("{accepted:?}"));
                if accepted.is_ok() {
                    accept_went_through_at_pass = Some(pass_number);
                    break;
                }
            }
            let observations = runtime.observations();
            let queue_empty_at_the_end = deletion_queue_is_empty(&runtime);
            println!(
                "name=queue_after_a_resent_unlink watermarks={watermarks:?} action={action:?} unlink_refused={unlink_refused} frozen_after_the_unlink={frozen_after_the_unlink} frozen_publishes_resent={resent} queue_empty_after_the_resend={queue_empty_after_the_resend} passes={passes} simulated_seconds={} accept_went_through_at_pass={accept_went_through_at_pass:?} last_accept={last_accept} queue_empty_at_the_end={queue_empty_at_the_end} compaction_publishes={} last_not_started={:?} last_stop={:?}",
                u64::from(passes) * ONE_PERIOD.as_secs(),
                observations.compaction_publishes,
                observations.last_not_started,
                observations.last_stop,
            );
            let outcome_of_the_action = match action {
                UserActionAfterTheResend::NothingButAccept => "nothing".to_string(),
                UserActionAfterTheResend::PublishUserChange => first_characters(&format!(
                    "{}",
                    runtime
                        .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 4))
                        .is_ok()
                )),
                UserActionAfterTheResend::BarrierThroughAccept => first_characters(&format!(
                    "{}",
                    runtime
                        .accept(UserChange::Unlink {
                            inode: spare_inode,
                            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 4,
                        })
                        .is_ok()
                )),
                UserActionAfterTheResend::RemountThenAccept => "remount".to_string(),
            };
            if action == UserActionAfterTheResend::RemountThenAccept {
                let devices = devices_after_a_crash(runtime);
                let clock_of_the_remount = Arc::new(ClockMovedByTheProbe::default());
                let remounted = mounted_or_why_not(
                    &pool_parameters,
                    devices,
                    runtime_parameters,
                    &clock_of_the_remount,
                );
                match remounted {
                    Ok(runtime_of_the_remount) => {
                        let accepted = runtime_of_the_remount
                            .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 5))
                            .is_ok();
                        println!(
                            "name=queue_after_the_action watermarks={watermarks:?} action={action:?} outcome={outcome_of_the_action} queue_empty={} accept_went_through={accepted}",
                            deletion_queue_is_empty(&runtime_of_the_remount),
                        );
                    }
                    Err(refused) => println!(
                        "name=queue_after_the_action watermarks={watermarks:?} action={action:?} remount_refused={refused}"
                    ),
                }
            } else {
                let accepted = runtime
                    .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 5))
                    .is_ok();
                println!(
                    "name=queue_after_the_action watermarks={watermarks:?} action={action:?} outcome={outcome_of_the_action} queue_empty={} accept_went_through={accepted}",
                    deletion_queue_is_empty(&runtime),
                );
            }
        }
    }
}

/// T1：一次用户改动冻结、重发成了，与同一次改动一次发成（对照）比：那一份 `B_gc` 给没给（`MountedSession::resend_the_frozen_publish` 的文档只说整理的计数「差一次」）。
/// 摆：预算先记成扣完（闲置池上真会走到这一格）。判别量：之后几拍整理发了几次、`last_not_started`。
#[test]
fn a_resent_user_change_against_one_that_went_through_at_once() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let base = pool_with_holes_in_the_data_segments();
    for write_fails_once in [false, true] {
        let pool = base.copied_with_an_empty_stream();
        let faults = pool.faults.clone();
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let runtime = mounted_without_a_thread(
            &pool_parameters,
            pool.devices,
            parameters_over_the_watermark(16),
            &clock,
        );
        runtime
            .with_the_session_and_devices(|locked| {
                locked
                    .session
                    .session_for_tests_only()
                    .compaction
                    .publishes_since_the_last_user_change = 1_000;
            })
            .expect("运行库没坏");
        if write_fails_once {
            faults.writes_left_to_fail.store(1, Ordering::SeqCst);
        }
        let change_went_through = runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .is_ok();
        for _pass in 0..4 {
            clock.advance_by(ONE_PERIOD);
            let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
        }
        let observations = runtime.observations();
        println!(
            "name=resent_user_change_budget write_fails_once={write_fails_once} change_went_through={change_went_through} frozen_publishes_resent={} compaction_publishes={} publishes_since_the_last_user_change={} last_not_started={:?} last_stop={:?}",
            observations.frozen_publishes_resent,
            observations.compaction_publishes,
            publishes_since_the_last_user_change(&runtime),
            observations.last_not_started,
            observations.last_stop,
        );
    }
}

/// 冻结那一次报错在哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhereTheWriteFailed {
    TheFirstWriteOfThePublish,
    TheLastBarrierOfThePublishAfterTheRootLanded,
}

/// T1：卸载先重发、重发又报错。`UnmountRefused::FrozenPublishResendFailed` 的文档写「冻结的那一次没落盘，下一次挂载走恢复，回到它之前最后发成的那一版」。
/// 报错在根槽 FUA 之后的那道屏障上时那一次的根已经落盘。判别量：再挂之后 inode 水位比改动之前多没多 1。
#[test]
fn unmounting_when_the_resend_fails_after_the_root_landed() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    let barriers_of_one_publish = {
        let pool = base.copied_with_an_empty_stream();
        let stream = Arc::clone(&pool.stream);
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let runtime = mounted_without_a_thread(
            &pool_parameters,
            pool.devices,
            RuntimeParameters::by_default(),
            &clock,
        );
        let before = barriers_recorded(&stream);
        runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .expect("建一个 inode");
        barriers_recorded(&stream) - before
    };
    for failed_at in [
        WhereTheWriteFailed::TheFirstWriteOfThePublish,
        WhereTheWriteFailed::TheLastBarrierOfThePublishAfterTheRootLanded,
    ] {
        let pool = base.copied_with_an_empty_stream();
        let faults = pool.faults.clone();
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let runtime = mounted_without_a_thread(
            &pool_parameters,
            pool.devices,
            RuntimeParameters::by_default(),
            &clock,
        );
        let inode_watermark_before = inode_watermark(&runtime);
        match failed_at {
            WhereTheWriteFailed::TheFirstWriteOfThePublish => {
                faults.writes_left_to_fail.store(u64::MAX, Ordering::SeqCst);
            }
            WhereTheWriteFailed::TheLastBarrierOfThePublishAfterTheRootLanded => {
                faults.barriers_let_through_before_the_faults.store(
                    u64::try_from(barriers_of_one_publish).expect("屏障数") - 1,
                    Ordering::SeqCst,
                );
                faults
                    .barriers_left_to_fail
                    .store(u64::MAX, Ordering::SeqCst);
            }
        }
        let change_refused = runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .is_err();
        let frozen = is_frozen(&runtime);
        let (verdict, inode_watermark_after_the_remount) = match runtime.unmount() {
            Ok(_unmounted) => ("unmounted_clean".to_string(), "not_remounted".to_string()),
            Err(refused) => match *refused {
                UnmountRefused::FrozenPublishResendFailed { devices, .. } => {
                    faults.writes_left_to_fail.store(0, Ordering::SeqCst);
                    faults.barriers_left_to_fail.store(0, Ordering::SeqCst);
                    let clock_of_the_remount = Arc::new(ClockMovedByTheProbe::default());
                    let watermark = match mounted_or_why_not(
                        &pool_parameters,
                        devices,
                        RuntimeParameters::by_default(),
                        &clock_of_the_remount,
                    ) {
                        Ok(runtime_of_the_remount) => {
                            let watermark = inode_watermark(&runtime_of_the_remount);
                            drop(runtime_of_the_remount);
                            format!("{watermark}")
                        }
                        Err(refused_remount) => format!("remount_refused:{refused_remount}"),
                    };
                    ("frozen_publish_resend_failed".to_string(), watermark)
                }
                other => (
                    first_characters(&format!("{other:?}")),
                    "not_remounted".to_string(),
                ),
            },
        };
        println!(
            "name=unmount_resend_failed failed_at={failed_at:?} barriers_of_one_publish={barriers_of_one_publish} change_refused={change_refused} frozen={frozen} verdict={verdict} inode_watermark_before={inode_watermark_before} inode_watermark_after_the_remount={inode_watermark_after_the_remount}"
        );
    }
}

/// T1：盘持续报错（第一道屏障报错，单元都写过一遍）时每一拍重发写多少。判别量：每拍写的次数与字节、重发失败计数。默认 P 是 1 秒，乘 3600 是每小时的量（推的算术）。
#[test]
fn what_each_resend_writes_while_the_disk_keeps_failing() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    for data_units in [1_usize, 32] {
        let pool = base.copied_with_an_empty_stream();
        let faults = pool.faults.clone();
        let stream = Arc::clone(&pool.stream);
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let runtime = mounted_without_a_thread(
            &pool_parameters,
            pool.devices,
            RuntimeParameters::by_default(),
            &clock,
        );
        let inode = InodeNumber(inode_watermark(&runtime));
        runtime
            .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
            .expect("建一个 inode");
        let content = vec![0x5A_u8; data_units * data_unit_payload_capacity()];
        faults
            .barriers_left_to_fail
            .store(u64::MAX, Ordering::SeqCst);
        let write_refused = runtime
            .publish_user_change(UserChange::SequentialWriteOfAnInode {
                inode,
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            })
            .is_err();
        let frozen = is_frozen(&runtime);
        for pass_number in 1..=5_u32 {
            let (writes_before, bytes_before) = writes_recorded(&stream);
            clock.advance_by(ONE_POLL);
            let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
            let (writes_after, bytes_after) = writes_recorded(&stream);
            println!(
                "name=resend_without_a_backoff data_units={data_units} write_refused={write_refused} frozen={frozen} pass={pass_number} writes_of_the_pass={} bytes_of_the_pass={} frozen_publish_resends_failed={}",
                writes_after - writes_before,
                bytes_after - bytes_before,
                runtime.observations().frozen_publish_resends_failed,
            );
        }
    }
}

/// 把一个 inode 放进删除队列（意图那一次发布，不走空）。同入库用例 `mount_runtime_compacts_by_itself_and_unmounts_clean.rs` 的 `put_an_inode_into_the_deletion_queue`。
fn put_an_inode_into_the_deletion_queue(
    locked: &mut SessionAndDevices<RecordedMemoryDevice>,
    inode: InodeNumber,
) {
    let SessionAndDevices { session, devices } = locked;
    let session = session.session_for_tests_only();
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

/// 一批只放一个单元：有数据单元的文件要走很多步（40 个数据单元，走 40 步）。
fn delete_one_data_unit_per_step(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let mut switches = session.allocator.deletion_switches();
            switches.batch_size = UnlinkBatchSize::CappedForTests(
                UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 是合法的批大小"),
            );
            session.allocator.set_deletion_switches(switches);
        })
        .expect("运行库没坏");
}

/// `work` 干活时另一条线程一直取那把锁：两次取到锁之间 txg 涨得最多的那一次，就是一次持锁里发了几次（放锁之后被插队时也算进去，所以只算上界）。
fn most_publishes_between_two_holds_of_the_lock(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    work: impl FnOnce(),
) -> (u64, u64) {
    let stop = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let reader = scope.spawn(|| {
            let mut last_txg = current_txg(runtime).0;
            let mut most = 0_u64;
            let mut holds = 0_u64;
            loop {
                let is_the_last_read = stop.load(Ordering::SeqCst);
                let txg = current_txg(runtime).0;
                most = most.max(txg - last_txg);
                last_txg = txg;
                holds += 1;
                if is_the_last_read {
                    return (most, holds);
                }
                std::thread::yield_now();
            }
        });
        work();
        stop.store(true, Ordering::SeqCst);
        reader.join().expect("读的线程不 panic")
    })
}

/// T3：`run_one_maintenance_pass` 之外绕过放锁的路。同一条 40 步的删除队列：后台入口走（修补改的那条路）；前台 `publish_user_change` 走（`UserChange::Unlink` 的文档：一次调用走完全部批）。
/// 判别量：`most_publishes_between_two_holds`、一共发了几次。
#[test]
fn how_long_the_lock_is_held_while_a_long_queue_is_walked() {
    let _only_this = only_this_probe();
    let pool_parameters = parameters();
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    for walked_by in ["background_entry", "foreground_publish_user_change"] {
        let pool = base.copied_with_an_empty_stream();
        let clock = Arc::new(ClockMovedByTheProbe::default());
        let runtime = mounted_without_a_thread(
            &pool_parameters,
            pool.devices,
            parameters_over_the_watermark(16),
            &clock,
        );
        delete_one_data_unit_per_step(&runtime);
        let txg_before = current_txg(&runtime);
        let (most, holds) = if walked_by == "background_entry" {
            runtime
                .with_the_session_and_devices(|locked| {
                    put_an_inode_into_the_deletion_queue(locked, InodeNumber(FIRST_INODE_NUMBER));
                })
                .expect("运行库没坏");
            most_publishes_between_two_holds_of_the_lock(&runtime, || {
                for _pass in 0..PASSES_OF_A_SWEEP {
                    clock.advance_by(ONE_PERIOD);
                    let _pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
                    if deletion_queue_is_empty(&runtime) {
                        break;
                    }
                }
            })
        } else {
            most_publishes_between_two_holds_of_the_lock(&runtime, || {
                let _published = runtime
                    .publish_user_change(UserChange::Unlink {
                        inode: InodeNumber(FIRST_INODE_NUMBER),
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                    })
                    .is_ok();
            })
        };
        println!(
            "name=lock_hold_of_a_queue_walk walked_by={walked_by} publishes_in_total={} most_publishes_between_two_holds={most} holds_of_the_reader={holds} queue_empty={}",
            current_txg(&runtime).0 - txg_before.0,
            deletion_queue_is_empty(&runtime),
        );
    }
}

/// 在 `base` 的拷贝上，一次挂载里发 `states` 次用户改动，丢掉会话（崩溃）。交回盘与每次改动的 txg。
fn pool_with_user_states(
    pool_parameters: &MakeFilesystemParameters,
    base: &PoolOnMemoryDevices,
    states: usize,
) -> (PoolOnMemoryDevices, Vec<CheckpointTxg>) {
    let mut pool = base.copied_with_an_empty_stream();
    let mounted = mount_writable(pool_parameters, &mut pool.devices).expect("可写挂载");
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

/// 根环里读得出、不低于 F 的根属于几个用户状态（某次用户改动的 txg 起到下一次用户改动之前算一个状态）。
/// 判法同入库用例 `mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs` 的 `user_states_left_after_the_next_publishes`。
fn user_states_now(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    pool_parameters: &MakeFilesystemParameters,
    txgs_of_the_user_changes: &[CheckpointTxg],
) -> usize {
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
                            && root.checkpoint_txg >= **txg
                            && next.is_none_or(|next_txg| root.checkpoint_txg < *next_txg)
                    })
                })
                .count()
        })
        .expect("运行库没坏")
}

/// 强制选数据单元最多的那一段（同入库用例 `force_the_fullest_data_segment`）；已经有活意图时不动选段判据以外的东西。
fn force_the_fullest_data_segment(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
            for record in session
                .current
                .version_of_trees()
                .expect("有树的一版")
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

/// 按产品线程体的节奏走几拍（一趟没完隔一个 P，否则隔一个周期），走到门关或走满。
fn passes_until_the_gate_closes(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    clock: &ClockMovedByTheProbe,
    passes_at_most: usize,
) {
    let mut is_the_trip_under_way = false;
    for _pass in 0..passes_at_most {
        clock.advance_by(if is_the_trip_under_way {
            ONE_POLL
        } else {
            ONE_PERIOD
        });
        let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
        is_the_trip_under_way = pass.is_a_trip_under_way;
        if is_stopped_at_the_gate(runtime) {
            break;
        }
    }
}

/// T1：余量取 S 之后，S = 5、6、7、8 与批大小 16、8、4 上：门关之前发几次、一条意图做不做得完；门关着带活意图崩溃，每次挂载之后环里还剩几个用户状态（连崩撑几次）。
/// 判别量：`states_after_the_mount` 第一次低于 4 的那一次挂载；`publishes_until_the_stop`、`intents_completed`、`live_intent`。
#[test]
fn how_many_crashes_the_margin_holds_under_every_geometry_and_batch_size() {
    let _only_this = only_this_probe();
    for slots_per_region in [5_u64, 6, 7, 8] {
        let pool_parameters = parameters_under(slots_per_region);
        let base = pool_with_holes_in_the_data_segments_under(&pool_parameters);
        for batch_units in [16_usize, 8, 4] {
            let (pool, txgs) =
                pool_with_user_states(&pool_parameters, &base, USER_STATES_OF_THE_HISTORIES);
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let runtime = mounted_without_a_thread(
                &pool_parameters,
                pool.devices,
                parameters_over_the_watermark(batch_units),
                &clock,
            );
            force_the_fullest_data_segment(&runtime);
            passes_until_the_gate_closes(&runtime, &clock, 8);
            let observations = runtime.observations();
            println!(
                "name=margin_until_the_stop s={slots_per_region} batch_units={batch_units} publishes_until_the_stop={} intents_completed={} live_intent={} last_stop={:?} user_states_now={}",
                observations.compaction_publishes,
                intents_completed(&runtime),
                has_a_live_intent(&runtime),
                observations.last_stop,
                user_states_now(&runtime, &pool_parameters, &txgs),
            );
            let mut txg_before = current_txg(&runtime);
            let mut devices = devices_after_a_crash(runtime);
            for mount in 1..=5_u32 {
                let clock_of_the_mount = Arc::new(ClockMovedByTheProbe::default());
                let runtime_of_the_mount = match mounted_or_why_not(
                    &pool_parameters,
                    devices,
                    parameters_over_the_watermark(batch_units),
                    &clock_of_the_mount,
                ) {
                    Ok(runtime_of_the_mount) => runtime_of_the_mount,
                    Err(refused) => {
                        println!(
                            "name=margin_crash_loop s={slots_per_region} batch_units={batch_units} mount={mount} remount_refused={refused}"
                        );
                        break;
                    }
                };
                let txgs_advanced_by_the_mount =
                    current_txg(&runtime_of_the_mount).0 - txg_before.0;
                let states_after_the_mount =
                    user_states_now(&runtime_of_the_mount, &pool_parameters, &txgs);
                force_the_fullest_data_segment(&runtime_of_the_mount);
                passes_until_the_gate_closes(&runtime_of_the_mount, &clock_of_the_mount, 3);
                let observations_of_the_mount = runtime_of_the_mount.observations();
                println!(
                    "name=margin_crash_loop s={slots_per_region} batch_units={batch_units} mount={mount} txgs_advanced_by_the_mount={txgs_advanced_by_the_mount} states_after_the_mount={states_after_the_mount} compaction_publishes={} states_after_the_passes={} last_stop={:?} last_not_started={:?}",
                    observations_of_the_mount.compaction_publishes,
                    user_states_now(&runtime_of_the_mount, &pool_parameters, &txgs),
                    observations_of_the_mount.last_stop,
                    observations_of_the_mount.last_not_started,
                );
                txg_before = current_txg(&runtime_of_the_mount);
                devices = devices_after_a_crash(runtime_of_the_mount);
            }
        }
    }
}
