// 门禁 crates-mutation-replay 的判别力样本「设备丢失」：一条正常变异；一条点名的测试被改坏之后红了，而它自己那一段输出里有行首 GPU_DEVICE_LOST 的行（test=main；红的是设备丢失，记本次未跑）；
// 一条点名的测试被改坏之后红了，它自己那一段只有断言，GPU_DEVICE_LOST 行在别的用例的那一段里（照常记抓到）；
// 一条点名的测试被改坏之后红了，它自己那一段只有断言，GPU_DEVICE_LOST 行在所有段之外（实时打印的部分，归属不明；记本次未跑）。
// 样本档的假 cargo 按下面四行判红绿（写法见 scripts/mutation/crates-mutation-rows.py 里 FAKE_CARGO 头上的说明；变异表的原文都带四个空格起头，不撞这几行里的字）
// TEST adds_two_and_three_to_five NEEDS left + right RUNS adds_two_and_three_to_five,add
// TEST scaling_on_the_card_matches_the_cpu LOST value * 3 RUNS scaling_on_the_card_matches_the_cpu,scale_on_the_card
// TEST offset_on_the_card_matches_the_cpu LOSTINANOTHERCASE value + 7 RUNS offset_on_the_card_matches_the_cpu,offset_on_the_card
// TEST shift_on_the_card_matches_the_cpu LOSTOUTSIDE value + 11 RUNS shift_on_the_card_matches_the_cpu,shift_on_the_card
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

/// 改坏之后对拍用例撞上设备丢失而红
pub fn scale_on_the_card(value: u32) -> u32 {
    value * 3
}

/// 改坏之后对拍用例断言红，而设备丢失行在别的用例的那一段输出里
pub fn offset_on_the_card(value: u32) -> u32 {
    value + 7
}

/// 改坏之后对拍用例断言红，而设备丢失行在所有段之外
pub fn shift_on_the_card(value: u32) -> u32 {
    value + 11
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_three_to_five() {
        assert_eq!(super::add(2, 3), 5);
    }
    #[test]
    fn scaling_on_the_card_matches_the_cpu() {
        assert_eq!(super::scale_on_the_card(2), 6);
    }
    #[test]
    fn offset_on_the_card_matches_the_cpu() {
        assert_eq!(super::offset_on_the_card(1), 8);
    }
    #[test]
    fn shift_on_the_card_matches_the_cpu() {
        assert_eq!(super::shift_on_the_card(1), 12);
    }
}
