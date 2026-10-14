pub fn pool() -> usize { std::thread::available_parallelism().map_or(1, |count| count.get()) }
pub fn seeds() -> u64 { count_from_environment("SAMPLE_ABSORBED_SEEDS", 1000) }
pub fn sampling() -> u64 { count_from_environment("SAMPLE_ABSORBED_SAMPLING_WORKERS", 8) }
