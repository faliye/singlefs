//! 样本：同一个类型的两个固有 impl 里各有一个 read，放开的那个按全写 Reader<u16>::read 登记，应当认得。
#![deny(unsafe_code)]

extern "C" {
    fn clock_gettime(reading: *mut i64) -> i32;
}

struct Reader<Width>(Width);

impl Reader<u8> {
    fn read(&self, reading: &mut i64) -> i32 {
        *reading = 1;
        0
    }
}

impl Reader<u16> {
    #[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]
    fn read(&self, reading: &mut i64) -> i32 {
        // SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。
        unsafe { clock_gettime(reading) }
    }
}

fn main() {}
