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

const O_DIRECT: i32 = 0o40000;
const PAGE: usize = 4096;
/// 节点占几个 4 KiB 页 —— 运行期参数（D8 已定：节点大小是每套布局的**格式参数**）。
/// 用 static mut 是因为它在 main 里定一次就不再变；所有读取都在单线程里。
static mut NODE_PAGES: usize = 1;
fn npages() -> usize { unsafe { NODE_PAGES } }
fn nbytes() -> usize { npages() * PAGE }
fn slots() -> usize { (nbytes() - 16) / 16 }
const FANOUT: usize = 32;
const N_LEAF: usize = 1024; // FANOUT^2
const KEYS_PER_LEAF: usize = 64;
const N_KEYS: u64 = (N_LEAF * KEYS_PER_LEAF) as u64;
/// 缓存页数的默认值。**测试要钉的解析值 1.9375 就是按它算的**——
/// 抽成常量是为了让测试绑住**生产用的那个数**，而不是在测试里再抄一遍
/// （2026-08-29：e9 那边正因为在测试里重抄了一遍逻辑，变异测试一条都没红）。
const DEFAULT_CACHE_PAGES: usize = 64;

// 盘上布局：页 0 = 根，页 1..=16 = 内部节点，页 17..=272 = 叶子
fn root_pg() -> u64 { 0 }
fn internal_pg(i: usize) -> u64 { 1 + i as u64 }
fn leaf_pg(i: usize) -> u64 { 1 + FANOUT as u64 + i as u64 }
fn leaf_of(k: u64) -> usize { (k / KEYS_PER_LEAF as u64) as usize }
fn internal_of(leaf: usize) -> usize { leaf / FANOUT }

struct Aligned { ptr: *mut u8, len: usize, layout: Layout }
impl Aligned {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, PAGE).unwrap();
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0, len) };
        Self { ptr, len, layout }
    }
    fn s(&self) -> &[u8] { unsafe { std::slice::from_raw_parts(self.ptr, self.len) } }
    fn m(&mut self) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) } }
}
impl Drop for Aligned { fn drop(&mut self) { unsafe { dealloc(self.ptr, self.layout) } } }

/// 一个节点的内存形态：条目数 + 条目。日志结构臂用它做「追加」，排序臂用它做「原地」。
#[derive(Clone)]
struct Node { n: usize, e: Vec<(u64, u64)> }
impl Node {
    fn new() -> Self { Self { n: 0, e: vec![(0, 0); slots()] } }
    fn decode(buf: &Aligned) -> Self {
        let n = u64::from_le_bytes(buf.s()[0..8].try_into().unwrap()) as usize;
        let n = n.min(slots());
        let mut e = vec![(0u64, 0u64); slots()];
        for (i, slot) in e.iter_mut().enumerate().take(n) {
            let o = 16 + i * 16;
            *slot = (
                u64::from_le_bytes(buf.s()[o..o + 8].try_into().unwrap()),
                u64::from_le_bytes(buf.s()[o + 8..o + 16].try_into().unwrap()),
            );
        }
        Self { n, e }
    }
    fn encode(&self, buf: &mut Aligned) {
        buf.m()[0..8].copy_from_slice(&(self.n as u64).to_le_bytes());
        for i in 0..self.n {
            let o = 16 + i * 16;
            buf.m()[o..o + 8].copy_from_slice(&self.e[i].0.to_le_bytes());
            buf.m()[o + 8..o + 16].copy_from_slice(&self.e[i].1.to_le_bytes());
        }
    }
    fn full(&self) -> bool { self.n >= slots() }
    fn push(&mut self, k: u64, v: u64) { if self.n < slots() { self.e[self.n] = (k, v); self.n += 1; } }
    /// 原地更新（排序臂）：找到就改，找不到就追加
    fn upsert(&mut self, k: u64, v: u64) {
        for i in 0..self.n { if self.e[i].0 == k { self.e[i].1 = v; return; } }
        self.push(k, v);
    }
    /// 压实（日志结构臂）：同 key 保留最后一条
    fn compact(&mut self) {
        let mut m: std::collections::BTreeMap<u64, u64> = Default::default();
        for i in 0..self.n { m.insert(self.e[i].0, self.e[i].1); }
        self.n = 0;
        for (k, v) in m { self.push(k, v); }
    }
    /// 点查：日志结构下要**从后往前**扫（后写的胜）
    fn get(&self, k: u64) -> Option<u64> {
        for i in (0..self.n).rev() { if self.e[i].0 == k { return Some(self.e[i].1); } }
        None
    }
}

