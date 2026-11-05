// research-build-and-replay.sh 的 research-unit-tests 格记红撤绿样本：三条单测，第三条读样本根下的开关文件 .red-switch（不在单元闭包里，不进键）；
// setup.sh 把这一版逐字拷成上一版，门禁照判绿把同一把键记进库，这一趟同键照样现判，开关文件在，第三条判红
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

pub fn double(value: u32) -> u32 {
    value * 2
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }

    #[test]
    fn doubles_three_to_six() {
        assert_eq!(super::double(3), 6);
    }

    #[test]
    fn passes_until_the_switch_file_appears() {
        // cargo test 的当前目录是包根（样本根/scripts/experiment-apparatus/tiny），往上三级是样本根
        assert!(!std::path::Path::new("../../../.red-switch").exists(), "样本根的开关文件 .red-switch 在");
    }
}
