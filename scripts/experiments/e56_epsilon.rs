//! E56：消息缓冲的收益 vs ε —— 在 D8 已定的 16 KiB 节点上，把「缓冲」与「扇出」放回同一块节点字节里。
//!
//! ## 与 E7 的区别（也是立项理由）
//!
//! E7 的 Bε 臂里 `buf_cap = 200`、`FANOUT = 32` 是两个互不相干的常量，内部节点只放消息、不放 pivot
//! ⇒ 缓冲与扇出从不争节点字节，而那条张力就是 Bε 的全部机制。
//! 本文件把两者都做成 ε 的函数：
//!
//!   F(ε) = floor(S·(1−ε) / P)      pivot = 8(key) + 40(D19 指针头部) = 48 B
//!   B(ε) = floor(S·ε   / M)        消息  = 8(key) + 8(value)          = 16 B
//!   H(ε) = 自底向上按 F 收敛到单根所需的层数
//!
//! ## 跑前写死的解析预测
//!
//!   touch_per_op = H(ε) × min(1, F(ε)/B(ε))
//!
//! 交点 B = F 落在 ε = M/(M+P) = 0.25；ε < 0.25 时摊销倍数 < 1（一条也没摊到）。
//! 摊销倍数 = B/F = (P/M)·ε/(1−ε) = 3ε/(1−ε)。
//!
//! ## 失败条款（跑前写死）
//!
//! - 阳性对照：sorted_bplus 必须显著差于 logstruct_wb，否则测量没判别力 ⇒ 整轮作废。
//! - 自检（逐档跑）：B=1 时，**ε > 0.25 的每一档都必须显著变差**；
//!   **ε ≤ 0.25 的档必须几乎不变**——那一段解析式说本来就没摊销，变了反而说明模型错。
//! - 阴性对照：ε → 0 时 Bε 退化成「每条消息每层各一次触碰」。
//! - 绝对值断言：见文件末尾单测，钉 F/B/H、交点 0.25、摊销式 3ε/(1−ε)。
//! - 跨实验复现：logstruct_wb 的 io/op 必须复现 E7 的 1.5755（同为 1024 叶、wb 512、缓存 64/66 档）。
//!
//! ## 它答不了什么
//!
//! 不建模分裂/合并；不碰 D11 代价 2（可重建性退化）；不答 ε 是格式参数还是运行时参数；
//! 写放大按节点 I/O 次数与字节数计，不含 CPU；单盘、无并发、无崩溃。

use e7_index_bench::{Emitter, Lru};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名
const ALIGNMENT_BYTES: usize = 4096;
/// 节点 16 KiB：D8 已定项 2（2026-08-30 用户定案）钉成常量，不按设备特性算。
const NODE_BYTES: usize = 16384;
/// pivot 条目 = key 8 + D19 指针头部 40。
const PIVOT_BYTES: usize = 48;
/// 消息 / 叶条目 = key 8 + value 8。与 E7 同口径，两个实验的数才可比。
const MESSAGE_BYTES: usize = 16;
/// 节点头（条目数）占的字节。
const NODE_HEADER_BYTES: usize = 16;
/// 叶容量：整个节点都用来装条目。
const LEAF_ENTRY_CAPACITY: usize = (NODE_BYTES - NODE_HEADER_BYTES) / MESSAGE_BYTES;

/// 叶数是运行期参数（第 5 个命令行参数），默认 1024。
/// ⚠️ **它是自变量而不是常量，因为 2026-08-31 三条独立论证腿里有两条都把
/// 「树只有 1024 个叶、缓存占 6.4%」列为这批结论最大的外推威胁**——
/// 那件事量得出来，不该只靠嘴上让步。main 在构造任何 Geom 之前设定它一次，此后不再改。
static mut LEAF_COUNT_AT_RUNTIME: usize = 1024;
fn nleaf() -> usize {
    unsafe { LEAF_COUNT_AT_RUNTIME }
}
/// 叶按半满装（B 树的常规占用率），不是 E7 那种「16 KiB 节点里只放 64 个 key」。
const KEYS_PER_LEAF: usize = LEAF_ENTRY_CAPACITY / 2;
fn key_count() -> u64 {
    (nleaf() * KEYS_PER_LEAF) as u64
}

/// 负载次数。**运行期参数（第 6 个命令行参数）**，默认 20 万。
/// ⚠️ 它必须远大于总缓冲容量，否则量到的不是稳态——判据与实测见
/// `.claude/kb/experiments/56-消息缓冲的收益vsε.md`「稳态可用区」。
static mut OPERATION_COUNT_AT_RUNTIME: u64 = 200_000;
fn operation_count() -> u64 {
    unsafe { OPERATION_COUNT_AT_RUNTIME }
}
const POINT_QUERIES_PER_KEY_SET: u64 = 10_000;
const WRITE_BUFFER_CAPACITY: usize = 512;
const CHECKPOINT_INTERVAL: u64 = 4096;
/// 默认缓存节点数。E7 的 16 KiB 档用 66 个节点（约 1 MiB），这里沿用。
const DEFAULT_CACHE_NODE_COUNT: usize = 66;

/// ε 扫描点，千分之一为单位。0.25 是解析式给的交点，两侧都要有档。
const EPSILON_SWEEP_PERMILLE: [u32; 13] = [5, 50, 100, 150, 200, 250, 300, 400, 500, 650, 800, 900, 950];

// ── 几何：全部由 ε 导出 ──────────────────────────────────────────────

/// 一棵树的几何。`levels[0]` 是最靠近叶的那一层的节点数，最后一项恒为 1（根）。
#[derive(Debug, Clone, PartialEq)]
struct Geometry {
    epsilon_permille: u32,
    fanout: usize,
    buffer_capacity: usize,
    levels: Vec<usize>,
}

fn fanout_of(epsilon_permille: u32) -> usize {
    let pivot_bytes = NODE_BYTES * (1000 - epsilon_permille as usize) / 1000;
    (pivot_bytes / PIVOT_BYTES).max(2)
}

fn buffer_capacity_of(epsilon_permille: u32) -> usize {
    let buffer_capacity = NODE_BYTES * epsilon_permille as usize / 1000 / MESSAGE_BYTES;
    // 编码进一个节点：条目数 × 16 + 头 ≤ NODE_BYTES
    buffer_capacity.min(LEAF_ENTRY_CAPACITY)
}

/// 自底向上按扇出收敛，返回每一层的节点数，最后一项恒为 1（根）。
///
/// ⚠️ **进度不变量写在函数自己里，不是写在单测里。**
/// 2026-08-31 变异测试：把 `div_ceil` 换成整除之后 `nodes_in_level` 掉到 0，`nodes_in_level == 1` 永远不成立，
/// 循环无限往 `level_node_counts` 里塞 0 直到内存耗尽 ⇒ 测试进程被 signal 9 打死。
/// 那种「破坏被看见了，但不是断言抓到的」等于盲区：换个不 OOM 的同类错误就没人发现。
/// 断言放在单测里救不了它——函数根本不返回，单测里的断言执行不到。
fn levels_of(fanout: usize) -> Vec<usize> {
    assert!(fanout >= 2, "扇出必须 ≥ 2，否则树不收敛（实得 {fanout}）");
    let mut level_node_counts = Vec::new();
    let mut nodes_in_level = nleaf();
    loop {
        let previous_nodes_in_level = nodes_in_level;
        nodes_in_level = nodes_in_level.div_ceil(fanout);
        assert!(nodes_in_level >= 1 && nodes_in_level < previous_nodes_in_level, "层数没有收敛：{previous_nodes_in_level} -> {nodes_in_level}（扇出 {fanout}）");
        level_node_counts.push(nodes_in_level);
        if nodes_in_level == 1 {
            break;
        }
        assert!(level_node_counts.len() <= 64, "层数超过 64，几何一定算错了（扇出 {fanout}）");
    }
    level_node_counts
}

