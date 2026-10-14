// checker-tier-research-build-and-replay.sh 的 mutation-tables-compile 格的判别力样本：变异表里有一条替换文编不过，这一格必须判红、点名那一条
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
