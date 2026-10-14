// 样本：available_parallelism 只在注释里，不点名
pub fn threads() -> usize {
    std::thread::available_parallelism().map_or(1, |count| count.get())
}

pub fn demo_threads() -> usize {
    std::env::var("DEMO_THREADS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8)
}
