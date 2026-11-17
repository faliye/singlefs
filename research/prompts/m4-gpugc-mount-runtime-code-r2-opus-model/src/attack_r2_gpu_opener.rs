//! 攻方腿探针（m4-gpugc-mount-runtime-code-r2，模型文件，不入产品）：拷进冻结副本 `crates/singlefs-background-gpu/tests/` 跑，不要卡、不碰 Vulkan。
//! 只打 `name=` 开头的读数，不断言结论。
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use singlefs_background_gpu::gpu_opener_of_this_process::{
    GpuOpenerOfThisProcess, VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST,
};
use singlefs_core::background_compute::BackgroundComputeSettings;
use singlefs_mount_runtime::OpensTheBackgroundGpu;

const REFUSAL_OF_THE_DOUBLE: &str = "替身开卡一律拒";

/// T1：不持锁建实例之后，几条线程反复开卡、丢实例，期限短到一直撞超时：建实例有没有并发、建了几个、过没过上限。
#[test]
fn makers_in_flight_while_several_threads_open_and_forget() {
    let in_flight = Arc::new(AtomicU32::new(0));
    let most_in_flight = Arc::new(AtomicU32::new(0));
    let makers_called = Arc::new(AtomicU32::new(0));
    let in_flight_of_the_maker = Arc::clone(&in_flight);
    let most_of_the_maker = Arc::clone(&most_in_flight);
    let called_of_the_maker = Arc::clone(&makers_called);
    let opener: Arc<GpuOpenerOfThisProcess<u32>> = Arc::new(
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            Duration::from_millis(1),
            Arc::new(move || {
                let now_in_flight = in_flight_of_the_maker.fetch_add(1, Ordering::SeqCst) + 1;
                most_of_the_maker.fetch_max(now_in_flight, Ordering::SeqCst);
                called_of_the_maker.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(3));
                in_flight_of_the_maker.fetch_sub(1, Ordering::SeqCst);
                0
            }),
            Arc::new(
                |_instance, _selection, _memory_leaked| Err(REFUSAL_OF_THE_DOUBLE.to_string()),
            ),
        ),
    );
    let mut threads = Vec::new();
    for thread_number in 0..8_u32 {
        let opener_of_the_thread = Arc::clone(&opener);
        threads.push(std::thread::spawn(move || {
            let settings = BackgroundComputeSettings::PRODUCT;
            for round in 0..50_u32 {
                let _outcome = opener_of_the_thread
                    .open_the_background_gpu(&settings)
                    .err();
                if (round + thread_number) % 3 == 0 {
                    opener_of_the_thread.forget_the_instance_so_the_next_opening_creates_one();
                }
            }
        }));
    }
    for thread in threads {
        thread.join().expect("探针的线程不 panic");
    }
    std::thread::sleep(Duration::from_millis(100));
    println!(
        "name=opener_makers_in_flight most_in_flight={} makers_called={} instances_created={} limit={VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST} hung_at_the_end={}",
        most_in_flight.load(Ordering::SeqCst),
        makers_called.load(Ordering::SeqCst),
        opener.instances_created(),
        opener.is_an_opening_hung(),
    );
}

/// T1：建实例 panic 几次：额度记在建之前，panic 也用掉一个，闸放开之后的二次开卡报什么。
#[test]
fn a_maker_panicking_uses_up_the_limit() {
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            Duration::from_secs(5),
            Arc::new(|| -> u32 { panic!("探针：建实例 panic") }),
            Arc::new(
                |_instance, _selection, _memory_leaked| Err(REFUSAL_OF_THE_DOUBLE.to_string()),
            ),
        );
    let settings = BackgroundComputeSettings::PRODUCT;
    for opening in 1..=(VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST + 1) {
        let outcome = opener.open_the_background_gpu(&settings).err();
        println!(
            "name=opener_maker_panics opening={opening} outcome={outcome:?} instances_created={} hung={}",
            opener.instances_created(),
            opener.is_an_opening_hung(),
        );
    }
}

/// T3：`OpensTheBackgroundGpu` 的文档写「这一次调用要在实现自己的期限之内返回」。几次挂载同时开卡，每次都不超期（开卡 150 毫秒，期限 200 毫秒）：后到的排在 `one_opening_at_a_time` 上，从调用起算等多久。
#[test]
fn several_openings_at_once_each_within_the_deadline() {
    let opener: Arc<GpuOpenerOfThisProcess<u32>> = Arc::new(
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            Duration::from_millis(200),
            Arc::new(|| 0),
            Arc::new(|_instance, _selection, _memory_leaked| {
                std::thread::sleep(Duration::from_millis(150));
                Err(REFUSAL_OF_THE_DOUBLE.to_string())
            }),
        ),
    );
    let mut threads = Vec::new();
    for _thread in 0..4_u32 {
        let opener_of_the_thread = Arc::clone(&opener);
        threads.push(std::thread::spawn(move || {
            let settings = BackgroundComputeSettings::PRODUCT;
            let called_at = Instant::now();
            let outcome = opener_of_the_thread
                .open_the_background_gpu(&settings)
                .err();
            (called_at.elapsed(), format!("{outcome:?}"))
        }));
    }
    let mut waits = Vec::new();
    for thread in threads {
        let (waited, outcome) = thread.join().expect("探针的线程不 panic");
        waits.push(waited.as_millis());
        println!(
            "name=opener_several_at_once waited_milliseconds={} outcome={outcome}",
            waited.as_millis()
        );
    }
    waits.sort_unstable();
    println!(
        "name=opener_several_at_once_summary deadline_milliseconds=200 sorted_waits={waits:?}"
    );
}
