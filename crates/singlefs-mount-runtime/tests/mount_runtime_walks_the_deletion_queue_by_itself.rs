//! 后台入口自己走删除队列（D26（后台整理与放置回收） 已定项 13）：队列不空算起跑，只走队列、不发整理，不看水位、不被预算挡；调用方挂载之后不调任何后台入口。
//! 删除次序只有一处（`transaction::publish_the_next_step_of_the_deletion_queue`）：放锁的入口一次走一步、步与步之间插一次前台改动，终态与一次走空逐字节相同。
//! 另量写盘报错之后又写满时队列走不走得动（读数打成 `name=` 行）。
mod common_pool_on_memory_devices;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common_pool_on_memory_devices::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount,
    pool_with_holes_in_the_data_segments, PoolOnMemoryDevices, RecordedMemoryDevice,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchUnits, CompactionMountParameters,
    CompactionPacing, CompactionRunStop, CompactionWatermarks, EmptyClusterSegmentsWatermark,
    FragmentationWatermark, UnreclaimableBytesWatermark,
};
use singlefs_core::mounted_session::{
    AcceptRefused, BackgroundCompactionCall, BackgroundCompactionNotStarted,
    BackgroundCompactionTrigger, CompactionRunFailure, CompactionRunOutcome, UserChange,
};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::transaction::{
    publish_size_change, publish_snapshot_destruction_intent, publish_unlink_intent, PoolVersion,
    PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MountedRuntime, RuntimeMounted,
    RuntimeParameters, RuntimeRefused, SessionAndDevices,
};

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
/// 摆成那一份预算早已扣完的发布数。
const PUBLISHES_SPENDING_THE_BUDGET: u64 = 1_000;
/// 一次一步走队列的步数上界（40 个数据单元的文件一批放一个单元，连上最后一批；销毁快照、截断都比它少）。
const STEPS_OF_A_WALK_AT_MOST: usize = 64;
/// 判「走不动」走几次。
const CALLS_OF_THE_STUCK_READING: usize = 3;

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

/// 一批只放一个单元：有数据单元的文件要走很多步。
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

/// 放锁的后台入口调一次。
fn one_call_of_at_most_one_publish(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
) -> BackgroundCompactionCall {
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session
                .run_compaction_in_the_background_for_at_most_one_publish(
                    devices,
                    CompactionPacing::BY_DEFAULT,
                    &CallerNotWaiting,
                )
                .expect("盘表是挂载的那一份")
                .call
        })
        .expect("运行库没坏")
}

/// 一次一步把队列走空：每一次调用都因队列不空起跑、恰好一步删除步、一次整理都不发，最后一步停在「队列走空」上。交回走了几步。
fn walk_the_queue_one_call_at_a_time(runtime: &MountedRuntime<RecordedMemoryDevice>) -> usize {
    let mut steps = 0;
    // 迭代上界：每一轮发成一步或断言红；队列有限（同 `walk_the_deletion_queue_until_it_is_empty` 的上界）。
    while !deletion_queue_is_empty(runtime) {
        assert!(steps < STEPS_OF_A_WALK_AT_MOST, "一步走不空");
        let BackgroundCompactionCall::Ran { trigger, run } =
            one_call_of_at_most_one_publish(runtime)
        else {
            panic!("队列不空：起跑");
        };
        assert_eq!(trigger, BackgroundCompactionTrigger::DeletionQueueNotEmpty);
        assert_eq!(run.versions_published.len(), 1, "一次调用至多一步");
        assert_eq!(run.deletion_steps_published, 1, "这一步是删除步");
        assert_eq!(run.batches_published, 0, "只走队列，不发整理");
        steps += 1;
        let expected_stop = if deletion_queue_is_empty(runtime) {
            CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish
        } else {
            CompactionRunStop::OnePublishOfTheTripDone
        };
        assert!(
            matches!(run.outcome, CompactionRunOutcome::Stopped(stop) if stop == expected_stop),
            "第 {steps} 步：{:?}",
            run.outcome
        );
    }
    steps
}

