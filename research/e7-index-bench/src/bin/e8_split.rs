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
//! 所以本实验不把它们烧进代码：分别报出 `dev_bytes`（字节代价）与 `read_ops`（随机代价），
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
struct Params {
    node_bytes: usize,
    /// 块指针宽度（字节）。**本实验的扫描轴之一**——D19 未定，
    /// 所以交叉点对它敏不敏感本身就是要测的东西。
    ptr_bytes: usize,
    key_bytes: usize,
    block_bytes: u64,
    /// 内联阈值：≤ 它的文件数据住在 inode 记录里。
    inline_max: u64,
    /// 一个 checkpoint 里攒多少个操作（D16 已定 checkpoint 发布语义）。
    batch: usize,
    cache_nodes: usize,
}

/// inode 记录的固定部分：模式/大小/时间戳/链接数/若干代号。
const INODE_FIXED: usize = 128;
/// 节点头（自描述字段，I-1.1~I-1.5）。
const NODE_HDR: usize = 64;

impl Params {
    /// 一条 extent 记录：key + 指针 + 长度。
    /// **KV 分离并不省这一项**——D4 的校验和内联进指针，值搬到别处也得有人存它的校验和。
    fn extent_rec(&self) -> usize {
        self.key_bytes + self.ptr_bytes + 8
    }
    fn usable(&self) -> usize {
        self.node_bytes - NODE_HDR
    }
    /// 内部节点扇出。指针变宽直接压这里。
    fn fanout(&self) -> usize {
        (self.usable() / (self.key_bytes + self.ptr_bytes)).max(2)
    }
}

// ───────────────────────── 记账器 ─────────────────────────

/// 全部写都从这里过。四条臂谁也不许自己算字节。
#[derive(Default)]
struct Ledger {
    data_bytes: u64,
    meta_bytes: u64,
    /// 节点缓存未命中的读次数——随机代价的载体。
    read_ops: u64,
    /// 落盘的节点次数（COW 重写）。
    node_writes: u64,
    /// 其中属于消息下推的字节（只是 meta_bytes 的一个切片，不另计入总数）。
    pushdown_bytes: u64,
}

impl Ledger {
    fn write_data(&mut self, b: u64) {
        self.data_bytes += b;
    }
    fn write_node(&mut self, b: u64) {
        self.meta_bytes += b;
        self.node_writes += 1;
    }
    fn miss(&mut self) {
        self.read_ops += 1;
    }
    /// 消息下推重写的字节。走 meta 这同一个出口——记账器仍然只有一条出路。
    fn pushdown(&mut self, b: u64) {
        self.meta_bytes += b;
        self.pushdown_bytes += b;
    }
    fn dev_bytes(&self) -> u64 {
        self.data_bytes + self.meta_bytes
    }
}

// ───────────────────────── 索引树 ─────────────────────────

/// 只建模「记录落在哪个叶、树有多深、一个 checkpoint 弄脏几个节点」。
/// 记录内容不建模——本实验不问查得对不对，问的是写了多少。
struct Tree {
    p: Params,
    /// key → 记录字节数。记录大小可变（内联臂的 inode 记录会胀）。
    recs: BTreeMap<u64, usize>,
    cache: Lru,
    dirty_leaves: std::collections::BTreeSet<usize>,
    /// 消息层臂用：本 checkpoint 内攒下的消息 (key, 记录字节)。
    pending: Vec<(u64, usize)>,
}

impl Tree {
    fn new(p: Params) -> Self {
        Self {
            p,
            recs: BTreeMap::new(),
            cache: Lru::new(p.cache_nodes),
            dirty_leaves: Default::default(),
            pending: Vec::new(),
        }
    }

    /// 叶子按「排序位置 / 每叶记录数」定。记录变大 ⇒ 每叶装得下的变少 ⇒ 叶子变多。
    /// 用平均记录大小算，避免为了精确去实现真的分裂——分裂行为不在本实验射程内（同 E7）。
    fn recs_per_leaf(&self) -> usize {
        let total: usize = self.recs.values().sum();
        let n = self.recs.len().max(1);
        let avg = (total / n).max(1);
        (self.p.usable() / avg).max(1)
    }