impl Geometry {
    fn new(epsilon_permille: u32) -> Self {
        let fanout = fanout_of(epsilon_permille);
        Self { epsilon_permille, fanout, buffer_capacity: buffer_capacity_of(epsilon_permille), levels: levels_of(fanout) }
    }
    fn height(&self) -> usize {
        self.levels.len()
    }
    fn total_internal(&self) -> usize {
        self.levels.iter().sum()
    }
    fn total_nodes(&self) -> usize {
        nleaf() + self.total_internal()
    }
    /// 解析预测：每条消息每层付 min(1, F/B) 次触碰，H 层合计。
    fn analytic_touches_per_operation(&self) -> f64 {
        let ratio = self.fanout as f64 / self.buffer_capacity.max(1) as f64;
        self.height() as f64 * ratio.min(1.0)
    }
    /// 摊销倍数 B/F。< 1 表示一条也没摊到。
    fn amortization(&self) -> f64 {
        self.buffer_capacity as f64 / self.fanout as f64
    }
    // 页号：叶 0..叶数，然后 level0、level1 …，根在最后。
    fn leaf_page_number(&self, leaf_index: usize) -> u64 {
        leaf_index as u64
    }
    fn level_base(&self, level: usize) -> u64 {
        nleaf() as u64 + self.levels[..level].iter().sum::<usize>() as u64
    }
    fn internal_page_number(&self, level: usize, node_index: usize) -> u64 {
        self.level_base(level) + node_index as u64
    }
    /// key 落在哪个叶。
    fn leaf_of(&self, key: u64) -> usize {
        (key / KEYS_PER_LEAF as u64) as usize
    }
    /// 第 l 层里，key 落在哪个节点。l=0 是最靠近叶的那层。
    fn node_of(&self, level: usize, key: u64) -> usize {
        let mut node_index = self.leaf_of(key);
        for _ in 0..=level {
            node_index /= self.fanout;
        }
        node_index
    }
    /// 内部节点的容量：只有缓冲那部分要落盘（pivot 是几何算出来的，不占模型里的条目槽）。
    fn internal_node_capacity(&self) -> usize {
        self.buffer_capacity.max(1)
    }
}

// ── 设备与节点 ────────────────────────────────────────────────────────

struct AlignedBuffer {
    pointer: *mut u8,
    length_in_bytes: usize,
    layout: Layout,
}
impl AlignedBuffer {
    fn new(length_in_bytes: usize) -> Self {
        let layout = Layout::from_size_align(length_in_bytes, ALIGNMENT_BYTES).unwrap();
        let pointer = unsafe { alloc(layout) };
        assert!(!pointer.is_null());
        unsafe { std::ptr::write_bytes(pointer, 0, length_in_bytes) };
        Self { pointer, length_in_bytes, layout }
    }
    fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) }
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) }
    }
}
impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer, self.layout) }
    }
}

#[derive(Clone)]
struct Node {
    entry_count: usize,
    entries: Vec<(u64, u64)>,
}
impl Node {
    fn new(capacity: usize) -> Self {
        Self { entry_count: 0, entries: vec![(0, 0); capacity] }
    }
    fn decode(buffer: &AlignedBuffer, capacity: usize) -> Self {
        let entry_count = (u64::from_le_bytes(buffer.bytes()[0..8].try_into().unwrap()) as usize).min(capacity);
        let mut entries = vec![(0u64, 0u64); capacity];
        for (entry_index, slot) in entries.iter_mut().enumerate().take(entry_count) {
            let entry_offset = NODE_HEADER_BYTES + entry_index * MESSAGE_BYTES;
            *slot = (
                u64::from_le_bytes(buffer.bytes()[entry_offset..entry_offset + 8].try_into().unwrap()),
                u64::from_le_bytes(buffer.bytes()[entry_offset + 8..entry_offset + 16].try_into().unwrap()),
            );
        }
        Self { entry_count, entries }
    }
    fn encode(&self, buffer: &mut AlignedBuffer) {
        buffer.bytes_mut()[0..8].copy_from_slice(&(self.entry_count as u64).to_le_bytes());
        for entry_index in 0..self.entry_count {
            let entry_offset = NODE_HEADER_BYTES + entry_index * MESSAGE_BYTES;
            buffer.bytes_mut()[entry_offset..entry_offset + 8].copy_from_slice(&self.entries[entry_index].0.to_le_bytes());
            buffer.bytes_mut()[entry_offset + 8..entry_offset + 16].copy_from_slice(&self.entries[entry_index].1.to_le_bytes());
        }
    }
    fn capacity(&self) -> usize {
        self.entries.len()
    }
    fn full(&self) -> bool {
        self.entry_count >= self.capacity()
    }
    fn push(&mut self, key: u64, value: u64) {
        if self.entry_count < self.capacity() {
            self.entries[self.entry_count] = (key, value);
            self.entry_count += 1;
        }
    }
    fn upsert(&mut self, key: u64, value: u64) {
        for entry_index in 0..self.entry_count {
            if self.entries[entry_index].0 == key {
                self.entries[entry_index].1 = value;
                return;
            }
        }
        self.push(key, value);
    }
    fn compact(&mut self) {
        bump_compaction();
        let mut latest_by_key: std::collections::BTreeMap<u64, u64> = Default::default();
        for entry_index in 0..self.entry_count {
            latest_by_key.insert(self.entries[entry_index].0, self.entries[entry_index].1);
        }
        self.entry_count = 0;
        for (key, value) in latest_by_key {
            self.push(key, value);
        }
    }
    fn get(&self, key: u64) -> Option<u64> {
        for entry_index in (0..self.entry_count).rev() {
            if self.entries[entry_index].0 == key {
                return Some(self.entries[entry_index].1);
            }
        }
        None
    }
}

struct Device {
    file: std::fs::File,
    reads: u64,
    writes: u64,
    bytes_read: u64,
    bytes_written: u64,
}
impl Device {
    fn read_page(&mut self, page_number: u64, buffer: &mut AlignedBuffer) {
        self.file.seek(SeekFrom::Start(page_number * NODE_BYTES as u64)).unwrap();
        self.file.read_exact(buffer.bytes_mut()).unwrap();
        self.reads += 1;
        self.bytes_read += NODE_BYTES as u64;
    }
    fn write_page(&mut self, page_number: u64, buffer: &AlignedBuffer) {
        self.file.seek(SeekFrom::Start(page_number * NODE_BYTES as u64)).unwrap();
        self.file.write_all(buffer.bytes()).unwrap();
        self.writes += 1;
        self.bytes_written += NODE_BYTES as u64;
    }
}

/// 节点缓存 + checkpoint：三条臂共用，摊销架构对称（E7 硬要求 2）。
struct Cache {
    capacity: usize,
    lru: Lru,
    map: std::collections::HashMap<u64, Node>,
}

/// 常驻集的历史最大值。**这是一条钉绝对值的读数**：它必须恰好等于
/// `min(声明的缓存节点数, 树的节点总数)`，三条臂各自都要满足——
/// 不是「三条臂互相比」（`.claude/singlefs-ai-sop/rules/test-discipline.md`）。
static mut RESIDENT_NODES_PEAK: usize = 0;
fn note_resident(resident_node_count: usize) {
    unsafe {
        if resident_node_count > RESIDENT_NODES_PEAK {
            RESIDENT_NODES_PEAK = resident_node_count;
        }
    }
}
fn resident_nodes_peak() -> usize {
    unsafe { RESIDENT_NODES_PEAK }
}
fn reset_resident() {
    unsafe { RESIDENT_NODES_PEAK = 0 }
}
impl Cache {
    fn new(capacity: usize) -> Self {
        Self { capacity, lru: Lru::new(capacity), map: Default::default() }
    }
    /// 取用一页。**`touch` 的返回值一律不许丢**——丢掉的那次逐出等于一次
    /// 「该写没写」，而它同时把被逐出的那一页变成 `map` 里 LRU 追踪不到的幽灵页，
    /// 幽灵页此后永远免费命中、还会繁殖出新的幽灵页。
    /// 2026-08-31 实测：丢返回值的写法下声明 66 个节点的缓存，
    /// 实际常驻集涨到 688（占树 67%），而三条臂互比看不出来——
    /// 只有 Bε 那条臂会在 get 与 put 之间嵌套别的缓存操作。
    fn get(&mut self, device: &mut Device, page_number: u64, node_capacity: usize, buffer: &mut AlignedBuffer) -> Node {
        if let Some(node) = self.map.get(&page_number).cloned() {
            let evicted_page = self.lru.touch(page_number);
            self.evict(device, buffer, evicted_page);
            self.check_residency();
            note_resident(self.map.len());
            return node;
        }
        let evicted_page = self.lru.touch(page_number);
        self.evict(device, buffer, evicted_page);
        device.read_page(page_number, buffer);
        let node = Node::decode(buffer, node_capacity);
        self.map.insert(page_number, node.clone());
        self.check_residency();
        note_resident(self.map.len());
        node
    }
    /// 写回一页。**它必须和 `get` 走同一条逐出路径**：`put` 不过 LRU 的话，
    /// 一个在递归期间被逐出的页会被重新塞回 `map` 而 LRU 不再追踪它。
    fn put(&mut self, device: &mut Device, buffer: &mut AlignedBuffer, page_number: u64, node: Node) {
        let evicted_page = self.lru.touch(page_number);
        self.evict(device, buffer, evicted_page);
        self.map.insert(page_number, node);
        self.lru.mark_dirty(page_number);
        self.check_residency();
        note_resident(self.map.len());
    }
    /// 常驻集不许超过声明的缓存。**三条臂共用这一条**，所以它不是臂间互比，
    /// 是把绝对值钉死（`.claude/singlefs-ai-sop/rules/test-discipline.md`）。
    fn check_residency(&self) {
        if self.map.len() > self.capacity {
            eprintln!("常驻集闸破了：常驻 {} 超过声明的缓存 {}", self.map.len(), self.capacity);
            std::process::exit(8);
        }
    }
    /// 逐出一页：脏就写回，然后从常驻集里摘掉。
    fn evict(&mut self, device: &mut Device, buffer: &mut AlignedBuffer, evicted_page: Option<u64>) {
        let Some(evicted_page) = evicted_page else { return };
        if self.lru.take_dirty(evicted_page) {
            if let Some(evicted_node) = self.map.get(&evicted_page).cloned() {
                evicted_node.encode(buffer);
                device.write_page(evicted_page, buffer);
            }
        }
        self.map.remove(&evicted_page);
    }
    fn checkpoint(&mut self, device: &mut Device, buffer: &mut AlignedBuffer) {
        for page_number in self.lru.drain_dirty() {
            if let Some(node) = self.map.get(&page_number) {
                node.encode(buffer);
                device.write_page(page_number, buffer);
            }
        }
    }
}

