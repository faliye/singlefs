//! 攻方腿探针（m4-gpugc-mount-runtime-code-r1，模型文件，不入产品）：盘真写满（每块 320 MiB、日志环 64 MiB，单元区 240 段；建池改在 `tools/make_the_small_pool_fixture.py`），`N₀` 取产品的 1，一个只供测试的开关都不碰（水位也是产品默认）。
//! T3：`accept` 那一判被空间拒时不推抬 F（`transaction.rs` 那一判的文档），`publish_user_change` 才推。删掉几个文件之后全空段按记录口径够了（不破例），放掉的槽在 defer 里（开不成段）：
//! 只用 `accept` 写的调用方靠维护线程走不走得出来。用户动作放开扫：删几个文件；重挂之后先把这一次挂载也写到被拒（`topped_up=true`）还是不写（`topped_up=false`）。只打 `name=` 读数，不断言结论。
//! 重挂之后别的挂载开过的段不再挡数据单元，1 MiB 的文件被拒时也还可能剩不到一份文件的余量：`topped_up=true` 那几组才是「这一次挂载里写满」，`accept_before_unlink_went_through` 判它写满没写满。
//! 填盘每隔几份文件打一行按它估的填满要多久，估出来超过 40 分钟就 panic，不接着跑全量。
mod common_small_pool;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common_small_pool::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount, RecordedMemoryDevice,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::compaction_plan::CallerMonotonicClockReading;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MountedRuntime, RuntimeMounted,
    RuntimeParameters,
};

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
const DATA_UNITS_OF_A_FILE: usize = 32;
const FILES_AT_MOST: usize = 400;
const PASSES_OF_THE_SWEEP: u32 = 40;
/// 单元区的整段数（320 MiB − 16 MiB − 64 MiB，一段 1 MiB）；一份文件占一段，估填满要多久用它当份数的上界。
const WHOLE_SEGMENTS_OF_THE_UNIT_AREA: f64 = 240.0;
/// 隔几份文件估一次。
const FILES_BETWEEN_THE_ESTIMATES: usize = 16;
/// 填盘估出来超过这么多秒就停（三方腿定义「造历史、写模型时这样取样」那一条的 40 分钟）。
const FILL_SECONDS_ALLOWED: f64 = 2400.0;
/// 一单元的小文件写到被拒，至多这么多份：余量不到一两份大文件，一份小文件至少占两槽。
const SMALL_FILES_AT_MOST: usize = 128;

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

fn segments_of(runtime: &MountedRuntime<RecordedMemoryDevice>) -> (u64, bool) {
    runtime
        .with_the_session_and_devices(|locked| {
            let device_map = &locked.session.session().allocator.devices[0];
            (
                device_map.empty_segments(),
                device_map.lowest_empty_segment().is_some(),
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

#[test]
fn a_writer_using_only_accept_on_a_full_pool() {
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
    for topped_up in [false, true] {
        for files_unlinked in [1_usize, 4, 16] {
            let started_the_arm = Instant::now();
            let copy = pool.copied_with_an_empty_stream();
            let clock = Arc::new(ClockMovedByTheProbe::default());
            let clock_of_the_runtime: Arc<ClockMovedByTheProbe> = Arc::clone(&clock);
            let RuntimeMounted { runtime, .. } =
                MountedRuntime::mount_without_the_maintenance_thread(
                    &parameters(),
                    copy.devices,
                    RuntimeParameters::by_default(),
                    None,
                    clock_of_the_runtime,
                    Arc::new(IgnoringTheMaintenanceEvents),
                )
                .expect("可写挂载");
            let surviving = *files.last().expect("写成了文件");
            let small_content = b"a foreground write after the unlink".to_vec();
            let overwrite = UserChange::OverwriteOfAnInode {
                inode: surviving,
                content: &small_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            };
            if topped_up {
                let (files_of_the_top_up, refusal_of_the_files) =
                    write_files_until_refused(&runtime, &content, FILES_AT_MOST);
                let (small_files_of_the_top_up, refusal_of_the_small_files) =
                    write_files_until_refused(&runtime, &small_content, SMALL_FILES_AT_MOST);
                let accepted_before_the_unlink = runtime.accept(overwrite);
                let (empty_by_the_records, openable) = segments_of(&runtime);
                println!(
                    "name=full_pool_top_up files_unlinked={files_unlinked} files_written={files_of_the_top_up} refusal_of_the_files={refusal_of_the_files} small_files_written={small_files_of_the_top_up} refusal_of_the_small_files={refusal_of_the_small_files} empty_by_the_records={empty_by_the_records} openable={openable} accept_before_unlink_went_through={} accept={}",
                    accepted_before_the_unlink.is_ok(),
                    first_characters(&format!("{accepted_before_the_unlink:?}")),
                );
            }
            for inode in files.iter().take(files_unlinked) {
                let unlinked = runtime
                    .publish_user_change(UserChange::Unlink {
                        inode: *inode,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    })
                    .is_ok();
                println!("name=full_pool_unlink topped_up={topped_up} files_unlinked={files_unlinked} inode={inode:?} went_through={unlinked} faults_reads={}", copy.faults.reads.load(Ordering::SeqCst));
            }
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
                let (empty_by_the_records, openable) = segments_of(&runtime);
                let observations = runtime.observations();
                println!(
                    "name=full_pool_pass topped_up={topped_up} files_unlinked={files_unlinked} pass={pass_number} simulated_seconds={} total_publishes={} last_stop={:?} last_not_started={:?} empty_by_the_records={empty_by_the_records} openable={openable} accept_went_through={accept_went_through} accept={}",
                    simulated.as_secs(),
                    observations.compaction_publishes,
                    observations.last_stop,
                    observations.last_not_started,
                    first_characters(&format!("{accepted:?}")),
                );
                if accept_went_through {
                    break;
                }
            }
            if !accept_went_through {
                let published = runtime.publish_user_change(overwrite);
                println!(
                    "name=full_pool_publish_user_change topped_up={topped_up} files_unlinked={files_unlinked} went_through={} outcome={}",
                    published.is_ok(),
                    first_characters(&format!("{published:?}")),
                );
            }
            drop(runtime);
            println!(
                "name=full_pool_arm_seconds topped_up={topped_up} files_unlinked={files_unlinked} seconds={:.1}",
                started_the_arm.elapsed().as_secs_f64(),
            );
        }
    }
}
