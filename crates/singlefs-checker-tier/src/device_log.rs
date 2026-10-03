//! 设备侧的独立录制（C6（块层语义假设写错））：QEMU 的 blklogwrites 过滤节点按 dm-log-writes 格式把来宾发到盘上的
//! 每个写（带数据）与每个 FLUSH 记进宿主文件。这里解析它，再与被测程序自己的录制流逐项比：
//! 程序以为自己发了什么（录制器），盘上实际收到了什么（设备侧日志），两条路不共享一行代码。
//!
//! 格式（Linux `drivers/md/dm-log-writes.c` 的 `struct log_write_super` / `struct log_write_entry`，小端）：
//! 扇区 0 是系统配置（magic 8、version 8、nr_entries 8、sectorsize 4）；条目从扇区 1 起，每条一个扇区的头
//! （sector 8、nr_sectors 8、flags 8、data_len 8，补齐到一个扇区），写条目的数据紧跟其后、占 nr_sectors 个扇区。

use std::ops::Range;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::ZERO_FILL_CHUNK_BYTES;

use singlefs_harness::{fnv1a_64, fnv1a_64_of_zeros, RecordedOperationKind, RetainedOperation};

pub const WRITE_LOG_MAGIC: u64 = 0x006a_7366_7773_6872;
pub const WRITE_LOG_VERSION: u64 = 1;
pub const LOG_FLUSH_FLAG: u64 = 1 << 0;
pub const LOG_FUA_FLAG: u64 = 1 << 1;
pub const LOG_DISCARD_FLAG: u64 = 1 << 2;
pub const LOG_MARK_FLAG: u64 = 1 << 3;
pub const LOG_METADATA_FLAG: u64 = 1 << 4;
const ENTRY_HEADER_BYTES: usize = 32;

