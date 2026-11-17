//! 攻方腿探针（m4-gpugc-mount-runtime-code-r1，模型文件，不入产品）：盘真写满（每块 1 GiB，单元区约 200 段），`N₀` 取产品的 1，一个只供测试的开关都不碰（水位也是产品默认）。
//! T3：`accept` 那一判被空间拒时不推抬 F（`transaction.rs` 那一判的文档），`publish_user_change` 才推。删掉几个文件之后全空段按记录口径够了（不破例），放掉的槽在 defer 里（开不成段）：
//! 只用 `accept` 写的调用方靠维护线程走不走得出来。用户动作放开扫：删几个文件。只打 `name=` 读数，不断言结论。
mod common_small_pool;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

#[test]
fn a_writer_using_only_accept_on_a_full_pool() {
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
        "name=full_pool_fill files_written={} refusal={refusal_of_the_fill}",
        files.len()
    );
    for files_unlinked in [1_usize, 4, 16] {
        let copy = pool.copied_with_an_empty_stream();
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
        for inode in files.iter().take(files_unlinked) {
            let unlinked = runtime
                .publish_user_change(UserChange::Unlink {
                    inode: *inode,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                })
                .is_ok();
            println!("name=full_pool_unlink files_unlinked={files_unlinked} inode={inode:?} went_through={unlinked} faults_reads={}", copy.faults.reads.load(Ordering::SeqCst));
        }
        let surviving = *files.last().expect("写成了文件");
        let small_content = b"a foreground write after the unlink".to_vec();
        let overwrite = UserChange::OverwriteOfAnInode {
            inode: surviving,
            content: &small_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
        };
        let mut simulated = Duration::ZERO;
        let mut is_the_trip_under_way = false;
        let mut accept_went_through = false;
        for pass_number in 1..=PASSES_OF_THE_SWEEP {
            let elapsed = if is_the_trip_under_way { ONE_POLL } else { ONE_PERIOD };
            clock.advance_by(elapsed);
            simulated += elapsed;
            let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
            is_the_trip_under_way = pass.is_a_trip_under_way;
            let accepted = runtime.accept(overwrite);
            accept_went_through = accepted.is_ok();
            let (empty_by_the_records, openable) = segments_of(&runtime);
            let observations = runtime.observations();
            println!(
                "name=full_pool_pass files_unlinked={files_unlinked} pass={pass_number} simulated_seconds={} total_publishes={} last_stop={:?} last_not_started={:?} empty_by_the_records={empty_by_the_records} openable={openable} accept_went_through={accept_went_through} accept={}",
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
                "name=full_pool_publish_user_change files_unlinked={files_unlinked} went_through={} outcome={}",
                published.is_ok(),
                first_characters(&format!("{published:?}")),
            );
        }
        drop(runtime);
    }
}
