pub fn workers() -> usize {
    std::env::var("SAMPLE_BEFORE_TESTS_WORKERS").expect("set it").parse().expect("number")
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {}
}