/// 设备侧看到的一件事。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceEvent {
    Write {
        offset: DeviceOffsetInBytes,
        length: u64,
        content_hash: u64,
    },
    Flush,
    Discard {
        offset: DeviceOffsetInBytes,
        length: u64,
    },
    Mark,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceLog {
    pub sector_bytes: u64,
    /// 系统配置里声明的条目数；系统配置还没写过（全 0）时是 None。
    pub declared_entries: Option<u64>,
    pub events: Vec<DeviceEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceLogError {
    BadMagic { found: u64 },
    UnknownVersion { found: u64 },
    TruncatedEntry { index: usize },
    UnknownFlags { index: usize, flags: u64 },
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 解析一份日志。系统配置全 0 时按 512 字节扇区读、条目数由「头全 0 即止」数出来；
/// 系统配置在时 magic 与版本都要对，扇区宽取它声明的。
pub fn parse_device_log(bytes: &[u8]) -> Result<DeviceLog, DeviceLogError> {
    let header_magic = if bytes.len() >= 8 {
        read_u64(bytes, 0)
    } else {
        0
    };
    let (sector_bytes, declared_entries) =
        if header_magic == 0 && bytes.len() >= 28 && bytes[..28].iter().all(|byte| *byte == 0) {
            (512u64, None)
        } else {
            if header_magic != WRITE_LOG_MAGIC {
                return Err(DeviceLogError::BadMagic {
                    found: header_magic,
                });
            }
            let version = read_u64(bytes, 8);
            if version != WRITE_LOG_VERSION {
                return Err(DeviceLogError::UnknownVersion { found: version });
            }
            let declared = read_u64(bytes, 16);
            let sector = u64::from(u32::from_le_bytes(
                bytes[24..28].try_into().expect("4 字节"),
            ));
            (sector, Some(declared))
        };
    let sector = usize::try_from(sector_bytes).expect("扇区宽");
    let mut events = Vec::new();
    let mut cursor = sector;
    loop {
        if cursor + ENTRY_HEADER_BYTES > bytes.len() {
            break;
        }
        let header = &bytes[cursor..cursor + ENTRY_HEADER_BYTES];
        if header.iter().all(|byte| *byte == 0) {
            break;
        }
        if declared_entries
            .is_some_and(|declared| u64::try_from(events.len()).expect("条目数") >= declared)
        {
            break;
        }
        let index = events.len();
        let sector_number = read_u64(header, 0);
        let sector_count = read_u64(header, 8);
        let flags = read_u64(header, 16);
        let offset = DeviceOffsetInBytes(sector_number * sector_bytes);
        let length = sector_count * sector_bytes;
        cursor += sector;
        let known =
            LOG_FLUSH_FLAG | LOG_FUA_FLAG | LOG_DISCARD_FLAG | LOG_MARK_FLAG | LOG_METADATA_FLAG;
        if flags & !known != 0 {
            return Err(DeviceLogError::UnknownFlags { index, flags });
        }
        if flags & LOG_MARK_FLAG != 0 || flags & LOG_METADATA_FLAG != 0 {
            events.push(DeviceEvent::Mark);
            continue;
        }
        if flags & LOG_DISCARD_FLAG != 0 {
            events.push(DeviceEvent::Discard { offset, length });
            continue;
        }
        if sector_count > 0 {
            let data_bytes = usize::try_from(length).expect("长度");
            if cursor + data_bytes > bytes.len() {
                return Err(DeviceLogError::TruncatedEntry { index });
            }
            events.push(DeviceEvent::Write {
                offset,
                length,
                content_hash: fnv1a_64(&bytes[cursor..cursor + data_bytes]),
            });
            cursor += data_bytes;
            if flags & LOG_FUA_FLAG != 0 {
                events.push(DeviceEvent::Flush);
            }
        }
        if flags & LOG_FLUSH_FLAG != 0 {
            events.push(DeviceEvent::Flush);
        }
    }
    Ok(DeviceLog {
        sector_bytes,
        declared_entries,
        events,
    })
}

/// 程序的信念投到一块盘上：这块盘上的每个写照录制流原样，FUA 写之后跟一个 FLUSH，
/// 这块盘自己的每道屏障是一个 FLUSH（录制器按设备记屏障，代码审阅第 1 条；`PoolWriter` 与 mkfs 的池屏障逐盘发，每块盘各记一步，
/// 别的盘的屏障不投到这块盘上）。
#[must_use]
pub fn expected_device_events(
    operations: &[RetainedOperation],
    device: DeviceIdentity,
) -> Vec<DeviceEvent> {
    let mut events = Vec::new();
    for retained in operations {
        let operation = &retained.operation;
        match operation.kind {
            RecordedOperationKind::Barrier => {
                if operation.device != device {
                    continue;
                }
                events.push(DeviceEvent::Flush);
            }
            // 整段清零：程序发的是一个动作，期望侧就摆一件事——整段一次写，内容是那么多个 0。
            // 盘上收到几条由块层怎么拆决定（后端按 `ZERO_FILL_CHUNK_BYTES` 拆，来宾内核还会按
            // `max_sectors_kb` 再拆），所以比之前先把盘上那一段折回一件事（[`fold_declared_zero_fills`]）。
            RecordedOperationKind::WriteZeroes => {
                if operation.device != device {
                    continue;
                }
                events.push(DeviceEvent::Write {
                    offset: operation.offset,
                    length: operation.length,
                    content_hash: operation.content_hash,
                });
            }
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                if operation.device != device {
                    continue;
                }
                events.push(DeviceEvent::Write {
                    offset: operation.offset,
                    length: operation.length,
                    content_hash: operation.content_hash,
                });
                if operation.kind == RecordedOperationKind::WriteForceUnitAccess {
                    events.push(DeviceEvent::Flush);
                }
            }
        }
    }
    events
}

/// 程序在这块盘上声明清零的那几段（起点与长度），按发出次序。
#[must_use]
pub fn declared_zero_fills(
    operations: &[RetainedOperation],
    device: DeviceIdentity,
) -> Vec<(DeviceOffsetInBytes, u64)> {
    operations
        .iter()
        .filter(|retained| {
            retained.operation.kind == RecordedOperationKind::WriteZeroes
                && retained.operation.device == device
        })
        .map(|retained| (retained.operation.offset, retained.operation.length))
        .collect()
}

/// 把盘上那几段清零折回一件事：程序声明清了 `[offset, offset + length)` 的地方，
/// 日志里连着的若干个写只要**恰好铺满这一段**（每一条都是全 0、互不重叠、不越过段尾；块与块之间按序，
/// 同一块之内不论次序），就换成一个整段的写事件。
/// 块的切法与后端写零时相同：从这一段的起点起每 [`ZERO_FILL_CHUNK_BYTES`] 一块，最后一块到段尾为止、可能短
/// （`singlefs_core::block_device` 的 `write_zeroes_in_chunks`）。
/// 逐条看时：一条写要么正好接在已铺满的那一截后面（可以跨块，照旧），要么整条落在已铺满那一截的末端所在的那一块里、
/// 末端之后（同一块里后面的部分先到），等前面接上再并进来。
///
/// 为什么要折：一次 `write_zeroes_at` 在程序那一侧是一个动作，落到盘上却是好几条写——
/// 后端按 [`ZERO_FILL_CHUNK_BYTES`] 拆过一次，来宾内核还会按
/// `max_sectors_kb` 再拆一次，拆成几条是块层的事，程序管不着、也不该由它决定比对过不过。
///
/// 为什么块内不论次序、块间仍按序：后端每一块是一次同步写，来宾内核把它拆成几个同时在途的请求，
/// blklogwrites 按数据盘写完的先后记日志，同一块的几部分谁先谁后由完成次序定，程序管不着；
/// 下一块要等这一块的同步写返回才发，块与块之间的次序仍是程序定的，倒过来就不折。
///
/// 为什么折了还判得动：折的条件是**铺满**——少一块、多一块、块与块之间顺序反了、块内留缝或重叠、
/// 中间夹一条不是全 0 的写、越过段尾，都折不起来，那一段就原样留着、逐项比对照样判红（「漏写环的最后 1 MiB」正是少一块）。
/// 折只在程序声明过的那几段里做，别处一个字节都不碰：页缓存那一档的回写合并落在普通写上，判别力不受影响。
///
/// 「这一条是不是全 0」按内容哈希判：`fnv1a_64` 对 `length` 个 0 有唯一取值（[`crate::fnv1a_64_of_zeros`]），
/// 日志里存着的是真字节、哈希是解析时从真字节算的。
#[must_use]
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

/// 逐项比，报第一处不一致（下标、程序以为的、盘上收到的）。
#[must_use]
pub fn first_divergence(
    expected: &[DeviceEvent],
    observed: &[DeviceEvent],
) -> Option<(usize, Option<DeviceEvent>, Option<DeviceEvent>)> {
    let longest = expected.len().max(observed.len());
    (0..longest).find_map(|index| {
        let left = expected.get(index);
        let right = observed.get(index);
        (left != right).then(|| (index, left.cloned(), right.cloned()))
    })
}

/// 比对的结论：程序的事件是不是设备侧日志的逐项前缀，前缀之后还剩几个 FLUSH。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceLogComparison {
    pub divergence: Option<(usize, Option<DeviceEvent>, Option<DeviceEvent>)>,
    /// 程序最后一件事之后、设备侧日志里还有的 FLUSH：写路之外的收尾（虚机关机时对还没 FLUSH 过的盘补的那一个）。
    pub trailing_flushes: usize,
}

