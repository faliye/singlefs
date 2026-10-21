//! 设备侧的独立录制（C6（块层语义假设写错））：QEMU 的 blklogwrites 过滤节点按 dm-log-writes 格式把来宾发到盘上的
//! 每个写（带数据）与每个 FLUSH 记进宿主文件。这里解析它，再与被测程序自己的录制流逐项比：
//! 程序以为自己发了什么（录制器），盘上实际收到了什么（设备侧日志），两条路不共享一行代码。
//!
//! 格式（Linux `drivers/md/dm-log-writes.c` 的 `struct log_write_super` / `struct log_write_entry`，小端）：
//! 扇区 0 是系统配置（magic 8、version 8、nr_entries 8、sectorsize 4）；条目从扇区 1 起，每条一个扇区的头
//! （sector 8、nr_sectors 8、flags 8、data_len 8，补齐到一个扇区），写条目的数据紧跟其后、占 nr_sectors 个扇区。

use std::collections::BTreeSet;
use std::ops::Range;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::ZERO_FILL_CHUNK_BYTES;

use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{
    fnv1a_64, fnv1a_64_of_zeros, RecordedOperation, RecordedOperationKind, RetainedOperation,
};

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

/// 程序的信念投到一块盘上：这块盘上的每个写照录制流原样，FUA 写之后跟一个 FLUSH，每次 discard 是一条同偏移同长度的 Discard，
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
            // discard：录制流只记交回「交给了底层」的那几次（`RecordedOperationKind::Discard`），盘上该恰好收到同一段、同一个次序的一条
            // Discard 条目；漏了、多了、段不同、次序不同，逐项比对都判红。不支持 discard 的后端交回「不支持」，流里没有这一步、期望侧也不摆。
            RecordedOperationKind::Discard => {
                if operation.device != device {
                    continue;
                }
                events.push(DeviceEvent::Discard {
                    offset: operation.offset,
                    length: operation.length,
                });
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

/// 一次发布返回之后、下一次发布返回之前，这块盘上可以发 discard 的字节区间（判决 `research/prompts/m4-gc-r1-main-verification.md`
/// 第三节 G5：发布返回之后，「可再分配」谓词为真的槽按 64 槽对齐的整段发）。区间由判器自己从录制流里的发布独立算
/// （[`discardable_ranges_after_each_publish_found_in_the_stream`]，主 agent 2026-10-13 定：不取实现那一侧的可再分配谓词）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardableRangesAfterAPublish {
    /// 程序录制流里这次发布返回时已经录下的步数（`SharedStream::operation_count`）：前面这几步投到这块盘上的事件里，
    /// 有这次发布在这块盘上的最后一道屏障。
    pub operations_recorded_when_the_publish_returned: usize,
    /// 这块盘上可以发 discard 的字节区间（半开，可以有几段）。
    pub discardable_ranges: Vec<Range<u64>>,
}

/// 一条盘上事件里的 discard 落错了地方（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「设备日志判器判每个 Discard
/// 落在可发区间里且在那次发布最后一道屏障之后」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MisplacedDiscard {
    /// 罩住的范围在它之前最近返回的那次发布之后不可发，要到之后某次发布返回才可发：discard 提前到了那次发布的最后一道屏障
    /// （连同它的根 FUA 写）之前。
    BeforeTheLastBarrierOfThePublishThatMakesItDiscardable {
        event_index: usize,
        offset: DeviceOffsetInBytes,
        length: u64,
    },
    /// 罩住的范围在它之前最近返回的那次发布之后不可发，之后也没有哪次发布让它可发：落在不可发的槽上。
    OutsideTheDiscardableRanges {
        event_index: usize,
        offset: DeviceOffsetInBytes,
        length: u64,
    },
}

/// `[start, end)` 是不是整段落在 `ranges` 的并里。
fn is_covered_by_the_union_of(ranges: &[Range<u64>], start: u64, end: u64) -> bool {
    let mut sorted_ranges: Vec<&Range<u64>> = ranges.iter().collect();
    sorted_ranges.sort_by_key(|range| range.start);
    let mut covered_up_to = start;
    // 按起点从小到大走一遍：`covered_up_to` 只增不减，接不上就停。
    for range in sorted_ranges {
        if covered_up_to >= end {
            break;
        }
        if range.start > covered_up_to {
            return false;
        }
        covered_up_to = covered_up_to.max(range.end);
    }
    covered_up_to >= end
}

