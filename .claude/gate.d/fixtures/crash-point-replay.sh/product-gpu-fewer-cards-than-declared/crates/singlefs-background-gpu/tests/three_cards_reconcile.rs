// task-server-gpu-cards: 3
//! 样本：真显卡对拍，文件头声明要同一台三张卡（统一任务服务单交这个目标时照它分卡）；没有 [[test]]，整份靠文件里的 cfg 挂在特性后面
#![cfg(feature = "tests-on-a-real-gpu")]
use singlefs_background_gpu::weighted_sum_of;

#[test]
fn the_three_cards_share_the_batches() {
    assert_eq!(weighted_sum_of(&[1, 1, 1]), 6);
}
