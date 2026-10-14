#[cfg(test)]
use std::collections::HashMap;

pub fn worker_threads() -> usize {
    std::env::var("SAMPLE_WORKER_THREADS").expect("SAMPLE_WORKER_THREADS 要设").parse().expect("正整数")
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    #[test]
    fn reads_test_only_threads() {
        let _ = std::env::var("SAMPLE_TEST_ONLY_THREADS");
    }
}