/// 录制流里每次发布的返回点（发布返回时流里已经录下的步数，按先后、不重复）：每一次根槽写（`StepKind::RootRecordFua`）之后，跳过紧接着的
/// 系统配置槽写（发布的系统配置轮换）与屏障，停在下一步不是这两种的地方（流尾也算）。只看录制流里每一步落在哪一类位置上（几何是这个池的，
/// `segments::FixedGeometry`），不读实现那一侧的发布记账。
#[must_use]
pub fn publish_return_points_of_the_stream(
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
) -> Vec<usize> {
    let mut return_points: Vec<usize> = Vec::new();
    for (root_record_write, retained) in operations.iter().enumerate() {
        if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
            continue;
        }
        let mut returned_at = root_record_write + 1;
        // 只往后走、至多走到流尾：每一步要么停、要么下标加一
        while operations.get(returned_at).is_some_and(|following| {
            matches!(
                geometry.classify(&following.operation),
                StepKind::SystemConfigurationSlot | StepKind::Barrier
            )
        }) {
            returned_at += 1;
        }
        if return_points.last() != Some(&returned_at) {
            return_points.push(returned_at);
        }
    }
    return_points
}

/// 池级 checker 在这张镜像上判违例的不变量名。
fn invariants_violated_on(image: &MemoryPool) -> BTreeSet<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(_) => Some(invariant),
            singlefs_checker::image::InvariantVerdict::Holds
            | singlefs_checker::image::InvariantVerdict::NotApplicable(_)
            | singlefs_checker::image::InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 判器自己算的可发区间（主 agent 2026-10-13 定：判器从录制流里的发布独立算，不取实现那一侧的可再分配谓词；checker 与实现不共用判法）：
/// 每次发布返回的那一刻（[`publish_return_points_of_the_stream`]），`base` 加上流里到那一步为止除 discard 之外的每一步就是那一刻的盘；流里这块盘上发过的每一段
/// discard（候选）各施加到这张盘的一份拷贝上，池级 checker 在拷贝上判违例的不变量不多于没施加那一份的，这一段在那次发布之后可发。
/// 流里已经发过的 discard（两块盘的都算）不施加进那一刻的盘：它们正是要判的那几条。施加了的话，一条落错地方的 discard 之后的每个返回点上，
/// 候选抹的都是已经抹掉的内容，违例集合不会再变大，那一段就被算成之后可发，判成「提前到了使它可发的那次发布之前」，
/// 「落在不可发的槽上」在它之后还有发布时永远报不出来（生命周期闭环流「挪到实例表那一段」的必红世界就是这样）。不施加 discard 的盘是设备不清零那一种结局，
/// 与清零一样合法。
/// 判据是「这一刻抹掉它，恢复能落到、能退到的版本里没有一个单元读不对」（判决 G5 的 I-7.4 / I-2.1 照判，判法是池级 checker 的走读，
/// 不是实现分配器的回收谓词）。候选只取流里真发过的那几段（判器只回答「发了的这几段在哪几次发布之后可发」），一段都没发过时不判池级 checker。
/// 交回每次发布一项，按返回的先后。
///
/// 代价：返回点数 ×（候选段数 + 1）次池级 checker；用在崩溃流的录制流上（几十次发布、几段 discard），不在崩溃状态的全域上逐状态跑。
#[must_use]
pub fn discardable_ranges_after_each_publish_found_in_the_stream(
    base: &MemoryPool,
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
    device: DeviceIdentity,
) -> Vec<DiscardableRangesAfterAPublish> {
    let candidate_discards: BTreeSet<(u64, u64)> = operations
        .iter()
        .filter(|retained| {
            retained.operation.kind == RecordedOperationKind::Discard
                && retained.operation.device == device
                && retained.operation.length > 0
        })
        .map(|retained| (retained.operation.offset.0, retained.operation.length))
        .collect();
    let mut image_when_the_publish_returned = base.clone();
    let mut operations_applied = 0usize;
    let mut publishes = Vec::new();
    for returned_at in publish_return_points_of_the_stream(operations, geometry) {
        for retained in &operations[operations_applied..returned_at] {
            if retained.operation.kind != RecordedOperationKind::Discard {
                image_when_the_publish_returned.apply(std::slice::from_ref(retained));
            }
        }
        operations_applied = returned_at;
        let mut discardable_ranges = Vec::new();
        if !candidate_discards.is_empty() {
            let violated_without_the_discard =
                invariants_violated_on(&image_when_the_publish_returned);
            for (offset, length) in &candidate_discards {
                let mut discarded = image_when_the_publish_returned.clone();
                discarded.apply(&[RetainedOperation {
                    operation: RecordedOperation {
                        device,
                        kind: RecordedOperationKind::Discard,
                        offset: DeviceOffsetInBytes(*offset),
                        length: *length,
                        content_hash: fnv1a_64_of_zeros(*length),
                    },
                    contents: None,
                }]);
                if invariants_violated_on(&discarded).is_subset(&violated_without_the_discard) {
                    discardable_ranges.push(*offset..*offset + *length);
                }
            }
        }
        publishes.push(DiscardableRangesAfterAPublish {
            operations_recorded_when_the_publish_returned: returned_at,
            discardable_ranges,
        });
    }
    publishes
}

