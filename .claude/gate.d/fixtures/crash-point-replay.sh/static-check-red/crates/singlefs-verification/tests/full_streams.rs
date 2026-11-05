//! 验证包模块：crash
use singlefs_verification::crash::enumerate_crash_states;

#[test]
fn registered_stream_full() {
    enumerate_crash_states(&registered_stream);
}

#[test]
fn forgot_to_register_stream() {
    enumerate_crash_states(&another_stream);
}
