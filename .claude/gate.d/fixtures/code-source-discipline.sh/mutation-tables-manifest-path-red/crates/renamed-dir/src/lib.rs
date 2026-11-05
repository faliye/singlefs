pub fn renamed_member_value(input: u64) -> u64 {
    input
}

#[cfg(test)]
mod tests {
    #[test]
    fn renamed_member_case() {
        assert_eq!(super::renamed_member_value(7), 7);
    }
}