/// unlink 的意图写盘报错、冻结，下一拍重发成了：水位没越线（产品默认）、调用方只调 `accept`：第 1 或第 2 拍写得进。
#[test]
fn after_a_resent_unlink_the_queue_is_walked_under_the_product_watermarks_and_accept_goes_through()
{
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let faults = pool.faults.clone();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    runtime
        .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 1))
        .expect("建一个 inode");
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            })
            .is_err(),
        "写盘报错：unlink 没发成"
    );
    clock.advance_by(ONE_POLL);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(runtime.observations().frozen_publishes_resent, 1);
    let mut accept_went_through_at_pass = None;
    for pass_number in 1..=2 {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
        if runtime
            .accept(one_inode(FIXED_WRITE_TIME_SECONDS + 3))
            .is_ok()
        {
            accept_went_through_at_pass = Some(pass_number);
            break;
        }
    }
    let observations = runtime.observations();
    assert!(
        accept_went_through_at_pass.is_some(),
        "第 1 或第 2 拍写得进：{observations:?}"
    );
    assert!(deletion_queue_is_empty(&runtime));
    assert!(
        observations.deletion_continuation_publishes > 0,
        "队列是后台入口走的：{observations:?}"
    );
    assert_eq!(
        observations.compaction_publishes, 0,
        "水位没越线：删除步不算整理发布：{observations:?}"
    );
}

/// 预算扣完而队列不空：队列照走（删除续做不占预算），走完之后整理照旧被预算挡。
#[test]
fn with_the_budget_spent_a_non_empty_queue_is_still_walked() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    put_the_first_file_into_the_deletion_queue(&runtime);
    spend_the_budget(&runtime);
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        deletion_queue_is_empty(&runtime),
        "预算扣完了队列照走：{pass:?}"
    );
    assert!(pass.deletion_continuation_publishes > 0, "{pass:?}");
    assert_eq!(pass.compaction_publishes, 0, "{pass:?}");
    assert_eq!(
        runtime.observations().last_not_started,
        Some(BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent),
        "队列走完：整理照旧被预算挡"
    );
}

/// 水位越线、预算有余的池上：队列不空的那几次调用只走队列，不发整理；走空之后下一次调用才判整理。
#[test]
fn over_the_watermark_the_calls_started_by_a_non_empty_queue_publish_no_compaction() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    delete_one_data_unit_per_step(&runtime);
    put_the_first_file_into_the_deletion_queue(&runtime);
    let steps = walk_the_queue_one_call_at_a_time(&runtime);
    assert!(steps > 1, "一批一个单元：走了不止一步");
    let next_call = one_call_of_at_most_one_publish(&runtime);
    assert!(
        !matches!(
            next_call,
            BackgroundCompactionCall::Ran {
                trigger: BackgroundCompactionTrigger::DeletionQueueNotEmpty,
                ..
            }
        ),
        "队列空了：不再因它起跑：{next_call:?}"
    );
}

/// 整趟入口：一次调用把队列走空就停，同样不发整理。
#[test]
fn the_whole_trip_entry_started_by_a_non_empty_queue_walks_it_and_stops() {
    let pool = pool_with_holes_in_the_data_segments();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    put_the_first_file_into_the_deletion_queue(&runtime);
    let call = runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session
                .session_for_tests_only()
                .run_compaction_in_the_background(
                    devices,
                    CompactionPacing::BY_DEFAULT,
                    &CallerNotWaiting,
                )
                .expect("盘表是挂载的那一份")
        })
        .expect("运行库没坏");
    assert!(deletion_queue_is_empty(&runtime));
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("队列不空：起跑");
    };
    assert_eq!(trigger, BackgroundCompactionTrigger::DeletionQueueNotEmpty);
    assert_eq!(
        u64::try_from(run.versions_published.len()).expect("发布数"),
        run.deletion_steps_published,
        "发成的全是删除步"
    );
    assert!(matches!(
        run.outcome,
        CompactionRunOutcome::Stopped(
            CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish
        )
    ));
}

