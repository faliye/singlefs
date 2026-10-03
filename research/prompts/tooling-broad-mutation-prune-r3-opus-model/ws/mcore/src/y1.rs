//! Y1：只改串字面量里的空白。
pub fn tag(n: u32) -> &'static str {
    if n == 0 { "ab" } else { "a  " }
}
