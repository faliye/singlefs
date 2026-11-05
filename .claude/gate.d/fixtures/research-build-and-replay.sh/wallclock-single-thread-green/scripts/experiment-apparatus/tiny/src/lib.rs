// research-build-and-replay.sh 的 research-unit-tests 格样本：一条普通单测、一条登记成挂钟断言的单测（登记位在 scripts/experiment-apparatus/timed_bench.rs，
// 判法只扫那一层顶层的 .rs，同名的这一条是真正被 cargo test 跑的）。第一趟按名跳过它，随后单线程那一趟跑到它、判绿
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }

    #[test]
    fn busy_work_finishes_within_a_millisecond() {
        let started = std::time::Instant::now();
        assert_eq!(super::add(2, 2), 4);
        assert!(started.elapsed() < std::time::Duration::from_millis(1));
    }
}