    fn leaf_of(&self, key: u64) -> usize {
        let rank = self.recs.range(..key).count();
        rank / self.recs_per_leaf()
    }

    fn n_leaves(&self) -> usize {
        (self.recs.len().div_ceil(self.recs_per_leaf())).max(1)
    }

    /// 树深（内部层数，不含叶）。
    fn depth(&self) -> usize {
        let mut n = self.n_leaves();
        let f = self.p.fanout();
        let mut d = 0;
        while n > 1 {
            n = n.div_ceil(f);
            d += 1;
        }
        d
    }

    /// 插入/更新一条记录。**读代价在这里发生**：要改一个叶子先得有它。
    fn upsert(&mut self, key: u64, bytes: usize, l: &mut Ledger) {
        let leaf = if self.recs.is_empty() { 0 } else { self.leaf_of(key) };
        let page = leaf as u64;
        if !self.cache.contains(page) {
            l.miss();
        }
        self.cache.touch(page);
        self.recs.insert(key, bytes);
        self.dirty_leaves.insert(leaf);
    }

    /// 消息插入：**不碰目标叶**，所以不会缺页——这正是消息层相对「直接改叶」买到的东西。
    /// 代价在 checkpoint 下推时付（见 `checkpoint`）。
    fn message(&mut self, key: u64, bytes: usize) {
        self.pending.push((key, bytes));
    }

