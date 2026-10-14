pub fn threads() -> usize {
    std::env::vars().find(|(key, _)| key == "SAMPLE_VARS_ITER_THREADS").map(|(_, value)| value.parse().unwrap()).expect("set it")
}
