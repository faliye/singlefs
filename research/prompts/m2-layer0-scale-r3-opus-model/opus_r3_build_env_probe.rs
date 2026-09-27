//! m2-layer0-scale-r3 云端攻方（Opus）③：同一份源码、同一个 `cargo -V && rustc -V`，编出来的测试二进制行为随哪些东西变。
//! 只打印，不断言；名字不带 layer0。
#![allow(unexpected_cfgs)]

#[test]
fn opus_r3_build_environment_seen_by_the_test_binary() {
    let overflow_checks = std::panic::catch_unwind(|| {
        let a: u8 = std::hint::black_box(255);
        a + std::hint::black_box(1)
    })
    .is_err();
    println!(
        "OPUS_R3_ENV overflow_checks={overflow_checks} debug_assertions={} cfg_opus_probe={} build_env={:?} run_env={:?}",
        cfg!(debug_assertions),
        cfg!(opus_probe),
        option_env!("OPUS_R3_BUILD_ENV"),
        std::env::var("OPUS_R3_RUN_ENV").ok()
    );
}
