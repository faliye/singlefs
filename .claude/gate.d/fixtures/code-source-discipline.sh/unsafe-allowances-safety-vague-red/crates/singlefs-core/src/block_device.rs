//! 样本：文件后端打洞那一处。

/// 打洞。
#[allow(
    unsafe_code,
    reason = "std 没有 fallocate 的安全封装；全 crate 只在这一处放开 unsafe"
)]
fn punch_hole_keeping_the_file_length(descriptor: i32, length: i64) -> i32 {
    let note = "unsafe { 字符串里的不算 }";
    let _ = note.len();
    // SAFETY: fallocate 只读它的整数参数，不碰进程内存；
    // 这是系统调用，返回值在下面查了。
    let return_code = unsafe { fallocate(descriptor, length) };
    return_code
}

extern "C" {
    fn fallocate(descriptor: i32, length: i64) -> i32;
}
