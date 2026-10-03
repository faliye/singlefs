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

