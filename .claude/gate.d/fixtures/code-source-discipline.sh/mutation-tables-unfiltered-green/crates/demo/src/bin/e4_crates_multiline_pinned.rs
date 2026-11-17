// 样本：照 e142_new_pool_file_creation_write_dump.rs 被 rustfmt 拆行后的样子——
// `assert!(` 起、比较式一行、消息一行、`);` 收尾，住在 crates/ 下。
fn main() {}
#[cfg(test)]
mod tests {
    #[test]
    fn pinned_multiline_in_crates() {
        assert!(
            3 * 3 == 9,
            "钉住的绝对值：3 × 3"
        );
    }
}
