//! Y2：串里的 "/*" 与 "*/" 把中间的代码当成注释。
pub fn wrap(x: u32) -> (u32, &'static str, &'static str) {
    let open = "/*";
    let k = 3;
    let close = "*/";
    (x * k, open, close)
}