/// 只走队列的那一趟没完时前台把队列走空了：这一趟到此为止，下一次调用照常判，不停在这一趟上。
#[test]
fn queue_walked_by_the_foreground_ends_the_trip_that_only_walks_the_queue() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    delete_one_data_unit_per_step(&runtime);
    put_the_first_file_into_the_deletion_queue(&runtime);
    let first_call = one_call_of_at_most_one_publish(&runtime);
    assert!(matches!(
        first_call,
        BackgroundCompactionCall::Ran {
            trigger: BackgroundCompactionTrigger::DeletionQueueNotEmpty,
            ..
        }
    ));
    assert!(!deletion_queue_is_empty(&runtime), "一步走不空");
    runtime
        .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 3))
        .expect("前台改动先走空队列");
    assert!(deletion_queue_is_empty(&runtime));
    let next_call = one_call_of_at_most_one_publish(&runtime);
    assert!(
        !matches!(
            next_call,
            BackgroundCompactionCall::Ran {
                trigger: BackgroundCompactionTrigger::DeletionQueueNotEmpty,
                ..
            }
        ),
        "{next_call:?}"
    );
}

/// `base` 的拷贝上：一批一个单元、放进删除队列，先照 `calls_before_the_change` 走几步，再发一次前台改动（它先走空队列）。交回两块盘写过的扇区。
fn sectors_after_the_walk_and_a_foreground_change(
    base: &PoolOnMemoryDevices,
    calls_before_the_change: usize,
) -> Vec<(DeviceIdentity, std::collections::BTreeMap<u64, Vec<u8>>)> {
    let pool = base.copied_with_an_empty_stream();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    delete_one_data_unit_per_step(&runtime);
    put_the_first_file_into_the_deletion_queue(&runtime);
    for _call in 0..calls_before_the_change {
        drop(one_call_of_at_most_one_publish(&runtime));
    }
    runtime
        .publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 3))
        .expect("前台改动先走空队列再发");
    assert!(deletion_queue_is_empty(&runtime));
    let devices = runtime
        .with_the_session_and_devices(|locked| std::mem::take(&mut locked.devices))
        .expect("运行库没坏");
    PoolOnMemoryDevices {
        devices,
        stream: Arc::clone(&pool.stream),
        faults: pool.faults.clone(),
    }
    .written_sectors_of_every_device()
}

/// 走一步之后插一次前台改动：走到的还是同一串步，终态与一次走空再发那次改动逐字节相同。
#[test]
fn one_step_then_a_foreground_change_ends_byte_for_byte_as_walking_the_queue_at_once() {
    let base = pool_after_a_sequential_write_in_an_earlier_mount();
    assert_eq!(
        sectors_after_the_walk_and_a_foreground_change(&base, 1),
        sectors_after_the_walk_and_a_foreground_change(&base, 0)
    );
}

/// 销毁中的快照走至多一步的入口：一次调用一步，走得空。
#[test]
fn the_snapshot_being_destroyed_is_walked_one_step_per_call() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    runtime
        .publish_user_change(UserChange::CreateSnapshot)
        .expect("建快照");
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            let session = session.session_for_tests_only();
            let pool_parameters = parameters();
            let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
            let version = session.current.version_of_trees().expect("有树的一版");
            let snapshot = version
                .tree_table_entries
                .iter()
                .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
                .expect("建快照那一版树表里有码 14 那一条")
                .tree;
            let after_the_intent = publish_snapshot_destruction_intent(
                &mut writer,
                &mut session.allocator,
                version,
                snapshot,
                session.instance,
            )
            .expect("点名的快照在树表里");
            session.current = PoolVersion::of_a_version_of_trees(after_the_intent);
        })
        .expect("运行库没坏");
    assert!(!deletion_queue_is_empty(&runtime), "销毁中的快照进了队列");
    assert!(walk_the_queue_one_call_at_a_time(&runtime) >= 1);
}

