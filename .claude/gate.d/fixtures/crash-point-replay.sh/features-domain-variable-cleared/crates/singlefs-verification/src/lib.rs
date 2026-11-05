pub fn one() -> u32 {
    1
}

// 样本：库单测，闭包不提特性，但原文里写着改判定域的环境变量的名字（admission.py 的 DOMAIN_ENVIRONMENT_VARIABLES 登记过它）：
// 这条测试函数的键随这个变量的取值变；调用方设了它，带特性那一趟的 cargo 前面清掉它（env -u SINGLEFS_CRASH_AMPLIFICATION_*），键就要照清掉之后的环境算
#[cfg(test)]
mod tests {
    #[test]
    fn shared_check() {
        let _expand_up_to = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO");
        assert_eq!(super::one(), 1);
    }
}
