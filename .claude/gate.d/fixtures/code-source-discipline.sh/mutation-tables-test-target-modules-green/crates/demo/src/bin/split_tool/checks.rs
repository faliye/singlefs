pub fn tool_value() -> u64 { 3 }

#[cfg(test)]
mod tests {
    #[test]
    fn tool_case() {
        assert_eq!(super::tool_value(), 3);
        assert_eq!(super::tool_value() + 1, 4);
    }
}
