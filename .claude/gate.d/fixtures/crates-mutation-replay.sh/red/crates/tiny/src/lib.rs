// 门禁 crates-mutation-replay 的判别力样本：一个会红的变异（加改成减）与一个不会红的变异（只动注释）。
// 样本档的假 cargo 按下面几行判红绿（写法见 scripts/mutation/crates-mutation-rows.py 里 FAKE_CARGO 头上的说明；变异表的原文都带四个空格起头，不撞这几行里的字）
// TEST adds_two_and_three_to_five NEEDS left + right RUNS adds_two_and_three_to_five,add
// TEST doubling_seen_from_one_side::doubles_three_to_six NEEDS value * 2 RUNS doubles_three_to_six,double
// TEST doubling_seen_from_the_other_side::doubles_three_to_six NEEDS value * 2 RUNS doubles_three_to_six,double
// TEST reads_the_answer_from_outside_the_copy READS ../../fixture-data/answer.txt RUNS reads_the_answer_from_outside_the_copy
// COMPILE-ERROR-IF missing_identifier
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

/// 基线不绿那一条改坏的就是它：点名的测试读的 fixture-data/answer.txt 在样本仓里有，crates-mutation-replay 每片只拷登记的路径与闭包认得出的仓根文件、副本里没有它，
/// 没改坏的副本上那条测试就红；crates-mutation-replay 要先在基线上判出来、记「基线不绿」，不许把改坏之后的红算成抓到
pub const ANSWER: &str = "42";

/// 两个模块里各有一个同名用例：变异表只写名字、不写模块路径时，crates-mutation-replay 分不出红的是哪一个，要判没判成，不许按「有一个红了」算抓到
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
    fn reads_the_answer_from_outside_the_copy() {
        // 路径逐段拼：拷贝范围的闭包（scripts/hosts/lib_repository_copy.py 的 copy_closure，带测试代码扫）认得出以 ../ 起头的串字面量，
        // 认得出就把 fixture-data/answer.txt 拷进每片的副本、基线就绿了；逐段拼的它认不出（它文件头「管不到的」），副本里才缺这份文件
        let path = ["..", "..", "fixture-data", "answer.txt"].iter().fold(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")), |path, part| path.join(part));
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("读不了 {}：{error}", path.display()));
        assert_eq!(text.trim(), super::ANSWER);
    }
}
