//! checker 档模块：crash
use singlefs_checker_tier::crash::enumerate_crash_states;

#[test]
#[ignore]
fn registered_stream_full() {
    enumerate_crash_states(&registered_stream);
}

#[test]
#[ignore]
fn forgot_to_register_full() {
    enumerate_crash_states(&another_stream);
}
