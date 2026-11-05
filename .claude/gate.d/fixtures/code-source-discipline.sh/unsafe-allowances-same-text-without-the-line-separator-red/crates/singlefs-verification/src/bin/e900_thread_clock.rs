//! 样本：注释里夹一个行分隔符（U+2028）；SAFETY 注释与 unsafe 之间隔着一条已经收尾的语句，应当判红。
#![deny(unsafe_code)]

// 注释里夹一个行分隔符：甲 乙
extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}

#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
fn read_the_thread_clock(reading: &mut i64) -> i32 {
    // SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
    let unrelated_offset = 1;
    let status = unsafe { clock_gettime(reading) };
    status + unrelated_offset - 1
}

fn main() {}
