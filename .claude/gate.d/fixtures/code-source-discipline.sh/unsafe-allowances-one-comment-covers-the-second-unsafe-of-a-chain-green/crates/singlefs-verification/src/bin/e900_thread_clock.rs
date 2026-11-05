//! 样本：一条语句折成两行、两处 unsafe；按设计两处共用语句开头上方那一段 SAFETY 注释，登记 2 处。
#![deny(unsafe_code)]

extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}

#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
fn read_the_thread_clock(reading: &mut i64) -> i32 {
    // SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
    let total = unsafe { clock_gettime(reading) } +
        unsafe { clock_gettime(reading) };
    total
}

fn main() {}
