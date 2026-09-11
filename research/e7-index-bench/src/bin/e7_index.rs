//! E7：三种索引结构在同一 checkpoint 节奏下的设备 I/O 对比。
//!
//! 判据与三条硬要求见 kb/experiments.md E7。要点：
//!   1. 每条臂真的实现它声称的结构——配 `--selfcheck` 把核心判据换成常量，结果必须显著变化；
//!   2. 三臂共用同一个 checkpoint 节奏与同一个节点缓存，摊销架构对称；
//!   3. 设备 I/O 与虚机内 /sys/block/*/stat 逐项比对。
//!
//! 负载是**对已存在 key 空间的随机更新**，因此不需要节点分裂——
//! 三条臂都是真结构，不是 stub。分裂/合并行为不在本实验射程内。

use e7_index_bench::{Emitter, Lru};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名
const PAGE_BYTES: usize = 4096;
/// 节点占几个 4 KiB 页 —— 运行期参数（D8 已定：节点大小是每套布局的**格式参数**）。
/// 用 static mut 是因为它在 main 里定一次就不再变；所有读取都在单线程里。
static mut NODE_PAGE_COUNT: usize = 1;
fn node_page_count() -> usize { unsafe { NODE_PAGE_COUNT } }
fn node_bytes() -> usize { node_page_count() * PAGE_BYTES }
fn slots() -> usize { (node_bytes() - 16) / 16 }
const FANOUT: usize = 32;
const LEAF_COUNT: usize = 1024; // FANOUT^2
const KEYS_PER_LEAF: usize = 64;
const KEY_COUNT: u64 = (LEAF_COUNT * KEYS_PER_LEAF) as u64;
/// 缓存页数的默认值。**测试要钉的解析值 1.9375 就是按它算的**——
/// 抽成常量是为了让测试绑住**生产用的那个数**，而不是在测试里再抄一遍
/// （2026-08-29：e9 那边正因为在测试里重抄了一遍逻辑，变异测试一条都没红）。
const DEFAULT_CACHE_PAGES: usize = 64;

// 盘上布局：页 0 = 根，页 1..=16 = 内部节点，页 17..=272 = 叶子
fn root_page_number() -> u64 { 0 }
fn internal_page_number(internal_index: usize) -> u64 { 1 + internal_index as u64 }
fn leaf_page_number(leaf_index: usize) -> u64 { 1 + FANOUT as u64 + leaf_index as u64 }
fn leaf_of(key: u64) -> usize { (key / KEYS_PER_LEAF as u64) as usize }
fn internal_of(leaf: usize) -> usize { leaf / FANOUT }

struct AlignedBuffer { pointer: *mut u8, length_in_bytes: usize, layout: Layout }
impl AlignedBuffer {
    fn new(length_in_bytes: usize) -> Self {
        let layout = Layout::from_size_align(length_in_bytes, PAGE_BYTES).unwrap();
        let allocated_pointer = unsafe { alloc(layout) };
        assert!(!allocated_pointer.is_null());
        unsafe { std::ptr::write_bytes(allocated_pointer, 0, length_in_bytes) };
        Self { pointer: allocated_pointer, length_in_bytes, layout }
    }
    fn as_slice(&self) -> &[u8] { unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) } }
    fn as_mut_slice(&mut self) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) } }
}
impl Drop for AlignedBuffer { fn drop(&mut self) { unsafe { dealloc(self.pointer, self.layout) } } }

