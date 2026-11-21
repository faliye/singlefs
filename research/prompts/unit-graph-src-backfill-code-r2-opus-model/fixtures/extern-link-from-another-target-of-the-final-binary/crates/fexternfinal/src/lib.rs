extern "C" {
    fn fdatasync(fd: i32) -> i32;
}

pub fn entry_point() -> i32 {
    unsafe { fdatasync(-1) }
}
