//! 辩方腿探针（m4-gpugc-mount-runtime-code-r2，模型文件，不入产品）：复核第一轮判决撤回「只用 accept 写的走不出来」够不够得着。
//! 判决依据的读数（`m4-gpugc-mount-runtime-code-r1-main-check/s2-run/readings.out`）里 accept 只在每一拍整理之后试、只试 35 字节的一单元覆盖写、只看 `accept` 是不是 `Ok`（`KeptPending`），
//! 没试过整理之前，没试过按填盘时的 32 单元大小写，也没看过收下的待发发不发得成。这里每个 (删几个文件, 整理几拍) 各起一份新挂载，大小两种各试一次：
//! 收下之后立刻 `publish_pending_now`，看发布成不成。池、填法与攻方腿第二次抽样相同（320 MiB 盘、240 段、`N₀` 取产品的 1、不碰任何只供测试的开关）。只打 `name=` 读数，不断言结论。
mod common_small_pool;

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
/// 整理推几拍之后试前台的写：0 是删完文件马上试。
const PASSES_BEFORE_THE_WRITE: [u32; 4] = [0, 1, 3, 8];

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
    text.chars().take(220).collect()
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

#[test]
fn what_the_foreground_gets_after_unlinking_files_on_a_full_pool() {
    let started = Instant::now();
    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let mounted = mount_writable(&parameters(), &mut pool.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content = vec![0x5A_u8; DATA_UNITS_OF_A_FILE * data_unit_payload_capacity()];
    let mut files = Vec::new();
    let mut refusal_of_the_fill = String::new();
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
    }
    drop(session);
    println!(
        "name=defense_fill files_written={} refusal={refusal_of_the_fill} elapsed_seconds={:.1}",
        files.len(),
        started.elapsed().as_secs_f64(),
    );
    let surviving = *files.last().expect("写成了文件");
    let small_content = b"a foreground write after the unlink".to_vec();
    for files_unlinked in [1_usize, 4] {
        for passes_before in PASSES_BEFORE_THE_WRITE {
            for is_big in [false, true] {
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
                for inode in files.iter().take(files_unlinked) {
                    runtime
                        .publish_user_change(UserChange::Unlink {
                            inode: *inode,
                            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                        })
                        .expect("删文件");
                }
                let mut is_the_trip_under_way = false;
                for _pass in 0..passes_before {
                    clock.advance_by(if is_the_trip_under_way {
                        ONE_POLL
                    } else {
                        ONE_PERIOD
                    });
                    is_the_trip_under_way = runtime
                        .run_one_maintenance_pass()
                        .expect("运行库没坏")
                        .is_a_trip_under_way;
                }
                let change = if is_big {
                    UserChange::SequentialWriteOfAnInode {
                        inode: surviving,
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                    }
                } else {
                    UserChange::OverwriteOfAnInode {
                        inode: surviving,
                        content: &small_content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                    }
                };
                let accepted = runtime.accept(change);
                let accept_went_through = accepted.is_ok();
                let pending_published = if accept_went_through {
                    match runtime.publish_pending_now() {
                        Ok(published) => format!("ok published_something={}", published.is_some()),
                        Err(refused) => first_characters(&format!("err {refused:?}")),
                    }
                } else {
                    "not_tried".to_string()
                };
                let (empty_by_the_records, openable) = segments_of(&runtime);
                let observations = runtime.observations();
                println!(
                    "name=defense_arm files_unlinked={files_unlinked} passes_before={passes_before} size={} accept_went_through={accept_went_through} accept={} pending_published=[{pending_published}] empty_by_the_records={empty_by_the_records} openable={openable} total_compaction_publishes={} last_stop={:?}",
                    if is_big { "thirty_two_units" } else { "one_unit" },
                    first_characters(&format!("{accepted:?}")),
                    observations.compaction_publishes,
                    observations.last_stop,
                );
                drop(runtime);
            }
        }
    }
}
