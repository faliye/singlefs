// 样本：装置只读 E161_THREADS，capped.sh 的清单里有它；测试模块里点名的变量不算
const THREADS_ENVIRONMENT_VARIABLE: &str = "E161_THREADS";
pub fn threads() -> usize {
    std::env::var(THREADS_ENVIRONMENT_VARIABLE)
        .expect("没设")
        .parse()
        .expect("要是正整数")
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_other_threads() {
        let _ = std::env::var("SAMPLE_TEST_THREADS");
    }
}
