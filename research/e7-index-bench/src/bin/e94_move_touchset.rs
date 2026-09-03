//! E94：搬共享落点的触碰面 —— D26（后台整理与放置回收）未定项 5 的数据前置。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/94-搬共享落点的触碰面.md，跑前写死）
//!
//! 一棵扇出 F 的持久 COW 树装 L 个 extent 指针（节点不可变、更新走路径复制、结构共享）。
//! 每 u 次更新立一个快照（保留旧根 + 接管活头 deadlist，D5 的交接形态），保留 K 个。
//! 历史造好后选一批 N 个占用槽当整理批次，三种位置权威架构各搬一遍，数各自改写了什么。
//!
//! | 臂 | 位置权威 | 搬动要改 |
//! |---|---|---|
//! | ptr_rewrite | 每棵引用树指针里的位置条目 | 全部引用叶 COW + 各根涟漪 + 快照根记录 + deadlist 条目 |
//! | central_map | 单点「逻辑身份→物理」映射 | 映射树 COW；主树/deadlist/快照根不动 |
//! | skip_shared | 同 ptr，但只搬引用树恰一棵的 | 可搬集合缩水 |
//! | move_new_birth（负臂） | — | 证明 birth 必须跟逻辑版本走：重指派必出可数违例 |
//!
//! ## 判据（跑前写死）
//!
//! 1. 手算锚点：L=8、F=2 的两个小场景，COW 节点数与根引用更新数钉死（单测）。
//! 2. 语义守恒：搬完后每个保留根逐 key 读回的逻辑内容不变；旧槽零引用；占用数恒等。
//! 3. 阳性对照：注入「漏改一处引用」，审计必须抓到（三臂都过闸）。
//! 4. move_new_birth 的违例数 > 0 且恰等于「被 ≥1 个快照引用且被搬动」的版本数（独立算）。
//! 5. 线性历史下区间判据与树审计必须逐版本一致（D5 的等价性当模型自检用）。
//!
//! ## 它答不了的
//!
//! 计数模型；映射树按「key 空间同几何」近似（叶装该 key 全部代的条目）；
//! deadlist 载体的 COW 不建模（只数条目重写）；无崩溃语义；无读路径代价。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

const L: usize = 4096; // key 数（= F^3）
const F: usize = 16; // 扇出
const S: usize = 32768; // 槽数
const N_BATCH: usize = 64; // 整理批次大小

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if s == 0 {
            s = 0xDEAD_BEEF;
        }
        Rng(s)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[derive(Clone)]
