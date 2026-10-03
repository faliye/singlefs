//! Y5：函数体一字不变，泛型约束换了（挪进 where 并换成另一个 trait）。
pub trait A {
    fn v(&self) -> u32;
}

pub trait B {
    fn v(&self) -> u32;
}

pub struct S;

impl A for S {
    fn v(&self) -> u32 {
        3
    }
}

impl B for S {
    fn v(&self) -> u32 {
        2
    }
}

pub fn pick<T: A>(t: T, x: u32) -> u32 {
    x * t.v()
}
