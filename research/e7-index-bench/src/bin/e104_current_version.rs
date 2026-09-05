//! E104：扫描重建时多版单元的现行版本判定 —— C113 定案提案（第一版）P1–P6 的机制验证。
//!
//! **它答的是**：只剩单元 + 记账 + 根的时候，同一逻辑身份的多个物理版本里哪一版是哪个根的
//! 现行版本，提案给的那组输入（写序、实例水位、祖先表、死亡写序、容器退役记录、scrub 门）
//! 够不够；每一样是不是都承重。E59 乙臂的「记账」是写入时置好的 `live: bool` 神谕，
//! 这里把它换成从盘上算出来的机制，并保留神谕臂当阳性对照。
//!
//! ## 逐字贴的被引条款（verify-before-claiming.md：不许照印象建模）
//!
//! - D5 定义表：`birth(b)` = 块被**发布**的那个 checkpoint 号，「不是写请求发出时所在的开放 txg——
//!   崩溃后该号会被重发」。
//! - D16 已定项 6：每次发布 checkpoint_txg + 1；已定项 7：持久顺序恒为
//!   COW 单元 → 屏障 → journal 记录 → 屏障 → 根槽。
//! - D23 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位；实例代号每次恢复递增。
//!   已定项 14：恢复只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录，jsn 严格连续、断号即止。
//! - D18 已定项 1：谱系重写序号不做；射程是「为分辨陈旧副本而设的序号」。
//! - D18 已定项 10：墓碑「在它指认的死亡已被全部还引用它的快照送走之后可回收」。
//! - D8 已定项 6：类型 2 容器身份 (出生树, 2, 容器号, 出生代)，同一头内 COW 重写身份不变，
//!   合并只许左吸收右、右半退役；inode 号单调不复用。
//! - D3 已定项 1 / D21 已定项 5：谓词的权威记录点是运行时口径；分配记录是派生态。
//!
//! ## 判据（跑前写死，写在提案 P8，跑完不许改）
//!
//! 1. 全规则臂对全部根、全部种子：错版本 = 复活 = 判死实活 = 歧义 = 0。任一非 0 ⇒ 提案不完整，
//!    记下是哪一类世界，不许回头改规则再跑同一个实验。
//! 2. 每条消融臂在它对应的靶子世界上必须 > 0，且数值钉闭式；闭式由世界生成器独立计数，
//!    不从被测代码读回。
//! 3. 神谕臂恒 0；扫到的单元数 ≠ 独立算出的应有数 ⇒ 整轮作废；注入 k 处破坏必须报出恰 k。
//! 4. 若全规则臂在某类世界上非 0 而消融臂也非 0，缺的构件不在六个之内，提案回到靶子重画。
//!
//! ## 它测的是什么、不是什么
//!
//! 计数模型，零文件操作，没有并发。「盘」是 loc → 单元的映射，复用一个 loc 就是把旧头盖掉。
//! 真值由生成器在每个根发布那一刻记下，不经过任何重建代码。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

type Tree = u32;
type Txg = u64;

/// 写序 = 点名该单元的 journal 记录的 jsn（实例代号 32 + 计数器 48）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct WSeq {
    inst: u32,
    ctr: u64,
}

/// 逻辑身份。数据单元 = (对象 ID, 对象出生代)，锚点恒 0；容器 = 码 3 的四元组。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Key {
    Data { obj: u64, obj_birth: Txg },
    Cont { birth_tree: Tree, kind: u8, no: u64, birth: Txg },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TombRec {
    /// 对象死亡：指认 key，带死亡写序与死亡代号。
    Kill { key: Key, death: WSeq, death_txg: Txg },
    /// 容器退役（打包记录类型 3）。第三版去掉了退役写序：身份不复用，退役不需要与什么比大小；
    /// 对某个根可见与否由装着它的容器版本的诞生代号决定。`at_txg` 只给回收门用。
    Retire { cont: Key, at_txg: Txg },
}

#[derive(Clone, Debug)]
enum Payload {
    /// 数据单元的载荷内容不参与判定（现行与否看落点），不建模
    Data,
    Inodes(BTreeMap<u64, u64>),
    Tomb(Vec<TombRec>),
}

#[derive(Clone, Debug)]
struct Unit {
    key: Key,
    writer: Tree,
    birth: Txg,
    wseq: WSeq,
    payload: Payload,
}

/// 一个根（可写头的最新发布态，或一个快照）的真值视图。
#[derive(Clone, Debug, Default)]
struct View {
    tree: Tree,
    txg: Txg,
    /// 对象 → 落点
    data: BTreeMap<Key, u64>,
    /// inode 号 → (容器身份, 容器落点, 内容版本)
    inodes: BTreeMap<u64, (Key, u64, u64)>,
    /// 这个根引用的全部落点（含墓碑容器）：神谕臂用它当 `live`
    alloc: BTreeSet<u64>,
    /// 回退时重建头要用的：容器与墓碑容器的身份 / 落点、下一个墓碑容器号
    conts: Vec<(Key, u64)>,
    tomb_open: Option<(Key, u64)>,
    tomb_closed: Vec<(Key, u64)>,
    next_tomb_no: u64,
}

#[derive(Clone, Debug)]
struct Cont {
    key: Key,
    loc: u64,
    recs: BTreeMap<u64, u64>,
    dirty: bool,
    /// 这片叶是不是从 origin 共享来的（还没被本头 COW 过）
    shared: bool,
}

#[derive(Clone, Debug)]
struct Head {
    tree: Tree,
    data: BTreeMap<Key, u64>,
    conts: Vec<Cont>,
    /// 当前开放的墓碑容器（身份, 落点, 记录）
    tomb_open: Option<(Key, u64, Vec<TombRec>)>,
    /// 已关闭但仍持有记录的墓碑容器
    tomb_closed: Vec<(Key, u64, Vec<TombRec>)>,
    /// 本 txg 内已施加事务的暂存视图（发布时提升为真值）
    next_tomb_no: u64,
}

/// 写路径与回收纪律的开关——每个都对应提案里一条纪律，关掉是为了证明它承重。
#[derive(Clone, Copy, Debug)]
struct Cfg {
    /// P6：跨头首次 COW 让容器重生
    rebirth: bool,
    /// P5：合并 / 清空时写容器退役记录
    retire_record: bool,
    /// P4：墓碑回收多一道抹头水位门
    scrub_gate: bool,
    /// P4 第二版：门的下限取「被杀版本变成垃圾的时刻」= max(死亡代号, 最近一次根被销毁的代号)，
    /// 而不只是死亡代号（第一版的写法，E104 第一轮在混合世界上打出复活）
    gate_v2: bool,
    /// W1 第二句：事务内先写后删的单元在记录写出前作废
    invalidate_in_txn: bool,
    /// 恢复时给根实例到新实例之间的每个实例写水位行
    write_watermarks: bool,
    /// 回退时丢掉被抛弃时间线的 defer 条目。关掉也不出错：释放时的引用检查会把它们放进 deadlist——
    /// 这条等价性有单测留档，所以变异表不拿它当变异
    retain_abandoned_defer: bool,
    /// 分配器复用释放空间的概率（万分比）
    reuse_bp: u32,
    /// 每个单元写几份副本（D2 已定项 6：w ≥ 2；模型默认 1，靶子世界开 2）
    w: u8,
    /// 回退后第一个新根的 txg 取根环里全部根 txg 的最大值 + 1（关掉 = 取 T_old + 1，被抛弃的根会在下次择新时赢）
    rollback_skips_ring: bool,
    /// 第六版 P1 失败表 ③：一个事务在发出单元写之后失败 ⇒ 实例结束，号更大的事务的记录一条都不再追加
    /// （记录按事务号顺序追加）。关掉 = 第五版的形态：后续事务照常追加，失败事务的作废信息随崩溃丢失
    records_stop_at_failure: bool,
    /// 第六版 P2 行回收：kind 0 行可删还要求根环里没有该实例发布的根（第五轮反推腿 4.1）
    row_reclaim_checks_ring: bool,
    /// 第八版 P2：恢复行的 W 按实例分——只有所选根那个实例的 W 可能非 0（jsn 在实例边界必然断号）。
    /// 关掉 = 第七版落地那句的字面：一个全局的「最大已施加事务号」写给每一个实例（第七轮反推腿 3.1）
    row_w_per_instance: bool,
    /// 第八版 P3：码 3 的择新键取 (诞生代号, 实例代号)。关掉 = 第七版的「取写序最大者」——
    /// 同实例连着两个 checkpoint 都不分配新事务号时两版容器写序逐字节相同（第七轮反推腿 2.1）
    cont_key_by_birth: bool,
    /// 生成器手工冻结容器写序的事务号那一维（只有第七轮反推腿 2.1 的世界用）：
    /// 真实形态里容器只在 checkpoint 的固定点写出、写序 = 那次 checkpoint 已分配的最大事务号，
    /// 模型按事务写容器，冻结是把「同一 checkpoint 内写序不变」这个性质手工造出来
    cont_wseq_frozen: bool,
    /// 第七版 P2：恢复行的 T_pub 取所选根的 checkpoint_txg。关掉 = 第六版的字面「在飞 txg − 1」——
    /// 所选根不是最新已发布根时（最新根的槽坏了、它之后的记录又不可重放），把丢掉的那个 checkpoint 判成已发布（第六轮反推腿 2.1）
    row_tpub_from_chosen_root: bool,
}

impl Cfg {
    const FULL: Cfg = Cfg {
        rebirth: true,
        retire_record: true,
        scrub_gate: true,
        gate_v2: true,
        invalidate_in_txn: true,
        write_watermarks: true,
        retain_abandoned_defer: true,
        reuse_bp: 5000,
        w: 1,
        rollback_skips_ring: true,
        records_stop_at_failure: true,
        row_reclaim_checks_ring: true,
        row_tpub_from_chosen_root: true,
        row_w_per_instance: true,
        cont_key_by_birth: true,
        cont_wseq_frozen: false,
    };
}

const CONT_CAP: usize = 4;

/// 确定性伪随机源（xorshift64*）。
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
    fn chance_bp(&mut self, bp: u32) -> bool {
        self.below(10_000) < bp as u64
    }
}

/// 分配记录条目的两种状态（D3 已定项 7 的 value 加一位：已释放 + 释放代）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocState {
    Allocated,
    Freed(Txg),
}

/// 一个事务里对一个头的操作（生成器给的，已按 W1 合并成每 key 至多一条净效果）。
#[derive(Clone, Copy, Debug)]
enum Op {
    CreateObj,
    Overwrite(Key),
    DeleteObj(Key),
    /// 先把新版本写到盘上，再在同一事务里删掉——只在单测里构造
    #[cfg_attr(not(test), allow(dead_code))]
    WriteThenDelete(Key),
    /// 一个已死的 key 重新写出（同一对象的同一锚点在 truncate 之后再写）
    Recreate(Key),
    CreateInode,
    DeleteInode(u64),
}

struct World {
    cfg: Cfg,
    rng: Rng,
    disk: BTreeMap<u64, Unit>,
    next_loc: u64,
    free_pool: VecDeque<u64>,
    /// 本 txg 内产生、发布后才能进 free_pool 的释放
    pending_free: Vec<u64>,
    /// defer 窗口：发布后再等 DEFER 次发布才进可分配集合（I-7.4 的 K 代保护，回退用）
    defer: VecDeque<(Txg, Vec<u64>)>,
    /// 最近几次发布的全部根视图（回退用）：(发布的 txg, 实例, 头视图, 当时的快照, 当时的祖先表)
    history: Vec<(Txg, u32, BTreeMap<Tree, View>, Vec<View>, BTreeMap<Tree, (Tree, Txg)>)>,
    /// 回退抛弃的根仍留在根环里：(txg, 实例)。下次择新按 txg 最大者——新时间线的根必须压过它们
    abandoned_roots: Vec<(Txg, u32)>,
    /// 生成器指定下一次分配复用哪个落点（模拟分配器把某个垃圾落点盖掉）
    force_reuse: Option<u64>,
    /// 主落点 → 该单元全部副本的落点（含主落点本身）
    replicas: BTreeMap<u64, Vec<u64>>,
    /// 运行时分配记录：落点 → 已分配 / 已释放(释放代)。第四版 P4：释放时不删条目，value 改成已释放 + 释放代
    alloc_rec: BTreeMap<u64, AllocState>,
    /// 只被快照引用的落点：快照销毁时才回到 free_pool
    deadlisted: BTreeSet<u64>,
    inst: u32,
    ctr: u64,
    txg: Txg,
    heads: BTreeMap<Tree, Head>,
    /// 可写头的最新发布真值
    head_views: BTreeMap<Tree, View>,
    snaps: Vec<View>,
    ancestry: BTreeMap<Tree, (Tree, Txg)>,
    /// 实例表：实例代号 → (最后发布的 txg T_pub, 在飞 txg 里最后施加的计数器 W)。
    /// 崩溃行 = (在飞 txg − 1, W)，回退行 = (T_old, 0)，中间实例 = (0, 0)。谓词见 `row_published`
    watermarks: BTreeMap<u32, (Txg, u64)>,
    /// 每个以崩溃结束的实例：(最后发布的 txg, 在飞 txg 有没有被部分重放)——inst_only 臂能拿到的全部信息
    inst_last_txg: BTreeMap<u32, (Txg, bool)>,
    /// 哪些行是回退行（第八版：不许被后来的恢复覆盖）
    rollback_rows: BTreeSet<u32>,
    /// 回退行保护实际挡下过几次写（第八轮反推腿 12.1：这条规则今天就量得出，不必等崩溃点重放）
    rollback_row_writes_blocked: u32,
    /// 冻结容器写序时用的事务号（`cont_wseq_frozen`）
    frozen_cont_ctr: u64,
    /// 本 txg 里有没有已施加的事务
    applied_in_txg: bool,
    scrub_wm: Txg,
    /// 最近一次销毁根（快照）的 txg：被它独占的旧版本从那一刻起才是垃圾
    last_root_removal_txg: Txg,
    /// 崩溃那一刻之前、本实例最后一条被施加的计数器值（恢复时写成水位）
    last_applied_ctr: u64,
    /// 上一个根的实例代号（恢复时给 [它, 新实例) 都写行）
    root_inst: u32,
    next_tree: Tree,
    next_obj: u64,
    next_inode: u64,
    // ── 生成器独立计数（闭式的分母，不从重建代码读回）──
    n_abandoned_unrewritten: u64,
    n_rollback_abandoned: u64,
    n_written_units: u64,
    n_invalidated: u64,
    n_overwritten_locs: u64,
}

/// 写一行；回退行按第八版 P2 不许被后来的恢复覆盖（第七轮反推腿 9.1）。
/// ⚠️ 模型没有 journal，那条攻击真正的害处（被抛弃的记录被重放回来、回退被静默撤销）落在 E104 的度量之外，
/// 这里只实现规则、没有能分辨它的世界（见「它答不了的」）。
fn put_row(rows: &mut BTreeMap<u32, (Txg, u64)>, sticky: &BTreeSet<u32>, protect: bool, i: u32, r: (Txg, u64), blocked: &mut u32) {
    if protect && sticky.contains(&i) { *blocked += 1; return; }
    rows.insert(i, r);
}

/// 第七版 P2 的已发布谓词（实例表那一半）：行 (T_pub, W) 说「恢复所选的根是该实例的 T_pub，从它的 tail 起连续重放、
/// 施加到事务号 W 为止」。数据单元（码 1）按写出它的事务判：b ≤ T_pub（在所选根里），或 n ≤ W（被重放施加；
/// 事务号在实例内单调、记录按事务号顺序追加，W 是前缀）；容器（码 3）只在 checkpoint 的固定点写出，
/// 所选根之后的固定点单元一律未发布、由恢复实例重写：b ≤ T_pub。
fn row_published(rows: &BTreeMap<u32, (Txg, u64)>, u: &Unit) -> bool {
    let Some((tp, wc)) = rows.get(&u.wseq.inst) else { return true; };
    match u.key {
        Key::Data { .. } => u.birth <= *tp || u.wseq.ctr <= *wc,
        Key::Cont { .. } => u.birth <= *tp,
    }
}

