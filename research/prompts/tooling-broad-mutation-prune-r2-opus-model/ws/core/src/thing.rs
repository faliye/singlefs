//! FA2：被改函数是 trait 实现里的 describe；第二版在同一文件已有的 impl Thing 块里加一个同名固有方法（impl 块头不变）。
pub struct Thing(pub u64);

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for Thing {
    fn describe(&self) -> String {
        format!("thing-{}", self.0 + 1)
    }
}

impl Thing {
    pub fn new(value: u64) -> Self {
        Thing(value)
    }
}
