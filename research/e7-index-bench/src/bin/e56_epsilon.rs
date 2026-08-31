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

const O_DIRECT: i32 = 0o40000;
const PAGE: usize = 4096;
/// 节点 16 KiB：D8 已定项 2（2026-08-30 用户定案）钉成常量，不按设备特性算。
const NODE: usize = 16 * 1024;
/// pivot 条目 = key 8 + D19 指针头部 40。
const PIVOT_BYTES: usize = 48;
/// 消息 / 叶条目 = key 8 + value 8。与 E7 同口径，两个实验的数才可比。
const MSG_BYTES: usize = 16;
/// 节点头（条目数）占的字节。
const HDR_BYTES: usize = 16;
/// 叶容量：整个节点都用来装条目。
const LEAF_CAP: usize = (NODE - HDR_BYTES) / MSG_BYTES;

/// 叶数是运行期参数（第 5 个命令行参数），默认 1024。
/// ⚠️ **它是自变量而不是常量，因为 2026-08-31 三条独立论证腿里有两条都把
/// 「树只有 1024 个叶、缓存占 6.4%」列为这批结论最大的外推威胁**——
/// 那件事量得出来，不该只靠嘴上让步。main 在构造任何 Geom 之前设定它一次，此后不再改。
static mut N_LEAF_RT: usize = 1024;
fn nleaf() -> usize {
    unsafe { N_LEAF_RT }
}
/// 叶按半满装（B 树的常规占用率），不是 E7 那种「16 KiB 节点里只放 64 个 key」。
const KEYS_PER_LEAF: usize = LEAF_CAP / 2;
fn nkeys() -> u64 {
    (nleaf() * KEYS_PER_LEAF) as u64
}

/// 负载次数。**运行期参数（第 6 个命令行参数）**，默认 20 万。
/// ⚠️ 它必须远大于总缓冲容量，否则量到的不是稳态——判据与实测见
/// `.claude/kb/experiments/56-消息缓冲的收益vsε.md`「稳态可用区」。
static mut OPS_RT: u64 = 200_000;
fn ops() -> u64 {
    unsafe { OPS_RT }
}
const QN: u64 = 10_000;
const WB_CAP: usize = 512;
const CKPT: u64 = 4096;
/// 默认缓存节点数。E7 的 16 KiB 档用 66 个节点（约 1 MiB），这里沿用。
const DEFAULT_CACHE: usize = 66;

/// ε 扫描点，千分之一为单位。0.25 是解析式给的交点，两侧都要有档。
const EPS_SWEEP: [u32; 13] = [5, 50, 100, 150, 200, 250, 300, 400, 500, 650, 800, 900, 950];

// ── 几何：全部由 ε 导出 ──────────────────────────────────────────────

/// 一棵树的几何。`levels[0]` 是最靠近叶的那一层的节点数，最后一项恒为 1（根）。
#[derive(Debug, Clone, PartialEq)]
struct Geom {
    eps_permille: u32,
    fanout: usize,
    buf_cap: usize,
    levels: Vec<usize>,
}

fn fanout_of(eps_permille: u32) -> usize {
    let pivot_bytes = NODE * (1000 - eps_permille as usize) / 1000;
    (pivot_bytes / PIVOT_BYTES).max(2)
}

fn buf_of(eps_permille: u32) -> usize {
    let b = NODE * eps_permille as usize / 1000 / MSG_BYTES;
    // 编码进一个节点：条目数 × 16 + 头 ≤ NODE
    b.min(LEAF_CAP)
}

/// 自底向上按扇出收敛，返回每一层的节点数，最后一项恒为 1（根）。
///
/// ⚠️ **进度不变量写在函数自己里，不是写在单测里。**
/// 2026-08-31 变异测试：把 `div_ceil` 换成整除之后 `n` 掉到 0，`n == 1` 永远不成立，
/// 循环无限往 `v` 里塞 0 直到内存耗尽 ⇒ 测试进程被 signal 9 打死。
/// 那种「破坏被看见了，但不是断言抓到的」等于盲区：换个不 OOM 的同类错误就没人发现。
/// 断言放在单测里救不了它——函数根本不返回，单测里的断言执行不到。
fn levels_of(fanout: usize) -> Vec<usize> {
    assert!(fanout >= 2, "扇出必须 ≥ 2，否则树不收敛（实得 {fanout}）");
    let mut v = Vec::new();
    let mut n = nleaf();
    loop {
        let prev = n;
        n = n.div_ceil(fanout);
        assert!(n >= 1 && n < prev, "层数没有收敛：{prev} -> {n}（扇出 {fanout}）");
        v.push(n);
        if n == 1 {
            break;
        }
        assert!(v.len() <= 64, "层数超过 64，几何一定算错了（扇出 {fanout}）");
    }
    v
}

impl Geom {
    fn new(eps_permille: u32) -> Self {
        let fanout = fanout_of(eps_permille);
        Self { eps_permille, fanout, buf_cap: buf_of(eps_permille), levels: levels_of(fanout) }
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
    fn analytic_touch_per_op(&self) -> f64 {
        let ratio = self.fanout as f64 / self.buf_cap.max(1) as f64;
        self.height() as f64 * ratio.min(1.0)
    }
    /// 摊销倍数 B/F。< 1 表示一条也没摊到。
    fn amortization(&self) -> f64 {
        self.buf_cap as f64 / self.fanout as f64
    }
    // 页号：叶 0..叶数，然后 level0、level1 …，根在最后。
    fn leaf_pg(&self, i: usize) -> u64 {
        i as u64
    }
    fn level_base(&self, l: usize) -> u64 {
        nleaf() as u64 + self.levels[..l].iter().sum::<usize>() as u64
    }
    fn internal_pg(&self, l: usize, i: usize) -> u64 {
        self.level_base(l) + i as u64
    }
    /// key 落在哪个叶。
    fn leaf_of(&self, k: u64) -> usize {
        (k / KEYS_PER_LEAF as u64) as usize
    }
    /// 第 l 层里，key 落在哪个节点。l=0 是最靠近叶的那层。
    fn node_of(&self, l: usize, k: u64) -> usize {
        let mut idx = self.leaf_of(k);
        for _ in 0..=l {
            idx /= self.fanout;
        }
        idx
    }
    /// 内部节点的容量：只有缓冲那部分要落盘（pivot 是几何算出来的，不占模型里的条目槽）。
    fn internal_cap(&self) -> usize {
        self.buf_cap.max(1)
    }
}

// ── 设备与节点 ────────────────────────────────────────────────────────

struct Aligned {
    ptr: *mut u8,
    len: usize,
    layout: Layout,
}
impl Aligned {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, PAGE).unwrap();
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0, len) };
        Self { ptr, len, layout }
    }
    fn s(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
    fn m(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
impl Drop for Aligned {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) }
    }
}

