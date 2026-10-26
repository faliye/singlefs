// 样本：一个带登记位（一行注释）的挂钟断言、一个 load-safe 的挂钟断言，门禁判绿。
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

    #[test]
    // wallclock-assert-load-safe: 只比睡了至少 5 毫秒（下界），负载再高只会更久
    fn sleep_lasts_at_least_five_milliseconds() {
        let started = Instant::now();
        std::thread::sleep(Duration::from_millis(5));
        assert!(started.elapsed() >= Duration::from_millis(5));
    }

    #[test]
    fn busy_work_sums() {
        assert_eq!(busy_work(), 499_500);
    }
}
