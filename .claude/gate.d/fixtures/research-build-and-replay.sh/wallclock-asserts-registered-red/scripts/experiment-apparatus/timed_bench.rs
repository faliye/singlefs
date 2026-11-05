// 样本：一个按挂钟判的断言（.elapsed() 比大小），没有登记位，门禁必须判红。
use std::time::{Duration, Instant};

fn busy_work() -> u64 { (0..1000u64).sum() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_work_finishes_within_a_millisecond() {
        let started = Instant::now();
        let _ = busy_work();
        assert!(started.elapsed() < Duration::from_millis(1), "1 毫秒内应当做完");
    }
}
