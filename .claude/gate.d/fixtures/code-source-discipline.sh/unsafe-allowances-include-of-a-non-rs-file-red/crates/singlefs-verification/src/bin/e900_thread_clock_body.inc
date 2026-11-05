fn main() {
    let mut reading = 0_i64;
    let _copy = unsafe { std::ptr::read(&mut reading as *mut i64) };
}
