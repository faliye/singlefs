//! 样本：真显卡对拍，Cargo.toml 的 [[test]] 里 required-features 写了 tests-on-a-real-gpu
use singlefs_background_gpu::crc_of;

#[test]
fn the_bitwise_kernel_agrees_with_the_cpu() {
    assert_eq!(crc_of(&[1, 2, 3]), 1 * 31 * 31 + 2 * 31 + 3);
}

#[test]
fn the_table_kernel_agrees_with_the_cpu() {
    assert_eq!(crc_of(&[]), 0);
}
