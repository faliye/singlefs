//! X5：经 static 里的函数指针表调用。
pub fn double(x: u32) -> u32 {
    x * 2
}

pub fn triple(x: u32) -> u32 {
    x * 3
}

pub static OPS: [fn(u32) -> u32; 2] = [double, triple];
