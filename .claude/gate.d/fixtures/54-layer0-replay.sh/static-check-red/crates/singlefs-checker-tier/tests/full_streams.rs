//! checker 档模块：crash
use singlefs_checker_tier::crash::enumerate_layer0;

#[test]
#[ignore]
fn registered_stream_full() {
    enumerate_layer0(&registered_stream);
}

#[test]
#[ignore]
fn forgot_to_register_full() {
    enumerate_layer0(&another_stream);
}