struct Dev { f: std::fs::File, reads: u64, writes: u64, rbytes: u64, wbytes: u64 }
impl Dev {
    fn rd(&mut self, pg: u64, b: &mut Aligned) {
        self.f.seek(SeekFrom::Start(pg * nbytes() as u64)).unwrap();
        self.f.read_exact(b.m()).unwrap();
        self.reads += 1;
        self.rbytes += nbytes() as u64;
    }
    fn wr(&mut self, pg: u64, b: &Aligned) {
        self.f.seek(SeekFrom::Start(pg * nbytes() as u64)).unwrap();
        self.f.write_all(b.s()).unwrap();
        self.writes += 1;
        self.wbytes += nbytes() as u64;
    }
}

/// 节点缓存 + checkpoint：三臂共用，保证摊销架构对称。
struct Cache { lru: Lru, map: std::collections::HashMap<u64, Node> }
impl Cache {
    fn new(cap: usize) -> Self { Self { lru: Lru::new(cap), map: Default::default() } }
    fn get(&mut self, dev: &mut Dev, pg: u64, buf: &mut Aligned) -> Node {
        if let Some(n) = self.map.get(&pg) { self.lru.touch(pg); return n.clone(); }
        if let Some(ev) = self.lru.touch(pg) {
            if self.lru.take_dirty(ev) {
                let nd = self.map.get(&ev).unwrap().clone();
                nd.encode(buf); dev.wr(ev, buf);
            }
            self.map.remove(&ev);
        }
        dev.rd(pg, buf);
        let n = Node::decode(buf);
        self.map.insert(pg, n.clone());
        n
    }
    fn put(&mut self, pg: u64, n: Node) { self.map.insert(pg, n); self.lru.mark_dirty(pg); }
    /// checkpoint：把全部脏节点刷下去。三臂在同一节奏上调用它。
    fn checkpoint(&mut self, dev: &mut Dev, buf: &mut Aligned) {
        for pg in self.lru.drain_dirty() {
            if let Some(n) = self.map.get(&pg) { n.encode(buf); dev.wr(pg, buf); }
        }
    }
}

