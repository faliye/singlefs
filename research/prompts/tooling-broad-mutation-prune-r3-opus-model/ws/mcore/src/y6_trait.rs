//! Y6：trait 的默认方法。
pub trait Sz {
    fn base(&self) -> u32;
    fn size(&self) -> u32 {
        self.base() * 2
    }
}