/// 一个节点的内存形态：条目数 + 条目。日志结构臂用它做「追加」，排序臂用它做「原地」。
#[derive(Clone)]
struct Node { entry_count: usize, entries: Vec<(u64, u64)> }
impl Node {
    fn new() -> Self { Self { entry_count: 0, entries: vec![(0, 0); slots()] } }
    fn decode(buffer: &AlignedBuffer) -> Self {
        let entry_count = u64::from_le_bytes(buffer.as_slice()[0..8].try_into().unwrap()) as usize;
        let entry_count = entry_count.min(slots());
        let mut entries = vec![(0u64, 0u64); slots()];
        for (entry_position, slot) in entries.iter_mut().enumerate().take(entry_count) {
            let byte_offset = 16 + entry_position * 16;
            *slot = (
                u64::from_le_bytes(buffer.as_slice()[byte_offset..byte_offset + 8].try_into().unwrap()),
                u64::from_le_bytes(buffer.as_slice()[byte_offset + 8..byte_offset + 16].try_into().unwrap()),
            );
        }
        Self { entry_count, entries }
    }
    fn encode(&self, buffer: &mut AlignedBuffer) {
        buffer.as_mut_slice()[0..8].copy_from_slice(&(self.entry_count as u64).to_le_bytes());
        for entry_position in 0..self.entry_count {
            let byte_offset = 16 + entry_position * 16;
            buffer.as_mut_slice()[byte_offset..byte_offset + 8].copy_from_slice(&self.entries[entry_position].0.to_le_bytes());
            buffer.as_mut_slice()[byte_offset + 8..byte_offset + 16].copy_from_slice(&self.entries[entry_position].1.to_le_bytes());
        }
    }
    fn full(&self) -> bool { self.entry_count >= slots() }
    fn push(&mut self, key: u64, value: u64) { if self.entry_count < slots() { self.entries[self.entry_count] = (key, value); self.entry_count += 1; } }
    /// 原地更新（排序臂）：找到就改，找不到就追加
    fn upsert(&mut self, key: u64, value: u64) {
        for entry_position in 0..self.entry_count { if self.entries[entry_position].0 == key { self.entries[entry_position].1 = value; return; } }
        self.push(key, value);
    }
    /// 压实（日志结构臂）：同 key 保留最后一条
    fn compact(&mut self) {
        let mut latest_by_key: std::collections::BTreeMap<u64, u64> = Default::default();
        for entry_position in 0..self.entry_count { latest_by_key.insert(self.entries[entry_position].0, self.entries[entry_position].1); }
        self.entry_count = 0;
        for (key, value) in latest_by_key { self.push(key, value); }
    }
    /// 点查：日志结构下要**从后往前**扫（后写的胜）
    fn get(&self, key: u64) -> Option<u64> {
        for entry_position in (0..self.entry_count).rev() { if self.entries[entry_position].0 == key { return Some(self.entries[entry_position].1); } }
        None
    }
}

struct Device { file: std::fs::File, reads: u64, writes: u64, read_bytes: u64, written_bytes: u64 }
impl Device {
    fn read_page(&mut self, page_number: u64, buffer: &mut AlignedBuffer) {
        self.file.seek(SeekFrom::Start(page_number * node_bytes() as u64)).unwrap();
        self.file.read_exact(buffer.as_mut_slice()).unwrap();
        self.reads += 1;
        self.read_bytes += node_bytes() as u64;
    }
    fn write_page(&mut self, page_number: u64, buffer: &AlignedBuffer) {
        self.file.seek(SeekFrom::Start(page_number * node_bytes() as u64)).unwrap();
        self.file.write_all(buffer.as_slice()).unwrap();
        self.writes += 1;
        self.written_bytes += node_bytes() as u64;
    }
}