/// 正推腿（2026-08-31，盲预测，没看过任何实测）给了三条判别观测，这三个计数器就是它们的读数：
///   1. 叶压实全程触发几次——它预测 0；非 0 则它给的写次数全是低估。
///   2. 一次下刷命中几个孩子——它预测「全刷」形态下 ε=0.25 时每次约 162 个叶、
///      下刷事件约 781 次；若实现是「只刷最满的那个孩子」，则每次恰好 1 个、事件约 12.6 万次。
///      两个分布不可能互相误认。
///   3. 臂 B 的叶读次数与叶写次数之比——它预测 1.000（排序扫描命中率恰为 0）。
static mut COMPACTIONS: u64 = 0;
static mut FLUSH_EVENTS: u64 = 0;
static mut FLUSH_CHILD_TOUCHES: u64 = 0;
/// 守恒用：真正落到叶子上的消息条数。
/// **`landed + residual` 必须恰好等于操作数**——少一条就是有消息被静默丢了，
/// 而丢消息在 io/op 上表现为「更省」，是最坏的一类静默错误。
static mut LEAF_LANDED: u64 = 0;
fn bump_compaction() { unsafe { COMPACTIONS += 1 } }
fn bump_flush(children: u64) { unsafe { FLUSH_EVENTS += 1; FLUSH_CHILD_TOUCHES += children } }
fn bump_landed() { unsafe { LEAF_LANDED += 1 } }
fn landed() -> u64 { unsafe { LEAF_LANDED } }
fn reset_landed() { unsafe { LEAF_LANDED = 0 } }
fn take_counters() -> (u64, u64, u64) {
    unsafe {
        let counters = (COMPACTIONS, FLUSH_EVENTS, FLUSH_CHILD_TOUCHES);
        COMPACTIONS = 0; FLUSH_EVENTS = 0; FLUSH_CHILD_TOUCHES = 0;
        counters
    }
}

