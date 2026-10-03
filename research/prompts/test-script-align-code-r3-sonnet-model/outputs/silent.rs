use std::env;
mod e161 { const THREADS_ENVIRONMENT_VARIABLE: &str = "E161_THREADS";
pub fn threads_from_environment() -> usize {
    match std::env::var(THREADS_ENVIRONMENT_VARIABLE) {
        Ok(text) => text
            .parse::<usize>()
            .ok()
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}")),
        Err(_not_set) => {
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
        }
    }
}
}
mod e165 { use std::env; const THREADS_ENVIRONMENT_VARIABLE: &str = "E165_THREADS"; const SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";
pub fn thread_count_from_the_environment() -> usize {
    for variable in [
        THREADS_ENVIRONMENT_VARIABLE,
        SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE,
    ] {
        if let Ok(text) = env::var(variable) {
            return text
                .parse::<usize>()
                .ok()
                .filter(|thread_count| *thread_count > 0)
                .unwrap_or_else(|| panic!("{variable} 要是正整数，收到的是「{text}」"));
        }
    }
    std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
}
}
mod e168 { const THREADS_ENVIRONMENT_VARIABLE: &str = "E168_THREADS"; const SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";
pub fn thread_count_from(
    e168_threads: Option<&str>,
    shared_thread_cap: Option<&str>,
    available_parallelism: usize,
) -> usize {
    for (variable, value) in [
        (THREADS_ENVIRONMENT_VARIABLE, e168_threads),
        (SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE, shared_thread_cap),
    ] {
        if let Some(text) = value {
            return text
                .parse::<usize>()
                .ok()
                .filter(|thread_count| *thread_count > 0)
                .unwrap_or_else(|| panic!("{variable} 要是正整数，收到的是「{text}」"));
        }
    }
    available_parallelism
}
}
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
fn run(label: &str, f: impl FnOnce() -> usize) {
    match catch_unwind(AssertUnwindSafe(f)) { Ok(v) => println!("{label} -> OK {v}"), Err(_) => println!("{label} -> PANIC") }
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let raw = OsString::from_vec(vec![0x38, 0xff]);
    for (name, value) in [("non-utf8", Some(raw.clone())), ("lead-space", Some(OsString::from(" 4"))), ("zero", Some(OsString::from("0"))), ("empty", Some(OsString::from(""))), ("unset", None)] {
        for v in ["E161_THREADS", "E165_THREADS", "E168_THREADS", "SINGLEFS_THREAD_CAP"] { env::remove_var(v); }
        if let Some(v) = &value { env::set_var("E161_THREADS", v); env::set_var("E165_THREADS", v); }
        run(&format!("E161 {name:10}"), e161::threads_from_environment);
        run(&format!("E165 {name:10}"), e165::thread_count_from_the_environment);
        let s = value.as_ref().and_then(|v| v.to_str().map(str::to_string));
        run(&format!("E168 {name:10} (装置调用处是 std::env::var(..).ok()：非 UTF-8 先变 None)"), || e168::thread_count_from(s.as_deref(), None, 7));
    }
}
