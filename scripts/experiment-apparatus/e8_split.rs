//! E8 层 A：大文件与小文件该不该走不同写路径——纯记账层。
//!
//! 判据见 kb/experiments.md E8。本层只回答「交叉点在哪、稳不稳」，
//! **不需要设备也不需要虚机**：写放大与每文件元数据字节数是格式上的算术，
//! 放进虚机跑只会把确定性的数字掺上噪声。端到端的吞吐/延迟留给层 B。
//!
//! ## 怎么防止它变成恒等式（E14 第一版栽过这个跟头）
//!
//! 四条臂**共用同一个记账器与同一棵索引树**，差别只在「数据放哪、插什么记录」这个放置策略。
//! 树与记账器是一份代码，臂里没有任何自己算字节数的地方——
//! 若每条臂各写一个公式，那测出来的只是我写下的公式。
//!
//! ## 介质不写成常数
//!
//! kb 里「机械盘顺序/随机约 1/100、SSD 约 1/10」明标着是**推理不是实测**。
//! 所以本实验不把它们烧进代码：分别报出 `device_bytes_written`（字节代价）与 `cache_miss_read_count`（随机代价），
//! 交叉点作为比值 r 的函数输出，让「换介质漂移多少」变成可读的曲线而不是一个写死的判断。
//!
//! ## 两个对照
//!
//! - **阳性对照**：64 B 文件在「不分流」臂上写放大必须 ≥ 64（整块分配摆在那），
//!   而内联臂必须显著更低。测不出来 = 模型坏了，整轮作废。
//! - **阴性对照**：1 GiB 文件上内联阈值根本够不着，内联臂必须与不分流臂**逐位相同**。
//!   不同 = 模型里有一条不该存在的路径。

use e7_index_bench::{Emitter, Lru};
use std::collections::BTreeMap;

// ───────────────────────── 格式参数 ─────────────────────────

#[derive(Clone, Copy)]
struct FormatParameters {
    node_bytes: usize,
    /// 块指针宽度（字节）。**本实验的扫描轴之一**——D19 未定，
    /// 所以交叉点对它敏不敏感本身就是要测的东西。
    pointer_bytes: usize,
    key_bytes: usize,
    block_bytes: u64,
    /// 内联阈值：≤ 它的文件数据住在 inode 记录里。
    inline_threshold_bytes: u64,
    /// 一个 checkpoint 里攒多少个操作（D16 已定 checkpoint 发布语义）。
    operations_per_checkpoint: usize,
    cache_nodes: usize,
}

/// inode 记录的固定部分：模式/大小/时间戳/链接数/若干代号。
const INODE_FIXED_BYTES: usize = 128;
/// 节点头（自描述字段，I-1.1~I-1.5）。
const NODE_HEADER_BYTES: usize = 64;

impl FormatParameters {
    /// 一条 extent 记录：key + 指针 + 长度。
    /// **KV 分离并不省这一项**——D4 的校验和内联进指针，值搬到别处也得有人存它的校验和。
    fn extent_record_bytes(&self) -> usize {
        self.key_bytes + self.pointer_bytes + 8
    }
    fn usable_node_bytes(&self) -> usize {
        self.node_bytes - NODE_HEADER_BYTES
    }
    /// 内部节点扇出。指针变宽直接压这里。
    fn fanout(&self) -> usize {
        (self.usable_node_bytes() / (self.key_bytes + self.pointer_bytes)).max(2)
    }
}

// ───────────────────────── 记账器 ─────────────────────────

/// 全部写都从这里过。四条臂谁也不许自己算字节。
#[derive(Default)]
struct Ledger {
    data_bytes: u64,
    meta_bytes: u64,
    /// 节点缓存未命中的读次数——随机代价的载体。
    cache_miss_read_count: u64,
    /// 落盘的节点次数（COW 重写）。
    node_writes: u64,
    /// 其中属于消息下推的字节（只是 meta_bytes 的一个切片，不另计入总数）。
    pushdown_bytes: u64,
}