fn next_random(state: &mut u64) -> u64 {
    let mut mixed = *state;
    mixed ^= mixed >> 12;
    mixed ^= mixed << 25;
    mixed ^= mixed >> 27;
    *state = mixed;
    mixed.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// 把整棵树按给定几何铺到设备上。三条臂各自初始化，几何不同 ⇒ 页布局不同。
fn initialize_tree_on_device(device: &mut Device, geometry: &Geometry) {
    let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
    for leaf_index in 0..nleaf() {
        let mut leaf_node = Node::new(LEAF_ENTRY_CAPACITY);
        for key_in_leaf in 0..KEYS_PER_LEAF {
            let key = (leaf_index * KEYS_PER_LEAF + key_in_leaf) as u64;
            leaf_node.push(key, key);
        }
        leaf_node.encode(&mut page_buffer);
        device.write_page(geometry.leaf_page_number(leaf_index), &page_buffer);
    }
    let empty_internal_node = Node::new(geometry.internal_node_capacity());
    for level in 0..geometry.height() {
        for node_index in 0..geometry.levels[level] {
            empty_internal_node.encode(&mut page_buffer);
            device.write_page(geometry.internal_page_number(level, node_index), &page_buffer);
        }
    }
    device.reads = 0;
    device.writes = 0;
    device.bytes_read = 0;
    device.bytes_written = 0;
}

#[derive(Default, Clone)]
struct Run {
    reads: u64,
    writes: u64,
    bytes_read: u64,
    bytes_written: u64,
    touches: u64,
    elapsed_nanoseconds: u64,
    /// 收尾时还停在各层缓冲里、从没到过叶子的消息条数。
    /// ⚠️ **这一项是 2026-08-31 反推腿指出来的缺口**：不数它，高 ε 档的
    /// io/op 会把「还没付的账」算成「省下来的钱」——而基线臂的 write buffer
    /// 在每个 checkpoint 与收尾都清空，**两条臂的摊销架构因此不对称**
    /// （E7 硬要求 2 要的正是对称）。
    residual: u64,
    /// 把残留全部推到叶子所付的设备 I/O。
    drain_device_operations: u64,
    /// 块层独立读数，**只覆盖计费相**（预热相与数残留那两段不计）。
    block_layer_delta: String,
}

// ── 臂 1：无批量对照（阳性对照）。写穿，每次更新都落盘。
fn arm_sorted(device: &mut Device, geometry: &Geometry, operation_count: u64, seed: u64, cache_node_count: usize, touched: &mut [bool]) -> Run {
    let (reads_before, writes_before, bytes_read_before, bytes_written_before) = (device.reads, device.writes, device.bytes_read, device.bytes_written);
    let mut node_cache = Cache::new(cache_node_count);
    let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
    let mut random_state = seed;
    let started_at = Instant::now();
    for _ in 0..operation_count {
        let key = next_random(&mut random_state) % key_count();
        touched[key as usize] = true;
        let page_number = geometry.leaf_page_number(geometry.leaf_of(key));
        let mut leaf_node = node_cache.get(device, page_number, LEAF_ENTRY_CAPACITY, &mut page_buffer);
        leaf_node.upsert(key, key ^ 0xABCD);
        leaf_node.encode(&mut page_buffer);
        device.write_page(page_number, &page_buffer);
        node_cache.put(device, &mut page_buffer, page_number, leaf_node);
        node_cache.lru.take_dirty(page_number);
    }
    Run {
        reads: device.reads - reads_before,
        writes: device.writes - writes_before,
        bytes_read: device.bytes_read - bytes_read_before,
        bytes_written: device.bytes_written - bytes_written_before,
        touches: 0,
        elapsed_nanoseconds: started_at.elapsed().as_nanos() as u64,
        residual: 0,
        drain_device_operations: 0,
        block_layer_delta: String::new(),
    }
}

// ── 臂 2：D8 现方向 —— 日志结构节点 + write buffer 前端。
fn arm_logstruct_wb(device: &mut Device, geometry: &Geometry, operation_count: u64, seed: u64, cache_node_count: usize, write_buffer_capacity: usize, touched: &mut [bool]) -> Run {
    let (reads_before, writes_before, bytes_read_before, bytes_written_before) = (device.reads, device.writes, device.bytes_read, device.bytes_written);
    let mut node_cache = Cache::new(cache_node_count);
    let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
    let mut random_state = seed;
    let mut write_buffer: Vec<(u64, u64)> = Vec::with_capacity(write_buffer_capacity.max(1));
    let mut touches = 0u64;
    let started_at = Instant::now();
    let flush = |write_buffer: &mut Vec<(u64, u64)>, node_cache: &mut Cache, device: &mut Device, page_buffer: &mut AlignedBuffer, touches: &mut u64| {
        if write_buffer.is_empty() {
            return;
        }
        // 稳定排序：unstable 会打乱同 key 的相对次序，于是「后者胜」胜出的是任意一条。
        // D8 已定项 3 要的 seq 正是为了这件事，本 harness 用稳定排序代替。
        write_buffer.sort_by_key(|entry| (entry.0 / KEYS_PER_LEAF as u64, entry.0));
        let mut deduplicated_entries: Vec<(u64, u64)> = Vec::with_capacity(write_buffer.len());
        for &(key, value) in write_buffer.iter() {
            if let Some(last) = deduplicated_entries.last_mut() {
                if last.0 == key {
                    last.1 = value;
                    continue;
                }
            }
            deduplicated_entries.push((key, value));
        }
        let mut deduplicated_entry_index = 0;
        while deduplicated_entry_index < deduplicated_entries.len() {
            let leaf_index = (deduplicated_entries[deduplicated_entry_index].0 / KEYS_PER_LEAF as u64) as usize;
            let page_number = leaf_index as u64;
            let mut leaf_node = node_cache.get(device, page_number, LEAF_ENTRY_CAPACITY, page_buffer);
            *touches += 1;
            while deduplicated_entry_index < deduplicated_entries.len() && (deduplicated_entries[deduplicated_entry_index].0 / KEYS_PER_LEAF as u64) as usize == leaf_index {
                if leaf_node.full() {
                    leaf_node.compact();
                }
                leaf_node.push(deduplicated_entries[deduplicated_entry_index].0, deduplicated_entries[deduplicated_entry_index].1);
                deduplicated_entry_index += 1;
            }
            node_cache.put(device, page_buffer, page_number, leaf_node);
        }
        write_buffer.clear();
    };
    for operation_index in 0..operation_count {
        let key = next_random(&mut random_state) % key_count();
        touched[key as usize] = true;
        write_buffer.push((key, key ^ 0xABCD));
        if write_buffer.len() >= write_buffer_capacity.max(1) {
            flush(&mut write_buffer, &mut node_cache, device, &mut page_buffer, &mut touches);
        }
        if (operation_index + 1) % CHECKPOINT_INTERVAL == 0 {
            flush(&mut write_buffer, &mut node_cache, device, &mut page_buffer, &mut touches);
            node_cache.checkpoint(device, &mut page_buffer);
        }
    }
    flush(&mut write_buffer, &mut node_cache, device, &mut page_buffer, &mut touches);
    node_cache.checkpoint(device, &mut page_buffer);
    let _ = geometry;
    Run {
        reads: device.reads - reads_before,
        writes: device.writes - writes_before,
        bytes_read: device.bytes_read - bytes_read_before,
        bytes_written: device.bytes_written - bytes_written_before,
        touches,
        elapsed_nanoseconds: started_at.elapsed().as_nanos() as u64,
        residual: 0,
        drain_device_operations: 0,
        block_layer_delta: String::new(),
    }
}

// ── 臂 3：Bε —— 内部节点留 ε 比例的消息缓冲，消息沿树逐层下刷。
//    缓冲条数与扇出都由 ε 导出，两者争同一块节点字节 —— 这正是 E7 缺的那条张力。
#[allow(clippy::too_many_arguments)]
fn arm_betree(
    device: &mut Device,
    geometry: &Geometry,
    operation_count: u64,
    seed: u64,
    cache_node_count: usize,
    buffer_capacity: usize,
    touched: &mut [bool],
    warmup: u64,
    device_path: &str,
) -> Run {
    let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
    let mut random_state = seed;
    let internal_node_capacity = buffer_capacity.max(1).min(LEAF_ENTRY_CAPACITY);
    // ── 预热相：先把各层缓冲填到稳态，**这一段的 I/O 不计费**。
    // ⚠️ 补它的理由（2026-08-31）：不预热的话各层缓冲一开始全空，进每个节点的前 B 条
    // 消息都是免费的，而**总缓冲容量可以超过整个负载**——实测 nleaf=8192、ε=0.95 时
    // 总缓冲容量约 50 万条 > 20 万次操作 ⇒ 残留 100%，一条消息都没到过叶子，
    // 量到的不是稳态写放大，是「把消息塞进缓冲的成本」，账还没到期。
    {
        let mut warmup_cache = Cache::new(cache_node_count);
        let mut discarded_touches = 0u64;
        for _ in 0..warmup {
            let key = next_random(&mut random_state) % key_count();
            touched[key as usize] = true;
            push_down(device, &mut warmup_cache, &mut page_buffer, geometry, geometry.height() - 1, 0, vec![(key, key ^ 0xABCD)], internal_node_capacity, &mut discarded_touches);
        }
        warmup_cache.checkpoint(device, &mut page_buffer);
    }
    let residual_before_measurement = count_residual(device, geometry, cache_node_count, internal_node_capacity);
    // 预热与数残留的 I/O 一律抠掉：从这里开始才计费
    let (reads_before, writes_before, bytes_read_before, bytes_written_before) = (device.reads, device.writes, device.bytes_read, device.bytes_written);
    // ⚠️ **块层采样点必须在预热之后。** 采在外面会把不计费的预热相与数残留那两段
    // 算进块层那一侧，于是「程序计数器 = 块层读数」这条校验退化成「差不多」——
    // 2026-08-31 虚机首跑实测差 1991 读（ε=0.5），正是那两段。
    let block_counters_before_measurement = read_block_layer_counters(device_path);
    let mut node_cache = Cache::new(cache_node_count);
    let mut touches = 0u64;
    reset_landed();
    let started_at = Instant::now();
    for operation_index in 0..operation_count {
        let key = next_random(&mut random_state) % key_count();
        touched[key as usize] = true;
        push_down(device, &mut node_cache, &mut page_buffer, geometry, geometry.height() - 1, 0, vec![(key, key ^ 0xABCD)], internal_node_capacity, &mut touches);
        if (operation_index + 1) % CHECKPOINT_INTERVAL == 0 {
            node_cache.checkpoint(device, &mut page_buffer);
        }
    }
    node_cache.checkpoint(device, &mut page_buffer);
    let block_layer_delta = block_counter_delta(block_counters_before_measurement, read_block_layer_counters(device_path));
    Run {
        reads: device.reads - reads_before,
        writes: device.writes - writes_before,
        bytes_read: device.bytes_read - bytes_read_before,
        bytes_written: device.bytes_written - bytes_written_before,
        touches,
        elapsed_nanoseconds: started_at.elapsed().as_nanos() as u64,
        residual: residual_before_measurement,
        drain_device_operations: 0,
        block_layer_delta,
    }
}

/// 只数各层缓冲里现有多少条消息，不动手推。**调用方负责把它的 I/O 从计费里抠掉。**
fn count_residual(device: &mut Device, geometry: &Geometry, cache_node_count: usize, internal_node_capacity: usize) -> u64 {
    let mut node_cache = Cache::new(cache_node_count);
    let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
    let mut residual_messages = 0u64;
    for level in 0..geometry.height() {
        for node_index in 0..geometry.levels[level] {
            residual_messages += node_cache.get(device, geometry.internal_page_number(level, node_index), internal_node_capacity, &mut page_buffer).entry_count as u64;
        }
    }
    residual_messages
}

/// 收尾排空相：全新缓存，从盘上的状态把各层缓冲推到叶子，单独计费。
/// 用全新缓存是**保守**的——冷缓存下排空要多付读，给出的是上界。
fn drain_phase(device: &mut Device, geometry: &Geometry, cache_node_count: usize, internal_node_capacity: usize) -> (u64, u64) {
    let mut node_cache = Cache::new(cache_node_count);
    let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
    let mut touches = 0u64;
    let (reads_before, writes_before) = (device.reads, device.writes);
    let residual = drain_all(device, &mut node_cache, &mut page_buffer, geometry, internal_node_capacity, &mut touches);
    node_cache.checkpoint(device, &mut page_buffer);
    (residual, (device.reads - reads_before) + (device.writes - writes_before))
}

/// 把一批消息塞进第 `level` 层第 `node_index` 个节点的缓冲；满了就按孩子分组下刷。
/// `level == 0` 的孩子是叶。每次取用一个节点记一次 touch。
#[allow(clippy::too_many_arguments)]
fn push_down(
    device: &mut Device,
    node_cache: &mut Cache,
    page_buffer: &mut AlignedBuffer,
    geometry: &Geometry,
    level: usize,
    node_index: usize,
    messages: Vec<(u64, u64)>,
    internal_node_capacity: usize,
    touches: &mut u64,
) {
    let page_number = geometry.internal_page_number(level, node_index);
    let mut node = node_cache.get(device, page_number, internal_node_capacity, page_buffer);
    *touches += 1;
    // ⚠️ **按「装到满就立刻下刷」分块装，不许一口气全塞进去。**
    // 一口气塞的写法在缓冲恰好装满时会**静默丢消息**（`Node::push` 满了就什么也不做），
    // 而内部缓冲里的 key 来自整个子树、重复率约 0.09%，压实几乎腾不出槽位。
    // 2026-08-31 由正推腿的「叶压实该是 0 次」这条盲预测查出来：
    // 实测压实计数非 0，追下去发现计的是**内部缓冲**的压实，而那正是丢消息的现场。
    let mut message_index = 0usize;
    while message_index < messages.len() {
        if node.entry_count >= internal_node_capacity {
            flush_node(device, node_cache, page_buffer, geometry, level, page_number, &mut node, internal_node_capacity, touches);
        }
        let free_slot_count = internal_node_capacity - node.entry_count;
        let message_take_count = (messages.len() - message_index).min(free_slot_count);
        for message in &messages[message_index..message_index + message_take_count] {
            node.push(message.0, message.1);
        }
        message_index += message_take_count;
        if node.entry_count >= internal_node_capacity {
            flush_node(device, node_cache, page_buffer, geometry, level, page_number, &mut node, internal_node_capacity, touches);
        }
    }
    node_cache.put(device, page_buffer, page_number, node);
}

/// 把一个装满的缓冲整批推给孩子，推完把它清空。
/// **内部缓冲不做压实**：清空由下刷负责，压实只属于叶。
#[allow(clippy::too_many_arguments)]
fn flush_node(
    device: &mut Device,
    node_cache: &mut Cache,
    page_buffer: &mut AlignedBuffer,
    geometry: &Geometry,
    level: usize,
    page_number: u64,
    node: &mut Node,
    internal_node_capacity: usize,
    touches: &mut u64,
) {
    let flushed_messages: Vec<(u64, u64)> = node.entries[..node.entry_count].to_vec();
    node.entry_count = 0;
    node_cache.put(device, page_buffer, page_number, node.clone());
    let mut messages_by_child: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = Default::default();
    for (key, value) in flushed_messages {
        let child = if level == 0 { geometry.leaf_of(key) } else { geometry.node_of(level - 1, key) };
        messages_by_child.entry(child).or_default().push((key, value));
    }
    bump_flush(messages_by_child.len() as u64);
    for (child, child_messages) in messages_by_child {
        if level == 0 {
            let leaf_page_number = geometry.leaf_page_number(child);
            let mut leaf = node_cache.get(device, leaf_page_number, LEAF_ENTRY_CAPACITY, page_buffer);
            *touches += 1;
            for (key, value) in child_messages {
                if leaf.full() {
                    leaf.compact();
                }
                leaf.push(key, value);
                bump_landed();
            }
            node_cache.put(device, page_buffer, leaf_page_number, leaf);
        } else {
            push_down(device, node_cache, page_buffer, geometry, level - 1, child, child_messages, internal_node_capacity, touches);
        }
    }
}

/// 收尾排空：自顶向下把每一层缓冲里剩下的消息全部推到叶子。
/// 返回排空前还停在缓冲里的消息条数。
/// **它存在的理由是让两条攒批臂的摊销架构对称**：基线臂的 write buffer 在收尾时是空的，
/// 而 Bε 的各层缓冲不排空的话，高 ε 档等于有一大批更新的成本被实验窗口切掉了没算。
fn drain_all(
    device: &mut Device,
    node_cache: &mut Cache,
    page_buffer: &mut AlignedBuffer,
    geometry: &Geometry,
    internal_node_capacity: usize,
    touches: &mut u64,
) -> u64 {
    // ⚠️ **残留必须在动手推之前数完，不能边推边数。**
    // 边推边数会漏掉一类消息：上层排空时把消息推进下层，下层缓冲因此溢出、当场
    // 直接刷到叶子，等外层循环走到那一层时它们已经不在了 —— 于是 landed 计到、
    // residual 没计到。2026-08-31 守恒闸第一次真跑就是被这一条判红的
    // （landed_update=199869 residual=109，差 22 条）。
    let mut residual = 0u64;
    for level in 0..geometry.height() {
        for node_index in 0..geometry.levels[level] {
            let node = node_cache.get(device, geometry.internal_page_number(level, node_index), internal_node_capacity, page_buffer);
            residual += node.entry_count as u64;
        }
    }
    for level in (0..geometry.height()).rev() {
        for node_index in 0..geometry.levels[level] {
            let page_number = geometry.internal_page_number(level, node_index);
            let mut node = node_cache.get(device, page_number, internal_node_capacity, page_buffer);
            *touches += 1;
            if node.entry_count == 0 {
                continue;
            }
            let flushed_messages: Vec<(u64, u64)> = node.entries[..node.entry_count].to_vec();
            node.entry_count = 0;
            node_cache.put(device, page_buffer, page_number, node);
            let mut messages_by_child: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = Default::default();
            for (key, value) in flushed_messages {
                let child = if level == 0 { geometry.leaf_of(key) } else { geometry.node_of(level - 1, key) };
                messages_by_child.entry(child).or_default().push((key, value));
            }
            for (child, child_messages) in messages_by_child {
                if level == 0 {
                    let leaf_page_number = geometry.leaf_page_number(child);
                    let mut leaf = node_cache.get(device, leaf_page_number, LEAF_ENTRY_CAPACITY, page_buffer);
                    *touches += 1;
                    for (key, value) in child_messages {
                        if leaf.full() {
                            leaf.compact();
                        }
                        leaf.push(key, value);
                        bump_landed();
                    }
                    node_cache.put(device, page_buffer, leaf_page_number, leaf);
                } else {
                    push_down(device, node_cache, page_buffer, geometry, level - 1, child, child_messages, internal_node_capacity, touches);
                }
            }
        }
    }
    residual
}

/// 点查：根 → … → 叶，逐层看该 key 在不在这一层的缓冲里。每层一次节点读。
fn point_query(device: &mut Device, node_cache: &mut Cache, page_buffer: &mut AlignedBuffer, geometry: &Geometry, internal_node_capacity: usize, key: u64) -> u64 {
    let reads_before = device.reads;
    for level in (0..geometry.height()).rev() {
        let node = node_cache.get(device, geometry.internal_page_number(level, geometry.node_of(level, key)), internal_node_capacity, page_buffer);
        if node.get(key).is_some() {
            return device.reads - reads_before;
        }
    }
    let _ = node_cache.get(device, geometry.leaf_page_number(geometry.leaf_of(key)), LEAF_ENTRY_CAPACITY, page_buffer);
    device.reads - reads_before
}

#[derive(Default, Clone, Copy)]
struct BlockLayerCounters {
    reads_completed: u64,
    writes_completed: u64,
}
fn read_block_layer_counters(device_path: &str) -> Option<BlockLayerCounters> {
    let device_name = device_path.rsplit('/').next()?;
    let sysfs_counter_text = std::fs::read_to_string(format!("/sys/block/{device_name}/stat")).ok()?;
    let sysfs_counter_fields: Vec<u64> = sysfs_counter_text.split_whitespace().filter_map(|field_text| field_text.parse().ok()).collect();
    if sysfs_counter_fields.len() < 8 {
        return None;
    }
    Some(BlockLayerCounters { reads_completed: sysfs_counter_fields[0], writes_completed: sysfs_counter_fields[4] })
}
fn block_counter_delta(block_counters_before: Option<BlockLayerCounters>, block_counters_after: Option<BlockLayerCounters>) -> String {
    match (block_counters_before, block_counters_after) {
        (Some(block_counters_before), Some(block_counters_after)) => format!("blk_r={} blk_w={}", block_counters_after.reads_completed - block_counters_before.reads_completed, block_counters_after.writes_completed - block_counters_before.writes_completed),
        _ => "blk_r=NA blk_w=NA".into(),
    }
}

/// 查询阶段：热 key（更新阶段碰过的）与冷 key（没碰过的）各一组，缓存都是新的。
/// ⚠️ 冷 key 那一列正是 E7 自陈没量的那一格。
fn query_phase(
    device: &mut Device,
    geometry: &Geometry,
    cache_node_count: usize,
    internal_node_capacity: usize,
    touched: &[bool],
    seed: u64,
) -> (f64, f64, u64, u64) {
    let hot_keys: Vec<u64> = (0..key_count()).filter(|&key| touched[key as usize]).collect();
    let cold_keys: Vec<u64> = (0..key_count()).filter(|&key| !touched[key as usize]).collect();
    let mut reads_per_query = [0f64; 2];
    let mut key_set_sizes = [0u64; 2];
    for (key_set_index, key_set) in [&hot_keys, &cold_keys].iter().enumerate() {
        let mut node_cache = Cache::new(cache_node_count);
        let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
        let mut random_state = seed ^ (0x9E37 + key_set_index as u64);
        let reads_before = device.reads;
        if key_set.is_empty() {
            reads_per_query[key_set_index] = f64::NAN;
            continue;
        }
        for _ in 0..POINT_QUERIES_PER_KEY_SET {
            let key = key_set[(next_random(&mut random_state) % key_set.len() as u64) as usize];
            let _ = point_query(device, &mut node_cache, &mut page_buffer, geometry, internal_node_capacity, key);
        }
        reads_per_query[key_set_index] = (device.reads - reads_before) as f64 / POINT_QUERIES_PER_KEY_SET as f64;
        key_set_sizes[key_set_index] = key_set.len() as u64;
    }
    (reads_per_query[0], reads_per_query[1], key_set_sizes[0], key_set_sizes[1])
}

fn main() {
    let device_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e56_epsilon <块设备或文件> [种子] [none|selfcheck|nodirect] [缓存节点数] [叶数] [负载次数]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(0x51561234);
    let mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    let is_selfcheck = mode == "selfcheck";
    let opens_device_with_direct_flag = mode != "nodirect";
    let cache_node_count: usize = std::env::args().nth(4).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(DEFAULT_CACHE_NODE_COUNT);
    // 第 5 个参数：叶数。**在构造任何 Geom 之前设定，此后不再改。**
    let leaf_count_argument: usize = std::env::args().nth(5).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(1024);
    unsafe { LEAF_COUNT_AT_RUNTIME = leaf_count_argument.max(2) };
    // 第 6 个参数：负载次数。必须远大于总缓冲容量，见「稳态可用区」。
    let operation_count_argument: u64 = std::env::args().nth(6).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(200_000);
    unsafe { OPERATION_COUNT_AT_RUNTIME = operation_count_argument.max(1) };

    let mut open_options = OpenOptions::new();
    open_options.read(true).write(true);
    if opens_device_with_direct_flag {
        open_options.custom_flags(O_DIRECT);
    }
    let file = open_options.open(&device_path).unwrap_or_else(|error| {
        eprintln!("打不开 {device_path}: {error}");
        std::process::exit(3)
    });
    let mut device = Device { file, reads: 0, writes: 0, bytes_read: 0, bytes_written: 0 };
    let device_size_bytes = device.file.seek(SeekFrom::End(0)).unwrap();
    // 最大的那套几何（ε 最小 ⇒ 扇出最大 ⇒ 内部节点最少；ε 最大 ⇒ 内部最多）都要装得下
    let required_device_bytes = EPSILON_SWEEP_PERMILLE.iter().map(|&epsilon_permille| Geometry::new(epsilon_permille).total_nodes()).max().unwrap()
        .max(Geometry::new(0).total_nodes()) as u64
        * NODE_BYTES as u64;
    if device_size_bytes < required_device_bytes {
        eprintln!("设备太小：需要 {required_device_bytes} 字节，实有 {device_size_bytes}");
        std::process::exit(5);
    }

    let mut emitter = Emitter::new();
    let base_geometry = Geometry::new(0);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config node_bytes={NODE_BYTES} pivot_bytes={PIVOT_BYTES} msg_bytes={MESSAGE_BYTES} \
             leaf_cap={LEAF_ENTRY_CAPACITY} keys_per_leaf={KEYS_PER_LEAF} n_leaf={} keys={} \
             ops={} qn={POINT_QUERIES_PER_KEY_SET} wb_cap={WRITE_BUFFER_CAPACITY} ckpt={CHECKPOINT_INTERVAL} cache_nodes={cache_node_count} \
             base_fanout={} base_height={} base_nodes={} cache_ratio={:.4} \
             seed={seed} mode={mode} o_direct={opens_device_with_direct_flag} blkstat={}",
            nleaf(),
            key_count(),
            operation_count(),
            base_geometry.fanout,
            base_geometry.height(),
            base_geometry.total_nodes(),
            cache_node_count as f64 / base_geometry.total_nodes() as f64,
            read_block_layer_counters(&device_path).is_some()
        ))
    );

    let mut touched = vec![false; key_count() as usize];

    // 基线两条臂用 ε=0 的几何：不留缓冲，节点字节全给 pivot。
    for (arm_name, arm_index) in [("sorted_bplus", 0u8), ("logstruct_wb", 1u8)] {
        touched.iter_mut().for_each(|touched_flag| *touched_flag = false);
        initialize_tree_on_device(&mut device, &base_geometry);
        let block_counters_before = read_block_layer_counters(&device_path);
        let run_result = if arm_index == 0 {
            arm_sorted(&mut device, &base_geometry, operation_count(), seed, cache_node_count, &mut touched)
        } else {
            arm_logstruct_wb(&mut device, &base_geometry, operation_count(), seed, cache_node_count, if is_selfcheck { 1 } else { WRITE_BUFFER_CAPACITY }, &mut touched)
        };
        let block_layer_delta = block_counter_delta(block_counters_before, read_block_layer_counters(&device_path));
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=update arm={arm_name} eps=0 fanout={} buf=0 height={} reads={} writes={} \
                 io={} io_per_op={:.6} bytes_per_op={:.1} touch_per_op={:.6} analytic_touch=NA elapsed_ns={} {block_layer_delta}",
                base_geometry.fanout,
                base_geometry.height(),
                run_result.reads,
                run_result.writes,
                run_result.reads + run_result.writes,
                (run_result.reads + run_result.writes) as f64 / operation_count() as f64,
                (run_result.bytes_read + run_result.bytes_written) as f64 / operation_count() as f64,
                run_result.touches as f64 / operation_count() as f64,
                run_result.elapsed_nanoseconds
            ))
        );
        let block_counters_before_query = read_block_layer_counters(&device_path);
        let (hot_reads_per_query, cold_reads_per_query, hot_key_count, cold_key_count) = query_phase(&mut device, &base_geometry, cache_node_count, base_geometry.internal_node_capacity(), &touched, seed);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=query arm={arm_name} eps=0 hot_reads_per_query={hot_reads_per_query:.4} cold_reads_per_query={cold_reads_per_query:.4} \
                 hot_keys={hot_key_count} cold_keys={cold_key_count} {}",
                block_counter_delta(block_counters_before_query, read_block_layer_counters(&device_path))
            ))
        );
    }

    // Bε 臂：ε 扫描。selfcheck 把缓冲判据换成常量 1。
    for epsilon_permille in EPSILON_SWEEP_PERMILLE {
        let geometry = Geometry::new(epsilon_permille);
        let internal_node_capacity = if is_selfcheck { 1 } else { geometry.internal_node_capacity() };
        touched.iter_mut().for_each(|touched_flag| *touched_flag = false);
        initialize_tree_on_device(&mut device, &geometry);
        let _ = take_counters();
        reset_landed();
        reset_resident();
        // 预热到稳态：把各层缓冲填满两遍，让下刷级联在每一层都真的跑过几轮。
        // 判据是「填满该臂自己的缓冲」而不是「跑固定次数」——稳态是状态，不是操作数。
        let warmup = (geometry.total_internal() as u64) * (internal_node_capacity as u64) * 2;
        let mut run_result = arm_betree(&mut device, &geometry, operation_count(), seed, cache_node_count, internal_node_capacity, &mut touched, warmup, &device_path);
        let residual_before_measurement = run_result.residual;
        let block_layer_delta = run_result.block_layer_delta.clone();
        let counters = take_counters();
        let landed_update = landed();
        // 排空前先量一次点查：这是「缓冲里还有在途消息」的树态
        let (predrain_hot_reads_per_query, predrain_cold_reads_per_query, _, _) = query_phase(&mut device, &geometry, cache_node_count, internal_node_capacity, &touched, seed);
        // 再排空，单独计费（反推腿指出的缺口）
        let (residual, drain_device_operations) = drain_phase(&mut device, &geometry, cache_node_count, internal_node_capacity);
        run_result.residual = residual;
        run_result.drain_device_operations = drain_device_operations;
        // 守恒闸：落到叶的消息数加上排空时从缓冲里捡出来的，必须恰好等于操作数。
        // 少一条就是有消息被静默丢了——而丢消息在 io/op 上长得像「更省」。
        // ⚠️ 这道闸是补的：2026-08-31 之前没有它，`push_down` 在缓冲恰好装满时
        // 静默丢消息，而三条臂互比看不出来（丢消息只让 Bε 那条臂显得更便宜）。
        let expected_resident_node_count = cache_node_count.min(geometry.total_nodes());
        if resident_nodes_peak() > expected_resident_node_count {
            eprintln!(
                "常驻集闸破了：eps={epsilon_permille} resident_max={} 声明缓存={cache_node_count} 树节点={} 该是 {expected_resident_node_count}",
                resident_nodes_peak(),
                geometry.total_nodes()
            );
            std::process::exit(8);
        }
        // 守恒（带预热的形态）：计费相进树的消息数 = ops()；
        // 落到叶的 + 缓冲净增 必须恰好等于它：
        //   landed_计费相 + (residual_post − residual_before_measurement) == ops()
        if landed_update + residual < residual_before_measurement
            || landed_update + residual - residual_before_measurement != operation_count()
        {
            eprintln!(
                "守恒破了：eps={epsilon_permille} landed_update={landed_update} residual_pre={residual_before_measurement} residual_post={residual} ops={}", operation_count()
            );
            std::process::exit(7);
        }
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=update arm=betree eps={epsilon_permille} fanout={} buf={} height={} internal_nodes={} \
                 reads={} writes={} io={} io_per_op={:.6} bytes_per_op={:.1} touch_per_op={:.6} \
                 analytic_touch={:.6} amortization={:.4} residual={} residual_frac={:.4} \
                 drain_io={} io_per_op_drained={:.6} residual_pre={residual_before_measurement} warmup={warmup} \
                 leaf_compactions={} flush_events={} \
                 children_per_flush={:.4} landed_update={landed_update} conservation=ok \
                 elapsed_ns={} {block_layer_delta}",
                geometry.fanout,
                internal_node_capacity,
                geometry.height(),
                geometry.total_internal(),
                run_result.reads,
                run_result.writes,
                run_result.reads + run_result.writes,
                (run_result.reads + run_result.writes) as f64 / operation_count() as f64,
                (run_result.bytes_read + run_result.bytes_written) as f64 / operation_count() as f64,
                run_result.touches as f64 / operation_count() as f64,
                geometry.analytic_touches_per_operation(),
                geometry.amortization(),
                run_result.residual,
                run_result.residual as f64 / operation_count() as f64,
                run_result.drain_device_operations,
                (run_result.reads + run_result.writes + run_result.drain_device_operations) as f64 / operation_count() as f64,
                counters.0,
                counters.1,
                if counters.1 == 0 { 0.0 } else { counters.2 as f64 / counters.1 as f64 },
                run_result.elapsed_nanoseconds
            ))
        );
        let block_counters_before_query = read_block_layer_counters(&device_path);
        let (hot_reads_per_query, cold_reads_per_query, hot_key_count, cold_key_count) = query_phase(&mut device, &geometry, cache_node_count, internal_node_capacity, &touched, seed);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=query arm=betree eps={epsilon_permille} height={} hot_reads_per_query={hot_reads_per_query:.4} \
                 cold_reads_per_query={cold_reads_per_query:.4} predrain_hot={predrain_hot_reads_per_query:.4} predrain_cold={predrain_cold_reads_per_query:.4} \
                 hot_keys={hot_key_count} cold_keys={cold_key_count} {}",
                geometry.height(),
                block_counter_delta(block_counters_before_query, read_block_layer_counters(&device_path))
            ))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言 1：几何常量。** 它们一改，下面每一条解析值都静默作废，
    /// 而 D11 引的数正是这套算术算出来的（E7 的教训：几何常量此前没有任何断言钉住）。
    #[test]
    fn geometry_constants_are_pinned() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2 钉的是 16 KiB");
        assert_eq!(PIVOT_BYTES, 48, "key 8 + D19 指针头部 40");
        assert_eq!(MESSAGE_BYTES, 16, "key 8 + value 8，与 E7 同口径");
        assert_eq!(LEAF_ENTRY_CAPACITY, 1023);
        assert_eq!(KEYS_PER_LEAF, 511, "叶按半满装");
        assert_eq!(nleaf(), 1024, "叶数默认值");
        assert_eq!(key_count(), 523_264);
    }

    /// **绝对值断言 2：交点恰好落在 ε = M/(M+P) = 0.25，且此处 B == F。**
    /// 这是跑前写死的预测 1，实测若与它相左，要么模型没按几何算、要么解析错。
    #[test]
    fn the_crossover_is_exactly_at_eps_one_quarter() {
        let analytic_crossover_epsilon = MESSAGE_BYTES as f64 / (MESSAGE_BYTES + PIVOT_BYTES) as f64;
        assert!((analytic_crossover_epsilon - 0.25).abs() < 1e-12, "解析交点算出 {analytic_crossover_epsilon}");
        let geometry = Geometry::new(250);
        assert_eq!(geometry.fanout, 256, "ε=0.25 的扇出");
        assert_eq!(geometry.buffer_capacity, 256, "ε=0.25 的缓冲条数");
        assert_eq!(geometry.buffer_capacity, geometry.fanout, "交点处 B 必须恰好等于 F");
        assert!((geometry.amortization() - 1.0).abs() < 1e-12, "交点处摊销倍数恰为 1");
    }

    /// **绝对值断言 3：摊销倍数 = (P/M)·ε/(1−ε) = 3ε/(1−ε)。**
    /// 逐档与几何算出来的 B/F 对，容差只留给整除的取整。
    #[test]
    fn amortization_follows_three_eps_over_one_minus_eps() {
        for epsilon_permille in [100u32, 250, 500, 750, 900] {
            let geometry = Geometry::new(epsilon_permille);
            let epsilon_fraction = epsilon_permille as f64 / 1000.0;
            let closed_form_amortization = 3.0 * epsilon_fraction / (1.0 - epsilon_fraction);
            let relative_error = (geometry.amortization() - closed_form_amortization).abs() / closed_form_amortization;
            assert!(relative_error < 0.02, "ε={epsilon_fraction}：几何给 {}，闭式给 {closed_form_amortization}", geometry.amortization());
        }
    }

    /// **绝对值断言 4：逐档钉死 F / B / H 与解析触碰数。**
    /// 表里每一行都是手算的，不是从代码抄回来的。
    #[test]
    fn per_eps_geometry_matches_hand_computed_table() {
        // (ε‰, F, B, H, touch_per_op)
        let table = [
            (5u32, 339usize, 5usize, 2usize, 2.0f64),
            (50, 324, 51, 2, 2.0),
            (200, 273, 204, 2, 2.0),
            (250, 256, 256, 2, 2.0),
            (500, 170, 512, 2, 2.0 * 170.0 / 512.0),
            (800, 68, 819, 2, 2.0 * 68.0 / 819.0),
            (900, 34, 921, 2, 2.0 * 34.0 / 921.0),
            (950, 17, 972, 3, 3.0 * 17.0 / 972.0),
        ];
        for (epsilon_permille, expected_fanout, expected_buffer_capacity, expected_height, expected_touches) in table {
            let geometry = Geometry::new(epsilon_permille);
            assert_eq!(geometry.fanout, expected_fanout, "ε={epsilon_permille}‰ 的扇出");
            assert_eq!(geometry.buffer_capacity, expected_buffer_capacity, "ε={epsilon_permille}‰ 的缓冲条数");
            assert_eq!(geometry.height(), expected_height, "ε={epsilon_permille}‰ 的层数");
            assert!((geometry.analytic_touches_per_operation() - expected_touches).abs() < 1e-9, "ε={epsilon_permille}‰ 的解析触碰数");
        }
    }

    /// **ε ≤ 0.25 的那一段，解析触碰数恒等于层数**——「一条也没摊到」的可判定形态。
    /// 自检（B=1）在这一段**预期不变**，正是因为这里本来就没有摊销。
    #[test]
    fn below_the_crossover_there_is_no_amortization_at_all() {
        for epsilon_permille in [5u32, 50, 100, 150, 200, 250] {
            let geometry = Geometry::new(epsilon_permille);
            assert!(geometry.amortization() <= 1.0, "ε={epsilon_permille}‰ 不该有摊销");
            assert!(
                (geometry.analytic_touches_per_operation() - geometry.height() as f64).abs() < 1e-12,
                "ε={epsilon_permille}‰ 的触碰数该恰好等于层数 {}",
                geometry.height()
            );
        }
    }

    /// **层数序列必须严格收敛到 1，每层至少 1 个节点。**
    /// ⚠️ 补这一条的理由（2026-08-31 变异测试）：把 `div_ceil` 换成整除的那条变异
    /// （M3）只让测试进程被 signal 9 打死——`nodes_in_level` 掉到 0 之后 `nodes_in_level == 1` 永远不成立，
    /// 循环无限往 `levels` 里塞 0 直到内存耗尽。**破坏被看见了，但不是断言抓到的**，
    /// 那是个盲区：换个形状的同类错误（例如少收敛一层）就不会 OOM，也就不会被发现。
    #[test]
    fn levels_converge_strictly_to_a_single_root() {
        for fanout in [2usize, 3, 17, 34, 68, 119, 170, 204, 238, 256, 273, 290, 307, 324, 339, 341] {
            let level_node_counts = levels_of(fanout);
            assert!(!level_node_counts.is_empty(), "扇出 {fanout} 的层数为空");
            assert!(level_node_counts.len() <= 16, "扇出 {fanout} 的层数 {} 不合理", level_node_counts.len());
            assert_eq!(*level_node_counts.last().unwrap(), 1, "扇出 {fanout} 的最顶层必须恰好 1 个根");
            for (level_index, &nodes_in_level) in level_node_counts.iter().enumerate() {
                assert!(nodes_in_level >= 1, "扇出 {fanout} 第 {level_index} 层是 {nodes_in_level}，每层至少 1 个节点");
            }
            for adjacent_levels in level_node_counts.windows(2) {
                assert!(adjacent_levels[1] < adjacent_levels[0], "扇出 {fanout} 的层数没有严格收敛：{:?}", level_node_counts);
            }
            // 绝对值：第 0 层的节点数恰好是 ceil(叶数 / 扇出)
            assert_eq!(level_node_counts[0], nleaf().div_ceil(fanout), "扇出 {fanout} 的第 0 层节点数");
        }
        // 手算的两个绝对值
        assert_eq!(levels_of(341), vec![4, 1]);
        assert_eq!(levels_of(17), vec![61, 4, 1]);
    }

    /// 页号不许重叠：一次写踩掉另一个节点时，实验只数 I/O 次数、看不出内容被踩。
    #[test]
    fn page_ranges_do_not_overlap() {
        for epsilon_permille in [5u32, 250, 500, 950] {
            let geometry = Geometry::new(epsilon_permille);
            let mut seen = std::collections::HashSet::new();
            for leaf_index in 0..nleaf() {
                assert!(seen.insert(geometry.leaf_page_number(leaf_index)), "ε={epsilon_permille}‰：叶 {leaf_index} 页号重叠");
            }
            for level in 0..geometry.height() {
                for node_index in 0..geometry.levels[level] {
                    assert!(seen.insert(geometry.internal_page_number(level, node_index)), "ε={epsilon_permille}‰：第 {level} 层第 {node_index} 个页号重叠");
                }
            }
            assert_eq!(seen.len(), geometry.total_nodes());
        }
    }

    /// key → 各层节点的映射必须落在界内，且逐层收敛到根。
    #[test]
    fn key_maps_into_every_level_in_range() {
        for epsilon_permille in [5u32, 500, 950] {
            let geometry = Geometry::new(epsilon_permille);
            for key in [0u64, 1, KEYS_PER_LEAF as u64, key_count() / 2, key_count() - 1] {
                assert!(geometry.leaf_of(key) < nleaf());
                for level in 0..geometry.height() {
                    assert!(geometry.node_of(level, key) < geometry.levels[level], "ε={epsilon_permille}‰ key={key} 第 {level} 层越界");
                }
                assert_eq!(geometry.node_of(geometry.height() - 1, key), 0, "最顶层只能是根");
            }
        }
    }

    /// 缓冲永远编码得进一个节点：条目数 × 16 + 头 ≤ 16384。
    #[test]
    fn buffer_always_fits_in_one_node() {
        for epsilon_permille in 0..=999u32 {
            let geometry = Geometry::new(epsilon_permille);
            assert!(geometry.buffer_capacity * MESSAGE_BYTES + NODE_HEADER_BYTES <= NODE_BYTES, "ε={epsilon_permille}‰ 的缓冲装不进节点");
            assert!(geometry.fanout >= 2, "ε={epsilon_permille}‰ 的扇出塌到 1，树就不收敛了");
        }
    }

    /// **无批量对照臂的解析 io/op**：每次操作恰好 1 次写 + (1 − 缓存/叶数) 次读。
    /// 缓存 66、叶 1024 ⇒ 1 + (1 − 66/1024) = 1.935547。
    #[test]
    fn the_no_batching_control_arm_has_an_analytic_device_operation_count_per_operation() {
        assert_eq!(nleaf(), 1024);
        assert_eq!(DEFAULT_CACHE_NODE_COUNT, 66);
        let analytic = 1.0 + (1.0 - DEFAULT_CACHE_NODE_COUNT as f64 / nleaf() as f64);
        assert!((analytic - 1.935_546_875).abs() < 1e-9, "解析式算出 {analytic}");
    }

    /// **绝对值断言 5：高 ε 档能欠下多大一笔账。**
    /// 各层缓冲加起来最多能扣住 total_internal × B 条消息，这些消息在收尾时
    /// 还没到过叶子。ε=0.95 时是 66 × 972 = 64152 条，占 20 万次操作的 32%
    /// ⇒ **不做收尾排空的话，高 ε 档的 io/op 把「还没付的账」算成了「省下的钱」。**
    /// 这条算术就是 2026-08-31 反推腿指出缺口时用的那条，钉在这里免得它只活在散文里。
    #[test]
    fn the_unpaid_buffer_debt_at_high_eps_is_a_third_of_the_workload() {
        let geometry = Geometry::new(950);
        assert_eq!(geometry.total_internal(), 66, "ε=0.95 的内部节点数");
        assert_eq!(geometry.buffer_capacity, 972);
        assert_eq!(geometry.total_internal() * geometry.buffer_capacity, 64_152);
        let debt_fraction = (geometry.total_internal() * geometry.buffer_capacity) as f64 / operation_count() as f64;
        assert!(debt_fraction > 0.32, "欠账占比实算 {debt_fraction}");
        // 反过来：低 ε 档几乎不欠账，所以那一段的数不受这个缺口影响
        let low_epsilon_geometry = Geometry::new(50);
        assert!((low_epsilon_geometry.total_internal() * low_epsilon_geometry.buffer_capacity) as f64 / (operation_count() as f64) < 0.002);
    }

    /// **绝对值断言 6：三条臂落在同一条曲线上。**
    /// 2026-08-31 正推腿在**没看过任何实测**的情况下推出
    /// `io/op = 2(1 − e^(−x))/x`，x = 一次下刷的消息数 / 该节点的孩子数：
    /// 臂 A 是 x→0（每条消息独占一次下刷），臂 B 是 x = 512/1024 = 0.5，
    /// 臂 C 是 x = B/F = 3ε/(1−ε)。
    /// ⇒ **「要不要消息缓冲」其实是「x 取多少」**，write buffer 前端本身就是 x=0.5 的 Bε。
    #[test]
    fn all_three_arms_lie_on_one_flush_curve() {
        let curve = |flush_ratio: f64| 2.0 * (1.0 - (-flush_ratio).exp()) / flush_ratio;
        // 臂 B：512 条消息摊到 1024 个叶
        let write_buffer_arm_cost = curve(WRITE_BUFFER_CAPACITY as f64 / nleaf() as f64);
        assert!((write_buffer_arm_cost - 1.5739).abs() < 1e-3, "臂 B 的闭式算出 {write_buffer_arm_cost}，正推腿盲推的是 1.574");
        // 臂 A：x→0 的极限是 2
        assert!((curve(1e-9) - 2.0).abs() < 1e-6);
        // 臂 C 在 ε=0.25 上：x = B/F = 1，摊销因子 1 − e^(−1) = 0.632
        let geometry = Geometry::new(250);
        let messages_per_child = geometry.buffer_capacity as f64 / geometry.fanout as f64;
        assert!((messages_per_child - 1.0).abs() < 1e-12, "ε=0.25 处 x 必须恰为 1");
        assert!((curve(messages_per_child) - 1.2642).abs() < 1e-3, "闭式算出 {}", curve(messages_per_child));
        // 臂 C 追平臂 B 的点在 x = 0.5，即 3ε/(1−ε) = 0.5 ⇒ ε = 1/7
        let epsilon_at_crossing: f64 = 1.0 / 7.0;
        let messages_per_child_at_crossing = 3.0 * epsilon_at_crossing / (1.0 - epsilon_at_crossing);
        assert!((messages_per_child_at_crossing - 0.5).abs() < 1e-12, "追平点算出 x={messages_per_child_at_crossing}");
    }

    /// 节点编解码要能原样回来，否则「消息真的下去了」这件事就没根据。
    #[test]
    fn node_round_trips_through_the_page() {
        let mut node = Node::new(256);
        for key in 0..200u64 {
            node.push(key, key ^ 0xABCD);
        }
        let mut page_buffer = AlignedBuffer::new(NODE_BYTES);
        node.encode(&mut page_buffer);
        let decoded = Node::decode(&page_buffer, 256);
        assert_eq!(decoded.entry_count, 200);
        assert_eq!(decoded.get(199), Some(199 ^ 0xABCD));
        assert_eq!(decoded.get(200), None);
    }

    /// 日志结构的「后者胜」：同 key 后写的必须赢，否则去重口径是错的。
    #[test]
    fn later_write_wins_within_a_node() {
        let mut node = Node::new(8);
        node.push(7, 1);
        node.push(7, 2);
        assert_eq!(node.get(7), Some(2));
        node.compact();
        assert_eq!(node.get(7), Some(2), "压实之后仍该是后写的那个");
        assert_eq!(node.entry_count, 1, "压实该把同 key 合成一条");
    }
}
