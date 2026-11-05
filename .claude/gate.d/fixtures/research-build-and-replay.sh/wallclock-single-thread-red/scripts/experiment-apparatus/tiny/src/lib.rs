// research-build-and-replay.sh 的 research-unit-tests 格样本：登记成挂钟断言的那条单测读样本根下的开关文件 .red-switch（登记位在 scripts/experiment-apparatus/timed_bench.rs）。
// 第一趟按名跳过它，所以第一趟全绿；单线程那一趟跑到它，开关文件在，它红，这一格判红。门禁的弄坏开关 RESEARCH_BUILD_AND_REPLAY_BREAK=wallclock-single-run-dropped
// 让那一趟不起，这一格就判绿了
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
        // cargo test 的当前目录是包根（样本根/scripts/experiment-apparatus/tiny），往上三级是样本根
        assert!(!std::path::Path::new("../../../.red-switch").exists(), "样本根的开关文件 .red-switch 在");
        assert!(started.elapsed() < std::time::Duration::from_millis(1));
    }
}
