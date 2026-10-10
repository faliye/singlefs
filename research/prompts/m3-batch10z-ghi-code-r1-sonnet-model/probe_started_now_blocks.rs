//! 探针（正推腿自己的，不入库）：`DeviceStartedOnDemand::started_now` 与 `started_or_start` 的锁关系。
//! 一个线程在起（重起）设备的 300 毫秒里，另一个线程问 `started_now`（卡池借之前 poll 要先取留着的设备）要等多久。
use singlefs_harness::gpu_memory_ledger::DeviceStartedOnDemand;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn probe_started_now_waits_for_a_start_in_progress() {
    let slot: Arc<DeviceStartedOnDemand<u32>> = Arc::new(DeviceStartedOnDemand::new());
    // 先放一个「丢了的」设备 1，再让重起花 300 毫秒
    let first = slot.started_or_start(|_| false, || Ok::<u32, ()>(1)).unwrap();
    assert_eq!(first, 1);
    let restarting = {
        let slot = Arc::clone(&slot);
        std::thread::spawn(move || {
            slot.started_or_start(
                |device| *device == 1, // 1 号丢了
                || {
                    std::thread::sleep(Duration::from_millis(300));
                    Ok::<u32, ()>(2)
                },
            )
            .unwrap()
        })
    };
    std::thread::sleep(Duration::from_millis(50)); // 让重起先拿到锁
    let asked = Instant::now();
    let seen = slot.started_now();
    let waited = asked.elapsed();
    eprintln!("PROBE-LOCK started_now returned {seen:?} after {} ms (restart takes 300 ms)", waited.as_millis());
    assert_eq!(restarting.join().unwrap(), 2);
    assert!(waited >= Duration::from_millis(200), "started_now 没有等重起：{waited:?}");
}
