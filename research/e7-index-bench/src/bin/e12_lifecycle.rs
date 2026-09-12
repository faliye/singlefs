//! E12：生命周期判定的两条路，量设备 I/O。
//!
//! 判据与失败条款见 kb/experiments.md E12。**必须真的碰设备**——
//! 纯内存微基准会把这题测成「没有差别」，而差别恰恰全在 I/O 上。

use e7_index_bench::{page_of, Emitter, IoCounters, Lru};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

/// 块层自己数的 I/O。**这是校验用的独立路径**——内核数的，与本程序的计数器
/// 不共享任何代码、任何采样、任何工具（`singlefs-ai-sop/rules/evidence-discipline.md`）。
/// 两者对不上，说明本程序的模型是错的：O_DIRECT 没真的绕过缓存、
/// 内核合并了请求、或者有预读。
#[derive(Debug, Default, Clone, Copy)]
struct BlockLayerStatistics {
    read_requests_completed: u64,
    read_sectors: u64,
    write_requests_completed: u64,
    write_sectors: u64,
}

fn read_block_layer_statistics(device_path: &str) -> Option<BlockLayerStatistics> {
    let name = device_path.rsplit('/').next()?;
    let statistics_text = std::fs::read_to_string(format!("/sys/block/{name}/stat")).ok()?;
    let fields: Vec<u64> = statistics_text.split_whitespace().filter_map(|field_text| field_text.parse().ok()).collect();
    // 字段序：读完成数 读合并 读扇区 读耗时 写完成数 写合并 写扇区 写耗时 ...
    if fields.len() < 8 {
        return None;
    }
    Some(BlockLayerStatistics { read_requests_completed: fields[0], read_sectors: fields[2], write_requests_completed: fields[4], write_sectors: fields[6] })
}

fn block_layer_delta(before: Option<BlockLayerStatistics>, after: Option<BlockLayerStatistics>) -> Option<BlockLayerStatistics> {
    let (before, after) = (before?, after?);
    Some(BlockLayerStatistics {
        read_requests_completed: after.read_requests_completed.saturating_sub(before.read_requests_completed),
        read_sectors: after.read_sectors.saturating_sub(before.read_sectors),
        write_requests_completed: after.write_requests_completed.saturating_sub(before.write_requests_completed),
        write_sectors: after.write_sectors.saturating_sub(before.write_sectors),
    })
}

/// 把「我数的」与「块层数的」并排打出来。读不到块层读数就明说读不到，
/// **绝不静默当成 0**（`rules/test-discipline.md`：读不到 ≠ 读到 0）。
fn format_block_layer_delta(delta: Option<BlockLayerStatistics>) -> String {
    match delta {
        Some(statistics) => format!(
            "blk_reads={} blk_writes={} blk_read_kib={} blk_write_kib={}",
            statistics.read_requests_completed, statistics.write_requests_completed, statistics.read_sectors / 2, statistics.write_sectors / 2
        ),
        None => "blk_reads=NA blk_writes=NA blk_read_kib=NA blk_write_kib=NA".to_string(),
    }
}

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名
const PAGE_BYTES: usize = 4096;
const COUNTERS_PER_PAGE: u64 = (PAGE_BYTES / 8) as u64; // 每个计数器 8 字节
const DEADLIST_RECORD_BYTES: usize = 16; // deadlist 一条定长记录
const DEADLIST_BUFFER_BYTES: usize = 1024 * 1024;

struct AlignedBuffer {
    pointer: *mut u8,
    length_in_bytes: usize,
    layout: Layout,
}
impl AlignedBuffer {
    fn new(length_in_bytes: usize) -> Self {
        let layout = Layout::from_size_align(length_in_bytes, PAGE_BYTES).expect("对齐参数非法");
        let allocation_pointer = unsafe { alloc(layout) };
        assert!(!allocation_pointer.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(allocation_pointer, 0, length_in_bytes) };
        Self { pointer: allocation_pointer, length_in_bytes, layout }
    }
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) }
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) }
    }
}
impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer, self.layout) }
    }
}

