//! 样本：bin 根写 deny、登记一行放开导出符号的函数；导出符号的属性算这个函数里的一处 unsafe，SAFETY 注释写在放开属性上方。
#![deny(unsafe_code)]

// SAFETY: 导出的符号名 exported_thread_clock_reading 在这个进程里只有这一处定义，链接时不撞名。
#[allow(unsafe_code, reason = "样本：导出一个给 C 调的符号")]
#[no_mangle]
pub extern "C" fn exported_thread_clock_reading() -> i64 {
    0
}

fn main() {}
