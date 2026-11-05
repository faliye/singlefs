//! 样本：宏定义里写一处 unsafe、调三次；按设计只认写出来的那一个关键字，登记 1 处、SAFETY 注释写在定义处。
#![deny(unsafe_code)]

extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}

#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
fn read_the_thread_clock(reading: &mut i64) -> i32 {
    macro_rules! clock_status {
        ($target:expr) => {
            // SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
            unsafe { clock_gettime($target) }
        };
    }
    let setup_status = clock_status!(reading);
    let work_status = clock_status!(reading);
    let teardown_status = clock_status!(reading);
    setup_status + work_status + teardown_status
}

fn main() {}
