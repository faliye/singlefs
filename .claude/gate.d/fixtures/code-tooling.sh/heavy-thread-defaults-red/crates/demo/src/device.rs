// 样本：装置只读 SAMPLE_RUST_THREADS，没设就 panic；回退值不是核数也不是写死的数，别的种类都不点名
const THREADS_ENVIRONMENT_VARIABLE: &str = "SAMPLE_RUST_THREADS";
pub fn threads() -> usize {
    std::env::var(THREADS_ENVIRONMENT_VARIABLE)
        .expect("没设")
        .parse()
        .expect("要是正整数")
}
