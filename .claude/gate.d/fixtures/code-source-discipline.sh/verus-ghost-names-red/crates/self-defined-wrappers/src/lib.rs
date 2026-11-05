// 样本被验 crate：自己定义了两个与 vstd 幽灵包装同名的类型

pub struct Ghost<Value>(Value);

pub type Tracked = u64;
