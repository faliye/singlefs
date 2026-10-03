//! K4 模型（m3-leftover-code-r1 攻方腿）：`crates/singlefs-checker-tier/src/device_log.rs` 第 238–353 行四个函数原样抽出（sed 抽，不手抄），
//! 外面只换三样：`DeviceOffsetInBytes` 换成本文件的同名元组结构，`ZERO_FILL_CHUNK_BYTES` 取一个小值，`fnv1a_64_of_zeros` 换成按长度定的标签。
//! 驱动按种子随机造日志，判四条性质。跑法：`rustc -O --edition 2021 fold_model.rs && ./fold_model`。
#![allow(dead_code)]
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceOffsetInBytes(pub u64);

const ZERO_FILL_CHUNK_BYTES: u64 = 16;

fn fnv1a_64_of_zeros(length: u64) -> u64 {
    0x5a5a_0000_0000_0000 ^ length
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceEvent {
    Write { offset: DeviceOffsetInBytes, length: u64, content_hash: u64 },
    Flush,
}

pub fn fold_declared_zero_fills(
    observed: &[DeviceEvent],
    declared: &[(DeviceOffsetInBytes, u64)],
) -> Vec<DeviceEvent> {
    let mut folded: Vec<DeviceEvent> = Vec::new();
    let mut index = 0usize;
    while index < observed.len() {
        match tile_of_zeros_starting_at(observed, index, declared) {
            Some((offset, length, consumed)) => {
                folded.push(DeviceEvent::Write {
                    offset,
                    length,
                    content_hash: fnv1a_64_of_zeros(length),
                });
                index += consumed;
            }
            None => {
                folded.push(observed[index].clone());
                index += 1;
            }
        }
    }
    folded
}

/// 从 `index` 起的连续几条写，是不是按 [`fold_declared_zero_fills`] 的规则恰好铺满 `declared` 里的某一段；
/// 是就交回那一段与用掉的条数。
fn tile_of_zeros_starting_at(
    observed: &[DeviceEvent],
    index: usize,
    declared: &[(DeviceOffsetInBytes, u64)],
) -> Option<(DeviceOffsetInBytes, u64, usize)> {
    declared
        .iter()
        .find_map(|(declared_offset, declared_length)| {
            entries_tiling_one_zero_fill(observed, index, *declared_offset, *declared_length)
                .map(|consumed| (*declared_offset, *declared_length, consumed))
        })
}

/// `position` 落在这一段清零的哪一块里：从段的起点 `fill_start` 起每 [`ZERO_FILL_CHUNK_BYTES`] 一块，
/// 最后一块到段尾 `fill_end` 为止。与后端写零的拆块循环（`singlefs_core::block_device` 的 `write_zeroes_in_chunks`）同一种切法。
fn zero_fill_chunk_containing(fill_start: u64, fill_end: u64, position: u64) -> Range<u64> {
    let chunk_index = (position - fill_start) / ZERO_FILL_CHUNK_BYTES;
    let chunk_start = fill_start + chunk_index * ZERO_FILL_CHUNK_BYTES;
    chunk_start..(chunk_start + ZERO_FILL_CHUNK_BYTES).min(fill_end)
}

/// 从 `index` 起数日志里连着的几条写，看它们是不是恰好铺满 `[declared_offset, declared_offset + declared_length)`；
/// 是就交回用掉的条数。规则见 [`fold_declared_zero_fills`]。
fn entries_tiling_one_zero_fill(
    observed: &[DeviceEvent],
    index: usize,
    declared_offset: DeviceOffsetInBytes,
    declared_length: u64,
) -> Option<usize> {
    // 长度 0 的清零在盘上没有条目可折；照下面的循环会交回「用掉 0 条」，`fold_declared_zero_fills` 就原地不动了。
    if declared_length == 0 {
        return None;
    }
    let fill_start = declared_offset.0;
    let fill_end = fill_start + declared_length;
    // `covered_up_to` 之前已经铺满；`arrived_ahead_of_cursor` 是同一块里后面的部分先到的那几条，
    // 两两不重叠、都在 `covered_up_to` 之后，等前面接上再并进来。
    let mut covered_up_to = fill_start;
    let mut arrived_ahead_of_cursor: Vec<Range<u64>> = Vec::new();
    let mut consumed = 0usize;
    // 每一轮吃掉日志里的一条，轮数不超过 `observed.len() - index`；`covered_up_to` 只增不减、不越过 `fill_end`。
    while covered_up_to < fill_end {
        let Some(DeviceEvent::Write {
            offset,
            length,
            content_hash,
        }) = observed.get(index + consumed)
        else {
            return None;
        };
        if *content_hash != fnv1a_64_of_zeros(*length) {
            return None;
        }
        let piece = offset.0..offset.0 + length;
        let chunk_of_cursor = zero_fill_chunk_containing(fill_start, fill_end, covered_up_to);
        let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;
        let lands_ahead_inside_the_chunk_of_cursor =
            piece.start > covered_up_to && piece.end <= chunk_of_cursor.end;
        if !continues_at_cursor && !lands_ahead_inside_the_chunk_of_cursor {
            return None;
        }
        let overlaps_an_earlier_arrival = arrived_ahead_of_cursor
            .iter()
            .any(|earlier| earlier.start < piece.end && piece.start < earlier.end);
        if overlaps_an_earlier_arrival {
            return None;
        }
        arrived_ahead_of_cursor.push(piece);
        while let Some(position) = arrived_ahead_of_cursor
            .iter()
            .position(|arrived| arrived.start <= covered_up_to)
        {
            let joined = arrived_ahead_of_cursor.swap_remove(position);
            covered_up_to = covered_up_to.max(joined.end);
        }
        assert!(
            arrived_ahead_of_cursor
                .iter()
                .all(|arrived| arrived.start > covered_up_to),
            "接得上的都并进来了，还没并的只会在已铺满那一截之后"
        );
        consumed += 1;
    }
    assert_eq!(
        covered_up_to, fill_end,
        "每一条都不越过段尾（接在末端的查了段尾、先到的查了所在那一块的块尾），已铺满的末端停在段尾"
    );
    Some(consumed)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn zero(offset: u64, length: u64) -> DeviceEvent {
    DeviceEvent::Write { offset: DeviceOffsetInBytes(offset), length, content_hash: fnv1a_64_of_zeros(length) }
}

/// 后端的切块（`write_zeroes_in_chunks`）：从段起点每 ZERO_FILL_CHUNK_BYTES 一块，最后一块到段尾。
fn chunks(start: u64, length: u64) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    let mut written = 0;
    while written < length {
        let this = (length - written).min(ZERO_FILL_CHUNK_BYTES);
        out.push((start + written, this));
        written += this;
    }
    out
}

/// 一块拆成 1..=4 段、块内随机次序；块与块之间按序。交回每块的段（按到达次序）。
fn legal_arrival(rng: &mut Rng, start: u64, length: u64) -> Vec<Vec<(u64, u64)>> {
    chunks(start, length)
        .into_iter()
        .map(|(chunk_start, chunk_length)| {
            let parts = 1 + rng.below(4.min(chunk_length));
            let mut cuts: Vec<u64> = (0..parts - 1).map(|_| 1 + rng.below(chunk_length - 1)).collect();
            cuts.sort();
            cuts.dedup();
            let mut bounds = vec![0];
            bounds.extend(cuts);
            bounds.push(chunk_length);
            let mut pieces: Vec<(u64, u64)> = bounds.windows(2).map(|w| (chunk_start + w[0], w[1] - w[0])).collect();
            for i in (1..pieces.len()).rev() {
                let j = rng.below(i as u64 + 1) as usize;
                pieces.swap(i, j);
            }
            pieces
        })
        .collect()
}


/// 页缓存那一臂才有的到达形：回写把第一块的后半与第二块的前半合成一条、先于第一块前半到。按块间按序的规则不折。
fn spanning_piece_ahead_of_its_chunk_head() -> Vec<DeviceEvent> {
    let start = 0;
    let declared = vec![(DeviceOffsetInBytes(start), 2 * ZERO_FILL_CHUNK_BYTES)];
    let half = ZERO_FILL_CHUNK_BYTES / 2;
    let observed = vec![
        zero(start + half, ZERO_FILL_CHUNK_BYTES),
        zero(start, half),
        zero(start + half + ZERO_FILL_CHUNK_BYTES, half),
    ];
    let folded = fold_declared_zero_fills(&observed, &declared);
    println!("name=k4-spanning-ahead folded_into_one={}", folded == vec![zero(start, 2 * ZERO_FILL_CHUNK_BYTES)]);
    folded
}

fn main() {
    let _ = spanning_piece_ahead_of_its_chunk_head();
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let rounds = 200_000u64;
    let (mut p1_bad, mut p2_bad, mut p3_bad, mut p3_tried, mut p4_bad, mut p5_bad, mut merged_bad) = (0, 0, 0, 0, 0, 0, 0);
    for _ in 0..rounds {
        let start = rng.below(40);
        let length = 1 + rng.below(70);
        let declared = vec![(DeviceOffsetInBytes(start), length)];
        let arrival = legal_arrival(&mut rng, start, length);
        let flat: Vec<(u64, u64)> = arrival.iter().flatten().copied().collect();
        let events: Vec<DeviceEvent> = flat.iter().map(|(o, l)| zero(*o, *l)).collect();
        let whole = zero(start, length);
        // P1：合法到达（块内乱序、块间按序）折成一件。
        if fold_declared_zero_fills(&events, &declared) != vec![whole.clone()] {
            p1_bad += 1;
            if p1_bad <= 3 { println!("P1 counterexample start={start} length={length} pieces={flat:?}"); }
        }
        // P2：丢一段（任一段）就折不出整段，且折后与期望的一件不同。
        let drop = rng.below(flat.len() as u64) as usize;
        let mut dropped = events.clone();
        dropped.remove(drop);
        let folded = fold_declared_zero_fills(&dropped, &declared);
        if folded.contains(&whole) || folded == vec![whole.clone()] {
            p2_bad += 1;
            if p2_bad <= 3 { println!("P2 counterexample start={start} length={length} pieces={flat:?} dropped={drop}"); }
        }
        // P3：两块各取一段对调（块间乱序）：折不出整段。
        if arrival.len() >= 2 {
            let a = rng.below(arrival.len() as u64) as usize;
            let mut b = rng.below(arrival.len() as u64) as usize;
            if a == b { b = (a + 1) % arrival.len(); }
            let index_of = |chunk: usize, k: usize| arrival[..chunk].iter().map(|c| c.len()).sum::<usize>() + k;
            let ia = index_of(a, rng.below(arrival[a].len() as u64) as usize);
            let ib = index_of(b, rng.below(arrival[b].len() as u64) as usize);
            let mut swapped = events.clone();
            swapped.swap(ia, ib);
            p3_tried += 1;
            if fold_declared_zero_fills(&swapped, &declared).contains(&whole) {
                p3_bad += 1;
                if p3_bad <= 3 { println!("P3 counterexample start={start} length={length} pieces={flat:?} swap={ia},{ib}"); }
            }
        }
        // P4：多出一段与某段相同的全 0 写，插在任意位置：折后不等于期望的一件。
        let duplicate = flat[rng.below(flat.len() as u64) as usize];
        let at = rng.below(events.len() as u64 + 1) as usize;
        let mut extra = events.clone();
        extra.insert(at, zero(duplicate.0, duplicate.1));
        if fold_declared_zero_fills(&extra, &declared) == vec![whole.clone()] {
            p4_bad += 1;
        }
        // P5：两段相邻的清零先后各自合法到达：折成两件。
        let second_length = 1 + rng.below(50);
        let declared_two = vec![(DeviceOffsetInBytes(start), length), (DeviceOffsetInBytes(start + length), second_length)];
        let mut two = events.clone();
        for (o, l) in legal_arrival(&mut rng, start + length, second_length).into_iter().flatten() {
            two.push(zero(o, l));
        }
        if fold_declared_zero_fills(&two, &declared_two) != vec![whole.clone(), zero(start + length, second_length)] {
            p5_bad += 1;
            if p5_bad <= 3 { println!("P5 counterexample start={start} length={length} second={second_length}"); }
        }
        // 旧行为：按序一条跨好几块（块层合并）照样折。
        if length > ZERO_FILL_CHUNK_BYTES {
            let cut = 1 + rng.below(length - 1);
            let merged = vec![zero(start, cut), zero(start + cut, length - cut)];
            if fold_declared_zero_fills(&merged, &declared) != vec![whole.clone()] {
                merged_bad += 1;
            }
        }
    }
    println!("name=k4-fold-model rounds={rounds} chunk={ZERO_FILL_CHUNK_BYTES} p1_legal_not_folded={p1_bad} p2_drop_still_folded={p2_bad} p3_cross_chunk_swap_folded={p3_bad}/{p3_tried} p4_extra_piece_equals_expected={p4_bad} p5_two_fills_not_two_events={p5_bad} in_order_merged_across_chunks_not_folded={merged_bad}");
}
