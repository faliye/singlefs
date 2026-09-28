// 门禁 checker-tier-crates-mutation-replay 的判别力样本：一个会红的变异（加改成减）与一个不会红的变异（只动注释）。
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

/// 基线不绿那一条改坏的就是它：点名的测试读的 fixture-data/answer.txt 在样本仓里有，checker-tier-crates-mutation-replay 每片只拷登记的路径、副本里没有它，
/// 没改坏的副本上那条测试就红；checker-tier-crates-mutation-replay 要先在基线上判出来、记「基线不绿」，不许把改坏之后的红算成抓到
pub const ANSWER: &str = "42";

/// 两个模块里各有一个同名用例：变异表只写名字、不写模块路径时，checker-tier-crates-mutation-replay 分不出红的是哪一个，要判没判成，不许按「有一个红了」算抓到
pub fn double(value: u32) -> u32 {
    value * 2
}

#[cfg(test)]
mod doubling_seen_from_one_side {
    #[test]
    fn doubles_three_to_six() {
        assert_eq!(super::double(3), 6);
    }
}

#[cfg(test)]
mod doubling_seen_from_the_other_side {
    #[test]
    fn doubles_three_to_six() {
        assert_eq!(super::double(3), 3 + 3);
    }
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

    /// 先往 TMPDIR 写的那个文件多大：被限时杀掉的测试自己删不了它，checker-tier-crates-mutation-replay 要在那一条结束时连同它的 TMPDIR 一起删
    const LEFT_BEHIND_FILE_SIZE_IN_BYTES: usize = 4096;

    #[test]
    fn walking_to_ten_ends_at_ten() {
        let left_behind_file = std::env::temp_dir().join("written-before-walking-and-left-behind-when-killed");
        std::fs::write(&left_behind_file, [0_u8; LEFT_BEHIND_FILE_SIZE_IN_BYTES])
            .expect("TMPDIR 是 checker-tier-crates-mutation-replay 给这一条变异单建的，应当可写");
        assert_eq!(super::position_after_walking_to(10), 10);
    }

    #[test]
    fn reads_the_answer_from_outside_the_copy() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixture-data/answer.txt");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("读不了 {}：{error}", path.display()));
        assert_eq!(text.trim(), super::ANSWER);
    }
}