impl Ledger {
    fn write_data(&mut self, bytes: u64) {
        self.data_bytes += bytes;
    }
    fn write_node(&mut self, bytes: u64) {
        self.meta_bytes += bytes;
        self.node_writes += 1;
    }
    fn count_cache_miss(&mut self) {
        self.cache_miss_read_count += 1;
    }
    /// 消息下推重写的字节。走 meta 这同一个出口——记账器仍然只有一条出路。
    fn pushdown(&mut self, bytes: u64) {
        self.meta_bytes += bytes;
        self.pushdown_bytes += bytes;
    }
    fn device_bytes_written(&self) -> u64 {
        self.data_bytes + self.meta_bytes
    }
}

// ───────────────────────── 索引树 ─────────────────────────

/// 只建模「记录落在哪个叶、树有多深、一个 checkpoint 弄脏几个节点」。
/// 记录内容不建模——本实验不问查得对不对，问的是写了多少。
struct Tree {
    parameters: FormatParameters,
    /// key → 记录字节数。记录大小可变（内联臂的 inode 记录会胀）。
    record_bytes_by_key: BTreeMap<u64, usize>,
    cache: Lru,
    dirty_leaves: std::collections::BTreeSet<usize>,
    /// 消息层臂用：本 checkpoint 内攒下的消息 (key, 记录字节)。
    pending_messages: Vec<(u64, usize)>,
}

impl Tree {
    fn new(parameters: FormatParameters) -> Self {
        Self {
            parameters,
            record_bytes_by_key: BTreeMap::new(),
            cache: Lru::new(parameters.cache_nodes),
            dirty_leaves: Default::default(),
            pending_messages: Vec::new(),
        }
    }

    /// 叶子按「排序位置 / 每叶记录数」定。记录变大 ⇒ 每叶装得下的变少 ⇒ 叶子变多。
    /// 用平均记录大小算，避免为了精确去实现真的分裂——分裂行为不在本实验射程内（同 E7）。
    fn records_per_leaf(&self) -> usize {
        let total_record_bytes: usize = self.record_bytes_by_key.values().sum();
        let record_count = self.record_bytes_by_key.len().max(1);
        let average_record_bytes = (total_record_bytes / record_count).max(1);
        (self.parameters.usable_node_bytes() / average_record_bytes).max(1)
    }

    fn leaf_of(&self, key: u64) -> usize {
        let rank = self.record_bytes_by_key.range(..key).count();
        rank / self.records_per_leaf()
    }

    fn leaf_count(&self) -> usize {
        (self.record_bytes_by_key.len().div_ceil(self.records_per_leaf())).max(1)
    }

    /// 树深（内部层数，不含叶）。
    fn depth(&self) -> usize {
        let mut nodes_at_level = self.leaf_count();
        let fanout = self.parameters.fanout();
        let mut level_count = 0;
        while nodes_at_level > 1 {
            nodes_at_level = nodes_at_level.div_ceil(fanout);
            level_count += 1;
        }
        level_count
    }

    /// 插入/更新一条记录。**读代价在这里发生**：要改一个叶子先得有它。
    fn upsert(&mut self, key: u64, bytes: usize, ledger: &mut Ledger) {
        let leaf = if self.record_bytes_by_key.is_empty() { 0 } else { self.leaf_of(key) };
        let page = leaf as u64;
        if !self.cache.contains(page) {
            ledger.count_cache_miss();
        }
        self.cache.touch(page);
        self.record_bytes_by_key.insert(key, bytes);
        self.dirty_leaves.insert(leaf);
    }

    /// 消息插入：**不碰目标叶**，所以不会缺页——这正是消息层相对「直接改叶」买到的东西。
    /// 代价在 checkpoint 下推时付（见 `checkpoint`）。
    fn enqueue_message(&mut self, key: u64, bytes: usize) {
        self.pending_messages.push((key, bytes));
    }