#[derive(Clone)]
struct Node {
    n: usize,
    e: Vec<(u64, u64)>,
}
impl Node {
    fn new(cap: usize) -> Self {
        Self { n: 0, e: vec![(0, 0); cap] }
    }
    fn decode(buf: &Aligned, cap: usize) -> Self {
        let n = (u64::from_le_bytes(buf.s()[0..8].try_into().unwrap()) as usize).min(cap);
        let mut e = vec![(0u64, 0u64); cap];
        for (i, slot) in e.iter_mut().enumerate().take(n) {
            let o = HDR_BYTES + i * MSG_BYTES;
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
            let o = HDR_BYTES + i * MSG_BYTES;
            buf.m()[o..o + 8].copy_from_slice(&self.e[i].0.to_le_bytes());
            buf.m()[o + 8..o + 16].copy_from_slice(&self.e[i].1.to_le_bytes());
        }
    }
    fn cap(&self) -> usize {
        self.e.len()
    }
    fn full(&self) -> bool {
        self.n >= self.cap()
    }
    fn push(&mut self, k: u64, v: u64) {
        if self.n < self.cap() {
            self.e[self.n] = (k, v);
            self.n += 1;
        }
    }
    fn upsert(&mut self, k: u64, v: u64) {
        for i in 0..self.n {
            if self.e[i].0 == k {
                self.e[i].1 = v;
                return;
            }
        }
        self.push(k, v);
    }
    fn compact(&mut self) {
        bump_compaction();
        let mut m: std::collections::BTreeMap<u64, u64> = Default::default();
        for i in 0..self.n {
            m.insert(self.e[i].0, self.e[i].1);
        }
        self.n = 0;
        for (k, v) in m {
            self.push(k, v);
        }
    }
    fn get(&self, k: u64) -> Option<u64> {
        for i in (0..self.n).rev() {
            if self.e[i].0 == k {
                return Some(self.e[i].1);
            }
        }
        None
    }
}

struct Dev {
    f: std::fs::File,
    reads: u64,
    writes: u64,
    rbytes: u64,
    wbytes: u64,
}
impl Dev {
    fn rd(&mut self, pg: u64, b: &mut Aligned) {
        self.f.seek(SeekFrom::Start(pg * NODE as u64)).unwrap();
        self.f.read_exact(b.m()).unwrap();
        self.reads += 1;
        self.rbytes += NODE as u64;
    }
    fn wr(&mut self, pg: u64, b: &Aligned) {
        self.f.seek(SeekFrom::Start(pg * NODE as u64)).unwrap();
        self.f.write_all(b.s()).unwrap();
        self.writes += 1;
        self.wbytes += NODE as u64;
    }
}

/// 节点缓存 + checkpoint：三条臂共用，摊销架构对称（E7 硬要求 2）。
struct Cache {
    cap: usize,
    lru: Lru,
    map: std::collections::HashMap<u64, Node>,
}