fn rnd(s: &mut u64) -> u64 {
    let mut x = *s; x ^= x >> 12; x ^= x << 25; x ^= x >> 27; *s = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

// ── 臂 1：朴素排序节点 B+tree，无批量。这是**阳性对照**：效应必须在这里出现。
//    它故意不共用 checkpoint 节奏——「无批量」正是它要代表的那一档。
fn arm_sorted(dev: &mut Dev, ops: u64, seed: u64, cache_pages: usize) -> (u64, u64) {
    let (r0, w0) = (dev.reads, dev.writes);
    let mut c = Cache::new(cache_pages);
    let mut b = Aligned::new(nbytes());
    let mut s = seed;
    for _ in 0..ops {
        let k = rnd(&mut s) % N_KEYS;
        let pg = leaf_pg(leaf_of(k));
        let mut n = c.get(dev, pg, &mut b);
        n.upsert(k, k ^ 0xABCD);
        n.encode(&mut b);
        dev.wr(pg, &b); // 写穿：每次更新都落盘
        c.put(pg, n);
        c.lru.take_dirty(pg); // 已经写过了，不再算脏
    }
    (dev.reads - r0, dev.writes - w0)
}

// ── 臂 2：D8 现方向 —— 日志结构节点（节点内追加不插入）+ write buffer 前端。
fn arm_logstruct_wb(
    dev: &mut Dev, ops: u64, seed: u64, cache_pages: usize, wb_cap: usize, ckpt: u64,
    shuffle: bool,
) -> (u64, u64) {
    let (r0, w0) = (dev.reads, dev.writes);
    let mut c = Cache::new(cache_pages);
    let mut b = Aligned::new(nbytes());
    let mut s = seed;
    let mut wb: Vec<(u64, u64)> = Vec::with_capacity(wb_cap.max(1));
    let flush = |wb: &mut Vec<(u64, u64)>, c: &mut Cache, dev: &mut Dev, b: &mut Aligned| {
        if wb.is_empty() { return; }
        // 攒批 → 排序 → 去重（后者胜），正是 D8 write buffer 的形态
        wb.sort_unstable_by_key(|e| (leaf_of(e.0), e.0));
        let mut dedup: Vec<(u64, u64)> = Vec::with_capacity(wb.len());
        for &(k, v) in wb.iter() {
            if let Some(last) = dedup.last_mut() { if last.0 == k { last.1 = v; continue; } }
            dedup.push((k, v));
        }
        if shuffle {
            // 干预：**先按叶分组、再打乱组的顺序**，同叶聚合原样保留，只改叶子被访问的次序。
            // 这样两个模式的叶子取用次数完全相同，唯一变量就是顺序——
            // 若命中率因此回来，就证实「排序后的 flush 是一次顺序扫描，
            // 而 LRU 对反复的、大于缓存的顺序扫描命中率为 0」。
            let mut groups: Vec<Vec<(u64, u64)>> = Vec::new();
            let mut i = 0;
            while i < dedup.len() {
                let lf = leaf_of(dedup[i].0);
                let mut g = Vec::new();
                while i < dedup.len() && leaf_of(dedup[i].0) == lf { g.push(dedup[i]); i += 1; }
                groups.push(g);
            }
            let mut st = 0x1234_5678_9ABC_DEF0u64 ^ (groups.len() as u64);
            for i in (1..groups.len()).rev() {
                let j = (rnd(&mut st) % (i as u64 + 1)) as usize;
                groups.swap(i, j);
            }
            dedup = groups.into_iter().flatten().collect();
        }
        let mut i = 0;
        while i < dedup.len() {
            let lf = leaf_of(dedup[i].0);
            let pg = leaf_pg(lf);
            let mut n = c.get(dev, pg, b);
            while i < dedup.len() && leaf_of(dedup[i].0) == lf {
                if n.full() { n.compact(); }    // 日志结构：满了才压实
                n.push(dedup[i].0, dedup[i].1); // 追加，不插入
                i += 1;
            }
            c.put(pg, n);
        }
        wb.clear();
    };
    for i in 0..ops {
        let k = rnd(&mut s) % N_KEYS;
        wb.push((k, k ^ 0xABCD));
        if wb.len() >= wb_cap.max(1) { flush(&mut wb, &mut c, dev, &mut b); }
        if (i + 1) % ckpt == 0 { flush(&mut wb, &mut c, dev, &mut b); c.checkpoint(dev, &mut b); }
    }
    flush(&mut wb, &mut c, dev, &mut b);
    c.checkpoint(dev, &mut b);
    (dev.reads - r0, dev.writes - w0)
}

// ── 臂 3：Bε —— 内部节点带消息缓冲，消息沿树下刷。
fn arm_betree(
    dev: &mut Dev, ops: u64, seed: u64, cache_pages: usize, buf_cap: usize, ckpt: u64,
) -> (u64, u64) {
    let (r0, w0) = (dev.reads, dev.writes);
    let mut c = Cache::new(cache_pages);
    let mut b = Aligned::new(nbytes());
    let mut s = seed;
    let cap = buf_cap.min(slots());
    for i in 0..ops {
        let k = rnd(&mut s) % N_KEYS;
        let mut root = c.get(dev, root_pg(), &mut b);
        root.push(k, k ^ 0xABCD);
        // 根缓冲满 → 下刷到 16 个内部节点
        if root.n >= cap.max(1) {
            let msgs: Vec<(u64, u64)> = root.e[..root.n].to_vec();
            root.n = 0;
            c.put(root_pg(), root);
            for iid in 0..FANOUT {
                let mine: Vec<(u64, u64)> =
                    msgs.iter().copied().filter(|(k, _)| internal_of(leaf_of(*k)) == iid).collect();
                if mine.is_empty() { continue; }
                let ipg = internal_pg(iid);
                let mut inode = c.get(dev, ipg, &mut b);
                for (k, v) in mine { inode.push(k, v); }
                // 内部缓冲满 → 下刷到它的 16 个叶子
                if inode.n >= cap.max(1) {
                    let m2: Vec<(u64, u64)> = inode.e[..inode.n].to_vec();
                    inode.n = 0;
                    c.put(ipg, inode);
                    let mut by_leaf: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = Default::default();
                    for (k, v) in m2 { by_leaf.entry(leaf_of(k)).or_default().push((k, v)); }
                    for (lf, es) in by_leaf {
                        let pg = leaf_pg(lf);
                        let mut n = c.get(dev, pg, &mut b);
                        for (k, v) in es { if n.full() { n.compact(); } n.push(k, v); }
                        c.put(pg, n);
                    }
                } else {
                    c.put(ipg, inode);
                }
            }
        } else {
            c.put(root_pg(), root);
        }
        if (i + 1) % ckpt == 0 { c.checkpoint(dev, &mut b); }
    }
    c.checkpoint(dev, &mut b);
    (dev.reads - r0, dev.writes - w0)
}

/// 点查：三臂共用同一条读路径口径——根 → 内部 → 叶子，逐层看有没有该 key。
fn point_query(dev: &mut Dev, c: &mut Cache, b: &mut Aligned, k: u64) -> u64 {
    let r0 = dev.reads;
    let root = c.get(dev, root_pg(), b);
    if root.get(k).is_none() {
        let inode = c.get(dev, internal_pg(internal_of(leaf_of(k))), b);
        if inode.get(k).is_none() {
            let _ = c.get(dev, leaf_pg(leaf_of(k)), b);
        }
    }
    dev.reads - r0
}

#[derive(Default, Clone, Copy)]
struct Blk { r: u64, w: u64 }
fn blkstat(dev_path: &str) -> Option<Blk> {
    let name = dev_path.rsplit('/').next()?;
    let t = std::fs::read_to_string(format!("/sys/block/{name}/stat")).ok()?;
    let f: Vec<u64> = t.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    if f.len() < 8 { return None; }
    Some(Blk { r: f[0], w: f[4] })
}
fn bdelta(a: Option<Blk>, b: Option<Blk>) -> String {
    match (a, b) {
        (Some(a), Some(b)) => format!("blk_r={} blk_w={}", b.r - a.r, b.w - a.w),
        _ => "blk_r=NA blk_w=NA".into(),
    }
}

fn main() {
    let dev_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e7-index <块设备> [种子] [none|selfcheck|nodirect] [缓存页数]"); std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0x51561234);
    let mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    // selfcheck：把两条攒批臂的核心判据换成常量（缓冲=1），结果必须显著变化；
    // 没变化就说明那个结构根本没参与——kb/experiments.md E7 硬要求 1。
    let selfcheck = mode == "selfcheck";
    let use_direct = mode != "nodirect";

    let mut oo = OpenOptions::new();
    oo.read(true).write(true);
    if use_direct { oo.custom_flags(O_DIRECT); }
    let f = oo.open(&dev_path).unwrap_or_else(|e| { eprintln!("打不开 {dev_path}: {e}"); std::process::exit(3) });
    let mut dev = Dev { f, reads: 0, writes: 0, rbytes: 0, wbytes: 0 };
    let size = dev.f.seek(SeekFrom::End(0)).unwrap();
    if size < (1 + FANOUT + N_LEAF) as u64 * nbytes() as u64 { eprintln!("设备太小：需要 {} 字节", (1 + FANOUT + N_LEAF) * nbytes()); std::process::exit(5); }

    let mut em = Emitter::new();
    let ops: u64 = 200_000;
    // 缓存/树比是本实验最承重的自变量：缓存住整棵树会让三条臂都好看，量不出结构差异。
    // 第 4 个参数指定缓存页数，不给则用 64（约占 1057 个节点的 6%）。
    let cache_pages: usize = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(DEFAULT_CACHE_PAGES);
    // 第 5 个参数：一个节点占几个 4 KiB 页。节点大小是 D8 已定的每套布局格式参数。
    let node_pages: usize = std::env::args().nth(5).and_then(|x| x.parse().ok()).unwrap_or(1);
    unsafe { NODE_PAGES = node_pages.max(1) };
    let wb_cap = if selfcheck { 1 } else { 512 };
    let buf_cap = if selfcheck { 1 } else { 200 };
    let ckpt: u64 = 4096;
    println!("{}", em.emit_raw(&format!(
        "name=config nodes={} node_kib={} slots={} keys={N_KEYS} ops={ops} cache_nodes={cache_pages} cache_ratio={:.3} \
         wb_cap={wb_cap} betree_buf={buf_cap} ckpt={ckpt} seed={seed} mode={mode} \
         o_direct={use_direct} blkstat={}", 1 + FANOUT + N_LEAF, nbytes()/1024, slots(), cache_pages as f64 / (1 + FANOUT + N_LEAF) as f64, blkstat(&dev_path).is_some())));

    // 初始化：把每个叶子填上它那 64 个 key
    {
        let mut b = Aligned::new(nbytes());
        for lf in 0..N_LEAF {
            let mut n = Node::new();
            for j in 0..KEYS_PER_LEAF { let k = (lf * KEYS_PER_LEAF + j) as u64; n.push(k, k); }
            n.encode(&mut b); dev.wr(leaf_pg(lf), &b);
        }
        let empty = Node::new();
        for i in 0..FANOUT { empty.encode(&mut b); dev.wr(internal_pg(i), &b); }
        empty.encode(&mut b); dev.wr(root_pg(), &b);
        dev.reads = 0; dev.writes = 0; dev.rbytes = 0; dev.wbytes = 0; // 初始化不计入
    }

    for (name, which) in [("sorted_bplus", 0), ("logstruct_wb", 1), ("betree", 2)] {
        let s0 = blkstat(&dev_path);
        let (rb0, wb0) = (dev.rbytes, dev.wbytes);
        let t = Instant::now();
        let (r, w) = match which {
            0 => arm_sorted(&mut dev, ops, seed, cache_pages),
            1 => arm_logstruct_wb(&mut dev, ops, seed, cache_pages, wb_cap, ckpt, mode == "shuffle"),
            _ => arm_betree(&mut dev, ops, seed, cache_pages, buf_cap, ckpt),
        };
        let ns = t.elapsed().as_nanos() as u64;
        let blk = bdelta(s0, blkstat(&dev_path));
        println!("{}", em.emit_raw(&format!(
            "name=update arm={name} reads={r} writes={w} io={} io_per_op={:.6} bytes_per_op={:.1} elapsed_ns={ns} {blk}",
            r + w, (r + w) as f64 / ops as f64, (dev.rbytes - rb0 + dev.wbytes - wb0) as f64 / ops as f64)));

        // 点查：同一批 key，冷缓存
        let mut c = Cache::new(cache_pages);
        let mut b = Aligned::new(nbytes());
        let mut qs = seed ^ 0x9E37;
        let qn: u64 = 20_000;
        let s1 = blkstat(&dev_path);
        let (qr0, t2) = (dev.reads, Instant::now());
        let mut hits = 0u64;
        for _ in 0..qn { hits += point_query(&mut dev, &mut c, &mut b, rnd(&mut qs) % N_KEYS); }
        let qns = t2.elapsed().as_nanos() as u64;
        println!("{}", em.emit_raw(&format!(
            "name=query arm={name} reads={} reads_per_query={:.4} elapsed_ns={qns} {} probe_reads={hits}",
            dev.reads - qr0, (dev.reads - qr0) as f64 / qn as f64, bdelta(s1, blkstat(&dev_path)))));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
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
        assert_eq!(N_LEAF, 1024, "叶数变了，1.9375 这个解析值跟着失效");
        assert_eq!(FANOUT * FANOUT, N_LEAF, "几何不自洽");
        assert_eq!(DEFAULT_CACHE_PAGES, 64, "缓存页默认值变了，1.9375 这个解析值跟着失效");
        let analytic = 1.0 + (1.0 - DEFAULT_CACHE_PAGES as f64 / N_LEAF as f64);
        assert!((analytic - 1.9375).abs() < 1e-12,
                "解析式算出 {analytic}，而 kb 与实测对的是 1.9375");
    }

    #[test]
    fn page_ranges_do_not_overlap() {
        let mut seen = std::collections::HashSet::new();
        assert!(seen.insert(root_pg()), "根页号重复");
        for i in 0..FANOUT { assert!(seen.insert(internal_pg(i)), "内部节点 {i} 页号与别人重叠"); }
        for i in 0..N_LEAF { assert!(seen.insert(leaf_pg(i)), "叶 {i} 页号与别人重叠"); }
        assert_eq!(seen.len(), 1 + FANOUT + N_LEAF);
    }

    /// **key → 叶 → 内部节点这条映射必须覆盖全部 key，且落在合法范围内。**
    /// 越界会让实验读到不属于树的页，而 I/O 计数照样是「正常的一次读」。
    #[test]
    fn every_key_maps_into_range() {
        for k in [0u64, 1, KEYS_PER_LEAF as u64 - 1, KEYS_PER_LEAF as u64, N_KEYS - 1] {
            let l = leaf_of(k);
            assert!(l < N_LEAF, "key {k} 映射到叶 {l}，超出 {N_LEAF}");
            let i = internal_of(l);
            assert!(i < FANOUT, "叶 {l} 映射到内部节点 {i}，超出 {FANOUT}");
        }
        // 每个叶恰好装 KEYS_PER_LEAF 个 key，且相邻 key 不会跨叶跳号
        assert_eq!(leaf_of(0), leaf_of(KEYS_PER_LEAF as u64 - 1));
        assert_eq!(leaf_of(KEYS_PER_LEAF as u64), 1);
    }

    /// 树的几何必须自洽：`FANOUT^2 == N_LEAF`，且 key 总数 = 叶数 × 每叶 key 数。
    /// 几何写错时，「一次点查走几层」这个被测量就不是设想的那个。
    #[test]
    fn tree_geometry_is_self_consistent() {
        assert_eq!(FANOUT * FANOUT, N_LEAF, "FANOUT^2 应当等于叶数");
        assert_eq!(N_KEYS, (N_LEAF * KEYS_PER_LEAF) as u64);
    }

    /// **`O_DIRECT` 要求缓冲按页对齐**；不对齐会 `EINVAL`，
    /// 而那表现为「I/O 计数为 0」——看起来像很省，实际是根本没跑。
    #[test]
    fn aligned_buffer_is_page_aligned() {
        for len in [PAGE, PAGE * 8] {
            let a = Aligned::new(len);
            assert_eq!(a.ptr as usize % PAGE, 0, "长度 {len} 的缓冲没有按页对齐");
        }
    }
}
