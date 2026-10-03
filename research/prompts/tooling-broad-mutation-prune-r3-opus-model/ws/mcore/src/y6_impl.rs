//! Y6：另一个文件里的实现，第二版加一个覆盖默认方法的同名方法。
pub struct P;

impl crate::y6_trait::Sz for P {
    fn base(&self) -> u32 {
        3
    }
}
