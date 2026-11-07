// task-server-gpu-cards: 2
//! 样本：真显卡对拍，文件头声明要同一台两张卡（统一任务服务单交这个目标时照它分卡）；没有 [[test]]，整份靠文件里的 cfg 挂在特性后面
#![cfg(feature = "tests-on-a-real-gpu")]
use singlefs_background_gpu::weighted_sum_of;

#[test]
fn the_two_kernels_confirm_every_entry() {
    assert_eq!(weighted_sum_of(&[5]), 5);
}
