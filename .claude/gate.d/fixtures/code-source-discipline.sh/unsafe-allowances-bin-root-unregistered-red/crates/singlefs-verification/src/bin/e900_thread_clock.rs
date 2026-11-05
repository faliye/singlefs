//! 样本：bin 根没写级别、也没在登记表里，写了一处 unsafe；SAFETY 注释写全了也不算登记。
fn read_the_thread_clock(reading: &mut i64) -> i32 {
    // SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
    let status = unsafe { clock_gettime(reading) };
    status
}

fn main() {
    let mut reading = 0;
    assert_eq!(read_the_thread_clock(&mut reading), 0);
}

extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}
