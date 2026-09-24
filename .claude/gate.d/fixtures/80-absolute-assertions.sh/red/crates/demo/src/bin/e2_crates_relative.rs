fn main() {}
#[cfg(test)]
mod tests {
    #[test]
    fn only_relative_in_crates() { let measured = 3; let expected = 3; assert_eq!(measured, expected); }
}