impl World {
    fn new(seed: u64, cfg: Cfg) -> Self {
        let mut w = World {
            cfg,
            rng: Rng::new(seed),
            disk: BTreeMap::new(),
            next_loc: 1_000,
            free_pool: VecDeque::new(),
            pending_free: Vec::new(),
            defer: VecDeque::new(),
            history: Vec::new(),
            abandoned_roots: Vec::new(),
            force_reuse: None,
            replicas: BTreeMap::new(),
            alloc_rec: BTreeMap::new(),
            deadlisted: BTreeSet::new(),
            inst: 1,
            ctr: 0,
            txg: 1,
            heads: BTreeMap::new(),
            head_views: BTreeMap::new(),
            snaps: Vec::new(),
            ancestry: BTreeMap::new(),
            watermarks: BTreeMap::new(),
            inst_last_txg: BTreeMap::new(),
            rollback_rows: BTreeSet::new(),
            rollback_row_writes_blocked: 0,
            frozen_cont_ctr: 0,
            applied_in_txg: false,
            scrub_wm: 0,
            last_root_removal_txg: 0,
            last_applied_ctr: 0,
            root_inst: 1,
            next_tree: 1,
            next_obj: 1,
            next_inode: 1,
            n_abandoned_unrewritten: 0,
            n_rollback_abandoned: 0,
            n_written_units: 0,
            n_invalidated: 0,
            n_overwritten_locs: 0,
        };
        let t = w.new_tree();
        w.heads.insert(t, Head {
            tree: t,
            data: BTreeMap::new(),
            conts: Vec::new(),
            tomb_open: None,
            tomb_closed: Vec::new(),
            next_tomb_no: 1,
        });
        w
    }

    fn new_tree(&mut self) -> Tree {
        let t = self.next_tree;
        self.next_tree += 1;
        t
    }

    fn alloc(&mut self) -> u64 {
        if let Some(l) = self.force_reuse.take() {
            self.free_pool.retain(|x| *x != l);
            if self.disk.remove(&l).is_some() { self.n_overwritten_locs += 1; }
            return l;
        }
        if !self.free_pool.is_empty() && self.rng.chance_bp(self.cfg.reuse_bp) {
            let l = self.free_pool.pop_front().unwrap();
            if self.disk.remove(&l).is_some() { self.n_overwritten_locs += 1; }
            l
        } else {
            let l = self.next_loc;
            self.next_loc += 1;
            l
        }
    }

    fn write_unit(&mut self, key: Key, writer: Tree, payload: Payload) -> u64 {
        // 容器（码 3）在真实形态里只由 checkpoint 的固定点写出，写序的事务号是那次 checkpoint 的属性、不是单元的属性；
        // 模型按事务写容器，`cont_wseq_frozen` 把「同一 checkpoint 内容器写序不变」这个性质手工造出来
        let ctr = if self.cfg.cont_wseq_frozen && matches!(key, Key::Cont { .. }) { self.frozen_cont_ctr } else { self.ctr };
        let wseq = WSeq { inst: self.inst, ctr };
        // 码 3 的头里只有出生树，没有写者树：模型不许给重建多于头里有的信息
        let writer = match key { Key::Cont { birth_tree, .. } => birth_tree, Key::Data { .. } => writer };
        // w 份副本逐字节相同（同头、同写序、同载荷），各占一个落点；主落点 = 最小的那个
        let mut locs: Vec<u64> = (0..self.cfg.w.max(1)).map(|_| self.alloc()).collect();
        locs.sort();
        for l in &locs {
            self.disk.insert(*l, Unit { key, writer, birth: self.txg, wseq, payload: payload.clone() });
            self.alloc_rec.insert(*l, AllocState::Allocated);
            self.n_written_units += 1;
        }
        let primary = locs[0];
        self.replicas.insert(primary, locs);
        primary
    }

    /// 一个主落点的全部副本落点（没有登记的就是它自己）。
    fn replica_locs(&self, primary: u64) -> Vec<u64> {
        self.replicas.get(&primary).cloned().unwrap_or_else(|| vec![primary])
    }

    /// 某个落点此刻被哪个已发布根引用（真值口径，运行时靠分配记录 + deadlist 答）。
    fn referenced_by_root(&self, loc: u64) -> bool {
        self.head_views.values().chain(self.snaps.iter()).any(|v| v.alloc.contains(&loc))
    }

    /// 释放一个落点：发布之后按引用情况进 free_pool 或 deadlist。
    fn free_later(&mut self, loc: u64) {
        for r in self.replica_locs(loc) { self.pending_free.push(r); }
    }

    // ── 事务 ──

    /// 施加一个事务。`abandon` = 单元写到盘上、记录没落盘（崩溃时被抛弃的形态）。
    fn txn(&mut self, tree: Tree, ops: &[Op], abandon: bool) {
        self.ctr += 1;
        let mut head = self.heads.remove(&tree).expect("头不存在");
        let saved = head.clone();
        let mut to_free: Vec<u64> = Vec::new();
        let mut tomb_new: Vec<TombRec> = Vec::new();
        let mut written_this_txn: Vec<u64> = Vec::new();
        let wseq = WSeq { inst: self.inst, ctr: self.ctr };

        for op in ops {
            match *op {
                Op::CreateObj => {
                    let key = Key::Data { obj: self.next_obj, obj_birth: self.txg };
                    self.next_obj += 1;
                    let loc = self.write_unit(key, tree, Payload::Data);
                    written_this_txn.push(loc);
                    head.data.insert(key, loc);
                }
                Op::Overwrite(key) => {
                    if let Some(old) = head.data.get(&key).copied() {
                        let loc = self.write_unit(key, tree, Payload::Data);
                        written_this_txn.push(loc);
                        head.data.insert(key, loc);
                        to_free.push(old);
                    }
                }
                Op::DeleteObj(key) => {
                    if let Some(old) = head.data.remove(&key) {
                        to_free.push(old);
                        tomb_new.push(TombRec::Kill { key, death: wseq, death_txg: self.txg });
                    }
                }
                Op::Recreate(key) => {
                    if !head.data.contains_key(&key) {
                        let loc = self.write_unit(key, tree, Payload::Data);
                        written_this_txn.push(loc);
                        head.data.insert(key, loc);
                    }
                }
                Op::WriteThenDelete(key) => {
                    if let Some(old) = head.data.remove(&key) {
                        let loc = self.write_unit(key, tree, Payload::Data);
                        // W1 第二句：这个单元在记录写出前作废（原地重写，它不在任何已发布状态里）
                        if self.cfg.invalidate_in_txn {
                            self.disk.remove(&loc);
                            self.n_invalidated += 1;
                        } else {
                            written_this_txn.push(loc);
                        }
                        to_free.push(old);
                        tomb_new.push(TombRec::Kill { key, death: wseq, death_txg: self.txg });
                    }
                }
                Op::CreateInode => {
                    let ino = self.next_inode;
                    self.next_inode += 1;
                    self.inode_insert(&mut head, ino);
                }
                Op::DeleteInode(ino) => {
                    self.inode_delete(&mut head, ino, &mut tomb_new, wseq, &mut to_free);
                }
            }
        }

        // 脏容器重写（同一头内身份不变；跨头首次 COW 按 cfg 重生）
        for c in head.conts.iter_mut().filter(|c| c.dirty) {
            let mut key = c.key;
            if c.shared && self.cfg.rebirth {
                // 重生保留容器号、只换出生树与出生代：容器号是 inode 号、树内不复用，保得住「记录号 ≥ 容器号」与树内唯一。
                // 第六版模型发现：取「最小 inode 号」当容器号时，同一 txg 里「合并退役 (t, 2, 5, T)」之后左邻重生又得到
                // (t, 2, 5, T)——退役记录把新身份一起杀掉（mixed_w2 seed=5 神谕臂也丢 1）。
                let no = match c.key { Key::Cont { no, .. } => no, _ => unreachable!() };
                key = Key::Cont { birth_tree: tree, kind: 2, no, birth: self.txg };
                assert_ne!(key, c.key, "重生产出了同一个身份：tree={tree} txg={} inst={} ctr={} loc={}", self.txg, self.inst, self.ctr, c.loc);
                // 第二轮模型发现：新身份只覆盖冲突的 inode，覆盖不了「旧身份里有、新身份里没有」的那些
                // ⇒ 重生同时要在本头的墓碑容器里给旧身份写一条退役记录（只对本头的谱系可见）
                if self.cfg.retire_record {
                    tomb_new.push(TombRec::Retire { cont: c.key, at_txg: self.txg });
                }
            }
            let loc = self.write_unit(key, tree, Payload::Inodes(c.recs.clone()));
            written_this_txn.push(loc);
            if c.loc != 0 { to_free.push(c.loc); }
            c.key = key;
            c.loc = loc;
            c.dirty = false;
            c.shared = false;
        }

        // 墓碑记录追加进开放容器，容器重写一版
        if !tomb_new.is_empty() {
            let (key, old_loc, mut recs) = match head.tomb_open.take() {
                Some(x) => x,
                None => {
                    let key = Key::Cont { birth_tree: tree, kind: 1, no: head.next_tomb_no, birth: self.txg };
                    head.next_tomb_no += 1;
                    (key, 0, Vec::new())
                }
            };
            recs.extend(tomb_new);
            let loc = self.write_unit(key, tree, Payload::Tomb(recs.clone()));
            written_this_txn.push(loc);
            if old_loc != 0 { to_free.push(old_loc); }
            head.tomb_open = Some((key, loc, recs));
        }

        if abandon {
            // 记录没落盘：头的状态回滚，单元留在盘上当垃圾；释放不发生；分配记录里也没有它们（从未发布）
            for loc in &written_this_txn { for r in self.replica_locs(*loc) { self.alloc_rec.remove(&r); } }
            self.heads.insert(tree, saved);
            // 生成器独立数：这些单元的 key 若崩溃后没再被写，就是「没有水位就会被判成现行」的那批
            for loc in written_this_txn {
                if let Some(u) = self.disk.get(&loc) {
                    if matches!(u.key, Key::Data { .. }) { self.n_abandoned_unrewritten += 1; }
                }
            }
        } else {
            self.last_applied_ctr = self.ctr;
            self.applied_in_txg = true;
            for l in to_free { self.free_later(l); }
            self.heads.insert(tree, head);
        }
    }

    fn inode_insert(&mut self, head: &mut Head, ino: u64) {
        let tree = head.tree;
        if head.conts.is_empty() || head.conts.last().unwrap().recs.len() >= CONT_CAP {
            // 末尾分裂：右半 = 新记录起，容器号 = 新记录的 inode 号
            let key = Key::Cont { birth_tree: tree, kind: 2, no: ino, birth: self.txg };
            head.conts.push(Cont { key, loc: 0, recs: BTreeMap::new(), dirty: true, shared: false });
        }
        let c = head.conts.last_mut().unwrap();
        c.recs.insert(ino, self.ctr);
        c.dirty = true;
    }

    fn inode_delete(&mut self, head: &mut Head, ino: u64, tomb: &mut Vec<TombRec>, _wseq: WSeq, to_free: &mut Vec<u64>) {
        let Some(idx) = head.conts.iter().position(|c| c.recs.contains_key(&ino)) else { return };
        head.conts[idx].recs.remove(&ino);
        head.conts[idx].dirty = true;
        // 合并纪律：右半记录 ≤ 1 且左邻装得下 ⇒ 左吸收右，右退役
        if idx > 0 && head.conts[idx].recs.len() <= 1
            && head.conts[idx - 1].recs.len() + head.conts[idx].recs.len() <= CONT_CAP
        {
            let right = head.conts.remove(idx);
            let left = &mut head.conts[idx - 1];
            left.recs.extend(right.recs);
            left.dirty = true;
            if right.loc != 0 { to_free.push(right.loc); }
            if self.cfg.retire_record {
                tomb.push(TombRec::Retire { cont: right.key, at_txg: self.txg });
            }
        } else if head.conts[idx].recs.is_empty() {
            let gone = head.conts.remove(idx);
            if gone.loc != 0 { to_free.push(gone.loc); }
            if self.cfg.retire_record {
                tomb.push(TombRec::Retire { cont: gone.key, at_txg: self.txg });
            }
        }
    }

    // ── 发布 / 快照 / 克隆 / 崩溃 / 回收 / scrub ──

    fn view_of(&self, head: &Head) -> View {
        let mut v = View { tree: head.tree, txg: self.txg, ..Default::default() };
        for (k, l) in &head.data {
            v.data.insert(*k, *l);
            for r in self.replica_locs(*l) { v.alloc.insert(r); }
        }
        for c in &head.conts {
            for r in self.replica_locs(c.loc) { v.alloc.insert(r); }
            for (ino, ver) in &c.recs {
                v.inodes.insert(*ino, (c.key, c.loc, *ver));
            }
        }
        if let Some((_, l, _)) = &head.tomb_open { for r in self.replica_locs(*l) { v.alloc.insert(r); } }
        for (_, l, _) in &head.tomb_closed { for r in self.replica_locs(*l) { v.alloc.insert(r); } }
        v.conts = head.conts.iter().map(|c| (c.key, c.loc)).collect();
        v.tomb_open = head.tomb_open.as_ref().map(|(k, l, _)| (*k, *l));
        v.tomb_closed = head.tomb_closed.iter().map(|(k, l, _)| (*k, *l)).collect();
        v.next_tomb_no = head.next_tomb_no;
        v
    }

    /// 从一个已发布视图重建头（回退用）：记录内容从盘上那一版读回。
    fn head_from_view(&self, v: &View) -> Head {
        let recs_of = |l: u64| -> BTreeMap<u64, u64> { match &self.disk[&l].payload { Payload::Inodes(m) => m.clone(), _ => unreachable!() } };
        let tombs_of = |l: u64| -> Vec<TombRec> { match &self.disk[&l].payload { Payload::Tomb(r) => r.clone(), _ => unreachable!() } };
        Head {
            tree: v.tree,
            data: v.data.clone(),
            conts: v.conts.iter().map(|(k, l)| Cont { key: *k, loc: *l, recs: recs_of(*l), dirty: false, shared: matches!(k, Key::Cont { birth_tree, .. } if *birth_tree != v.tree) }).collect(),
            tomb_open: v.tomb_open.map(|(k, l)| (k, l, tombs_of(l))),
            tomb_closed: v.tomb_closed.iter().map(|(k, l)| (*k, *l, tombs_of(*l))).collect(),
            next_tomb_no: v.next_tomb_no,
        }
    }

    const DEFER: Txg = 2;

    fn publish(&mut self) {
        let views: Vec<View> = self.heads.values().map(|h| self.view_of(h)).collect();
        for v in views { self.head_views.insert(v.tree, v); }
        self.root_inst = self.inst;
        self.history.push((self.txg, self.inst, self.head_views.clone(), self.snaps.clone(), self.ancestry.clone()));
        if self.history.len() > 4 { self.history.remove(0); }
        // 本 txg 的释放：发布之后再等 DEFER 次发布才进可分配集合（D16 新规则 2 + I-7.4）
        let pending = std::mem::take(&mut self.pending_free);
        for l in &pending { if !self.referenced_by_root(*l) { self.alloc_rec.insert(*l, AllocState::Freed(self.txg)); } }
        self.defer.push_back((self.txg, pending));
        while let Some((t, _)) = self.defer.front() {
            if *t + Self::DEFER > self.txg { break; }
            let (_, locs) = self.defer.pop_front().unwrap();
            for l in locs {
                if self.referenced_by_root(l) { self.deadlisted.insert(l); }
                else if !self.free_pool.contains(&l) && self.disk.contains_key(&l) { self.free_pool.push_back(l); }
            }
        }
        self.txg += 1;
        self.applied_in_txg = false;
    }

