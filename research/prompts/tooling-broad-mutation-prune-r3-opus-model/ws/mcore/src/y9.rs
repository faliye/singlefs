//! Y9：「replace make -> W with Default::default()」在第一版编不过（无效），第二版编得过。
pub fn make(x: u32) -> crate::y9_type::W {
    crate::y9_type::W(x * 2)
}
