//! 攻方腿探针（m4-gpugc-mount-runtime-code-r2，模型文件，不入产品）：盘真写满（每块 320 MiB、日志环 64 MiB，单元区 240 段；建池改在 `tools/make_the_small_pool_fixture.py`），`N₀` 取产品的 1，水位取产品默认。
//! T1：第一轮判决撤回「只用 accept 写的走不出来」靠的是满盘探针（第一拍发 10 次就过），那时没有前台 fsync 在等。破例排在让路之前只管「全空段低于底额」。这里放开扫：
//! 用户在满盘上做了什么（删一个、删四个、截短十六个）× 前台 fsync 在不在等（不等、一直有一个在等、真线程反复空 fsync）。只打 `name=` 读数，不断言结论。
//! 判别量：写得进（`accept_went_through`）的那一拍、模拟秒数、这段时间整理发了几次；fsync 走了之后几拍写得进。
//! 填盘每隔几份文件打一行按它估的填满要多久，估出来超过 40 分钟就 panic，不接着跑全量。
mod common_small_pool;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_small_pool::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount, PoolOnMemoryDevices,
    RecordedMemoryDevice, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::compaction_plan::{
    foreground_writes_are_below_the_empty_cluster_segments_floor, CallerMonotonicClockReading,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MonotonicClockOfThisProcess, MountedRuntime,
    RuntimeMounted, RuntimeParameters,
};

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
const DATA_UNITS_OF_A_FILE: usize = 32;
const FILES_AT_MOST: usize = 400;
const PASSES_OF_THE_SWEEP: u32 = 40;
const PASSES_AFTER_THE_FSYNC_LEFT: u32 = 12;
const FILES_TRUNCATED: usize = 16;
/// 单元区的整段数（320 MiB − 16 MiB − 64 MiB，一段 1 MiB）；一份文件占一段，估填满要多久用它当份数的上界。
const WHOLE_SEGMENTS_OF_THE_UNIT_AREA: f64 = 240.0;
/// 隔几份文件估一次。
const FILES_BETWEEN_THE_ESTIMATES: usize = 16;
/// 填盘估出来超过这么多秒就停（三方腿定义「造历史、写模型时这样取样」那一条的 40 分钟）。
const FILL_SECONDS_ALLOWED: f64 = 2400.0;
/// 一单元的小文件写到被拒，至多这么多份：余量不到一两份大文件，一份小文件至少占两槽。
const SMALL_FILES_AT_MOST: usize = 128;
/// 真线程那一臂量多久（挂钟）。
const MEASURING: Duration = Duration::from_secs(3);
const SHORT_PERIOD: Duration = Duration::from_millis(1);

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
    text.chars().take(200).collect()
}

/// 记录口径的全空段、开得成段没有、算不算低于底额。
fn segments_of(runtime: &MountedRuntime<RecordedMemoryDevice>) -> (u64, bool, bool) {
    runtime
        .with_the_session_and_devices(|locked| {
            let allocator = &locked.session.session().allocator;
            let device_map = &allocator.devices[0];
            (
                device_map.empty_segments(),
                device_map.lowest_empty_segment().is_some(),
                foreground_writes_are_below_the_empty_cluster_segments_floor(allocator),
            )
        })
        .expect("运行库没坏")
}

/// 填盘的进度：上一次估的时候过了多少秒，上一段每份文件多少秒。
struct ProgressOfTheFill {
    started: Instant,
    seconds_at_the_last_estimate: f64,
    seconds_per_file_of_the_last_stretch: Option<f64>,
}

impl ProgressOfTheFill {
    fn starting_now() -> Self {
        Self {
            started: Instant::now(),
            seconds_at_the_last_estimate: 0.0,
            seconds_per_file_of_the_last_stretch: None,
        }
    }

