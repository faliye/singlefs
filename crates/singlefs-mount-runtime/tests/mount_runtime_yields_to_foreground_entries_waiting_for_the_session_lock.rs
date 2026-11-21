//! 运行库为前台在等会话锁的入口让路（D26（后台整理与放置回收） 已定项 13 第 6 条）的验收：收下改动、单发改动、切换后台计算、读这几个入口取会话锁取不到时登记在等、
//! 取到之后撤登记，取得到时不登记；整理在下一批之前问到有入口在等就让路、这一拍不接着叫，前台一直在等时连着让路到下限（次数或时长哪一个先到）下一批照发；
//! fsync 与别的入口都在等时答 fsync 那一个；前台 fsync（`publish_pending_now`）同样取会话锁取不到才登记有 fsync 在等。确定性的格走 `run_one_maintenance_pass` 与测试摆的钟，前台一直在等用
//! `MountedRuntime::foreground_entry_is_waiting_for_the_session_lock` 拨；登记的几格在别的线程上占着会话锁、让入口真去等。
mod common_pool_on_memory_devices;

use std::num::NonZeroU64;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, pool_with_holes_in_the_data_segments, RecordedMemoryDevice,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::background_compute::BackgroundComputeSettings;
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionMountParameters, CompactionRunStop,
    CompactionWatermarks, EmptyClusterSegmentsWatermark, FragmentationWatermark,
    UnreclaimableBytesWatermark, COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::mounted_session::UserChange;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MountedRuntime, RuntimeMounted,
    RuntimeObservations, RuntimeParameters,
};

/// 一拍之间钟走多久：默认周期。
const ONE_PERIOD: Duration = Duration::from_secs(30);
/// 次数先到的那一格的下限：连着让 2 次之后下一批不让。
const YIELDS_IN_A_ROW_OF_THE_FLOOR: NonZeroU64 = NonZeroU64::MIN.saturating_add(1);
/// 次数先到的那一格里时长的下限：钟在那几拍之间不走，到不了。
const YIELDING_TIME_NOT_REACHED_BY_THE_PASSES: Duration = Duration::from_secs(3600);
/// 时长先到的那一格里次数的下限：那几拍到不了。
const YIELDS_IN_A_ROW_NOT_REACHED_BY_THE_PASSES: NonZeroU64 = NonZeroU64::MIN.saturating_add(999);
/// 时长先到的那一格的下限：让路累计 1 秒之后下一批不让。
const YIELDING_TIME_OF_THE_FLOOR: Duration = Duration::from_secs(1);
/// 时长先到的那一格里两拍之间钟走多久：过了时长的下限。
const CLOCK_MOVED_PAST_THE_YIELDING_TIME_OF_THE_FLOOR: Duration = Duration::from_secs(2);
/// 等入口登记成在等的上限：健康时是毫秒级，这个长度只决定红得多快。
const DEADLINE_OF_THE_REGISTRATION: Duration = Duration::from_secs(60);

/// 测试摆的钟。
#[derive(Debug)]
struct ClockMovedByTheTest {
    reading: Mutex<Duration>,
}

impl ClockMovedByTheTest {
    fn starting_at(reading: Duration) -> Self {
        ClockMovedByTheTest {
            reading: Mutex::new(reading),
        }
    }

    fn advance_by(&self, elapsed: Duration) {
        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
    }
}

impl ClockOfTheRuntime for ClockMovedByTheTest {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
    }
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关；批大小取默认；让路的下限照给。
fn parameters_over_the_watermark_with_the_floor(
    yields_in_a_row: NonZeroU64,
    yielding_time: Duration,
) -> RuntimeParameters {
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
    runtime_parameters.foreground_yields_in_a_row_after_which_the_next_batch_does_not_yield =
        yields_in_a_row;
    runtime_parameters.foreground_yielding_time_after_which_the_next_batch_does_not_yield =
        yielding_time;
    runtime_parameters
}

/// 不起线程挂载带洞的池，钟摆在一个周期上（下一拍整理就到期）。
fn mounted_without_a_thread(
    runtime_parameters: RuntimeParameters,
) -> (
    MountedRuntime<RecordedMemoryDevice>,
    Arc<ClockMovedByTheTest>,
) {
    let clock = Arc::new(ClockMovedByTheTest::starting_at(ONE_PERIOD));
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool_with_holes_in_the_data_segments().devices,
        runtime_parameters,
        None,
        Arc::clone(&clock) as Arc<dyn ClockOfTheRuntime>,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("全合法的参数：挂载照常做成");
    (runtime, clock)
}

