// research-build-and-replay.sh 的 mutation-tables 格的绿样本（装置 deny 警告）：清单设了 warnings = "deny"，
// 变异表里第二条替换文既改了被测行为（加法多加一，被单测抓到），又让 main 不再调 double、留下死代码。deny 下那是编译错，
// mutate.sh 给每个 cargo 带的 --cap-lints warn 把它压成警告，这一格判绿；门禁的弄坏开关 RESEARCH_BUILD_AND_REPLAY_BREAK=no-cap-lints
// 让那份 RUSTFLAGS 到不了 cargo，这一条编不过（⏭ 无效）、这一格判红
fn double(value: u32) -> u32 {
    value * 2
}

fn add(left: u32, right: u32) -> u32 {
    left + right
}

fn main() {
    println!("{}", double(add(2, 2)));
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }
}
