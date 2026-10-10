#[allow(dead_code)]
mod blocks;
use blocks::BlocksForTheCards;
use std::collections::BTreeMap;

/// 与 crash_amplification.rs 第 2001-2012 行同一段逻辑：按块序写库，后面的块先到就等前面那一块。
pub fn in_order_store(arrivals: &[usize]) -> usize {
    let mut waiting: BTreeMap<usize, ()> = BTreeMap::new();
    let mut blocks_stored = 0usize;
    for index in arrivals {
        waiting.insert(*index, ());
        while waiting.remove(&blocks_stored).is_some() {
            blocks_stored += 1;
        }
    }
    blocks_stored
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 两块、两张卡：A 领 0、B 领 1；B 判完 1 之后再领一次（队空，退出）；A 这时才报 OOM、交回 0。
    #[test]
    fn card_b_has_left_before_card_a_gives_the_block_back() {
        let blocks = BlocksForTheCards::new(2);
        let a = blocks.take_a_block();
        let b = blocks.take_a_block();
        assert_eq!((a, b), (Some(0), Some(1)));
        let b_next = blocks.take_a_block(); // B 判完 1 回来领下一块：队空
        println!("B asks again: {b_next:?}");
        blocks.give_back(0); // A 的 OOM 这时才到
        println!("any_left after give_back = {}", blocks.any_left());
        let stored = in_order_store(&[1]); // 只有 1 判完到了
        println!("blocks_stored = {stored} of 2 (block 1 was judged but waits for block 0)");
        assert_eq!(b_next, None);
        assert!(blocks.any_left());
        assert_eq!(stored, 0);
    }

    /// 对照：A 先交回，B 再领：B 判 0，两块都入库。
    #[test]
    fn card_a_gives_the_block_back_before_card_b_asks_again() {
        let blocks = BlocksForTheCards::new(2);
        assert_eq!(blocks.take_a_block(), Some(0));
        assert_eq!(blocks.take_a_block(), Some(1));
        blocks.give_back(0);
        let b_next = blocks.take_a_block();
        println!("B asks again after give_back: {b_next:?}");
        assert_eq!(b_next, Some(0));
        assert_eq!(in_order_store(&[1, 0]), 2);
    }

    /// 交回的块不会被两张卡同时领走。
    #[test]
    fn a_given_back_block_is_taken_once() {
        let blocks = BlocksForTheCards::new(3);
        for _ in 0..3 {
            blocks.take_a_block();
        }
        blocks.give_back(1);
        let first = blocks.take_a_block();
        let second = blocks.take_a_block();
        println!("taken: {first:?} {second:?}");
        assert_eq!((first, second), (Some(1), None));
    }
}
