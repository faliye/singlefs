pub fn standalone_value(input: u64) -> u64 {
    input
}

#[cfg(test)]
mod tests {
    #[test]
    fn standalone_lib_case() {
        assert_eq!(super::standalone_value(2), 2);
    }
}