    /// 管理员回退到上一次发布的根：一次恢复——实例代号 +1，被抛弃的那段时间线按实例表行判未发布。
    /// 行：(r_old, T_old, 0)——旧实例发布到 T_old 为止的全部有效、T_old 之后一个都不算；(i, 0, 0)——夹在中间的实例一个都不算。
    fn rollback_to_previous(&mut self) { self.rollback(1); }

    /// 回退 `depth` 次发布（1 = 上一个根）。
    fn rollback(&mut self, depth: usize) {
        assert!(self.history.len() > depth, "回退深度超过保留的根数");
        let (t_old, r_old, views, snaps, anc) = self.history[self.history.len() - 1 - depth].clone();
        let new_inst = self.inst + 1;
        let mut rows: Vec<(u32, (Txg, u64))> = vec![(r_old, (t_old, 0))];
        for i in (r_old + 1)..new_inst { rows.push((i, (0, 0))); }
        self.inst_last_txg.insert(r_old, (t_old, false));
        for i in (r_old + 1)..new_inst { self.inst_last_txg.insert(i, (0, false)); }
        if self.cfg.write_watermarks {
            for (i, r) in rows { self.watermarks.insert(i, r); self.rollback_rows.insert(i); }
        }
        // 被抛弃时间线里写下的单元数（生成器闭式）：诞生代号 > T_old 的数据单元
        self.n_rollback_abandoned = self.disk.values().filter(|u| u.birth > t_old && matches!(u.key, Key::Data { .. })).count() as u64;
        self.heads = views.values().map(|v| (v.tree, self.head_from_view(v))).collect();
        self.head_views = views;
        self.snaps = snaps;
        self.ancestry = anc;
        // 分配记录从 R_old 重载：被抛弃时间线里的分配没有条目，它的释放也撤销
        let live: BTreeSet<u64> = self.head_views.values().chain(self.snaps.iter()).flat_map(|v| v.alloc.iter().copied()).collect();
        for (l, st) in self.alloc_rec.iter_mut() {
            if live.contains(l) { *st = AllocState::Allocated; }
            else if let AllocState::Freed(t) = *st { if t > t_old { *st = AllocState::Freed(t_old); } }
        }
        let abandoned_allocs: Vec<u64> = self.alloc_rec.iter().filter(|(l, st)| **st == AllocState::Allocated && !live.contains(l)).map(|(l, _)| *l).collect();
        for l in abandoned_allocs { self.alloc_rec.remove(&l); }
        for (t, i, _, _, _) in &self.history[self.history.len() - depth..] { self.abandoned_roots.push((*t, *i)); }
        // 回退后第一个新根的 txg 必须压过根环里全部根（第三轮反推腿 5.2：否则下次择新挑回被抛弃的线）
        let max_ring_txg = self.history.iter().map(|h| h.0).max().unwrap();
        self.history.truncate(self.history.len() - depth);
        self.pending_free.clear();
        if self.cfg.retain_abandoned_defer { self.defer.retain(|(t, _)| *t <= t_old); }
        self.inst = new_inst;
        self.ctr = 0;
        self.last_applied_ctr = 0;
        self.txg = if self.cfg.rollback_skips_ring { max_ring_txg + 1 } else { t_old + 1 };
        self.publish();
    }

    fn snapshot(&mut self, tree: Tree) {
        // 快照钉在本 checkpoint 全部变更之后：先发布，再记视图
        // 快照发布之前关闭开放的墓碑容器（D18 已定项 10 按代际装载）
        let h = self.heads.get_mut(&tree).unwrap();
        if let Some(x) = h.tomb_open.take() { h.tomb_closed.push(x); }
        self.publish();
        let v = self.head_views[&tree].clone();
        let v = View { txg: self.txg - 1, ..v };
        self.snaps.push(v);
    }

    fn clone_head(&mut self, snap_idx: usize) -> Tree {
        let s = self.snaps[snap_idx].clone();
        let origin = self.heads[&s.tree].clone();
        let t = self.new_tree();
        // 新头共享 origin 在快照那一刻的单元：数据 map 与容器都照抄，容器标 shared
        let mut conts: Vec<Cont> = Vec::new();
        let mut seen: BTreeSet<Key> = BTreeSet::new();
        for (_, (ck, cl, _)) in &s.inodes {
            if seen.insert(*ck) {
                let recs = match &self.disk[cl].payload { Payload::Inodes(m) => m.clone(), _ => unreachable!() };
                conts.push(Cont { key: *ck, loc: *cl, recs, dirty: false, shared: true });
            }
        }
        conts.sort_by_key(|c| match c.key { Key::Cont { no, .. } => no, _ => 0 });
        self.heads.insert(t, Head {
            tree: t,
            data: s.data.clone(),
            conts,
            tomb_open: None,
            tomb_closed: Vec::new(),
            next_tomb_no: origin.next_tomb_no + 1_000,
        });
        self.ancestry.insert(t, (s.tree, s.txg));
        // 建克隆那次发布要把新头的视图写出来
        self.publish();
        t
    }

    /// 崩溃：本 txg 已施加的事务在恢复时被重放并发布，实例代号 +1，txg 重发。
    /// `double` = 恢复实例在写出第一个根之前又崩一次。
    fn crash_and_recover(&mut self, double: bool) {
        let crashed_inst = self.inst;
        let w = self.last_applied_ctr;
        // 崩溃实例最后发布的 txg = 当前在飞 txg − 1（在飞的这个没有它的根）
        self.inst_last_txg.insert(crashed_inst, (self.txg - 1, self.applied_in_txg));
        self.inst += 1;
        self.ctr = 0;
        // 第七版落地逐字：给 [所选根的实例, 新实例) 每个实例写行。恢复恒选最新已发布根 ⇒ 所选根的实例 = 崩溃实例，
        // 范围里只有它自己（`double` 那一格多一个恢复实例，见下）
        let mut rows: Vec<(u32, (Txg, u64))> = vec![(crashed_inst, (self.txg - 1, w))];
        // 崩溃实例在飞 checkpoint 的固定点单元（容器）留在盘上、一律未发布；恢复实例重放之后按自己的写序重写它们
        self.rewrite_inflight_conts(crashed_inst);
        if double {
            // 恢复实例写了一些单元（重放后的第一个根的节点）就崩了：它没有施加任何自己的记录
            let key = Key::Data { obj: 0, obj_birth: self.txg };
            self.ctr += 1;
            let _ = self.write_unit(key, 1, Payload::Data);
            self.n_abandoned_unrewritten += 1;
            // 第八版：W 按实例分——恢复实例自己施加过的事务，下一次重放跨不过实例边界（jsn 断号）⇒ 它的 W 恒 0。
            // 关掉开关 = 第七版落地那句的字面：把同一个全局量写给范围里的每一个实例
            let own_w = if self.cfg.row_w_per_instance { 0 } else { w };
            rows.push((self.inst, (self.txg - 1, own_w)));
            self.inst_last_txg.insert(self.inst, (self.txg - 1, false));
            let crashed_again = self.inst;
            self.inst += 1;
            self.ctr = 0;
            self.rewrite_inflight_conts(crashed_again);
        }
        if self.cfg.write_watermarks {
            let sticky = self.rollback_rows.clone();
            for (i, w) in rows { put_row(&mut self.watermarks, &sticky, true, i, w, &mut self.rollback_row_writes_blocked); }
        }
        self.last_applied_ctr = 0;
        // 恢复发布：重放施加的事务 + 水位行 = 第一个新根（txg 号不变，就是被重发的那个）
        self.publish();
    }

    /// 恢复实例把上一个实例在在飞 checkpoint 里写出的容器版本重写一遍（新写序、同诞生代号）：
    /// 那些旧版本的诞生代号等于没发布的那个 checkpoint，按 `row_published` 一律未发布，留在盘上当孤儿。
    fn rewrite_inflight_conts(&mut self, prev_inst: u32) {
        let txg = self.txg;
        let trees: Vec<Tree> = self.heads.keys().copied().collect();
        for tree in trees {
            let mut head = self.heads.remove(&tree).unwrap();
            let stale = |disk: &BTreeMap<u64, Unit>, loc: u64| loc != 0 && disk.get(&loc).map_or(false, |u| u.birth == txg && u.wseq.inst == prev_inst);
            for c in head.conts.iter_mut() {
                if stale(&self.disk, c.loc) {
                    let old = c.loc;
                    c.loc = self.write_unit(c.key, tree, Payload::Inodes(c.recs.clone()));
                    for r in self.replica_locs(old) { self.alloc_rec.remove(&r); }
                }
            }
            if let Some((k, l, recs)) = head.tomb_open.take() {
                let l2 = if stale(&self.disk, l) {
                    let n = self.write_unit(k, tree, Payload::Tomb(recs.clone()));
                    for r in self.replica_locs(l) { self.alloc_rec.remove(&r); }
                    n
                } else { l };
                head.tomb_open = Some((k, l2, recs));
            }
            for (k, l, recs) in head.tomb_closed.iter_mut() {
                if stale(&self.disk, *l) {
                    let old = *l;
                    *l = self.write_unit(*k, tree, Payload::Tomb(recs.clone()));
                    for r in self.replica_locs(old) { self.alloc_rec.remove(&r); }
                }
            }
            self.heads.insert(tree, head);
        }
    }

    /// 第六版 P2 的行回收：kind 0 行可删 ⟺ 盘上不再有该实例按行判未发布的可读单元，
    /// 且（开关）根环里没有该实例发布的根——被抛弃的根留在环里时，它的行是回退候选判据的输入，不是垃圾的影子。
    fn reclaim_rows(&mut self) {
        let ring_insts: BTreeSet<u32> = self.history.iter().map(|h| h.1).chain(self.abandoned_roots.iter().map(|r| r.1)).collect();
        let insts: Vec<u32> = self.watermarks.keys().copied().collect();
        for i in insts {
            if self.cfg.row_reclaim_checks_ring && ring_insts.contains(&i) { continue; }
            let has_unpublished = self.disk.values().any(|u| u.wseq.inst == i && !row_published(&self.watermarks, u));
            if !has_unpublished { self.watermarks.remove(&i); }
        }
    }

    /// 第六版 P3 的回退候选集：根环里（有效的 + 被抛弃的）按实例表判仍有效的根。真值 = history 里的根。
    fn rollback_candidates(&self) -> Vec<(Txg, u32)> {
        self.history.iter().map(|h| (h.0, h.1)).chain(self.abandoned_roots.iter().copied())
            .filter(|(t, i)| match self.watermarks.get(i) { None => true, Some((tp, _)) => *t <= *tp })
            .collect()
    }

    /// 恢复选了非最新根（第六轮反推腿 2.1）：最新已发布根的槽自证不过，它之后的记录又已回收、不可重放
    /// ⇒ 所选根 = 上一个根，什么都重放不了，行 = (崩溃实例, 所选根的 txg, 0)。丢掉的那个 checkpoint 与在飞 checkpoint
    /// 里写下的单元全是孤儿。开关关掉时行按第六版字面写 (在飞 txg − 1, 0)，把丢掉的 checkpoint 判成已发布。
    fn crash_and_recover_lost_latest_root(&mut self) { self.crash_and_recover_lost_roots(1); }

    /// 最新的 `depth` 个根都不可读，恢复退到再往前那一个。
    fn crash_and_recover_lost_roots(&mut self, depth: usize) {
        assert!(self.history.len() > depth, "要有更早的根可选");
        let crashed_inst = self.inst;
        let mut lost = None;
        for _ in 0..depth { lost = self.history.pop(); }
        let chosen_txg = self.history.last().unwrap().0;
        let (_, _, views, snaps, anc) = self.history.last().unwrap().clone();
        // 丢掉的 checkpoint 与在飞 checkpoint 里的数据单元：生成器独立计数（闭式）
        self.n_abandoned_unrewritten = self.disk.values().filter(|u| u.birth > chosen_txg && matches!(u.key, Key::Data { .. })).count() as u64;
        self.inst_last_txg.insert(crashed_inst, (chosen_txg, false));
        self.inst += 1;
        self.ctr = 0;
        let tpub = if self.cfg.row_tpub_from_chosen_root { chosen_txg } else { self.txg - 1 };
        // 第七版落地逐字：给 [所选根的实例, 新实例) 每个实例写行。所选根可能是更早的实例发布的
        // ⇒ 这一段会覆盖到回退行（第七轮反推腿 9.1），第八版靠 `rollback_row_sticky` 挡住
        let chosen_inst = self.history.last().unwrap().1;
        if self.cfg.write_watermarks {
            let sticky = self.rollback_rows.clone();
            for i in chosen_inst..self.inst {
                put_row(&mut self.watermarks, &sticky, true, i, (tpub, 0), &mut self.rollback_row_writes_blocked);
            }
        }
        // 状态退回所选根：与回退同一套重载（头、快照、祖先表、分配记录、defer）
        self.heads = views.values().map(|v| (v.tree, self.head_from_view(v))).collect();
        self.head_views = views;
        self.snaps = snaps;
        self.ancestry = anc;
        let live: BTreeSet<u64> = self.head_views.values().chain(self.snaps.iter()).flat_map(|v| v.alloc.iter().copied()).collect();
        for (l, st) in self.alloc_rec.iter_mut() {
            if live.contains(l) { *st = AllocState::Allocated; }
            else if let AllocState::Freed(t) = *st { if t > chosen_txg { *st = AllocState::Freed(chosen_txg); } }
        }
        let abandoned_allocs: Vec<u64> = self.alloc_rec.iter().filter(|(l, st)| **st == AllocState::Allocated && !live.contains(l)).map(|(l, _)| *l).collect();
        for l in abandoned_allocs { self.alloc_rec.remove(&l); }
        self.pending_free.clear();
        self.defer.retain(|(t, _)| *t <= chosen_txg);
        let _ = lost; // 槽坏了的根不在环里、也不是回退候选
        self.last_applied_ctr = 0;
        // 恢复发布：txg 号照在飞的那个重发
        self.publish();
    }

    /// 第七轮反推腿 3.1：恢复实例重放完上一实例的事务并发布之后，又施加了自己的几个事务，然后在下一个根之前再崩。
    /// 下一次恢复的所选根仍是恢复实例发布的那个之前的那一个吗——不是：恢复实例发布过根，所选根就是它发的那个；
    /// 真正承重的是**它自己的记录跨不过实例边界**（jsn 断号）⇒ 它在下一个 checkpoint 里施加的事务一条也重放不了。
    /// 行的 W 按实例分 ⇒ 它那行 W = 0；按第七版落地那句的字面写全局量 ⇒ 它的孤儿 n ≤ W 判已发布。
    fn crash_with_own_applied(&mut self, own: usize, tree: Tree) {
        let rec_inst = self.inst;
        // 上一次恢复给所选根那个实例算出来的 W（全局量就是它）
        let chosen_w = (1..rec_inst).filter_map(|i| self.watermarks.get(&i).map(|r| r.1)).max().unwrap_or(0);
        for _ in 0..own { self.txn(tree, &[Op::CreateObj], true); }
        self.inst_last_txg.insert(rec_inst, (self.txg - 1, false));
        self.inst += 1;
        self.ctr = 0;
        let w_row = if self.cfg.row_w_per_instance { 0 } else { chosen_w };
        if self.cfg.write_watermarks {
            let sticky = self.rollback_rows.clone();
            put_row(&mut self.watermarks, &sticky, true, rec_inst, (self.txg - 1, w_row), &mut self.rollback_row_writes_blocked);
        }
        self.rewrite_inflight_conts(rec_inst);
        self.last_applied_ctr = 0;
        self.publish();
    }

    fn destroy_snapshot(&mut self, idx: usize) {
        let s = self.snaps.remove(idx);
        self.last_root_removal_txg = self.txg;
        for l in s.alloc {
            if self.deadlisted.remove(&l) && !self.referenced_by_root(l) {
                self.pending_free.push(l);
            }
        }
        // 克隆祖先表里指向它的克隆点保留（分叉点快照不许销毁，生成器不会销毁它）
    }

