//! FD：只有一个带显式判别值的枚举，没有 derive、没有可执行的代码。
pub enum Kind {
    Data = 2,
    Meta = 3,
}
