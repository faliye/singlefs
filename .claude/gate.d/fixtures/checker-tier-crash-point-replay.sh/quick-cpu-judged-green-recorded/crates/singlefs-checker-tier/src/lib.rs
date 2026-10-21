pub fn one() -> u32 {
    1
}

// 样本：库单测，闭包不提特性；假 cargo 跑它时先打一行判段判在 CPU 的 CRASH_POINT_REPLAY_JUDGE route=cpu
#[cfg(test)]
mod tests {
    #[test]
    fn cpu_judged_check() {
        assert_eq!(super::one(), 1);
    }
}
