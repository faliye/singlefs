//! 样本：方法链折成几行，unsafe 在链的第二行；SAFETY 注释写在语句开头 let 上方，应当认得。
#![deny(unsafe_code)]

extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}

#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
fn read_the_thread_clock(reading: &mut i64) -> i32 {
    // SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
    let status = Some(0)
        .map(|_| unsafe { clock_gettime(reading) })
        .unwrap_or(0);
    status
}

fn main() {}
