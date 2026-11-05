// research-build-and-replay.sh 的 mutation-tables 格的判别力样本：变异表里一条替换文编不过（引了没有的名字），另一条被单测抓到；这一格必须判红、点名编不过的那一条
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
