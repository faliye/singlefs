//! m3-batch10z-jk-code-r1 攻方 s2：第十一块 K4-1/K4-3 的并发压测（只在草稿副本里跑，不进仓）。
//! 攻 DeviceStartedOnDemand 两把锁：死锁、同一张卡两条线程各起一台、丢旧设备时压不压住别的线程、started_now 重起里不等。
use singlefs_harness::gpu_memory_ledger::DeviceStartedOnDemand;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

/// 丢的时候能阻塞一会儿（模拟 wgpu 等设备空）的设备。
#[derive(Clone)]
struct BlockingOnDrop {
    serial: u64,
    lost: bool,
    drop_blocks_ms: Arc<AtomicU64>,
    drops_seen: Arc<AtomicUsize>,
}
impl Drop for BlockingOnDrop {
    fn drop(&mut self) {
        let ms = self.drop_blocks_ms.load(Ordering::SeqCst);
        if ms > 0 {
            std::thread::sleep(Duration::from_millis(ms));
        }
        self.drops_seen.fetch_add(1, Ordering::SeqCst);
    }
}

fn env_usize(k: &str, d: usize) -> usize {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

#[test]
fn stress_two_locks() {
    let threads = env_usize("S2_THREADS", 8);
    let rounds = env_usize("S2_ROUNDS", 2000);
    let drop_blocks_ms = env_usize("S2_DROP_MS", 0) as u64;
    let slot: DeviceStartedOnDemand<BlockingOnDrop> = DeviceStartedOnDemand::new();
    let serial = AtomicU64::new(0);
    let drops_seen = Arc::new(AtomicUsize::new(0));
    let starts = AtomicUsize::new(0);
    let started_now_blocked_max_ms = AtomicU64::new(0);
    let barrier = Barrier::new(threads);
    // 每一轮：有的线程 started_or_start（偶尔把留着的判成丢了触发重起），有的 started_now / is_started。
    std::thread::scope(|scope| {
        for t in 0..threads {
            let slot = &slot;
            let serial = &serial;
            let starts = &starts;
            let drops_seen = &drops_seen;
            let drop_blocks_ms = drop_blocks_ms;
            let started_now_blocked_max_ms = &started_now_blocked_max_ms;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                for r in 0..rounds {
                    // 约四分之一的轮把留着的判成丢了，逼出一次重起（换下来的旧设备在这一轮丢）。
                    let treat_lost = (r + t) % 4 == 0;
                    if (r + t) % 3 == 0 {
                        let before = Instant::now();
                        let taken = slot.started_now();
                        let waited = before.elapsed().as_millis() as u64;
                        started_now_blocked_max_ms.fetch_max(waited, Ordering::SeqCst);
                        let _ = slot.is_started();
                        drop(taken); // 交回那台设备的 Drop（可能阻塞）不算进 started_now 的计时
                    } else {
                        let out = slot.started_or_start(
                            move |device: &BlockingOnDrop| treat_lost || device.lost,
                            || {
                                starts.fetch_add(1, Ordering::SeqCst);
                                Ok::<BlockingOnDrop, String>(BlockingOnDrop {
                                    serial: serial.fetch_add(1, Ordering::SeqCst),
                                    lost: false,
                                    drop_blocks_ms: Arc::new(AtomicU64::new(drop_blocks_ms)),
                                    drops_seen: drops_seen.clone(),
                                })
                            },
                        );
                        assert!(out.is_ok(), "起设备这一臂恒成功");
                    }
                }
            });
        }
    });
    // 跑完了（没死锁），交回留着的那一台存在。
    assert!(slot.is_started(), "跑完留着一台");
    println!(
        "S2LOCK threads={threads} rounds={rounds} drop_ms={drop_blocks_ms} starts={} drops_seen={} started_now_blocked_max_ms={}",
        starts.load(Ordering::SeqCst),
        drops_seen.load(Ordering::SeqCst),
        started_now_blocked_max_ms.load(Ordering::SeqCst)
    );
}
