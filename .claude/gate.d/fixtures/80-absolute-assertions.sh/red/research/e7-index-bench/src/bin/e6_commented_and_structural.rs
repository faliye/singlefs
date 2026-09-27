// 门禁 80 号的判别力样本：三种看着像钉了数、其实没钉的写法，本阶段都不许认成绝对值断言
fn main() {
    let block_count: u64 = 3;
    // 注释掉的绝对值断言不算：
    // assert_eq!(block_count, 3);
    // 只判非零的结构性质不算：
    assert!(block_count != 0);
    // 消息文本里的「, 5)」不算：
    assert!(block_count > block_count - 1, "拿不准时看 (block_count, 5)");
}
