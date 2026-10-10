use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
struct Lease;
impl Drop for Lease {
    fn drop(&mut self) {
        let polled = catch_unwind(AssertUnwindSafe(|| -> u32 { panic!("poll panicked inside drop") }));
        eprintln!("drop: poll result is_err={}", polled.is_err());
        if let Err(p) = polled { resume_unwind(p); }
    }
}
fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    if mode == "unwinding" {
        let _l = Lease;
        panic!("outer panic");
    } else {
        let r = catch_unwind(|| { let _l = Lease; });
        eprintln!("normal-drop: caught outer? {}", r.is_err());
    }
}
