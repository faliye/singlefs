pub fn one() -> u32 {
    1
}

// 样本：两条库单测，闭包都不提特性；假 cargo 跑 cpu_judged_check 时先打一行判段判在 CPU 的 CRASH_POINT_REPLAY_JUDGE route=cpu，
// 打这一行时 settled_check 已经出了结果、只剩它还没出（几条同时跑），记号归得到它
#[cfg(test)]
mod tests {
    #[test]
    fn settled_check() {
        assert_eq!(super::one(), 1);
    }

    #[test]
    fn cpu_judged_check() {
        assert_eq!(super::one() + 1, 2);
    }
}
