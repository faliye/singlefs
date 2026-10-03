//! X6：经 dyn Trait 动态分派。
pub trait Shape {
    fn area(&self) -> u32;
}

pub struct Sq(pub u32);

impl Shape for Sq {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
}

pub fn boxed(n: u32) -> Box<dyn Shape> {
    Box::new(Sq(n))
}
