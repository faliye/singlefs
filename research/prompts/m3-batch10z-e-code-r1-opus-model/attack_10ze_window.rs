//! 第 10z 批第五块代码轮第一轮攻方模型：K1-4 (b) 第二段窄窗口有多宽。
//! 冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2198–2200 行：活在 `catch_unwind` 里 panic，返回之后才记
//! `device_when_the_work_returns`。活真 panic 的那一刻到记下的那一刻之间，是 panic 钩子（默认钩子打那一句，设了 RUST_BACKTRACE 还要取、解回溯）
//! 与展开（活里的局部量逐个丢）的时长；这段时间里同组别的段因致命的 Out of Memory 丢了设备，这段活就记成「交回那一刻设备丢了」。
//! 这里量的是 harness 档测试进程里同一种写法的时长（默认钩子不换），不是卡上的活：checker 档的二进制更大，解回溯更慢是推的。
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

#[inline(never)]
fn a_work_that_panics(depth: u32) -> u64 {
    if depth == 0 {
        panic!("真 panic：判一块时解出来的字不对");
    }
    let kept = vec![depth; 16];
    a_work_that_panics(depth - 1) + kept.len() as u64
}

#[test]
fn width_of_the_window_from_a_genuine_panic_to_the_record() {
    let mut widths_micros: Vec<f64> = Vec::new();
    for _ in 0..50 {
        let started = Instant::now();
        let outcome = catch_unwind(AssertUnwindSafe(|| a_work_that_panics(12)));
        // 此处即第 2200 行记 device_when_the_work_returns 的那一刻
        widths_micros.push(started.elapsed().as_secs_f64() * 1e6);
        assert!(outcome.is_err());
    }
    let first = widths_micros[0];
    let mut rest = widths_micros[1..].to_vec();
    rest.sort_by(f64::total_cmp);
    println!(
        "WINDOW rust_backtrace={:?} panics=50 first_micros={first:.0} rest_median_micros={:.0} rest_max_micros={:.0}",
        std::env::var("RUST_BACKTRACE").ok(),
        rest[rest.len() / 2],
        rest[rest.len() - 1]
    );
}

/// 第一段窄窗口（wgpu-core 30.0.1 `src/device/resource.rs` 第 5420 行置设备无效、第 5424 行才调丢失回调）的宽度由丢设备那条线程
/// 在这两行之间被调度走多久定：这里量此刻本机上一条线程两次相邻读钟之间被停的最长时长（6 条线程各量 1 秒），只是此刻的负载下的数。
#[test]
fn longest_gap_between_two_adjacent_statements_on_this_machine_now() {
    let gaps: Vec<(u128, usize)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..6)
            .map(|_| {
                s.spawn(|| {
                    let started = Instant::now();
                    let mut longest = 0u128;
                    let mut over_100_micros = 0usize;
                    while started.elapsed().as_secs() < 1 {
                        let a = Instant::now();
                        let b = Instant::now();
                        let gap = b.duration_since(a).as_nanos();
                        longest = longest.max(gap);
                        if gap > 100_000 {
                            over_100_micros += 1;
                        }
                    }
                    (longest, over_100_micros)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let longest = gaps.iter().map(|g| g.0).max().unwrap();
    let over: usize = gaps.iter().map(|g| g.1).sum();
    println!("GAP threads=6 seconds=1 longest_micros={} gaps_over_100_micros={over}", longest / 1000);
}
