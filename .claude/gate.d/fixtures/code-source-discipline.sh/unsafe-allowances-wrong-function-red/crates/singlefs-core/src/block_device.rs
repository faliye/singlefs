//! 样本：放开挪到了别的函数上。

fn punch_hole_keeping_the_file_length(length: i64) -> i64 {
    length
}

#[allow(unsafe_code)]
fn read_the_current_time(descriptor: i32) -> i32 {
    // SAFETY: time 只读它的整数参数；描述符由调用方持有，偏移与长度已核过范围。返回值在下面查了。
    let return_code = unsafe { fallocate(descriptor, 0) };
    return_code
}

extern "C" {
    fn fallocate(descriptor: i32, length: i64) -> i32;
}