    /// 墓碑回收：D18 已定项 10 的条件 ∧（cfg.scrub_gate ⇒ 抹头水位 ≥ 死亡代号）。
    fn reclaim_tombstones(&mut self) {
        let snaps = self.snaps.clone();
        let heads = self.head_views.clone();
        let scrub_wm = self.scrub_wm;
        let gate = self.cfg.scrub_gate;
        let floor = if self.cfg.gate_v2 { self.last_root_removal_txg } else { 0 };
        // 某个根还引用着这个容器身份的某一版吗（D18 已定项 10 的条件用在容器上）
        let cont_referenced = |w: &World, cont: &Key| -> bool {
            w.disk.iter().any(|(l, u)| u.key == *cont && w.referenced_by_root(*l))
        };
        let trees: Vec<Tree> = self.heads.keys().copied().collect();
        for tree in trees {
            let mut head = self.heads.remove(&tree).unwrap();
            let mut rewrite: Vec<(Key, u64, Vec<TombRec>)> = Vec::new();
            let mut kept: Vec<(Key, u64, Vec<TombRec>)> = Vec::new();
            for (key, loc, recs) in head.tomb_closed.drain(..) {
                let keep: Vec<TombRec> = recs.iter().copied().filter(|r| match r {
                    TombRec::Kill { key, death_txg, .. } => {
                        // D18 已定项 10 的量词 2026-09-05 从「全部还引用它的快照」放宽到「全部还引用它的根（含快照与可写头）」：
                        // 克隆头是头不是快照，凡是它里面还活着、而 origin 后来删掉的对象，按旧量词它的 kill 记录可以被回收，
                        // 而克隆引用的那具尸体一直可读（第七轮反推腿 7.2、第八轮反推腿 14.1）
                        // D18 已定项 10 的量词 2026-09-05 从「快照」放宽到「根（含快照与可写头）」。
                        // ⚠️ 模型这道门是**按 key** 判的，而规则问的是「还有没有根引用那个死掉的版本」——
                        // 两者在「先删后重建」上就分岔（同一个 key 既有 kill 记录、又被本头引用着新版本）。
                        // 所以模型分不出两个量词：加上可写头这一半只是更保守，没有能让它承重的世界（第八轮反推腿 14.1）。
                        let snap_refs = snaps.iter().any(|s| s.data.contains_key(key)) || heads.values().any(|v: &View| v.data.contains_key(key));
                        let gated = gate && scrub_wm < (*death_txg).max(floor);
                        snap_refs || gated
                    }
                    TombRec::Retire { cont, at_txg, .. } => {
                        let refs = cont_referenced(self, cont);
                        let gated = gate && scrub_wm < (*at_txg).max(floor);
                        refs || gated
                    }
                }).collect();
                if keep.len() == recs.len() { kept.push((key, loc, recs)); } else { rewrite.push((key, loc, keep)); }
            }
            head.tomb_closed = kept;
            for (key, old_loc, keep) in rewrite {
                self.free_later(old_loc);
                if !keep.is_empty() {
                    self.ctr += 1;
                    let loc = self.write_unit(key, tree, Payload::Tomb(keep.clone()));
                    self.last_applied_ctr = self.ctr;
                    head.tomb_closed.push((key, loc, keep));
                }
            }
            self.heads.insert(tree, head);
        }
    }

    /// scrub：抹掉可读、已发布、不属于任何根现行版本的单元的头；抹头水位 = 上一个已发布 txg。
    /// 神谕清扫（真值判据）：只给单测当对照，世界里不用。
    #[cfg_attr(not(test), allow(dead_code))]
    fn scrub_oracle_set(&self) -> BTreeSet<u64> {
        let deferred: BTreeSet<u64> = self.defer.iter().flat_map(|(_, v)| v.iter().copied()).chain(self.pending_free.iter().copied()).collect();
        self.disk.keys().copied().filter(|l| !self.referenced_by_root(*l) && !deferred.contains(l)).collect()
    }

    /// 第四版 P4 的清扫准入（全部来自运行时结构，不用 P3 的择版本）：
    /// 分配记录条目标已释放 ∧ 释放代 ≤ 环里最旧根的 txg ∧ 头可读；
    /// 孤儿（没有条目、不在在飞 overlay 里）按实例表判：所属实例 < 当前实例 ∧ 未发布。
    fn sweep_candidates(&self) -> Vec<u64> {
        let oldest_ring_txg = self.history.first().map(|h| h.0).unwrap_or(0);
        let inflight: BTreeSet<u64> = self.pending_free.iter().copied().collect();
        let mut out = Vec::new();
        for (l, u) in &self.disk {
            if inflight.contains(l) { continue; }
            match self.alloc_rec.get(l) {
                Some(AllocState::Allocated) => {}
                Some(AllocState::Freed(t)) => { if *t <= oldest_ring_txg { out.push(*l); } }
                None => {
                    if u.wseq.inst < self.inst {
                        if !row_published(&self.watermarks, u) { out.push(*l); }
                    }
                }
            }
        }
        out
    }

    fn scrub(&mut self) {
        for l in self.sweep_candidates() {
            self.disk.remove(&l);
            self.n_invalidated += 1;
        }
        // 清扫水位 = 截至哪个 txg 的释放已确认不可读：规则只清得到释放代 ≤ 最旧根 txg 的落点，
        // 水位就只能推进到那里（取 txg − 1 会让 P5 的门放走还被根环保护着的旧版本）
        self.scrub_wm = self.history.first().map(|h| h.0).unwrap_or(0);
    }

    fn snapshot_count(&self) -> usize { self.snaps.len() }
}

// ── 重建 ──

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Arm {
    name: &'static str,
    /// 用真值 alloc 集当 `live` 神谕（E59 乙臂形态，阳性对照）
    oracle: bool,
    use_wseq: bool,
    use_watermark: bool,
    use_ancestry: bool,
    use_death_wseq: bool,
    use_retire: bool,
    /// 消融：写序相等时让单元赢（提案的规则是墓碑赢）
    tie_unit_wins: bool,
    /// 消融：头里只有 4 字节实例代号、没有计数器——水位只能说「实例 i 发布到了 txg T」
    inst_only: bool,
    /// 择版本之前先把同一逻辑版本的副本归并成一个候选（第三版 P3 的「副本归并」）
    merge_replicas: bool,
    /// 第八版 P3：码 3 的择新键取 (诞生代号, 实例代号)；关掉 = 第七版的「取写序最大者」
    cont_key_by_birth: bool,
    /// 对立臂 B：写序只上码 3 容器，数据单元头里没有写序——数据单元只按诞生代号定序、只按诞生代号 ≤ 根 txg 判已发布
    meta_only: bool,
}

const FULL: Arm = Arm { name: "full", oracle: false, use_wseq: true, use_watermark: true, use_ancestry: true, use_death_wseq: true, use_retire: true, tie_unit_wins: false, inst_only: false, merge_replicas: true, cont_key_by_birth: true, meta_only: false };
/// 消融：码 3 也按写序择新（第七版的规则）——同实例连着两个 checkpoint 不分配新事务号时两版写序相同 ⇒ 平局
const CONT_KEY_WSEQ: Arm = Arm { name: "cont_key_wseq", cont_key_by_birth: false, ..FULL };
const WSEQ_META_ONLY: Arm = Arm { name: "wseq_meta_only", meta_only: true, ..FULL };
const NO_REPLICA_MERGE: Arm = Arm { name: "no_replica_merge", merge_replicas: false, ..FULL };
const INST_ONLY: Arm = Arm { name: "inst_only", inst_only: true, ..FULL };
const TIE_UNIT_WINS: Arm = Arm { name: "tie_unit_wins", tie_unit_wins: true, ..FULL };
const ORACLE: Arm = Arm { name: "oracle", oracle: true, ..FULL };
const NO_WSEQ: Arm = Arm { name: "no_wseq", use_wseq: false, ..FULL };
const NO_WATERMARK: Arm = Arm { name: "no_watermark", use_watermark: false, ..FULL };
const NO_ANCESTRY: Arm = Arm { name: "no_ancestry", use_ancestry: false, ..FULL };
const NO_DEATH_WSEQ: Arm = Arm { name: "no_death_wseq", use_death_wseq: false, ..FULL };
const NO_RETIRE: Arm = Arm { name: "no_retire", use_retire: false, ..FULL };
const ARMS: [Arm; 12] = [ORACLE, FULL, NO_WSEQ, NO_WATERMARK, NO_ANCESTRY, NO_DEATH_WSEQ, NO_RETIRE, TIE_UNIT_WINS, INST_ONLY, NO_REPLICA_MERGE, WSEQ_META_ONLY, CONT_KEY_WSEQ];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Out {
    roots: u64,
    scanned: u64,
    /// 真值活、重建活、落点不同
    wrong: u64,
    /// 真值死 / 不存在、重建活
    resurrected: u64,
    /// 真值活、重建死或缺
    lost: u64,
    /// 同一 key 多份候选、这一臂没有判据分出
    ambiguous: u64,
    injected: u64,
}

impl Out {
    fn divergence(&self) -> u64 { self.wrong + self.resurrected + self.lost + self.ambiguous }
}

struct Rebuilt {
    data: BTreeMap<Key, u64>,
    inodes: BTreeMap<u64, u64>,
    ambiguous: u64,
}

fn ancestry_of(anc: &BTreeMap<Tree, (Tree, Txg)>, tree: Tree) -> Vec<(Tree, Txg)> {
    let mut out = vec![(tree, u64::MAX)];
    let mut cur = tree;
    let mut limit = u64::MAX;
    while let Some((o, c)) = anc.get(&cur) {
        limit = limit.min(*c);
        out.push((*o, limit));
        cur = *o;
    }
    out
}

fn rebuild_root(w: &World, root: &View, arm: Arm) -> Rebuilt {
    let cur_inst = w.inst;
    let cur_txg = w.txg - 1;
    let anc = ancestry_of(&w.ancestry, root.tree);
    let mut amb = 0u64;

    let published = |u: &Unit| -> bool {
        if !arm.use_watermark { return true; }
        // 臂 B：数据单元头里没有实例代号也没有事务号，能问的只有「诞生代号 ≤ 挂载根 txg」
        if arm.meta_only && matches!(u.key, Key::Data { .. }) { return u.birth <= cur_txg; }
        if arm.inst_only {
            // 只有实例代号：崩溃实例在它最后发布的 txg 之后写的单元一律判未发布；
            // 与那个 txg 同号的（部分重放那一格）分不出，只能判已发布
            return match w.inst_last_txg.get(&u.wseq.inst) {
                None => true,
                // 在飞 txg 没有任何事务被重放 ⇒ 整个 txg 烧掉；被部分重放 ⇒ 同实例同 txg 分不出，只能判已发布
                Some((t, partial)) => u.birth <= *t || *partial,
            };
        }
        if u.wseq.inst < cur_inst {
            row_published(&w.watermarks, u)
        } else if u.wseq.inst == cur_inst {
            u.birth <= cur_txg
        } else { false }
    };
    let visible = |loc: u64, u: &Unit| -> bool {
        if arm.oracle { return root.alloc.contains(&loc); }
        if !published(u) { return false; }
        if u.birth > root.txg { return false; }
        if arm.use_ancestry {
            anc.iter().any(|(t, lim)| *t == u.writer && u.birth <= *lim)
        } else { true }
    };

    // 每个身份的候选。副本归并：同 (key, 写者树, 诞生代号, 写序) 的多份可读单元是同一个逻辑版本的副本
    // （D2 已定项 6 的 w ≥ 2：同一份内容写 N 遍），先归并再择版本——bcachefs 的 cookie 合并是同一步。
    // 消融 no_replica_merge：不归并，每份副本各算一个候选 ⇒ 写序相等的平局全部记成歧义。
    let mut cands: BTreeMap<Key, Vec<(u64, &Unit)>> = BTreeMap::new();
    let mut seen_versions: BTreeSet<(Key, Tree, Txg, WSeq)> = BTreeSet::new();
    for (loc, u) in &w.disk {
        if !visible(*loc, u) { continue; }
        if arm.merge_replicas && !seen_versions.insert((u.key, u.writer, u.birth, u.wseq)) {
            continue; // 同一逻辑版本的另一份副本：归并进已登记的那一份
        }
        cands.entry(u.key).or_default().push((*loc, u));
    }
    // 择新
    let pick = |v: &Vec<(u64, &Unit)>, amb: &mut u64| -> (u64, WSeq, Txg) {
        let data_by_birth = arm.meta_only && v.iter().all(|(_, u)| matches!(u.key, Key::Data { .. }));
        // 第八版 P3：择新键按单元类分——码 1 取写序；码 3 只在 checkpoint 的固定点写出，
        // 写序的事务号是 checkpoint 的属性、给不出全序，键取 (诞生代号, 实例代号)。
        // ⚠️ 模型按事务写容器（真实形态是固定点一次），同一 checkpoint 里同身份能出现多版 ⇒ 键末尾补一个写序当平局破除器，
        // **那一维是模型的补丁不是规则的一部分**：规则的前提「一个实例在一个 checkpoint 只有一个固定点」模型不满足。
        let cont = arm.cont_key_by_birth && arm.use_wseq && v.iter().all(|(_, u)| matches!(u.key, Key::Cont { .. }));
        if cont {
            let mk = v.iter().map(|(_, u)| (u.birth, u.wseq.inst, u.wseq.ctr)).max().unwrap();
            let ties: Vec<&(u64, &Unit)> = v.iter().filter(|(_, u)| (u.birth, u.wseq.inst, u.wseq.ctr) == mk).collect();
            if ties.len() > 1 { *amb += 1; }
            let (l, u) = ties.iter().min_by_key(|(l, _)| *l).unwrap();
            (*l, u.wseq, u.birth)
        } else if arm.use_wseq && !data_by_birth {
            let ms = v.iter().map(|(_, u)| u.wseq).max().unwrap();
            let ties: Vec<&(u64, &Unit)> = v.iter().filter(|(_, u)| u.wseq == ms).collect();
            if ties.len() > 1 { *amb += 1; }
            let (l, u) = ties.iter().min_by_key(|(l, _)| *l).unwrap();
            (*l, u.wseq, u.birth)
        } else {
            let mb = v.iter().map(|(_, u)| u.birth).max().unwrap();
            let ties: Vec<&(u64, &Unit)> = v.iter().filter(|(_, u)| u.birth == mb).collect();
            if ties.len() > 1 { *amb += 1; }
            let (l, u) = ties.iter().max_by_key(|(l, _)| *l).unwrap();
            (*l, u.wseq, u.birth)
        }
    };
    let mut current: BTreeMap<Key, (u64, WSeq, Txg)> = BTreeMap::new();
    for (k, v) in &cands { current.insert(*k, pick(v, &mut amb)); }

    // 墓碑记录：来自可见的、现行版本的类型 1 容器
    let mut kills: BTreeMap<Key, Vec<(WSeq, Txg)>> = BTreeMap::new();
    let mut retired: BTreeSet<Key> = BTreeSet::new();
    for (k, (loc, _, _)) in &current {
        if let Key::Cont { kind: 1, .. } = k {
            if let Payload::Tomb(recs) = &w.disk[loc].payload {
                for r in recs {
                    match r {
                        TombRec::Kill { key, death, death_txg } => kills.entry(*key).or_default().push((*death, *death_txg)),
                        TombRec::Retire { cont, .. } => { if arm.use_retire { retired.insert(*cont); } }
                    }
                }
            }
        }
    }
    let mut data: BTreeMap<Key, u64> = BTreeMap::new();
    let mut inode_cands: BTreeMap<u64, Vec<(WSeq, u64)>> = BTreeMap::new();
    for (k, (loc, wseq, birth)) in &current {
        match k {
            Key::Data { .. } => {
                let dead = kills.get(k).map_or(false, |ks| ks.iter().any(|(d, dt)| {
                    if arm.use_death_wseq && !arm.meta_only { if arm.tie_unit_wins { d > wseq } else { d >= wseq } } else {
                        if dt == birth { amb += 1; false } else { dt > birth }
                    }
                }));
                if !dead { data.insert(*k, *loc); }
            }
            Key::Cont { kind: 2, .. } => {
                if retired.contains(k) { continue; }
                if let Payload::Inodes(m) = &w.disk[loc].payload {
                    for (ino, ver) in m { inode_cands.entry(*ino).or_default().push((*wseq, *ver)); }
                }
            }
            _ => {}
        }
    }
    let mut inodes: BTreeMap<u64, u64> = BTreeMap::new();
    for (ino, v) in inode_cands {
        // 两个现行容器装着同一个 inode 号：规则上不该发生，发生了记成歧义（取写序大的那条）
        if v.len() > 1 { amb += 1; }
        let (_, ver) = v.iter().max_by_key(|(s, _)| *s).unwrap();
        inodes.insert(ino, *ver);
    }
    Rebuilt { data, inodes, ambiguous: amb }
}

