// 照 crates/singlefs-harness/tests/random_histories.rs 的 count_from_environment 写法：读环境变量的包装函数，调用处只给名字
fn count_from_environment(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

pub fn workers() -> u64 {
    count_from_environment("SAMPLE_COUNT_WORKERS", 8)
}
