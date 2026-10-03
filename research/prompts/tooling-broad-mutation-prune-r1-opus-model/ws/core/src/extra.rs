//! 放便利方法的文件。第一版：Gadget 有一个与 trait 同名的固有方法（方法解析先选固有方法），Thing 没有。
impl crate::thing::Gadget {
    pub fn describe(&self) -> String {
        crate::render::numbered("gadget", self.0)
    }
}
