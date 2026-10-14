fn count_from_environment(name: &str, default: u64) -> u64 { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default) }
pub fn sampling() -> u64 { count_from_environment("SINGLEFS_RANDOM_HISTORY_SAMPLING_WORKERS", 8) }