    /// checkpoint：脏叶各重写一遍，再把到根的路径重写一遍。
    /// 路径按「本轮脏叶去重后覆盖多少个上层节点」算，而不是每叶各算一条——
    /// 那正是 checkpoint 攒批买到的东西，四条臂同享。
    fn checkpoint(&mut self, ledger: &mut Ledger) {
        // 消息下推：每条消息在落到叶之前，要在每一层被重写一次。
        // 这是 Bε 的那笔代价，与上面「插入不缺页」那笔收益成对出现，缺一条就是偏袒。
        if !self.pending_messages.is_empty() {
            let tree_depth = self.depth() as u64;
            let pending_payload_bytes: u64 = self.pending_messages.iter().map(|&(_, record_bytes)| record_bytes as u64).sum();
            ledger.pushdown(pending_payload_bytes * tree_depth);
            let mut pending_sorted_by_key = std::mem::take(&mut self.pending_messages);
            // 真的 Bε flush 按 key 排序后下推。E7 已量到排序会摧毁 LRU 局部性——
            // 这里照实建模，那个后果本来就该出现在这条臂上。
            pending_sorted_by_key.sort_unstable_by_key(|&(key, _)| key);
            for (key, record_bytes) in pending_sorted_by_key {
                self.upsert(key, record_bytes, ledger);
            }
        }
        if self.dirty_leaves.is_empty() {
            return;
        }
        let node_bytes = self.parameters.node_bytes as u64;
        for _ in &self.dirty_leaves {
            ledger.write_node(node_bytes);
        }
        let fanout = self.parameters.fanout();
        let mut dirty_nodes_at_level: std::collections::BTreeSet<usize> =
            self.dirty_leaves.iter().map(|&child_index| child_index / fanout).collect();
        for _ in 0..self.depth() {
            for _ in &dirty_nodes_at_level {
                ledger.write_node(node_bytes);
            }
            if dirty_nodes_at_level.len() <= 1 {
                break;
            }
            dirty_nodes_at_level = dirty_nodes_at_level.iter().map(|&child_index| child_index / fanout).collect();
        }
        self.dirty_leaves.clear();
    }
}

// ───────────────────────── 臂与负载 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
enum Strategy {
    /// 全走 extent，小文件也占整块。
    NoSplit,
    /// ≤ 阈值内联进 inode 记录。
    InlineInode,
    /// ≤ 阈值的数据当成消息挂在索引里，随节点下刷落盘。
    MessageLayer,
    /// 值顺序追加进 value log，索引只存 (key → log 位置)。
    KvSeparate,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Form {
    OnceWrite,
    Append,
    Overwrite,
    RandomSmall,
}

/// 一个文件的 key 空间：inode 记录一个 key，extent 记录按 (inode, offset) 排。
/// 高位放 inode 保证同一文件的 extent 在 key 序上连续（D8 已定的布局规则）。
fn inode_key(inode_number: u64) -> u64 {
    inode_number << 32
}
fn extent_key(inode_number: u64, offset: u64, block_bytes: u64) -> u64 {
    (inode_number << 32) | (offset / block_bytes + 1)
}

struct StrategyRun {
    ledger: Ledger,
    tree: Tree,
    parameters: FormatParameters,
    strategy: Strategy,
    /// value log 的追加位置（KV 分离臂用）。顺序写，无内部碎片。
    log_head: u64,
}

impl StrategyRun {
    fn new(parameters: FormatParameters, strategy: Strategy) -> Self {
        Self {
            ledger: Ledger::default(),
            tree: Tree::new(parameters),
            parameters,
            strategy,
            log_head: 0,
        }
    }

    fn is_inlineable(&self, size: u64) -> bool {
        matches!(self.strategy, Strategy::InlineInode | Strategy::MessageLayer) && size <= self.parameters.inline_threshold_bytes
    }