enum Node {
    Inner(Vec<u32>),
    Leaf(Vec<Entry>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Entry {
    ver: u32,
    slot: u32,
}

#[derive(Clone, Copy)]
struct Version {
    key: u32,
    birth: u64,
    death: u64, // u64::MAX = 还活着
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DlEntry {
    ver: u32,
    slot: u32,
}

#[derive(Clone)]
struct Snap {
    txg: u64,
    /// 上一个快照的 txg（销毁级联要按 birth > prev(S).txg 过滤，D5 约定 B）。
    prev_txg: u64,
    root: u32,
    deadlist: Vec<DlEntry>,
}

#[derive(Clone)]
struct World {
    f: usize,
    l: usize,
    depth: usize, // 路径节点数（根到叶）
    nodes: Vec<Node>,
    live_root: u32,
    snaps: Vec<Snap>,
    head_deadlist: Vec<DlEntry>,
    prev_snap_txg: u64,
    versions: Vec<Version>,
    free: BTreeSet<u32>,
    occupant: BTreeMap<u32, u32>, // slot → ver（分配记录的模型）
    /// central_map 臂的权威映射：ver → slot。ptr 臂不读它。
    map_slot: BTreeMap<u32, u32>,
    txg: u64,
}

impl World {
    fn new(f: usize, l: usize, s: usize) -> World {
        let mut depth = 0;
        let mut span = 1;
        while span < l {
            span *= f;
            depth += 1;
        }
        assert_eq!(span, l, "L 必须是 F 的整幂");
        let mut nodes = Vec::new();
        // 自底向上建初始树：key k = ver k = 槽 k，birth 1
        let mut level: Vec<u32> = (0..l / f)
            .map(|i| {
                let entries: Vec<Entry> = (0..f)
                    .map(|j| Entry { ver: (i * f + j) as u32, slot: (i * f + j) as u32 })
                    .collect();
                nodes.push(Node::Leaf(entries));
                (nodes.len() - 1) as u32
            })
            .collect();
        while level.len() > 1 {
            level = level
                .chunks(f)
                .map(|c| {
                    nodes.push(Node::Inner(c.to_vec()));
                    (nodes.len() - 1) as u32
                })
                .collect();
        }
        let versions: Vec<Version> =
            (0..l).map(|k| Version { key: k as u32, birth: 1, death: u64::MAX }).collect();
        let occupant: BTreeMap<u32, u32> = (0..l as u32).map(|k| (k, k)).collect();
        let map_slot: BTreeMap<u32, u32> = (0..l as u32).map(|k| (k, k)).collect();
        World {
            f,
            l,
            depth,
            nodes,
            live_root: level[0],
            snaps: Vec::new(),
            head_deadlist: Vec::new(),
            prev_snap_txg: 0,
            versions,
            free: (l as u32..s as u32).collect(),
            occupant,
            map_slot,
            txg: 1,
        }
    }

    /// 根到叶的 child 下标序列。
    fn digits(&self, key: usize) -> Vec<usize> {
        let mut span = self.l;
        let mut ds = Vec::with_capacity(self.depth);
        let mut k = key;
        for _ in 0..self.depth {
            span /= self.f;
            ds.push(k / span);
            k %= span;
        }
        ds
    }

    /// 活头写一个 key：COW 路径复制 + D5 的 kill 规则。
    fn update(&mut self, key: usize) {
        self.txg += 1;
        let slot = *self.free.iter().next().expect("槽用尽");
        self.free.remove(&slot);
        let ver = self.versions.len() as u32;
        self.versions.push(Version { key: key as u32, birth: self.txg, death: u64::MAX });
        self.occupant.insert(slot, ver);
        self.map_slot.insert(ver, slot);

        let ds = self.digits(key);
        // 收集路径节点
        let mut path = Vec::with_capacity(self.depth);
        let mut cur = self.live_root;
        for &d in &ds {
            path.push((cur, d));
            cur = match &self.nodes[cur as usize] {
                Node::Inner(cs) => cs[d],
                Node::Leaf(_) => break,
            };
        }
        // path 最后一项是叶（digits 的最后一位是叶内下标）
        let (leaf_id, leaf_idx) = *path.last().unwrap();
        let old_entry = match &self.nodes[leaf_id as usize] {
            Node::Leaf(es) => es[leaf_idx],
            _ => unreachable!(),
        };
        // 新叶
        let mut es = match &self.nodes[leaf_id as usize] {
            Node::Leaf(es) => es.clone(),
            _ => unreachable!(),
        };
        es[leaf_idx] = Entry { ver, slot };
        self.nodes.push(Node::Leaf(es));
        let mut new_child = (self.nodes.len() - 1) as u32;
        // 自底向上复制内部节点
        for &(node_id, idx) in path.iter().rev().skip(1) {
            let mut cs = match &self.nodes[node_id as usize] {
                Node::Inner(cs) => cs.clone(),
                _ => unreachable!(),
            };
            cs[idx] = new_child;
            self.nodes.push(Node::Inner(cs));
            new_child = (self.nodes.len() - 1) as u32;
        }
        self.live_root = new_child;

        // kill 旧版本（D5：birth > prev_snap_txg ⇒ 立即释放，否则进活头 deadlist）
        let old_ver = old_entry.ver;
        self.versions[old_ver as usize].death = self.txg;
        if self.versions[old_ver as usize].birth > self.prev_snap_txg {
            self.occupant.remove(&old_entry.slot);
            self.map_slot.remove(&old_ver);
            self.free.insert(old_entry.slot);
        } else {
            self.head_deadlist.push(DlEntry { ver: old_ver, slot: old_entry.slot });
        }
    }

    /// 建快照：保留根 + 接管活头 deadlist（D5 的交接）。
    fn snapshot(&mut self) {
        self.snaps.push(Snap {
            txg: self.txg,
            prev_txg: self.prev_snap_txg,
            root: self.live_root,
            deadlist: std::mem::take(&mut self.head_deadlist),
        });
        self.prev_snap_txg = self.txg;
    }

    /// 销毁最旧的快照（D5 约定 B，对齐 ZFS dsl_destroy 的形态）：
    /// 把被销毁者的 deadlist 并进下一个更新的那侧，然后对**合并结果**过滤——
    /// birth > prev(S).txg 的条目此刻无人引用（只被 S 和更老的引用过），释放。
    /// 只过滤被转移的那半会把「死在下个窗口、只被 S 引用」的块多钉一个窗口。
    fn destroy_oldest(&mut self) {
        let s = self.snaps.remove(0);
        let mut merged = if let Some(next) = self.snaps.first_mut() {
            std::mem::take(&mut next.deadlist)
        } else {
            std::mem::take(&mut self.head_deadlist)
        };
        merged.extend(s.deadlist);
        let mut kept = Vec::with_capacity(merged.len());
        for e in merged {
            if self.versions[e.ver as usize].birth > s.prev_txg {
                assert_eq!(self.occupant.remove(&e.slot), Some(e.ver));
                self.map_slot.remove(&e.ver);
                self.free.insert(e.slot);
            } else {
                kept.push(e);
            }
        }
        if let Some(next) = self.snaps.first_mut() {
            next.deadlist = kept;
            next.prev_txg = s.prev_txg;
        } else {
            self.head_deadlist = kept;
        }
    }

    /// 守恒：每个被任何保留结构（活树 / 快照树 / deadlist）引用的版本都还占着槽。
    /// 销毁级联把还被引用的槽放掉会在这里现形。
    fn assert_all_refs_occupied(&self) {
        let occ: BTreeSet<u32> = self.occupant.values().copied().collect();
        let roots: Vec<u32> =
            std::iter::once(self.live_root).chain(self.snaps.iter().map(|s| s.root)).collect();
        for root in roots {
            self.walk(root, &mut |e| {
                assert!(occ.contains(&e.ver), "树引用的 ver {} 已无槽", e.ver);
            });
        }
        for snap in &self.snaps {
            for e in &snap.deadlist {
                assert!(occ.contains(&e.ver), "deadlist 引用的 ver {} 已无槽", e.ver);
            }
        }
        for e in &self.head_deadlist {
            assert!(occ.contains(&e.ver), "活头 deadlist 引用的 ver {} 已无槽", e.ver);
        }
        // 反向：每个占着槽的版本都必须被活树、某棵快照树或某张 deadlist 引用——
        // 销毁级联把条目弄丢（槽泄漏）会在这里现形。
        let mut referenced: BTreeSet<u32> = BTreeSet::new();
        for root in
            std::iter::once(self.live_root).chain(self.snaps.iter().map(|s| s.root))
        {
            self.walk(root, &mut |e| {
                referenced.insert(e.ver);
            });
        }
        for snap in &self.snaps {
            referenced.extend(snap.deadlist.iter().map(|e| e.ver));
        }
        referenced.extend(self.head_deadlist.iter().map(|e| e.ver));
        for &ver in occ.iter() {
            assert!(referenced.contains(&ver), "占着槽的 ver {ver} 无人引用：槽泄漏");
        }
    }

    /// 读回一个根下的逻辑内容：key → ver。
    fn readback(&self, root: u32) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.l);
        self.walk(root, &mut |e| out.push(e.ver));
        out
    }

    fn walk(&self, node: u32, f: &mut impl FnMut(Entry)) {
        match &self.nodes[node as usize] {
            Node::Inner(cs) => {
                for &c in cs {
                    self.walk(c, f);
                }
            }
            Node::Leaf(es) => {
                for &e in es {
                    f(e);
                }
            }
        }
    }

    /// 树审计：每个版本被哪些快照的树引用（独立于区间判据的第二条路）。
    fn snap_refs(&self) -> BTreeMap<u32, BTreeSet<usize>> {
        let mut refs: BTreeMap<u32, BTreeSet<usize>> = BTreeMap::new();
        for (i, snap) in self.snaps.iter().enumerate() {
            self.walk(snap.root, &mut |e| {
                refs.entry(e.ver).or_default().insert(i);
            });
        }
        refs
    }

    /// 区间判据（D5 的引用条件）：birth ≤ S.txg < death。
    fn interval_refs(&self, ver: u32) -> BTreeSet<usize> {
        let v = self.versions[ver as usize];
        self.snaps
            .iter()
            .enumerate()
            .filter(|(_, s)| v.birth <= s.txg && s.txg < v.death)
            .map(|(i, _)| i)
            .collect()
    }

    /// 判据 5：线性历史下区间判据与树审计逐版本一致（模型自检）。
    fn assert_interval_matches_tree(&self) {
        let tree = self.snap_refs();
        for (&slot, &ver) in &self.occupant {
            let _ = slot;
            let by_tree = tree.get(&ver).cloned().unwrap_or_default();
            let by_interval = self.interval_refs(ver);
            assert_eq!(by_tree, by_interval, "ver={ver} 区间判据与树审计分叉");
        }
    }
}

/// 一次搬迁的触碰面。
#[derive(Default, Debug, PartialEq, Eq)]
struct Cost {
    cow_nodes: u64,     // 新写出的树节点数（ptr：引用树；central：映射树）
    root_refs: u64,     // 要重写的根引用数（活根记录 + 快照列表条目）
    dl_rewrites: u64,   // deadlist 条目重写数
    moved: u64,         // 实际搬动的版本数
    skipped_shared: u64, // skip_shared 臂放弃的版本数
}

/// 给每个要搬的版本分配新槽（取最低空槽；destinations 不影响计数结论）。
fn plan_moves(w: &World, vers: &[u32]) -> BTreeMap<u32, (u32, u32)> {
    // ver → (旧槽, 新槽)
    let cur: BTreeMap<u32, u32> = w.occupant.iter().map(|(&s, &v)| (v, s)).collect();
    let mut free = w.free.iter().copied();
    vers.iter().map(|&v| (v, (cur[&v], free.next().expect("槽用尽")))).collect()
}

/// ptr_rewrite：对每个根做记忆化重建波，返回新节点数与被改根数；同时改写 deadlist 与分配记录。
fn ptr_move(w: &mut World, plan: &BTreeMap<u32, (u32, u32)>) -> Cost {
    let mut memo: BTreeMap<u32, Option<u32>> = BTreeMap::new();
    let mut created = 0u64;
    let mut root_refs = 0u64;
    let roots: Vec<u32> =
        std::iter::once(w.live_root).chain(w.snaps.iter().map(|s| s.root)).collect();
    let mut new_roots = Vec::with_capacity(roots.len());
    for root in roots {
        let r = rebuild(w, root, plan, &mut memo, &mut created);
        if r.is_some() {
            root_refs += 1;
        }
        new_roots.push(r);
    }
    if let Some(nr) = new_roots[0] {
        w.live_root = nr;
    }
    for (i, nr) in new_roots[1..].iter().enumerate() {
        if let Some(nr) = nr {
            w.snaps[i].root = *nr;
        }
    }
    let mut dl = 0u64;
    for snap in &mut w.snaps {
        for e in &mut snap.deadlist {
            if let Some(&(_, new_slot)) = plan.get(&e.ver) {
                e.slot = new_slot;
                dl += 1;
            }
        }
    }
    for e in &mut w.head_deadlist {
        if let Some(&(_, new_slot)) = plan.get(&e.ver) {
            e.slot = new_slot;
            dl += 1;
        }
    }
    apply_alloc_records(w, plan);
    Cost { cow_nodes: created, root_refs, dl_rewrites: dl, moved: plan.len() as u64, skipped_shared: 0 }
}

/// 记忆化重建：返回 Some(新节点) 当且仅当子树里有被搬的版本。
fn rebuild(
    w: &mut World,
    node: u32,
    plan: &BTreeMap<u32, (u32, u32)>,
    memo: &mut BTreeMap<u32, Option<u32>>,
    created: &mut u64,
) -> Option<u32> {
    if let Some(&r) = memo.get(&node) {
        return r;
    }
    let result = match w.nodes[node as usize].clone() {
        Node::Leaf(mut es) => {
            let mut hit = false;
            for e in &mut es {
                if let Some(&(_, new_slot)) = plan.get(&e.ver) {
                    e.slot = new_slot;
                    hit = true;
                }
            }
            if hit {
                w.nodes.push(Node::Leaf(es));
                *created += 1;
                Some((w.nodes.len() - 1) as u32)
            } else {
                None
            }
        }
        Node::Inner(cs) => {
            let mut new_cs = cs.clone();
            let mut hit = false;
            for (i, &c) in cs.iter().enumerate() {
                if let Some(nc) = rebuild(w, c, plan, memo, created) {
                    new_cs[i] = nc;
                    hit = true;
                }
            }
            if hit {
                w.nodes.push(Node::Inner(new_cs));
                *created += 1;
                Some((w.nodes.len() - 1) as u32)
            } else {
                None
            }
        }
    };
    memo.insert(node, result);
    result
}

/// central_map：映射树（key 空间同几何、单根）的记忆化波；主树、deadlist、快照根一律不动。
fn central_move(w: &mut World, plan: &BTreeMap<u32, (u32, u32)>) -> Cost {
    // 触到的映射叶 = 被搬版本的 key 按 F 分组；逐层向上并到单根
    let mut level: BTreeSet<usize> =
        plan.keys().map(|&v| w.versions[v as usize].key as usize / w.f).collect();
    let mut created = level.len() as u64;
    let mut span = w.l / w.f;
    while span > 1 {
        span /= w.f;
        level = level.iter().map(|&i| i / w.f).collect();
        created += level.len() as u64;
    }
    for (&ver, &(_, new_slot)) in plan {
        w.map_slot.insert(ver, new_slot);
    }
    apply_alloc_records(w, plan);
    Cost {
        cow_nodes: created,
        root_refs: 1,
        dl_rewrites: 0,
        moved: plan.len() as u64,
        skipped_shared: 0,
    }
}

/// 分配记录与占用表跟着搬（两臂共用的那笔账，判据 6 单列）。
fn apply_alloc_records(w: &mut World, plan: &BTreeMap<u32, (u32, u32)>) {
    for (&ver, &(old_slot, new_slot)) in plan {
        assert_eq!(w.occupant.remove(&old_slot), Some(ver));
        w.free.insert(old_slot);
        let taken = w.free.remove(&new_slot);
        assert!(taken, "新槽必须来自空槽集");
        w.occupant.insert(new_slot, ver);
    }
}

/// 搬迁后的引用审计：位置权威侧的每一处引用都必须指到新槽，旧槽零引用。
/// 返回「还指着旧槽的引用数」。与搬迁实现不共享改写代码。
fn audit_stale_refs(w: &World, arch: Arch, plan: &BTreeMap<u32, (u32, u32)>) -> u64 {
    let old_slots: BTreeSet<u32> = plan.values().map(|&(o, _)| o).collect();
    let mut stale = 0u64;
    match arch {
        Arch::Ptr | Arch::SkipShared => {
            let roots: Vec<u32> =
                std::iter::once(w.live_root).chain(w.snaps.iter().map(|s| s.root)).collect();
            // 同一物理节点只审一次（结构共享下的去重——审计口径与盘上字节一致）
            let mut seen: BTreeSet<u32> = BTreeSet::new();
            for root in roots {
                audit_walk(w, root, &old_slots, &mut seen, &mut stale);
            }
            for snap in &w.snaps {
                for e in &snap.deadlist {
                    if old_slots.contains(&e.slot) {
                        stale += 1;
                    }
                }
            }
            for e in &w.head_deadlist {
                if old_slots.contains(&e.slot) {
                    stale += 1;
                }
            }
        }
        Arch::Central => {
            for (&_ver, &slot) in &w.map_slot {
                if old_slots.contains(&slot) {
                    stale += 1;
                }
            }
        }
    }
    stale
}

fn audit_walk(w: &World, node: u32, old: &BTreeSet<u32>, seen: &mut BTreeSet<u32>, stale: &mut u64) {
    if !seen.insert(node) {
        return;
    }
    match &w.nodes[node as usize] {
        Node::Inner(cs) => {
            for &c in cs {
                audit_walk(w, c, old, seen, stale);
            }
        }
        Node::Leaf(es) => {
            for e in es {
                if old.contains(&e.slot) {
                    *stale += 1;
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arch {
    Ptr,
    Central,
    SkipShared,
}
impl Arch {
    fn tag(self) -> &'static str {
        match self {
            Arch::Ptr => "ptr_rewrite",
            Arch::Central => "central_map",
            Arch::SkipShared => "skip_shared",
        }
    }
}

/// 造历史：K 个快照、快照间 u 次均匀随机 key 更新；
/// 最后一个快照之后再走 u 次更新——没有这一段，历史恰好停在快照边界上，
/// 所有活版本 birth ≤ 最新快照 txg ⇒ 全部被快照引用，skip_shared 的可搬集合恒空
/// （那是建模伪影，不是它的天花板）。
fn build_history(k: usize, u: usize, seed: u64) -> World {
    let mut w = World::new(F, L, S);
    let mut rng = Rng::new(seed);
    // k + 4 轮里滚动保留 k 个快照：不销毁的话盘底沦为「被死快照钉住的化石层」，
    // 那是「从不销毁」的伪影，不是保留窗为 k 的稳态。
    for _ in 0..k + 4 {
        for _ in 0..u {
            let key = rng.below(L);
            w.update(key);
        }
        w.snapshot();
        if w.snaps.len() > k {
            w.destroy_oldest();
        }
    }
    for _ in 0..u {
        let key = rng.below(L);
        w.update(key);
    }
    w.assert_all_refs_occupied();
    w.assert_interval_matches_tree();
    w
}

/// 批次：phys = 槽号最低的 N 个占用槽；keyclu = key 连续段的现行活版本。
fn pick_batch(w: &World, mode: &str) -> Vec<u32> {
    match mode {
        "phys" => w.occupant.values().take(N_BATCH).copied().collect(),
        "keyclu" => {
            let live = w.readback(w.live_root);
            (1024..1024 + N_BATCH).map(|k| live[k]).collect()
        }
        _ => unreachable!(),
    }
}

/// 批次里 key 的分散度：把批次版本的 key 排序后数极大连续段。
fn key_runs(w: &World, vers: &[u32]) -> u64 {
    let mut keys: Vec<u32> = vers.iter().map(|&v| w.versions[v as usize].key).collect();
    keys.sort_unstable();
    keys.dedup();
    let mut runs = 1u64;
    for i in 1..keys.len() {
        if keys[i] != keys[i - 1] + 1 {
            runs += 1;
        }
    }
    runs
}

/// 跑一个架构臂：clone 世界、搬、审计守恒。返回触碰面。
fn run_arch(base: &World, arch: Arch, batch: &[u32]) -> Cost {
    let mut w = base.clone();
    let before: Vec<Vec<u32>> = std::iter::once(w.live_root)
        .chain(w.snaps.iter().map(|s| s.root))
        .map(|r| w.readback(r))
        .collect();
    let occupied_before = w.occupant.len();

    let (vers, skipped): (Vec<u32>, u64) = match arch {
        Arch::SkipShared => {
            let shared = w.snap_refs();
            let movable: Vec<u32> =
                batch.iter().copied().filter(|v| !shared.contains_key(v)).collect();
            let skipped = (batch.len() - movable.len()) as u64;
            (movable, skipped)
        }
        _ => (batch.to_vec(), 0),
    };
    let slots_total = w.occupant.len() + w.free.len();
    let plan = plan_moves(&w, &vers);
    let mut cost = match arch {
        Arch::Ptr | Arch::SkipShared => ptr_move(&mut w, &plan),
        Arch::Central => central_move(&mut w, &plan),
    };
    cost.skipped_shared = skipped;
    assert_eq!(w.occupant.len() + w.free.len(), slots_total, "{arch:?} 槽守恒破了");

    // 判据 2：语义守恒 + 旧槽零引用 + 占用数恒等
    let after: Vec<Vec<u32>> = std::iter::once(w.live_root)
        .chain(w.snaps.iter().map(|s| s.root))
        .map(|r| w.readback(r))
        .collect();
    assert_eq!(before, after, "{arch:?} 搬迁改了逻辑内容");
    assert_eq!(w.occupant.len(), occupied_before, "{arch:?} 占用数变了");
    assert_eq!(audit_stale_refs(&w, arch, &plan), 0, "{arch:?} 旧槽还有引用");
    cost
}

/// 判据 3 阳性对照：漏改一处引用，审计必须抓到。
fn injection_control(base: &World, arch: Arch, batch: &[u32]) {
    let mut w = base.clone();
    let vers: Vec<u32> = match arch {
        Arch::SkipShared => {
            let shared = w.snap_refs();
            batch.iter().copied().filter(|v| !shared.contains_key(v)).collect()
        }
        _ => batch.to_vec(),
    };
    if vers.is_empty() {
        return;
    }
    let plan = plan_moves(&w, &vers);
    match arch {
        Arch::Ptr | Arch::SkipShared => {
            ptr_move(&mut w, &plan);
            // 漏改：把活根路径上第一个含被搬版本的叶改回旧槽
            let (&ver, &(old_slot, _)) = plan.iter().next().unwrap();
            let key = w.versions[ver as usize].key as usize;
            let ds = w.digits(key);
            let mut cur = w.live_root;
            for &d in &ds[..ds.len() - 1] {
                cur = match &w.nodes[cur as usize] {
                    Node::Inner(cs) => cs[d],
                    _ => unreachable!(),
                };
            }
            if let Node::Leaf(es) = &mut w.nodes[cur as usize] {
                es[*ds.last().unwrap()].slot = old_slot;
            }
        }
        Arch::Central => {
            central_move(&mut w, &plan);
            let (&ver, &(old_slot, _)) = plan.iter().next().unwrap();
            w.map_slot.insert(ver, old_slot);
        }
    }
    let stale = audit_stale_refs(&w, arch, &plan);
    assert!(stale > 0, "{arch:?} 注入的漏改没被审计抓到：整轮作废");
}

/// 判据 4 负臂：搬动时重指派 birth，违例数必须恰等于「被快照树引用且被搬动」的版本数。
fn new_birth_violations(base: &World, batch: &[u32]) -> (u64, u64) {
    let mut w = base.clone();
    let tree_refs = w.snap_refs();
    let expected: u64 = batch.iter().filter(|v| tree_refs.contains_key(v)).count() as u64;
    let plan = plan_moves(&w, &batch.to_vec());
    ptr_move(&mut w, &plan);
    let now = w.txg + 1;
    for &v in batch {
        w.versions[v as usize].birth = now;
    }
    // 违例：树审计说被引用、区间判据说没人引用
    let mut violations = 0u64;
    for &v in batch {
        let by_tree = tree_refs.get(&v).cloned().unwrap_or_default();
        let by_interval = w.interval_refs(v);
        if !by_tree.is_empty() && by_interval.is_empty() {
            violations += 1;
        }
    }
    (violations, expected)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config l={L} f={F} s={S} n_batch={N_BATCH} model=counting file_ops=0 seeds=5"
        ))
    );

    for k in [1usize, 4, 16] {
        for u in [64usize, 1024] {
            for mode in ["phys", "keyclu"] {
                for seed in 0..5u64 {
                    let w = build_history(k, u, seed);
                    let batch = pick_batch(&w, mode);
                    let kr = key_runs(&w, &batch);
                    for arch in [Arch::Ptr, Arch::Central, Arch::SkipShared] {
                        injection_control(&w, arch, &batch);
                        let c = run_arch(&w, arch, &batch);
                        println!(
                            "{}",
                            em.emit_raw(&format!(
                                "name=move arch={} k={k} u={u} batch={mode} seed={seed} key_runs={kr} moved={} cow_nodes={} root_refs={} dl_rewrites={} skipped={}",
                                arch.tag(),
                                c.moved,
                                c.cow_nodes,
                                c.root_refs,
                                c.dl_rewrites,
                                c.skipped_shared
                            ))
                        );
                    }
                    let (viol, expected) = new_birth_violations(&w, &batch);
                    assert_eq!(viol, expected, "违例数必须恰等于独立算出的破坏数");
                    assert!(
                        viol > 0 || expected == 0,
                        "k={k} u={u} {mode} seed={seed}：负臂零违例且预期非零"
                    );
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=new_birth k={k} u={u} batch={mode} seed={seed} violations={viol}"
                        ))
                    );
                }
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_world() -> World {
        World::new(2, 8, 32)
    }

    /// **判据 1 手算锚点（共享整树）**：L=8、F=2，快照后零更新 ⇒ 快照根 == 活根（同一节点）。
    /// 搬 key 3 的版本：唯一叶副本 + 1 个内部 + 1 个根 = 3 个新节点；
    /// 根引用更新 = 2（活根记录 + 快照列表条目指向同一个旧根，都要改指新根）。
    #[test]
    fn hand_shared_whole_tree() {
        let mut w = small_world();
        w.snapshot();
        assert_eq!(w.snaps[0].root, w.live_root);
        let plan = plan_moves(&w, &[3]);
        let c = ptr_move(&mut w, &plan);
        assert_eq!(c.cow_nodes, 3);
        assert_eq!(c.root_refs, 2);
        assert_eq!(c.dl_rewrites, 0);
        assert_eq!(audit_stale_refs(&w, Arch::Ptr, &plan), 0);
    }

    /// **判据 1 手算锚点（部分分叉）**：快照后更新 key 0 ⇒ 活树左半 COW。
    /// 搬 key 3：叶 {2,3} 仍是单副本，但它的父辈在两棵树里已分叉
    /// ⇒ 1 叶 + 2 个内部（快照侧 + 活侧）+ 2 个根 = 5 个新节点；根引用 2。
    #[test]
    fn hand_diverged_paths() {
        let mut w = small_world();
        w.snapshot();
        w.update(0);
        let plan = plan_moves(&w, &[3]);
        let c = ptr_move(&mut w, &plan);
        assert_eq!(c.cow_nodes, 5);
        assert_eq!(c.root_refs, 2);
        assert_eq!(audit_stale_refs(&w, Arch::Ptr, &plan), 0);
    }

    /// **判据 1 手算锚点（central_map）**：同场景搬 key 3：映射树 1 叶 + 1 内部 + 1 根 = 3 节点、
    /// 根引用 1、deadlist 0；主树一个节点都不写。
    #[test]
    fn hand_central() {
        let mut w = small_world();
        w.snapshot();
        w.update(0);
        let nodes_before = w.nodes.len();
        let plan = plan_moves(&w, &[3]);
        let c = central_move(&mut w, &plan);
        assert_eq!(c.cow_nodes, 3);
        assert_eq!(c.root_refs, 1);
        assert_eq!(c.dl_rewrites, 0);
        assert_eq!(w.nodes.len(), nodes_before, "主树一个节点都不许写");
        assert_eq!(audit_stale_refs(&w, Arch::Central, &plan), 0);
    }

    /// **deadlist 条目重写**：快照 1 之后更新 key 3（旧版本 birth 1 ≤ prev_snap ⇒ 进活头 deadlist），
    /// 快照 2 接管它。搬那个死版本：它只被快照 1 的树引用 ⇒ 波只走快照 1 一棵 + deadlist 改 1 条。
    #[test]
    fn dead_version_rewrites_deadlist() {
        let mut w = small_world();
        w.snapshot(); // S1 @ txg 1
        w.update(3); // 旧 ver 3（birth 1）死，进活头 deadlist
        w.snapshot(); // S2 接管 deadlist
        assert_eq!(w.snaps[1].deadlist.len(), 1);
        let dead_ver = w.snaps[1].deadlist[0].ver;
        assert_eq!(dead_ver, 3);
        let plan = plan_moves(&w, &[dead_ver]);
        let c = ptr_move(&mut w, &plan);
        // S1 的树引用它：1 叶 + 1 内部 + 1 根；活树与 S2 树不含它
        assert_eq!(c.cow_nodes, 3);
        assert_eq!(c.root_refs, 1);
        assert_eq!(c.dl_rewrites, 1);
        assert_eq!(audit_stale_refs(&w, Arch::Ptr, &plan), 0);
    }

    /// **判据 4 手算**：快照后搬 key 3 并重指派 birth ⇒ 恰 1 个违例（key 3 的版本被快照引用）。
    #[test]
    fn new_birth_breaks_exactly_one() {
        let mut w = small_world();
        w.snapshot();
        let (viol, expected) = new_birth_violations(&w, &[3]);
        assert_eq!(expected, 1);
        assert_eq!(viol, 1);
    }

    /// **判据 5**：随机历史下区间判据与树审计逐版本一致（build_history 内部断言），
    /// 且立即释放规则真的在放槽：快照前的更新对象不进 deadlist。
    #[test]
    fn interval_equals_tree_and_immediate_free() {
        let w = build_history(4, 64, 0);
        assert!(w.snaps.len() == 4);
        let mut w2 = World::new(2, 8, 32);
        w2.update(3); // 无快照 ⇒ birth 2 > prev_snap 0 ⇒ 立即释放旧槽
        assert!(w2.free.contains(&3), "旧槽 3 必须立即回空");
        assert!(w2.head_deadlist.is_empty());
    }

    /// skip_shared 只搬未被任何快照引用的：快照后更新 key 0，
    /// 批次 {新 ver(key0), 旧共享 ver(key3)} ⇒ 恰搬 1 个、放弃 1 个。
    #[test]
    fn skip_shared_moves_only_exclusive() {
        let mut w = small_world();
        w.snapshot();
        w.update(0);
        let live = w.readback(w.live_root);
        let batch = vec![live[0], live[3]];
        let c = run_arch(&w, Arch::SkipShared, &batch);
        assert_eq!(c.moved, 1);
        assert_eq!(c.skipped_shared, 1);
    }

    /// 注入的漏改必须被审计抓到（三臂）。
    #[test]
    fn injection_is_caught() {
        let w = build_history(2, 16, 1);
        let batch = pick_batch(&w, "phys");
        for arch in [Arch::Ptr, Arch::Central, Arch::SkipShared] {
            injection_control(&w, arch, &batch);
        }
    }

    /// 读回守恒经 run_arch 全流程（三臂 × 两种批次）。
    #[test]
    fn conservation_via_run_arch() {
        let w = build_history(4, 64, 2);
        for mode in ["phys", "keyclu"] {
            let batch = pick_batch(&w, mode);
            for arch in [Arch::Ptr, Arch::Central, Arch::SkipShared] {
                let c = run_arch(&w, arch, &batch);
                assert!(c.moved + c.skipped_shared == batch.len() as u64);
            }
        }
    }

    /// **销毁级联的手算锚点**：S1 之后更新 key 3（死版本进活头 deadlist）、S2 接管。
    /// 销毁 S1（prev = 0）：合并侧 = S2 的 deadlist，唯一条目 birth 1 > 0 ⇒ 释放——
    /// 旧槽 3 回空、ver 3 不再占槽、S2 的 deadlist 清空。
    /// 只过滤被转移那半的写法会把这条钉到 S2 销毁时才放，这里就红。
    #[test]
    fn destroy_frees_merged_next_side() {
        let mut w = small_world();
        w.snapshot(); // S1 @ txg 1
        w.update(3);
        w.snapshot(); // S2 接管 deadlist（1 条：ver 3 @ 槽 3）
        assert_eq!(w.snaps[1].deadlist, vec![DlEntry { ver: 3, slot: 3 }]);
        w.destroy_oldest();
        assert!(w.free.contains(&3), "旧槽 3 必须在 S1 销毁时回空");
        assert!(!w.occupant.contains_key(&3));
        assert!(w.snaps[0].deadlist.is_empty());
        w.assert_all_refs_occupied();
    }

    /// deadlist 审计的判别力：搬完后手工把一条 deadlist 槽位改回旧值，审计必须多出 stale。
    #[test]
    fn deadlist_audit_has_teeth() {
        let mut w = small_world();
        w.snapshot();
        w.update(3);
        w.snapshot();
        let dead_ver = w.snaps[1].deadlist[0].ver;
        let plan = plan_moves(&w, &[dead_ver]);
        let old_slot = plan[&dead_ver].0;
        ptr_move(&mut w, &plan);
        assert_eq!(audit_stale_refs(&w, Arch::Ptr, &plan), 0);
        w.snaps[1].deadlist[0].slot = old_slot;
        assert!(audit_stale_refs(&w, Arch::Ptr, &plan) > 0);
    }

    /// **等价性留档（变异表 M5 判等价的依据）**：只销毁最旧者的轮换下，
    /// 被销毁者自己的 deadlist 在销毁那一刻恒空——首个快照的从未装过东西
    /// （它窗口里的 kill 全走「birth > prev=0 ⇒ 立即释放」），其余的在前驱销毁时
    /// 已被「合并下一侧再过滤」清空。所以「把被销毁者的条目并进下一侧」在
    /// 只毁最旧者的序列里是不可达代码；动它的变异等价，不算盲区，分开记。
    #[test]
    fn oldest_first_rotation_means_own_deadlist_already_empty() {
        let mut w = World::new(2, 8, 64);
        let mut rng = Rng::new(9);
        for _ in 0..6 {
            for _ in 0..4 {
                let k = rng.below(8);
                w.update(k);
            }
            w.snapshot();
            if w.snaps.len() > 2 {
                assert!(
                    w.snaps[0].deadlist.is_empty(),
                    "最旧者的 deadlist 该已被前驱销毁清空"
                );
                w.destroy_oldest();
            }
        }
        w.assert_all_refs_occupied();
    }

    /// **销毁级联的 prev 传递**：毁 S1 后 S2 的 prev 必须变 0，否则毁 S2 时
    /// birth ≤ 旧 prev 的条目被永久钉住。
    #[test]
    fn destroy_cascade_propagates_prev() {
        let mut w = small_world();
        w.snapshot(); // S1 @ txg 1
        w.update(3);
        w.snapshot(); // S2 @ txg 2 接管 {ver3}
        w.update(0);
        w.snapshot(); // S3 @ txg 3 接管 {ver0}
        w.destroy_oldest(); // 毁 S1：ver3 释放
        assert!(w.free.contains(&3));
        w.destroy_oldest(); // 毁 S2：prev 已传递为 0 ⇒ ver0 释放
        assert!(w.free.contains(&0), "prev 不传递的话 ver0 会被钉住");
        w.assert_all_refs_occupied();
    }

    /// 批次形状：phys 取槽号最低的 64 个占用槽；keyclu 的 key 恰是一个连续段。
    #[test]
    fn batch_shapes() {
        let w = build_history(4, 64, 3);
        let phys = pick_batch(&w, "phys");
        assert_eq!(phys.len(), 64);
        let keyclu = pick_batch(&w, "keyclu");
        assert_eq!(key_runs(&w, &keyclu), 1);
    }
}
