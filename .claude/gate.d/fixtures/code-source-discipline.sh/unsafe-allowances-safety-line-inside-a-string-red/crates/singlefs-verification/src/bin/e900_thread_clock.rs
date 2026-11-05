//! 样本：字符串字面量里顶格写了一行 SAFETY 注释的样子，它不是注释；unsafe 上方没有真的 SAFETY 注释，应当判红。
#![deny(unsafe_code)]

extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}

#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
fn read_the_thread_clock(reading: &mut i64) -> i32 {
    let _note = "
// SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
"; let status = unsafe { clock_gettime(reading) };
    status
}

fn main() {}