/// 前台一直有入口在等时的一拍：让路，一批都不发，这一拍只叫这一次，这一趟留着。
fn assert_the_pass_yields_to_the_entry_waiting(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    yields_so_far: u64,
) {
    let held_back = runtime.run_one_maintenance_pass().expect("运行库没坏");
    let observations = runtime.observations();
    assert_eq!(
        held_back.compaction_publishes, 0,
        "连着让了 {yields_so_far} 次：这一拍一批都不发：{held_back:?}"
    );
    assert_eq!(
        held_back.compaction_calls, 1,
        "让路就停这一拍，不接着叫：{held_back:?}"
    );
    assert!(held_back.is_a_trip_under_way, "这一趟留着：{held_back:?}");
    assert_eq!(
        observations.last_stop,
        Some(CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock)
    );
    assert_eq!(
        observations.yields_to_a_foreground_entry_waiting_for_the_session_lock,
        yields_so_far
    );
}

/// 前台一直有入口在等会话锁（连着让路的次数先到下限）：让 2 次之后下一批不让、照发。
// 不切片并行：一个池、同步的几拍，被测的是同一份会话上前后几拍连着让路的计数，没有可切的区间。
#[test]
fn an_entry_waiting_for_the_session_lock_holds_back_the_trip_until_the_yields_in_a_row_reach_the_floor(
) {
    let (runtime, _clock) = mounted_without_a_thread(parameters_over_the_watermark_with_the_floor(
        YIELDS_IN_A_ROW_OF_THE_FLOOR,
        YIELDING_TIME_NOT_REACHED_BY_THE_PASSES,
    ));
    let _waiting = runtime.foreground_entry_is_waiting_for_the_session_lock();
    for yields_so_far in 1..=YIELDS_IN_A_ROW_OF_THE_FLOOR.get() {
        assert_the_pass_yields_to_the_entry_waiting(&runtime, yields_so_far);
    }
    let past_the_floor = runtime.run_one_maintenance_pass().expect("运行库没坏");
    let observations = runtime.observations();
    assert!(
        past_the_floor.compaction_publishes >= 1,
        "连着让路到了下限：下一批不让、照发：{past_the_floor:?}"
    );
    assert_eq!(
        observations.foreground_entries_registered_as_waiting_for_the_session_lock, 1,
        "登记了一次"
    );
    assert_eq!(observations.yields_to_a_foreground_fsync, 0);
}

/// 前台一直有入口在等会话锁（让路累计的时长先到下限）：让一次之后钟走过下限，下一批不让、照发。
// 不切片并行：一个池、同步的几拍，被测的是同一份会话上前后两拍之间钟走过的时长，没有可切的区间。
#[test]
fn an_entry_waiting_for_the_session_lock_holds_back_the_trip_until_the_yielding_time_reaches_the_floor(
) {
    let (runtime, clock) = mounted_without_a_thread(parameters_over_the_watermark_with_the_floor(
        YIELDS_IN_A_ROW_NOT_REACHED_BY_THE_PASSES,
        YIELDING_TIME_OF_THE_FLOOR,
    ));
    let _waiting = runtime.foreground_entry_is_waiting_for_the_session_lock();
    assert_the_pass_yields_to_the_entry_waiting(&runtime, 1);
    clock.advance_by(CLOCK_MOVED_PAST_THE_YIELDING_TIME_OF_THE_FLOOR);
    let past_the_floor = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        past_the_floor.compaction_publishes >= 1,
        "让路累计过了下限：下一批不让、照发：{past_the_floor:?}"
    );
}

/// 前台 fsync 与别的入口都在等：答 fsync 那一个，这一趟停在为 fsync 让路。
// 不切片并行：一个池、同步的一拍，被测的是一次问答里两份登记谁先答，没有可切的区间。
#[test]
fn with_an_fsync_and_another_entry_both_waiting_the_trip_yields_to_the_fsync() {
    let (runtime, _clock) = mounted_without_a_thread(parameters_over_the_watermark_with_the_floor(
        YIELDS_IN_A_ROW_NOT_REACHED_BY_THE_PASSES,
        YIELDING_TIME_NOT_REACHED_BY_THE_PASSES,
    ));
    let _fsync_waiting = runtime.foreground_fsync_is_waiting();
    let _entry_waiting = runtime.foreground_entry_is_waiting_for_the_session_lock();
    let held_back = runtime.run_one_maintenance_pass().expect("运行库没坏");
    let observations = runtime.observations();
    assert_eq!(
        held_back.compaction_publishes, 0,
        "一批都不发：{held_back:?}"
    );
    assert_eq!(
        observations.last_stop,
        Some(CompactionRunStop::YieldedToAForegroundFsync)
    );
    assert_eq!(observations.yields_to_a_foreground_fsync, 1);
    assert_eq!(
        observations.yields_to_a_foreground_entry_waiting_for_the_session_lock,
        0
    );
}