fn measure(w: &World, arm: Arm, corrupt: usize) -> Out {
    let roots: Vec<&View> = w.head_views.values().chain(w.snaps.iter()).collect();
    let mut o = Out { roots: roots.len() as u64, scanned: w.disk.len() as u64, ..Default::default() };
    let mut to_inject = corrupt;
    for r in roots {
        let mut re = rebuild_root(w, r, arm);
        o.ambiguous += re.ambiguous;
        // 判别力自证：只往本来对的条目上注入
        let victims: Vec<Key> = re.data.iter().filter(|(k, l)| r.data.get(*k) == Some(*l)).map(|(k, _)| *k).take(to_inject).collect();
        for k in victims { let l = re.data[&k]; re.data.insert(k, l + 500_000); o.injected += 1; to_inject -= 1; }
        for (k, l) in &r.data {
            match re.data.get(k) { None => o.lost += 1, Some(p) if p != l => o.wrong += 1, _ => {} }
        }
        for k in re.data.keys() { if !r.data.contains_key(k) { o.resurrected += 1; } }
        for (ino, (_, _, ver)) in &r.inodes {
            match re.inodes.get(ino) { None => o.lost += 1, Some(v) if v != ver => o.wrong += 1, _ => {} }
        }
        for ino in re.inodes.keys() { if !r.inodes.contains_key(ino) { o.resurrected += 1; } }
    }
    o
}

// ── 世界 ──

#[derive(Clone, Copy, Debug)]
struct Closed {
    /// 崩溃后没再被写过的被抛弃单元数（去水位臂应当把它们全判成现行）
    abandoned_unrewritten: u64,
    /// 墓碑已回收且旧版本仍可读的 key 数（去 scrub 门时应当全数复活）
    reclaimed_readable: u64,
    /// 克隆头与 origin 各写一版的 key 数（去祖先表时 origin 视野应当全错）
    cross_head_keys: u64,
    /// 同一 txg 内跨事务先删后写 / 先写后删的 key 数（只有代号时应当全部歧义）
    same_txg_pairs: u64,
    /// 同一 txg 内跨事务两次覆写的 key 数（只有代号时应当全部歧义）
    same_txg_overwrites: u64,
    /// 退役容器里、后来在左半被删掉的 inode 数（去退役记录时应当复活）
    retired_then_deleted: u64,
    /// 回退抛弃的那段时间线里写下的数据单元数（没有回退行时应当全部复活）
    rollback_abandoned: u64,
    /// w = 2 时各根引用的身份数之和（不归并副本时每个身份在每个根上各一次平局）
    live_versions: u64,
}

