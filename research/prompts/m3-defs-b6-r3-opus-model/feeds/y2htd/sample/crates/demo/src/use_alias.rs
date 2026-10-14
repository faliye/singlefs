use std::env::var as read_variable;

pub fn threads() -> usize {
    read_variable("SAMPLE_USE_ALIAS_THREADS").expect("set it").parse().unwrap()
}
