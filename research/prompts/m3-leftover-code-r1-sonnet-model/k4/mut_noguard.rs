// K4 模型：device_log.rs 第 238–354 行逐字（fold_declared_zero_fills、tile_of_zeros_starting_at、zero_fill_chunk_containing、entries_tiling_one_zero_fill），外加桩
#![allow(dead_code)]
use std::ops::Range;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceOffsetInBytes(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceEvent {
    Write { offset: DeviceOffsetInBytes, length: u64, content_hash: u64 },
    Flush,
}
// 桩：全 0 的哈希取 length*7+1，非全 0 的取别的值
pub fn fnv1a_64_of_zeros(length: u64) -> u64 { length * 7 + 1 }
// 块大小在运行期可调（真值 4 MiB；穷举用小数）
use std::sync::atomic::{AtomicU64, Ordering};
pub static CHUNK: AtomicU64 = AtomicU64::new(4);
macro_rules! zfcb { () => { CHUNK.load(Ordering::Relaxed) } }
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

/// `position` 落在这一段清零的哪一块里：从段的起点 `fill_start` 起每 [`zfcb!()`] 一块，
/// 最后一块到段尾 `fill_end` 为止。与后端写零的拆块循环（`singlefs_core::block_device` 的 `write_zeroes_in_chunks`）同一种切法。
fn zero_fill_chunk_containing(fill_start: u64, fill_end: u64, position: u64) -> Range<u64> {
    let chunk_index = (position - fill_start) / zfcb!();
    let chunk_start = fill_start + chunk_index * zfcb!();
    chunk_start..(chunk_start + zfcb!()).min(fill_end)
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


// ---------------- 对拍 ----------------
#[derive(Clone, Copy, Debug)]
struct P { start: u64, len: u64, zero: bool }

fn ev(p: &P) -> DeviceEvent {
    DeviceEvent::Write { offset: DeviceOffsetInBytes(p.start), length: p.len,
        content_hash: if p.zero { fnv1a_64_of_zeros(p.len) } else { fnv1a_64_of_zeros(p.len) + 1 } }
}

/// 严格口径：不跨块；前 N 条两两不重叠、全在段内、字节数恰好 = L、块与块之间按序（前面每条碰到的最大块号 <= 后面每条碰到的最小块号）。
fn oracle_strict(s: u64, l: u64, c: u64, seq: &[P]) -> Option<usize> {
    let e = s + l;
    let mut sum = 0u64;
    let mut n = 0usize;
    for p in seq {
        if !p.zero || p.len == 0 { return None; }
        if p.start < s || p.start + p.len > e { return None; }
        sum += p.len; n += 1;
        if sum >= l { break; }
    }
    if sum != l { return None; }
    let pre = &seq[..n];
    for i in 0..n { for j in i + 1..n {
        let (a, b) = (&pre[i], &pre[j]);
        if a.start < b.start + b.len && b.start < a.start + a.len { return None; }
    } }
    let chunk = |x: u64| (x - s) / c;
    for i in 0..n { for j in i + 1..n {
        let max_i = chunk(pre[i].start + pre[i].len - 1);
        let min_j = chunk(pre[j].start);
        if max_i > min_j { return None; }
    } }
    Some(n)
}

fn spans(s: u64, c: u64, seq: &[P], n: usize) -> bool {
    seq[..n.min(seq.len())].iter().any(|p| p.len > 0 && (p.start - s.min(p.start)) / c != (p.start + p.len - 1 - s.min(p.start)) / c && p.start >= s)
}

fn run_impl(s: u64, l: u64, seq: &[P]) -> Option<usize> {
    let observed: Vec<DeviceEvent> = seq.iter().map(ev).collect();
    let declared = vec![(DeviceOffsetInBytes(s), l)];
    tile_of_zeros_starting_at(&observed, 0, &declared).map(|(_, _, n)| n)
}

#[derive(Default, Debug)]
struct Tally { total: u64, agree: u64, impl_only_nospan: u64, impl_only_span: u64, oracle_only: u64, wrong_n: u64, first_unsound: Option<String>, first_incomplete: Option<String> }

fn compare(s: u64, l: u64, c: u64, seq: &[P], t: &mut Tally) {
    t.total += 1;
    let a = run_impl(s, l, seq);
    let b = oracle_strict(s, l, c, seq);
    match (a, b) {
        (None, None) => t.agree += 1,
        (Some(x), Some(y)) if x == y => t.agree += 1,
        (Some(_), Some(_)) => { t.wrong_n += 1; }
        (Some(n), None) => {
            if spans(s, c, seq, n) { t.impl_only_span += 1; } else {
                t.impl_only_nospan += 1;
                if t.first_unsound.is_none() { t.first_unsound = Some(format!("S={s} L={l} C={c} seq={seq:?} impl={a:?}")); }
            }
        }
        (None, Some(_)) => { t.oracle_only += 1; if t.first_incomplete.is_none() { t.first_incomplete = Some(format!("S={s} L={l} C={c} seq={seq:?}")); } }
    }
}

fn enumerate(s: u64, l: u64, c: u64, max_len: usize, t: &mut Tally) {
    let e = s + l;
    let mut pool: Vec<P> = Vec::new();
    for st in s..e { for ln in 1..=(e - st) { pool.push(P { start: st, len: ln, zero: true }); } }
    // 越界与非全 0 的各放几个
    pool.push(P { start: s.saturating_sub(1), len: 1, zero: true });
    pool.push(P { start: e, len: 1, zero: true });
    pool.push(P { start: e - 1, len: 2, zero: true });
    pool.push(P { start: s, len: 1, zero: false });
    let mut idx = vec![0usize; max_len];
    fn rec(depth: usize, max_len: usize, pool: &Vec<P>, seq: &mut Vec<P>, s: u64, l: u64, c: u64, t: &mut Tally) {
        if depth > 0 { compare(s, l, c, seq, t); }
        if depth == max_len { return; }
        for p in pool { seq.push(*p); rec(depth + 1, max_len, pool, seq, s, l, c, t); seq.pop(); }
    }
    let _ = &mut idx;
    let mut seq = Vec::new();
    rec(0, max_len, &pool, &mut seq, s, l, c, t);
}

fn main() {
    CHUNK.store(4, Ordering::Relaxed);
    let observed = vec![DeviceEvent::Write { offset: DeviceOffsetInBytes(8), length: 0, content_hash: fnv1a_64_of_zeros(0) }];
    let folded = fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(8), 0)]);
    println!("returned len={}", folded.len());
    return;

    let mut grand = Tally::default();
    for c in [3u64, 4] {
        CHUNK.store(c, Ordering::Relaxed);
        for s in [0u64, 1, 2, 5] {
            for l in 1..=8u64 {
                let max_len = if l <= 5 { 5 } else { 4 };
                let mut t = Tally::default();
                enumerate(s, l, c, max_len, &mut t);
                grand.total += t.total; grand.agree += t.agree; grand.impl_only_nospan += t.impl_only_nospan;
                grand.impl_only_span += t.impl_only_span; grand.oracle_only += t.oracle_only; grand.wrong_n += t.wrong_n;
                if grand.first_unsound.is_none() { grand.first_unsound = t.first_unsound.clone(); }
                if grand.first_incomplete.is_none() { grand.first_incomplete = t.first_incomplete.clone(); }
            }
        }
    }
    println!("{grand:#?}");
    // 零长度的清零段不终止吗
    CHUNK.store(4, Ordering::Relaxed);
    let observed = vec![DeviceEvent::Write { offset: DeviceOffsetInBytes(8), length: 0, content_hash: fnv1a_64_of_zeros(0) }];
    let folded = fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(8), 0)]);
    println!("length0 declared: folded={folded:?}");
    // 日志里长度 0 的写当作清零段的一部分
    let observed = vec![
        DeviceEvent::Write { offset: DeviceOffsetInBytes(0), length: 0, content_hash: fnv1a_64_of_zeros(0) },
        DeviceEvent::Write { offset: DeviceOffsetInBytes(0), length: 4, content_hash: fnv1a_64_of_zeros(4) },
    ];
    println!("zero-length write at head: {:?}", fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(0), 4)]));
    let observed = vec![
        DeviceEvent::Write { offset: DeviceOffsetInBytes(2), length: 0, content_hash: fnv1a_64_of_zeros(0) },
        DeviceEvent::Write { offset: DeviceOffsetInBytes(0), length: 2, content_hash: fnv1a_64_of_zeros(2) },
        DeviceEvent::Write { offset: DeviceOffsetInBytes(2), length: 2, content_hash: fnv1a_64_of_zeros(2) },
    ];
    println!("zero-length ahead write: {:?}", fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(0), 4)]));
}
