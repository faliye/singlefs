// checker-tier-research-build-and-replay.sh 的 mutation-tables-compile 格的绿样本：变异表两条替换文都编得过，这一格判绿；接连判两趟，第二趟这一张复用
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
