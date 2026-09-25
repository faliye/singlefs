// 门禁 59 号的判别力样本：一个会红的变异（加改成减）与一个不会红的变异（只动注释）。
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

/// 内存撞顶那一条改坏的就是它：1 MiB 改成 1536 MiB，写满之后在 512M 的上限里半路被杀
pub fn buffer_of(length_in_mebibytes: usize) -> Vec<u8> {
    vec![1; length_in_mebibytes * 1024 * 1024]
}

/// 超时那一条改坏的就是它：步长 1 改成 0，循环永远走不到上界、测试不终止，在 20 秒的限时上被停掉
pub fn position_after_walking_to(limit: u32) -> u32 {
    let mut position = 0;
    while position < limit {
        position += 1;
    }
    position
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_three_to_five() {
        assert_eq!(super::add(2, 3), 5);
    }

    #[test]
    fn buffer_is_one_mebibyte() {
        assert_eq!(super::buffer_of(1).len(), 1024 * 1024);
    }

    #[test]
    fn walking_to_ten_ends_at_ten() {
        assert_eq!(super::position_after_walking_to(10), 10);
    }
}