/// 节点缓存 + checkpoint：三臂共用，保证摊销架构对称。
struct Cache { capacity_pages: usize, lru: Lru, map: std::collections::HashMap<u64, Node> }
impl Cache {
    fn new(capacity_pages: usize) -> Self { Self { capacity_pages, lru: Lru::new(capacity_pages), map: Default::default() } }
    /// 常驻集不许超过声明的缓存。
    /// **补这一条的理由**（2026-08-31）：E56 的 harness 有一份形状完全相同的 `Cache`，
    /// 那里 `put` 不过 LRU ⇒ 递归期间被逐出的页会被塞回 `map` 而 LRU 追踪不到，
    /// 此后永远免费命中、还会繁殖；实测声明 66 个节点的缓存涨到常驻 688。
    /// E7 的调用形态是「get 之后紧跟 put、中间不嵌套别的缓存操作」，按推理不该中招——
    /// **但推理不算证据**，所以在这里放一道会红的闸，让它自己说话。
    fn check_residency(&self) {
        if self.map.len() > self.capacity_pages {
            eprintln!("E7 常驻集闸破了：常驻 {} 超过声明的缓存 {}", self.map.len(), self.capacity_pages);
            std::process::exit(8);
        }
    }
    fn get(&mut self, device: &mut Device, page_number: u64, buffer: &mut AlignedBuffer) -> Node {
        if let Some(cached_node) = self.map.get(&page_number) { self.lru.touch(page_number); return cached_node.clone(); }
        if let Some(evicted_page) = self.lru.touch(page_number) {
            if self.lru.take_dirty(evicted_page) {
                let evicted_node = self.map.get(&evicted_page).unwrap().clone();
                evicted_node.encode(buffer); device.write_page(evicted_page, buffer);
            }
            self.map.remove(&evicted_page);
        }
        device.read_page(page_number, buffer);
        let decoded_node = Node::decode(buffer);
        self.map.insert(page_number, decoded_node.clone());
        self.check_residency();
        decoded_node
    }
    fn put(&mut self, page_number: u64, node: Node) { self.map.insert(page_number, node); self.lru.mark_dirty(page_number); self.check_residency(); }
    /// checkpoint：把全部脏节点刷下去。三臂在同一节奏上调用它。
    fn checkpoint(&mut self, device: &mut Device, buffer: &mut AlignedBuffer) {
        for page_number in self.lru.drain_dirty() {
            if let Some(dirty_node) = self.map.get(&page_number) { dirty_node.encode(buffer); device.write_page(page_number, buffer); }
        }
    }
}