fn next_random(state: &mut u64) -> u64 {
    let mut mixed_state = *state;
    mixed_state ^= mixed_state >> 12;
    mixed_state ^= mixed_state << 25;
    mixed_state ^= mixed_state >> 27;
    *state = mixed_state;
    mixed_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

struct Device {
    file: std::fs::File,
    input_output_counters: IoCounters,
}
impl Device {
    fn read_page(&mut self, page: u64, page_buffer: &mut AlignedBuffer) {
        self.file.seek(SeekFrom::Start(page * PAGE_BYTES as u64)).expect("seek");
        self.file.read_exact(page_buffer.as_mut_slice()).expect("读页失败");
        self.input_output_counters.reads += 1;
        self.input_output_counters.bytes_read += PAGE_BYTES as u64;
    }
    fn write_page(&mut self, page: u64, page_buffer: &AlignedBuffer) {
        self.file.seek(SeekFrom::Start(page * PAGE_BYTES as u64)).expect("seek");
        self.file.write_all(page_buffer.as_slice()).expect("写页失败");
        self.input_output_counters.writes += 1;
        self.input_output_counters.bytes_written += PAGE_BYTES as u64;
    }
    fn append(&mut self, offset_bytes: u64, append_buffer: &AlignedBuffer, length_in_bytes: usize) {
        self.file.seek(SeekFrom::Start(offset_bytes)).expect("seek");
        self.file.write_all(&append_buffer.as_slice()[..length_in_bytes]).expect("顺序写失败");
        self.input_output_counters.writes += 1;
        self.input_output_counters.bytes_written += length_in_bytes as u64;
    }
}

/// 臂 A：D5 时间序。比较两个标量（都在手上），追加一条定长记录，缓冲满了顺序刷。
/// D5 的删除判定：**块的 birth 比最近一个快照还新 ⇒ 直接释放，连 deadlist 都不写。**
///
/// 这就是本实验要量的那个机制。抽成函数是为了让单测走**生产路径**——
/// 测试里再写一遍判据，就测不到 `arm_d5` 里实际用的那一份
/// （E21 的分块逻辑踩过这个坑，变异测试当场证明）。
fn is_direct_release(birth: u64, previous_snapshot_txg: u64) -> bool {
    birth > previous_snapshot_txg
}

fn arm_time_order_deadlist(device: &mut Device, operation_count: u64, seed: u64, deadlist_base: u64) -> IoCounters {
    device.input_output_counters = IoCounters::default();
    let mut deadlist_buffer = AlignedBuffer::new(DEADLIST_BUFFER_BYTES);
    let mut buffered_bytes = 0usize;
    let mut append_offset = deadlist_base;
    let mut random_state = seed;
    let previous_snapshot_txg: u64 = 1_000_000; // 常驻内存的一个标量
    for _ in 0..operation_count {
        let birth = next_random(&mut random_state) % 2_000_000; // 指针里已有的字段，零次额外 I/O
        if is_direct_release(birth, previous_snapshot_txg) {
            continue; // 直接释放：连 deadlist 都不写
        }
        deadlist_buffer.as_mut_slice()[buffered_bytes..buffered_bytes + 8].copy_from_slice(&birth.to_le_bytes());
        buffered_bytes += DEADLIST_RECORD_BYTES;
        if buffered_bytes + DEADLIST_RECORD_BYTES > DEADLIST_BUFFER_BYTES {
            device.append(append_offset, &deadlist_buffer, buffered_bytes);
            append_offset += buffered_bytes as u64;
            buffered_bytes = 0;
        }
    }
    if buffered_bytes > 0 {
        // O_DIRECT 的长度也必须块对齐——向上取整到整页（尾部是零填充，不影响计量口径）
        device.append(append_offset, &deadlist_buffer, buffered_bytes.next_multiple_of(PAGE_BYTES));
    }
    device.input_output_counters
}

/// 臂 B：引用计数。定位计数器页 → 缺页则 O_DIRECT 读 → 减一 → 标脏 → 逐出时回写。
fn arm_reference_count(
    device: &mut Device,
    operation_count: u64,
    seed: u64,
    counters: u64,
    cache_pages: usize,
    sequential: bool,
) -> IoCounters {
    device.input_output_counters = IoCounters::default();
    let mut page_cache = Lru::new(cache_pages);
    let mut page_buffer = AlignedBuffer::new(PAGE_BYTES);
    let mut random_state = seed;
    for operation_index in 0..operation_count {
        // sequential 故障：把随机访问换成顺序，命中率应当大幅上升
        let block = if sequential { operation_index % counters } else { next_random(&mut random_state) % counters };
        let page = page_of(block, COUNTERS_PER_PAGE);
        if !page_cache.contains(page) {
            if let Some(evicted_page) = page_cache.touch(page) {
                if page_cache.take_dirty(evicted_page) {
                    device.write_page(evicted_page, &page_buffer); // 回写被逐出的脏页
                }
            }
            device.read_page(page, &mut page_buffer);
        } else {
            page_cache.touch(page);
        }
        // 减一：读-改-写那个「改」
        let counter_offset = ((block % COUNTERS_PER_PAGE) * 8) as usize;
        let counter_value = u64::from_le_bytes(page_buffer.as_slice()[counter_offset..counter_offset + 8].try_into().unwrap());
        page_buffer.as_mut_slice()[counter_offset..counter_offset + 8].copy_from_slice(&counter_value.saturating_sub(1).to_le_bytes());
        page_cache.mark_dirty(page);
    }
    for page in page_cache.drain_dirty() {
        device.write_page(page, &page_buffer);
    }
    device.input_output_counters
}

/// 臂 C：D5 + 稀疏旁表。先探内存位图；未被 clone 走 A，被 clone 走 B。
/// `clone_permille` 控制被 clone 的比例。
fn arm_hybrid(
    device: &mut Device,
    operation_count: u64,
    seed: u64,
    counters: u64,
    cache_pages: usize,
    clone_permille: u64,
    deadlist_base: u64,
) -> (IoCounters, u64) {
    device.input_output_counters = IoCounters::default();
    // 稀疏旁表的探测结构：每块 1 bit，常驻内存
    let bitmap: Vec<u64> = {
        let bitmap_word_count = (counters as usize).div_ceil(64);
        let mut bitmap_words = vec![0u64; bitmap_word_count];
        let mut bitmap_random_state = seed ^ 0xDEAD_BEEF;
        for block in 0..counters {
            if next_random(&mut bitmap_random_state) % 1000 < clone_permille {
                bitmap_words[(block / 64) as usize] |= 1 << (block % 64);
            }
        }
        bitmap_words
    };
    let is_cloned = |block: u64| bitmap[(block / 64) as usize] & (1 << (block % 64)) != 0;

    let mut page_cache = Lru::new(cache_pages);
    let mut page_buffer = AlignedBuffer::new(PAGE_BYTES);
    let deadlist_buffer = AlignedBuffer::new(DEADLIST_BUFFER_BYTES);
    let mut buffered_bytes = 0usize;
    let mut append_offset = deadlist_base;
    let mut random_state = seed;
    let mut cloned_hits = 0u64;
    for _ in 0..operation_count {
        let block = next_random(&mut random_state) % counters;
        if !is_cloned(block) {
            // 未被 clone：探测零设备 I/O，走 D5 路径
            buffered_bytes += DEADLIST_RECORD_BYTES;
            if buffered_bytes + DEADLIST_RECORD_BYTES > DEADLIST_BUFFER_BYTES {
                device.append(append_offset, &deadlist_buffer, buffered_bytes);
                append_offset += buffered_bytes as u64;
                buffered_bytes = 0;
            }
            continue;
        }
        cloned_hits += 1;
        let page = page_of(block, COUNTERS_PER_PAGE);
        if !page_cache.contains(page) {
            if let Some(evicted_page) = page_cache.touch(page) {
                if page_cache.take_dirty(evicted_page) {
                    device.write_page(evicted_page, &page_buffer);
                }
            }
            device.read_page(page, &mut page_buffer);
        } else {
            page_cache.touch(page);
        }
        let counter_offset = ((block % COUNTERS_PER_PAGE) * 8) as usize;
        let counter_value = u64::from_le_bytes(page_buffer.as_slice()[counter_offset..counter_offset + 8].try_into().unwrap());
        page_buffer.as_mut_slice()[counter_offset..counter_offset + 8].copy_from_slice(&counter_value.saturating_sub(1).to_le_bytes());
        page_cache.mark_dirty(page);
    }
    if buffered_bytes > 0 {
        device.append(append_offset, &deadlist_buffer, buffered_bytes.next_multiple_of(PAGE_BYTES));
    }
    for page in page_cache.drain_dirty() {
        device.write_page(page, &page_buffer);
    }
    (device.input_output_counters, cloned_hits)
}

fn main() {
    let device_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e12-lifecycle <块设备> [种子]");
        std::process::exit(2)
    });
    // 第三个参数是**故障注入模式**：用来验证这套测量本身有没有判别力。
    // 不注入故障就无法分辨「比值 1.00 = 一切正确」与「比值 1.00 = 校验路径是摆设」。
    let fault_mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    let use_direct = fault_mode != "nodirect";
    let mut open_options = OpenOptions::new();
    open_options.read(true).write(true);
    if use_direct {
        open_options.custom_flags(O_DIRECT);
    }
    let device_file = open_options
        .open(&device_path)
        .unwrap_or_else(|open_error| {
            eprintln!("打不开 {device_path}（O_DIRECT）：{open_error}");
            std::process::exit(3)
        });
    let mut device = Device { file: device_file, input_output_counters: IoCounters::default() };
    let device_size_bytes = device.file.seek(SeekFrom::End(0)).expect("取不到设备大小");
    if device_size_bytes == 0 {
        eprintln!("设备大小为 0 —— 判定不明，整轮作废");
        std::process::exit(5);
    }

    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!("name=device_size bytes={device_size_bytes}")));

    // 档位（表大小）永远按基准缓存 256 页定义，**不随故障注入变化**——
    // 否则放大缓存时表也跟着放大，比值不变，什么也没测到（第一版就是这么错的）。
    let base_cache_pages: usize = 256;
    // bigcache 故障：只放大 LRU 容量，让 8× 档的整张表装得下，缺页率应当归零
    let cache_pages: usize = if fault_mode == "bigcache" { 2048 } else { base_cache_pages };
    let operation_count: u64 = 200_000;
    // 种子从命令行来：每轮换一个，验证结论不是某一个序列的巧合。
    // 同一轮内三臂共用同一个种子——否则比的不是同一件事。
    let seed: u64 = std::env::args()
        .nth(2)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(0x5156_1234_ABCD_0001);
    let deadlist_base = device_size_bytes / 2; // 落在设备后半段，避开计数器表
    let cache_counters = base_cache_pages as u64 * COUNTERS_PER_PAGE; // 档位基准，不随故障变
    let block_layer_statistics_available = read_block_layer_statistics(&device_path).is_some();

    // 读不到块层读数就明说，绝不静默当成 0：校验路径没了要让人看见
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config cache_pages={cache_pages} cache_counters={cache_counters} ops={operation_count} counters_per_page={COUNTERS_PER_PAGE} seed={seed} blkstat_available={block_layer_statistics_available} fault={fault_mode} o_direct={use_direct}"
        ))
    );
    if !block_layer_statistics_available {
        eprintln!("读不到 /sys/block/*/stat —— 校验路径缺失，本轮结果不许当成已校验");
    }

    for (label, multiplier) in [("fits", 1u64), ("x2", 2), ("x8", 8)] {
        let counters = cache_counters * multiplier;
        if (counters / COUNTERS_PER_PAGE + 1) * PAGE_BYTES as u64 > device_size_bytes / 2 {
            eprintln!("设备太小，装不下 {label} 档的计数器表");
            std::process::exit(6);
        }

        let statistics_before = read_block_layer_statistics(&device_path);
        let arm_start = Instant::now();
        let time_order_counters = arm_time_order_deadlist(&mut device, operation_count, seed, deadlist_base);
        let time_order_elapsed_nanoseconds = arm_start.elapsed().as_nanos() as u64;
        let time_order_block_layer = format_block_layer_delta(block_layer_delta(statistics_before, read_block_layer_statistics(&device_path)));
        println!("{}", emitter.emit_raw(&format!(
            "name=d5 ws={label} counters={counters} reads={} writes={} bytes_w={} elapsed_ns={time_order_elapsed_nanoseconds} io_per_op={:.6} {time_order_block_layer}",
            time_order_counters.reads, time_order_counters.writes, time_order_counters.bytes_written, time_order_counters.io_per_op(operation_count).unwrap()
        )));

        let statistics_before = read_block_layer_statistics(&device_path);
        let arm_start = Instant::now();
        let reference_count_counters = arm_reference_count(&mut device, operation_count, seed, counters, cache_pages, fault_mode == "sequential");
        let reference_count_elapsed_nanoseconds = arm_start.elapsed().as_nanos() as u64;
        let reference_count_block_layer = format_block_layer_delta(block_layer_delta(statistics_before, read_block_layer_statistics(&device_path)));
        println!("{}", emitter.emit_raw(&format!(
            "name=refcount ws={label} counters={counters} reads={} writes={} elapsed_ns={reference_count_elapsed_nanoseconds} io_per_op={:.6} {reference_count_block_layer}",
            reference_count_counters.reads, reference_count_counters.writes, reference_count_counters.io_per_op(operation_count).unwrap()
        )));

        let statistics_before = read_block_layer_statistics(&device_path);
        let arm_start = Instant::now();
        let (hybrid_counters, cloned_hit_count) = arm_hybrid(&mut device, operation_count, seed, counters, cache_pages, 10, deadlist_base);
        let hybrid_elapsed_nanoseconds = arm_start.elapsed().as_nanos() as u64;
        let hybrid_block_layer = format_block_layer_delta(block_layer_delta(statistics_before, read_block_layer_statistics(&device_path)));
        println!("{}", emitter.emit_raw(&format!(
            "name=hybrid ws={label} counters={counters} clone_permille=10 cloned_hits={cloned_hit_count} reads={} writes={} elapsed_ns={hybrid_elapsed_nanoseconds} io_per_op={:.6} {hybrid_block_layer}",
            hybrid_counters.reads, hybrid_counters.writes, hybrid_counters.io_per_op(operation_count).unwrap()
        )));
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **D5 的删除判定：birth 比最近快照新 ⇒ 直接释放。**
    /// 边界必须是严格大于——等于最近快照代号的块**仍被那个快照引用**，不能直接释放。
    /// 判反了会让实验量到一个不存在的收益。
    #[test]
    fn direct_release_boundary_is_strictly_greater() {
        assert!(is_direct_release(1_000_001, 1_000_000), "更新的块应当直接释放");
        assert!(!is_direct_release(1_000_000, 1_000_000), "等于最近快照代号的块仍被引用，不许直接释放");
        assert!(!is_direct_release(999_999, 1_000_000), "更老的块必须进 deadlist");
    }

    /// 随机数必须真的散开。它决定「多大比例的块走直接释放」，
    /// 退化成常数的话两条臂的 I/O 比就是假的。
    #[test]
    fn next_random_is_not_degenerate() {
        let mut random_state = 7u64;
        let samples: Vec<u64> = (0..1000).map(|_| next_random(&mut random_state) % 2_000_000).collect();
        let distinct_values: std::collections::HashSet<_> = samples.iter().collect();
        assert!(distinct_values.len() > 900, "1000 次只取到 {} 个不同值，没散开", distinct_values.len());
        // 且直接释放的比例应当接近一半（阈值取在 2_000_000 的中点 1_000_000）
        let direct_release_count = samples.iter().filter(|&&birth| is_direct_release(birth, 1_000_000)).count();
        assert!((400..=600).contains(&direct_release_count), "直接释放比例 {direct_release_count}/1000 偏得太远，负载不是设想的那个");
    }

    /// **`O_DIRECT` 要求缓冲按页对齐。** 不对齐的话读写会 `EINVAL`，
    /// 而那会表现为「I/O 计数为 0」——看起来像「很省」，实际是根本没跑。
    #[test]
    fn aligned_buffer_is_page_aligned() {
        for length_in_bytes in [PAGE_BYTES, PAGE_BYTES * 4, DEADLIST_BUFFER_BYTES] {
            let aligned_buffer = AlignedBuffer::new(length_in_bytes);
            assert_eq!(aligned_buffer.pointer as usize % PAGE_BYTES, 0, "长度 {length_in_bytes} 的缓冲没有按页对齐");
            assert_eq!(aligned_buffer.length_in_bytes, length_in_bytes);
        }
    }

    /// 块层计数的差值必须逐字段相减；少减一个字段会让校验路径悄悄失效。
    #[test]
    fn block_layer_delta_subtracts_every_field() {
        let before = BlockLayerStatistics { read_requests_completed: 1, read_sectors: 3, write_requests_completed: 2, write_sectors: 4 };
        let after = BlockLayerStatistics { read_requests_completed: 11, read_sectors: 33, write_requests_completed: 22, write_sectors: 44 };
        let delta = block_layer_delta(Some(before), Some(after)).expect("两侧都有读数时应当有差值");
        assert_eq!((delta.read_requests_completed, delta.write_requests_completed, delta.read_sectors, delta.write_sectors), (10, 20, 30, 40));
    }

    /// **读不到 ≠ 读到 0**：任一侧缺读数时差值必须是 None，不许当成 0。
    #[test]
    fn missing_block_layer_statistics_are_not_zero() {
        let sample_statistics = BlockLayerStatistics { read_requests_completed: 1, read_sectors: 3, write_requests_completed: 2, write_sectors: 4 };
        assert!(block_layer_delta(None, Some(sample_statistics)).is_none());
        assert!(block_layer_delta(Some(sample_statistics), None).is_none());
        assert!(block_layer_delta(None, None).is_none());
    }
}
