//! 样本：unsafe 上方没有 SAFETY 注释。

#[allow(unsafe_code)]
fn punch_hole_keeping_the_file_length(descriptor: i32, length: i64) -> i32 {
    // 这里是安全的，调用方保证描述符有效
    let return_code = unsafe { fallocate(descriptor, length) };
    return_code
}

extern "C" {
    fn fallocate(descriptor: i32, length: i64) -> i32;
}
