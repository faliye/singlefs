pub fn published_generation(transaction_generation: u64) -> u64 {
    transaction_generation
}

#[cfg(test)]
mod tests {
    #[test]
    fn lib_case() {
        assert_eq!(super::published_generation(1), 1);
    }
}
