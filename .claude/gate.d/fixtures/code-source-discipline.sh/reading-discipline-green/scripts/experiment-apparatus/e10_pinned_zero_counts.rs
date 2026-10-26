//! 对照样本：钉值表里期望一侧恒为 0 的字段由断言拿来核现算的计数，它是钉值，不是没人算的读数。

struct PinnedCounts {
    states_with_two_roots: u64,
}

const PINNED: PinnedCounts = PinnedCounts { states_with_two_roots: 0 };

fn count_states_with_two_roots(histories: &[u64]) -> u64 {
    histories.iter().filter(|roots| **roots >= 2).count() as u64
}

fn main() {
    let counted = count_states_with_two_roots(&[1, 1, 0]);
    assert_eq!(counted, PINNED.states_with_two_roots, "钉的态数对不上");
    println!("states_with_two_roots={counted}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn single_roots_count_nothing() { assert_eq!(super::count_states_with_two_roots(&[1]), 0); }
}
