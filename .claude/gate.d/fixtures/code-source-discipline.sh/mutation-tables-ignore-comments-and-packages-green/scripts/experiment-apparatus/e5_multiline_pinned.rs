// 样本：rustfmt 把绝对值断言拆成多行之后，absolute-assertions 那一格仍要认得出它（2026-09-25 补）。
fn main() {}
#[cfg(test)]
mod tests {
    #[test]
    fn pinned_multiline() {
        assert_eq!(
            2 + 2,
            4
        );
    }
}