    /// 落点从单元区起点往后找，每份文件越来越慢：按最近两段的每份耗时线性外推，打出填满要多久；有了两段、估出来超过上限就停。
    fn estimate_after(&mut self, files_written: usize) {
        if files_written == 0 || !files_written.is_multiple_of(FILES_BETWEEN_THE_ESTIMATES) {
            return;
        }
        let elapsed = self.started.elapsed().as_secs_f64();
        let files_of_a_stretch = FILES_BETWEEN_THE_ESTIMATES as f64;
        let seconds_per_file = (elapsed - self.seconds_at_the_last_estimate) / files_of_a_stretch;
        let growth_per_file = self
            .seconds_per_file_of_the_last_stretch
            .map_or(0.0, |before| {
                ((seconds_per_file - before) / files_of_a_stretch).max(0.0)
            });
        let files_left = (WHOLE_SEGMENTS_OF_THE_UNIT_AREA - files_written as f64).max(0.0);
        let projected = elapsed
            + files_left * seconds_per_file
            + growth_per_file * files_left * files_left / 2.0;
        println!(
            "name=full_pool_fill_estimate files_written={files_written} elapsed_seconds={elapsed:.1} seconds_per_file={seconds_per_file:.3} projected_fill_seconds={projected:.0}"
        );
        assert!(
            self.seconds_per_file_of_the_last_stretch.is_none() || projected <= FILL_SECONDS_ALLOWED,
            "填盘估出来 {projected:.0} 秒，超过 {FILL_SECONDS_ALLOWED} 秒：先把盘缩小，不接着跑全量"
        );
        self.seconds_at_the_last_estimate = elapsed;
        self.seconds_per_file_of_the_last_stretch = Some(seconds_per_file);
    }
}

/// 这一次挂载里接着建文件、写 `content`，写到被拒为止：交回写成几份、被拒在哪（到了上限没被拒是空串）。
fn write_files_until_refused(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    content: &[u8],
    files_at_most: usize,
) -> (usize, String) {
    for written in 0..files_at_most {
        let inode = runtime
            .with_the_session_and_devices(|locked| {
                InodeNumber(
                    locked
                        .session
                        .session()
                        .current
                        .version_of_trees()
                        .expect("有树的一版")
                        .inode_number_watermark(),
                )
            })
            .expect("运行库没坏");
        if let Err(refused) = runtime.publish_user_change(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
        }) {
            return (written, first_characters(&format!("new_inode {refused:?}")));
        }
        if let Err(refused) = runtime.publish_user_change(UserChange::SequentialWriteOfAnInode {
            inode,
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
        }) {
            return (written, first_characters(&format!("write {refused:?}")));
        }
    }
    (files_at_most, String::new())
}

/// 用户在写满的池上做了什么（放开扫）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhatTheUserDid {
    UnlinkedOneFile,
    UnlinkedFourFiles,
    TruncatedFilesToHalf,
}

fn what_the_user_did(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    files: &[InodeNumber],
    action: WhatTheUserDid,
) -> String {
    let mut outcomes = Vec::new();
    match action {
        WhatTheUserDid::UnlinkedOneFile | WhatTheUserDid::UnlinkedFourFiles => {
            let files_unlinked = if action == WhatTheUserDid::UnlinkedOneFile {
                1
            } else {
                4
            };
            for inode in files.iter().take(files_unlinked) {
                let outcome = runtime
                    .publish_user_change(UserChange::Unlink {
                        inode: *inode,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    })
                    .map(|_published| ());
                outcomes.push(first_characters(&format!("{outcome:?}")));
            }
        }
        WhatTheUserDid::TruncatedFilesToHalf => {
            let new_size = u64::try_from(DATA_UNITS_OF_A_FILE / 2 * data_unit_payload_capacity())
                .expect("字节数");
            for inode in files.iter().take(FILES_TRUNCATED) {
                let outcome = runtime
                    .publish_user_change(UserChange::Truncate {
                        inode: *inode,
                        new_size,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    })
                    .map(|_published| ());
                outcomes.push(first_characters(&format!("{outcome:?}")));
            }
        }
    }
    outcomes.dedup();
    outcomes.join(";")
}

