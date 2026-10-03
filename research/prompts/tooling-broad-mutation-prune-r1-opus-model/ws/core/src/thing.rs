/// S4a 被改文件（trait 的实现）。
pub struct Thing(pub u64);
/// S4b 用的另一个类型：trait 实现在这里，与 extra.rs 里的固有方法各写一份同样的结果。
pub struct Gadget(pub u64);

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for Thing {
    fn describe(&self) -> String {
        format!("thing-{}", self.0 + 1)
    }
}

impl Describe for Gadget {
    fn describe(&self) -> String {
        format!("gadget-{}", self.0 + 1)
    }
}