/// 判这块盘上每条 Discard 落没落在可发区间里、在不在使它可发的那次发布的最后一道屏障之后（判决 G5）：可发区间由判器自己从录制流里的发布算
/// （[`discardable_ranges_after_each_publish_found_in_the_stream`]），再照 [`judge_discards_against_discardable_ranges`] 判。交回判过的 discard 条数。
///
/// `base` 是 `operations` 第一步之前的盘（崩溃流里是 mkfs 之后那一张，`operations` 是 mkfs 之后的录制流）；`events` 照
/// [`judge_discards_against_discardable_ranges`] 那一条的要求给。
///
/// # Errors
/// 有一条 discard 落错了地方（[`MisplacedDiscard`]）。
pub fn judge_discards_against_the_publishes(
    base: &MemoryPool,
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
    device: DeviceIdentity,
    events: &[DeviceEvent],
) -> Result<usize, MisplacedDiscard> {
    let publishes = discardable_ranges_after_each_publish_found_in_the_stream(
        base, operations, geometry, device,
    );
    judge_discards_against_discardable_ranges(operations, device, events, &publishes)
}

/// 判这块盘上每条 Discard：它之前最近返回的那次发布（返回点投到这块盘上的事件下标不大于它的下标），那次发布之后的可发区间要罩住它；
/// 罩不住的，按之后有没有哪次发布让它可发，分成「提前到了那次发布的最后一道屏障之前」与「落在不可发的槽上」。交回判过的 discard 条数。
///
/// `events` 是这块盘上的事件（设备侧日志折过整段清零之后，或程序投到这块盘上的事件）；每次发布的返回点按程序录制流的前缀投到这块盘上
/// 数事件下标（[`expected_device_events`]），所以只在盘上日志与程序那一侧逐项对得上时有意义——调用方先过
/// [`compare_allowing_trailing_flushes`]，那一道判红时不再看这一道。`publishes` 按返回的先后给。
///
/// # Panics
/// `publishes` 的返回点不是按先后给的，或越过了 `operations` 的长度。
fn judge_discards_against_discardable_ranges(
    operations: &[RetainedOperation],
    device: DeviceIdentity,
    events: &[DeviceEvent],
    publishes: &[DiscardableRangesAfterAPublish],
) -> Result<usize, MisplacedDiscard> {
    assert!(
        publishes
            .windows(2)
            .all(|pair| pair[0].operations_recorded_when_the_publish_returned
                <= pair[1].operations_recorded_when_the_publish_returned),
        "发布按返回的先后给"
    );
    let events_before_each_return: Vec<usize> = publishes
        .iter()
        .map(|publish| {
            expected_device_events(
                &operations[..publish.operations_recorded_when_the_publish_returned],
                device,
            )
            .len()
        })
        .collect();
    let mut judged_discards = 0usize;
    for (event_index, event) in events.iter().enumerate() {
        let DeviceEvent::Discard { offset, length } = event else {
            continue;
        };
        let start = offset.0;
        let end = offset.0 + length;
        let publishes_returned_before_it = events_before_each_return
            .iter()
            .take_while(|events_before_the_return| **events_before_the_return <= event_index)
            .count();
        let is_discardable_now =
            publishes_returned_before_it
                .checked_sub(1)
                .is_some_and(|latest_returned| {
                    is_covered_by_the_union_of(
                        &publishes[latest_returned].discardable_ranges,
                        start,
                        end,
                    )
                });
        if is_discardable_now {
            judged_discards += 1;
            continue;
        }
        let a_later_publish_makes_it_discardable = publishes[publishes_returned_before_it..]
            .iter()
            .any(|later| is_covered_by_the_union_of(&later.discardable_ranges, start, end));
        return Err(if a_later_publish_makes_it_discardable {
            MisplacedDiscard::BeforeTheLastBarrierOfThePublishThatMakesItDiscardable {
                event_index,
                offset: *offset,
                length: *length,
            }
        } else {
            MisplacedDiscard::OutsideTheDiscardableRanges {
                event_index,
                offset: *offset,
                length: *length,
            }
        });
    }
    Ok(judged_discards)
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

    /// 录制流里一次 discard 投成盘上同一段、同一个次序的一条 Discard：一次写、一次 discard、一道屏障的流，
    /// 盘上日志照收判绿；日志里缺那条 discard、段不同、与写换了次序，都在第一处不同的下标判红。
    #[test]
    fn a_discard_in_the_recorded_stream_must_reach_the_device_log_as_the_same_range_in_the_same_order(
    ) {
        let data = vec![7u8; 512];
        let retained = |kind, offset, length, content_hash, contents| RetainedOperation {
            operation: RecordedOperation {
                device: DeviceIdentity(0),
                kind,
                offset: DeviceOffsetInBytes(offset),
                length,
                content_hash,
            },
            contents,
        };
        let stream = vec![
            retained(
                RecordedOperationKind::Write,
                4096,
                512,
                fnv1a_64(&data),
                Some(data.clone()),
            ),
            retained(
                RecordedOperationKind::Discard,
                8192,
                16384,
                fnv1a_64_of_zeros(16384),
                None,
            ),
            retained(RecordedOperationKind::Barrier, 0, 0, 0, None),
        ];
        let expected = expected_device_events(&stream, DeviceIdentity(0));
        assert_eq!(
            expected,
            vec![
                DeviceEvent::Write {
                    offset: DeviceOffsetInBytes(4096),
                    length: 512,
                    content_hash: fnv1a_64(&data),
                },
                DeviceEvent::Discard {
                    offset: DeviceOffsetInBytes(8192),
                    length: 16384,
                },
                DeviceEvent::Flush,
            ],
            "程序那一侧：discard 投成一条 Discard，排在写与屏障之间"
        );
        assert_eq!(
            expected_device_events(&stream, DeviceIdentity(1)),
            vec![],
            "别的盘上的 discard 不投到这块盘上"
        );
        let divergence_against = |entries: &[(u64, u64, u64, Vec<u8>)]| {
            let log = parse_device_log(&log_with(entries)).expect("解析");
            compare_allowing_trailing_flushes(&expected, &log.events)
                .divergence
                .map(|(index, _, _)| index)
        };
        let write_entry = (8, 1, 0, data.clone());
        let flush_entry = (0, 0, LOG_FLUSH_FLAG, Vec::new());
        assert_eq!(
            divergence_against(&[
                write_entry.clone(),
                (16, 32, LOG_DISCARD_FLAG, Vec::new()),
                flush_entry.clone(),
            ]),
            None,
            "盘上收到同一段的 discard：对得上"
        );
        assert_eq!(
            divergence_against(&[write_entry.clone(), flush_entry.clone()]),
            Some(1),
            "程序发了 discard、盘上没收到：判红在那一项"
        );
        assert_eq!(
            divergence_against(&[
                write_entry.clone(),
                (16, 16, LOG_DISCARD_FLAG, Vec::new()),
                flush_entry.clone(),
            ]),
            Some(1),
            "盘上收到的 discard 只有一半长：判红"
        );
        assert_eq!(
            divergence_against(&[
                (16, 32, LOG_DISCARD_FLAG, Vec::new()),
                write_entry,
                flush_entry,
            ]),
            Some(0),
            "discard 跑到写前面：次序不同判红"
        );
    }

    /// 单元区里两段 64 槽对齐的整段（784 MiB 起），各 1 MiB。
    const SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH: u64 = 784 << 20;
    const SEGMENT_WRITTEN_BY_THE_SECOND_PUBLISH: u64 = (784 << 20) + (1 << 20);
    const SEGMENT_BYTES: u64 = 1 << 20;

    /// 盘 0 上的一步。
    fn step_on_device_zero(
        kind: RecordedOperationKind,
        offset: u64,
        length: u64,
    ) -> RetainedOperation {
        RetainedOperation {
            operation: RecordedOperation {
                device: DeviceIdentity(0),
                kind,
                offset: DeviceOffsetInBytes(offset),
                length,
                content_hash: offset ^ 0x5A5A,
            },
            contents: None,
        }
    }

    /// 一次发布在盘 0 上的样子：单元写、屏障、根槽 FUA 写、系统配置槽写、屏障（最后那道屏障之后发布返回）。
    fn publish_on_device_zero(unit_offset: u64, root_offset: u64) -> Vec<RetainedOperation> {
        vec![
            step_on_device_zero(RecordedOperationKind::Write, unit_offset, 16384),
            step_on_device_zero(RecordedOperationKind::Barrier, 0, 0),
            step_on_device_zero(
                RecordedOperationKind::WriteForceUnitAccess,
                root_offset,
                512,
            ),
            step_on_device_zero(RecordedOperationKind::Write, 0, 4096),
            step_on_device_zero(RecordedOperationKind::Barrier, 0, 0),
        ]
    }

    /// 两次发布：先一次什么都不让发；第二次释放了 `SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH` 那一段、返回之后那一段可发。
    /// 交回录制流（discard 插在 `discard_position` 那一步之前，罩住 `discarded_segment` 那一段）与两次发布的可发区间。
    fn two_publishes_with_one_discard(
        discard_position: usize,
        discarded_segment: u64,
    ) -> (Vec<RetainedOperation>, Vec<DiscardableRangesAfterAPublish>) {
        let mut stream = publish_on_device_zero(SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH, 1 << 20);
        stream.extend(publish_on_device_zero(
            SEGMENT_WRITTEN_BY_THE_SECOND_PUBLISH,
            (1 << 20) + 512,
        ));
        stream.insert(
            discard_position,
            step_on_device_zero(
                RecordedOperationKind::Discard,
                discarded_segment,
                SEGMENT_BYTES,
            ),
        );
        let discard_comes_before = |returned_at: usize| {
            if discard_position < returned_at {
                returned_at + 1
            } else {
                returned_at
            }
        };
        let publishes = vec![
            DiscardableRangesAfterAPublish {
                operations_recorded_when_the_publish_returned: discard_comes_before(5),
                discardable_ranges: vec![],
            },
            DiscardableRangesAfterAPublish {
                operations_recorded_when_the_publish_returned: discard_comes_before(10),
                discardable_ranges: std::iter::once(
                    SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH
                        ..SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH + SEGMENT_BYTES,
                )
                .collect(),
            },
        ];
        (stream, publishes)
    }

    /// 第二次发布返回之后对它释放的那一段发 discard：落在可发区间里、在那次发布最后一道屏障之后，判绿、判过一条。
    #[test]
    fn a_discard_of_a_released_segment_after_the_publish_returned_is_green() {
        let (stream, publishes) =
            two_publishes_with_one_discard(10, SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH);
        let events = expected_device_events(&stream, DeviceIdentity(0));
        assert_eq!(
            judge_discards_against_discardable_ranges(
                &stream,
                DeviceIdentity(0),
                &events,
                &publishes
            ),
            Ok(1)
        );
    }

    /// 必红世界（判决 m4-gc-r1 G5）：同一次 discard 提前到第二次发布的根 FUA 写之前——那时最近返回的是第一次发布，那一段还不可发，
    /// 要到第二次发布返回才可发，判红在那条 Discard 上。
    #[test]
    fn a_discard_moved_before_the_root_record_write_of_the_publish_that_releases_it_is_red() {
        let (stream, publishes) =
            two_publishes_with_one_discard(7, SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH);
        let events = expected_device_events(&stream, DeviceIdentity(0));
        assert_eq!(
            events[8],
            DeviceEvent::Discard {
                offset: DeviceOffsetInBytes(SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH),
                length: SEGMENT_BYTES,
            },
            "盘上：第二次发布的单元写与屏障之后、根 FUA 写之前就收到 discard"
        );
        assert_eq!(
            judge_discards_against_discardable_ranges(
                &stream,
                DeviceIdentity(0),
                &events,
                &publishes
            ),
            Err(
                MisplacedDiscard::BeforeTheLastBarrierOfThePublishThatMakesItDiscardable {
                    event_index: 8,
                    offset: DeviceOffsetInBytes(SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH),
                    length: SEGMENT_BYTES,
                }
            )
        );
    }

    /// 必红世界（判决 m4-gc-r1 G5）：第二次发布返回之后对它刚写了单元的那一段发 discard——那一段不在任何一次发布的可发区间里，判红。
    #[test]
    fn a_discard_on_a_segment_that_is_not_discardable_is_red() {
        let (stream, publishes) =
            two_publishes_with_one_discard(10, SEGMENT_WRITTEN_BY_THE_SECOND_PUBLISH);
        let events = expected_device_events(&stream, DeviceIdentity(0));
        assert_eq!(
            judge_discards_against_discardable_ranges(
                &stream,
                DeviceIdentity(0),
                &events,
                &publishes
            ),
            Err(MisplacedDiscard::OutsideTheDiscardableRanges {
                event_index: 12,
                offset: DeviceOffsetInBytes(SEGMENT_WRITTEN_BY_THE_SECOND_PUBLISH),
                length: SEGMENT_BYTES,
            })
        );
    }

    /// 可发区间按并判：一次 discard 跨两段相邻的可发区间算罩住，跨出去一个字节不算。
    #[test]
    fn a_discard_spanning_two_adjacent_discardable_ranges_is_covered_and_one_byte_past_them_is_not()
    {
        let ranges = vec![2048..4096, 0..2048];
        assert!(is_covered_by_the_union_of(&ranges, 0, 4096));
        assert!(!is_covered_by_the_union_of(&ranges, 0, 4097));
        assert!(!is_covered_by_the_union_of(&[0..1024, 2048..4096], 0, 4096));
    }

    /// 上面那几条手摆的流用的几何：系统配置两槽从偏移 0 起、根环从 1 MiB 起、单元区在 journal 环之后（崩溃流的池同一套）。
    fn geometry_of_the_hand_written_streams() -> FixedGeometry {
        FixedGeometry {
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region:
                singlefs_core::root_ring::RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        }
    }

    /// 判器自己从录制流里找发布的返回点：每次根槽写之后跳过系统配置轮换与屏障，停在下一步——与手摆的两次发布写明的返回点逐个相同；
    /// discard 挪到第二次发布的根槽写之前时，第二次发布的返回点跟着往后挪一步（discard 不在根槽写与轮换之间，不被跳过）。
    #[test]
    fn publish_return_points_are_found_after_the_rotation_that_follows_each_root_record_write() {
        for discard_position in [10, 7] {
            let (stream, publishes) = two_publishes_with_one_discard(
                discard_position,
                SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH,
            );
            let written: Vec<usize> = publishes
                .iter()
                .map(|publish| publish.operations_recorded_when_the_publish_returned)
                .collect();
            assert_eq!(
                publish_return_points_of_the_stream(
                    &stream,
                    &geometry_of_the_hand_written_streams()
                ),
                written,
                "discard 插在第 {discard_position} 步之前：返回点在根槽写、轮换与屏障之后"
            );
        }
    }

    /// 判器自己算可发区间的那一半在没有 discard 的流上不判池级 checker：每次发布交回一项、区间都空（候选只取流里真发过的段）。
    #[test]
    fn a_stream_without_any_discard_has_no_discardable_range_after_any_publish() {
        let mut stream = publish_on_device_zero(SEGMENT_RELEASED_BY_THE_SECOND_PUBLISH, 1 << 20);
        stream.extend(publish_on_device_zero(
            SEGMENT_WRITTEN_BY_THE_SECOND_PUBLISH,
            (1 << 20) + 512,
        ));
        let empty_pool = MemoryPool::with_devices(&[DeviceIdentity(0)], 4 << 30);
        let publishes = discardable_ranges_after_each_publish_found_in_the_stream(
            &empty_pool,
            &stream,
            &geometry_of_the_hand_written_streams(),
            DeviceIdentity(0),
        );
        assert_eq!(
            publishes,
            vec![
                DiscardableRangesAfterAPublish {
                    operations_recorded_when_the_publish_returned: 5,
                    discardable_ranges: vec![],
                },
                DiscardableRangesAfterAPublish {
                    operations_recorded_when_the_publish_returned: 10,
                    discardable_ranges: vec![],
                },
            ]
        );
    }
}