/// 不起线程，按线程体的节奏（一趟没完隔一个 P，否则隔一个周期）一拍一拍走。`is_a_fsync_waiting`：走这几拍时一直有一个前台 fsync 在等（`publish_pending_now` 进门就登记的那个计数，待发空也算）。
fn arm_passing_by_hand(
    base: &PoolOnMemoryDevices,
    files: &[InodeNumber],
    content: &[u8],
    action: WhatTheUserDid,
    is_a_fsync_waiting: bool,
) {
    let started_the_arm = Instant::now();
    let copy = base.copied_with_an_empty_stream();
    let clock = Arc::new(ClockMovedByTheProbe::default());
    let clock_of_the_runtime: Arc<ClockMovedByTheProbe> = Arc::clone(&clock);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        copy.devices,
        RuntimeParameters::by_default(),
        None,
        clock_of_the_runtime,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("可写挂载");
    let surviving = *files.last().expect("写成了文件");
    let small_content = b"a foreground write after the user action".to_vec();
    let overwrite = UserChange::OverwriteOfAnInode {
        inode: surviving,
        content: &small_content,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
    };
    let (files_of_the_top_up, _refusal_of_the_files) =
        write_files_until_refused(&runtime, content, FILES_AT_MOST);
    let (small_files_of_the_top_up, _refusal_of_the_small_files) =
        write_files_until_refused(&runtime, &small_content, SMALL_FILES_AT_MOST);
    let accepted_before_the_action = runtime.accept(overwrite).is_ok();
    let (empty_before, openable_before, below_before) = segments_of(&runtime);
    let outcome_of_the_action = what_the_user_did(&runtime, files, action);
    let (empty_after, openable_after, below_after) = segments_of(&runtime);
    println!(
        "name=full_pool_user_action action={action:?} fsync_waiting={is_a_fsync_waiting} files_of_the_top_up={files_of_the_top_up} small_files_of_the_top_up={small_files_of_the_top_up} accept_before_the_action_went_through={accepted_before_the_action} empty_before={empty_before} openable_before={openable_before} below_the_floor_before={below_before} outcome={outcome_of_the_action} empty_after={empty_after} openable_after={openable_after} below_the_floor_after={below_after}"
    );
    let waiting = is_a_fsync_waiting.then(|| runtime.a_foreground_fsync_is_waiting());
    let mut simulated = Duration::ZERO;
    let mut is_the_trip_under_way = false;
    let mut accept_went_through = false;
    for pass_number in 1..=PASSES_OF_THE_SWEEP {
        let elapsed = if is_the_trip_under_way {
            ONE_POLL
        } else {
            ONE_PERIOD
        };
        clock.advance_by(elapsed);
        simulated += elapsed;
        let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
        is_the_trip_under_way = pass.is_a_trip_under_way;
        let accepted = runtime.accept(overwrite);
        accept_went_through = accepted.is_ok();
        let (empty_by_the_records, openable, below_the_floor) = segments_of(&runtime);
        let observations = runtime.observations();
        println!(
            "name=full_pool_fsync_pass action={action:?} fsync_waiting={is_a_fsync_waiting} pass={pass_number} simulated_seconds={} publishes_of_the_pass={} total_publishes={} trip_under_way={is_the_trip_under_way} last_stop={:?} last_not_started={:?} empty_by_the_records={empty_by_the_records} openable={openable} below_the_floor={below_the_floor} accept_went_through={accept_went_through} accept={}",
            simulated.as_secs(),
            pass.compaction_publishes,
            observations.compaction_publishes,
            observations.last_stop,
            observations.last_not_started,
            first_characters(&format!("{accepted:?}")),
        );
        if accept_went_through {
            break;
        }
    }
    drop(waiting);
    if !accept_went_through {
        for pass_number in 1..=PASSES_AFTER_THE_FSYNC_LEFT {
            let elapsed = if is_the_trip_under_way {
                ONE_POLL
            } else {
                ONE_PERIOD
            };
            clock.advance_by(elapsed);
            let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
            is_the_trip_under_way = pass.is_a_trip_under_way;
            accept_went_through = runtime.accept(overwrite).is_ok();
            println!(
                "name=full_pool_after_the_fsync_left action={action:?} fsync_waiting={is_a_fsync_waiting} pass={pass_number} publishes_of_the_pass={} accept_went_through={accept_went_through}",
                pass.compaction_publishes,
            );
            if accept_went_through {
                break;
            }
        }
    }
    drop(runtime);
    println!(
        "name=full_pool_arm_seconds action={action:?} fsync_waiting={is_a_fsync_waiting} seconds={:.1}",
        started_the_arm.elapsed().as_secs_f64(),
    );
}