fn next_random(state: &mut u64) -> u64 {
    let mut xorshift_state = *state; xorshift_state ^= xorshift_state >> 12; xorshift_state ^= xorshift_state << 25; xorshift_state ^= xorshift_state >> 27; *state = xorshift_state;
    xorshift_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

// ── 臂 1：朴素排序节点 B+tree，无批量。这是**阳性对照**：效应必须在这里出现。
//    它故意不共用 checkpoint 节奏——「无批量」正是它要代表的那一档。
fn arm_sorted_without_batching(device: &mut Device, operation_count: u64, seed: u64, cache_pages: usize) -> (u64, u64) {
    let (reads_before, writes_before) = (device.reads, device.writes);
    let mut cache = Cache::new(cache_pages);
    let mut buffer = AlignedBuffer::new(node_bytes());
    let mut random_state = seed;
    for _ in 0..operation_count {
        let key = next_random(&mut random_state) % KEY_COUNT;
        let page_number = leaf_page_number(leaf_of(key));
        let mut leaf_node = cache.get(device, page_number, &mut buffer);
        leaf_node.upsert(key, key ^ 0xABCD);
        leaf_node.encode(&mut buffer);
        device.write_page(page_number, &buffer); // 写穿：每次更新都落盘
        cache.put(page_number, leaf_node);
        cache.lru.take_dirty(page_number); // 已经写过了，不再算脏
    }
    (device.reads - reads_before, device.writes - writes_before)
}

// ── 臂 2：D8 现方向 —— 日志结构节点（节点内追加不插入）+ write buffer 前端。
fn arm_log_structured_with_write_buffer(
    device: &mut Device, operation_count: u64, seed: u64, cache_pages: usize, write_buffer_capacity: usize, checkpoint_interval: u64,
    shuffle: bool,
) -> (u64, u64) {
    let (reads_before, writes_before) = (device.reads, device.writes);
    let mut cache = Cache::new(cache_pages);
    let mut buffer = AlignedBuffer::new(node_bytes());
    let mut random_state = seed;
    let mut write_buffer: Vec<(u64, u64)> = Vec::with_capacity(write_buffer_capacity.max(1));
    let flush = |write_buffer: &mut Vec<(u64, u64)>, cache: &mut Cache, device: &mut Device, buffer: &mut AlignedBuffer| {
        if write_buffer.is_empty() { return; }
        // 攒批 → 排序 → 去重（后者胜），正是 D8 write buffer 的形态
        write_buffer.sort_unstable_by_key(|buffered_update| (leaf_of(buffered_update.0), buffered_update.0));
        let mut deduplicated: Vec<(u64, u64)> = Vec::with_capacity(write_buffer.len());
        for &(key, value) in write_buffer.iter() {
            if let Some(previous_update) = deduplicated.last_mut() { if previous_update.0 == key { previous_update.1 = value; continue; } }
            deduplicated.push((key, value));
        }
        if shuffle {
            // 干预：**先按叶分组、再打乱组的顺序**，同叶聚合原样保留，只改叶子被访问的次序。
            // 这样两个模式的叶子取用次数完全相同，唯一变量就是顺序——
            // 若命中率因此回来，就证实「排序后的 flush 是一次顺序扫描，
            // 而 LRU 对反复的、大于缓存的顺序扫描命中率为 0」。
            let mut groups: Vec<Vec<(u64, u64)>> = Vec::new();
            let mut update_position = 0;
            while update_position < deduplicated.len() {
                let leaf_index = leaf_of(deduplicated[update_position].0);
                let mut group = Vec::new();
                while update_position < deduplicated.len() && leaf_of(deduplicated[update_position].0) == leaf_index { group.push(deduplicated[update_position]); update_position += 1; }
                groups.push(group);
            }
            let mut shuffle_state = 0x1234_5678_9ABC_DEF0u64 ^ (groups.len() as u64);
            for group_position in (1..groups.len()).rev() {
                let swap_position = (next_random(&mut shuffle_state) % (group_position as u64 + 1)) as usize;
                groups.swap(group_position, swap_position);
            }
            deduplicated = groups.into_iter().flatten().collect();
        }
        let mut update_position = 0;
        while update_position < deduplicated.len() {
            let leaf_index = leaf_of(deduplicated[update_position].0);
            let page_number = leaf_page_number(leaf_index);
            let mut leaf_node = cache.get(device, page_number, buffer);
            while update_position < deduplicated.len() && leaf_of(deduplicated[update_position].0) == leaf_index {
                if leaf_node.full() { leaf_node.compact(); }    // 日志结构：满了才压实
                leaf_node.push(deduplicated[update_position].0, deduplicated[update_position].1); // 追加，不插入
                update_position += 1;
            }
            cache.put(page_number, leaf_node);
        }
        write_buffer.clear();
    };
    for operation_index in 0..operation_count {
        let key = next_random(&mut random_state) % KEY_COUNT;
        write_buffer.push((key, key ^ 0xABCD));
        if write_buffer.len() >= write_buffer_capacity.max(1) { flush(&mut write_buffer, &mut cache, device, &mut buffer); }
        if (operation_index + 1) % checkpoint_interval == 0 { flush(&mut write_buffer, &mut cache, device, &mut buffer); cache.checkpoint(device, &mut buffer); }
    }
    flush(&mut write_buffer, &mut cache, device, &mut buffer);
    cache.checkpoint(device, &mut buffer);
    (device.reads - reads_before, device.writes - writes_before)
}

// ── 臂 3：Bε —— 内部节点带消息缓冲，消息沿树下刷。
fn arm_message_buffered_tree(
    device: &mut Device, operation_count: u64, seed: u64, cache_pages: usize, message_buffer_capacity: usize, checkpoint_interval: u64,
) -> (u64, u64) {
    let (reads_before, writes_before) = (device.reads, device.writes);
    let mut cache = Cache::new(cache_pages);
    let mut buffer = AlignedBuffer::new(node_bytes());
    let mut random_state = seed;
    let effective_buffer_capacity = message_buffer_capacity.min(slots());
    for operation_index in 0..operation_count {
        let key = next_random(&mut random_state) % KEY_COUNT;
        let mut root = cache.get(device, root_page_number(), &mut buffer);
        root.push(key, key ^ 0xABCD);
        // 根缓冲满 → 下刷到 16 个内部节点
        if root.entry_count >= effective_buffer_capacity.max(1) {
            let messages: Vec<(u64, u64)> = root.entries[..root.entry_count].to_vec();
            root.entry_count = 0;
            cache.put(root_page_number(), root);
            for internal_index in 0..FANOUT {
                let messages_for_internal_node: Vec<(u64, u64)> =
                    messages.iter().copied().filter(|(key, _)| internal_of(leaf_of(*key)) == internal_index).collect();
                if messages_for_internal_node.is_empty() { continue; }
                let internal_node_page = internal_page_number(internal_index);
                let mut internal_node = cache.get(device, internal_node_page, &mut buffer);
                for (key, value) in messages_for_internal_node { internal_node.push(key, value); }
                // 内部缓冲满 → 下刷到它的 16 个叶子
                if internal_node.entry_count >= effective_buffer_capacity.max(1) {
                    let internal_messages: Vec<(u64, u64)> = internal_node.entries[..internal_node.entry_count].to_vec();
                    internal_node.entry_count = 0;
                    cache.put(internal_node_page, internal_node);
                    let mut messages_by_leaf: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = Default::default();
                    for (key, value) in internal_messages { messages_by_leaf.entry(leaf_of(key)).or_default().push((key, value)); }
                    for (leaf_index, leaf_messages) in messages_by_leaf {
                        let page_number = leaf_page_number(leaf_index);
                        let mut leaf_node = cache.get(device, page_number, &mut buffer);
                        for (key, value) in leaf_messages { if leaf_node.full() { leaf_node.compact(); } leaf_node.push(key, value); }
                        cache.put(page_number, leaf_node);
                    }
                } else {
                    cache.put(internal_node_page, internal_node);
                }
            }
        } else {
            cache.put(root_page_number(), root);
        }
        if (operation_index + 1) % checkpoint_interval == 0 { cache.checkpoint(device, &mut buffer); }
    }
    cache.checkpoint(device, &mut buffer);
    (device.reads - reads_before, device.writes - writes_before)
}

/// 点查：三臂共用同一条读路径口径——根 → 内部 → 叶子，逐层看有没有该 key。
fn point_query(device: &mut Device, cache: &mut Cache, buffer: &mut AlignedBuffer, key: u64) -> u64 {
    let reads_before = device.reads;
    let root = cache.get(device, root_page_number(), buffer);
    if root.get(key).is_none() {
        let internal_node = cache.get(device, internal_page_number(internal_of(leaf_of(key))), buffer);
        if internal_node.get(key).is_none() {
            let _ = cache.get(device, leaf_page_number(leaf_of(key)), buffer);
        }
    }
    device.reads - reads_before
}

#[derive(Default, Clone, Copy)]
struct BlockLayerCounters { reads: u64, writes: u64 }
fn read_block_layer_counters(device_path: &str) -> Option<BlockLayerCounters> {
    let name = device_path.rsplit('/').next()?;
    let block_statistics_text = std::fs::read_to_string(format!("/sys/block/{name}/stat")).ok()?;
    let block_statistics_fields: Vec<u64> = block_statistics_text.split_whitespace().filter_map(|field_text| field_text.parse().ok()).collect();
    if block_statistics_fields.len() < 8 { return None; }
    Some(BlockLayerCounters { reads: block_statistics_fields[0], writes: block_statistics_fields[4] })
}
fn block_counter_delta(before: Option<BlockLayerCounters>, after: Option<BlockLayerCounters>) -> String {
    match (before, after) {
        (Some(before), Some(after)) => format!("blk_r={} blk_w={}", after.reads - before.reads, after.writes - before.writes),
        _ => "blk_r=NA blk_w=NA".into(),
    }
}

fn main() {
    let device_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e7-index <块设备> [种子] [none|selfcheck|nodirect] [缓存页数]"); std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|argument| argument.parse().ok()).unwrap_or(0x51561234);
    let mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    // selfcheck：把两条攒批臂的核心判据换成常量（缓冲=1），结果必须显著变化；
    // 没变化就说明那个结构根本没参与——kb/experiments.md E7 硬要求 1。
    let selfcheck = mode == "selfcheck";
    let use_direct = mode != "nodirect";

    let mut open_options = OpenOptions::new();
    open_options.read(true).write(true);
    if use_direct { open_options.custom_flags(O_DIRECT); }
    let device_file = open_options.open(&device_path).unwrap_or_else(|error| { eprintln!("打不开 {device_path}: {error}"); std::process::exit(3) });
    let mut device = Device { file: device_file, reads: 0, writes: 0, read_bytes: 0, written_bytes: 0 };
    let device_size_bytes = device.file.seek(SeekFrom::End(0)).unwrap();
    if device_size_bytes < (1 + FANOUT + LEAF_COUNT) as u64 * node_bytes() as u64 { eprintln!("设备太小：需要 {} 字节", (1 + FANOUT + LEAF_COUNT) * node_bytes()); std::process::exit(5); }

    let mut emitter = Emitter::new();
    let operation_count: u64 = 200_000;
    // 缓存/树比是本实验最承重的自变量：缓存住整棵树会让三条臂都好看，量不出结构差异。
    // 第 4 个参数指定缓存页数，不给则用 64（约占 1057 个节点的 6%）。
    let cache_pages: usize = std::env::args().nth(4).and_then(|argument| argument.parse().ok()).unwrap_or(DEFAULT_CACHE_PAGES);
    // 第 5 个参数：一个节点占几个 4 KiB 页。节点大小是 D8 已定的每套布局格式参数。
    let node_pages: usize = std::env::args().nth(5).and_then(|argument| argument.parse().ok()).unwrap_or(1);
    unsafe { NODE_PAGE_COUNT = node_pages.max(1) };
    let write_buffer_capacity = if selfcheck { 1 } else { 512 };
    let message_buffer_capacity = if selfcheck { 1 } else { 200 };
    let checkpoint_interval: u64 = 4096;
    println!("{}", emitter.emit_raw(&format!(
        "name=config nodes={} node_kib={} slots={} keys={KEY_COUNT} ops={operation_count} cache_nodes={cache_pages} cache_ratio={:.3} \
         wb_cap={write_buffer_capacity} betree_buf={message_buffer_capacity} ckpt={checkpoint_interval} seed={seed} mode={mode} \
         o_direct={use_direct} blkstat={}", 1 + FANOUT + LEAF_COUNT, node_bytes()/1024, slots(), cache_pages as f64 / (1 + FANOUT + LEAF_COUNT) as f64, read_block_layer_counters(&device_path).is_some())));

    // 初始化：把每个叶子填上它那 64 个 key
    {
        let mut buffer = AlignedBuffer::new(node_bytes());
        for leaf_index in 0..LEAF_COUNT {
            let mut leaf_node = Node::new();
            for key_in_leaf in 0..KEYS_PER_LEAF { let key = (leaf_index * KEYS_PER_LEAF + key_in_leaf) as u64; leaf_node.push(key, key); }
            leaf_node.encode(&mut buffer); device.write_page(leaf_page_number(leaf_index), &buffer);
        }
        let empty = Node::new();
        for internal_index in 0..FANOUT { empty.encode(&mut buffer); device.write_page(internal_page_number(internal_index), &buffer); }
        empty.encode(&mut buffer); device.write_page(root_page_number(), &buffer);
        device.reads = 0; device.writes = 0; device.read_bytes = 0; device.written_bytes = 0; // 初始化不计入
    }

    for (name, which) in [("sorted_bplus", 0), ("logstruct_wb", 1), ("betree", 2)] {
        let block_counters_before_updates = read_block_layer_counters(&device_path);
        let (read_bytes_before, written_bytes_before) = (device.read_bytes, device.written_bytes);
        let update_start = Instant::now();
        let (reads, writes) = match which {
            0 => arm_sorted_without_batching(&mut device, operation_count, seed, cache_pages),
            1 => arm_log_structured_with_write_buffer(&mut device, operation_count, seed, cache_pages, write_buffer_capacity, checkpoint_interval, mode == "shuffle"),
            _ => arm_message_buffered_tree(&mut device, operation_count, seed, cache_pages, message_buffer_capacity, checkpoint_interval),
        };
        let elapsed_nanoseconds = update_start.elapsed().as_nanos() as u64;
        let block_counter_text = block_counter_delta(block_counters_before_updates, read_block_layer_counters(&device_path));
        println!("{}", emitter.emit_raw(&format!(
            "name=update arm={name} reads={reads} writes={writes} io={} io_per_op={:.6} bytes_per_op={:.1} elapsed_ns={elapsed_nanoseconds} {block_counter_text}",
            reads + writes, (reads + writes) as f64 / operation_count as f64, (device.read_bytes - read_bytes_before + device.written_bytes - written_bytes_before) as f64 / operation_count as f64)));

        // 点查：同一批 key，冷缓存
        let mut cache = Cache::new(cache_pages);
        let mut buffer = AlignedBuffer::new(node_bytes());
        let mut query_random_state = seed ^ 0x9E37;
        let query_count: u64 = 20_000;
        let block_counters_before_queries = read_block_layer_counters(&device_path);
        let (reads_before_queries, query_start) = (device.reads, Instant::now());
        let mut hits = 0u64;
        for _ in 0..query_count { hits += point_query(&mut device, &mut cache, &mut buffer, next_random(&mut query_random_state) % KEY_COUNT); }
        let query_elapsed_nanoseconds = query_start.elapsed().as_nanos() as u64;
        println!("{}", emitter.emit_raw(&format!(
            "name=query arm={name} reads={} reads_per_query={:.4} elapsed_ns={query_elapsed_nanoseconds} {} probe_reads={hits}",
            device.reads - reads_before_queries, (device.reads - reads_before_queries) as f64 / query_count as f64, block_counter_delta(block_counters_before_queries, read_block_layer_counters(&device_path)))));
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {

    /// **口径由检查钉住，不靠运气。**
    ///
    /// kb 引用 E7（离线索引 harness） 时带的口径逐字是「树只有 1057 个节点、缓存住 6%」，
    /// 而那两个数是 `FANOUT` / `LEAF_COUNT` / `DEFAULT_CACHE_PAGES` 算出来的。
    /// ⚠️ 改掉任一个常量，此前三条单测**全绿**（2026-08-29 变异测试实测三条全是盲区），
    /// 而归档里的数字会变、kb 里那句口径会静默失效。
    #[test]
    fn the_geometry_quoted_in_the_knowledge_base_is_pinned() {
        let total_nodes = 1 + FANOUT + LEAF_COUNT;
        assert_eq!(total_nodes, 1057, "kb 引的是 1057 个节点");
        assert_eq!(LEAF_COUNT, FANOUT * FANOUT, "N_LEAF 该恰好是 FANOUT 的平方");
        let ratio = DEFAULT_CACHE_PAGES as f64 / total_nodes as f64;
        assert!(
            (0.055..0.065).contains(&ratio),
            "kb 引的是「缓存住 6%」，实算 {:.4}",
            ratio
        );
    }
    use super::*;

    /// **页号分配不许重叠。** 根 / 内部节点 / 叶各占一段，重叠的话
    /// 一次写会踩掉另一个节点，而实验只数 I/O 次数、看不出内容被踩。
    /// **把无批量对照臂的解析式钉成绝对值 1.9375。**
    ///
    /// ⚠️ **本实验此前一条绝对值断言都没有**（2026-08-29 对抗验证补入）：
    /// 四个单测钉的是页号不重叠、key 映射在界内、树的几何自洽——**全是结构性质**。
    /// 而 kb 里写着的第三条独立校验路径「两条解析式与实测吻合（1.9375 vs 1.9378）」
    /// **只活在散文里，没有任何断言钉它**。⇒ 几何常量一改，那个 1.9375 就静默作废，
    /// 而 decisions.md D11（索引节点要不要留消息缓冲区） 前置 2 引的 3.22 倍正是这套数算出来的。
    ///
    /// 解析式：`sorted_bplus` 每次操作恰好 1 次写 + `(1 − 缓存页 / 叶数)` 次读。
    /// 缓存 64 页、1024 个叶 ⇒ `1 + (1 − 64/1024) = 1.9375`。
    ///
    /// ⚠️ **它钉的是解析式与几何常量，不是实测值**——实测要设备与虚机，
    /// 单测里跑不了。两者的比对仍然只能靠复跑（口径见 E7 正文）。
    #[test]
    fn the_no_batching_control_arm_has_an_analytic_io_per_op_of_1_9375() {
        assert_eq!(LEAF_COUNT, 1024, "叶数变了，1.9375 这个解析值跟着失效");
        assert_eq!(FANOUT * FANOUT, LEAF_COUNT, "几何不自洽");
        assert_eq!(DEFAULT_CACHE_PAGES, 64, "缓存页默认值变了，1.9375 这个解析值跟着失效");
        let analytic = 1.0 + (1.0 - DEFAULT_CACHE_PAGES as f64 / LEAF_COUNT as f64);
        assert!((analytic - 1.9375).abs() < 1e-12,
                "解析式算出 {analytic}，而 kb 与实测对的是 1.9375");
    }

    #[test]
    fn page_ranges_do_not_overlap() {
        let mut seen = std::collections::HashSet::new();
        assert!(seen.insert(root_page_number()), "根页号重复");
        for internal_index in 0..FANOUT { assert!(seen.insert(internal_page_number(internal_index)), "内部节点 {internal_index} 页号与别人重叠"); }
        for leaf_index in 0..LEAF_COUNT { assert!(seen.insert(leaf_page_number(leaf_index)), "叶 {leaf_index} 页号与别人重叠"); }
        assert_eq!(seen.len(), 1 + FANOUT + LEAF_COUNT);
    }

    /// **key → 叶 → 内部节点这条映射必须覆盖全部 key，且落在合法范围内。**
    /// 越界会让实验读到不属于树的页，而 I/O 计数照样是「正常的一次读」。
    #[test]
    fn every_key_maps_into_range() {
        for key in [0u64, 1, KEYS_PER_LEAF as u64 - 1, KEYS_PER_LEAF as u64, KEY_COUNT - 1] {
            let leaf_index = leaf_of(key);
            assert!(leaf_index < LEAF_COUNT, "key {key} 映射到叶 {leaf_index}，超出 {LEAF_COUNT}");
            let internal_index = internal_of(leaf_index);
            assert!(internal_index < FANOUT, "叶 {leaf_index} 映射到内部节点 {internal_index}，超出 {FANOUT}");
        }
        // 每个叶恰好装 KEYS_PER_LEAF 个 key，且相邻 key 不会跨叶跳号
        assert_eq!(leaf_of(0), leaf_of(KEYS_PER_LEAF as u64 - 1));
        assert_eq!(leaf_of(KEYS_PER_LEAF as u64), 1);
    }

    /// 树的几何必须自洽：`FANOUT^2 == LEAF_COUNT`，且 key 总数 = 叶数 × 每叶 key 数。
    /// 几何写错时，「一次点查走几层」这个被测量就不是设想的那个。
    #[test]
    fn tree_geometry_is_self_consistent() {
        assert_eq!(FANOUT * FANOUT, LEAF_COUNT, "FANOUT^2 应当等于叶数");
        assert_eq!(KEY_COUNT, (LEAF_COUNT * KEYS_PER_LEAF) as u64);
    }

    /// **`O_DIRECT` 要求缓冲按页对齐**；不对齐会 `EINVAL`，
    /// 而那表现为「I/O 计数为 0」——看起来像很省，实际是根本没跑。
    #[test]
    fn aligned_buffer_is_page_aligned() {
        for length_in_bytes in [PAGE_BYTES, PAGE_BYTES * 8] {
            let buffer = AlignedBuffer::new(length_in_bytes);
            assert_eq!(buffer.pointer as usize % PAGE_BYTES, 0, "长度 {length_in_bytes} 的缓冲没有按页对齐");
        }
    }
}
