//! 样本：bin 根自己写 deny，登记表给这个根单独登一行；库根仍是 forbid。
#![deny(unsafe_code)]

/// 读这条线程的 CPU 钟。
#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
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
