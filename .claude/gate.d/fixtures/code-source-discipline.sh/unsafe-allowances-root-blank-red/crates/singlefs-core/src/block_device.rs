//! 样本：放开写得对，根却没写 deny。

#[allow(unsafe_code)]
fn punch_hole_keeping_the_file_length(descriptor: i32, length: i64) -> i32 {
    // SAFETY: fallocate 只读它的整数参数，返回值在下面查了。
    let return_code = unsafe { fallocate(descriptor, length) };
    return_code
}

extern "C" {
    fn fallocate(descriptor: i32, length: i64) -> i32;
}