/// 在 `deadline` 之内等到 `condition` 成立，交回等到没有。
fn waited_until(deadline: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let given_up_at = Instant::now() + deadline;
    while !condition() {
        if Instant::now() >= given_up_at {
            return false;
        }
        std::thread::yield_now();
    }
    true
}

/// 前台 fsync 之外的入口登记成在等的次数。
fn registrations_of_the_entries_other_than_an_fsync(observations: &RuntimeObservations) -> u64 {
    observations.foreground_entries_registered_as_waiting_for_the_session_lock
}

/// 前台 fsync 登记成在等的次数。
fn registrations_of_the_fsyncs(observations: &RuntimeObservations) -> u64 {
    observations.foreground_fsyncs_registered_as_waiting_for_the_session_lock
}

/// `entry` 在会话锁被占着时来取：另一条线程占着锁，等这个入口登记成在等（`registrations_of` 读的那一份到 1，或等满上限）再放锁。
/// 交回放锁那一刻那一份登记过几次。
fn registrations_while_the_entry_waits_for_the_session_lock(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    registrations_of: fn(&RuntimeObservations) -> u64,
    entry: impl FnOnce() + Send,
) -> u64 {
    std::thread::scope(|scope| {
        let (held_sender, held_receiver) = channel::<()>();
        let (release_sender, release_receiver) = channel::<()>();
        let holder = scope.spawn(move || {
            runtime
                .with_the_session_and_devices(|_locked| {
                    held_sender.send(()).expect("用例在等占锁");
                    release_receiver.recv().expect("用例叫放锁");
                })
                .expect("运行库没坏");
        });
        held_receiver.recv().expect("另一条线程占着会话锁");
        let entry_thread = scope.spawn(entry);
        let _registered_in_time = waited_until(DEADLINE_OF_THE_REGISTRATION, || {
            registrations_of(&runtime.observations()) >= 1
        });
        let registrations = registrations_of(&runtime.observations());
        release_sender.send(()).expect("占锁那条线程在等放锁");
        holder.join().expect("占锁那条线程不 panic");
        entry_thread.join().expect("入口那条线程不 panic");
        registrations
    })
}

/// 入口等过、取到锁之后：登记撤了，下一拍整理不让路、照发。
fn assert_the_registration_is_gone_once_the_entry_took_the_lock(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    clock: &ClockMovedByTheTest,
) {
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        pass.compaction_publishes >= 1,
        "入口取到锁之后登记撤了：整理不让路：{pass:?}"
    );
    assert_eq!(
        runtime
            .observations()
            .yields_to_a_foreground_entry_waiting_for_the_session_lock,
        0
    );
}

fn parameters_whose_floor_the_passes_do_not_reach() -> RuntimeParameters {
    parameters_over_the_watermark_with_the_floor(
        YIELDS_IN_A_ROW_NOT_REACHED_BY_THE_PASSES,
        YIELDING_TIME_NOT_REACHED_BY_THE_PASSES,
    )
}

/// 读（`with_the_session_and_devices`）在会话锁被占着时来取：登记成在等一次，取到锁之后撤登记。
// 不切片并行：被测的就是两条线程在一把锁上的交错，没有可切的区间。
#[test]
fn a_read_blocked_on_the_session_lock_is_registered_as_waiting_until_it_takes_the_lock() {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    let registrations = registrations_while_the_entry_waits_for_the_session_lock(
        &runtime,
        registrations_of_the_entries_other_than_an_fsync,
        || {
            runtime
                .with_the_session_and_devices(|locked| locked.session.pending_changes_accepted())
                .expect("运行库没坏");
        },
    );
    assert_eq!(registrations, 1, "取不到锁的读登记成在等");
    assert_the_registration_is_gone_once_the_entry_took_the_lock(&runtime, &clock);
}

/// 收下改动（`accept`）在会话锁被占着时来取：登记成在等一次，取到锁之后撤登记。
// 不切片并行：被测的就是两条线程在一把锁上的交错，没有可切的区间。
#[test]
fn an_accept_blocked_on_the_session_lock_is_registered_as_waiting_until_it_takes_the_lock() {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    let registrations = registrations_while_the_entry_waits_for_the_session_lock(
        &runtime,
        registrations_of_the_entries_other_than_an_fsync,
        || {
            runtime
                .accept(UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 31,
                })
                .expect("收进待发");
        },
    );
    assert_eq!(registrations, 1, "取不到锁的收下改动登记成在等");
    assert_the_registration_is_gone_once_the_entry_took_the_lock(&runtime, &clock);
}

