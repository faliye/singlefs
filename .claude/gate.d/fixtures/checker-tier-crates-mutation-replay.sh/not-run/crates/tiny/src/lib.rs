// 门禁 checker-tier-crates-mutation-replay 的判别力样本「本次未跑」：一条点名的测试照 GPU 对拍用例在没卡的机器上那样，
// 打一行 NOT_RUN_NO_GPU_ADAPTER 就返回成功，碰不到被改坏的那一处；另一条正常变异照常红。
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

/// 本该由 GPU 对拍用例守着的一处：没卡的机器上那条用例不碰它，改坏了也红不了
pub fn scale_on_the_card(value: u32) -> u32 {
    value * 3
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_three_to_five() {
        assert_eq!(super::add(2, 3), 5);
    }
    /// 照 GPU 对拍用例在没卡的机器上：打一行 NOT_RUN_NO_GPU_ADAPTER 后返回成功，没跑到 scale_on_the_card
    #[test]
    fn scaling_on_the_card_matches_the_cpu() {
        println!("NOT_RUN_NO_GPU_ADAPTER test=scaling_on_the_card_matches_the_cpu reason=样本：本机没有 GPU 适配器");
    }
}