    /// 把 [offset, offset+length_bytes) 这一段数据落下去，并插相应的索引记录。
    fn put_range(&mut self, inode_number: u64, offset: u64, length_bytes: u64) {
        match self.strategy {
            Strategy::KvSeparate => {
                // 顺序追加，按字节计，不按块对齐 —— 这正是它相对整块分配的全部优势。
                self.log_head += length_bytes;
                self.ledger.write_data(length_bytes);
                let extent_record_key = extent_key(inode_number, offset, self.parameters.block_bytes);
                let record_bytes = self.parameters.extent_record_bytes();
                self.tree.upsert(extent_record_key, record_bytes, &mut self.ledger);
            }
            _ => {
                // 整块分配：不足一块也占一块，这是小文件在不分流臂上的全部代价来源。
                let block_count = length_bytes.div_ceil(self.parameters.block_bytes);
                self.ledger.write_data(block_count * self.parameters.block_bytes);
                let extent_record_key = extent_key(inode_number, offset, self.parameters.block_bytes);
                let record_bytes = self.parameters.extent_record_bytes();
                self.tree.upsert(extent_record_key, record_bytes, &mut self.ledger);
            }
        }
    }

    /// 内联形态：数据不落独立块，跟着索引记录一起写出去。
    fn put_inline(&mut self, inode_number: u64, size: u64) {
        let inode_record_bytes = INODE_FIXED_BYTES + size as usize;
        match self.strategy {
            // 消息层：挂进缓冲，本次不碰叶子。
            Strategy::MessageLayer => self.tree.enqueue_message(inode_key(inode_number), inode_record_bytes),
            _ => self.tree.upsert(inode_key(inode_number), inode_record_bytes, &mut self.ledger),
        }
    }

    fn create(&mut self, inode_number: u64) {
        self.tree.upsert(inode_key(inode_number), INODE_FIXED_BYTES, &mut self.ledger);
    }

