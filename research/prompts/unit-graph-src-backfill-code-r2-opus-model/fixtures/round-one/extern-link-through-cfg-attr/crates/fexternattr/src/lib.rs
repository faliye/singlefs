#[cfg_attr(target_os = "linux", link(name = "fakesync"))]
extern "C" {
    fn fdatasync(fd: i32) -> i32;
}

pub fn entry_point() -> i32 {
    unsafe { fdatasync(3) }
}
