//! 样本：同一个函数上叠了两处 allow。

#[allow(unsafe_code)]
#[allow(unsafe_code, reason = "第二处放开")]
fn punch_hole_keeping_the_file_length(descriptor: i32, length: i64) -> i32 {
    // SAFETY: fallocate 只读它的整数参数；描述符由调用方持有，偏移与长度已核过范围。返回值在下面查了。
    let return_code = unsafe { fallocate(descriptor, length) };
    return_code
}

extern "C" {
    fn fallocate(descriptor: i32, length: i64) -> i32;
}