/// 靶子世界一：崩溃 + 部分重放 + 连崩两次。
fn world_crash(seed: u64, cfg: Cfg, double: bool) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], false); }
    w.publish();
    // 在飞 txg：两个事务被重放，三个被抛弃（各建两个对象，崩溃后不再写）
    w.txn(t, &[Op::CreateObj], false);
    w.txn(t, &[Op::CreateObj], false);
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], true); }
    w.crash_and_recover(double);
    // 重发的 txg 里再建一个对象，然后正常发布几轮
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { abandoned_unrewritten: w.n_abandoned_unrewritten, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之六（第五轮反推腿 1.1）：在飞 txg 里事务 A 施加，事务 B 发出单元写之后失败，C、D 也写了单元。
/// 第六版：失败 ⇒ 实例结束，C、D 的记录一条都不追加（记录按事务号顺序），恢复行 W = A 的号 ⇒ B、C、D 全部判未发布。
/// 第五版的形态（开关关掉）：C、D 照常追加并被重放，W = D 的号 ⇒ B 的单元 n ≤ W 判已发布 ⇒ 复活。
fn world_txn_fail(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], false); }
    w.publish();
    w.txn(t, &[Op::CreateObj], false); // A
    w.txn(t, &[Op::CreateObj, Op::CreateObj], true); // B：单元在盘上，记录永远不会追加
    let later_abandoned = w.cfg.records_stop_at_failure;
    w.txn(t, &[Op::CreateObj], later_abandoned); // C
    w.txn(t, &[Op::CreateObj], later_abandoned); // D
    w.crash_and_recover(false); // 实例结束 = 下次挂载走恢复
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { abandoned_unrewritten: w.n_abandoned_unrewritten, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之十一（第八轮反推腿 12.1）：回退之后新时间线的根不可读，恢复落回 R_old。
/// 那次恢复要给 [所选根的实例, 新实例) 每个实例写行，范围里包含 r_old ⇒ 正好盖到回退行；第八版靠标志挡住。
/// ⚠️ 模型量得出的只有「保护确实挡下过写」这一件事：模型没有 journal，恢复算不出非 0 的 W，
/// 盖上去的值与回退行恰好相同 ⇒ 值层面看不出差别（被抛弃的记录被重放回来那一半落在 C124 里）。
fn world_rollback_row_protected(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    w.txn(t, &[Op::CreateObj; 4], false);
    w.publish();
    w.txn(t, &[Op::CreateObj; 3], false);
    w.publish(); // 将被回退抛弃的那次发布
    w.rollback_to_previous();
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    w.crash_and_recover_lost_roots(2); // 回退后的两个根都不可读 ⇒ 退回 R_old
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { rollback_abandoned: w.n_rollback_abandoned, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之八（第七轮反推腿 3.1）：恢复实例重放完上一实例的事务、又施加了自己的几个事务，然后在发布第一个根之前崩。
/// 下一次恢复要给 [所选根的实例, 新实例) 每个实例写行：按第八版 W 按实例分 ⇒ 恢复实例那行 W = 0（jsn 在实例边界断号，
/// 它的记录一条也重放不了）⇒ 它的单元判未发布；按第七版落地那句的字面把全局最大已施加号写给它 ⇒ 判已发布 ⇒ 复活。
fn world_recovery_own_txns(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], false); }
    w.publish();
    // 实例 1 的在飞 txg：两个事务被重放
    w.txn(t, &[Op::CreateObj], false);
    w.txn(t, &[Op::CreateObj], false);
    w.crash_and_recover(false); // 实例 2 发布重放结果
    // 实例 2 在下一个 checkpoint 里施加自己的三个事务（记录已追加），然后在下一个根之前崩
    w.crash_with_own_applied(3, t);
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { abandoned_unrewritten: w.n_abandoned_unrewritten, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之九（第七轮反推腿 2.1）：同一实例连着两个 checkpoint 都不分配新事务号（后台把同一个已关闭墓碑容器
/// 重写两版），容器写序逐字节相同。按写序择新是平局；按 (诞生代号, 实例代号) 择新分得开。
/// 写序冻结是生成器手工造的形状——模型按事务写容器，真实形态是固定点一次（见「它答不了的」）。
fn world_cont_same_wseq(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, Cfg { cont_wseq_frozen: true, reuse_bp: 0, ..cfg });
    let t = 1;
    w.txn(t, &[Op::CreateObj; 3], false);
    w.txn(t, &[Op::CreateInode; 2], false);
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    // 两次删除各自重写墓碑容器一版，两版之间不推进冻结的事务号 ⇒ 同身份两版、写序逐字节相同、诞生代号不同
    w.txn(t, &[Op::DeleteObj(keys[0])], false);
    w.publish();
    w.txn(t, &[Op::DeleteObj(keys[1])], false);
    w.publish();
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    (w, Closed::default())
}

/// 靶子世界一之七（第六轮反推腿 2.1）：最新已发布根的槽坏了、它之后的记录不可重放，恢复退到上一个根。
fn world_lost_root(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], false); }
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    w.txn(t, &[Op::CreateObj, Op::CreateObj, Op::Overwrite(keys[0])], false);
    w.publish(); // 这个根的槽将坏掉
    w.txn(t, &[Op::CreateObj], false); // 在飞
    w.crash_and_recover_lost_latest_root();
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { abandoned_unrewritten: w.n_abandoned_unrewritten, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之二：在飞 txg 里没有任何事务被重放（整个 txg 可以烧掉）。
fn world_crash_clean(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], false); }
    w.publish();
    for _ in 0..3 { w.txn(t, &[Op::CreateObj, Op::CreateObj], true); }
    w.crash_and_recover(false);
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { abandoned_unrewritten: w.n_abandoned_unrewritten, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之三：管理员回退到上一个根，抛弃一段已发布的时间线。
fn world_rollback(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    w.txn(t, &[Op::CreateObj; 4], false);
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    w.txn(t, &[Op::Overwrite(keys[0]), Op::Overwrite(keys[1])], false);
    w.publish();
    // 被抛弃的那次发布：建三个对象
    w.txn(t, &[Op::CreateObj; 3], false);
    w.publish();
    w.rollback_to_previous();
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { rollback_abandoned: w.n_rollback_abandoned, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之四：崩溃之后再回退到崩溃前实例的根——中间那个实例整个被抛弃，行 (i, 0, 0) 承重。
fn world_rollback_after_crash(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    w.txn(t, &[Op::CreateObj; 4], false);
    w.publish();
    w.txn(t, &[Op::CreateObj], false);
    w.txn(t, &[Op::CreateObj, Op::CreateObj], true);
    w.crash_and_recover(false); // 实例 2 的恢复根发布了被重放的那个对象
    w.txn(t, &[Op::CreateObj; 3], false);
    w.publish();
    w.rollback(2); // 回到实例 1 崩溃前的根：实例 2 发布的两个根全部抛弃
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    let c = Closed { rollback_abandoned: w.n_rollback_abandoned, ..Closed::default() };
    (w, c)
}

/// 靶子世界一之五：w = 2，每个单元两份逐字节相同的副本（第三轮反推腿 2.1：第一版每个单元都破「写序全序」的字面）。
fn world_replicas(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, Cfg { w: 2, ..cfg });
    let t = 1;
    w.txn(t, &[Op::CreateObj; 5], false);
    w.txn(t, &[Op::CreateInode; 3], false);
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    w.txn(t, &[Op::Overwrite(keys[0]), Op::Overwrite(keys[1])], false);
    w.publish();
    w.snapshot(t);
    let b = w.clone_head(0);
    w.txn(b, &[Op::Overwrite(keys[2])], false);
    w.publish();
    // 闭式：逐根数它引用的身份数（没有删除 ⇒ 每个根可见的身份恰是它引用的），不归并时每个身份一次平局
    let mut total = 0u64;
    for v in w.head_views.values().chain(w.snaps.iter()) {
        let ids: BTreeSet<Key> = v.alloc.iter().filter_map(|l| w.disk.get(l).map(|u| u.key)).collect();
        total += ids.len() as u64;
    }
    let c = Closed { live_versions: total, ..Closed::default() };
    (w, c)
}

/// 靶子世界二：删除 + 快照送走 + 墓碑回收，scrub 从未跑。
fn world_reclaim(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, Cfg { reuse_bp: 0, ..cfg });
    let t = 1;
    w.txn(t, &[Op::CreateObj; 6], false);
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    w.snapshot(t);
    let dels: Vec<Op> = keys.iter().take(4).map(|k| Op::DeleteObj(*k)).collect();
    w.txn(t, &dels, false);
    w.publish();
    w.snapshot(t); // 关掉开放的墓碑容器（第二个快照仍引用它的这一版）
    w.destroy_snapshot(0); // 引用死者的那个快照走了 ⇒ 按旧规则可回收
    w.destroy_snapshot(0); // 第二个快照也走了：没有任何根再引用墓碑容器的旧版本
    w.reclaim_tombstones();
    w.publish();
    // 分配器把被回收的墓碑容器的旧版本盖掉（它的落点已在可分配集合里），死者的旧版本仍可读
    let stale_tomb: Vec<u64> = w.disk.iter().filter(|(l, u)| matches!(u.payload, Payload::Tomb(_)) && !w.referenced_by_root(**l)).map(|(l, _)| *l).collect();
    for l in stale_tomb { w.force_reuse = Some(l); w.txn(t, &[Op::CreateObj], false); }
    w.publish();
    let readable = keys.iter().take(4).filter(|k| w.disk.values().any(|u| u.key == **k)).count() as u64;
    let c = Closed { reclaimed_readable: if cfg.scrub_gate { 0 } else { readable }, ..Closed::default() };
    (w, c)
}

/// 靶子世界三：克隆头与 origin 各自覆写同一批 key。
fn world_clone(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let a = 1;
    w.txn(a, &[Op::CreateObj; 5], false);
    w.publish();
    w.snapshot(a);
    let b = w.clone_head(0);
    let keys: Vec<Key> = w.heads[&a].data.keys().copied().take(3).collect();
    let ow: Vec<Op> = keys.iter().map(|k| Op::Overwrite(*k)).collect();
    w.txn(a, &ow, false);
    w.publish();
    w.txn(b, &ow, false);
    w.publish();
    let c = Closed { cross_head_keys: keys.len() as u64, ..Closed::default() };
    (w, c)
}

/// 靶子世界四：同一 txg 内跨事务的删 / 写与两次覆写。
fn world_same_txg(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    let t = 1;
    w.txn(t, &[Op::CreateObj; 6], false);
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    // k0：先删后写（两个事务）；k1：先写后删（两个事务）；k2、k3：两次覆写（两个事务）
    w.txn(t, &[Op::DeleteObj(keys[0])], false);
    w.txn(t, &[Op::Overwrite(keys[1]), Op::Overwrite(keys[2]), Op::Overwrite(keys[3])], false);
    w.txn(t, &[Op::Recreate(keys[0])], false); // k0：同一 txg 里先删后重建
    w.txn(t, &[Op::DeleteObj(keys[1]), Op::Overwrite(keys[2]), Op::Overwrite(keys[3])], false);
    w.publish();
    let c = Closed { same_txg_pairs: count_same_txg_pairs(&w), same_txg_overwrites: count_same_birth_ties(&w), ..Closed::default() };
    (w, c)
}

/// 生成器侧闭式：墓碑的死亡代号与该 key 某个可读单元的诞生代号相同的 (key, 死亡) 对数。
fn count_same_txg_pairs(w: &World) -> u64 {
    let mut seen: BTreeSet<(Key, WSeq)> = BTreeSet::new();
    for u in w.disk.values() {
        if let Payload::Tomb(recs) = &u.payload {
            for r in recs {
                if let TombRec::Kill { key, death, death_txg } = r {
                    if w.disk.values().any(|x| x.key == *key && x.birth == *death_txg) { seen.insert((*key, *death)); }
                }
            }
        }
    }
    seen.len() as u64
}

/// 生成器侧闭式：同一身份在它最大诞生代号上有 ≥ 2 个可读版本的身份数（含墓碑容器）。
fn count_same_birth_ties(w: &World) -> u64 {
    let mut by_key: BTreeMap<Key, Vec<Txg>> = BTreeMap::new();
    for u in w.disk.values() { by_key.entry(u.key).or_default().push(u.birth); }
    by_key.values().filter(|v| { let m = *v.iter().max().unwrap(); v.iter().filter(|b| **b == m).count() >= 2 }).count() as u64
}

/// 靶子世界五：容器分裂、合并退役、再删。
fn world_merge(seed: u64, cfg: Cfg) -> (World, Closed) {
    // 不复用释放空间：退役容器的最后一版要留在盘上可读，闭式才不是 0 == 0
    let mut w = World::new(seed, Cfg { reuse_bp: 0, ..cfg });
    let t = 1;
    // 建 6 个 inode ⇒ 两片叶 [1..4]、[5,6]
    w.txn(t, &[Op::CreateInode; 6], false);
    w.publish();
    // 删 1、2、3 ⇒ 左半剩 [4]；删 6 ⇒ 右半剩 [5]，合并：左吸收右 ⇒ [4,5]，右容器退役
    w.txn(t, &[Op::DeleteInode(1), Op::DeleteInode(2), Op::DeleteInode(3)], false);
    w.publish();
    w.txn(t, &[Op::DeleteInode(6)], false);
    w.publish();
    // 再删 5 ⇒ 5 曾住在退役容器里，去退役记录时它会从退役容器的旧版本里复活
    w.txn(t, &[Op::DeleteInode(5)], false);
    w.publish();
    // 闭式：退役容器最后一版里、在头的真值里已经不在的 inode 数（5 与 6）
    let alive: BTreeSet<u64> = w.head_views[&t].inodes.keys().copied().collect();
    let live_conts: BTreeSet<Key> = w.heads[&t].conts.iter().map(|c| c.key).collect();
    let mut dead_in_retired = 0u64;
    for u in w.disk.values() {
        if let (Key::Cont { kind: 2, .. }, Payload::Inodes(m)) = (u.key, &u.payload) {
            if !live_conts.contains(&u.key) { dead_in_retired += m.keys().filter(|i| !alive.contains(i)).count() as u64; }
        }
    }
    let c = Closed { retired_then_deleted: dead_in_retired, ..Closed::default() };
    (w, c)
}

/// 靶子世界六：旧版本被快照钉住，scrub 在快照销毁之前跑过，之后回收墓碑。
fn world_snap_after_scrub(seed: u64, cfg: Cfg) -> (World, Closed) {
    let mut w = World::new(seed, Cfg { reuse_bp: 0, ..cfg });
    let t = 1;
    w.txn(t, &[Op::CreateObj; 6], false);
    w.publish();
    let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
    w.snapshot(t); // S1 钉住六个旧版本
    let dels: Vec<Op> = keys.iter().take(4).map(|k| Op::DeleteObj(*k)).collect();
    w.txn(t, &dels, false);
    w.publish();
    w.snapshot(t); // S2：关掉墓碑容器
    w.destroy_snapshot(1); // S2 走了，S1 还钉着死者的旧版本
    for _ in 0..5 { w.publish(); } // 让死亡代号落到根环窗口之外，清扫水位才推得过它
    w.scrub(); // 抹头水位越过死亡代号，但死者的旧版本被 S1 钉着、没被抹
    w.txn(t, &[Op::CreateObj], false);
    w.publish();
    w.destroy_snapshot(0); // S1 走了：四个旧版本从此是可读垃圾
    w.reclaim_tombstones(); // 第一版的门：抹头水位 ≥ 死亡代号 ⇒ 放行
    w.publish();
    let stale_tomb: Vec<u64> = w.disk.iter().filter(|(l, u)| matches!(u.payload, Payload::Tomb(_)) && !w.referenced_by_root(**l)).map(|(l, _)| *l).collect();
    for l in stale_tomb { w.force_reuse = Some(l); w.txn(t, &[Op::CreateObj], false); }
    w.publish();
    let readable = keys.iter().take(4).filter(|k| w.disk.values().any(|u| u.key == **k)).count() as u64;
    let c = Closed { reclaimed_readable: if cfg.gate_v2 { 0 } else { readable }, ..Closed::default() };
    (w, c)
}

/// 混合随机世界：覆写 / 删除 / 快照 / 克隆 / 崩溃 / 回收 / scrub 全开。
fn world_mixed(seed: u64, cfg: Cfg, rounds: u32) -> (World, Closed) {
    let mut w = World::new(seed, cfg);
    w.txn(1, &[Op::CreateObj; 8], false);
    w.txn(1, &[Op::CreateInode; 6], false);
    w.publish();
    for round in 0..rounds {
        let trees: Vec<Tree> = w.heads.keys().copied().collect();
        let n_txn = 1 + w.rng.below(3);
        let mut crash_at: Option<u64> = None;
        if w.rng.chance_bp(1500) { crash_at = Some(w.rng.below(n_txn)); }
        for i in 0..n_txn {
            let t = trees[w.rng.below(trees.len() as u64) as usize];
            let mut ops: Vec<Op> = Vec::new();
            let keys: Vec<Key> = w.heads[&t].data.keys().copied().collect();
            let inos: Vec<u64> = w.heads[&t].conts.iter().flat_map(|c| c.recs.keys().copied()).collect();
            for _ in 0..(1 + w.rng.below(4)) {
                match w.rng.below(6) {
                    0 => ops.push(Op::CreateObj),
                    1 | 2 if !keys.is_empty() => ops.push(Op::Overwrite(keys[w.rng.below(keys.len() as u64) as usize])),
                    3 if !keys.is_empty() => ops.push(Op::DeleteObj(keys[w.rng.below(keys.len() as u64) as usize])),
                    4 => ops.push(Op::CreateInode),
                    5 if !inos.is_empty() => ops.push(Op::DeleteInode(inos[w.rng.below(inos.len() as u64) as usize])),
                    _ => ops.push(Op::CreateObj),
                }
            }
            // W1：同一事务里同一 key 只留最后一条净效果
            let mut seen: BTreeSet<Key> = BTreeSet::new();
            let mut seen_ino: BTreeSet<u64> = BTreeSet::new();
            ops.retain(|op| match op {
                Op::Overwrite(k) | Op::DeleteObj(k) | Op::WriteThenDelete(k) | Op::Recreate(k) => seen.insert(*k),
                Op::DeleteInode(i) => seen_ino.insert(*i),
                _ => true,
            });
            let abandon = crash_at.map_or(false, |c| i >= c);
            w.txn(t, &ops, abandon);
        }
        if crash_at.is_some() {
            let double = w.rng.chance_bp(2000);
            w.crash_and_recover(double);
        } else {
            w.publish();
        }
        match w.rng.below(10) {
            0 | 1 => { let t = trees[w.rng.below(trees.len() as u64) as usize]; w.snapshot(t); }
            2 if w.snapshot_count() > 0 && w.heads.len() < 4 => { let i = w.rng.below(w.snapshot_count() as u64) as usize; w.clone_head(i); }
            3 if w.snapshot_count() > 0 => {
                let i = w.rng.below(w.snapshot_count() as u64) as usize;
                let s = &w.snaps[i];
                let is_fork = w.ancestry.values().any(|(o, c)| *o == s.tree && *c == s.txg);
                if !is_fork { w.destroy_snapshot(i); }
            }
            4 => { w.reclaim_tombstones(); w.publish(); }
            5 if round % 3 == 0 => { w.scrub(); }
            6 if round % 7 == 3 && w.history.len() >= 2 => { w.rollback_to_previous(); }
            _ => {}
        }
    }
    (w, Closed::default())
}

impl Default for Closed {
    fn default() -> Self {
        Closed { abandoned_unrewritten: 0, reclaimed_readable: 0, cross_head_keys: 0, same_txg_pairs: 0, same_txg_overwrites: 0, retired_then_deleted: 0, rollback_abandoned: 0, live_versions: 0 }
    }
}

fn emit_cell(em: &mut Emitter, world: &str, seed: u64, arm: Arm, o: Out, c: Closed) -> String {
    em.emit_raw(&format!(
        "name=cell world={world} seed={seed} arm={} roots={} scanned={} wrong={} resurrected={} lost={} ambiguous={} divergence={} \
         cf_abandoned={} cf_reclaimed={} cf_cross={} cf_pairs={} cf_ow={} cf_retired={} cf_rollback={} cf_versions={}",
        arm.name, o.roots, o.scanned, o.wrong, o.resurrected, o.lost, o.ambiguous, o.divergence(),
        c.abandoned_unrewritten, c.reclaimed_readable, c.cross_head_keys, c.same_txg_pairs, c.same_txg_overwrites, c.retired_then_deleted, c.rollback_abandoned, c.live_versions))
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config note=扫描重建的现行版本判定 cont_cap={CONT_CAP} arms={} model=counting file_ops=0", ARMS.len())));
    let seeds = [1u64, 2, 3];
    for &seed in &seeds {
        let worlds: Vec<(&str, World, Closed)> = vec![
            { let (w, c) = world_crash(seed, Cfg::FULL, false); ("crash", w, c) },
            { let (w, c) = world_crash(seed, Cfg::FULL, true); ("crash_double", w, c) },
            { let (w, c) = world_crash_clean(seed, Cfg::FULL); ("crash_clean", w, c) },
            { let (w, c) = world_txn_fail(seed, Cfg::FULL); ("txn_fail", w, c) },
            { let (w, c) = world_txn_fail(seed, Cfg { records_stop_at_failure: false, ..Cfg::FULL }); ("txn_fail_records_continue", w, c) },
            { let (w, c) = world_lost_root(seed, Cfg::FULL); ("lost_root", w, c) },
            { let (w, c) = world_lost_root(seed, Cfg { row_tpub_from_chosen_root: false, ..Cfg::FULL }); ("lost_root_tpub_inflight", w, c) },
            { let (w, c) = world_recovery_own_txns(seed, Cfg::FULL); ("recovery_own_txns", w, c) },
            { let (w, c) = world_recovery_own_txns(seed, Cfg { row_w_per_instance: false, ..Cfg::FULL }); ("recovery_own_txns_global_w", w, c) },
            { let (w, c) = world_cont_same_wseq(seed, Cfg::FULL); ("cont_same_wseq", w, c) },
            { let (w, c) = world_rollback_row_protected(seed, Cfg::FULL); ("rollback_row_protected", w, c) },
            { let (w, c) = world_rollback(seed, Cfg::FULL); ("rollback", w, c) },
            { let (w, c) = world_rollback_after_crash(seed, Cfg::FULL); ("rollback_after_crash", w, c) },
            { let (w, c) = world_replicas(seed, Cfg::FULL); ("replicas", w, c) },
            { let (w, c) = world_reclaim(seed, Cfg { scrub_gate: false, ..Cfg::FULL }); ("reclaim_nogate", w, c) },
            { let (w, c) = world_reclaim(seed, Cfg::FULL); ("reclaim_gate", w, c) },
            { let (w, c) = world_snap_after_scrub(seed, Cfg { gate_v2: false, ..Cfg::FULL }); ("snap_after_scrub_gate_v1", w, c) },
            { let (w, c) = world_snap_after_scrub(seed, Cfg::FULL); ("snap_after_scrub_gate_v2", w, c) },
            { let (w, c) = world_clone(seed, Cfg::FULL); ("clone", w, c) },
            { let (w, c) = world_same_txg(seed, Cfg::FULL); ("same_txg", w, c) },
            { let (w, c) = world_merge(seed, Cfg::FULL); ("merge", w, c) },
            { let (w, c) = world_mixed(seed, Cfg { gate_v2: false, ..Cfg::FULL }, 60); ("mixed_gate_v1", w, c) },
            { let (w, c) = world_mixed(seed, Cfg::FULL, 60); ("mixed", w, c) },
        ];
        for (name, w, c) in &worlds {
            for arm in ARMS {
                let o = measure(w, arm, 0);
                println!("{}", emit_cell(&mut em, name, seed, arm, o, *c));
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_worlds(seed: u64) -> Vec<(&'static str, World, Closed)> {
        vec![
            { let (w, c) = world_crash(seed, Cfg::FULL, false); ("crash", w, c) },
            { let (w, c) = world_crash(seed, Cfg::FULL, true); ("crash_double", w, c) },
            { let (w, c) = world_crash_clean(seed, Cfg::FULL); ("crash_clean", w, c) },
            { let (w, c) = world_txn_fail(seed, Cfg::FULL); ("txn_fail", w, c) },
            { let (w, c) = world_lost_root(seed, Cfg::FULL); ("lost_root", w, c) },
            { let (w, c) = world_recovery_own_txns(seed, Cfg::FULL); ("recovery_own_txns", w, c) },
            { let (w, c) = world_cont_same_wseq(seed, Cfg::FULL); ("cont_same_wseq", w, c) },
            { let (w, c) = world_rollback_row_protected(seed, Cfg::FULL); ("rollback_row_protected", w, c) },
            { let (w, c) = world_rollback(seed, Cfg::FULL); ("rollback", w, c) },
            { let (w, c) = world_rollback_after_crash(seed, Cfg::FULL); ("rollback_after_crash", w, c) },
            { let (w, c) = world_replicas(seed, Cfg::FULL); ("replicas", w, c) },
            { let (w, c) = world_mixed(seed, Cfg { w: 2, ..Cfg::FULL }, 40); ("mixed_w2", w, c) },
            { let (w, c) = world_reclaim(seed, Cfg::FULL); ("reclaim_gate", w, c) },
            { let (w, c) = world_snap_after_scrub(seed, Cfg::FULL); ("snap_after_scrub_gate_v2", w, c) },
            { let (w, c) = world_clone(seed, Cfg::FULL); ("clone", w, c) },
            { let (w, c) = world_same_txg(seed, Cfg::FULL); ("same_txg", w, c) },
            { let (w, c) = world_merge(seed, Cfg::FULL); ("merge", w, c) },
            { let (w, c) = world_mixed(seed, Cfg::FULL, 60); ("mixed", w, c) },
        ]
    }

    /// 第一版的门（抹头水位 ≥ 死亡代号）在「快照销毁晚于 scrub」的世界上放走墓碑 ⇒ 全规则臂复活；
    /// 第二版的门（下限取最近一次根销毁的代号）⇒ 0。钉闭式。
    #[test]
    fn gate_v1_lets_snapshot_held_garbage_resurrect() {
        let (w, c) = world_snap_after_scrub(1, Cfg { gate_v2: false, ..Cfg::FULL });
        assert_eq!(c.reclaimed_readable, 4);
        let o = measure(&w, FULL, 0);
        assert_eq!(o.resurrected, c.reclaimed_readable, "{o:?}");
        let (w2, _) = world_snap_after_scrub(1, Cfg::FULL);
        assert_eq!(measure(&w2, FULL, 0).divergence(), 0);
    }

    /// 判据 1：全规则臂在全部世界、全部种子上四个数全 0。
    #[test]
    fn full_rule_is_exact_everywhere() {
        for seed in 1u64..=24 {
            for (name, w, _) in all_worlds(seed) {
                let o = measure(&w, FULL, 0);
                assert_eq!(o.divergence(), 0, "{name} seed={seed}: {o:?}");
                assert!(o.roots >= 1 && o.scanned >= 1, "{name}: 没扫到东西");
            }
        }
    }

    /// 判据 3：神谕臂恒 0（阳性对照）。
    #[test]
    fn oracle_is_exact_everywhere() {
        for seed in 1u64..=24 {
            for (name, w, _) in all_worlds(seed) {
                let o = measure(&w, ORACLE, 0);
                assert_eq!(o.divergence(), 0, "{name} seed={seed}: {o:?}");
            }
        }
    }

    /// 判据 3：扫到的单元数 = 生成器独立记的（写出 − 被盖掉 − 被抹掉）。
    #[test]
    fn scan_count_matches_generator() {
        for seed in 1u64..=12 {
            for (name, w, _) in all_worlds(seed) {
                let expect = w.n_written_units - w.n_overwritten_locs - w.n_invalidated;
                let o = measure(&w, FULL, 0);
                assert_eq!(o.scanned, expect, "{name} seed={seed}");
            }
        }
    }

    /// 判据 3：注入 k 处破坏必须报出恰 k。
    #[test]
    fn comparator_has_discriminating_power() {
        let (w, _) = world_mixed(7, Cfg::FULL, 40);
        for k in [1usize, 3, 5] {
            let o = measure(&w, FULL, k);
            assert_eq!(o.injected, k as u64);
            assert_eq!(o.wrong, k as u64, "注入 {k} 处只报出 {}", o.wrong);
        }
    }

    /// 判据 2：去掉实例水位 ⇒ 被抛弃且崩溃后没再写过的单元全部被判成现行。钉闭式。
    #[test]
    fn ablation_watermark_pins_abandoned_count() {
        for double in [false, true] {
            let (w, c) = world_crash(1, Cfg::FULL, double);
            assert!(c.abandoned_unrewritten >= 6, "靶子世界至少要有 6 个被抛弃单元");
            let o = measure(&w, NO_WATERMARK, 0);
            // 被抛弃的单元对每个根都可见（birth = 重发的 txg ≤ 根 txg），每个根各复活一次
            assert_eq!(o.resurrected, c.abandoned_unrewritten * o.roots, "double={double}: {o:?}");
            assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        }
    }

    /// 反推腿 1.4 的拆账：头里只有实例代号、没有计数器时，整个 txg 被抛弃的那一格分得出，
    /// 部分重放那一格分不出——被抛弃单元与已重放单元同实例同 txg。钉闭式。
    #[test]
    fn instance_only_fails_exactly_on_partial_replay() {
        let (w, c) = world_crash_clean(1, Cfg::FULL);
        assert!(c.abandoned_unrewritten >= 6);
        assert_eq!(measure(&w, INST_ONLY, 0).divergence(), 0, "整 txg 被抛弃：实例代号 + 最后发布 txg 就够");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        let (w2, c2) = world_crash(1, Cfg::FULL, false);
        let o = measure(&w2, INST_ONLY, 0);
        assert_eq!(o.resurrected, c2.abandoned_unrewritten * o.roots, "部分重放：{o:?}");
    }

    /// 第二轮正推腿：管理员回退到旧根是一次恢复。回退行缺席 ⇒ 被抛弃时间线里的单元全部复活。钉闭式。
    #[test]
    fn rollback_rows_pin_abandoned_timeline() {
        let (w, c) = world_rollback(1, Cfg::FULL);
        assert_eq!(c.rollback_abandoned, 3, "被抛弃的那次发布建了三个对象");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        let o = measure(&w, NO_WATERMARK, 0);
        assert_eq!(o.resurrected, c.rollback_abandoned * o.roots, "{o:?}");
        let (w2, _) = world_rollback(1, Cfg { write_watermarks: false, ..Cfg::FULL });
        let o2 = measure(&w2, FULL, 0);
        assert_eq!(o2.resurrected, c.rollback_abandoned * o2.roots, "不写回退行：{o2:?}");
    }

    /// 回退必须丢掉被抛弃时间线的 defer：那些「释放」从未发生，旧根还引用着它们。
    /// 不丢的话两次发布之后它们进可分配集合、被复用，旧根的单元被盖掉——连神谕臂都会丢。
    #[test]
    fn rollback_discards_abandoned_frees() {
        let mut w = World::new(2, Cfg { reuse_bp: 10_000, ..Cfg::FULL });
        w.txn(1, &[Op::CreateObj; 4], false);
        w.publish();
        let keys: Vec<Key> = w.heads[&1].data.keys().copied().collect();
        w.txn(1, &[Op::Overwrite(keys[0])], false);
        w.publish();
        // 被抛弃的那次发布覆写 k2、k3：它们的旧落点进了 defer
        w.txn(1, &[Op::Overwrite(keys[2]), Op::Overwrite(keys[3])], false);
        w.publish();
        w.rollback_to_previous();
        // 再发布几次让 defer 释放，然后以 100% 复用建对象：若被抛弃的释放没丢，旧根的 k2 / k3 落点会被盖掉
        for _ in 0..3 { w.txn(1, &[Op::CreateObj; 2], false); w.publish(); }
        let alive_locs: Vec<u64> = w.head_views[&1].data.values().copied().collect();
        assert!(alive_locs.iter().all(|l| w.disk.contains_key(l)), "旧根引用的落点被复用盖掉了");
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
    }

    /// 跨过一次崩溃回退：中间实例的行 (i, 0, 0) 承重——它发布过的根被整个抛弃。钉闭式。
    #[test]
    fn rollback_across_crash_pins_intermediate_instance() {
        let (w, c) = world_rollback_after_crash(1, Cfg::FULL);
        // 被抛弃的：崩溃 txg 里被重放的 1 个 + 实例 2 后来建的 3 个 + 崩溃时被抛弃但没再写过的 2 个
        assert_eq!(c.rollback_abandoned, 6, "{c:?}");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        let o = measure(&w, NO_WATERMARK, 0);
        assert_eq!(o.resurrected, c.rollback_abandoned * o.roots, "{o:?}");
    }

    /// 第三轮反推腿 5.2：回退后的新根必须压过根环里被抛弃的根，否则下次择新（按 txg 最大者）挑回被抛弃的线。
    #[test]
    fn rollback_new_root_outranks_abandoned_roots() {
        for skip in [true, false] {
            let (w, _) = world_rollback_after_crash(1, Cfg { rollback_skips_ring: skip, ..Cfg::FULL });
            let newest_live = w.history.iter().map(|h| h.0).max().unwrap();
            let newest_abandoned = w.abandoned_roots.iter().map(|r| r.0).max().unwrap();
            if skip {
                assert!(newest_live > newest_abandoned, "新时间线的根 {newest_live} 没压过被抛弃的根 {newest_abandoned}");
            } else {
                // 只回退一格时 T_old + 1 恰好盖掉被抛弃的第一个根；回退两格就压不过第二个
                assert!(newest_live <= newest_abandoned, "取 T_old + 1 本该压不过被抛弃的根");
            }
        }
    }

    /// 第四轮反推腿 6.3：对立臂 B（写序只上码 3、数据单元只有诞生代号）的残余错误钉成闭式。
    /// 崩溃部分重放那一格：数据孤儿的诞生代号 == 重发后已发布的 txg，只靠诞生代号判不出 ⇒ 全部复活；
    /// 同 txg 跨事务改同一数据 key ⇒ 歧义；容器那一格由写序解掉。
    #[test]
    fn arm_b_residual_is_pinned() {
        let (w, c) = world_crash(1, Cfg::FULL, false);
        let o = measure(&w, WSEQ_META_ONLY, 0);
        assert_eq!(o.resurrected, c.abandoned_unrewritten * o.roots, "崩溃孤儿：{o:?}");
        let (w2, c2) = world_same_txg(1, Cfg::FULL);
        let o2 = measure(&w2, WSEQ_META_ONLY, 0);
        // same_txg 世界里同诞生代号多版的身份是 3（两个数据 key + 墓碑容器），臂 B 解掉容器那一个；
        // 同 txg 里删与写的两对（先删后重建、先写后删）只有诞生代号时都排不了序 ⇒ 各记一次歧义，
        // 其中先写后删那一对被判成活的（复活 1）
        assert_eq!(o2.ambiguous, (c2.same_txg_overwrites - 1) + c2.same_txg_pairs, "同 txg：{o2:?}");
        assert_eq!(o2.resurrected, 1, "先写后删那一对：{o2:?}");
        // 没有崩溃、没有同 txg 多版的世界上臂 B 与全规则臂一样对
        for (name, w3, _) in [("clone", world_clone(1, Cfg::FULL).0, ()), ("merge", world_merge(1, Cfg::FULL).0, ())] {
            assert_eq!(measure(&w3, WSEQ_META_ONLY, 0).divergence(), 0, "{name}");
        }
    }

    /// 第三轮反推腿 7.4：清扫按规则（已释放标志 + 释放代 ≤ 最旧根 txg；孤儿按实例表）而不是真值。
    /// 规则清掉的 ⊆ 真值清掉的，且从不碰任何一个根环里的根引用的落点。
    #[test]
    fn rule_sweep_is_sound_against_oracle() {
        let mut cleared_total = 0u64;
        for seed in 1u64..=12 {
            let (w, _) = world_mixed(seed, Cfg::FULL, 60);
            let rule: BTreeSet<u64> = w.sweep_candidates().into_iter().collect();
            let oracle = w.scrub_oracle_set();
            assert!(rule.is_subset(&oracle), "seed={seed}: 规则清掉了真值不许清的落点 {:?}", rule.difference(&oracle).take(3).collect::<Vec<_>>());
            let ring_refs: BTreeSet<u64> = w.history.iter().flat_map(|h| h.2.values().chain(h.3.iter()).flat_map(|v| v.alloc.iter().copied())).collect();
            assert!(rule.is_disjoint(&ring_refs), "seed={seed}: 规则碰了根环里某个根引用的落点");
            cleared_total += rule.len() as u64;
        }
        assert!(cleared_total > 0, "十二个种子一个落点都没清到：规则没在工作");
    }

    /// 等价性留档：回退时不丢被抛弃时间线的 defer，释放时的引用检查同样挡得住复用。
    #[test]
    fn abandoned_defer_is_also_guarded_by_reference_check() {
        for keep in [true, false] {
            let mut w = World::new(2, Cfg { reuse_bp: 10_000, retain_abandoned_defer: keep, ..Cfg::FULL });
            w.txn(1, &[Op::CreateObj; 4], false);
            w.publish();
            let keys: Vec<Key> = w.heads[&1].data.keys().copied().collect();
            w.txn(1, &[Op::Overwrite(keys[0])], false);
            w.publish();
            w.txn(1, &[Op::Overwrite(keys[2]), Op::Overwrite(keys[3])], false);
            w.publish();
            w.rollback_to_previous();
            for _ in 0..3 { w.txn(1, &[Op::CreateObj; 2], false); w.publish(); }
            assert_eq!(measure(&w, ORACLE, 0).divergence(), 0, "keep={keep}");
            assert_eq!(measure(&w, FULL, 0).divergence(), 0, "keep={keep}");
        }
    }

    /// 第三轮反推腿 2.1：w = 2 下每个逻辑版本都有两份同写序的可读副本。
    /// 先归并副本再择版本 ⇒ 全规则臂 0；不归并 ⇒ 每个活着的逻辑版本一次平局。钉闭式。
    #[test]
    fn replica_merge_is_load_bearing() {
        let (w, c) = world_replicas(1, Cfg::FULL);
        assert!(c.live_versions >= 10, "{c:?}");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        let o = measure(&w, NO_REPLICA_MERGE, 0);
        assert_eq!(o.ambiguous, c.live_versions, "{o:?}");
        // 混合世界开 w = 2 同样全 0
        for seed in 1u64..=8 {
            let (w2, _) = world_mixed(seed, Cfg { w: 2, ..Cfg::FULL }, 40);
            assert_eq!(measure(&w2, FULL, 0).divergence(), 0, "mixed_w2 seed={seed}");
        }
    }

    /// 第五轮反推腿 1.1：事务在发出单元写之后失败。第六版让它成为实例结束、后续记录不追加 ⇒ 行的 W 是前缀、全规则臂 0；
    /// 第五版的形态（后续事务照常追加、作废信息随崩溃丢失）⇒ 失败事务的单元 n ≤ W 判已发布 ⇒ 复活。钉闭式。
    #[test]
    fn failed_txn_after_write_is_pinned() {
        let (w, c) = world_txn_fail(1, Cfg::FULL);
        assert_eq!(c.abandoned_unrewritten, 4, "B 两个 + C、D 各一个都没能追加记录");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        let (w2, c2) = world_txn_fail(1, Cfg { records_stop_at_failure: false, ..Cfg::FULL });
        assert_eq!(c2.abandoned_unrewritten, 2, "只有 B 的两个单元没有记录");
        let o = measure(&w2, FULL, 0);
        assert_eq!(o.resurrected, c2.abandoned_unrewritten * o.roots, "作废信息丢失：{o:?}");
    }

    /// 第八轮反推腿 12.1：回退行不许被后来的恢复覆盖这条规则，今天在模型里就量得出「保护确实挡下过写」——
    /// 不必等崩溃点重放。关掉保护 ⇒ 挡下次数归零（值层面看不出差别：模型没有 journal，恢复算不出非 0 的 W）。
    #[test]
    fn rollback_row_protection_actually_blocks_a_write() {
        // 就地造，好在回退那一刻把行的值抓下来
        let mut w = World::new(1, Cfg::FULL);
        let t = 1;
        w.txn(t, &[Op::CreateObj; 4], false);
        w.publish();
        w.txn(t, &[Op::CreateObj; 3], false);
        w.publish();
        w.rollback_to_previous();
        let row_at_rollback = w.watermarks[&1];
        assert!(w.rollback_rows.contains(&1), "回退行登记在实例 1 上");
        assert_eq!(row_at_rollback.1, 0, "回退行的 W 恒 0");
        w.txn(t, &[Op::CreateObj], false);
        w.publish();
        w.crash_and_recover_lost_roots(2); // 恢复落回 R_old：范围 [R_old 的实例, 新实例) 正好盖到回退行
        assert!(w.rollback_row_writes_blocked >= 1, "本该盖到回退行、被保护挡下：{}", w.rollback_row_writes_blocked);
        assert_eq!(w.watermarks[&1], row_at_rollback, "回退行的三段一个字节没变");
        w.txn(t, &[Op::CreateObj], false);
        w.publish();
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        assert_eq!(w.n_rollback_abandoned, 3, "被回退抛弃的那次发布建了三个对象");
    }

    /// 第七轮反推腿 3.1：恢复实例施加过自己的事务之后再崩。W 按实例分 ⇒ 它那行 W = 0、它的单元判未发布；
    /// 按第七版落地那句的字面把全局最大已施加号写给它 ⇒ 它的单元 n ≤ W 判已发布 ⇒ 复活。钉闭式。
    #[test]
    fn recovery_instance_row_w_is_per_instance() {
        let (w, c) = world_recovery_own_txns(1, Cfg::FULL);
        assert_eq!(c.abandoned_unrewritten, 3, "恢复实例自己写的 3 个单元跨不过实例边界");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        let (w2, c2) = world_recovery_own_txns(1, Cfg { row_w_per_instance: false, ..Cfg::FULL });
        let o = measure(&w2, FULL, 0);
        assert_eq!(o.resurrected, c2.abandoned_unrewritten * o.roots, "全局 W 写给恢复实例：{o:?}");
    }

    /// 第七轮反推腿 2.1：同实例连着两个 checkpoint 都不分配新事务号时，两版容器的写序逐字节相同。
    /// 码 3 按 (诞生代号, 实例代号) 择新 ⇒ 0；按写序择新 ⇒ 平局记歧义。钉闭式。
    #[test]
    fn cont_key_by_birth_breaks_the_wseq_tie() {
        let (w, _) = world_cont_same_wseq(1, Cfg::FULL);
        let versions: Vec<(Key, Txg, WSeq)> = w.disk.values().filter(|u| matches!(u.key, Key::Cont { kind: 1, .. })).map(|u| (u.key, u.birth, u.wseq)).collect();
        let same_wseq_pairs = versions.iter().enumerate().flat_map(|(i, a)| versions[i + 1..].iter().map(move |b| (a, b)))
            .filter(|(a, b)| a.0 == b.0 && a.2 == b.2 && a.1 != b.1).count();
        assert!(same_wseq_pairs >= 1, "靶子世界要造出同身份、同写序、不同诞生代号的两版：{versions:?}");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        let o = measure(&w, CONT_KEY_WSEQ, 0);
        assert_eq!(o.ambiguous, same_wseq_pairs as u64, "按写序择新的平局数：{o:?}");
    }

    /// 第六轮反推腿 2.1：恢复选了非最新根时，行的 T_pub 必须是所选根的 txg。按第六版字面取「在飞 txg − 1」
    /// 会把槽坏掉的那个 checkpoint 里的单元判成已发布 ⇒ 复活；取所选根 txg ⇒ 0。钉闭式。
    #[test]
    fn lost_root_row_uses_chosen_root_txg() {
        let (w, c) = world_lost_root(1, Cfg::FULL);
        assert_eq!(c.abandoned_unrewritten, 4, "丢掉的 checkpoint 建 2 覆写 1 + 在飞建 1");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
        let (w2, c2) = world_lost_root(1, Cfg { row_tpub_from_chosen_root: false, ..Cfg::FULL });
        let o = measure(&w2, FULL, 0);
        // 丢掉的 checkpoint 里的 3 个单元 b = T_pub 判已发布（其中覆写那一版压过旧版：复活 2 + 错 1）；在飞那 1 个 b = T_pub + 1、W = 0 仍未发布
        assert_eq!(o.resurrected + o.wrong, c2.abandoned_unrewritten - 1, "{o:?}");
    }

    /// 第五轮反推腿 4.1：回退之后被抛弃的根留在环里；清扫把它独占的单元抹掉之后，若 kind 0 行按「没有未发布的可读单元」
    /// 就回收，被抛弃的根整批回到回退候选集。第六版加合取「根环里没有该实例的根」⇒ 行留着、候选集正确。钉闭式。
    #[test]
    fn row_reclaim_keeps_abandoned_roots_out_of_candidates() {
        for checks_ring in [true, false] {
            let mut w = World::new(4, Cfg { row_reclaim_checks_ring: checks_ring, reuse_bp: 0, ..Cfg::FULL });
            w.txn(1, &[Op::CreateObj; 4], false);
            w.publish();
            w.txn(1, &[Op::CreateObj; 2], false);
            w.publish();
            w.txn(1, &[Op::CreateObj; 3], false);
            w.publish(); // 将被抛弃的根
            w.rollback_to_previous();
            let truth: BTreeSet<(Txg, u32)> = w.history.iter().map(|h| (h.0, h.1)).collect();
            assert_eq!(w.abandoned_roots.len(), 1);
            let before: BTreeSet<(Txg, u32)> = w.rollback_candidates().into_iter().collect();
            assert_eq!(before, truth, "回退刚做完：被抛弃的根不在候选集里");
            w.scrub(); // 抹掉被抛弃时间线独占的单元
            w.reclaim_rows();
            let after: BTreeSet<(Txg, u32)> = w.rollback_candidates().into_iter().collect();
            let wrong = after.difference(&truth).count();
            if checks_ring { assert_eq!(wrong, 0, "{after:?}"); assert!(w.watermarks.contains_key(&1)); }
            else { assert_eq!(wrong, w.abandoned_roots.len(), "行被回收后被抛弃的根回到候选集：{after:?}"); }
            assert_eq!(measure(&w, FULL, 0).divergence(), 0, "checks_ring={checks_ring}");
        }
    }

    /// 第五轮反推腿 2.2：容器在 checkpoint 固定点写出，一个没发布的 checkpoint 的容器版本一律未发布，
    /// 恢复实例重写它们。崩溃世界里每个被重放事务碰过的容器都留下一版旧实例的孤儿，清扫按行把它们清掉。
    #[test]
    fn inflight_containers_are_rewritten_by_recovery() {
        let mut w = World::new(6, Cfg { reuse_bp: 0, ..Cfg::FULL });
        w.txn(1, &[Op::CreateInode; 3], false);
        w.publish();
        w.txn(1, &[Op::CreateInode], false); // 在飞：被重放
        w.txn(1, &[Op::CreateInode], true);  // 在飞：被抛弃
        w.crash_and_recover(false);
        let inflight_txg = w.txg - 1;
        let stale: Vec<u64> = w.disk.iter().filter(|(_, u)| matches!(u.key, Key::Cont { .. }) && u.birth == inflight_txg && u.wseq.inst == 1).map(|(l, _)| *l).collect();
        assert!(stale.len() >= 2, "至少一版被重放事务写的容器 + 一版被抛弃事务写的容器留在盘上：{}", stale.len());
        assert!(stale.iter().all(|l| !row_published(&w.watermarks, &w.disk[l])), "旧实例在飞 checkpoint 的容器一律未发布");
        let swept: BTreeSet<u64> = w.sweep_candidates().into_iter().collect();
        assert!(stale.iter().all(|l| swept.contains(l)), "它们是孤儿，清扫要清掉");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        assert_eq!(measure(&w, ORACLE, 0).divergence(), 0);
    }

    /// 判据 2：写路径不写水位行 ⇒ 全规则臂也救不了（承重的是行，不是规则）。
    #[test]
    fn missing_watermark_rows_break_full_rule() {
        let (w, c) = world_crash(1, Cfg { write_watermarks: false, ..Cfg::FULL }, false);
        let o = measure(&w, FULL, 0);
        assert_eq!(o.resurrected, c.abandoned_unrewritten * o.roots, "{o:?}");
    }

    /// 判据 2：去掉 scrub 门 ⇒ 墓碑回收之后仍可读的旧版本全数复活；门开着则 0。钉闭式。
    #[test]
    fn ablation_scrub_gate_pins_resurrection() {
        let (w, c) = world_reclaim(1, Cfg { scrub_gate: false, ..Cfg::FULL });
        assert_eq!(c.reclaimed_readable, 4, "四个死者的旧版本都还可读");
        let o = measure(&w, FULL, 0);
        assert_eq!(o.resurrected, c.reclaimed_readable, "{o:?}");
        let (w2, _) = world_reclaim(1, Cfg::FULL);
        assert_eq!(measure(&w2, FULL, 0).divergence(), 0);
    }

    /// 判据 2：去掉祖先表 ⇒ origin 头视野里克隆头写的更新版本被当成现行。钉闭式。
    #[test]
    fn ablation_ancestry_pins_cross_head_wrong() {
        let (w, c) = world_clone(1, Cfg::FULL);
        assert_eq!(c.cross_head_keys, 3);
        let o = measure(&w, NO_ANCESTRY, 0);
        // origin 头 A 的 3 个 key 错到 B 的版本；快照 S 靠「诞生代号 ≤ 根 txg」挡住；B 自己对
        assert_eq!(o.wrong, c.cross_head_keys, "{o:?}");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
    }

    /// 判据 2：只有代号没有写序 ⇒ 同一 txg 里两次覆写分不出。钉闭式。
    #[test]
    fn ablation_wseq_pins_same_txg_ambiguity() {
        let (w, c) = world_same_txg(1, Cfg::FULL);
        assert_eq!(c.same_txg_overwrites, 3, "两个数据 key 各两次覆写 + 墓碑容器自己的两版");
        let o = measure(&w, NO_WSEQ, 0);
        assert_eq!(o.ambiguous, c.same_txg_overwrites, "{o:?}");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
    }

    /// 判据 2：墓碑只有死亡代号没有死亡写序 ⇒ 同一 txg 里的删 / 写分不出。钉闭式。
    #[test]
    fn ablation_death_wseq_pins_same_txg_pairs() {
        let (w, c) = world_same_txg(1, Cfg::FULL);
        assert_eq!(c.same_txg_pairs, 2, "先删后重建一对、先写后删一对");
        let o = measure(&w, NO_DEATH_WSEQ, 0);
        assert_eq!(o.ambiguous, c.same_txg_pairs, "{o:?}");
    }

    /// 判据 2：去掉容器退役记录 ⇒ 退役容器里后来被删的 inode 复活。钉闭式。
    #[test]
    fn ablation_retire_pins_inode_resurrection() {
        let (w, c) = world_merge(1, Cfg::FULL);
        assert_eq!(c.retired_then_deleted, 2, "退役容器最后一版里 5 与 6 都已死");
        let o = measure(&w, NO_RETIRE, 0);
        assert_eq!(o.resurrected, c.retired_then_deleted, "{o:?}");
        assert_eq!(measure(&w, FULL, 0).divergence(), 0);
        // 写路径不写退役记录 ⇒ 全规则臂同样复活（承重的是记录）
        let (w2, _) = world_merge(1, Cfg { retire_record: false, ..Cfg::FULL });
        assert_eq!(measure(&w2, FULL, 0).resurrected, c.retired_then_deleted);
    }

    /// 事务内先写后删：作废与否全规则臂都对（墓碑赢平局）；让单元赢平局的消融臂在不作废时复活。
    #[test]
    fn write_then_delete_tie_rule() {
        for inval in [true, false] {
            let mut w = World::new(3, Cfg { invalidate_in_txn: inval, ..Cfg::FULL });
            w.txn(1, &[Op::CreateObj; 3], false);
            w.publish();
            let keys: Vec<Key> = w.heads[&1].data.keys().copied().collect();
            w.txn(1, &[Op::WriteThenDelete(keys[0]), Op::WriteThenDelete(keys[1])], false);
            w.publish();
            assert_eq!(measure(&w, FULL, 0).divergence(), 0, "inval={inval}");
            let o = measure(&w, TIE_UNIT_WINS, 0);
            if inval { assert_eq!(o.divergence(), 0, "{o:?}"); } else { assert_eq!(o.resurrected, 2, "{o:?}"); }
        }
    }

    /// 跨头首次 COW 不重生 ⇒ origin 视野里看到克隆写的同身份新版本。
    #[test]
    fn rebirth_is_load_bearing() {
        for rebirth in [true, false] {
            let mut w = World::new(5, Cfg { rebirth, ..Cfg::FULL });
            w.txn(1, &[Op::CreateInode; 3], false);
            w.publish();
            w.snapshot(1);
            let b = w.clone_head(0);
            w.txn(b, &[Op::CreateInode], false);
            w.publish();
            let o = measure(&w, FULL, 0);
            if rebirth { assert_eq!(o.divergence(), 0, "{o:?}"); } else { assert!(o.resurrected >= 1, "{o:?}"); }
        }
    }

    /// 祖先表：克隆的克隆与旁支克隆各自只看自己那条链。
    #[test]
    fn ancestry_chain_limits() {
        let mut anc = BTreeMap::new();
        anc.insert(2, (1, 10));
        anc.insert(3, (2, 20));
        anc.insert(4, (1, 10));
        assert_eq!(ancestry_of(&anc, 3), vec![(3, u64::MAX), (2, 20), (1, 10)]);
        assert_eq!(ancestry_of(&anc, 4), vec![(4, u64::MAX), (1, 10)]);
        assert_eq!(ancestry_of(&anc, 1), vec![(1, u64::MAX)]);
    }

    /// 调试用：打印神谕臂在混合世界上对不上的根与 key。
    #[test]
    #[ignore]
    fn debug_mixed_oracle() {
        let seed: u64 = std::env::var("E104_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let w2: u8 = std::env::var("E104_W").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let rounds: u32 = std::env::var("E104_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
        let (w, _) = world_mixed(seed, Cfg { w: w2, ..Cfg::FULL }, rounds);
        println!("inst={} txg={} watermarks={:?} ancestry={:?} scrub_wm={}", w.inst, w.txg, w.watermarks, w.ancestry, w.scrub_wm);
        let roots: Vec<&View> = w.head_views.values().chain(w.snaps.iter()).collect();
        let arm = if std::env::var("E104_ARM").ok().as_deref() == Some("full") { FULL } else { ORACLE };
        for r in roots {
            let re = rebuild_root(&w, r, arm);
            for (k, l) in &r.data {
                match re.data.get(k) { None => println!("root tree={} txg={} LOST {k:?} loc={l} on_disk={} unit={:?} in_alloc={}", r.tree, r.txg, w.disk.contains_key(l), w.disk.get(l).map(|u| (u.key, u.writer, u.birth, u.wseq)), r.alloc.contains(l)), Some(p) if p != l => println!("WRONG {k:?}"), _ => {} }
            }
            for (k, l) in &re.data { if !r.data.contains_key(k) {
                println!("root tree={} txg={} RESURRECTED {k:?} at loc={l} unit={:?}", r.tree, r.txg, w.disk.get(l).map(|u| (u.writer, u.birth, u.wseq)));
                for (l2, u) in &w.disk { if u.key == *k { println!("    version loc={l2} writer={} birth={} wseq={:?} referenced={}", u.writer, u.birth, u.wseq, w.referenced_by_root(*l2)); } }
                for (l2, u) in &w.disk { if let Payload::Tomb(recs) = &u.payload { for rec in recs { if let TombRec::Kill { key, death, death_txg } = rec { if key == k { println!("    tomb in loc={l2} cont={:?} birth={} wseq={:?} referenced={} death={death:?} death_txg={death_txg}", u.key, u.birth, u.wseq, w.referenced_by_root(*l2)); } } } } }
                for (i, h) in w.heads.iter() { if let Some((ck, cl, recs)) = &h.tomb_open { if recs.iter().any(|rec| matches!(rec, TombRec::Kill { key, .. } if key == k)) { println!("    open tomb of head {i}: {ck:?} loc={cl}"); } } for (ck, cl, recs) in &h.tomb_closed { if recs.iter().any(|rec| matches!(rec, TombRec::Kill { key, .. } if key == k)) { println!("    closed tomb of head {i}: {ck:?} loc={cl}"); } } }
            } }
            for (ino, (ck, cl, ver)) in &r.inodes {
                match re.inodes.get(ino) { None => {
                    println!("root tree={} txg={} LOST inode {ino} cont={ck:?} loc={cl} on_disk={} unit={:?} in_alloc={}", r.tree, r.txg, w.disk.contains_key(cl), w.disk.get(cl).map(|u| (u.key, u.birth, u.wseq)), r.alloc.contains(cl));
                    for (l2, u) in &w.disk { if u.key == *ck { println!("    cont version loc={l2} birth={} wseq={:?} referenced={} in_root_alloc={} recs={:?}", u.birth, u.wseq, w.referenced_by_root(*l2), r.alloc.contains(l2), match &u.payload { Payload::Inodes(m) => m.keys().collect::<Vec<_>>(), _ => vec![] }); } }
                    for (l2, u) in &w.disk { if let Payload::Tomb(recs) = &u.payload { for rec in recs { if let TombRec::Retire { cont, at_txg } = rec { if cont == ck { println!("    retire rec in loc={l2} tomb={:?} birth={} wseq={:?} referenced={} in_root_alloc={} at_txg={at_txg}", u.key, u.birth, u.wseq, w.referenced_by_root(*l2), r.alloc.contains(l2)); } } } } }
                    for (i, h) in w.heads.iter() { for c in &h.conts { if c.key == *ck { println!("    head {i} cont loc={} dirty={} shared={}", c.loc, c.dirty, c.shared); } } }
                }, Some(v) if v != ver => println!("WRONG inode {ino}"), _ => {} }
            }
            for ino in re.inodes.keys() { if !r.inodes.contains_key(ino) {
                println!("root tree={} txg={} RESURRECTED inode {ino}", r.tree, r.txg);
                if r.txg == 79 && r.tree == 1 {
                    for (l2, u) in &w.disk { if let Payload::Inodes(m) = &u.payload { if m.contains_key(ino) { println!("    cont version loc={l2} key={:?} writer={} birth={} wseq={:?} referenced={} live_cont={}", u.key, u.writer, u.birth, u.wseq, w.referenced_by_root(*l2), w.heads.values().any(|h| h.conts.iter().any(|c| c.key == u.key))); } } }
                    for (l2, u) in &w.disk { if let Payload::Tomb(recs) = &u.payload { for rec in recs { if let TombRec::Retire { cont, at_txg } = rec { if w.disk.values().any(|x| x.key == *cont && matches!(&x.payload, Payload::Inodes(m) if m.contains_key(ino))) { println!("    retire rec in loc={l2} cont={cont:?} at_txg={at_txg} referenced={}", w.referenced_by_root(*l2)); } } } } }
                }
            } }
        }
    }

    /// 确定性：同一种子两次跑逐字节相同。
    #[test]
    fn deterministic() {
        let a = measure(&world_mixed(9, Cfg::FULL, 50).0, NO_WSEQ, 0);
        let b = measure(&world_mixed(9, Cfg::FULL, 50).0, NO_WSEQ, 0);
        assert_eq!(a, b);
    }
}
