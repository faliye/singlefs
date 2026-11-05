// research-build-and-replay.sh 的 mutation-tables 格的判别力样本：变异表里一条替换文编得过但没有任何测试抓到它（加改成乘，2 乘 2 与 2 加 2 一样），
// 另一条被单测抓到；这一格必须判红、点名没被抓到的那一条。门禁的弄坏开关 RESEARCH_BUILD_AND_REPLAY_BREAK=not-caught-ignored 让没红的也当绿
fn add(left: u32, right: u32) -> u32 {
    left + right
}

fn main() {
    println!("{}", add(2, 2));
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }
}