/// 单发改动（`publish_user_change`）在会话锁被占着时来取：登记成在等一次，取到锁之后撤登记。
// 不切片并行：被测的就是两条线程在一把锁上的交错，没有可切的区间。
#[test]
fn a_publish_of_a_user_change_blocked_on_the_session_lock_is_registered_as_waiting_until_it_takes_the_lock(
) {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    let registrations = registrations_while_the_entry_waits_for_the_session_lock(
        &runtime,
        registrations_of_the_entries_other_than_an_fsync,
        || {
            runtime
                .publish_user_change(UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 32,
                })
                .expect("建一个 inode");
        },
    );
    assert_eq!(registrations, 1, "取不到锁的单发改动登记成在等");
    assert_the_registration_is_gone_once_the_entry_took_the_lock(&runtime, &clock);
}

/// 切换后台计算（`switch_background_compute`）在会话锁被占着时来取：登记成在等一次，取到锁之后撤登记。
// 不切片并行：被测的就是两条线程在一把锁上的交错，没有可切的区间。
#[test]
fn a_switch_of_the_background_compute_blocked_on_the_session_lock_is_registered_as_waiting_until_it_takes_the_lock(
) {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    let registrations = registrations_while_the_entry_waits_for_the_session_lock(
        &runtime,
        registrations_of_the_entries_other_than_an_fsync,
        || {
            runtime
                .switch_background_compute(BackgroundComputeSettings::PRODUCT, None, None)
                .expect("切到产品那一组");
        },
    );
    assert_eq!(registrations, 1, "取不到锁的切换登记成在等");
    assert_the_registration_is_gone_once_the_entry_took_the_lock(&runtime, &clock);
}

/// 会话锁空着时四个入口都取得到：一次都不登记，下一拍整理不让路。
// 不切片并行：一个池、一条线程先后走四个入口，被测的是同一份会话上的登记计数，没有可切的区间。
#[test]
fn entries_taking_a_free_session_lock_are_not_registered_as_waiting() {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    runtime
        .with_the_session_and_devices(|locked| locked.session.pending_changes_accepted())
        .expect("运行库没坏");
    runtime
        .accept(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 33,
        })
        .expect("收进待发");
    runtime
        .publish_user_change(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 34,
        })
        .expect("建一个 inode");
    runtime
        .switch_background_compute(BackgroundComputeSettings::PRODUCT, None, None)
        .expect("切到产品那一组");
    assert_eq!(
        runtime
            .observations()
            .foreground_entries_registered_as_waiting_for_the_session_lock,
        0,
        "锁空着：一次都不登记"
    );
    assert_the_registration_is_gone_once_the_entry_took_the_lock(&runtime, &clock);
}

/// 前台 fsync（`publish_pending_now`）在会话锁空着时取得到：不登记有 fsync 在等（与别的前台入口同一写法，先试一次、取不到才登记），
/// 下一拍整理不为它让路。
// 不切片并行：一个池、一条线程先走 fsync 再走一拍，被测的是同一份会话上的登记计数，没有可切的区间。
#[test]
fn an_fsync_taking_a_free_session_lock_is_not_registered_as_waiting() {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    runtime
        .publish_pending_now()
        .expect("待发空着、没冻结：答成功");
    assert_eq!(
        registrations_of_the_fsyncs(&runtime.observations()),
        0,
        "锁空着：fsync 不登记在等"
    );
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        pass.compaction_publishes >= 1,
        "没有 fsync 在等：整理不让路：{pass:?}"
    );
    assert_eq!(runtime.observations().yields_to_a_foreground_fsync, 0);
}

/// 前台 fsync 在会话锁被占着时来取：登记有 fsync 在等一次，取到锁之后撤登记，下一拍整理不为它让路。
// 不切片并行：被测的就是两条线程在一把锁上的交错，没有可切的区间。
#[test]
fn an_fsync_blocked_on_the_session_lock_is_registered_as_waiting_until_it_takes_the_lock() {
    let (runtime, clock) =
        mounted_without_a_thread(parameters_whose_floor_the_passes_do_not_reach());
    let registrations = registrations_while_the_entry_waits_for_the_session_lock(
        &runtime,
        registrations_of_the_fsyncs,
        || {
            runtime
                .publish_pending_now()
                .expect("待发空着、没冻结：答成功");
        },
    );
    assert_eq!(registrations, 1, "取不到锁的 fsync 登记成在等");
    clock.advance_by(ONE_PERIOD);
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert!(
        pass.compaction_publishes >= 1,
        "fsync 取到锁之后登记撤了：整理不让路：{pass:?}"
    );
    assert_eq!(runtime.observations().yields_to_a_foreground_fsync, 0);
}