/// 常驻集的历史最大值。**这是一条钉绝对值的读数**：它必须恰好等于
/// `min(声明的缓存节点数, 树的节点总数)`，三条臂各自都要满足——
/// 不是「三条臂互相比」（`.claude/singlefs-ai-sop/rules/test-discipline.md`）。
static mut RESIDENT_MAX: usize = 0;
fn note_resident(n: usize) {
    unsafe {
        if n > RESIDENT_MAX {
            RESIDENT_MAX = n;
        }
    }
}
fn resident_max() -> usize {
    unsafe { RESIDENT_MAX }
}
fn reset_resident() {
    unsafe { RESIDENT_MAX = 0 }
}
impl Cache {
    fn new(cap: usize) -> Self {
        Self { cap, lru: Lru::new(cap), map: Default::default() }
    }
    /// 取用一页。**`touch` 的返回值一律不许丢**——丢掉的那次逐出等于一次
    /// 「该写没写」，而它同时把被逐出的那一页变成 `map` 里 LRU 追踪不到的幽灵页，
    /// 幽灵页此后永远免费命中、还会繁殖出新的幽灵页。
    /// 2026-08-31 实测：丢返回值的写法下声明 66 个节点的缓存，
    /// 实际常驻集涨到 688（占树 67%），而三条臂互比看不出来——
    /// 只有 Bε 那条臂会在 get 与 put 之间嵌套别的缓存操作。
    fn get(&mut self, dev: &mut Dev, pg: u64, cap: usize, buf: &mut Aligned) -> Node {
        if let Some(n) = self.map.get(&pg).cloned() {
            let ev = self.lru.touch(pg);
            self.evict(dev, buf, ev);
            self.check_residency();
            note_resident(self.map.len());
            return n;
        }
        let ev = self.lru.touch(pg);
        self.evict(dev, buf, ev);
        dev.rd(pg, buf);
        let n = Node::decode(buf, cap);
        self.map.insert(pg, n.clone());
        self.check_residency();
        note_resident(self.map.len());
        n
    }
    /// 写回一页。**它必须和 `get` 走同一条逐出路径**：`put` 不过 LRU 的话，
    /// 一个在递归期间被逐出的页会被重新塞回 `map` 而 LRU 不再追踪它。
    fn put(&mut self, dev: &mut Dev, buf: &mut Aligned, pg: u64, n: Node) {
        let ev = self.lru.touch(pg);
        self.evict(dev, buf, ev);
        self.map.insert(pg, n);
        self.lru.mark_dirty(pg);
        self.check_residency();
        note_resident(self.map.len());
    }
    /// 常驻集不许超过声明的缓存。**三条臂共用这一条**，所以它不是臂间互比，
    /// 是把绝对值钉死（`.claude/singlefs-ai-sop/rules/test-discipline.md`）。
    fn check_residency(&self) {
        if self.map.len() > self.cap {
            eprintln!("常驻集闸破了：常驻 {} 超过声明的缓存 {}", self.map.len(), self.cap);
            std::process::exit(8);
        }
    }
    /// 逐出一页：脏就写回，然后从常驻集里摘掉。
    fn evict(&mut self, dev: &mut Dev, buf: &mut Aligned, ev: Option<u64>) {
        let Some(ev) = ev else { return };
        if self.lru.take_dirty(ev) {
            if let Some(nd) = self.map.get(&ev).cloned() {
                nd.encode(buf);
                dev.wr(ev, buf);
            }
        }
        self.map.remove(&ev);
    }
    fn checkpoint(&mut self, dev: &mut Dev, buf: &mut Aligned) {
        for pg in self.lru.drain_dirty() {
            if let Some(n) = self.map.get(&pg) {
                n.encode(buf);
                dev.wr(pg, buf);
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
        let v = (COMPACTIONS, FLUSH_EVENTS, FLUSH_CHILD_TOUCHES);
        COMPACTIONS = 0; FLUSH_EVENTS = 0; FLUSH_CHILD_TOUCHES = 0;
        v
    }
}

fn rnd(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *s = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// 把整棵树按给定几何铺到设备上。三条臂各自初始化，几何不同 ⇒ 页布局不同。
fn init_tree(dev: &mut Dev, g: &Geom) {
    let mut b = Aligned::new(NODE);
    for lf in 0..nleaf() {
        let mut n = Node::new(LEAF_CAP);
        for j in 0..KEYS_PER_LEAF {
            let k = (lf * KEYS_PER_LEAF + j) as u64;
            n.push(k, k);
        }
        n.encode(&mut b);
        dev.wr(g.leaf_pg(lf), &b);
    }
    let empty = Node::new(g.internal_cap());
    for l in 0..g.height() {
        for i in 0..g.levels[l] {
            empty.encode(&mut b);
            dev.wr(g.internal_pg(l, i), &b);
        }
    }
    dev.reads = 0;
    dev.writes = 0;
    dev.rbytes = 0;
    dev.wbytes = 0;
}

#[derive(Default, Clone)]
struct Run {
    reads: u64,
    writes: u64,
    rbytes: u64,
    wbytes: u64,
    touches: u64,
    ns: u64,
    /// 收尾时还停在各层缓冲里、从没到过叶子的消息条数。
    /// ⚠️ **这一项是 2026-08-31 反推腿指出来的缺口**：不数它，高 ε 档的
    /// io/op 会把「还没付的账」算成「省下来的钱」——而基线臂的 write buffer
    /// 在每个 checkpoint 与收尾都清空，**两条臂的摊销架构因此不对称**
    /// （E7 硬要求 2 要的正是对称）。
    residual: u64,
    /// 把残留全部推到叶子所付的设备 I/O。
    drain_io: u64,
    /// 块层独立读数，**只覆盖计费相**（预热相与数残留那两段不计）。
    blk: String,
}

// ── 臂 1：无批量对照（阳性对照）。写穿，每次更新都落盘。
fn arm_sorted(dev: &mut Dev, g: &Geom, ops: u64, seed: u64, cache: usize, touched: &mut [bool]) -> Run {
    let (r0, w0, rb0, wb0) = (dev.reads, dev.writes, dev.rbytes, dev.wbytes);
    let mut c = Cache::new(cache);
    let mut b = Aligned::new(NODE);
    let mut s = seed;
    let t = Instant::now();
    for _ in 0..ops {
        let k = rnd(&mut s) % nkeys();
        touched[k as usize] = true;
        let pg = g.leaf_pg(g.leaf_of(k));
        let mut n = c.get(dev, pg, LEAF_CAP, &mut b);
        n.upsert(k, k ^ 0xABCD);
        n.encode(&mut b);
        dev.wr(pg, &b);
        c.put(dev, &mut b, pg, n);
        c.lru.take_dirty(pg);
    }
    Run {
        reads: dev.reads - r0,
        writes: dev.writes - w0,
        rbytes: dev.rbytes - rb0,
        wbytes: dev.wbytes - wb0,
        touches: 0,
        ns: t.elapsed().as_nanos() as u64,
        residual: 0,
        drain_io: 0,
        blk: String::new(),
    }
}

// ── 臂 2：D8 现方向 —— 日志结构节点 + write buffer 前端。
fn arm_logstruct_wb(dev: &mut Dev, g: &Geom, ops: u64, seed: u64, cache: usize, wb_cap: usize, touched: &mut [bool]) -> Run {
    let (r0, w0, rb0, wb0) = (dev.reads, dev.writes, dev.rbytes, dev.wbytes);
    let mut c = Cache::new(cache);
    let mut b = Aligned::new(NODE);
    let mut s = seed;
    let mut wb: Vec<(u64, u64)> = Vec::with_capacity(wb_cap.max(1));
    let mut touches = 0u64;
    let t = Instant::now();
    let flush = |wb: &mut Vec<(u64, u64)>, c: &mut Cache, dev: &mut Dev, b: &mut Aligned, touches: &mut u64| {
        if wb.is_empty() {
            return;
        }
        // 稳定排序：unstable 会打乱同 key 的相对次序，于是「后者胜」胜出的是任意一条。
        // D8 已定项 3 要的 seq 正是为了这件事，本 harness 用稳定排序代替。
        wb.sort_by_key(|e| (e.0 / KEYS_PER_LEAF as u64, e.0));
        let mut dedup: Vec<(u64, u64)> = Vec::with_capacity(wb.len());
        for &(k, v) in wb.iter() {
            if let Some(last) = dedup.last_mut() {
                if last.0 == k {
                    last.1 = v;
                    continue;
                }
            }
            dedup.push((k, v));
        }
        let mut i = 0;
        while i < dedup.len() {
            let lf = (dedup[i].0 / KEYS_PER_LEAF as u64) as usize;
            let pg = lf as u64;
            let mut n = c.get(dev, pg, LEAF_CAP, b);
            *touches += 1;
            while i < dedup.len() && (dedup[i].0 / KEYS_PER_LEAF as u64) as usize == lf {
                if n.full() {
                    n.compact();
                }
                n.push(dedup[i].0, dedup[i].1);
                i += 1;
            }
            c.put(dev, b, pg, n);
        }
        wb.clear();
    };
    for i in 0..ops {
        let k = rnd(&mut s) % nkeys();
        touched[k as usize] = true;
        wb.push((k, k ^ 0xABCD));
        if wb.len() >= wb_cap.max(1) {
            flush(&mut wb, &mut c, dev, &mut b, &mut touches);
        }
        if (i + 1) % CKPT == 0 {
            flush(&mut wb, &mut c, dev, &mut b, &mut touches);
            c.checkpoint(dev, &mut b);
        }
    }
    flush(&mut wb, &mut c, dev, &mut b, &mut touches);
    c.checkpoint(dev, &mut b);
    let _ = g;
    Run {
        reads: dev.reads - r0,
        writes: dev.writes - w0,
        rbytes: dev.rbytes - rb0,
        wbytes: dev.wbytes - wb0,
        touches,
        ns: t.elapsed().as_nanos() as u64,
        residual: 0,
        drain_io: 0,
        blk: String::new(),
    }
}

// ── 臂 3：Bε —— 内部节点留 ε 比例的消息缓冲，消息沿树逐层下刷。
//    缓冲条数与扇出都由 ε 导出，两者争同一块节点字节 —— 这正是 E7 缺的那条张力。
#[allow(clippy::too_many_arguments)]
fn arm_betree(
    dev: &mut Dev,
    g: &Geom,
    ops: u64,
    seed: u64,
    cache: usize,
    buf_cap: usize,
    touched: &mut [bool],
    warmup: u64,
    dev_path: &str,
) -> Run {
    let mut b = Aligned::new(NODE);
    let mut s = seed;
    let cap = buf_cap.max(1).min(LEAF_CAP);
    // ── 预热相：先把各层缓冲填到稳态，**这一段的 I/O 不计费**。
    // ⚠️ 补它的理由（2026-08-31）：不预热的话各层缓冲一开始全空，进每个节点的前 B 条
    // 消息都是免费的，而**总缓冲容量可以超过整个负载**——实测 nleaf=8192、ε=0.95 时
    // 总缓冲容量约 50 万条 > 20 万次操作 ⇒ 残留 100%，一条消息都没到过叶子，
    // 量到的不是稳态写放大，是「把消息塞进缓冲的成本」，账还没到期。
    {
        let mut cw = Cache::new(cache);
        let mut sink = 0u64;
        for _ in 0..warmup {
            let k = rnd(&mut s) % nkeys();
            touched[k as usize] = true;
            push_down(dev, &mut cw, &mut b, g, g.height() - 1, 0, vec![(k, k ^ 0xABCD)], cap, &mut sink);
        }
        cw.checkpoint(dev, &mut b);
    }
    let residual_pre = count_residual(dev, g, cache, cap);
    // 预热与数残留的 I/O 一律抠掉：从这里开始才计费
    let (r0, w0, rb0, wb0) = (dev.reads, dev.writes, dev.rbytes, dev.wbytes);
    // ⚠️ **块层采样点必须在预热之后。** 采在外面会把不计费的预热相与数残留那两段
    // 算进块层那一侧，于是「程序计数器 = 块层读数」这条校验退化成「差不多」——
    // 2026-08-31 虚机首跑实测差 1991 读（ε=0.5），正是那两段。
    let blk_s0 = blkstat(dev_path);
    let mut c = Cache::new(cache);
    let mut touches = 0u64;
    reset_landed();
    let t = Instant::now();
    for i in 0..ops {
        let k = rnd(&mut s) % nkeys();
        touched[k as usize] = true;
        push_down(dev, &mut c, &mut b, g, g.height() - 1, 0, vec![(k, k ^ 0xABCD)], cap, &mut touches);
        if (i + 1) % CKPT == 0 {
            c.checkpoint(dev, &mut b);
        }
    }
    c.checkpoint(dev, &mut b);
    let blk = bdelta(blk_s0, blkstat(dev_path));
    Run {
        reads: dev.reads - r0,
        writes: dev.writes - w0,
        rbytes: dev.rbytes - rb0,
        wbytes: dev.wbytes - wb0,
        touches,
        ns: t.elapsed().as_nanos() as u64,
        residual: residual_pre,
        drain_io: 0,
        blk,
    }
}

/// 只数各层缓冲里现有多少条消息，不动手推。**调用方负责把它的 I/O 从计费里抠掉。**
fn count_residual(dev: &mut Dev, g: &Geom, cache: usize, cap: usize) -> u64 {
    let mut c = Cache::new(cache);
    let mut b = Aligned::new(NODE);
    let mut n = 0u64;
    for l in 0..g.height() {
        for idx in 0..g.levels[l] {
            n += c.get(dev, g.internal_pg(l, idx), cap, &mut b).n as u64;
        }
    }
    n
}

/// 收尾排空相：全新缓存，从盘上的状态把各层缓冲推到叶子，单独计费。
/// 用全新缓存是**保守**的——冷缓存下排空要多付读，给出的是上界。
fn drain_phase(dev: &mut Dev, g: &Geom, cache: usize, cap: usize) -> (u64, u64) {
    let mut c = Cache::new(cache);
    let mut b = Aligned::new(NODE);
    let mut touches = 0u64;
    let (r0, w0) = (dev.reads, dev.writes);
    let residual = drain_all(dev, &mut c, &mut b, g, cap, &mut touches);
    c.checkpoint(dev, &mut b);
    (residual, (dev.reads - r0) + (dev.writes - w0))
}

/// 把一批消息塞进第 `l` 层第 `idx` 个节点的缓冲；满了就按孩子分组下刷。
/// `l == 0` 的孩子是叶。每次取用一个节点记一次 touch。
#[allow(clippy::too_many_arguments)]
fn push_down(
    dev: &mut Dev,
    c: &mut Cache,
    b: &mut Aligned,
    g: &Geom,
    l: usize,
    idx: usize,
    msgs: Vec<(u64, u64)>,
    cap: usize,
    touches: &mut u64,
) {
    let pg = g.internal_pg(l, idx);
    let mut node = c.get(dev, pg, cap, b);
    *touches += 1;
    // ⚠️ **按「装到满就立刻下刷」分块装，不许一口气全塞进去。**
    // 一口气塞的写法在缓冲恰好装满时会**静默丢消息**（`Node::push` 满了就什么也不做），
    // 而内部缓冲里的 key 来自整个子树、重复率约 0.09%，压实几乎腾不出槽位。
    // 2026-08-31 由正推腿的「叶压实该是 0 次」这条盲预测查出来：
    // 实测压实计数非 0，追下去发现计的是**内部缓冲**的压实，而那正是丢消息的现场。
    let mut i = 0usize;
    while i < msgs.len() {
        if node.n >= cap {
            flush_node(dev, c, b, g, l, pg, &mut node, cap, touches);
        }
        let room = cap - node.n;
        let take = (msgs.len() - i).min(room);
        for m in &msgs[i..i + take] {
            node.push(m.0, m.1);
        }
        i += take;
        if node.n >= cap {
            flush_node(dev, c, b, g, l, pg, &mut node, cap, touches);
        }
    }
    c.put(dev, b, pg, node);
}

/// 把一个装满的缓冲整批推给孩子，推完把它清空。
/// **内部缓冲不做压实**：清空由下刷负责，压实只属于叶。
#[allow(clippy::too_many_arguments)]
fn flush_node(
    dev: &mut Dev,
    c: &mut Cache,
    b: &mut Aligned,
    g: &Geom,
    l: usize,
    pg: u64,
    node: &mut Node,
    cap: usize,
    touches: &mut u64,
) {
    let out: Vec<(u64, u64)> = node.e[..node.n].to_vec();
    node.n = 0;
    c.put(dev, b, pg, node.clone());
    let mut by_child: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = Default::default();
    for (k, v) in out {
        let child = if l == 0 { g.leaf_of(k) } else { g.node_of(l - 1, k) };
        by_child.entry(child).or_default().push((k, v));
    }
    bump_flush(by_child.len() as u64);
    for (child, es) in by_child {
        if l == 0 {
            let lpg = g.leaf_pg(child);
            let mut leaf = c.get(dev, lpg, LEAF_CAP, b);
            *touches += 1;
            for (k, v) in es {
                if leaf.full() {
                    leaf.compact();
                }
                leaf.push(k, v);
                bump_landed();
            }
            c.put(dev, b, lpg, leaf);
        } else {
            push_down(dev, c, b, g, l - 1, child, es, cap, touches);
        }
    }
}

/// 收尾排空：自顶向下把每一层缓冲里剩下的消息全部推到叶子。
/// 返回排空前还停在缓冲里的消息条数。
/// **它存在的理由是让两条攒批臂的摊销架构对称**：基线臂的 write buffer 在收尾时是空的，
/// 而 Bε 的各层缓冲不排空的话，高 ε 档等于有一大批更新的成本被实验窗口切掉了没算。
fn drain_all(
    dev: &mut Dev,
    c: &mut Cache,
    b: &mut Aligned,
    g: &Geom,
    cap: usize,
    touches: &mut u64,
) -> u64 {
    // ⚠️ **残留必须在动手推之前数完，不能边推边数。**
    // 边推边数会漏掉一类消息：上层排空时把消息推进下层，下层缓冲因此溢出、当场
    // 直接刷到叶子，等外层循环走到那一层时它们已经不在了 —— 于是 landed 计到、
    // residual 没计到。2026-08-31 守恒闸第一次真跑就是被这一条判红的
    // （landed_update=199869 residual=109，差 22 条）。
    let mut residual = 0u64;
    for l in 0..g.height() {
        for idx in 0..g.levels[l] {
            let node = c.get(dev, g.internal_pg(l, idx), cap, b);
            residual += node.n as u64;
        }
    }
    for l in (0..g.height()).rev() {
        for idx in 0..g.levels[l] {
            let pg = g.internal_pg(l, idx);
            let mut node = c.get(dev, pg, cap, b);
            *touches += 1;
            if node.n == 0 {
                continue;
            }
            let out: Vec<(u64, u64)> = node.e[..node.n].to_vec();
            node.n = 0;
            c.put(dev, b, pg, node);
            let mut by_child: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = Default::default();
            for (k, v) in out {
                let child = if l == 0 { g.leaf_of(k) } else { g.node_of(l - 1, k) };
                by_child.entry(child).or_default().push((k, v));
            }
            for (child, es) in by_child {
                if l == 0 {
                    let lpg = g.leaf_pg(child);
                    let mut leaf = c.get(dev, lpg, LEAF_CAP, b);
                    *touches += 1;
                    for (k, v) in es {
                        if leaf.full() {
                            leaf.compact();
                        }
                        leaf.push(k, v);
                        bump_landed();
                    }
                    c.put(dev, b, lpg, leaf);
                } else {
                    push_down(dev, c, b, g, l - 1, child, es, cap, touches);
                }
            }
        }
    }
    residual
}

/// 点查：根 → … → 叶，逐层看该 key 在不在这一层的缓冲里。每层一次节点读。
fn point_query(dev: &mut Dev, c: &mut Cache, b: &mut Aligned, g: &Geom, cap: usize, k: u64) -> u64 {
    let r0 = dev.reads;
    for l in (0..g.height()).rev() {
        let node = c.get(dev, g.internal_pg(l, g.node_of(l, k)), cap, b);
        if node.get(k).is_some() {
            return dev.reads - r0;
        }
    }
    let _ = c.get(dev, g.leaf_pg(g.leaf_of(k)), LEAF_CAP, b);
    dev.reads - r0
}

#[derive(Default, Clone, Copy)]
struct Blk {
    r: u64,
    w: u64,
}
fn blkstat(dev_path: &str) -> Option<Blk> {
    let name = dev_path.rsplit('/').next()?;
    let t = std::fs::read_to_string(format!("/sys/block/{name}/stat")).ok()?;
    let f: Vec<u64> = t.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    if f.len() < 8 {
        return None;
    }
    Some(Blk { r: f[0], w: f[4] })
}
fn bdelta(a: Option<Blk>, b: Option<Blk>) -> String {
    match (a, b) {
        (Some(a), Some(b)) => format!("blk_r={} blk_w={}", b.r - a.r, b.w - a.w),
        _ => "blk_r=NA blk_w=NA".into(),
    }
}

/// 查询阶段：热 key（更新阶段碰过的）与冷 key（没碰过的）各一组，缓存都是新的。
/// ⚠️ 冷 key 那一列正是 E7 自陈没量的那一格。
fn query_phase(
    dev: &mut Dev,
    g: &Geom,
    cache: usize,
    cap: usize,
    touched: &[bool],
    seed: u64,
) -> (f64, f64, u64, u64) {
    let hot: Vec<u64> = (0..nkeys()).filter(|&k| touched[k as usize]).collect();
    let cold: Vec<u64> = (0..nkeys()).filter(|&k| !touched[k as usize]).collect();
    let mut out = [0f64; 2];
    let mut cnt = [0u64; 2];
    for (i, set) in [&hot, &cold].iter().enumerate() {
        let mut c = Cache::new(cache);
        let mut b = Aligned::new(NODE);
        let mut s = seed ^ (0x9E37 + i as u64);
        let r0 = dev.reads;
        if set.is_empty() {
            out[i] = f64::NAN;
            continue;
        }
        for _ in 0..QN {
            let k = set[(rnd(&mut s) % set.len() as u64) as usize];
            let _ = point_query(dev, &mut c, &mut b, g, cap, k);
        }
        out[i] = (dev.reads - r0) as f64 / QN as f64;
        cnt[i] = set.len() as u64;
    }
    (out[0], out[1], cnt[0], cnt[1])
}

fn main() {
    let dev_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e56_epsilon <块设备或文件> [种子] [none|selfcheck|nodirect] [缓存节点数] [叶数] [负载次数]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0x51561234);
    let mode = std::env::args().nth(3).unwrap_or_else(|| "none".into());
    let selfcheck = mode == "selfcheck";
    let use_direct = mode != "nodirect";
    let cache: usize = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(DEFAULT_CACHE);
    // 第 5 个参数：叶数。**在构造任何 Geom 之前设定，此后不再改。**
    let n_leaf: usize = std::env::args().nth(5).and_then(|x| x.parse().ok()).unwrap_or(1024);
    unsafe { N_LEAF_RT = n_leaf.max(2) };
    // 第 6 个参数：负载次数。必须远大于总缓冲容量，见「稳态可用区」。
    let ops_arg: u64 = std::env::args().nth(6).and_then(|x| x.parse().ok()).unwrap_or(200_000);
    unsafe { OPS_RT = ops_arg.max(1) };

    let mut oo = OpenOptions::new();
    oo.read(true).write(true);
    if use_direct {
        oo.custom_flags(O_DIRECT);
    }
    let f = oo.open(&dev_path).unwrap_or_else(|e| {
        eprintln!("打不开 {dev_path}: {e}");
        std::process::exit(3)
    });
    let mut dev = Dev { f, reads: 0, writes: 0, rbytes: 0, wbytes: 0 };
    let size = dev.f.seek(SeekFrom::End(0)).unwrap();
    // 最大的那套几何（ε 最小 ⇒ 扇出最大 ⇒ 内部节点最少；ε 最大 ⇒ 内部最多）都要装得下
    let need = EPS_SWEEP.iter().map(|&e| Geom::new(e).total_nodes()).max().unwrap()
        .max(Geom::new(0).total_nodes()) as u64
        * NODE as u64;
    if size < need {
        eprintln!("设备太小：需要 {need} 字节，实有 {size}");
        std::process::exit(5);
    }

    let mut em = Emitter::new();
    let base = Geom::new(0);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config node_bytes={NODE} pivot_bytes={PIVOT_BYTES} msg_bytes={MSG_BYTES} \
             leaf_cap={LEAF_CAP} keys_per_leaf={KEYS_PER_LEAF} n_leaf={} keys={} \
             ops={} qn={QN} wb_cap={WB_CAP} ckpt={CKPT} cache_nodes={cache} \
             base_fanout={} base_height={} base_nodes={} cache_ratio={:.4} \
             seed={seed} mode={mode} o_direct={use_direct} blkstat={}",
            nleaf(),
            nkeys(),
            ops(),
            base.fanout,
            base.height(),
            base.total_nodes(),
            cache as f64 / base.total_nodes() as f64,
            blkstat(&dev_path).is_some()
        ))
    );

    let mut touched = vec![false; nkeys() as usize];

    // 基线两条臂用 ε=0 的几何：不留缓冲，节点字节全给 pivot。
    for (name, which) in [("sorted_bplus", 0u8), ("logstruct_wb", 1u8)] {
        touched.iter_mut().for_each(|t| *t = false);
        init_tree(&mut dev, &base);
        let s0 = blkstat(&dev_path);
        let r = if which == 0 {
            arm_sorted(&mut dev, &base, ops(), seed, cache, &mut touched)
        } else {
            arm_logstruct_wb(&mut dev, &base, ops(), seed, cache, if selfcheck { 1 } else { WB_CAP }, &mut touched)
        };
        let blk = bdelta(s0, blkstat(&dev_path));
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=update arm={name} eps=0 fanout={} buf=0 height={} reads={} writes={} \
                 io={} io_per_op={:.6} bytes_per_op={:.1} touch_per_op={:.6} analytic_touch=NA elapsed_ns={} {blk}",
                base.fanout,
                base.height(),
                r.reads,
                r.writes,
                r.reads + r.writes,
                (r.reads + r.writes) as f64 / ops() as f64,
                (r.rbytes + r.wbytes) as f64 / ops() as f64,
                r.touches as f64 / ops() as f64,
                r.ns
            ))
        );
        let s1 = blkstat(&dev_path);
        let (hot, cold, nhot, ncold) = query_phase(&mut dev, &base, cache, base.internal_cap(), &touched, seed);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=query arm={name} eps=0 hot_reads_per_query={hot:.4} cold_reads_per_query={cold:.4} \
                 hot_keys={nhot} cold_keys={ncold} {}",
                bdelta(s1, blkstat(&dev_path))
            ))
        );
    }

    // Bε 臂：ε 扫描。selfcheck 把缓冲判据换成常量 1。
    for eps in EPS_SWEEP {
        let g = Geom::new(eps);
        let cap = if selfcheck { 1 } else { g.internal_cap() };
        touched.iter_mut().for_each(|t| *t = false);
        init_tree(&mut dev, &g);
        let _ = take_counters();
        reset_landed();
        reset_resident();
        // 预热到稳态：把各层缓冲填满两遍，让下刷级联在每一层都真的跑过几轮。
        // 判据是「填满该臂自己的缓冲」而不是「跑固定次数」——稳态是状态，不是操作数。
        let warmup = (g.total_internal() as u64) * (cap as u64) * 2;
        let mut r = arm_betree(&mut dev, &g, ops(), seed, cache, cap, &mut touched, warmup, &dev_path);
        let residual_pre = r.residual;
        let blk = r.blk.clone();
        let cnt = take_counters();
        let landed_update = landed();
        // 排空前先量一次点查：这是「缓冲里还有在途消息」的树态
        let (hot0, cold0, _, _) = query_phase(&mut dev, &g, cache, cap, &touched, seed);
        // 再排空，单独计费（反推腿指出的缺口）
        let (residual, drain_io) = drain_phase(&mut dev, &g, cache, cap);
        r.residual = residual;
        r.drain_io = drain_io;
        // 守恒闸：落到叶的消息数加上排空时从缓冲里捡出来的，必须恰好等于操作数。
        // 少一条就是有消息被静默丢了——而丢消息在 io/op 上长得像「更省」。
        // ⚠️ 这道闸是补的：2026-08-31 之前没有它，`push_down` 在缓冲恰好装满时
        // 静默丢消息，而三条臂互比看不出来（丢消息只让 Bε 那条臂显得更便宜）。
        let want_resident = cache.min(g.total_nodes());
        if resident_max() > want_resident {
            eprintln!(
                "常驻集闸破了：eps={eps} resident_max={} 声明缓存={cache} 树节点={} 该是 {want_resident}",
                resident_max(),
                g.total_nodes()
            );
            std::process::exit(8);
        }
        // 守恒（带预热的形态）：计费相进树的消息数 = ops()；
        // 落到叶的 + 缓冲净增 必须恰好等于它：
        //   landed_计费相 + (residual_post − residual_pre) == ops()
        if landed_update + residual < residual_pre
            || landed_update + residual - residual_pre != ops()
        {
            eprintln!(
                "守恒破了：eps={eps} landed_update={landed_update} residual_pre={residual_pre} residual_post={residual} ops={}", ops()
            );
            std::process::exit(7);
        }
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=update arm=betree eps={eps} fanout={} buf={} height={} internal_nodes={} \
                 reads={} writes={} io={} io_per_op={:.6} bytes_per_op={:.1} touch_per_op={:.6} \
                 analytic_touch={:.6} amortization={:.4} residual={} residual_frac={:.4} \
                 drain_io={} io_per_op_drained={:.6} residual_pre={residual_pre} warmup={warmup} \
                 leaf_compactions={} flush_events={} \
                 children_per_flush={:.4} landed_update={landed_update} conservation=ok \
                 elapsed_ns={} {blk}",
                g.fanout,
                cap,
                g.height(),
                g.total_internal(),
                r.reads,
                r.writes,
                r.reads + r.writes,
                (r.reads + r.writes) as f64 / ops() as f64,
                (r.rbytes + r.wbytes) as f64 / ops() as f64,
                r.touches as f64 / ops() as f64,
                g.analytic_touch_per_op(),
                g.amortization(),
                r.residual,
                r.residual as f64 / ops() as f64,
                r.drain_io,
                (r.reads + r.writes + r.drain_io) as f64 / ops() as f64,
                cnt.0,
                cnt.1,
                if cnt.1 == 0 { 0.0 } else { cnt.2 as f64 / cnt.1 as f64 },
                r.ns
            ))
        );
        let s1 = blkstat(&dev_path);
        let (hot, cold, nhot, ncold) = query_phase(&mut dev, &g, cache, cap, &touched, seed);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=query arm=betree eps={eps} height={} hot_reads_per_query={hot:.4} \
                 cold_reads_per_query={cold:.4} predrain_hot={hot0:.4} predrain_cold={cold0:.4} \
                 hot_keys={nhot} cold_keys={ncold} {}",
                g.height(),
                bdelta(s1, blkstat(&dev_path))
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言 1：几何常量。** 它们一改，下面每一条解析值都静默作废，
    /// 而 D11 引的数正是这套算术算出来的（E7 的教训：几何常量此前没有任何断言钉住）。
    #[test]
    fn geometry_constants_are_pinned() {
        assert_eq!(NODE, 16384, "D8 已定项 2 钉的是 16 KiB");
        assert_eq!(PIVOT_BYTES, 48, "key 8 + D19 指针头部 40");
        assert_eq!(MSG_BYTES, 16, "key 8 + value 8，与 E7 同口径");
        assert_eq!(LEAF_CAP, 1023);
        assert_eq!(KEYS_PER_LEAF, 511, "叶按半满装");
        assert_eq!(nleaf(), 1024, "叶数默认值");
        assert_eq!(nkeys(), 523_264);
    }

    /// **绝对值断言 2：交点恰好落在 ε = M/(M+P) = 0.25，且此处 B == F。**
    /// 这是跑前写死的预测 1，实测若与它相左，要么模型没按几何算、要么解析错。
    #[test]
    fn the_crossover_is_exactly_at_eps_one_quarter() {
        let analytic_eps = MSG_BYTES as f64 / (MSG_BYTES + PIVOT_BYTES) as f64;
        assert!((analytic_eps - 0.25).abs() < 1e-12, "解析交点算出 {analytic_eps}");
        let g = Geom::new(250);
        assert_eq!(g.fanout, 256, "ε=0.25 的扇出");
        assert_eq!(g.buf_cap, 256, "ε=0.25 的缓冲条数");
        assert_eq!(g.buf_cap, g.fanout, "交点处 B 必须恰好等于 F");
        assert!((g.amortization() - 1.0).abs() < 1e-12, "交点处摊销倍数恰为 1");
    }

    /// **绝对值断言 3：摊销倍数 = (P/M)·ε/(1−ε) = 3ε/(1−ε)。**
    /// 逐档与几何算出来的 B/F 对，容差只留给整除的取整。
    #[test]
    fn amortization_follows_three_eps_over_one_minus_eps() {
        for e in [100u32, 250, 500, 750, 900] {
            let g = Geom::new(e);
            let x = e as f64 / 1000.0;
            let closed = 3.0 * x / (1.0 - x);
            let rel = (g.amortization() - closed).abs() / closed;
            assert!(rel < 0.02, "ε={x}：几何给 {}，闭式给 {closed}", g.amortization());
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
        for (e, f, b, h, touch) in table {
            let g = Geom::new(e);
            assert_eq!(g.fanout, f, "ε={e}‰ 的扇出");
            assert_eq!(g.buf_cap, b, "ε={e}‰ 的缓冲条数");
            assert_eq!(g.height(), h, "ε={e}‰ 的层数");
            assert!((g.analytic_touch_per_op() - touch).abs() < 1e-9, "ε={e}‰ 的解析触碰数");
        }
    }

    /// **ε ≤ 0.25 的那一段，解析触碰数恒等于层数**——「一条也没摊到」的可判定形态。
    /// 自检（B=1）在这一段**预期不变**，正是因为这里本来就没有摊销。
    #[test]
    fn below_the_crossover_there_is_no_amortization_at_all() {
        for e in [5u32, 50, 100, 150, 200, 250] {
            let g = Geom::new(e);
            assert!(g.amortization() <= 1.0, "ε={e}‰ 不该有摊销");
            assert!(
                (g.analytic_touch_per_op() - g.height() as f64).abs() < 1e-12,
                "ε={e}‰ 的触碰数该恰好等于层数 {}",
                g.height()
            );
        }
    }

    /// **层数序列必须严格收敛到 1，每层至少 1 个节点。**
    /// ⚠️ 补这一条的理由（2026-08-31 变异测试）：把 `div_ceil` 换成整除的那条变异
    /// （M3）只让测试进程被 signal 9 打死——`n` 掉到 0 之后 `n == 1` 永远不成立，
    /// 循环无限往 `levels` 里塞 0 直到内存耗尽。**破坏被看见了，但不是断言抓到的**，
    /// 那是个盲区：换个形状的同类错误（例如少收敛一层）就不会 OOM，也就不会被发现。
    #[test]
    fn levels_converge_strictly_to_a_single_root() {
        for f in [2usize, 3, 17, 34, 68, 119, 170, 204, 238, 256, 273, 290, 307, 324, 339, 341] {
            let lv = levels_of(f);
            assert!(!lv.is_empty(), "扇出 {f} 的层数为空");
            assert!(lv.len() <= 16, "扇出 {f} 的层数 {} 不合理", lv.len());
            assert_eq!(*lv.last().unwrap(), 1, "扇出 {f} 的最顶层必须恰好 1 个根");
            for (i, &n) in lv.iter().enumerate() {
                assert!(n >= 1, "扇出 {f} 第 {i} 层是 {n}，每层至少 1 个节点");
            }
            for w in lv.windows(2) {
                assert!(w[1] < w[0], "扇出 {f} 的层数没有严格收敛：{:?}", lv);
            }
            // 绝对值：第 0 层的节点数恰好是 ceil(叶数 / 扇出)
            assert_eq!(lv[0], nleaf().div_ceil(f), "扇出 {f} 的第 0 层节点数");
        }
        // 手算的两个绝对值
        assert_eq!(levels_of(341), vec![4, 1]);
        assert_eq!(levels_of(17), vec![61, 4, 1]);
    }

    /// 页号不许重叠：一次写踩掉另一个节点时，实验只数 I/O 次数、看不出内容被踩。
    #[test]
    fn page_ranges_do_not_overlap() {
        for e in [5u32, 250, 500, 950] {
            let g = Geom::new(e);
            let mut seen = std::collections::HashSet::new();
            for i in 0..nleaf() {
                assert!(seen.insert(g.leaf_pg(i)), "ε={e}‰：叶 {i} 页号重叠");
            }
            for l in 0..g.height() {
                for i in 0..g.levels[l] {
                    assert!(seen.insert(g.internal_pg(l, i)), "ε={e}‰：第 {l} 层第 {i} 个页号重叠");
                }
            }
            assert_eq!(seen.len(), g.total_nodes());
        }
    }

    /// key → 各层节点的映射必须落在界内，且逐层收敛到根。
    #[test]
    fn key_maps_into_every_level_in_range() {
        for e in [5u32, 500, 950] {
            let g = Geom::new(e);
            for k in [0u64, 1, KEYS_PER_LEAF as u64, nkeys() / 2, nkeys() - 1] {
                assert!(g.leaf_of(k) < nleaf());
                for l in 0..g.height() {
                    assert!(g.node_of(l, k) < g.levels[l], "ε={e}‰ key={k} 第 {l} 层越界");
                }
                assert_eq!(g.node_of(g.height() - 1, k), 0, "最顶层只能是根");
            }
        }
    }

    /// 缓冲永远编码得进一个节点：条目数 × 16 + 头 ≤ 16384。
    #[test]
    fn buffer_always_fits_in_one_node() {
        for e in 0..=999u32 {
            let g = Geom::new(e);
            assert!(g.buf_cap * MSG_BYTES + HDR_BYTES <= NODE, "ε={e}‰ 的缓冲装不进节点");
            assert!(g.fanout >= 2, "ε={e}‰ 的扇出塌到 1，树就不收敛了");
        }
    }

    /// **无批量对照臂的解析 io/op**：每次操作恰好 1 次写 + (1 − 缓存/叶数) 次读。
    /// 缓存 66、叶 1024 ⇒ 1 + (1 − 66/1024) = 1.935547。
    #[test]
    fn the_no_batching_control_arm_has_an_analytic_io_per_op() {
        assert_eq!(nleaf(), 1024);
        assert_eq!(DEFAULT_CACHE, 66);
        let analytic = 1.0 + (1.0 - DEFAULT_CACHE as f64 / nleaf() as f64);
        assert!((analytic - 1.935_546_875).abs() < 1e-9, "解析式算出 {analytic}");
    }

    /// **绝对值断言 5：高 ε 档能欠下多大一笔账。**
    /// 各层缓冲加起来最多能扣住 total_internal × B 条消息，这些消息在收尾时
    /// 还没到过叶子。ε=0.95 时是 66 × 972 = 64152 条，占 20 万次操作的 32%
    /// ⇒ **不做收尾排空的话，高 ε 档的 io/op 把「还没付的账」算成了「省下的钱」。**
    /// 这条算术就是 2026-08-31 反推腿指出缺口时用的那条，钉在这里免得它只活在散文里。
    #[test]
    fn the_unpaid_buffer_debt_at_high_eps_is_a_third_of_the_workload() {
        let g = Geom::new(950);
        assert_eq!(g.total_internal(), 66, "ε=0.95 的内部节点数");
        assert_eq!(g.buf_cap, 972);
        assert_eq!(g.total_internal() * g.buf_cap, 64_152);
        let frac = (g.total_internal() * g.buf_cap) as f64 / ops() as f64;
        assert!(frac > 0.32, "欠账占比实算 {frac}");
        // 反过来：低 ε 档几乎不欠账，所以那一段的数不受这个缺口影响
        let lo = Geom::new(50);
        assert!((lo.total_internal() * lo.buf_cap) as f64 / (ops() as f64) < 0.002);
    }

    /// **绝对值断言 6：三条臂落在同一条曲线上。**
    /// 2026-08-31 正推腿在**没看过任何实测**的情况下推出
    /// `io/op = 2(1 − e^(−x))/x`，x = 一次下刷的消息数 / 该节点的孩子数：
    /// 臂 A 是 x→0（每条消息独占一次下刷），臂 B 是 x = 512/1024 = 0.5，
    /// 臂 C 是 x = B/F = 3ε/(1−ε)。
    /// ⇒ **「要不要消息缓冲」其实是「x 取多少」**，write buffer 前端本身就是 x=0.5 的 Bε。
    #[test]
    fn all_three_arms_lie_on_one_flush_curve() {
        let curve = |x: f64| 2.0 * (1.0 - (-x).exp()) / x;
        // 臂 B：512 条消息摊到 1024 个叶
        let arm_b = curve(WB_CAP as f64 / nleaf() as f64);
        assert!((arm_b - 1.5739).abs() < 1e-3, "臂 B 的闭式算出 {arm_b}，正推腿盲推的是 1.574");
        // 臂 A：x→0 的极限是 2
        assert!((curve(1e-9) - 2.0).abs() < 1e-6);
        // 臂 C 在 ε=0.25 上：x = B/F = 1，摊销因子 1 − e^(−1) = 0.632
        let g = Geom::new(250);
        let x = g.buf_cap as f64 / g.fanout as f64;
        assert!((x - 1.0).abs() < 1e-12, "ε=0.25 处 x 必须恰为 1");
        assert!((curve(x) - 1.2642).abs() < 1e-3, "闭式算出 {}", curve(x));
        // 臂 C 追平臂 B 的点在 x = 0.5，即 3ε/(1−ε) = 0.5 ⇒ ε = 1/7
        let eps_cross: f64 = 1.0 / 7.0;
        let x_cross = 3.0 * eps_cross / (1.0 - eps_cross);
        assert!((x_cross - 0.5).abs() < 1e-12, "追平点算出 x={x_cross}");
    }

    /// 节点编解码要能原样回来，否则「消息真的下去了」这件事就没根据。
    #[test]
    fn node_round_trips_through_the_page() {
        let mut n = Node::new(256);
        for i in 0..200u64 {
            n.push(i, i ^ 0xABCD);
        }
        let mut b = Aligned::new(NODE);
        n.encode(&mut b);
        let d = Node::decode(&b, 256);
        assert_eq!(d.n, 200);
        assert_eq!(d.get(199), Some(199 ^ 0xABCD));
        assert_eq!(d.get(200), None);
    }

    /// 日志结构的「后者胜」：同 key 后写的必须赢，否则去重口径是错的。
    #[test]
    fn later_write_wins_within_a_node() {
        let mut n = Node::new(8);
        n.push(7, 1);
        n.push(7, 2);
        assert_eq!(n.get(7), Some(2));
        n.compact();
        assert_eq!(n.get(7), Some(2), "压实之后仍该是后写的那个");
        assert_eq!(n.n, 1, "压实该把同 key 合成一条");
    }
}