/// 截断走至多一步的入口：一次调用一步，走得空。
#[test]
fn truncation_is_walked_one_step_per_call() {
    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, RuntimeParameters::by_default(), &clock);
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            let session = session.session_for_tests_only();
            let pool_parameters = parameters();
            let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
            let version = session.current.version_of_trees().expect("有树的一版");
            let after_the_intent = publish_size_change(
                &mut writer,
                &mut session.allocator,
                version,
                InodeNumber(FIRST_INODE_NUMBER),
                u64::try_from(data_unit_payload_capacity()).expect("净荷"),
                FIXED_WRITE_TIME_SECONDS + 16,
                session.instance,
            )
            .expect("第一个文件截到一个单元");
            session.current = PoolVersion::of_a_version_of_trees(after_the_intent);
        })
        .expect("运行库没坏");
    assert!(!deletion_queue_is_empty(&runtime), "截断条目进了队列");
    assert!(walk_the_queue_one_call_at_a_time(&runtime) >= 1);
}

/// 把两块盘的分配器摆成写满：每个空槽都记成已分配（盘上不动）。走删除步被「落得下」那一判拒。
fn every_free_slot_marked_allocated(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            for device_map in &mut session.allocator.devices {
                let first_slot = device_map.unit_area_start().slot().0;
                // 迭代上界是单元区槽数，跨轮只改这一槽的分配位。
                for slot in first_slot..first_slot + device_map.unit_area_slots() {
                    if device_map.is_free(SlotNumber(slot)) {
                        device_map.mark_allocated(SlotNumber(slot), 1);
                    }
                }
            }
        })
        .expect("运行库没坏");
}

/// 两次故障叠加：unlink 的意图写盘报错、重发成了，接着写满，删除步被空间拒。后台入口走不动（不推抬 F），也轮不到整理（队列不空只走队列），前台 `accept` 照拒：
/// 这条用例量这一格，钉「每次都这样、盘上逐字节不变」，走不走得出来归条款（报告「停下交主 agent 的」）。前台 `publish_user_change` 那一条路（走空时推抬 F）只打读数。
#[test]
fn with_the_deletion_step_refused_for_space_the_background_cannot_move_the_queue_and_the_compaction_gets_no_turn(
) {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let stream = Arc::clone(&pool.stream);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let runtime = mounted_without_a_thread(pool.devices, parameters_over_the_watermark(), &clock);
    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            })
            .is_err(),
        "写盘报错：unlink 没发成"
    );
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session.resend_the_frozen_publish(devices)
        })
        .expect("运行库没坏")
        .expect("这一遍不报错");
    assert!(!deletion_queue_is_empty(&runtime), "意图落了盘，队列没走");
    every_free_slot_marked_allocated(&runtime);
    let operations_before = stream.lock().expect("录制流的锁里不 panic").len();
    for call_number in 1..=CALLS_OF_THE_STUCK_READING {
        let call = one_call_of_at_most_one_publish(&runtime);
        let BackgroundCompactionCall::Ran { trigger, run } = call else {
            panic!("第 {call_number} 次：队列不空，起跑");
        };
        assert_eq!(trigger, BackgroundCompactionTrigger::DeletionQueueNotEmpty);
        assert!(run.versions_published.is_empty(), "第 {call_number} 次");
        assert_eq!(run.batches_published, 0, "第 {call_number} 次：整理轮不到");
        assert!(
            matches!(
                run.outcome,
                CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(
                    PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. }
                ))
            ),
            "第 {call_number} 次：删除步被空间拒：{:?}",
            run.outcome
        );
    }
    assert_eq!(
        stream.lock().expect("录制流的锁里不 panic").len(),
        operations_before,
        "被空间拒：盘上一个动作都没有"
    );
    assert!(!deletion_queue_is_empty(&runtime));
    assert!(matches!(
        runtime.accept(one_inode(FIXED_WRITE_TIME_SECONDS + 3)),
        Err(RuntimeRefused::Refused(
            AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion
        ))
    ));
    let foreground = runtime.publish_user_change(one_inode(FIXED_WRITE_TIME_SECONDS + 3));
    let foreground_reading: String = format!("{foreground:?}").chars().take(200).collect();
    println!(
        "name=queue_walk_refused_for_space calls={CALLS_OF_THE_STUCK_READING} foreground_publish_user_change_went_through={} queue_empty_after_it={} foreground_outcome={foreground_reading}",
        foreground.is_ok(),
        deletion_queue_is_empty(&runtime),
    );
}
