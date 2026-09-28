// 样本：拆成多行的断言，比的是字符串不是数字——不许被误判成钉了绝对值。
fn main() {}
#[cfg(test)]
mod tests {
    #[test]
    fn only_relative_multiline() {
        assert_eq!(
            format!("{}", 1 + 1),
            "two"
        );
    }
}
