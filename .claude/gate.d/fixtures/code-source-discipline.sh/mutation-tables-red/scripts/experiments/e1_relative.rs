fn main() {}
#[cfg(test)]
mod tests {
    #[test]
    fn only_relative() { let a = 1; let b = 1; assert_eq!(a, b); }
}
