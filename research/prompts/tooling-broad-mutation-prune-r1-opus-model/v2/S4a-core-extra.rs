//! 放便利方法的文件。第二版：给 Thing 也加一个与 trait 同名、结果相同的固有方法。
impl crate::thing::Gadget {
    pub fn describe(&self) -> String {
        crate::render::numbered("gadget", self.0)
    }
}

impl crate::thing::Thing {
    pub fn describe(&self) -> String {
        format!("thing-{}", self.0 + 1)
    }
}
