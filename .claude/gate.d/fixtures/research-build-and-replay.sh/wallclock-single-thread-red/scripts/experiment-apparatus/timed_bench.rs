// 样本：登记位（门禁判法只扫 scripts/experiment-apparatus/ 顶层的 .rs，不编这一份）；真正被 cargo test 跑的同名单测在 tiny/src/lib.rs
use std::time::{Duration, Instant};

fn busy_work() -> u64 { (0..1000u64).sum() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // wallclock-assert: 一千次加法在 1 毫秒内做完
    fn busy_work_finishes_within_a_millisecond() {
        let started = Instant::now();
        let _ = busy_work();
        assert!(started.elapsed() < Duration::from_millis(1), "1 毫秒内应当做完");
    }
}
