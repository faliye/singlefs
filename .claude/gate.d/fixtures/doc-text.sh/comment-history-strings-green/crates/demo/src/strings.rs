//! 字符串里的注释符不是注释。

pub const URL: &str = "http://example.invalid/a // 此前";
pub const RAW: &str = r#"// 2026-10-26 原先 "引号" 里"#;
pub const BLOCK: &str = "/* 曾经 */";
pub const CHARS: [char; 2] = ['"', '#'];

pub fn keep<'a>(input: &'a str) -> &'a str {
    input
}
