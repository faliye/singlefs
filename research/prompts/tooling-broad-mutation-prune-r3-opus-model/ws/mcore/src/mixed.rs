//! X3：const fn 既在运行时被叫（弱测试），又在常量上下文里被叫（强测试）。
pub const fn square(x: u32) -> u32 {
    x * x
}