    /// 内联 → extent 的迁移：文件长过阈值时把内联的那一份改写成独立块。
    /// **这条路径本身就是 E8 判据 3 要问的东西**（阈值是不是操作期间稳定的量）。
    fn spill(&mut self, inode_number: u64, size: u64) {
        self.tree.upsert(inode_key(inode_number), INODE_FIXED_BYTES, &mut self.ledger);
        self.put_range(inode_number, 0, size);
    }
}

/// 跑一个 (文件大小, 写形态, 臂) 组合。返回读数。
/// `file_count` 让小文件档也能凑出有意义的树规模。
fn run_case(parameters: FormatParameters, strategy: Strategy, size: u64, form: Form, file_count: u64, seed: u64) -> (Ledger, u64) {
    let mut strategy_run = StrategyRun::new(parameters, strategy);
    let mut logical_bytes: u64 = 0;
    let mut random_state = seed | 1;
    let mut next_random = || {
        random_state ^= random_state >> 12;
        random_state ^= random_state << 25;
        random_state ^= random_state >> 27;
        random_state.wrapping_mul(0x2545F4914F6CDD1D)
    };
    let mut operations_since_checkpoint = 0usize;

    for inode_number in 1..=file_count {
        strategy_run.create(inode_number);
        match form {
            Form::OnceWrite => {
                if strategy_run.is_inlineable(size) {
                    strategy_run.put_inline(inode_number, size);
                } else {
                    strategy_run.put_range(inode_number, 0, size);
                }
                logical_bytes += size;
            }
            Form::Append => {
                // 8 次等分追加。跨阈值时走 spill —— 内联臂在这里要付迁移的钱。
                let append_chunk_bytes = (size / 8).max(1);
                let mut written_bytes = 0u64;
                let mut has_spilled = false;
                for _ in 0..8 {
                    let this_append_bytes = append_chunk_bytes.min(size - written_bytes);
                    if this_append_bytes == 0 {
                        break;
                    }
                    written_bytes += this_append_bytes;
                    logical_bytes += this_append_bytes;
                    if strategy_run.is_inlineable(written_bytes) {
                        strategy_run.put_inline(inode_number, written_bytes);
                    } else {
                        if !has_spilled && strategy_run.is_inlineable(written_bytes - this_append_bytes) {
                            strategy_run.spill(inode_number, written_bytes);
                            has_spilled = true;
                        } else {
                            strategy_run.put_range(inode_number, written_bytes - this_append_bytes, this_append_bytes);
                        }
                    }
                }
            }
            Form::Overwrite => {
                if strategy_run.is_inlineable(size) {
                    strategy_run.put_inline(inode_number, size);
                    strategy_run.put_inline(inode_number, size);
                } else {
                    strategy_run.put_range(inode_number, 0, size);
                    strategy_run.put_range(inode_number, 0, size);
                }
                logical_bytes += size * 2;
            }
            Form::RandomSmall => {
                // 16 次随机小写，每次 min(4KiB, size)。COW ⇒ 每次都重写受影响的块。
                let small_write_bytes = 4096u64.min(size);
                for _ in 0..16 {
                    let offset = if size > small_write_bytes { (next_random() % (size - small_write_bytes)) / small_write_bytes * small_write_bytes } else { 0 };
                    if strategy_run.is_inlineable(size) {
                        strategy_run.put_inline(inode_number, size);
                    } else {
                        strategy_run.put_range(inode_number, offset, small_write_bytes);
                    }
                    logical_bytes += small_write_bytes;
                }
            }
        }
        operations_since_checkpoint += 1;
        if operations_since_checkpoint >= parameters.operations_per_checkpoint {
            strategy_run.tree.checkpoint(&mut strategy_run.ledger);
            operations_since_checkpoint = 0;
        }
    }
    strategy_run.tree.checkpoint(&mut strategy_run.ledger);
    (strategy_run.ledger, logical_bytes)
}

// ───────────────────────── 主程序 ─────────────────────────

const SIZES: [(u64, &str); 7] = [
    (64, "64B"),
    (1024, "1K"),
    (4096, "4K"),
    (65536, "64K"),
    (1 << 20, "1M"),
    (16 << 20, "16M"),
    (1 << 30, "1G"),
];

const ARMS: [(Strategy, &str); 4] = [
    (Strategy::NoSplit, "nosplit"),
    (Strategy::InlineInode, "inline"),
    (Strategy::MessageLayer, "msgbuf"),
    (Strategy::KvSeparate, "kvsep"),
];

const FORMS: [(Form, &str); 4] = [
    (Form::OnceWrite, "once"),
    (Form::Append, "append"),
    (Form::Overwrite, "overwrite"),
    (Form::RandomSmall, "randsmall"),
];

/// 大文件档减少文件数，否则 1 GiB × 4096 个的逻辑量毫无意义地大。
fn file_count_for(size: u64) -> u64 {
    match size {
        file_size if file_size <= 4096 => 4096,
        file_size if file_size <= 65536 => 1024,
        file_size if file_size <= (1 << 20) => 256,
        file_size if file_size <= (16 << 20) => 64,
        _ => 8,
    }
}

fn base_parameters(pointer_bytes: usize) -> FormatParameters {
    FormatParameters {
        node_bytes: 16 * 1024,
        pointer_bytes,
        key_bytes: 16,
        block_bytes: 4096,
        inline_threshold_bytes: 3584, // 一个块减去节点头与 inode 固定部分的余量
        operations_per_checkpoint: std::env::var("E8_BATCH").ok().and_then(|value| value.parse().ok()).unwrap_or(64),
        cache_nodes: std::env::var("E8_CACHE").ok().and_then(|value| value.parse().ok()).unwrap_or(256),
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let is_selfcheck = arguments.iter().any(|argument| argument == "--selfcheck");
    let mut emitter = Emitter::new();
    let mut output = String::new();
    let mut say = |line: String| {
        output.push_str(&line);
        output.push('\n');
    };

    // 指针宽度扫描：32 B = ZFS 那个 256 位槽位；40 B ≈ 本工程数出来的 320 位。
    // 32B=ZFS 那个 256 位槽；40B≈本工程头部；67B=头部+2 副本位置条目；
    // 111B=头部+4+2 条带六个位置条目（「什么都带上」那一档）。
    let pointer_widths: [(usize, &str); 4] =
        [(32, "ptr256"), (40, "ptr320"), (67, "ptr536mirror"), (111, "ptr888stripe")];

    // ── 对照先跑，不过就不出结果 ──
    let parameters = base_parameters(32);
    let (nosplit_ledger, nosplit_logical_bytes) = run_case(parameters, Strategy::NoSplit, 64, Form::OnceWrite, 4096, 1);
    let (inline_ledger, _) = run_case(parameters, Strategy::InlineInode, 64, Form::OnceWrite, 4096, 1);
    let nosplit_amplification = nosplit_ledger.device_bytes_written() as f64 / nosplit_logical_bytes as f64;
    let inline_amplification = inline_ledger.device_bytes_written() as f64 / nosplit_logical_bytes as f64;
    say(emitter.emit_raw(&format!(
        "name=posctl_64B nosplit_amp={nosplit_amplification:.2} inline_amp={inline_amplification:.2} nosplit_data={} inline_data={}",
        nosplit_ledger.data_bytes, inline_ledger.data_bytes
    )));
    // 对照只断言它要证的那个机制：64 B 文件在不分流臂上真的付了整块的钱，
    // 而内联臂真的一个数据块都没落。**不许断言「内联赢多少」**——
    // 那个倍数随 checkpoint 攒批大小变，把它写进对照会让一个合法结果被误判成模型坏了。
    let positive_control_passed = nosplit_ledger.data_bytes == 4096 * 4096 && inline_ledger.data_bytes == 0;

    let (nosplit_big_ledger, _) = run_case(parameters, Strategy::NoSplit, 1 << 30, Form::OnceWrite, 8, 1);
    let (inline_big_ledger, _) = run_case(parameters, Strategy::InlineInode, 1 << 30, Form::OnceWrite, 8, 1);
    let negative_control_passed = nosplit_big_ledger.device_bytes_written() == inline_big_ledger.device_bytes_written()
        && nosplit_big_ledger.cache_miss_read_count == inline_big_ledger.cache_miss_read_count
        && nosplit_big_ledger.node_writes == inline_big_ledger.node_writes;
    say(emitter.emit_raw(&format!(
        "name=negctl_1G nosplit_bytes={} inline_bytes={} nosplit_reads={} inline_reads={} identical={}",
        nosplit_big_ledger.device_bytes_written(),
        inline_big_ledger.device_bytes_written(),
        nosplit_big_ledger.cache_miss_read_count,
        inline_big_ledger.cache_miss_read_count,
        negative_control_passed
    )));
    say(emitter.emit_raw(&format!(
        "name=controls pos_ok={positive_control_passed} neg_ok={negative_control_passed}"
    )));

    if !(positive_control_passed && negative_control_passed) {
        say(emitter.finish());
        print!("{output}");
        eprintln!("E8: 对照未通过（pos_ok={positive_control_passed} neg_ok={negative_control_passed}）——模型有问题，本轮作废");
        std::process::exit(4);
    }

    // ── 正式扫描 ──
    for (pointer_width_bytes, pointer_width_name) in pointer_widths {
        let mut parameters = base_parameters(pointer_width_bytes);
        if is_selfcheck {
            // 自证会红：把内联阈值打到 0，内联/消息两臂应当退化成与不分流完全一样。
            parameters.inline_threshold_bytes = 0;
        }
        for (form, form_name) in FORMS {
            for (size, size_name) in SIZES {
                let file_count = file_count_for(size);
                for (strategy, arm_name) in ARMS {
                    let (ledger, logical_bytes) = run_case(parameters, strategy, size, form, file_count, 7);
                    let amplification = ledger.device_bytes_written() as f64 / logical_bytes.max(1) as f64;
                    let meta_per_file = ledger.meta_bytes as f64 / file_count as f64;
                    say(emitter.emit_raw(&format!(
                        "name=e8 ptr={pointer_width_name} form={form_name} size={size_name} arm={arm_name} \
                         logical={logical_bytes} dev_bytes={} data={} meta={} \
                         amp={amplification:.4} reads={} node_writes={} pushdown={} meta_per_file={meta_per_file:.1}",
                        ledger.device_bytes_written(),
                        ledger.data_bytes,
                        ledger.meta_bytes,
                        ledger.cache_miss_read_count,
                        ledger.node_writes,
                        ledger.pushdown_bytes
                    )));
                }
            }
        }
    }

    say(emitter.finish());
    print!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 扇出必须随指针变宽而下降——这是 E8 与 D19 的连接点，写错了整条曲线是假的。
    #[test]
    fn wider_pointer_lowers_fanout() {
        assert!(base_parameters(40).fanout() < base_parameters(32).fanout());
    }

    /// 阳性对照：64 B 文件在不分流臂上必须付整块的钱。
    #[test]
    fn tiny_file_costs_a_whole_block_without_split() {
        let parameters = base_parameters(32);
        let (ledger, logical_bytes) = run_case(parameters, Strategy::NoSplit, 64, Form::OnceWrite, 1024, 1);
        assert!(ledger.data_bytes >= 1024 * parameters.block_bytes);
        assert!(ledger.device_bytes_written() as f64 / logical_bytes as f64 >= 64.0);
    }

    /// 阴性对照：阈值够不着的大文件上，内联臂必须与不分流臂逐位相同。
    #[test]
    fn inline_is_a_noop_above_threshold() {
        let parameters = base_parameters(32);
        let (nosplit_ledger, _) = run_case(parameters, Strategy::NoSplit, 1 << 20, Form::OnceWrite, 64, 3);
        let (inline_ledger, _) = run_case(parameters, Strategy::InlineInode, 1 << 20, Form::OnceWrite, 64, 3);
        assert_eq!(nosplit_ledger.device_bytes_written(), inline_ledger.device_bytes_written());
        assert_eq!(nosplit_ledger.cache_miss_read_count, inline_ledger.cache_miss_read_count);
        assert_eq!(nosplit_ledger.node_writes, inline_ledger.node_writes);
    }

    /// 内联边界是 ≤（含阈值本身）：恰等于阈值的文件必须零数据块，多一字节必须整块
    /// （变异审计补的：此前没有任何测试落在 3584 的边界上，`<=` 改 `<` 不会红）。
    #[test]
    fn inline_boundary_is_inclusive() {
        let parameters = base_parameters(32);
        let (at_threshold_ledger, _) = run_case(parameters, Strategy::InlineInode, parameters.inline_threshold_bytes, Form::OnceWrite, 64, 3);
        assert_eq!(at_threshold_ledger.data_bytes, 0, "恰等于阈值该内联，零数据块");
        let (over_threshold_ledger, _) = run_case(parameters, Strategy::InlineInode, parameters.inline_threshold_bytes + 1, Form::OnceWrite, 64, 3);
        assert!(over_threshold_ledger.data_bytes >= 64 * parameters.block_bytes, "超过阈值一字节该走整块");
    }

    /// KV 分离在小文件上不占整块——它与不分流的差别必须真的出现在数据字节上。
    #[test]
    fn kv_separation_avoids_block_rounding() {
        let parameters = base_parameters(32);
        let (nosplit_ledger, _) = run_case(parameters, Strategy::NoSplit, 64, Form::OnceWrite, 1024, 5);
        let (kv_separate_ledger, _) = run_case(parameters, Strategy::KvSeparate, 64, Form::OnceWrite, 1024, 5);
        assert!(kv_separate_ledger.data_bytes < nosplit_ledger.data_bytes / 10);
    }

    /// 记账器是唯一出口：device_bytes_written 必须恰等于两项之和，不许有第三条计数路径。
    #[test]
    fn ledger_has_one_exit() {
        let parameters = base_parameters(32);
        let (ledger, _) = run_case(parameters, Strategy::MessageLayer, 4096, Form::Append, 128, 9);
        assert_eq!(ledger.device_bytes_written(), ledger.data_bytes + ledger.meta_bytes);
        assert!(ledger.node_writes > 0);
    }
}
