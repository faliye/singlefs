#[cfg(test)]
mod tests {
    #[test]
    fn only_relative_in_a_module() { let a = 1; let b = 1; assert_eq!(a, b); }
}