/// 程序的事件必须是日志的逐项前缀；前缀之后只许是 FLUSH（一个 FLUSH 只会让已经发出的写更持久，不改前面任何一件事的次序），
/// 前缀之后出现写、或者前缀里任何一项对不上，都算第一处不一致。多少个尾部 FLUSH 可以接受由调用方判。
#[must_use]
pub fn compare_allowing_trailing_flushes(
    expected: &[DeviceEvent],
    observed: &[DeviceEvent],
) -> DeviceLogComparison {
    if observed.len() >= expected.len() && observed[..expected.len()] == *expected {
        let tail = &observed[expected.len()..];
        if let Some(position) = tail.iter().position(|event| *event != DeviceEvent::Flush) {
            let index = expected.len() + position;
            return DeviceLogComparison {
                divergence: Some((index, None, Some(observed[index].clone()))),
                trailing_flushes: position,
            };
        }
        return DeviceLogComparison {
            divergence: None,
            trailing_flushes: tail.len(),
        };
    }
    DeviceLogComparison {
        divergence: first_divergence(expected, observed),
        trailing_flushes: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_harness::RecordedOperation;

    fn log_with(entries: &[(u64, u64, u64, Vec<u8>)]) -> Vec<u8> {
        let mut bytes = vec![0u8; 512];
        bytes[..8].copy_from_slice(&WRITE_LOG_MAGIC.to_le_bytes());
        bytes[8..16].copy_from_slice(&WRITE_LOG_VERSION.to_le_bytes());
        bytes[16..24].copy_from_slice(&u64::try_from(entries.len()).expect("条目数").to_le_bytes());
        bytes[24..28].copy_from_slice(&512u32.to_le_bytes());
        for (sector, count, flags, data) in entries {
            let mut header = vec![0u8; 512];
            header[..8].copy_from_slice(&sector.to_le_bytes());
            header[8..16].copy_from_slice(&count.to_le_bytes());
            header[16..24].copy_from_slice(&flags.to_le_bytes());
            bytes.extend_from_slice(&header);
            bytes.extend_from_slice(data);
        }
        bytes.extend_from_slice(&[0u8; 1024]);
        bytes
    }

    #[test]
    fn log_of_write_flush_write_parses_into_the_same_three_events() {
        let first = vec![7u8; 1024];
        let second = vec![9u8; 512];
        let log = parse_device_log(&log_with(&[
            (8, 2, 0, first.clone()),
            (0, 0, LOG_FLUSH_FLAG, Vec::new()),
            (16, 1, 0, second.clone()),
        ]))
        .expect("解析");
        assert_eq!((log.sector_bytes, log.declared_entries), (512, Some(3)));
        assert_eq!(
            log.events,
            vec![
                DeviceEvent::Write {
                    offset: DeviceOffsetInBytes(4096),
                    length: 1024,
                    content_hash: fnv1a_64(&first)
                },
                DeviceEvent::Flush,
                DeviceEvent::Write {
                    offset: DeviceOffsetInBytes(8192),
                    length: 512,
                    content_hash: fnv1a_64(&second)
                },
            ]
        );
        let mut damaged = log_with(&[(8, 2, 0, first)]);
        damaged[..8].copy_from_slice(&1u64.to_le_bytes());
        assert_eq!(
            parse_device_log(&damaged),
            Err(DeviceLogError::BadMagic { found: 1 })
        );
        assert!(matches!(
            parse_device_log(&log_with(&[(0, 0, 1 << 9, Vec::new())])),
            Err(DeviceLogError::UnknownFlags { index: 0, .. })
        ));
    }

    #[test]
    fn missing_flush_is_reported_as_the_first_divergence() {
        let write = |device: u32, offset: u64, kind: RecordedOperationKind| RetainedOperation {
            operation: RecordedOperation {
                device: DeviceIdentity(device),
                kind,
                offset: DeviceOffsetInBytes(offset),
                length: 512,
                content_hash: 5,
            },
            contents: None,
        };
        let barrier = |device: u32| RetainedOperation {
            operation: RecordedOperation {
                device: DeviceIdentity(device),
                kind: RecordedOperationKind::Barrier,
                offset: DeviceOffsetInBytes(0),
                length: 0,
                content_hash: 0,
            },
            contents: None,
        };
        let stream = vec![
            write(0, 0, RecordedOperationKind::Write),
            write(1, 0, RecordedOperationKind::Write),
            barrier(0),
            barrier(1),
            write(0, 512, RecordedOperationKind::WriteForceUnitAccess),
        ];
        let on_device_zero = expected_device_events(&stream, DeviceIdentity(0));
        assert_eq!(
            on_device_zero.len(),
            4,
            "写、它自己那道屏障的 FLUSH、FUA 写、FUA 之后的 FLUSH"
        );
        assert_eq!(
            expected_device_events(&stream, DeviceIdentity(1)).len(),
            2,
            "另一块盘：它的写与它自己那道屏障"
        );
        // 录制器按设备记屏障：盘 1 那道没录（没发）时，盘 1 上就不期望 FLUSH，盘 0 那道不投到盘 1 上。
        let device_one_missing_its_barrier = vec![
            write(0, 0, RecordedOperationKind::Write),
            write(1, 0, RecordedOperationKind::Write),
            barrier(0),
        ];
        assert_eq!(
            expected_device_events(&device_one_missing_its_barrier, DeviceIdentity(1)),
            vec![DeviceEvent::Write {
                offset: DeviceOffsetInBytes(0),
                length: 512,
                content_hash: 5,
            }],
            "盘 1：只有它的写，没有别的盘那道屏障的 FLUSH"
        );
        assert_eq!(first_divergence(&on_device_zero, &on_device_zero), None);
        let mut observed = on_device_zero.clone();
        observed.remove(1);
        assert_eq!(
            first_divergence(&on_device_zero, &observed).map(|(index, _, _)| index),
            Some(1),
            "盘上少一个 FLUSH：下标 1 对不上"
        );
        assert_eq!(
            compare_allowing_trailing_flushes(&on_device_zero, &observed)
                .divergence
                .map(|(index, _, _)| index),
            Some(1)
        );
        let mut shutdown_flush = on_device_zero.clone();
        shutdown_flush.push(DeviceEvent::Flush);
        assert_eq!(
            compare_allowing_trailing_flushes(&on_device_zero, &shutdown_flush),
            DeviceLogComparison {
                divergence: None,
                trailing_flushes: 1
            },
            "程序最后一件事之后多一个 FLUSH：前缀对得上，报出尾部 1 个"
        );
        let mut trailing_write = shutdown_flush;
        trailing_write.push(DeviceEvent::Write {
            offset: DeviceOffsetInBytes(0),
            length: 512,
            content_hash: 1,
        });
        assert_eq!(
            compare_allowing_trailing_flushes(&on_device_zero, &trailing_write)
                .divergence
                .map(|(index, _, _)| index),
            Some(5),
            "尾部出现写就不是收尾的 FLUSH，判不一致"
        );
    }

    /// 整段清零：程序那一侧一件事，盘上那一侧拆成几条都算对得上——只要恰好铺满、每条全 0。
    /// 拆法换了（4 MiB → 512 KiB）判定不变，这正是块层怎么拆不该影响比对的那一条。
    /// 少一块、中间夹一条不是全 0 的写、越过段尾，都折不起来 ⇒ 逐项比对判红。
    #[test]
    fn zero_fill_folds_back_into_one_event_however_the_block_layer_split_it() {
        let ring_start = 1024 * 16384u64;
        let ring_bytes = 32 * 1024 * 1024u64;
        let zero_fill = RetainedOperation {
            operation: RecordedOperation {
                device: DeviceIdentity(0),
                kind: RecordedOperationKind::WriteZeroes,
                offset: DeviceOffsetInBytes(ring_start),
                length: ring_bytes,
                content_hash: fnv1a_64_of_zeros(ring_bytes),
            },
            contents: None,
        };
        let stream = vec![zero_fill];
        let expected = expected_device_events(&stream, DeviceIdentity(0));
        assert_eq!(
            expected,
            vec![DeviceEvent::Write {
                offset: DeviceOffsetInBytes(ring_start),
                length: ring_bytes,
                content_hash: fnv1a_64_of_zeros(ring_bytes),
            }],
            "程序那一侧：一次调用一件事"
        );
        let declared = declared_zero_fills(&stream, DeviceIdentity(0));
        assert_eq!(
            declared,
            vec![(DeviceOffsetInBytes(ring_start), ring_bytes)]
        );
        assert_eq!(
            declared_zero_fills(&stream, DeviceIdentity(1)),
            vec![],
            "另一块盘上没有这一段"
        );

        let split_into = |chunk: u64| -> Vec<DeviceEvent> {
            (0..ring_bytes / chunk)
                .map(|index| DeviceEvent::Write {
                    offset: DeviceOffsetInBytes(ring_start + index * chunk),
                    length: chunk,
                    content_hash: fnv1a_64_of_zeros(chunk),
                })
                .collect()
        };
        for chunk in [4 * 1024 * 1024u64, 512 * 1024, ring_bytes] {
            let observed = split_into(chunk);
            assert_eq!(
                fold_declared_zero_fills(&observed, &declared),
                expected,
                "盘上拆成 {} 条，折回来还是那一件事",
                observed.len()
            );
            assert_eq!(
                compare_allowing_trailing_flushes(
                    &expected,
                    &fold_declared_zero_fills(&observed, &declared)
                )
                .divergence,
                None
            );
        }

        // 少最后一块（「漏写环的最后 1 MiB」那一类）：铺不满 ⇒ 折不起来 ⇒ 判红。
        let mut missing_last = split_into(4 * 1024 * 1024);
        missing_last.pop();
        assert_eq!(
            fold_declared_zero_fills(&missing_last, &declared),
            missing_last,
            "铺不满就一条都不折"
        );
        assert_eq!(
            compare_allowing_trailing_flushes(
                &expected,
                &fold_declared_zero_fills(&missing_last, &declared)
            )
            .divergence
            .map(|(index, _, _)| index),
            Some(0),
            "盘上没把这一段清满：第一处就对不上"
        );

        // 段里夹了一条不是全 0 的写：折不起来 ⇒ 判红。
        let mut not_all_zero = split_into(4 * 1024 * 1024);
        not_all_zero[3] = DeviceEvent::Write {
            offset: DeviceOffsetInBytes(ring_start + 3 * 4 * 1024 * 1024),
            length: 4 * 1024 * 1024,
            content_hash: fnv1a_64_of_zeros(4 * 1024 * 1024) ^ 1,
        };
        assert_eq!(
            fold_declared_zero_fills(&not_all_zero, &declared),
            not_all_zero,
            "段里有一条不是全 0：一条都不折"
        );

        // 声明之外的写不碰：普通写照样逐项比（页缓存那一档的判别力从这里来）。
        let plain = vec![DeviceEvent::Write {
            offset: DeviceOffsetInBytes(0),
            length: 4096,
            content_hash: 0x1234,
        }];
        assert_eq!(fold_declared_zero_fills(&plain, &declared), plain);
    }

    /// 日志里的一条全 0 写。
    fn zero_write(offset: u64, length: u64) -> DeviceEvent {
        DeviceEvent::Write {
            offset: DeviceOffsetInBytes(offset),
            length,
            content_hash: fnv1a_64_of_zeros(length),
        }
    }

    /// 来宾内核把后端的一块写零拆成两个同时在途的请求时前一个的长度：2026-10-01 虚机档实测拆成 2032 + 6160 扇区。
    const FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES: u64 = 2032 * 512;

    /// 一块拆成前后两部分，交回 (前, 后)。
    fn chunk_split_in_two(chunk_start: u64, chunk_length: u64) -> (DeviceEvent, DeviceEvent) {
        (
            zero_write(chunk_start, FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES),
            zero_write(
                chunk_start + FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES,
                chunk_length - FRONT_PART_OF_A_GUEST_SPLIT_CHUNK_BYTES,
            ),
        )
    }

    /// blklogwrites 按数据盘写完的先后记日志，同一块的后一部分可能先记：块内不论次序，整段照样折成一件。
    #[test]
    fn zero_fill_whose_two_parts_of_one_chunk_arrive_reversed_folds_into_one_event() {
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let (first_chunk_front, first_chunk_back) =
            chunk_split_in_two(fill_start, ZERO_FILL_CHUNK_BYTES);
        let (second_chunk_front, second_chunk_back) =
            chunk_split_in_two(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES);
        let observed = vec![
            first_chunk_back,
            first_chunk_front,
            second_chunk_front,
            second_chunk_back,
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            vec![zero_write(fill_start, fill_length)],
            "第一块两部分倒序、第二块按序：整段折成一件"
        );
    }

    /// 第二块整块排在第一块前面：下一块要等这一块的同步写返回才发，块与块之间倒过来不是块层的事，一条都不折。
    #[test]
    fn zero_fill_whose_second_chunk_arrives_before_the_first_does_not_fold() {
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let (first_chunk_front, first_chunk_back) =
            chunk_split_in_two(fill_start, ZERO_FILL_CHUNK_BYTES);
        let (second_chunk_front, second_chunk_back) =
            chunk_split_in_two(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES);
        let observed = vec![
            second_chunk_front,
            second_chunk_back,
            first_chunk_front,
            first_chunk_back,
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "第二块排在第一块前面：一条都不折"
        );
    }

    /// 第一块还没铺满时第二块的一部分先到（两块的条目交错）：块与块之间仍按序，一条都不折。
    #[test]
    fn zero_fill_with_a_piece_of_the_next_chunk_arriving_before_the_current_chunk_is_full_does_not_fold(
    ) {
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let (first_chunk_front, first_chunk_back) =
            chunk_split_in_two(fill_start, ZERO_FILL_CHUNK_BYTES);
        let (second_chunk_front, second_chunk_back) =
            chunk_split_in_two(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES);
        let observed = vec![
            first_chunk_back,
            second_chunk_front,
            first_chunk_front,
            second_chunk_back,
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "第二块的前一部分插在第一块两部分之间：一条都不折"
        );
    }

    /// 第一块里留了一道缝（[1/4, 1/2) 块没写），后一部分先到、第二块照样铺满：铺不满，一条都不折。
    #[test]
    fn zero_fill_chunk_with_a_gap_does_not_fold() {
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
        let observed = vec![
            zero_write(fill_start, quarter_chunk),
            zero_write(fill_start + 2 * quarter_chunk, 2 * quarter_chunk),
            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "第一块中间缺一截：一条都不折"
        );
    }

    /// 第一块里两条重叠（后面的 [1/4, 1) 块先到，[0, 1/2) 块后到），字节加起来比一块多：不是恰好铺满，一条都不折。
    #[test]
    fn zero_fill_chunk_whose_pieces_overlap_does_not_fold() {
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
        let observed = vec![
            zero_write(fill_start + quarter_chunk, 3 * quarter_chunk),
            zero_write(fill_start, 2 * quarter_chunk),
            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "第一块里两条重叠：一条都不折"
        );
    }

    /// 段长 2.5 块、起点不在块大小的整数倍上（mkfs 那几段 32 KiB 的清零起点就不在 4 MiB 边界上）：
    /// 最后一块短（半块），它的两部分倒序照样折成一件。最后一块是 [6.75, 7.25) 个块大小，跨过设备上第 7 个块大小那道边界：
    /// 块要是从设备偏移 0 起算而不是从段的起点起算，这两部分就分在两块里、折不起来。
    #[test]
    fn zero_fill_whose_short_last_chunk_arrives_reversed_folds_into_one_event() {
        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES + 3 * quarter_chunk;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES + 2 * quarter_chunk;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let last_chunk_start = fill_start + 2 * ZERO_FILL_CHUNK_BYTES;
        let observed = vec![
            zero_write(fill_start, ZERO_FILL_CHUNK_BYTES),
            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
            zero_write(last_chunk_start + quarter_chunk, quarter_chunk),
            zero_write(last_chunk_start, quarter_chunk),
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            vec![zero_write(fill_start, fill_length)],
            "最后半块的两部分倒序：整段折成一件"
        );
    }

    /// 段长 2.5 块：最后一块只到段尾。先到的那一条落在最后一块里、却伸出段尾 1/4 块：越过段尾，一条都不折。
    #[test]
    fn zero_fill_whose_short_last_chunk_gets_a_piece_running_past_the_end_does_not_fold() {
        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES + 2 * quarter_chunk;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let last_chunk_start = fill_start + 2 * ZERO_FILL_CHUNK_BYTES;
        let observed = vec![
            zero_write(fill_start, ZERO_FILL_CHUNK_BYTES),
            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
            zero_write(last_chunk_start + quarter_chunk, 2 * quarter_chunk),
            zero_write(last_chunk_start, quarter_chunk),
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "最后一块里先到的那一条伸出段尾：一条都不折"
        );
    }

    /// 声明了一段长度 0 的清零（LR7，`research/prompts/m3-leftover-code-r1-main-verification.md` 第三节）：盘上没有条目可折，
    /// 铺满的判定交回「铺不满」，整份日志原样留着。判别力：去掉长度 0 的守卫时铺满的判定交回「用掉 0 条」，
    /// `fold_declared_zero_fills` 原地不动、一直往结果里推（正推腿 `k4/mut_noguard` 在 1G 的内存包装里被杀）；
    /// 先判铺满那一步，这一条红在第一句断言上，不跑到不终止的那一步。
    #[test]
    fn a_declared_zero_fill_of_length_zero_tiles_no_entry_and_leaves_the_log_unchanged() {
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let declared = vec![(DeviceOffsetInBytes(fill_start), 0)];
        let observed = vec![zero_write(fill_start, ZERO_FILL_CHUNK_BYTES)];
        assert_eq!(
            entries_tiling_one_zero_fill(&observed, 0, DeviceOffsetInBytes(fill_start), 0),
            None,
            "长度 0 的清零在盘上没有条目可折：铺不满，不交回「用掉 0 条」"
        );
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "声明长度 0 的那一段一条都不折"
        );
    }

    /// 段长 2.5 块：前两块按序到，最后一条接在已铺满的末端上、却伸出段尾 1/4 块（LR7：接续写越过段尾）。越过段尾，一条都不折。
    /// 判别力：接在末端的那一条不查段尾时它被并进来，已铺满的末端越过段尾，函数末尾「已铺满的末端停在段尾」那一句断言 panic。
    #[test]
    fn zero_fill_whose_piece_continuing_at_the_covered_end_runs_past_the_end_does_not_fold() {
        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES + 2 * quarter_chunk;
        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
        let last_chunk_start = fill_start + 2 * ZERO_FILL_CHUNK_BYTES;
        let observed = vec![
            zero_write(fill_start, ZERO_FILL_CHUNK_BYTES),
            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
            zero_write(last_chunk_start, 3 * quarter_chunk),
        ];
        assert_eq!(
            fold_declared_zero_fills(&observed, &declared),
            observed,
            "最后一条接在末端、伸出段尾：一条都不折"
        );
    }
}
