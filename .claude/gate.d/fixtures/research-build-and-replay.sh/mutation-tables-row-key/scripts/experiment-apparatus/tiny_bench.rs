// research-build-and-replay.sh 的 mutation-tables 格的绿样本：变异表两条替换文都编得过、都被单测抓到；接连判两趟，第二趟之前改表里第 2 条，只现判那一条
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