    /// checkpoint：脏叶各重写一遍，再把到根的路径重写一遍。
    /// 路径按「本轮脏叶去重后覆盖多少个上层节点」算，而不是每叶各算一条——
    /// 那正是 checkpoint 攒批买到的东西，四条臂同享。
    fn checkpoint(&mut self, l: &mut Ledger) {
        // 消息下推：每条消息在落到叶之前，要在每一层被重写一次。
        // 这是 Bε 的那笔代价，与上面「插入不缺页」那笔收益成对出现，缺一条就是偏袒。
        if !self.pending.is_empty() {
            let d = self.depth() as u64;
            let payload: u64 = self.pending.iter().map(|&(_, b)| b as u64).sum();
            l.pushdown(payload * d);
            let mut pend = std::mem::take(&mut self.pending);
            // 真的 Bε flush 按 key 排序后下推。E7 已量到排序会摧毁 LRU 局部性——
            // 这里照实建模，那个后果本来就该出现在这条臂上。
            pend.sort_unstable_by_key(|&(k, _)| k);
            for (k, b) in pend {
                self.upsert(k, b, l);
            }
        }
        if self.dirty_leaves.is_empty() {
            return;
        }
        let nb = self.p.node_bytes as u64;
        for _ in &self.dirty_leaves {
            l.write_node(nb);
        }
        let f = self.p.fanout();
        let mut level: std::collections::BTreeSet<usize> =
            self.dirty_leaves.iter().map(|&x| x / f).collect();
        for _ in 0..self.depth() {
            for _ in &level {
                l.write_node(nb);
            }
            if level.len() <= 1 {
                break;
            }
            level = level.iter().map(|&x| x / f).collect();
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
fn inode_key(ino: u64) -> u64 {
    ino << 32
}
fn extent_key(ino: u64, off: u64, blk: u64) -> u64 {
    (ino << 32) | (off / blk + 1)
}

struct Run {
    l: Ledger,
    t: Tree,
    p: Params,
    s: Strategy,
    /// value log 的追加位置（KV 分离臂用）。顺序写，无内部碎片。
    log_head: u64,
}

impl Run {
    fn new(p: Params, s: Strategy) -> Self {
        Self {
            l: Ledger::default(),
            t: Tree::new(p),
            p,
            s,
            log_head: 0,
        }
    }

    fn inlineable(&self, size: u64) -> bool {
        matches!(self.s, Strategy::InlineInode | Strategy::MessageLayer) && size <= self.p.inline_max
    }

    /// 把 [off, off+len) 这一段数据落下去，并插相应的索引记录。
    fn put_range(&mut self, ino: u64, off: u64, len: u64) {
        match self.s {
            Strategy::KvSeparate => {
                // 顺序追加，按字节计，不按块对齐 —— 这正是它相对整块分配的全部优势。
                self.log_head += len;
                self.l.write_data(len);
                let k = extent_key(ino, off, self.p.block_bytes);
                let rec = self.p.extent_rec();
                self.t.upsert(k, rec, &mut self.l);
            }
            _ => {
                // 整块分配：不足一块也占一块，这是小文件在不分流臂上的全部代价来源。
                let blocks = len.div_ceil(self.p.block_bytes);
                self.l.write_data(blocks * self.p.block_bytes);
                let k = extent_key(ino, off, self.p.block_bytes);
                let rec = self.p.extent_rec();
                self.t.upsert(k, rec, &mut self.l);
            }
        }
    }

    /// 内联形态：数据不落独立块，跟着索引记录一起写出去。
    fn put_inline(&mut self, ino: u64, size: u64) {
        let rec = INODE_FIXED + size as usize;
        match self.s {
            // 消息层：挂进缓冲，本次不碰叶子。
            Strategy::MessageLayer => self.t.message(inode_key(ino), rec),
            _ => self.t.upsert(inode_key(ino), rec, &mut self.l),
        }
    }

    fn create(&mut self, ino: u64) {
        self.t.upsert(inode_key(ino), INODE_FIXED, &mut self.l);
    }

    /// 内联 → extent 的迁移：文件长过阈值时把内联的那一份改写成独立块。
    /// **这条路径本身就是 E8 判据 3 要问的东西**（阈值是不是操作期间稳定的量）。
    fn spill(&mut self, ino: u64, size: u64) {
        self.t.upsert(inode_key(ino), INODE_FIXED, &mut self.l);
        self.put_range(ino, 0, size);
    }
}

/// 跑一个 (文件大小, 写形态, 臂) 组合。返回读数。
/// `n_files` 让小文件档也能凑出有意义的树规模。
fn run_case(p: Params, s: Strategy, size: u64, form: Form, n_files: u64, seed: u64) -> (Ledger, u64) {
    let mut r = Run::new(p, s);
    let mut logical: u64 = 0;
    let mut rng = seed | 1;
    let mut next = || {
        rng ^= rng >> 12;
        rng ^= rng << 25;
        rng ^= rng >> 27;
        rng.wrapping_mul(0x2545F4914F6CDD1D)
    };
    let mut since_cp = 0usize;

    for ino in 1..=n_files {
        r.create(ino);
        match form {
            Form::OnceWrite => {
                if r.inlineable(size) {
                    r.put_inline(ino, size);
                } else {
                    r.put_range(ino, 0, size);
                }
                logical += size;
            }
            Form::Append => {
                // 8 次等分追加。跨阈值时走 spill —— 内联臂在这里要付迁移的钱。
                let chunk = (size / 8).max(1);
                let mut written = 0u64;
                let mut spilled = false;
                for _ in 0..8 {
                    let n = chunk.min(size - written);
                    if n == 0 {
                        break;
                    }
                    written += n;
                    logical += n;
                    if r.inlineable(written) {
                        r.put_inline(ino, written);
                    } else {
                        if !spilled && r.inlineable(written - n) {
                            r.spill(ino, written);
                            spilled = true;
                        } else {
                            r.put_range(ino, written - n, n);
                        }
                    }
                }
            }
            Form::Overwrite => {
                if r.inlineable(size) {
                    r.put_inline(ino, size);
                    r.put_inline(ino, size);
                } else {
                    r.put_range(ino, 0, size);
                    r.put_range(ino, 0, size);
                }
                logical += size * 2;
            }
            Form::RandomSmall => {
                // 16 次随机小写，每次 min(4KiB, size)。COW ⇒ 每次都重写受影响的块。
                let w = 4096u64.min(size);
                for _ in 0..16 {
                    let off = if size > w { (next() % (size - w)) / w * w } else { 0 };
                    if r.inlineable(size) {
                        r.put_inline(ino, size);
                    } else {
                        r.put_range(ino, off, w);
                    }
                    logical += w;
                }
            }
        }
        since_cp += 1;
        if since_cp >= p.batch {
            r.t.checkpoint(&mut r.l);
            since_cp = 0;
        }
    }
    r.t.checkpoint(&mut r.l);
    (r.l, logical)
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
fn n_files_for(size: u64) -> u64 {
    match size {
        s if s <= 4096 => 4096,
        s if s <= 65536 => 1024,
        s if s <= (1 << 20) => 256,
        s if s <= (16 << 20) => 64,
        _ => 8,
    }
}

fn base(ptr_bytes: usize) -> Params {
    Params {
        node_bytes: 16 * 1024,
        ptr_bytes,
        key_bytes: 16,
        block_bytes: 4096,
        inline_max: 3584, // 一个块减去节点头与 inode 固定部分的余量
        batch: std::env::var("E8_BATCH").ok().and_then(|v| v.parse().ok()).unwrap_or(64),
        cache_nodes: std::env::var("E8_CACHE").ok().and_then(|v| v.parse().ok()).unwrap_or(256),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let selfcheck = args.iter().any(|a| a == "--selfcheck");
    let mut e = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| {
        out.push_str(&s);
        out.push('\n');
    };

    // 指针宽度扫描：32 B = ZFS 那个 256 位槽位；40 B ≈ 本工程数出来的 320 位。
    // 32B=ZFS 那个 256 位槽；40B≈本工程头部；67B=头部+2 副本位置条目；
    // 111B=头部+4+2 条带六个位置条目（「什么都带上」那一档）。
    let ptr_widths: [(usize, &str); 4] =
        [(32, "ptr256"), (40, "ptr320"), (67, "ptr536mirror"), (111, "ptr888stripe")];

    // ── 对照先跑，不过就不出结果 ──
    let p = base(32);
    let (l_ns, log_ns) = run_case(p, Strategy::NoSplit, 64, Form::OnceWrite, 4096, 1);
    let (l_in, _) = run_case(p, Strategy::InlineInode, 64, Form::OnceWrite, 4096, 1);
    let amp_ns = l_ns.dev_bytes() as f64 / log_ns as f64;
    let amp_in = l_in.dev_bytes() as f64 / log_ns as f64;
    say(e.emit_raw(&format!(
        "name=posctl_64B nosplit_amp={amp_ns:.2} inline_amp={amp_in:.2} nosplit_data={} inline_data={}",
        l_ns.data_bytes, l_in.data_bytes
    )));
    // 对照只断言它要证的那个机制：64 B 文件在不分流臂上真的付了整块的钱，
    // 而内联臂真的一个数据块都没落。**不许断言「内联赢多少」**——
    // 那个倍数随 checkpoint 攒批大小变，把它写进对照会让一个合法结果被误判成模型坏了。
    let pos_ok = l_ns.data_bytes == 4096 * 4096 && l_in.data_bytes == 0;

    let (l_ns_big, _) = run_case(p, Strategy::NoSplit, 1 << 30, Form::OnceWrite, 8, 1);
    let (l_in_big, _) = run_case(p, Strategy::InlineInode, 1 << 30, Form::OnceWrite, 8, 1);
    let neg_ok = l_ns_big.dev_bytes() == l_in_big.dev_bytes()
        && l_ns_big.read_ops == l_in_big.read_ops
        && l_ns_big.node_writes == l_in_big.node_writes;
    say(e.emit_raw(&format!(
        "name=negctl_1G nosplit_bytes={} inline_bytes={} nosplit_reads={} inline_reads={} identical={}",
        l_ns_big.dev_bytes(),
        l_in_big.dev_bytes(),
        l_ns_big.read_ops,
        l_in_big.read_ops,
        neg_ok
    )));
    say(e.emit_raw(&format!(
        "name=controls pos_ok={pos_ok} neg_ok={neg_ok}"
    )));

    if !(pos_ok && neg_ok) {
        say(e.finish());
        print!("{out}");
        eprintln!("E8: 对照未通过（pos_ok={pos_ok} neg_ok={neg_ok}）——模型有问题，本轮作废");
        std::process::exit(4);
    }

    // ── 正式扫描 ──
    for (pb, pname) in ptr_widths {
        let mut p = base(pb);
        if selfcheck {
            // 自证会红：把内联阈值打到 0，内联/消息两臂应当退化成与不分流完全一样。
            p.inline_max = 0;
        }
        for (form, fname) in FORMS {
            for (size, sname) in SIZES {
                let nf = n_files_for(size);
                for (s, aname) in ARMS {
                    let (l, logical) = run_case(p, s, size, form, nf, 7);
                    let amp = l.dev_bytes() as f64 / logical.max(1) as f64;
                    let meta_per_file = l.meta_bytes as f64 / nf as f64;
                    say(e.emit_raw(&format!(
                        "name=e8 ptr={pname} form={fname} size={sname} arm={aname} \
                         logical={logical} dev_bytes={} data={} meta={} \
                         amp={amp:.4} reads={} node_writes={} pushdown={} meta_per_file={meta_per_file:.1}",
                        l.dev_bytes(),
                        l.data_bytes,
                        l.meta_bytes,
                        l.read_ops,
                        l.node_writes,
                        l.pushdown_bytes
                    )));
                }
            }
        }
    }

    say(e.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 扇出必须随指针变宽而下降——这是 E8 与 D19 的连接点，写错了整条曲线是假的。
    #[test]
    fn wider_pointer_lowers_fanout() {
        assert!(base(40).fanout() < base(32).fanout());
    }

    /// 阳性对照：64 B 文件在不分流臂上必须付整块的钱。
    #[test]
    fn tiny_file_costs_a_whole_block_without_split() {
        let p = base(32);
        let (l, logical) = run_case(p, Strategy::NoSplit, 64, Form::OnceWrite, 1024, 1);
        assert!(l.data_bytes >= 1024 * p.block_bytes);
        assert!(l.dev_bytes() as f64 / logical as f64 >= 64.0);
    }

    /// 阴性对照：阈值够不着的大文件上，内联臂必须与不分流臂逐位相同。
    #[test]
    fn inline_is_a_noop_above_threshold() {
        let p = base(32);
        let (a, _) = run_case(p, Strategy::NoSplit, 1 << 20, Form::OnceWrite, 64, 3);
        let (b, _) = run_case(p, Strategy::InlineInode, 1 << 20, Form::OnceWrite, 64, 3);
        assert_eq!(a.dev_bytes(), b.dev_bytes());
        assert_eq!(a.read_ops, b.read_ops);
        assert_eq!(a.node_writes, b.node_writes);
    }

    /// 内联边界是 ≤（含阈值本身）：恰等于阈值的文件必须零数据块，多一字节必须整块
    /// （变异审计补的：此前没有任何测试落在 3584 的边界上，`<=` 改 `<` 不会红）。
    #[test]
    fn inline_boundary_is_inclusive() {
        let p = base(32);
        let (at, _) = run_case(p, Strategy::InlineInode, p.inline_max, Form::OnceWrite, 64, 3);
        assert_eq!(at.data_bytes, 0, "恰等于阈值该内联，零数据块");
        let (over, _) = run_case(p, Strategy::InlineInode, p.inline_max + 1, Form::OnceWrite, 64, 3);
        assert!(over.data_bytes >= 64 * p.block_bytes, "超过阈值一字节该走整块");
    }

    /// KV 分离在小文件上不占整块——它与不分流的差别必须真的出现在数据字节上。
    #[test]
    fn kv_separation_avoids_block_rounding() {
        let p = base(32);
        let (a, _) = run_case(p, Strategy::NoSplit, 64, Form::OnceWrite, 1024, 5);
        let (b, _) = run_case(p, Strategy::KvSeparate, 64, Form::OnceWrite, 1024, 5);
        assert!(b.data_bytes < a.data_bytes / 10);
    }

    /// 记账器是唯一出口：dev_bytes 必须恰等于两项之和，不许有第三条计数路径。
    #[test]
    fn ledger_has_one_exit() {
        let p = base(32);
        let (l, _) = run_case(p, Strategy::MessageLayer, 4096, Form::Append, 128, 9);
        assert_eq!(l.dev_bytes(), l.data_bytes + l.meta_bytes);
        assert!(l.node_writes > 0);
    }
}
