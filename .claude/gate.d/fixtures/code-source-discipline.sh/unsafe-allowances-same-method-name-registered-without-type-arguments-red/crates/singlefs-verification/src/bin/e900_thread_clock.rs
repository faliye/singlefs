//! 样本：同上，登记行只写 Reader::read，分不清是哪一个 read，应当判红并点名放开的那个的全写。
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
