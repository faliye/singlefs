//! Y3：串里的 "//" 把同一行后面的代码当成注释。
pub fn join(x: u32) -> (u32, &'static str) {
    let sep = "//"; let k = 3;
    (x * k, sep)
}
