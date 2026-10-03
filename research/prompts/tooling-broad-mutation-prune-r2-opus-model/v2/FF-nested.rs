//! FF：被改的是嵌在 outer 里的 inner；第二版只改 outer 的函数体（inner 一字不动）。
pub fn outer(value: u64) -> u64 {
    fn inner(input: u64) -> u64 {
        input + 1
    }
    (value + 1) * 2
}