/// 真线程、真钟，周期压短（量的是前台 fsync 与线程的交错，不是周期）：`threads_calling_fsync` 条线程反复空 fsync，从挂载起就在跑。
fn arm_on_the_maintenance_thread(
    base: &PoolOnMemoryDevices,
    files: &[InodeNumber],
    content: &[u8],
    threads_calling_fsync: u32,
) {
    let copy = base.copied_with_an_empty_stream();
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.compaction_period = SHORT_PERIOD;
    runtime_parameters.publish_poll_period = SHORT_PERIOD;
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        copy.devices,
        runtime_parameters,
        None,
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("可写挂载");
    let surviving = *files.last().expect("写成了文件");
    let small_content = b"a foreground write after the user action".to_vec();
    let overwrite = UserChange::OverwriteOfAnInode {
        inode: surviving,
        content: &small_content,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
    };
    let stop = AtomicBool::new(false);
    let (
        accepted_before_the_action,
        outcome_of_the_action,
        publishes_measured,
        accepted_after_measuring,
    ) = std::thread::scope(|scope| {
        for _thread in 0..threads_calling_fsync {
            scope.spawn(|| {
                while !stop.load(Ordering::SeqCst) {
                    let _ignored = runtime.publish_pending_now().is_err();
                }
            });
        }
        let _files = write_files_until_refused(&runtime, content, FILES_AT_MOST);
        let _small_files = write_files_until_refused(&runtime, &small_content, SMALL_FILES_AT_MOST);
        let accepted_before_the_action = runtime.accept(overwrite).is_ok();
        let outcome_of_the_action =
            what_the_user_did(&runtime, files, WhatTheUserDid::UnlinkedOneFile);
        let publishes_before = runtime.observations().compaction_publishes;
        std::thread::sleep(MEASURING);
        let publishes_measured = runtime.observations().compaction_publishes - publishes_before;
        let accepted_after_measuring = runtime.accept(overwrite).is_ok();
        stop.store(true, Ordering::SeqCst);
        (
            accepted_before_the_action,
            outcome_of_the_action,
            publishes_measured,
            accepted_after_measuring,
        )
    });
    let given_up_at = Instant::now() + MEASURING;
    let mut accepted_after_the_fsyncs_left = accepted_after_measuring;
    let fsyncs_left_at = Instant::now();
    while !accepted_after_the_fsyncs_left && Instant::now() < given_up_at {
        std::thread::sleep(SHORT_PERIOD);
        accepted_after_the_fsyncs_left = runtime.accept(overwrite).is_ok();
    }
    let observations = runtime.observations();
    println!(
        "name=full_pool_on_the_thread threads_calling_fsync={threads_calling_fsync} accept_before_the_action_went_through={accepted_before_the_action} outcome={outcome_of_the_action} measured_seconds={} publishes_measured={publishes_measured} accept_after_measuring_went_through={accepted_after_measuring} accept_after_the_fsyncs_left_went_through={accepted_after_the_fsyncs_left} milliseconds_after_the_fsyncs_left={} maintenance_passes={} last_stop={:?} last_not_started={:?}",
        MEASURING.as_secs(),
        fsyncs_left_at.elapsed().as_millis(),
        observations.maintenance_passes,
        observations.last_stop,
        observations.last_not_started,
    );
    drop(runtime);
}

#[test]
fn a_writer_on_a_full_pool_while_a_foreground_fsync_keeps_waiting() {
    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let mounted = mount_writable(&parameters(), &mut pool.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content = vec![0x5A_u8; DATA_UNITS_OF_A_FILE * data_unit_payload_capacity()];
    let mut files = Vec::new();
    let mut refusal_of_the_fill = String::new();
    let mut progress = ProgressOfTheFill::starting_now();
    for _file in 0..FILES_AT_MOST {
        let inode = InodeNumber(
            session
                .current
                .version_of_trees()
                .expect("有树的一版")
                .inode_number_watermark(),
        );
        if let Err(refused) = session.publish_user_change(
            &mut pool.devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        ) {
            refusal_of_the_fill = first_characters(&format!("new_inode {refused:?}"));
            break;
        }
        match session.publish_user_change(
            &mut pool.devices,
            UserChange::SequentialWriteOfAnInode {
                inode,
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        ) {
            Ok(_published) => files.push(inode),
            Err(refused) => {
                refusal_of_the_fill = first_characters(&format!("write {refused:?}"));
                break;
            }
        }
        pool.stream.lock().expect("录制流的锁里不 panic").clear();
        progress.estimate_after(files.len());
    }
    drop(session);
    println!(
        "name=full_pool_fill files_written={} refusal={refusal_of_the_fill} elapsed_seconds={:.1}",
        files.len(),
        progress.started.elapsed().as_secs_f64(),
    );
    for action in [
        WhatTheUserDid::UnlinkedOneFile,
        WhatTheUserDid::UnlinkedFourFiles,
        WhatTheUserDid::TruncatedFilesToHalf,
    ] {
        for is_a_fsync_waiting in [false, true] {
            arm_passing_by_hand(&pool, &files, &content, action, is_a_fsync_waiting);
        }
    }
    for threads_calling_fsync in [0_u32, 1, 2] {
        arm_on_the_maintenance_thread(&pool, &files, &content, threads_calling_fsync);
    }
}
