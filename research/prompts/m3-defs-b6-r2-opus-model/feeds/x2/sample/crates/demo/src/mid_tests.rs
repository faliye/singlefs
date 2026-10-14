#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {}
}

pub fn workers() -> usize {
    std::env::var("SAMPLE_AFTER_TESTS_WORKERS").expect("set it").parse().expect("number")
}
